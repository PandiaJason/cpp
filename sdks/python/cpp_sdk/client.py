"""Async CPP client for agents and tools.

``CppClient`` communicates with a running CPP server over HTTP
(JSON-RPC on ``/api/rpc``) and optionally WebSocket (``/api/events``).

Usage::

    async with CppClient("http://localhost:3030") as client:
        bundle = await client.query(Goal.code(), budget_max_bytes=4096)
        for obj in bundle.objects:
            print(obj.title, obj.uri)
"""

from __future__ import annotations

import itertools
import json
import logging
from typing import Any, AsyncIterator

import httpx

from .context import ContextBundle, ContextObject
from .errors import (
    CppConnectionError,
    CppProtocolError,
    CppTimeoutError,
)
from .protocol import (
    CPP_CAPABILITIES,
    CPP_EVENT,
    CPP_INITIALIZE,
    CPP_QUERY,
    CPP_RESOLVE,
    CapabilitiesParams,
    CapabilitiesResult,
    InitializeParams,
    InitializeResult,
    JsonRpcRequest,
    JsonRpcResponse,
    ProviderManifestInfo,
    QueryParams,
    QueryResult,
    ResolveParams,
    ResolveResult,
)
from .query import ContextQuery, ContextQueryBuilder
from .types import (
    BudgetPreference,
    ContextBudget,
    ContextEvent,
    Goal,
)

logger = logging.getLogger("cpp_sdk.client")

_id_counter = itertools.count(1)


