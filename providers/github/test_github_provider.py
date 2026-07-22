"""Unit tests for GitHubProvider using unittest.IsolatedAsyncioTestCase."""

from __future__ import annotations

import unittest
from unittest.mock import AsyncMock, MagicMock, patch

from cpp_sdk.context import ContextBundle, ContextObject
from cpp_sdk.query import ContextQueryBuilder
from cpp_sdk.types import Goal, Certainty, FreshnessKind, RelationType
from cpp_sdk.errors import CppProviderError

from github_provider import GitHubProvider


def make_mock_client():
    mock_client = AsyncMock()
    mock_client.is_closed = False
    return mock_client


class TestGitHubProvider(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.provider = GitHubProvider(owner="octocat", repo="Hello-World", token="fake-token")

    def test_provider_manifest(self):
        manifest = self.provider.manifest
        self.assertEqual(manifest.id, "github")
        self.assertEqual(manifest.name, "GitHub Context Provider")
        self.assertIn("application/cpp.pull_request", manifest.capabilities.context_types)
        self.assertIn("application/cpp.issue", manifest.capabilities.context_types)
        self.assertIn("application/cpp.commit", manifest.capabilities.context_types)
        self.assertIn("code", manifest.capabilities.goals)
        self.assertIn("project", manifest.capabilities.goals)

    async def test_lifecycle_start_stop(self):
        await self.provider.start()
        self.assertIsNotNone(self.provider._client)
        self.assertFalse(self.provider._client.is_closed)

        await self.provider.stop()
        self.assertIsNone(self.provider._client)

    async def test_query_code_goal(self):
        mock_client = make_mock_client()

        # Mock PR response
        mock_prs_resp = MagicMock()
        mock_prs_resp.status_code = 200
        mock_prs_resp.json.return_value = [
            {
                "number": 42,
                "title": "Fix bug",
                "body": "Detailed PR body text that explains the changes made.",
                "state": "open",
                "user": {"login": "octocat"},
                "html_url": "https://github.com/octocat/Hello-World/pull/42",
                "labels": [{"name": "bug"}],
                "head": {"sha": "abc123commit"},
            }
        ]

        # Mock Commits response
        mock_commits_resp = MagicMock()
        mock_commits_resp.status_code = 200
        mock_commits_resp.json.return_value = [
            {
                "sha": "abc123commit",
                "commit": {
                    "message": "Fix bug\n\nDetailed commit message.",
                    "author": {"name": "Mona Lisa", "date": "2026-07-22T10:00:00Z"},
                },
                "html_url": "https://github.com/octocat/Hello-World/commit/abc123commit",
            }
        ]

        def get_side_effect(url, params=None):
            if "pulls" in url:
                return mock_prs_resp
            elif "commits" in url:
                return mock_commits_resp
            return MagicMock(status_code=404)

        mock_client.get.side_effect = get_side_effect
        self.provider._client = mock_client

        query = ContextQueryBuilder(Goal.code()).build()
        bundle = await self.provider.query(query)

        self.assertIsInstance(bundle, ContextBundle)
        self.assertEqual(len(bundle.objects), 2)

        # Verify PR SCO
        pr_obj = bundle.objects[0]
        self.assertEqual(pr_obj.uri, "cpp://github/pull_request/42")
        self.assertEqual(pr_obj.context_type, "application/cpp.pull_request")
        self.assertEqual(pr_obj.title, "Fix bug")
        self.assertTrue(pr_obj.summary.startswith("Detailed PR body"))
        self.assertEqual(pr_obj.metadata["number"], 42)
        self.assertEqual(pr_obj.metadata["author"], "octocat")
        self.assertTrue(
            any(rel.target_uri == "cpp://github/commit/abc123commit" for rel in pr_obj.relations)
        )

        # Verify Commit SCO
        commit_obj = bundle.objects[1]
        self.assertEqual(commit_obj.uri, "cpp://github/commit/abc123commit")
        self.assertEqual(commit_obj.context_type, "application/cpp.commit")
        self.assertEqual(commit_obj.title, "Fix bug")
        self.assertEqual(commit_obj.certainty, Certainty.AUTHORITATIVE.value)
        self.assertEqual(commit_obj.freshness.kind, FreshnessKind.IMMUTABLE)

    async def test_query_project_goal(self):
        mock_client = make_mock_client()

        mock_issues_resp = MagicMock()
        mock_issues_resp.status_code = 200
        mock_issues_resp.json.return_value = [
            {
                "number": 10,
                "title": "Feature request",
                "body": "Issue body text",
                "state": "open",
                "user": {"login": "alice"},
                "html_url": "https://github.com/octocat/Hello-World/issues/10",
                "labels": [{"name": "enhancement"}],
                "assignees": [{"login": "bob"}],
            },
            {
                "number": 11,
                "title": "PR in issues list",
                "pull_request": {"url": "..."},  # should be filtered out
            },
        ]

        mock_client.get.return_value = mock_issues_resp
        self.provider._client = mock_client

        query = ContextQueryBuilder(Goal.project()).build()
        bundle = await self.provider.query(query)

        self.assertEqual(len(bundle.objects), 1)
        issue_obj = bundle.objects[0]
        self.assertEqual(issue_obj.uri, "cpp://github/issue/10")
        self.assertEqual(issue_obj.context_type, "application/cpp.issue")
        self.assertEqual(issue_obj.title, "Feature request")
        self.assertEqual(issue_obj.metadata["assignees"], ["bob"])

    async def test_query_budget_limit(self):
        mock_client = make_mock_client()

        mock_prs_resp = MagicMock()
        mock_prs_resp.status_code = 200
        mock_prs_resp.json.return_value = [
            {"number": 1, "title": "PR 1", "body": "Body 1"},
            {"number": 2, "title": "PR 2", "body": "Body 2"},
        ]

        mock_client.get.return_value = mock_prs_resp
        self.provider._client = mock_client

        # Budget max_objects = 1
        query = ContextQueryBuilder(Goal.code()).budget(max_objects=1).build()
        bundle = await self.provider.query(query)

        self.assertEqual(len(bundle.objects), 1)
        self.assertEqual(bundle.objects[0].metadata["number"], 1)

    async def test_resolve_pull_request(self):
        mock_client = make_mock_client()

        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = {
            "number": 42,
            "title": "Fix bug",
            "body": "Full body content of PR #42",
            "state": "open",
            "user": {"login": "octocat"},
            "html_url": "https://github.com/octocat/Hello-World/pull/42",
            "labels": [],
            "head": {"sha": "headsha123"},
        }

        mock_client.get.return_value = mock_resp
        self.provider._client = mock_client

        obj = await self.provider.resolve("cpp://github/pull_request/42")
        self.assertEqual(obj.uri, "cpp://github/pull_request/42")
        self.assertEqual(obj.content, "Full body content of PR #42")
        self.assertEqual(obj.title, "Fix bug")
        mock_client.get.assert_called_with("/repos/octocat/Hello-World/pulls/42")

    async def test_resolve_invalid_uri(self):
        with self.assertRaises(CppProviderError) as ctx:
            await self.provider.resolve("cpp://git/commit/123")
        self.assertIn("Invalid URI scheme", str(ctx.exception))

        with self.assertRaises(CppProviderError) as ctx:
            await self.provider.resolve("cpp://github/invalid")
        self.assertIn("Invalid GitHub CPP URI format", str(ctx.exception))

    async def test_resolve_not_found(self):
        mock_client = make_mock_client()
        mock_resp = MagicMock()
        mock_resp.status_code = 404
        mock_client.get.return_value = mock_resp
        self.provider._client = mock_client

        with self.assertRaises(CppProviderError) as ctx:
            await self.provider.resolve("cpp://github/pull_request/999")
        self.assertIn("Pull request not found", str(ctx.exception))


if __name__ == "__main__":
    unittest.main()
