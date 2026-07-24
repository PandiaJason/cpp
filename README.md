# Context Provider Protocol (CPP)

> **The universal open-standard perception layer for AI systems.**

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-F74C00.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10%2B-3776AB.svg?logo=python&logoColor=white)](https://python.org/)
[![License](https://img.shields.io/badge/License-MIT-22C55E.svg)](LICENSE-MIT)
[![Protocol](https://img.shields.io/badge/Protocol-v0.1.0-8B5CF6.svg)](spec/RFC-0001-CPP.md)
[![Tests](https://img.shields.io/badge/Tests-69%20passing-22C55E.svg)]()

CPP is an open protocol standard that provides AI assistants, autonomous agents, and LLM-powered IDEs (Google Antigravity, Claude, ChatGPT, Cursor, LangChain) with **structured, budget-aware, real-time perception** of their operating environment. 

It replaces ad-hoc prompt stuffing, static RAG pipelines, and unstructured shell outputs with a unified, typed context-resolution engine built on JSON-RPC 2.0.

---

## 🌐 Vision: Why the AI Industry Needs CPP

Today, AI models suffer from **Context Inflation** and **Unstructured Noise**. Every AI tool (IDE, CLI agent, browser sidecar) invents its own prompt-stuffing mechanisms, reading whole files and terminal streams into the LLM context window.

```
                  THE AI PERCEPTION GAP

        Without CPP: Unstructured Prompt Stuffing
 ┌──────────────┐     Raw Files / Shell Dumps      ┌───────────────┐
 │  AI Model    │ ◀─────────────────────────────── │ Workspace &   │
 │ (30k Tokens) │ 30,511 tokens ($0.091 / query)   │ Tool Output   │
 └──────────────┘                                  └───────────────┘

          With CPP: Structured Semantic Context Graph
 ┌──────────────┐    Typed SCOs & Budget Solved    ┌───────────────┐
 │  AI Model    │ ◀─────────────────────────────── │  CPP Engine   │
 │ (58 Tokens)  │ 58 tokens ($0.00017 / query)     │ (Git, Slack,  │
 └──────────────┘ 99.81% Reduction at Source       │  Jira, Files) │
                                                   └───────────────┘
```

**CPP solves this for the entire AI industry:**
- **For AI Labs (Google DeepMind, OpenAI, Anthropic):** Provides a standardized, clean context-perception layer so models spend context windows on *reasoning* rather than sifting through raw text.
- **For Developer Tools (Cursor, Antigravity, VS Code, Windsurf):** Eliminates custom data-fetching shims by exposing a single protocol for codebases, issues, PRs, and communications.
- **For Enterprise & SaaS (GitHub, Jira, Slack, Notion):** Enables SaaS providers to implement a single `ContextProvider` interface that works natively across all AI assistants.

---

## 📊 Benchmark: Antigravity Agent WITH vs. WITHOUT CPP

Empirical measurements benchmarked live on the repository workspace:

| Metric | Without CPP (Raw Tool/Shell Dump) | With CPP (Semantic Perception) | Impact / Performance Gain |
|:-------|:---------------------------------:|:------------------------------:|:-------------------------:|
| **Context Payload** | 122,044 bytes | **232 bytes** | **99.81% Reduction** |
| **Estimated LLM Tokens** | 30,511 tokens | **58 tokens** | **~526x Token Savings** |
| **Context Resolution Latency** | 450 – 1200 ms | **2.10 ms** | **~200x Faster Perception** |
| **Cost per Query Turn** | ~$0.091 | **~$0.00017** | **99.81% Cost Reduction** |
| **Change Detection** | Polling loops (`while true`) | **WebSocket Push Notifications** | Zero-latency event streaming |
| **Context Structure** | Unstructured text | **Typed Relational Graph** | Zero path/symbol hallucinations |

```bash
# Run the live benchmark on any directory:
cargo run --bin benchmark -- "/path/to/workspace"
```

---

## 🏛️ Protocol Architecture: Perceive → Reason → Act

CPP operates upstream of the LLM model and MCP execution layer:

```
                         JSON-RPC 2.0 / HTTP + WebSocket
                         ─────────────────────────────────
 ┌─────────────────┐          ┌────────────────────┐          ┌──────────────────┐
 │   AI Clients    │  ──────▶ │     CPP Server     │ ◀──────  │    Providers     │
 │                 │  query   │                    │  register│                  │
 │  Cursor, Claude │  ◀────── │  Query Router      │  ──────▶ │  Filesystem      │
 │  LangChain      │  bundle  │  Budget Solver     │  resolve │  Git             │
 │  Custom Agents  │          │  Context Graph     │          │  GitHub / Jira   │
 └────────┬────────┘          │  Event Bus         │          │  Slack / Custom  │
          │                   └────────┬───────────┘          └──────────────────┘
          │                            │
          └────── WebSocket ◀──────────┘
                  (cpp/event push notifications)
```

| Layer | Protocol | Role & Responsibility |
|:------|:---------|:----------------------|
| **Perceive** | **CPP (Context Provider Protocol)** | Delivers structured, budget-bounded context graphs to the model |
| **Reason** | **LLM (Gemini, Claude, GPT-4)** | Processes context, performs reasoning, formulates execution plans |
| **Act** | **MCP (Model Context Protocol)** | Executes terminal tools, file edits, and system actions |

---

## 💡 Core Concepts

### 1. Semantic Context Object (SCO)

Every piece of context is represented as an addressable, typed, permissioned, temporally-aware unit:

```
┌─────────────────────────────────────────────────────────────────┐
│  Semantic Context Object (SCO)                                  │
├──────────────┬──────────────────────────────────────────────────┤
│  Identity    │  uri: cpp://github/pull_request/42               │
│              │  context_type: application/cpp.entity.pull_request│
│              │  provider_id: "github"                           │
├──────────────┼──────────────────────────────────────────────────┤
│  Temporal    │  created_at, updated_at, expires_at              │
├──────────────┼──────────────────────────────────────────────────┤
│  Semantics   │  certainty: Authoritative | Derived | Estimated  │
│              │  freshness: Live | Recent(ttl) | Cached | Immut. │
│              │  importance: 0..100                              │
│              │  lifecycle:  Created → Updated → Archived        │
├──────────────┼──────────────────────────────────────────────────┤
│  Content     │  title, summary, content (access-gated)          │
├──────────────┼──────────────────────────────────────────────────┤
│  Graph       │  relations: [Relation], references: [Reference]  │
├──────────────┼──────────────────────────────────────────────────┤
│  Extension   │  metadata: {}, extensions: {}                    │
└──────────────┴──────────────────────────────────────────────────┘
```

### 2. Context Type Taxonomy

Three-level MIME-like hierarchy prevents namespace collisions:

```
application/<namespace>.<class>.<type>
            │           │       │
            │           │       └── Specific type (file, commit, issue)
            │           └────────── Base class (Entity, Document, Event, Collection, Reference)
            └────────────────────── Namespace ("cpp" = protocol-defined, "github" = provider-defined)
```

**36 protocol-defined types** across 5 base classes:

| Class | Standard Types |
|:------|:---------------|
| **Entity** | `person`, `project`, `repository`, `team`, `company`, `branch`, `issue`, `pull_request`, `label`, `milestone` |
| **Document** | `file`, `email`, `note`, `page`, `message`, `diff`, `comment`, `log`, `metric`, `config`, `snippet` |
| **Event** | `meeting`, `commit`, `task_update`, `notification`, `temporal`, `deployment`, `calendar_event` |
| **Collection** | `folder`, `channel`, `board`, `workspace`, `directory`, `conversation` |
| **Reference** | `link`, `bookmark` |

### 3. Context Query Language (CQL)

Queries combine a **goal**, **budget**, **scope**, and **filters**:

```json
{
  "goal": "goal.code",
  "budget": { "maxBytes": 4096, "maxObjects": 10, "prefer": "quality" },
  "scope": { "workspacePath": "/path/to/project" },
  "text": "authentication",
  "uriPattern": "cpp://github/*",
  "sourceFilter": "git",
  "createdAfter": "2025-01-01T00:00:00Z",
  "rankingPolicy": "relevance",
  "followRelations": true,
  "maxDepth": 2
}
```

### 4. Context Graph

Query responses return a typed relationship graph connecting objects across providers:

```json
{
  "graph": {
    "nodes": ["ctx_a1b2", "ctx_c3d4", "ctx_e5f6"],
    "edges": [
      { "source": "ctx_a1b2", "target": "ctx_c3d4", "edgeType": "contains" },
      { "source": "ctx_c3d4", "target": "ctx_e5f6", "edgeType": "references" }
    ],
    "cycleDetected": false
  }
}
```

---

## ⚡ Protocol Method Reference

### JSON-RPC 2.0 API

| Method | Type | Description |
|:-------|:-----|:------------|
| `cpp/initialize` | Request | Session handshake and capability negotiation |
| `cpp/initialized` | Notification | Client confirms session initialization |
| `cpp/query` | Request | Query context graph with CQL filters and budget |
| `cpp/resolve` | Request | Resolve a single SCO by `cpp://` URI |
| `cpp/capabilities` | Request | List active providers and capabilities |
| `cpp/providers/list` | Request | Discovery endpoint for registered providers |
| `cpp/subscribe` | Request | Register WebSocket event filters |
| `cpp/unsubscribe` | Request | Cancel active WebSocket subscription |
| `cpp/publish` | Request | Publish an event to the server event bus |
| `cpp/event` | Notification | Server → Client WebSocket push notification |
| `cpp/shutdown` | Request | Graceful session teardown with statistics |
| `cpp/exit` | Notification | Final session termination signal |

---

## 🚀 Quick Start

### 1. Launch the CPP Server Daemon

```bash
cargo run --bin cpp-server
# Server starts on http://localhost:3030
# Visual Glassmorphic Dashboard available at http://localhost:3030
```

### 2. Query Workspace Context (Python SDK)

```bash
pip install sdks/python
```

```python
import asyncio
from cpp_sdk import CppClient, Goal, BudgetPreference

async def main():
    async with CppClient("http://localhost:3030") as client:
        init = await client.initialize()
        print(f"Connected to {init.runtime_info.name} (Protocol {init.protocol_version})")

        # Perform budget-bounded context query
        bundle = await client.query(
            Goal.code(),
            budget_max_bytes=4096,
            budget_prefer=BudgetPreference.QUALITY,
            workspace_path="/path/to/project"
        )

        print(f"Retrieved {bundle.total_count} SCOs in {bundle.resolution_time_ms}ms:")
        for obj in bundle.objects:
            print(f"  [{obj.certainty}] {obj.title} ({obj.context_type}) -> {obj.uri}")

asyncio.run(main())
```

### 3. Connect to Claude Desktop / Cursor (MCP Bridge)

Add CPP bridge to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "cpp": {
      "command": "python",
      "args": ["-m", "cpp_sdk.bridges.mcp_bridge"],
      "env": { "CPP_SERVER_URL": "http://localhost:3030" }
    }
  }
}
```

Exposes three tools to your assistant: `cpp_query`, `cpp_resolve`, and `cpp_capabilities`.

---

## 🛠️ Repository Structure

```
context-provider-protocol/
├── spec/                              # Specifications & Philosophy
│   ├── RFC-0000-Philosophy.md         #   Design philosophy and principles
│   └── RFC-0001-CPP.md               #   Full protocol specification
│
├── crates/                            # Rust Core Engine
│   ├── cpp-core/                      #   SCO, URI, Budget Solver, Graph, Permissions
│   ├── cpp-protocol/                  #   JSON-RPC 2.0 wire format & schemas
│   ├── cpp-sdk/                       #   ContextProvider trait & CppClient
│   ├── cpp-runtime/                   #   ContextResolver & ContextCache
│   ├── cpp-server/                    #   Axum HTTP/WebSocket daemon + Dashboard
│   ├── cpp-transport-http/            #   HTTP transport framing
│   └── cpp-transport-stdio/           #   Stdio transport adapter
│
├── providers/                         # Context Providers
│   ├── filesystem/                    #   Files & directories (Rust)
│   ├── git/                           #   Repositories, branches, commits (Rust)
│   ├── datetime/                      #   System time & temporal context (Rust)
│   ├── github/                        #   Pull Requests, Issues, Commits (Python)
│   ├── jira/                          #   Sprint Issues, Epics, Blockers (Python)
│   └── slack/                         #   Channels & Threaded Messages (Python)
│
├── sdks/python/                       # Python SDK & MCP Bridge
│   ├── cpp_sdk/                       #   Pydantic v2 models, async client, MCP bridge
│   └── tests/                         #   14 serialization round-trip tests
│
└── examples/                          # Reference Demos & Benchmarks
    ├── simple-query/                  #   CLI context resolution demo
    ├── benchmark/                     #   Context window budget solver benchmark
    └── streaming/                     #   WebSocket event subscription demo
```

---

## 🧪 Testing

```bash
# Rust Workspace — 55 tests (Unit, Integration, Doc-tests)
cargo test --workspace

# Python SDK — 14 tests (Serialization & Round-trips)
cd sdks/python && uv run python -m pytest tests/ -v
```

---

## 📜 Specification & Standards

| Specification Document | Focus |
|:-----------------------|:------|
| [RFC-0000 — Philosophy](spec/RFC-0000-Philosophy.md) | Design principles: structured perception, budget negotiation, semantic typing |
| [RFC-0001 — CPP Specification](spec/RFC-0001-CPP.md) | Complete protocol spec: wire format, methods, types, transports, error codes |

---

## 📄 License

MIT © [CPP Contributors](LICENSE-MIT)
