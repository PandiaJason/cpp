# Context Provider Protocol (CPP)

**The perception layer for AI agents.**

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10%2B-blue)](https://python.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)
[![Protocol](https://img.shields.io/badge/Protocol-v0.1.0-purple)]()

CPP is an open standard that gives AI assistants **structured, budget-aware, real-time perception** of your workspace. It replaces prompt stuffing, static RAG, and ad-hoc shell commands with a single protocol that connects any context source to any AI tool.

```
┌─────────────┐     ┌─────────────────┐     ┌──────────────┐
│  AI Agent    │────▶│   CPP Server    │◀────│  Providers   │
│ (Cursor,     │     │  (Perception    │     │ (Git, FS,    │
│  Copilot,    │ RPC │   Orchestrator) │     │  GitHub,     │
│  Claude)     │◀────│                 │────▶│  Jira, Slack)│
└─────────────┘     └─────────────────┘     └──────────────┘
      ▲                     │
      │              WebSocket Events
      │              (real-time push)
      └─────────────────────┘
```

---

## Table of Contents

- [Why CPP?](#why-cpp)
- [How It Works](#how-it-works)
- [Project Structure](#project-structure)
- [Getting Started](#getting-started)
- [Python SDK](#python-sdk)
- [MCP-to-CPP Bridge](#mcp-to-cpp-bridge)
- [SaaS Providers](#saas-context-providers)
- [Benchmarks](#benchmarks)
- [Specifications](#specifications)
- [Contributing](#contributing)

---

## Why CPP?

AI assistants today are **blind**. They rely on brute-force file dumping, vector searches, or ad-hoc terminal commands to understand your workspace. This leads to:

| Problem | Without CPP | With CPP |
| :--- | :--- | :--- |
| **Token waste** | Dumps 50,000 tokens of irrelevant files | Sends only 500 tokens of relevant context |
| **No relationships** | Flat list of files, no structure | Graph of nodes + edges (commit → modifies → file) |
| **Stale context** | Polls `git status` every 5 seconds | WebSocket push events, zero polling |
| **Per-tool custom code** | Custom integration for each AI tool | One standard API works everywhere |
| **No budget control** | Overflows context window silently | Negotiates strict byte/object limits |

### CPP vs Existing Approaches

| Feature | Prompt Stuffing | RAG | MCP Tools | **CPP** |
| :--- | :--- | :--- | :--- | :--- |
| Budget enforcement | ❌ | ❌ | ❌ | ✅ Server-side |
| Relationship graph | ❌ | ❌ | ❌ | ✅ Directed edges |
| Real-time events | ❌ | ❌ | ❌ | ✅ WebSocket push |
| Cross-tool standard | ❌ | ❌ | ✅ | ✅ |
| Context ranking | ❌ | Similarity | ❌ | ✅ Importance + recency |

---

## How It Works

CPP does exactly three things:

### 1. Standard API (Plug-and-Play)
Every context source — filesystem, git, GitHub, Jira, Slack — plugs into one universal JSON-RPC 2.0 API. Any AI tool connects to the same `cpp/query` endpoint.

### 2. Relationship Graph (Connect the Dots)
CPP builds a directed graph of **Semantic Context Objects (SCOs)** linked by typed edges:

```
[Repository: cpp] ──contains──▶ [Branch: main]
                                      │
                                  references
                                      ▼
                               [Commit: c75815e]
                                      │
                                   modifies
                                      ▼
                               [File: main.rs] ──imports──▶ [File: client.rs]
```

Relationships are parsed **locally by your CPU** (git logs, import statements) — not guessed by the LLM. 100% accurate, zero tokens.

### 3. Budget Engine (Trim the Fat)
Before sending context to the AI, CPP negotiates a strict budget:

```python
budget = ContextBudget(max_bytes=4096, max_objects=10, prefer="quality")
```

The server ranks objects by importance, drops the irrelevant ones, and guarantees the response fits within the AI's context window.

---

## Project Structure

```
context-provider-protocol/
├── spec/                          # Protocol specifications
│   ├── RFC-0000-Philosophy.md     # Design philosophy
│   └── RFC-0001-CPP.md            # Wire protocol spec (JSON-RPC 2.0)
│
├── crates/                        # Rust core (the engine)
│   ├── cpp-core/                  # Types, SCO schema, query models, budget logic
│   ├── cpp-protocol/              # JSON-RPC message definitions
│   ├── cpp-sdk/                   # ContextProvider trait, CppClient, ProviderAdapter
│   ├── cpp-runtime/               # ContextResolver, ContextCache orchestration
│   └── cpp-server/                # Axum HTTP daemon + WebSocket + dashboard
│
├── providers/                     # Context source plugins
│   ├── filesystem/                # Local file scanner (Rust)
│   ├── git/                       # Git history, branches, commits (Rust)
│   ├── datetime/                  # System timezone + calendar (Rust)
│   ├── github/                    # GitHub PRs, issues, commits (Python)
│   ├── jira/                      # Jira issues, sprints, epics (Python)
│   └── slack/                     # Slack messages, channels (Python)
│
├── sdks/
│   └── python/                    # Python SDK
│       ├── cpp_sdk/               # Pydantic models, async client, provider base
│       │   └── bridges/           # MCP-to-CPP bridge
│       └── tests/                 # Serialization round-trip tests
│
└── examples/
    ├── simple-query/              # Minimal context query example
    ├── streaming/                 # WebSocket event streaming example
    └── benchmark/                 # Budget optimization benchmark
```

---

## Getting Started

### Prerequisites

- **Rust** 1.75+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Python** 3.10+ (for Python SDK and SaaS providers)

### 1. Launch the Server & Dashboard

```bash
cargo run --bin cpp-server
```

Navigate to **http://localhost:3030** to see the live context relationship graph in the glassmorphic dashboard.

### 2. Run a Context Query (Rust)

```bash
cargo run --bin simple-query
```

### 3. Run a Context Query (Python)

```bash
cd sdks/python
pip install -e .
```

```python
import asyncio
from cpp_sdk import CppClient, Goal

async def main():
    async with CppClient("http://localhost:3030") as client:
        bundle = await client.query(Goal.code(), budget_max_bytes=4096)
        for obj in bundle.objects:
            print(f"  {obj.title} → {obj.uri}")
        print(f"\n  {bundle.total_count} objects, {bundle.resolution_time_ms}ms")

asyncio.run(main())
```

### 4. Run the Budget Benchmark

```bash
# On this workspace
cargo run --bin benchmark

# On any folder
cargo run --bin benchmark -- "/path/to/your/project"
```

### 5. Explore the Specifications

- [RFC-0000: Philosophy](spec/RFC-0000-Philosophy.md) — Design principles and rationale
- [RFC-0001: Wire Protocol](spec/RFC-0001-CPP.md) — Full JSON-RPC 2.0 specification

---

## Python SDK

The Python SDK provides a complete client library with Pydantic v2 models that serialize to the exact camelCase JSON format expected by the CPP server.

### Installation

```bash
pip install cpp-sdk
```

### Core Components

| Module | Description |
| :--- | :--- |
| `cpp_sdk.types` | Goal, ContextType, ContextBudget, Certainty, Freshness, AccessLevel, etc. |
| `cpp_sdk.context` | ContextObject (SCO), ContextBundle, ContextObjectBuilder |
| `cpp_sdk.query` | ContextQuery, ContextQueryBuilder with fluent API |
| `cpp_sdk.client` | Async CppClient (HTTP + WebSocket) |
| `cpp_sdk.provider` | Abstract ContextProvider and ProviderAdapter base classes |
| `cpp_sdk.protocol` | JSON-RPC 2.0 message types and CPP method constants |
| `cpp_sdk.errors` | Exception hierarchy (CppError, CppConnectionError, etc.) |

### Query with Budget

```python
from cpp_sdk import CppClient, Goal, BudgetPreference

async with CppClient("http://localhost:3030") as client:
    bundle = await client.query(
        Goal.code(),
        budget_max_bytes=4096,
        budget_prefer=BudgetPreference.QUALITY,
        workspace_path="/my/project",
    )
    print(f"Received {bundle.total_count} objects in {bundle.resolution_time_ms}ms")
```

### Build a Custom Provider

```python
from cpp_sdk import (
    ContextProvider, ProviderManifest, ProviderCapabilities,
    ContextType, Goal, ContextBundle, ContextObjectBuilder,
    Certainty, Freshness,
)

class DatabaseProvider(ContextProvider):
    @property
    def manifest(self) -> ProviderManifest:
        return ProviderManifest(
            id="database",
            name="Database Schema Provider",
            capabilities=ProviderCapabilities.basic(
                context_types=[ContextType.file()],
                goals=[Goal.code()],
            ),
        )

    async def query(self, query):
        schema = await self._fetch_schema()
        obj = (
            ContextObjectBuilder("cpp://db/schema/main", ContextType.file(), "database")
            .title("Database Schema")
            .content(schema)
            .certainty(Certainty.AUTHORITATIVE)
            .freshness(Freshness.live())
            .build()
        )
        return ContextBundle(objects=[obj], total_count=1)

    async def resolve(self, uri):
        ...
```

### Subscribe to Live Events

```python
async with CppClient("http://localhost:3030") as client:
    async for event in client.subscribe():
        print(f"[{event.kind}] {event.uri} from {event.provider_id}")
```

---

## MCP-to-CPP Bridge

Connect Claude Desktop, Cursor, or any MCP client to a running CPP server — zero code required.

### Setup

1. Start the CPP server:
   ```bash
   cargo run --bin cpp-server
   ```

2. Add to your Claude Desktop config (`claude_desktop_config.json`):
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

3. Restart Claude Desktop. Three new tools appear:

| MCP Tool | What It Does |
| :--- | :--- |
| `cpp_query` | Query workspace context by goal with optional byte budget |
| `cpp_resolve` | Resolve a single SCO by its CPP URI |
| `cpp_capabilities` | List available providers and their capabilities |

---

## SaaS Context Providers

Ready-made providers that translate external SaaS APIs into standard SCOs.

### GitHub Provider

Surfaces pull requests, issues, and commits with relationship edges (PR → modifies → commit).

```python
from providers.github.github_provider import GitHubProvider

provider = GitHubProvider(owner="myorg", repo="myrepo")
await provider.start()

bundle = await provider.query(query)
# Returns: PRs, issues, commits as SCOs with relations
```

**Environment**: `GITHUB_TOKEN`

### Jira Provider

Surfaces issues, sprints, and epics with blocking/relating relationship edges.

```python
from providers.jira.jira_provider import JiraProvider

provider = JiraProvider(project_key="PROJ")
await provider.start()

bundle = await provider.query(query)
# Returns: Current sprint issues with blocks/relates_to edges
```

**Environment**: `JIRA_BASE_URL`, `JIRA_EMAIL`, `JIRA_API_TOKEN`

### Slack Provider

Surfaces channel messages and threads with reaction-based importance weighting.

```python
from providers.slack.slack_provider import SlackProvider

provider = SlackProvider(channels=["C01ABC123"])
await provider.start()

bundle = await provider.query(query)
# Returns: Recent messages ranked by reactions
```

**Environment**: `SLACK_BOT_TOKEN`

---

## Benchmarks

CPP's budget engine achieves **98%+ context reduction** at the source level:

```text
-----------------------------------------------------
| Metric             | Unbudgeted       | Budgeted        |
-----------------------------------------------------
| File Count         | 48               | 4               |
| Total Content Size | 222,287 bytes    | 1,998 bytes     |
-----------------------------------------------------
>>> Combined context volume reduced by 98.95% at source!
```

Run it yourself:
```bash
cargo run --bin benchmark -- "/path/to/any/folder"
```

---

## Specifications

| Document | Description |
| :--- | :--- |
| [RFC-0000: Philosophy](spec/RFC-0000-Philosophy.md) | Design principles, the Perceive → Reason → Act cycle, and why CPP exists |
| [RFC-0001: Wire Protocol](spec/RFC-0001-CPP.md) | Complete JSON-RPC 2.0 specification with all methods, types, and schemas |

### Protocol Methods

| Method | Direction | Purpose |
| :--- | :--- | :--- |
| `cpp/initialize` | Client → Server | Handshake, capability negotiation |
| `cpp/query` | Client → Server | Query context with goal, budget, scope |
| `cpp/resolve` | Client → Server | Resolve a single SCO by URI |
| `cpp/capabilities` | Client → Server | List available providers |
| `cpp/subscribe` | Client → Server | Subscribe to live events |
| `cpp/event` | Server → Client | Push real-time context changes |
| `cpp/publish` | Client → Server | Publish a context event |
| `cpp/shutdown` | Client → Server | Graceful shutdown |

---

## Contributing

CPP is an open-source project built to solve the context fragmentation facing modern AI agents. We welcome contributions across all layers:

- **New Providers** — Connect a new data source (databases, CI/CD, monitoring, etc.)
- **SDK Ports** — Port the Python SDK to TypeScript, Go, or other languages
- **Protocol Extensions** — Propose new RFC specifications
- **Bug Reports** — File issues for any problems you encounter

### Running Tests

```bash
# Rust tests (52 tests)
cargo test --workspace

# Python SDK tests (14 tests)
cd sdks/python && pip install -e ".[dev]" && pytest tests/ -v
```

---

## License

MIT
