# Context Provider Protocol (CPP)

> **The open-standard perception layer for AI systems.**

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-F74C00.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10%2B-3776AB.svg?logo=python&logoColor=white)](https://python.org/)
[![License](https://img.shields.io/badge/License-MIT-22C55E.svg)](LICENSE-MIT)
[![Protocol](https://img.shields.io/badge/Protocol-v0.1.0-8B5CF6.svg)](spec/RFC-0001-CPP.md)
[![Tests](https://img.shields.io/badge/Tests-69%20passing-22C55E.svg)]()

CPP is an open protocol specification designed for AI assistants, autonomous coding agents, and IDEs (Cursor, Claude Code, GitHub Copilot, OpenAI Codex, Gemini CLI). It provides a **structured, budget-aware, real-time perception layer** that operates upstream of execution tools.

CPP complements the **Model Context Protocol (MCP)**: while MCP standardizes how AI systems **act** (tool execution, file edits, mutations), CPP standardizes how AI systems **perceive** (situated context resolution, token-budget enforcement, and relational graphs).

---

## 🏛️ The AI Execution Stack

```
             HUMAN                          AI SYSTEM
        ┌─────────────┐                 ┌──────────────┐
        │    Sees     │                 │     CPP      │  Perceives
        ├─────────────┤                 │ (Perception) │  (Budgeted Graph)
        │ Understands │                 ├──────────────┤
        ├─────────────┤  ─────────────▶ │     LLM      │  Reasons
        │    Acts     │                 │ (Reasoning)  │  (Plan & Strategy)
        └─────────────┘                 ├──────────────┤
                                        │     MCP      │  Acts
                                        │ (Execution)  │  (Tools & Edits)
                                        └──────────────┘
```

---

## ❓ Why Existing Approaches Fail

Before CPP, AI tools gathered workspace context using ad-hoc mechanisms that struggle at scale:

1. **Prompt Stuffing:** Reading whole files and terminal outputs fills the context window, triggers "lost in the middle" degradation, and inflates API costs.
2. **Standard RAG:** Unstructured text similarity retrieves isolated text chunks but misses critical software relationships (`Branch` $\rightarrow$ `Commit` $\rightarrow$ `Issue` $\rightarrow$ `PR` $\rightarrow$ `File`).
3. **Unstructured Tool Output:** MCP tool execution returns raw text blobs that must be parsed repeatedly by the LLM.
4. **Passive Polling Loops:** Agents run continuous `while true` polling loops to detect workspace changes, wasting time and compute.

**CPP replaces prompt stuffing and RAG with a source-side, budget-enforced Semantic Context Graph.**

---

## ⚔️ Protocol Feature Matrix

| Feature | Traditional RAG | Model Context Protocol (MCP) | Context Provider Protocol (CPP) |
|:--------|:---------------:|:---------------------------:|:-------------------------------:|
| **Executes System Tools** | ❌ | **✅ Primary Role** | ❌ *(Delegated to MCP)* |
| **Structured Context Perception** | ❌ | Partial *(Resources)* | **✅ Primary Role** |
| **Source-Side Token Budgeting** | ❌ | ❌ | **✅ Built-in Solver** |
| **Relational Context Graph** | ❌ | ❌ | **✅ Typed Nodes & Edges** |
| **Semantic Type Taxonomy** | ❌ | Partial | **✅ 36 Standard MIME Types** |
| **Real-time Push Notifications** | Partial | Partial | **✅ WebSocket Event Bus** |

---

## 📈 Empirical Repository Benchmark (Methodology & Results)

* **Test Methodology:** Benchmark run on the CPP codebase workspace comparing unbudgeted raw file scanning against CPP's budget-enforced context solver with a 4 KB budget constraint.

```
-------------------------------------------------------------------------
| Metric                     | Raw Unbudgeted Dump | CPP Budget Solved  |
-------------------------------------------------------------------------
| Context Volume             | 122,044 bytes       | 232 bytes          |
| Estimated LLM Tokens       | ~30,511 tokens      | ~58 tokens         |
| Source Volume Reduction    | 0%                  | 99.81% Reduction   |
| Resolution Time            | 450 – 1200 ms (shell)| 2.10 ms (in-memory) |
-------------------------------------------------------------------------
```

> **Note on Methodology:** This benchmark represents a single-turn code context resolution query on a 122 KB codebase. Actual token savings depend on workspace size, goal intent, and requested budget preferences.

```bash
# Run the local benchmark against any codebase:
cargo run --bin benchmark -- "/path/to/workspace"
```

---

## 🔗 End-to-End Query Example

When an AI developer tool resolves a question like *"Where is the authentication bug?"*, CPP resolves a connected multi-provider graph rather than a isolated text blob:

```
User Query: "Where is the authentication bug?"
                           │
                           ▼
                  CPP Query (goal.code)
                           │
       ┌───────────────────┼───────────────────┐
       ▼                   ▼                   ▼
 Git Provider       Jira Provider       Slack Provider
 ├─ Branch: main    ├─ Issue: AUTH-104  ├─ Msg: "auth fix"
 └─ Commit: f4a291  └─ Status: Blocked  └─ Channel: #dev
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                           ▼
            Unified Semantic Context Graph
  [Branch: main] ──(references)──▶ [Issue: AUTH-104]
  [Issue: AUTH-104] ──(associated_with)──▶ [Slack: #dev]
  [Commit: f4a291] ──(modifies)──▶ [File: auth_provider.rs]
                           │
                           ▼
   Delivered to LLM (Budget Solved: 58 Semantic Objects)
```

---

## ⚡ Benefits for Context Providers & SaaS Platforms

Why should platforms (GitHub, Jira, Slack, Linear, Notion, Local Filesystems) implement a CPP `ContextProvider`?

1. **Implement Once, Work Everywhere:** One CPP adapter connects your platform to Cursor, Claude Code, GitHub Copilot, OpenAI Codex, and custom AI agents.
2. **Source-Side Budget Enforcement:** Protect your API rate limits by downsampling and filtering context server-side before payload transmission.
3. **Relational Context Graph:** Expose rich relational edges (`DependsOn`, `CreatedBy`, `PrecededBy`) rather than flat text.
4. **Real-time Event Streaming:** Push changes (`cpp/event`) over WebSockets instead of answering high-frequency client polling requests.
5. **Fine-Grained Capability Control:** Enforce security policies (`MetadataOnly`, `Summarize`, `FullRead`) per token or session.

---

## 💡 Core Concepts

### 1. Semantic Context Object (SCO)

Every piece of context in CPP is represented as a globally addressable, typed, permissioned, temporally-aware unit:

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
# Visual Dashboard available at http://localhost:3030
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

Exposes three tools: `cpp_query`, `cpp_resolve`, and `cpp_capabilities`.

---

## 🗺️ Project Roadmap

- [x] **v0.1 — Core Specification & Engine**
  - [x] RFC-0000 Philosophy & RFC-0001 Protocol Specifications
  - [x] Core Rust Crate Ecosystem (`cpp-core`, `cpp-protocol`, `cpp-runtime`, `cpp-server`)
  - [x] Local Providers (Filesystem, Git, Datetime)
  - [x] Python SDK (`cpp_sdk`) & MCP-to-CPP Bridge (`mcp_bridge.py`)
  - [x] SaaS Providers (GitHub, Jira, Slack)
- [ ] **v0.2 — Enterprise & Distributed Context**
  - [ ] stdio Transport Adapter (`cpp-transport-stdio`)
  - [ ] Multi-tenant Authentication Tokens (`CapabilityToken` verification)
  - [ ] Vector Index Provider Integration (Qdrant, Pinecone, LanceDB)
  - [ ] TypeScript / Node.js SDK
- [ ] **v1.0 — Ecosystem Standardization**
  - [ ] Finalized Stable RFC
  - [ ] Official Browser Extension & Agent Plugins
  - [ ] Multi-Language Provider Test Suite

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
│   └── cpp-transport-stdio/           #   stdio transport adapter
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
