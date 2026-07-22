"""GitHub Context Provider for the Context Provider Protocol (CPP).

Provides access to GitHub pull requests, issues, and commits as
Semantic Context Objects (SCOs) over the CPP interface.
"""

from __future__ import annotations

import os
from typing import Any

import httpx

from cpp_sdk.provider import ContextProvider, ProviderManifest, ProviderCapabilities
from cpp_sdk.context import ContextBundle, ContextObject, ContextObjectBuilder
from cpp_sdk.query import ContextQuery
from cpp_sdk.types import ContextType, Goal, Certainty, Freshness, Importance, Reference, Relation, RelationType
from cpp_sdk.errors import CppProviderError


class GitHubProvider(ContextProvider):
    """Context Provider Protocol implementation for GitHub repositories.

    Fetches open pull requests, issues, and recent commits from GitHub's REST API v3
    and translates them into Semantic Context Objects (SCOs).
    """

    def __init__(self, owner: str, repo: str, token: str | None = None) -> None:
        """Initialize the GitHub context provider.

        Args:
            owner: The GitHub organization or account name.
            repo: The repository name.
            token: Optional Personal Access Token. Defaults to GITHUB_TOKEN environment variable.
        """
        self.owner = owner
        self.repo = repo
        self.token = token if token is not None else os.environ.get("GITHUB_TOKEN")
        self._client: httpx.AsyncClient | None = None

        self._manifest = ProviderManifest(
            id="github",
            name="GitHub Context Provider",
            description="GitHub context provider for CPP supporting PRs, issues, and commits",
            version="0.1.0",
            capabilities=ProviderCapabilities.basic(
                context_types=[
                    ContextType.pull_request(),
                    ContextType.issue(),
                    ContextType.commit(),
                ],
                goals=[
                    Goal.code(),
                    Goal.project(),
                ],
            ),
        )

    @property
    def manifest(self) -> ProviderManifest:
        """Return this provider's registration manifest."""
        return self._manifest

    async def start(self) -> None:
        """Lifecycle hook called when the provider starts. Creates the httpx client."""
        if self._client is None or self._client.is_closed:
            headers = {
                "Accept": "application/vnd.github+json",
                "User-Agent": "cpp-provider-github",
                "X-GitHub-Api-Version": "2022-11-28",
            }
            if self.token:
                headers["Authorization"] = f"Bearer {self.token}"
            self._client = httpx.AsyncClient(
                base_url="https://api.github.com",
                headers=headers,
                timeout=30.0,
            )

    async def stop(self) -> None:
        """Lifecycle hook called when the provider stops. Closes the httpx client."""
        if self._client is not None and not self._client.is_closed:
            await self._client.aclose()
            self._client = None

    def _get_client(self) -> httpx.AsyncClient:
        """Retrieve or initialize the httpx AsyncClient."""
        if self._client is None or self._client.is_closed:
            headers = {
                "Accept": "application/vnd.github+json",
                "User-Agent": "cpp-provider-github",
                "X-GitHub-Api-Version": "2022-11-28",
            }
            if self.token:
                headers["Authorization"] = f"Bearer {self.token}"
            self._client = httpx.AsyncClient(
                base_url="https://api.github.com",
                headers=headers,
                timeout=30.0,
            )
        return self._client

    async def query(self, query: ContextQuery) -> ContextBundle:
        """Query GitHub for context matching the specified goal and constraints.

        Args:
            query: The CPP ContextQuery request object.

        Returns:
            A ContextBundle containing the resolved ContextObjects.

        Raises:
            CppProviderError: If the GitHub API request fails or returns an error.
        """
        client = self._get_client()
        objects: list[ContextObject] = []
        budget = query.budget

        goal_intent = (
            query.goal.intent
            if isinstance(query.goal, Goal)
            else (query.goal.get("intent") if isinstance(query.goal, dict) else str(query.goal))
        )

        try:
            if goal_intent == "code":
                # Fetch open PRs
                prs_resp = await client.get(
                    f"/repos/{self.owner}/{self.repo}/pulls",
                    params={"state": "open", "per_page": 10},
                )
                if prs_resp.status_code == 200:
                    for pr in prs_resp.json():
                        if self._should_stop_budget(objects, budget):
                            break
                        sco = self._build_pr_object(pr)
                        if self._would_exceed_bytes(objects, sco, budget):
                            break
                        objects.append(sco)
                elif prs_resp.status_code != 404:
                    raise CppProviderError(
                        f"Failed to fetch pull requests from GitHub (HTTP {prs_resp.status_code})"
                    )

                # Fetch recent commits
                commits_resp = await client.get(
                    f"/repos/{self.owner}/{self.repo}/commits",
                    params={"per_page": 10},
                )
                if commits_resp.status_code == 200:
                    for commit in commits_resp.json():
                        if self._should_stop_budget(objects, budget):
                            break
                        sco = self._build_commit_object(commit)
                        if self._would_exceed_bytes(objects, sco, budget):
                            break
                        objects.append(sco)
                elif commits_resp.status_code != 404:
                    raise CppProviderError(
                        f"Failed to fetch commits from GitHub (HTTP {commits_resp.status_code})"
                    )

            elif goal_intent == "project":
                # Fetch open issues (excluding pull requests returned by GitHub's issues endpoint)
                issues_resp = await client.get(
                    f"/repos/{self.owner}/{self.repo}/issues",
                    params={"state": "open", "per_page": 10},
                )
                if issues_resp.status_code == 200:
                    for item in issues_resp.json():
                        if "pull_request" in item:
                            continue
                        if self._should_stop_budget(objects, budget):
                            break
                        sco = self._build_issue_object(item)
                        if self._would_exceed_bytes(objects, sco, budget):
                            break
                        objects.append(sco)
                elif issues_resp.status_code != 404:
                    raise CppProviderError(
                        f"Failed to fetch issues from GitHub (HTTP {issues_resp.status_code})"
                    )

        except httpx.HTTPError as err:
            raise CppProviderError(f"GitHub API HTTP error: {err}") from err

        return ContextBundle(
            objects=objects,
            total_count=len(objects),
            providers=["github"],
        )

    async def resolve(self, uri: str) -> ContextObject:
        """Resolve a single GitHub SCO by its CPP URI.

        Supported URI schemes:
          - cpp://github/pull_request/{number}
          - cpp://github/issue/{number}
          - cpp://github/commit/{sha}

        Args:
            uri: The CPP URI to resolve.

        Returns:
            The resolved ContextObject with detailed content.

        Raises:
            CppProviderError: If the URI is invalid or entity is not found.
        """
        prefix = "cpp://github/"
        if not uri.startswith(prefix):
            raise CppProviderError(f"Invalid URI scheme for GitHub provider: '{uri}'")

        path = uri[len(prefix) :]
        parts = path.split("/", 1)
        if len(parts) != 2 or not parts[0] or not parts[1]:
            raise CppProviderError(f"Invalid GitHub CPP URI format: '{uri}'")

        entity_type, entity_id = parts[0], parts[1]
        client = self._get_client()

        try:
            if entity_type == "pull_request":
                response = await client.get(f"/repos/{self.owner}/{self.repo}/pulls/{entity_id}")
                if response.status_code == 404:
                    raise CppProviderError(f"Pull request not found: #{entity_id}")
                if response.status_code != 200:
                    raise CppProviderError(
                        f"GitHub API error fetching PR #{entity_id} (HTTP {response.status_code})"
                    )
                pr_data = response.json()
                body = pr_data.get("body") or ""
                sco = (
                    self._build_pr_object(pr_data)
                    .model_copy(update={"content": body})
                )
                return sco

            elif entity_type == "issue":
                response = await client.get(f"/repos/{self.owner}/{self.repo}/issues/{entity_id}")
                if response.status_code == 404:
                    raise CppProviderError(f"Issue not found: #{entity_id}")
                if response.status_code != 200:
                    raise CppProviderError(
                        f"GitHub API error fetching issue #{entity_id} (HTTP {response.status_code})"
                    )
                issue_data = response.json()
                body = issue_data.get("body") or ""
                sco = (
                    self._build_issue_object(issue_data)
                    .model_copy(update={"content": body})
                )
                return sco

            elif entity_type == "commit":
                response = await client.get(f"/repos/{self.owner}/{self.repo}/commits/{entity_id}")
                if response.status_code == 404:
                    raise CppProviderError(f"Commit not found: {entity_id}")
                if response.status_code != 200:
                    raise CppProviderError(
                        f"GitHub API error fetching commit {entity_id} (HTTP {response.status_code})"
                    )
                commit_data = response.json()
                commit_info = commit_data.get("commit", {})
                full_message = commit_info.get("message", "") if isinstance(commit_info, dict) else ""
                sco = (
                    self._build_commit_object(commit_data)
                    .model_copy(update={"content": full_message})
                )
                return sco

            else:
                raise CppProviderError(f"Unsupported entity type in URI: '{entity_type}'")

        except httpx.HTTPError as err:
            raise CppProviderError(f"GitHub API request failed during resolve: {err}") from err

    def _build_pr_object(self, pr: dict[str, Any]) -> ContextObject:
        """Convert a GitHub pull request dict into a ContextObject.

        Args:
            pr: The pull request response dict from GitHub API.

        Returns:
            Constructed ContextObject representing the pull request.
        """
        number = pr.get("number")
        title = pr.get("title", "")
        body = pr.get("body") or ""
        summary = body[:200]
        state = pr.get("state", "open")
        author = pr.get("user", {}).get("login", "") if pr.get("user") else ""
        url = pr.get("html_url", "")
        labels = [
            lbl.get("name")
            for lbl in pr.get("labels", [])
            if isinstance(lbl, dict) and lbl.get("name")
        ]

        builder = (
            ContextObjectBuilder(
                f"cpp://github/pull_request/{number}",
                ContextType.pull_request(),
                "github",
            )
            .title(title)
            .summary(summary)
            .metadata_field("number", number)
            .metadata_field("state", state)
            .metadata_field("author", author)
            .metadata_field("url", url)
            .metadata_field("labels", labels)
        )

        if url:
            builder.reference(url, label="GitHub Pull Request")

        head = pr.get("head")
        if isinstance(head, dict) and head.get("sha"):
            head_sha = head.get("sha")
            builder.relation(RelationType.MODIFIES.value, f"cpp://github/commit/{head_sha}")

        return builder.build()

    def _build_issue_object(self, issue: dict[str, Any]) -> ContextObject:
        """Convert a GitHub issue dict into a ContextObject.

        Args:
            issue: The issue response dict from GitHub API.

        Returns:
            Constructed ContextObject representing the issue.
        """
        number = issue.get("number")
        title = issue.get("title", "")
        body = issue.get("body") or ""
        summary = body[:200]
        state = issue.get("state", "open")
        author = issue.get("user", {}).get("login", "") if issue.get("user") else ""
        url = issue.get("html_url", "")
        labels = [
            lbl.get("name")
            for lbl in issue.get("labels", [])
            if isinstance(lbl, dict) and lbl.get("name")
        ]
        assignees = [
            ass.get("login")
            for ass in issue.get("assignees", [])
            if isinstance(ass, dict) and ass.get("login")
        ]

        builder = (
            ContextObjectBuilder(
                f"cpp://github/issue/{number}",
                ContextType.issue(),
                "github",
            )
            .title(title)
            .summary(summary)
            .metadata_field("number", number)
            .metadata_field("state", state)
            .metadata_field("author", author)
            .metadata_field("labels", labels)
            .metadata_field("assignees", assignees)
        )

        if url:
            builder.reference(url, label="GitHub Issue")

        return builder.build()

    def _build_commit_object(self, commit: dict[str, Any]) -> ContextObject:
        """Convert a GitHub commit dict into a ContextObject.

        Args:
            commit: The commit response dict from GitHub API.

        Returns:
            Constructed ContextObject representing the commit.
        """
        sha = commit.get("sha", "")
        commit_info = commit.get("commit", {}) if isinstance(commit.get("commit"), dict) else {}
        message = commit_info.get("message", "")
        first_line = message.split("\n")[0] if message else ""
        author_info = commit_info.get("author", {}) if isinstance(commit_info.get("author"), dict) else {}
        author_name = author_info.get("name", "")
        commit_date = author_info.get("date", "")
        url = commit.get("html_url", "")

        builder = (
            ContextObjectBuilder(
                f"cpp://github/commit/{sha}",
                ContextType.commit(),
                "github",
            )
            .title(first_line)
            .summary(message)
            .certainty(Certainty.AUTHORITATIVE)
            .freshness(Freshness.immutable())
            .metadata_field("sha", sha)
            .metadata_field("author", author_name)
            .metadata_field("date", commit_date)
        )

        if url:
            builder.reference(url, label="GitHub Commit")

        return builder.build()

    @staticmethod
    def _should_stop_budget(objects: list[ContextObject], budget: Any) -> bool:
        """Check if max_objects limit in the budget has been reached."""
        if not budget:
            return False
        max_objects = getattr(budget, "max_objects", None)
        return max_objects is not None and len(objects) >= max_objects

    @staticmethod
    def _would_exceed_bytes(objects: list[ContextObject], new_obj: ContextObject, budget: Any) -> bool:
        """Check if adding new_obj would exceed max_bytes limit in the budget."""
        if not budget:
            return False
        max_bytes = getattr(budget, "max_bytes", None)
        if max_bytes is None:
            return False
        current_bytes = sum(len(obj.model_dump_json().encode("utf-8")) for obj in objects)
        new_bytes = len(new_obj.model_dump_json().encode("utf-8"))
        return (current_bytes + new_bytes) > max_bytes
