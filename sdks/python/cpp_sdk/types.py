"""Core CPP type definitions.

Every model uses ``alias_generator = to_camel`` so that
``model.model_dump(by_alias=True)`` produces the exact camelCase
JSON expected by the CPP wire protocol (JSON-RPC 2.0).
"""

from __future__ import annotations

from datetime import datetime, timedelta
from enum import Enum
from typing import Any, NewType

from pydantic import BaseModel, ConfigDict, Field, model_validator
from pydantic.alias_generators import to_camel

# ---------------------------------------------------------------------------
# Opaque identifiers
# ---------------------------------------------------------------------------

ContextUri = NewType("ContextUri", str)
"""A CPP URI of the form ``cpp://<provider>/<type>/<path>``."""

ContextId = NewType("ContextId", str)
"""UUID-style unique identifier for a single SCO."""

ProviderId = NewType("ProviderId", str)
"""Unique provider slug, e.g. ``"git"``, ``"github"``, ``"jira"``."""

SessionId = NewType("SessionId", str)
"""Session identifier bound to a sequence of queries."""

SubscriptionId = NewType("SubscriptionId", str)
"""Identifier for a live event subscription."""


# ---------------------------------------------------------------------------
# Enumerations
# ---------------------------------------------------------------------------


class AccessLevel(str, Enum):
    """Hierarchical access tier (ordinal 0-4)."""

    NONE = "none"
    METADATA = "metadata"
    READ = "read"
    WRITE = "write"
    ADMIN = "admin"


class Certainty(str, Enum):
    """How confident the provider is about an SCO's accuracy."""

    AUTHORITATIVE = "authoritative"
    DERIVED = "derived"
    ESTIMATED = "estimated"


class FreshnessKind(str, Enum):
    """Temporal classification of an SCO."""

    LIVE = "live"
    CACHED = "cached"
    IMMUTABLE = "immutable"


class LifecycleState(str, Enum):
    """Where an SCO sits in its lifecycle."""

    CREATED = "created"
    UPDATED = "updated"
    MERGED = "merged"
    ARCHIVED = "archived"
    EXPIRED = "expired"
    DELETED = "deleted"


class BudgetPreference(str, Enum):
    """Optimisation strategy when the budget is tight."""

    QUALITY = "quality"
    SPEED = "speed"
    COVERAGE = "coverage"


class RelationType(str, Enum):
    """Named edge type in the SCO relationship graph."""

    MODIFIES = "modifies"
    REFERENCES = "references"
    IMPORTS = "imports"
    CONTAINS = "contains"
    DEPENDS_ON = "depends_on"
    BLOCKS = "blocks"
    IS_BLOCKED_BY = "is_blocked_by"
    RELATES_TO = "relates_to"
    CREATED_BY = "created_by"
    ASSIGNED_TO = "assigned_to"


class EventKind(str, Enum):
    """Classification of a CPP context event."""

    CREATED = "created"
    UPDATED = "updated"
    DELETED = "deleted"
    EXPIRED = "expired"


# ---------------------------------------------------------------------------
# Shared value-objects (camelCase wire format)
# ---------------------------------------------------------------------------

_CAMEL_CONFIG = ConfigDict(
    populate_by_name=True,
    alias_generator=to_camel,
    use_enum_values=True,
)


class ContextType(BaseModel):
    """Fully-qualified SCO type string, e.g. ``application/cpp.document.file``."""

    model_config = _CAMEL_CONFIG

    value: str

    # Convenience factories matching the Rust helpers
    @classmethod
    def file(cls) -> ContextType:
        return cls(value="application/cpp.document.file")

    @classmethod
    def repository(cls) -> ContextType:
        return cls(value="application/cpp.repository")

    @classmethod
    def commit(cls) -> ContextType:
        return cls(value="application/cpp.commit")

    @classmethod
    def branch(cls) -> ContextType:
        return cls(value="application/cpp.branch")

    @classmethod
    def pull_request(cls) -> ContextType:
        return cls(value="application/cpp.pull_request")

    @classmethod
    def issue(cls) -> ContextType:
        return cls(value="application/cpp.issue")

    @classmethod
    def message(cls) -> ContextType:
        return cls(value="application/cpp.message")

    @classmethod
    def channel(cls) -> ContextType:
        return cls(value="application/cpp.channel")

    @classmethod
    def sprint(cls) -> ContextType:
        return cls(value="application/cpp.sprint")

    @classmethod
    def epic(cls) -> ContextType:
        return cls(value="application/cpp.epic")

    @classmethod
    def datetime(cls) -> ContextType:
        return cls(value="application/cpp.datetime")

    def __str__(self) -> str:
        return self.value

    def __eq__(self, other: object) -> bool:
        if isinstance(other, ContextType):
            return self.value == other.value
        return NotImplemented

    def __hash__(self) -> int:
        return hash(self.value)