class CppClient:
    """High-level async client for a CPP server.

    Parameters
    ----------
    base_url:
        Root URL of the CPP server, e.g. ``"http://localhost:3030"``.
    timeout:
        HTTP request timeout in seconds (default 30).
    """

    def __init__(
        self,
        base_url: str = "http://localhost:3030",
        *,
        timeout: float = 30.0,
    ) -> None:
        self._base_url = base_url.rstrip("/")
        self._rpc_url = f"{self._base_url}/api/rpc"
        self._events_url = f"{self._base_url}/api/events".replace("http", "ws", 1)
        self._http = httpx.AsyncClient(timeout=timeout)
        self._server_info: InitializeResult | None = None

    # ------------------------------------------------------------------
    # Context manager
    # ------------------------------------------------------------------

    async def __aenter__(self) -> CppClient:
        return self

    async def __aexit__(self, *exc: object) -> None:
        await self.close()

    async def close(self) -> None:
        """Close the underlying HTTP connection pool."""
        await self._http.aclose()

    # ------------------------------------------------------------------
    # Low-level JSON-RPC transport
    # ------------------------------------------------------------------

    async def _rpc_call(self, method: str, params: dict[str, Any] | None = None) -> Any:
        """Send a JSON-RPC 2.0 request and return the ``result`` value."""
        request_id = next(_id_counter)
        payload = JsonRpcRequest(id=request_id, method=method, params=params)

        try:
            resp = await self._http.post(
                self._rpc_url,
                json=payload.model_dump(by_alias=True, exclude_none=True),
            )
        except httpx.ConnectError as exc:
            raise CppConnectionError(
                f"Cannot reach CPP server at {self._rpc_url}: {exc}"
            ) from exc
        except httpx.TimeoutException as exc:
            raise CppTimeoutError(
                f"Request to {method} timed out: {exc}"
            ) from exc

        if resp.status_code != 200:
            raise CppProtocolError(
                f"HTTP {resp.status_code} from CPP server",
                code=resp.status_code,
            )

        rpc_resp = JsonRpcResponse.model_validate(resp.json())

        if rpc_resp.error is not None:
            raise CppProtocolError(
                rpc_resp.error.message,
                code=rpc_resp.error.code,
                data=rpc_resp.error.data,
            )

        return rpc_resp.result

    # ------------------------------------------------------------------
    # Protocol lifecycle
    # ------------------------------------------------------------------

    async def initialize(self, params: InitializeParams | None = None) -> InitializeResult:
        """Perform the ``cpp/initialize`` handshake.

        Returns server capabilities and available providers.
        """
        p = params or InitializeParams()
        raw = await self._rpc_call(
            CPP_INITIALIZE,
            p.model_dump(by_alias=True, exclude_none=True),
        )
        result = InitializeResult.model_validate(raw)
        self._server_info = result
        logger.info(
            "Connected to CPP server %s (protocol %s, %d providers)",
            result.runtime_info.name,
            result.protocol_version,
            len(result.providers),
        )
        return result

    # ------------------------------------------------------------------
    # Query
    # ------------------------------------------------------------------

    async def query(
        self,
        goal: Goal,
        *,
        budget_max_bytes: int | None = None,
        budget_max_objects: int | None = None,
        budget_prefer: BudgetPreference = BudgetPreference.QUALITY,
        workspace_path: str | None = None,
        max_results: int = 50,
    ) -> ContextBundle:
        """Send a ``cpp/query`` request and return the resolved context bundle.

        This is the primary method agents will call.
        """
        builder = ContextQueryBuilder(goal).max_results(max_results)

        if budget_max_bytes or budget_max_objects:
            builder = builder.budget(
                max_bytes=budget_max_bytes,
                max_objects=budget_max_objects,
                prefer=budget_prefer,
            )

        if workspace_path:
            builder = builder.hint("workspacePath", workspace_path)

        return await self.query_raw(builder.build())

    async def query_raw(self, query: ContextQuery) -> ContextBundle:
        """Send an already-built ``ContextQuery``."""
        params = QueryParams(query=query)
        raw = await self._rpc_call(
            CPP_QUERY,
            params.model_dump(by_alias=True, exclude_none=True),
        )
        if isinstance(raw, dict) and "bundle" in raw:
            return ContextBundle.model_validate(raw["bundle"])
        return ContextBundle.model_validate(raw)

    # ------------------------------------------------------------------
    # Resolve
    # ------------------------------------------------------------------

    async def resolve(self, uri: str, *, depth: int | None = None) -> ContextObject:
        """Resolve a single SCO by its CPP URI."""
        params = ResolveParams(uri=uri, depth=depth)
        raw = await self._rpc_call(
            CPP_RESOLVE,
            params.model_dump(by_alias=True, exclude_none=True),
        )
        result = ResolveResult.model_validate(raw)
        return result.object

    # ------------------------------------------------------------------
    # Capabilities
    # ------------------------------------------------------------------

    async def capabilities(self, goal: Goal | None = None) -> list[ProviderManifestInfo]:
        """Retrieve available provider manifests, optionally filtered by goal."""
        params = CapabilitiesParams(goal=goal)
        raw = await self._rpc_call(
            CPP_CAPABILITIES,
            params.model_dump(by_alias=True, exclude_none=True),
        )
        result = CapabilitiesResult.model_validate(raw)
        return result.providers

    # ------------------------------------------------------------------
    # Subscriptions (WebSocket)
    # ------------------------------------------------------------------

    async def subscribe(self) -> AsyncIterator[ContextEvent]:
        """Open a WebSocket connection and yield live ``ContextEvent`` objects.

        Usage::

            async for event in client.subscribe():
                print(event.kind, event.uri)

        Requires the ``websockets`` package.
        """
        try:
            import websockets
        except ImportError as exc:
            raise ImportError(
                "websockets package required for subscriptions: pip install websockets"
            ) from exc

        async for ws in websockets.connect(self._events_url):
            try:
                async for raw_msg in ws:
                    try:
                        data = json.loads(raw_msg)
                        if data.get("method") == CPP_EVENT and "params" in data:
                            event_data = data["params"].get("event", data["params"])
                            yield ContextEvent.model_validate(event_data)
                    except (json.JSONDecodeError, ValueError) as exc:
                        logger.warning("Malformed event: %s", exc)
                        continue
            except websockets.ConnectionClosed:
                logger.info("WebSocket disconnected, reconnecting...")
                continue
