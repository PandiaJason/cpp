# Context Provider Protocol (CPP)

**The unified perception layer for AI agents.**

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10%2B-blue.svg)](https://python.org/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Protocol](https://img.shields.io/badge/Protocol-v0.1.0-purple.svg)](spec/RFC-0001-CPP.md)

CPP is an open standard that gives AI assistants **structured, budget-aware, real-time perception** of their workspace. It replaces prompt stuffing, static RAG, and ad-hoc shell commands with a universal protocol connecting dynamic context sources to AI applications.

```
┌─────────────────┐       ┌────────────────────┐       ┌──────────────────┐
│    AI Client    │ ────▶ │     CPP Server     │ ◀──── │ Context Provider │
│ (Cursor, Claude,│  RPC  │ (Perception Engine │       │  (Git, Files,    │
│  LangChain)     │ ◀──── │  & Budget Solver)  │ ────▶ │   Jira, Slack)   │
└─────────────────┘       └────────────────────┘       └──────────────────┘
         ▲                          │
         │                   WebSocket Events
         └──────────────────────────┘
```

---

## Key Features

- **Source-Side Budget Negotiation**: Clients specify token/byte budgets (`ContextBudget`); the server ranks and downsamples data at the source before sending to the LLM (**98%+ token savings**).
- **Relational Context Graphs**: Replaces flat file dumps with typed graph nodes and edges (`Branch` $\rightarrow$ `Commit` $\rightarrow$ `File` $\rightarrow$ `Issue`).
- **Passive Event Streams**: Replaces polling loops (`while true { check status }`) with real-time WebSocket push notifications (`cpp/event`).
- **Universal Standard**: One single JSON-RPC 2.0 API connects any data provider to any AI agent tool or IDE plugin.

---

## Overview: Perceive $\rightarrow$ Reason $\rightarrow$ Act

| Phase | Protocol / Layer | Role |
| :--- | :--- | :--- |
| **Perceive** | **CPP (Context Provider Protocol)** | Delivers structured, token-budgeted workspace context graphs |
| **Reason** | **LLM (Gemini, Claude, GPT-4)** | Processes context and formulates execution plans |
| **Act** | **MCP (Model Context Protocol)** | Executes terminal tools, file edits, and system actions |

---

## Quick Start

### 1. Launch the Server Daemon & Visual Dashboard

```bash
cargo run --bin cpp-server
```
Open **http://localhost:3030** in your browser to inspect the live glassmorphic relationship graph visualizer.

### 2. Query Workspace Context (Python SDK)

```bash
pip install sdks/python
```

```python
import asyncio
from cpp_sdk import CppClient, Goal, BudgetPreference

async def main():
    async with CppClient("http://localhost:3030") as client:
        # Perform handshake
        init = await client.initialize()
        print(f"Connected to {init.runtime_info.name} (Protocol {init.protocol_version})")

        # Query workspace with a strict 4KB token budget
        bundle = await client.query(
            Goal.code(),
            budget_max_bytes=4096,
            budget_prefer=BudgetPreference.QUALITY,
        )

        print(f"\nRetrieved {bundle.total_count} objects in {bundle.resolution_time_ms}ms:")
        for obj in bundle.objects:
            print(f" - [{obj.certainty}] {obj.title} ({obj.context_type}) -> {obj.uri}")

asyncio.run(main())
```

### 3. Connect to Claude Desktop or Cursor (MCP Bridge)

Add the CPP bridge to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "cpp": {
      "command": "python",
      "args": ["-m", "cpp_sdk.bridges.mcp_bridge"],
      "env": {
        "CPP_SERVER_URL": "http://localhost:3030"
      }
    }
  }
}
```

This exposes three tools to your assistant: `cpp_query`, `cpp_resolve`, and `cpp_capabilities`.

---

## Repository Structure

```
context-provider-protocol/
├── spec/                        # Protocol Specifications (RFC-0000 & RFC-0001)
├── crates/                      # Core Rust Engine
│   ├── cpp-core/                # SCO schema, types, budget solver & permissions
│   ├── cpp-protocol/            # JSON-RPC 2.0 wire models
│   ├── cpp-sdk/                 # ContextProvider trait & CppClient
│   ├── cpp-runtime/             # ContextResolver & ContextCache orchestration
│   └── cpp-server/              # Axum HTTP daemon + WebSocket + Dashboard
├── providers/                   # Context Provider Adapters
│   ├── filesystem/ & git/       # Local codebase & repository history (Rust)
│   ├── github/                  # Pull requests, issues, commits (Python)
│   ├── jira/                    # Sprint issues, epics, blockers (Python)
│   └── slack/                   # Channel messages & threads (Python)
└── sdks/python/                 # Full Python SDK & MCP Bridge
```

---

## Protocol Method Reference

| Method | Type | Description |
| :--- | :--- | :--- |
| `cpp/initialize` | Request | Handshake and capability negotiation |
| `cpp/query` | Request | Query context graph with goal, budget, and scope filters |
| `cpp/resolve` | Request | Resolve a single Semantic Context Object by URI |
| `cpp/capabilities` | Request | List active context providers and their capabilities |
| `cpp/subscribe` | Request | Register client-specific WebSocket event filters |
| `cpp/unsubscribe` | Request | Cancel active WebSocket event subscription |
| `cpp/publish` | Request | Publish an event to the server event bus |
| `cpp/event` | Notification | Server-to-client push notification of context changes |

---

## Benchmarks

CPP's budget engine reduces raw context payload size by **98%+** at the source:

```text
-----------------------------------------------------
| Metric             | Unbudgeted       | Budgeted        |
-----------------------------------------------------
| File Count         | 48               | 4               |
| Total Content Size | 222,287 bytes    | 1,998 bytes     |
-----------------------------------------------------
>>> Combined context volume reduced by 98.95% at source!
```

To run the local benchmark against any workspace:
```bash
cargo run --bin benchmark -- "/path/to/target/folder"
```

---

## Running Tests

```bash
# Run Rust workspace test suite (55 tests)
cargo test --workspace

# Run Python SDK test suite (14 tests)
cd sdks/python && uv run python -m pytest tests/ -v
```

---

## License

MIT © [CPP Contributors](LICENSE)
