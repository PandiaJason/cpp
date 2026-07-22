from __future__ import annotations

from datetime import datetime, timedelta, timezone
import json

from cpp_sdk import (
    AccessLevel,
    BudgetPreference,
    Certainty,
    ContextBudget,
    ContextBundle,
    ContextObject,
    ContextObjectBuilder,
    ContextPermissions,
    ContextQuery,
    ContextQueryBuilder,
    ContextType,
    Freshness,
    FreshnessKind,
    Goal,
    Importance,
    LifecycleState,
    Reference,
    Relation,
    RelationType,
)
from cpp_sdk.protocol import JsonRpcError, JsonRpcRequest, JsonRpcResponse


def test_goal_serialization():
    goal = Goal.code()
    assert str(goal) == "goal.code"
    assert goal.intent == "goal.code"
    assert goal.description == "Code investigation and editing"


def test_context_type_factories():
    assert ContextType.file().value == "application/cpp.document.file"
    assert ContextType.repository().value == "application/cpp.repository"
    assert ContextType.commit().value == "application/cpp.commit"
    assert ContextType.branch().value == "application/cpp.branch"
    assert ContextType.pull_request().value == "application/cpp.pull_request"
    assert ContextType.issue().value == "application/cpp.issue"
    assert ContextType.message().value == "application/cpp.message"
    assert ContextType.channel().value == "application/cpp.channel"
    assert ContextType.sprint().value == "application/cpp.sprint"
    assert ContextType.epic().value == "application/cpp.epic"
    assert ContextType.datetime().value == "application/cpp.datetime"


def test_context_budget_serialization():
    budget = ContextBudget(
        max_bytes=4096,
        max_objects=10,
        prefer=BudgetPreference.QUALITY,
    )
    data = budget.model_dump(by_alias=True)
    assert "maxBytes" in data
    assert "maxObjects" in data
    assert "maxLatencyMs" in data
    assert "prefer" in data
    assert data["maxBytes"] == 4096
    assert data["maxObjects"] == 10
    assert data["maxLatencyMs"] is None
    assert data["prefer"] == "quality"


def test_freshness_factories():
    live = Freshness.live()
    assert live.kind == FreshnessKind.LIVE
    assert live.max_age_seconds is None

    immutable = Freshness.immutable()
    assert immutable.kind == FreshnessKind.IMMUTABLE

    cached = Freshness.cached(timedelta(hours=1))
    assert cached.kind == FreshnessKind.CACHED
    assert cached.max_age_seconds == 3600


def test_importance_factories():
    high = Importance.high()
    assert high.priority == 0.9

    medium = Importance.medium()
    assert medium.priority == 0.5

    low = Importance.low()
    assert low.priority == 0.2


def test_relation_serialization():
    relation = Relation(
        relation_type=RelationType.MODIFIES,
        target_uri="cpp://git/file/db.rs",
    )
    data = relation.model_dump(by_alias=True)
    assert "relationType" in data
    assert "targetUri" in data
    assert data["relationType"] == "modifies"
    assert data["targetUri"] == "cpp://git/file/db.rs"


def test_context_object_serialization():
    now = datetime.now(timezone.utc)
    obj = ContextObject(
        uri="cpp://git/commit/12345",
        id="obj-123",
        version=1,
        context_type="application/cpp.commit",
        provider_id="git",
        created_at=now,
        updated_at=now,
        expires_at=now,
        certainty=Certainty.AUTHORITATIVE.value,
        freshness=Freshness.live(),
        lifecycle=LifecycleState.CREATED.value,
        importance=Importance.high(),
        title="Commit 12345",
        summary="Test commit summary",
        content="Commit details",
        permissions=ContextPermissions(level=AccessLevel.READ),
        relations=[Relation(relation_type=RelationType.MODIFIES, target_uri="cpp://git/file/a.py")],
        references=[Reference.source("https://github.com/repo/commit/12345")],
        metadata={"author": "alice"},
        extensions={"custom": True},
    )
    data = obj.model_dump(by_alias=True)
    assert "contextType" in data
    assert "providerId" in data
    assert "createdAt" in data
    assert "updatedAt" in data
    assert "expiresAt" in data
    assert "certainty" in data
    assert "freshness" in data
    assert "lifecycle" in data
    assert "importance" in data
    assert "title" in data
    assert "summary" in data
    assert "content" in data
    assert "permissions" in data
    assert "relations" in data
    assert "references" in data
    assert "metadata" in data
    assert "extensions" in data
    assert "uri" in data
    assert "id" in data
    assert "version" in data


def test_context_object_builder():
    obj = (
        ContextObjectBuilder("cpp://git/commit/abc123", ContextType.commit(), "git")
        .title("Fix bug")
        .summary("Bug fix summary")
        .content("Full content")
        .certainty(Certainty.AUTHORITATIVE)
        .freshness(Freshness.live())
        .importance(Importance.high())
        .lifecycle(LifecycleState.CREATED)
        .permissions(AccessLevel.READ, ["repo:read"])
        .relation(RelationType.MODIFIES.value, "cpp://git/file/main.py")
        .reference("https://example.com/ref")
        .metadata_field("branch", "main")
        .extension("ext_key", "ext_val")
        .build()
    )
    assert isinstance(obj, ContextObject)
    assert obj.uri == "cpp://git/commit/abc123"
    assert obj.context_type == "application/cpp.commit"
    assert obj.provider_id == "git"
    assert obj.title == "Fix bug"
    assert obj.summary == "Bug fix summary"
    assert obj.certainty == "authoritative"
    assert obj.importance.priority == 0.9
    assert len(obj.relations) == 1
    assert obj.relations[0].relation_type == "modifies"
    assert obj.metadata["branch"] == "main"


