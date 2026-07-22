"""Context query models and fluent builder.

``ContextQuery`` is the primary request payload sent to a CPP server
via the ``cpp/query`` JSON-RPC method.  ``ContextQueryBuilder``
provides a more ergonomic construction API for agent code.
"""

from __future__ import annotations

from datetime import timedelta
from typing import Any

from pydantic import BaseModel, ConfigDict, Field
from pydantic.alias_generators import to_camel

from .types import (
    AccessLevel,
    BudgetPreference,
    Certainty,
    ContextBudget,
    ContextType,
    FreshnessKind,
    Goal,
    ProviderId,
    RelationType,
    SessionId,
)

_CAMEL_CONFIG = ConfigDict(
    populate_by_name=True,
    alias_generator=to_camel,
    use_enum_values=True,
)


class RangeConstraint(BaseModel):
    """Numeric range filter (inclusive bounds)."""

    model_config = _CAMEL_CONFIG

    min: float | None = None
    max: float | None = None


class QueryScope(BaseModel):
    """Scoping constraints that limit which providers/URIs are queried."""

    model_config = _CAMEL_CONFIG

    current: bool = True
    recent: int | None = None  # seconds
    providers: list[str] = Field(default_factory=list)
    uri_patterns: list[str] = Field(default_factory=list)


class QueryConstraints(BaseModel):
    """Filtering constraints applied after provider resolution."""

    model_config = _CAMEL_CONFIG

    importance: RangeConstraint | None = None
    tags: list[str] = Field(default_factory=list)
    attributes: dict[str, Any] = Field(default_factory=dict)
    min_certainty: str | None = None
    freshness_kind: str | None = None


class ContextQuery(BaseModel):
    """The primary query payload for ``cpp/query``.

    Mirrors the Rust ``ContextQuery`` struct field-for-field with
    camelCase aliasing for the JSON-RPC wire format.
    """

    model_config = _CAMEL_CONFIG

    goal: Goal
    scope: QueryScope = Field(default_factory=QueryScope)
    include: list[str] = Field(default_factory=list)
    exclude: list[str] = Field(default_factory=list)
    constraints: QueryConstraints = Field(default_factory=QueryConstraints)
    depth: int = 0
    follow_relations: list[str] = Field(default_factory=list)
    max_results: int = 50
    offset: int = 0
    access_level: str = AccessLevel.READ.value
    budget: ContextBudget | None = None
    session_id: str | None = None
    hints: dict[str, Any] = Field(default_factory=dict)


# ---------------------------------------------------------------------------
# Fluent builder
# ---------------------------------------------------------------------------


class ContextQueryBuilder:
    """Ergonomic builder for ``ContextQuery``.

    Usage::

        query = (
            ContextQueryBuilder(Goal.code())
            .budget(max_bytes=4096)
            .scope_providers("git", "filesystem")
            .hint("workspacePath", "/my/project")
            .include_types(ContextType.file(), ContextType.commit())
            .max_results(10)
            .build()
        )
    """

    def __init__(self, goal: Goal) -> None:
        self._goal = goal
        self._scope = QueryScope()
        self._constraints = QueryConstraints()
        self._include: list[str] = []
        self._exclude: list[str] = []
        self._follow: list[str] = []
        self._depth: int = 0
        self._max_results: int = 50
        self._offset: int = 0
        self._access_level: str = AccessLevel.READ.value
        self._budget: ContextBudget | None = None
        self._session_id: str | None = None
        self._hints: dict[str, Any] = {}

    # Budget

    def budget(
        self,
        max_bytes: int | None = None,
        max_objects: int | None = None,
        max_latency_ms: int | None = None,
        prefer: BudgetPreference = BudgetPreference.QUALITY,
    ) -> ContextQueryBuilder:
        self._budget = ContextBudget(
            max_bytes=max_bytes,
            max_objects=max_objects,
            max_latency_ms=max_latency_ms,
            prefer=prefer,
        )
        return self

    # Scope

    def scope_providers(self, *providers: str) -> ContextQueryBuilder:
        self._scope.providers = list(providers)
        return self

    def scope_recent(self, duration: timedelta) -> ContextQueryBuilder:
        self._scope.recent = int(duration.total_seconds())
        return self

    def scope_uri_patterns(self, *patterns: str) -> ContextQueryBuilder:
        self._scope.uri_patterns = list(patterns)
        return self

    # Type filters

    def include_types(self, *types: ContextType) -> ContextQueryBuilder:
        self._include = [t.value for t in types]
        return self

    def exclude_types(self, *types: ContextType) -> ContextQueryBuilder:
        self._exclude = [t.value for t in types]
        return self

    # Constraints

    def min_importance(self, value: float) -> ContextQueryBuilder:
        self._constraints.importance = RangeConstraint(min=value)
        return self

    def min_certainty(self, certainty: Certainty) -> ContextQueryBuilder:
        self._constraints.min_certainty = certainty.value
        return self

    def freshness_kind(self, kind: FreshnessKind) -> ContextQueryBuilder:
        self._constraints.freshness_kind = kind.value
        return self

    def tags(self, *tags: str) -> ContextQueryBuilder:
        self._constraints.tags = list(tags)
        return self

    # Graph traversal

    def depth(self, depth: int) -> ContextQueryBuilder:
        self._depth = depth
        return self

    def follow_relations(self, *relations: RelationType) -> ContextQueryBuilder:
        self._follow = [r.value for r in relations]
        return self

    # Pagination

    def max_results(self, n: int) -> ContextQueryBuilder:
        self._max_results = n
        return self

    def offset(self, n: int) -> ContextQueryBuilder:
        self._offset = n
        return self

    # Misc

    def access_level(self, level: AccessLevel) -> ContextQueryBuilder:
        self._access_level = level.value
        return self

    def session(self, session_id: str | SessionId) -> ContextQueryBuilder:
        self._session_id = str(session_id)
        return self

    def hint(self, key: str, value: Any) -> ContextQueryBuilder:
        self._hints[key] = value
        return self

    # Terminal

    def build(self) -> ContextQuery:
        return ContextQuery(
            goal=self._goal,
            scope=self._scope,
            include=self._include,
            exclude=self._exclude,
            constraints=self._constraints,
            depth=self._depth,
            follow_relations=self._follow,
            max_results=self._max_results,
            offset=self._offset,
            access_level=self._access_level,
            budget=self._budget,
            session_id=self._session_id,
            hints=self._hints,
        )
