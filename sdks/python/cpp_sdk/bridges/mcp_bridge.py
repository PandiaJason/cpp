"""MCP-to-CPP bridge server.

Exposes CPP query, resolve, and capabilities as Model Context Protocol (MCP) tools,
allowing MCP clients (such as Claude Desktop or Antigravity IDE) to interact with
a running CPP server.
"""

from __future__ import annotations

import asyncio
import inspect
import json
import os
from typing import Any

from mcp.server import Server
from mcp.types import TextContent, Tool
import mcp.server.stdio

from cpp_sdk.client import CppClient
from cpp_sdk.types import Goal


def _get_server_url() -> str:
    return os.environ.get("CPP_SERVER_URL", "http://localhost:3030")


# Ensure Server supports @server.tool() decorator
def _register_tool_support(srv: Server) -> None:
    if hasattr(srv, "_registered_tools"):
        return
    srv._registered_tools = {}
    srv._tool_handlers = {}

    @srv.list_tools()
    async def _list_tools() -> list[Tool]:
        return list(srv._registered_tools.values())

    @srv.call_tool()
    async def _call_tool(name: str, arguments: dict[str, Any]) -> list[TextContent]:
        if name not in srv._tool_handlers:
            return [TextContent(type="text", text=f"Error: Unknown tool '{name}'")]
        handler = srv._tool_handlers[name]
        try:
            result = await handler(**arguments)
            if isinstance(result, list) and all(isinstance(x, TextContent) for x in result):
                return result
            if isinstance(result, TextContent):
                return [result]
            if isinstance(result, str):
                return [TextContent(type="text", text=result)]
            return [TextContent(type="text", text=json.dumps(result, indent=2))]
        except Exception as exc:
            return [TextContent(type="text", text=f"Error executing tool '{name}': {str(exc)}")]


def tool(self: Server, name: str | None = None, description: str | None = None):
    """Decorator to register an MCP tool on this Server instance."""
    _register_tool_support(self)

    def decorator(func):
        tool_name = name or func.__name__
        tool_doc = description or (func.__doc__ or "").strip()

        sig = inspect.signature(func)
        properties = {}
        required = []
        for param_name, param in sig.parameters.items():
            param_type = "string"
            if param.annotation in (int, int | None, "int | None", "Optional[int]"):
                param_type = "integer"
            properties[param_name] = {"type": param_type, "description": f"Parameter {param_name}"}
            if param.default == inspect.Parameter.empty:
                required.append(param_name)

        input_schema = {
            "type": "object",
            "properties": properties,
        }
        if required:
            input_schema["required"] = required

        self._registered_tools[tool_name] = Tool(
            name=tool_name,
            description=tool_doc,
            inputSchema=input_schema,
        )
        self._tool_handlers[tool_name] = func
        return func

    if callable(name):
        func = name
        name = None
        return decorator(func)
    return decorator


if not hasattr(Server, "tool"):
    Server.tool = tool  # type: ignore[attr-defined]


# Ensure mcp.server.stdio has run_server helper if needed
if not hasattr(mcp.server.stdio, "run_server"):

    async def _run_server(srv: Server) -> None:
        from mcp.server.models import InitializationOptions
        from mcp.server.stdio import stdio_server
        import mcp.types as types

        async with stdio_server() as (read_stream, write_stream):
            init_options = InitializationOptions(
                server_name=srv.name,
                server_version="0.1.0",
                capabilities=srv.get_capabilities(
                    notification_options=types.NotificationOptions(),
                    experimental_capabilities={},
                ),
            )
            await srv.run(read_stream, write_stream, init_options)

    mcp.server.stdio.run_server = _run_server  # type: ignore[attr-defined]


server = Server("cpp-bridge")


@server.tool()
async def cpp_query(
    goal: str,
    budget_max_bytes: int | None = None,
    workspace_path: str | None = None,
) -> str:
    """Execute a CPP query with an intent goal and budget constraints."""
    try:
        goal_lower = goal.lower()
        if goal_lower == "code":
            goal_obj = Goal.code()
        elif goal_lower == "project":
            goal_obj = Goal.project()
        elif goal_lower == "document":
            goal_obj = Goal.document()
        elif goal_lower == "calendar":
            goal_obj = Goal.calendar()
        else:
            goal_obj = Goal(intent=goal)

        async with CppClient(_get_server_url()) as client:
            bundle = await client.query(
                goal_obj,
                budget_max_bytes=budget_max_bytes,
                workspace_path=workspace_path,
            )
            return bundle.model_dump_json(by_alias=True, exclude_none=True)
    except Exception as exc:
        return json.dumps({"error": f"cpp_query failed: {str(exc)}"})


@server.tool()
async def cpp_resolve(uri: str) -> str:
    """Resolve a single Semantic Context Object (SCO) by its CPP URI."""
    try:
        async with CppClient(_get_server_url()) as client:
            obj = await client.resolve(uri)
            return obj.model_dump_json(by_alias=True, exclude_none=True)
    except Exception as exc:
        return json.dumps({"error": f"cpp_resolve failed: {str(exc)}"})


@server.tool()
async def cpp_capabilities() -> str:
    """Query capabilities and registered provider manifests from the CPP server."""
    try:
        async with CppClient(_get_server_url()) as client:
            providers = await client.capabilities()
            return json.dumps(
                [p.model_dump(by_alias=True, exclude_none=True) for p in providers],
                indent=2,
            )
    except Exception as exc:
        return json.dumps({"error": f"cpp_capabilities failed: {str(exc)}"})


async def main() -> None:
    """Run the MCP-to-CPP bridge server with stdio transport."""
    await mcp.server.stdio.run_server(server)


if __name__ == "__main__":
    asyncio.run(main())
