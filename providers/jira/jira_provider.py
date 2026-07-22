"""Jira Cloud context provider for the Context Provider Protocol (CPP).

Provides integration with Jira REST API v3 to fetch issues, sprints, and epics
and convert them into Semantic Context Objects (SCOs).
"""

from __future__ import annotations

import os
import re
from typing import Any

import httpx

from cpp_sdk.context import ContextBundle, ContextObject, ContextObjectBuilder
from cpp_sdk.errors import CppProviderError
from cpp_sdk.provider import ContextProvider, ProviderCapabilities, ProviderManifest
from cpp_sdk.query import ContextQuery
from cpp_sdk.types import Certainty, ContextType, Freshness, Goal, Importance, RelationType


def _extract_text_from_adf(node: Any) -> str:
    """Recursively extract plain text from Atlassian Document Format (ADF)."""
    if isinstance(node, str):
        return node
    if not isinstance(node, dict):
        return ""

    text_parts = []
    if node.get("type") == "text":
        text_parts.append(node.get("text", ""))

    for child in node.get("content", []):
        text_parts.append(_extract_text_from_adf(child))

    return " ".join(filter(None, text_parts))


def _clean_description_text(raw_desc: str | dict | None) -> str:
    """Clean and extract plain text from a Jira description (HTML or ADF)."""
    if not raw_desc:
        return ""

    if isinstance(raw_desc, dict):
        text = _extract_text_from_adf(raw_desc)
    else:
        text = str(raw_desc)

    # Strip HTML tags
    clean = re.sub(r"<[^>]+>", "", text)
    # Normalize whitespace
    return re.sub(r"\s+", " ", clean).strip()


