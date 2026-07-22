"""JSON-RPC 2.0 message types and CPP method constants.

This module defines the wire-level protocol objects used for
communication between CPP clients and servers.  All param/result
types are included so that ``CppClient`` can serialize and
deserialize without importing from multiple modules.
"""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict, Field
from pydantic.alias_generators import to_camel

from .context import ContextBundle, ContextObject
from .query import ContextQuery
from .types import (
    AccessLevel,
    ContextEvent,
    Goal,
    SubscriptionId,
)

_CAMEL_CONFIG = ConfigDict(
    populate_by_name=True,
    alias_generator=to_camel,
    use_enum_values=True,
)

# ---------------------------------------------------------------------------
# JSON-RPC 2.0 base messages
# ---------------------------------------------------------------------------

JSONRPC_VERSION = "2.0"


class JsonRpcRequest(BaseModel):
    """A JSON-RPC 2.0 request."""

    model_config = _CAMEL_CONFIG

    jsonrpc: str = JSONRPC_VERSION
    id: int | str
    method: str
    params: dict[str, Any] | None = None


class JsonRpcError(BaseModel):
    """A JSON-RPC 2.0 error object."""

    model_config = _CAMEL_CONFIG

    code: int
    message: str
    data: Any | None = None


class JsonRpcResponse(BaseModel):
    """A JSON-RPC 2.0 response (success or error)."""

    model_config = _CAMEL_CONFIG

    jsonrpc: str = JSONRPC_VERSION
    id: int | str | None = None
    result: Any | None = None
    error: JsonRpcError | None = None


class JsonRpcNotification(BaseModel):
    """A JSON-RPC 2.0 notification (no ``id``)."""

    model_config = _CAMEL_CONFIG

    jsonrpc: str = JSONRPC_VERSION
    method: str
    params: dict[str, Any] | None = None


# ---------------------------------------------------------------------------
# CPP method constants
# ---------------------------------------------------------------------------

CPP_INITIALIZE = "cpp/initialize"
CPP_INITIALIZED = "cpp/initialized"
CPP_QUERY = "cpp/query"
CPP_RESOLVE = "cpp/resolve"
CPP_CAPABILITIES = "cpp/capabilities"
CPP_SUBSCRIBE = "cpp/subscribe"
CPP_UNSUBSCRIBE = "cpp/unsubscribe"
CPP_EVENT = "cpp/event"
CPP_PUBLISH = "cpp/publish"
CPP_SHUTDOWN = "cpp/shutdown"
CPP_EXIT = "cpp/exit"


# ---------------------------------------------------------------------------
# Method-specific param / result types
# ---------------------------------------------------------------------------


class ClientInfo(BaseModel):
    model_config = _CAMEL_CONFIG
    name: str
    version: str


class ClientCapabilities(BaseModel):
    model_config = _CAMEL_CONFIG
    streaming: bool = False
    subscriptions: bool = False


class InitializeParams(BaseModel):
    model_config = _CAMEL_CONFIG
    protocol_version: str = "0.1.0"
    client_info: ClientInfo = Field(default_factory=lambda: ClientInfo(name="cpp-python-sdk", version="0.1.0"))
    capabilities: ClientCapabilities = Field(default_factory=ClientCapabilities)


class ProviderManifestInfo(BaseModel):
    """Minimal provider manifest returned by the server."""

    model_config = _CAMEL_CONFIG
    id: str
    name: str
    context_types: list[str] = Field(default_factory=list)
    goals: list[str] = Field(default_factory=list)


class ServerCapabilities(BaseModel):
    model_config = _CAMEL_CONFIG
    streaming: bool = False
    subscriptions: bool = False
    budget_negotiation: bool = True


class RuntimeInfo(BaseModel):
    model_config = _CAMEL_CONFIG
    name: str = ""
    version: str = ""


class InitializeResult(BaseModel):
    model_config = _CAMEL_CONFIG
    protocol_version: str = "0.1.0"
    runtime_info: RuntimeInfo = Field(default_factory=RuntimeInfo)
    capabilities: ServerCapabilities = Field(default_factory=ServerCapabilities)
    providers: list[ProviderManifestInfo] = Field(default_factory=list)


class QueryParams(BaseModel):
    model_config = _CAMEL_CONFIG
    query: ContextQuery


class QueryResult(BaseModel):
    model_config = _CAMEL_CONFIG
    bundle: ContextBundle


class ResolveParams(BaseModel):
    model_config = _CAMEL_CONFIG
    uri: str
    depth: int | None = None
    access_level: str | None = None


class ResolveResult(BaseModel):
    model_config = _CAMEL_CONFIG
    object: ContextObject


class CapabilitiesParams(BaseModel):
    model_config = _CAMEL_CONFIG
    goal: Goal | None = None


class CapabilitiesResult(BaseModel):
    model_config = _CAMEL_CONFIG
    providers: list[ProviderManifestInfo] = Field(default_factory=list)


class SubscriptionFilter(BaseModel):
    model_config = _CAMEL_CONFIG
    providers: list[str] = Field(default_factory=list)
    context_types: list[str] = Field(default_factory=list)
    uri_patterns: list[str] = Field(default_factory=list)


class SubscribeParams(BaseModel):
    model_config = _CAMEL_CONFIG
    filter: SubscriptionFilter
    access_level: str | None = None


class SubscribeResult(BaseModel):
    model_config = _CAMEL_CONFIG
    subscription_id: str


class UnsubscribeParams(BaseModel):
    model_config = _CAMEL_CONFIG
    subscription_id: str


class UnsubscribeResult(BaseModel):
    model_config = _CAMEL_CONFIG
    success: bool


class EventParams(BaseModel):
    model_config = _CAMEL_CONFIG
    event: ContextEvent
    subscription_id: str


class PublishParams(BaseModel):
    model_config = _CAMEL_CONFIG
    event: ContextEvent


class PublishResult(BaseModel):
    model_config = _CAMEL_CONFIG
    accepted: bool
