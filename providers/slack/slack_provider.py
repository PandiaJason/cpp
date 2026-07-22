"""Slack context provider for the Context Provider Protocol (CPP).

Provides integration with the Slack Web API to fetch messages and channels
and convert them into Semantic Context Objects (SCOs).
"""

from __future__ import annotations

import os
from typing import Any

import httpx

from cpp_sdk.context import ContextBundle, ContextObject, ContextObjectBuilder
from cpp_sdk.errors import CppProviderError
from cpp_sdk.provider import ContextProvider, ProviderCapabilities, ProviderManifest
from cpp_sdk.query import ContextQuery
from cpp_sdk.types import Certainty, ContextType, Freshness, Goal, Importance


class SlackProvider(ContextProvider):
    """Slack Context Provider implementation.

    Attributes:
        token: Slack bot user OAuth token (xoxb-...)
        channels: List of Slack channel IDs to monitor
        base_url: Base URL for Slack Web API (default: 'https://slack.com/api')
    """

    def __init__(
        self,
        token: str | None = None,
        channels: list[str] | None = None,
    ) -> None:
        self.token = token or os.getenv("SLACK_BOT_TOKEN") or ""
        env_channels = os.getenv("SLACK_CHANNELS", "")
        if channels is not None:
            self.channels = channels
        elif env_channels:
            self.channels = [c.strip() for c in env_channels.split(",") if c.strip()]
        else:
            self.channels = []

        self.base_url = "https://slack.com/api"
        self._client: httpx.AsyncClient | None = None
        self._manifest = ProviderManifest(
            id="slack",
            name="Slack Context Provider",
            description="Slack context provider for the Context Provider Protocol (CPP)",
            capabilities=ProviderCapabilities.basic(
                context_types=[ContextType.message(), ContextType.channel()],
                goals=[Goal.document(), Goal.project()],
            ),
        )

    @property
    def manifest(self) -> ProviderManifest:
        """Return this provider's manifest declaring its capabilities."""
        return self._manifest

    async def start(self) -> None:
        """Lifecycle hook called when provider is registered. Initializes httpx AsyncClient."""
        if not self.token:
            raise CppProviderError("Slack bot token is required. Set SLACK_BOT_TOKEN environment variable or pass token.")

        if self._client is None or self._client.is_closed:
            self._client = httpx.AsyncClient(
                base_url=self.base_url,
                headers={
                    "Authorization": f"Bearer {self.token}",
                },
                timeout=30.0,
            )

    async def stop(self) -> None:
        """Lifecycle hook called when server shuts down. Closes httpx AsyncClient."""
        if self._client is not None and not self._client.is_closed:
            await self._client.aclose()
            self._client = None

    def _get_client(self) -> httpx.AsyncClient:
        """Return active HTTP client or raise error if provider is not started."""
        if self._client is None or self._client.is_closed:
            raise CppProviderError("SlackProvider is not started. Call start() before querying.")
        return self._client

    async def query(self, query: ContextQuery) -> ContextBundle:
        """Query Slack workspace for channel metadata and recent channel history messages."""
        client = self._get_client()
        objects: list[ContextObject] = []

        for channel_id in self.channels:
            # Emit Channel SCO
            channel_uri = f"cpp://slack/channel/{channel_id}"
            channel_obj = (
                ContextObjectBuilder(channel_uri, ContextType.channel(), "slack")
                .title(f"Slack Channel #{channel_id}")
                .summary(f"Slack channel context for {channel_id}")
                .metadata_field("channel_id", channel_id)
                .importance(Importance.medium())
                .certainty(Certainty.AUTHORITATIVE)
                .build()
            )
            objects.append(channel_obj)

            # Query channel message history
            try:
                response = await client.get("/conversations.history", params={"channel": channel_id, "limit": 20})
                if response.status_code != 200:
                    raise CppProviderError(f"Slack API error for channel '{channel_id}' (HTTP {response.status_code}): {response.text}")
                data = response.json()
                if not data.get("ok"):
                    raise CppProviderError(f"Slack API error for channel '{channel_id}': {data.get('error', 'unknown_error')}")
            except httpx.HTTPError as exc:
                raise CppProviderError(f"HTTP request to Slack failed for channel '{channel_id}': {exc}") from exc

            messages = data.get("messages", [])
            for msg in messages:
                text = msg.get("text", "")
                ts = msg.get("ts", "")
                if not ts:
                    continue

                reactions = msg.get("reactions", [])
                reaction_count = sum(int(r.get("count", 0)) for r in reactions)
                # Compute higher priority for messages with more reactions
                priority = min(1.0, 0.5 + (reaction_count * 0.1))

                title_text = text[:80].replace("\n", " ").strip() if text else f"Message {ts}"
                msg_uri = f"cpp://slack/message/{channel_id}/{ts}"

                builder = (
                    ContextObjectBuilder(msg_uri, ContextType.message(), "slack")
                    .title(title_text)
                    .content(text)
                    .summary(text[:200] if len(text) > 200 else text)
                    .certainty(Certainty.AUTHORITATIVE)
                    .importance(Importance(priority=priority))
                    .metadata_field("channel_id", channel_id)
                    .metadata_field("user", msg.get("user", ""))
                    .metadata_field("ts", ts)
                    .metadata_field("reaction_count", reaction_count)
                )

                if "thread_ts" in msg:
                    builder.metadata_field("thread_ts", msg["thread_ts"])

                objects.append(builder.build())

        return ContextBundle(
            objects=objects,
            total_count=len(objects),
            providers=["slack"],
        )

    async def resolve(self, uri: str) -> ContextObject:
        """Resolve a single Slack message or channel by its CPP URI.

        Supported URI formats:
        - Message: 'cpp://slack/message/{channel_id}/{ts}'
        - Channel: 'cpp://slack/channel/{channel_id}'
        """
        client = self._get_client()
        clean_path = uri.replace("cpp://slack/", "").strip("/")
        parts = clean_path.split("/")

        if len(parts) >= 2 and parts[0] == "channel":
            channel_id = parts[1]
            return (
                ContextObjectBuilder(uri, ContextType.channel(), "slack")
                .title(f"Slack Channel #{channel_id}")
                .summary(f"Slack channel context for {channel_id}")
                .metadata_field("channel_id", channel_id)
                .importance(Importance.medium())
                .certainty(Certainty.AUTHORITATIVE)
                .build()
            )

        if len(parts) >= 3 and parts[0] == "message":
            channel_id = parts[1]
            ts = parts[2]

            try:
                response = await client.get(
                    "/conversations.history",
                    params={"channel": channel_id, "latest": ts, "inclusive": "true", "limit": 1},
                )
                if response.status_code != 200:
                    raise CppProviderError(f"Slack API error resolving message (HTTP {response.status_code}): {response.text}")
                data = response.json()
                if not data.get("ok"):
                    raise CppProviderError(f"Slack API error resolving message: {data.get('error', 'unknown_error')}")
            except httpx.HTTPError as exc:
                raise CppProviderError(f"HTTP request to Slack failed resolving message: {exc}") from exc

            messages = data.get("messages", [])
            if not messages:
                raise CppProviderError(f"Slack message not found for ts '{ts}' in channel '{channel_id}'")

            msg = messages[0]
            text = msg.get("text", "")
            thread_ts = msg.get("thread_ts")
            reactions = msg.get("reactions", [])
            reaction_count = sum(int(r.get("count", 0)) for r in reactions)

            full_content = text
            replies_data: list[dict[str, Any]] = []

            # If message is part of a thread, fetch full thread replies
            if thread_ts:
                try:
                    rep_resp = await client.get("/conversations.replies", params={"channel": channel_id, "ts": thread_ts})
                    if rep_resp.status_code == 200:
                        rep_json = rep_resp.json()
                        if rep_json.get("ok"):
                            all_replies = rep_json.get("messages", [])
                            # Exclude parent message if present as first element
                            replies_data = all_replies[1:] if len(all_replies) > 1 else []
                except httpx.HTTPError:
                    pass  # Non-fatal if replies call fails

            if replies_data:
                reply_texts = [f"--- Thread Reply by {r.get('user', 'unknown')} ---\n{r.get('text', '')}" for r in replies_data]
                full_content = f"{text}\n\n" + "\n\n".join(reply_texts)

            priority = min(1.0, 0.5 + (reaction_count * 0.1))
            title_text = text[:80].replace("\n", " ").strip() if text else f"Message {ts}"

            builder = (
                ContextObjectBuilder(uri, ContextType.message(), "slack")
                .title(title_text)
                .content(full_content)
                .summary(text[:200] if len(text) > 200 else text)
                .certainty(Certainty.AUTHORITATIVE)
                .importance(Importance(priority=priority))
                .metadata_field("channel_id", channel_id)
                .metadata_field("user", msg.get("user", ""))
                .metadata_field("ts", ts)
                .metadata_field("reaction_count", reaction_count)
            )

            if thread_ts:
                builder.metadata_field("thread_ts", thread_ts)

            return builder.build()

        raise CppProviderError(f"Invalid Slack URI format: '{uri}'. Expected 'cpp://slack/message/channel/ts' or 'cpp://slack/channel/channel'")