class JiraProvider(ContextProvider):
    """Jira Cloud Context Provider implementation.

    Attributes:
        base_url: Base URL of the Jira Cloud instance (e.g. 'https://your-domain.atlassian.net')
        email: Jira account email for Basic Auth
        api_token: Jira API token for Basic Auth
        project_key: Key of the default Jira project (e.g. 'PROJ')
    """

    def __init__(
        self,
        base_url: str | None = None,
        email: str | None = None,
        api_token: str | None = None,
        project_key: str | None = None,
    ) -> None:
        self.base_url = (base_url or os.getenv("JIRA_BASE_URL") or "").rstrip("/")
        self.email = email or os.getenv("JIRA_EMAIL") or ""
        self.api_token = api_token or os.getenv("JIRA_API_TOKEN") or ""
        self.project_key = project_key or os.getenv("JIRA_PROJECT_KEY") or ""

        self._client: httpx.AsyncClient | None = None
        self._manifest = ProviderManifest(
            id="jira",
            name="Jira Cloud Context Provider",
            description="Jira Cloud context provider for the Context Provider Protocol (CPP)",
            capabilities=ProviderCapabilities.basic(
                context_types=[ContextType.issue(), ContextType.sprint(), ContextType.epic()],
                goals=[Goal.project(), Goal.calendar()],
            ),
        )

    @property
    def manifest(self) -> ProviderManifest:
        """Return this provider's manifest declaring its capabilities."""
        return self._manifest

    async def start(self) -> None:
        """Lifecycle hook called when provider is registered. Initializes httpx AsyncClient."""
        if not self.base_url:
            raise CppProviderError("Jira base_url is required. Set JIRA_BASE_URL environment variable or pass base_url.")
        if not self.email or not self.api_token:
            raise CppProviderError("Jira email and api_token are required. Set JIRA_EMAIL and JIRA_API_TOKEN environment variables.")

        if self._client is None or self._client.is_closed:
            self._client = httpx.AsyncClient(
                base_url=self.base_url,
                auth=httpx.BasicAuth(self.email, self.api_token),
                headers={
                    "Accept": "application/json",
                    "Content-Type": "application/json",
                },
                timeout=30.0,
            )

    async def stop(self) -> None:
        """Lifecycle hook called when server shuts down. Closes httpx AsyncClient."""
        if self._client is not None and not self._client.is_closed:
            await self._client.aclose()
            self._client = None

    def _get_client(self) -> httpx.AsyncClient:
        """Return the active HTTP client or raise error if provider is not started."""
        if self._client is None or self._client.is_closed:
            raise CppProviderError("JiraProvider is not started. Call start() before querying.")
        return self._client

    async def query(self, query: ContextQuery) -> ContextBundle:
        """Query Jira Cloud for issues matching project and sprint scope.

        Calls Jira REST API v3 search endpoint with JQL.
        """
        client = self._get_client()
        max_results = min(query.max_results, 15) if query.max_results else 15

        jql_primary = (
            f"project={self.project_key} AND sprint in openSprints() ORDER BY updated DESC"
            if self.project_key
            else "sprint in openSprints() ORDER BY updated DESC"
        )

        try:
            response = await client.get("/rest/api/3/search", params={"jql": jql_primary, "maxResults": max_results})
            if response.status_code != 200:
                # Fallback to project query if openSprints JQL is unsupported or empty
                jql_fallback = f"project={self.project_key} ORDER BY updated DESC" if self.project_key else "ORDER BY updated DESC"
                response = await client.get("/rest/api/3/search", params={"jql": jql_fallback, "maxResults": max_results})
                if response.status_code != 200:
                    raise CppProviderError(f"Jira API search failed with status {response.status_code}: {response.text}")
        except httpx.HTTPError as exc:
            raise CppProviderError(f"HTTP request to Jira API failed: {exc}") from exc

        data = response.json()
        issues = data.get("issues", [])

        objects: list[ContextObject] = []
        for issue in issues:
            objects.append(self._convert_issue_to_context_object(issue, full_content=False))

        return ContextBundle(
            objects=objects,
            total_count=len(objects),
            providers=["jira"],
        )

    async def resolve(self, uri: str) -> ContextObject:
        """Resolve a single Jira issue by its CPP URI (e.g. 'cpp://jira/issue/PROJ-123')."""
        client = self._get_client()
        issue_key = uri.rstrip("/").split("/")[-1]

        if not issue_key or issue_key == "issue":
            raise CppProviderError(f"Invalid Jira URI format: '{uri}'. Expected 'cpp://jira/issue/{{issue_key}}'")

        try:
            response = await client.get(f"/rest/api/3/issue/{issue_key}")
            if response.status_code != 200:
                raise CppProviderError(f"Failed to resolve Jira issue '{issue_key}' (HTTP {response.status_code}): {response.text}")
        except httpx.HTTPError as exc:
            raise CppProviderError(f"HTTP request to Jira API failed for issue '{issue_key}': {exc}") from exc

        issue = response.json()
        return self._convert_issue_to_context_object(issue, full_content=True)

    def _convert_issue_to_context_object(self, issue: dict[str, Any], full_content: bool = False) -> ContextObject:
        """Convert a Jira issue payload into a CPP ContextObject (SCO)."""
        key = issue.get("key", "")
        fields = issue.get("fields", {})

        summary_text = fields.get("summary", "")
        raw_description = fields.get("description")
        cleaned_description = _clean_description_text(raw_description)

        status_obj = fields.get("status") or {}
        status_name = status_obj.get("name") if isinstance(status_obj, dict) else None

        assignee_obj = fields.get("assignee") or {}
        assignee_name = assignee_obj.get("displayName") if isinstance(assignee_obj, dict) else None

        priority_obj = fields.get("priority") or {}
        priority_name = priority_obj.get("name") if isinstance(priority_obj, dict) else None

        issuetype_obj = fields.get("issuetype") or {}
        issuetype_name = issuetype_obj.get("name") if isinstance(issuetype_obj, dict) else None

        labels = fields.get("labels", [])

        uri = f"cpp://jira/issue/{key}"
        builder = (
            ContextObjectBuilder(uri, ContextType.issue(), "jira")
            .title(summary_text)
            .summary(cleaned_description[:200] if cleaned_description else "")
            .certainty(Certainty.AUTHORITATIVE)
            .importance(Importance.medium())
            .metadata_field("key", key)
            .metadata_field("status", status_name)
            .metadata_field("assignee", assignee_name)
            .metadata_field("priority", priority_name)
            .metadata_field("issue_type", issuetype_name)
            .metadata_field("labels", labels)
        )

        if full_content:
            builder.content(cleaned_description)

        # Parse issue links for graph relations (BLOCKS, IS_BLOCKED_BY, RELATES_TO)
        issue_links = fields.get("issuelinks", [])
        for link in issue_links:
            link_type = link.get("type", {})
            type_name = str(link_type.get("name", "")).lower()
            inward_name = str(link_type.get("inward", "")).lower()
            outward_name = str(link_type.get("outward", "")).lower()

            if "outwardIssue" in link:
                target_key = link["outwardIssue"].get("key", "")
                if target_key:
                    target_uri = f"cpp://jira/issue/{target_key}"
                    if "block" in type_name or "block" in outward_name:
                        rel_type = RelationType.BLOCKS.value
                    else:
                        rel_type = RelationType.RELATES_TO.value
                    builder.relation(rel_type, target_uri, label=outward_name or link_type.get("name"))

            if "inwardIssue" in link:
                target_key = link["inwardIssue"].get("key", "")
                if target_key:
                    target_uri = f"cpp://jira/issue/{target_key}"
                    if "block" in type_name or "block" in inward_name:
                        rel_type = RelationType.IS_BLOCKED_BY.value
                    else:
                        rel_type = RelationType.RELATES_TO.value
                    builder.relation(rel_type, target_uri, label=inward_name or link_type.get("name"))

        return builder.build()
