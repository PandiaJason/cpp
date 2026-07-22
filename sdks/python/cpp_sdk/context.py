"""Semantic Context Object (SCO) and ContextBundle models.

These are the primary data containers returned by CPP queries.
``ContextObjectBuilder`` provides a fluent API matching the Rust SDK
builder pattern.
"""

from __future__ import annotations

import uuid
from datetime import datetime, timezone
from typing import Any

from pydantic import BaseModel, ConfigDict, Field
from pydantic.alias_generators import to_camel

from .types import (
    AccessLevel,
    Certainty,
    ContextId,
    ContextPermissions,
    ContextType,
    ContextUri,
    Freshness,
    Importance,
    LifecycleState,
    ProviderId,
    Relation,
    Reference,
)

_CAMEL_CONFIG = ConfigDict(
    populate_by_name=True,
    alias_generator=to_camel,
    use_enum_values=True,
)


class ContextObject(BaseModel):
    """A single Semantic Context Object (SCO).

    This is the fundamental data unit of the CPP protocol — a semantically
    enriched node in the context graph carrying identity, temporal state,
    content, access control, relationships, and extensibility metadata.
    """

    model_config = _CAMEL_CONFIG

    # Identity
    uri: str
    id: str
    version: int = 1
    context_type: str
    provider_id: str

    # Temporal
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
    expires_at: datetime | None = None

    # Semantic properties
    certainty: str = Certainty.AUTHORITATIVE.value
    freshness: Freshness = Field(default_factory=Freshness.live)
    lifecycle: str = LifecycleState.CREATED.value
    importance: Importance = Field(default_factory=Importance.medium)

    # Content
    title: str = ""
    summary: str | None = None
    content: str | None = None

    # Access control
    permissions: ContextPermissions = Field(default_factory=ContextPermissions)

    # Graph
    relations: list[Relation] = Field(default_factory=list)
    references: list[Reference] = Field(default_factory=list)

    # Extension
    metadata: dict[str, Any] = Field(default_factory=dict)
    extensions: dict[str, Any] = Field(default_factory=dict)


class ContextBundle(BaseModel):
    """Response container returned by ``cpp/query``.

    Wraps a list of ``ContextObject`` instances alongside resolution
    metadata (timing, cache status, provider list).
    """

    model_config = _CAMEL_CONFIG

    objects: list[ContextObject] = Field(default_factory=list)
    total_count: int = 0
    providers: list[str] = Field(default_factory=list)
    resolution_time_ms: int = 0
    from_cache: bool = False
    metadata: dict[str, Any] = Field(default_factory=dict)


# ---------------------------------------------------------------------------
# Builder
# ---------------------------------------------------------------------------


class ContextObjectBuilder:
    """Fluent builder for constructing ``ContextObject`` instances.

    Usage::

        obj = (
            ContextObjectBuilder("cpp://git/commit/abc123", "application/cpp.commit", "git")
            .title("Fix timeout bug")
            .summary("Increased the database connection timeout from 5s to 30s")
            .certainty(Certainty.AUTHORITATIVE)
            .importance(Importance.high())
            .metadata("commitSha", "abc123")
            .relation(RelationType.MODIFIES, "cpp://fs/file/db.rs")
            .build()
        )
    """

    def __init__(
        self,
        uri: str | ContextUri,
        context_type: str | ContextType,
        provider_id: str | ProviderId,
    ) -> None:
        ct = context_type.value if isinstance(context_type, ContextType) else context_type
        pid = str(provider_id)
        self._data: dict[str, Any] = {
            "uri": str(uri),
            "id": str(uuid.uuid4()),
            "context_type": ct,
            "provider_id": pid,
            "version": 1,
            "relations": [],
            "references": [],
            "metadata": {},
            "extensions": {},
        }

    # Content setters

    def title(self, title: str) -> ContextObjectBuilder:
        self._data["title"] = title
        return self

    def summary(self, summary: str) -> ContextObjectBuilder:
        self._data["summary"] = summary
        return self

    def content(self, content: str) -> ContextObjectBuilder:
        self._data["content"] = content
        return self

    # Semantic property setters

    def certainty(self, certainty: Certainty) -> ContextObjectBuilder:
        self._data["certainty"] = certainty.value
        return self

    def freshness(self, freshness: Freshness) -> ContextObjectBuilder:
        self._data["freshness"] = freshness
        return self

    def importance(self, importance: Importance) -> ContextObjectBuilder:
        self._data["importance"] = importance
        return self

    def lifecycle(self, state: LifecycleState) -> ContextObjectBuilder:
        self._data["lifecycle"] = state.value
        return self

    # Access control

    def permissions(self, level: AccessLevel, scopes: list[str] | None = None) -> ContextObjectBuilder:
        self._data["permissions"] = ContextPermissions(level=level, scopes=scopes or [])
        return self

    # Graph edges

    def relation(self, relation_type: str, target_uri: str, label: str | None = None) -> ContextObjectBuilder:
        self._data["relations"].append(Relation(relation_type=relation_type, target_uri=target_uri, label=label))
        return self

    def reference(self, url: str, label: str | None = None) -> ContextObjectBuilder:
        self._data["references"].append(Reference.source(url, label))
        return self

    # Extension

    def metadata_field(self, key: str, value: Any) -> ContextObjectBuilder:
        self._data["metadata"][key] = value
        return self

    def extension(self, key: str, value: Any) -> ContextObjectBuilder:
        self._data["extensions"][key] = value
        return self

    # Terminal

    def build(self) -> ContextObject:
        """Construct and return the ``ContextObject``."""
        return ContextObject(**self._data)