def test_context_bundle_serialization():
    obj1 = ContextObjectBuilder("cpp://test/1", ContextType.file(), "test").build()
    obj2 = ContextObjectBuilder("cpp://test/2", ContextType.file(), "test").build()
    bundle = ContextBundle(
        objects=[obj1, obj2],
        total_count=2,
        providers=["test"],
        resolution_time_ms=45,
        from_cache=False,
    )
    data = bundle.model_dump(by_alias=True)
    assert "totalCount" in data
    assert "resolutionTimeMs" in data
    assert "fromCache" in data
    assert "objects" in data
    assert "providers" in data
    assert data["totalCount"] == 2
    assert data["resolutionTimeMs"] == 45
    assert len(data["objects"]) == 2


def test_context_query_serialization():
    query = ContextQuery(
        goal=Goal.code(),
        budget=ContextBudget(max_bytes=4096),
        follow_relations=[RelationType.MODIFIES.value],
        max_results=25,
        access_level=AccessLevel.READ.value,
        session_id="session-123",
        hints={"workspace": "/app"},
    )
    data = query.model_dump(by_alias=True)
    assert "followRelations" in data
    assert "maxResults" in data
    assert "accessLevel" in data
    assert "sessionId" in data
    assert data["followRelations"] == ["modifies"]
    assert data["maxResults"] == 25
    assert data["accessLevel"] == "read"
    assert data["sessionId"] == "session-123"


def test_query_builder():
    query = (
        ContextQueryBuilder(Goal.code())
        .budget(max_bytes=2048, max_objects=5, prefer=BudgetPreference.SPEED)
        .scope_providers("git", "filesystem")
        .scope_uri_patterns("cpp://git/*")
        .include_types(ContextType.file(), ContextType.commit())
        .exclude_types(ContextType.issue())
        .min_importance(0.8)
        .min_certainty(Certainty.AUTHORITATIVE)
        .freshness_kind(FreshnessKind.LIVE)
        .tags("urgent")
        .depth(2)
        .follow_relations(RelationType.MODIFIES)
        .max_results(15)
        .offset(5)
        .access_level(AccessLevel.READ)
        .session("sess-abc")
        .hint("env", "prod")
        .build()
    )
    assert isinstance(query, ContextQuery)
    assert str(query.goal) == "goal.code"
    assert query.budget is not None
    assert query.budget.max_bytes == 2048
    assert query.budget.prefer == BudgetPreference.SPEED
    assert query.scope.providers == ["git", "filesystem"]
    assert query.scope.uri_patterns == ["cpp://git/*"]
    assert query.include == ["application/cpp.document.file", "application/cpp.commit"]
    assert query.exclude == ["application/cpp.issue"]
    assert query.constraints.importance is not None
    assert query.constraints.importance.min == 0.8
    assert query.constraints.min_certainty == "authoritative"
    assert query.constraints.freshness_kind == "live"
    assert query.constraints.tags == ["urgent"]
    assert query.depth == 2
    assert query.follow_relations == ["modifies"]
    assert query.max_results == 15
    assert query.offset == 5
    assert query.access_level == "read"
    assert query.session_id == "sess-abc"
    assert query.hints == {"env": "prod"}


def test_jsonrpc_request_serialization():
    req = JsonRpcRequest(
        id=1,
        method="cpp/query",
        params={"query": {"goal": {"intent": "code"}}},
    )
    data = req.model_dump(by_alias=True)
    assert "jsonrpc" in data
    assert "id" in data
    assert "method" in data
    assert "params" in data
    assert data["jsonrpc"] == "2.0"
    assert data["id"] == 1
    assert data["method"] == "cpp/query"


def test_jsonrpc_response_with_error():
    err = JsonRpcError(code=-32600, message="Invalid Request", data={"details": "bad format"})
    resp = JsonRpcResponse(id=1, error=err)
    data = resp.model_dump(by_alias=True)
    assert "jsonrpc" in data
    assert "id" in data
    assert "error" in data
    assert data["error"]["code"] == -32600
    assert data["error"]["message"] == "Invalid Request"
    assert data["error"]["data"] == {"details": "bad format"}


def test_round_trip_context_object():
    obj = ContextObjectBuilder("cpp://git/commit/123", ContextType.commit(), "git").title("Round trip test").build()
    json_str = obj.model_dump_json(by_alias=True)
    parsed = ContextObject.model_validate_json(json_str)
    assert parsed.uri == obj.uri
    assert parsed.id == obj.id
    assert parsed.title == obj.title
    assert parsed.context_type == obj.context_type
    assert parsed.provider_id == obj.provider_id
    assert parsed == obj