class Goal(BaseModel):
    """Intent tag that providers use to filter relevant SCOs."""

    model_config = _CAMEL_CONFIG

    intent: str
    description: str = ""

    def __str__(self) -> str:
        if self.intent.startswith("goal."):
            return self.intent
        return f"goal.{self.intent}"

    @property
    def value(self) -> str:
        return str(self)

    def model_dump(self, *args: Any, **kwargs: Any) -> Any:
        return str(self)

    @classmethod
    def code(cls) -> Goal:
        return cls(intent="goal.code", description="Code investigation and editing")

    @classmethod
    def project(cls) -> Goal:
        return cls(intent="goal.project", description="Project and repository metadata")

    @classmethod
    def document(cls) -> Goal:
        return cls(intent="goal.document", description="Documentation and text files")

    @classmethod
    def calendar(cls) -> Goal:
        return cls(intent="goal.calendar", description="Temporal and scheduling context")


class Freshness(BaseModel):
    """Temporal freshness metadata of an SCO."""

    model_config = _CAMEL_CONFIG

    kind: FreshnessKind = FreshnessKind.LIVE
    max_age_seconds: int | None = None
    cached_at: datetime | None = None

    @classmethod
    def live(cls) -> Freshness:
        return cls(kind=FreshnessKind.LIVE)

    @classmethod
    def cached(cls, max_age: timedelta) -> Freshness:
        return cls(kind=FreshnessKind.CACHED, max_age_seconds=int(max_age.total_seconds()))

    @classmethod
    def immutable(cls) -> Freshness:
        return cls(kind=FreshnessKind.IMMUTABLE)


class Importance(BaseModel):
    """Priority weighting (0.0 to 1.0) for budget ranking."""

    model_config = _CAMEL_CONFIG

    priority: float = Field(default=0.5, ge=0.0, le=1.0)

    @model_validator(mode="before")
    @classmethod
    def validate_importance(cls, data: Any) -> Any:
        if isinstance(data, (int, float)):
            return {"priority": float(data)}
        return data

    @classmethod
    def high(cls) -> Importance:
        return cls(priority=0.9)

    @classmethod
    def medium(cls) -> Importance:
        return cls(priority=0.5)

    @classmethod
    def low(cls) -> Importance:
        return cls(priority=0.2)


class Relation(BaseModel):
    """Typed directed edge between two SCOs."""

    model_config = _CAMEL_CONFIG

    relation_type: RelationType
    target_uri: str
    label: str | None = None


class Reference(BaseModel):
    """External source reference attached to an SCO."""

    model_config = _CAMEL_CONFIG

    url: str = ""
    uri: str | None = None
    label: str | None = None
    ref_type: str = "source"

    @model_validator(mode="before")
    @classmethod
    def validate_reference(cls, data: Any) -> Any:
        if isinstance(data, dict):
            if not data.get("url") and data.get("uri"):
                data["url"] = data["uri"]
            if "type" in data and "ref_type" not in data and "refType" not in data:
                data["ref_type"] = data["type"]
        return data

    @classmethod
    def source(cls, url: str, label: str | None = None) -> Reference:
        return cls(url=url, label=label, ref_type="source")


class ContextPermissions(BaseModel):
    """Access control metadata for an SCO."""

    model_config = _CAMEL_CONFIG

    level: AccessLevel = AccessLevel.READ
    scopes: list[str] = Field(default_factory=list)


class ContextBudget(BaseModel):
    """Token / byte budget constraints for a query."""

    model_config = _CAMEL_CONFIG

    max_bytes: int | None = None
    max_objects: int | None = None
    max_latency_ms: int | None = None
    prefer: BudgetPreference = BudgetPreference.QUALITY


class ContextEvent(BaseModel):
    """A real-time context change event."""

    model_config = _CAMEL_CONFIG

    kind: EventKind
    uri: str
    provider_id: str
    timestamp: datetime
    data: dict[str, Any] = Field(default_factory=dict)
