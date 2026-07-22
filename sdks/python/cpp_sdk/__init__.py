"""CPP Python SDK — Context Provider Protocol client library.

This package provides Pydantic v2 models, an async HTTP/WebSocket
client, and abstract provider base classes for building AI-agent
context integrations using the Context Provider Protocol.

Quick start::

    from cpp_sdk import CppClient, Goal

    async with CppClient("http://localhost:3030") as client:
        bundle = await client.query(Goal.code(), budget_max_bytes=4096)
        for obj in bundle.objects:
            print(obj.title, obj.uri)
"""

from __future__ import annotations

# Core types
from .types import (
    AccessLevel,
    BudgetPreference,
    Certainty,
    ContextBudget,
    ContextEvent,
    ContextId,
    ContextPermissions,
    ContextType,
    ContextUri,
    EventKind,
    Freshness,
    FreshnessKind,
    Goal,
    Importance,
    LifecycleState,
    ProviderId,
    Reference,
    Relation,
    RelationType,
    SessionId,
    SubscriptionId,
)

# Context objects
from .context import (
    ContextBundle,
    ContextObject,
    ContextObjectBuilder,
)

# Query
from .query import (
    ContextQuery,
    ContextQueryBuilder,
    QueryConstraints,
    QueryScope,
    RangeConstraint,
)

# Protocol
from .protocol import (
    CPP_CAPABILITIES,
    CPP_EVENT,
    CPP_INITIALIZE,
    CPP_INITIALIZED,
    CPP_QUERY,
    CPP_RESOLVE,
    CPP_SUBSCRIBE,
    CPP_UNSUBSCRIBE,
    JsonRpcError,
    JsonRpcNotification,
    JsonRpcRequest,
    JsonRpcResponse,
)

# Client
from .client import CppClient

# Provider
from .provider import (
    ContextProvider,
    ProviderAdapter,
    ProviderCapabilities,
    ProviderManifest,
)

# Errors
from .errors import (
    CppAuthenticationError,
    CppConnectionError,
    CppError,
    CppProtocolError,
    CppProviderError,
    CppTimeoutError,
)

__version__ = "0.1.0"

__all__ = [
    # Types
    "AccessLevel",
    "BudgetPreference",
    "Certainty",
    "ContextBudget",
    "ContextEvent",
    "ContextId",
    "ContextPermissions",
    "ContextType",
    "ContextUri",
    "EventKind",
    "Freshness",
    "FreshnessKind",
    "Goal",
    "Importance",
    "LifecycleState",
    "ProviderId",
    "Reference",
    "Relation",
    "RelationType",
    "SessionId",
    "SubscriptionId",
    # Context
    "ContextBundle",
    "ContextObject",
    "ContextObjectBuilder",
    # Query
    "ContextQuery",
    "ContextQueryBuilder",
    "QueryConstraints",
    "QueryScope",
    "RangeConstraint",
    # Protocol
    "CPP_CAPABILITIES",
    "CPP_EVENT",
    "CPP_INITIALIZE",
    "CPP_INITIALIZED",
    "CPP_QUERY",
    "CPP_RESOLVE",
    "CPP_SUBSCRIBE",
    "CPP_UNSUBSCRIBE",
    "JsonRpcError",
    "JsonRpcNotification",
    "JsonRpcRequest",
    "JsonRpcResponse",
    # Client
    "CppClient",
    # Provider
    "ContextProvider",
    "ProviderAdapter",
    "ProviderCapabilities",
    "ProviderManifest",
    # Errors
    "CppAuthenticationError",
    "CppConnectionError",
    "CppError",
    "CppProtocolError",
    "CppProviderError",
    "CppTimeoutError",
]
