# Context Provider Protocol (CPP)

> **The open-standard perception layer for AI systems.**

[![Protocol](https://img.shields.io/badge/Protocol-v0.1.0-8B5CF6.svg)](spec/RFC-0001-CPP.md)
[![RFC-0000](https://img.shields.io/badge/RFC-Philosophy-blue.svg)](spec/RFC-0000-Philosophy.md)
[![License](https://img.shields.io/badge/License-MIT-22C55E.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-F74C00.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10%2B-3776AB.svg?logo=python&logoColor=white)](https://python.org/)
[![Tests](https://img.shields.io/badge/Tests-69%20passing-22C55E.svg)]()

---

## What is CPP?

CPP is an open protocol that defines how AI systems **discover, filter, and deliver structured context before reasoning begins**.

Every AI coding agent — whether it's Cursor, Claude Code, GitHub Copilot, or Gemini CLI — needs to understand your codebase before it can help you. Today, most agents do this by dumping raw file contents and terminal output into the prompt. This is expensive, slow, and lossy.

CPP solves this by providing a **standardized perception layer** that sits between your data sources and the LLM. It resolves context from multiple providers (filesystem, Git, GitHub, Jira, Slack), ranks it by relevance, enforces a token budget, and delivers a clean, structured bundle — all before the model sees a single token.

### Where CPP fits

```
┌──────────────────────────────────────────────────────┐
│                    AI Agent Stack                     │
├──────────────────────────────────────────────────────┤
│                                                      │
│   CPP    →  Perceive    "What do I need to know?"    │
│   LLM    →  Reason      "What should I do?"          │
│   MCP    →  Act          "Execute this action."      │
│                                                      │
└──────────────────────────────────────────────────────┘
```

CPP complements the [Model Context Protocol (MCP)](https://modelcontextprotocol.io). MCP standardizes how AI systems **act** (tool execution, file edits, mutations). CPP standardizes how AI systems **perceive** (context discovery, budget enforcement, relational graphs).

| | MCP | CPP |
|:--|:--|:--|
| **Role** | Execute tools and mutations | Resolve structured context |
| **Core operation** | `tools/call` | `cpp/query` |
| **Prompt** | *"Do this action."* | *"What should I know before acting?"* |

---

## Why CPP exists

Before CPP, AI tools gathered workspace context using ad-hoc mechanisms that break at scale:

1. **Prompt stuffing.** Reading whole files fills the context window, triggers "lost in the middle" degradation, and inflates API costs.
2. **Flat retrieval.** Vector-based RAG ranks by text similarity alone. It cannot preserve structured relationships like `Branch → Commit → Issue → PR → File`.
3. **Unstructured tool output.** MCP tool calls return raw text blobs that the LLM must re-parse on every turn.
4. **Polling loops.** Agents run `while true` loops to detect workspace changes, wasting time and compute.

> **How is CPP different from RAG?**
>
> RAG retrieves text snippets ranked by embedding similarity. CPP resolves *structured context objects* with typed relationships, metadata (certainty, freshness, importance), and enforced token budgets. They address different layers of the problem.

---

## How it works

### Protocol sequence

```
  AI Client                    CPP Server                    Context Providers
  (Cursor, Claude, etc.)       (localhost:3030)              (Git, Filesystem, Jira...)
     │                              │                               │
     │ ─── cpp/initialize ────────▶ │                               │
     │ ◀── capabilities & session ─ │                               │
     │                              │                               │
     │ ─── cpp/query ─────────────▶ │ ─── resolve from providers ─▶ │
     │                              │ ◀── raw context objects ───── │
     │                              │                               │
     │                              │ ── rank, filter, budget ──    │
     │ ◀── ContextBundle (SCOs) ─── │                               │
     │                              │                               │
     │ ─── cpp/subscribe ─────────▶ │ ─── register event watch ──▶  │
     │ ◀── cpp/event (push) ─────── │ ◀── file/git change event ── │
```

### API methods

All communication uses **JSON-RPC 2.0** over HTTP or WebSocket.

| Method | Type | Description |
|:--|:--|:--|
| `cpp/initialize` | Request | Session handshake and capability negotiation |
| `cpp/initialized` | Notification | Client confirms initialization |
| `cpp/query` | Request | Query the context graph with filters and budget |
| `cpp/resolve` | Request | Fetch a single object by its `cpp://` URI |
| `cpp/capabilities` | Request | List server capabilities |
| `cpp/providers/list` | Request | List registered context providers |
| `cpp/subscribe` | Request | Subscribe to WebSocket event notifications |
| `cpp/unsubscribe` | Request | Cancel a subscription |
| `cpp/publish` | Request | Publish an event to the event bus |
| `cpp/event` | Notification | Server-to-client push notification |
| `cpp/shutdown` | Request | Graceful session teardown |
| `cpp/exit` | Notification | Final termination signal |

---

## Core concepts

### Semantic Context Objects (SCOs)

Every piece of context in CPP is a **Semantic Context Object** — a structured, globally addressable unit of knowledge with metadata:

```json
{
  "uri": "cpp://git/branch/main",
  "contextType": "application/cpp.entity.branch",
  "providerId": "git",
  "certainty": "authoritative",
  "freshness": { "kind": "live" },
  "importance": 90,
  "title": "main"
}
```

Each SCO has:
- A **URI** (`cpp://provider/type/path`) — globally unique, no ambiguity.
- A **context type** — structured MIME taxonomy (see below).
- **Certainty** — `authoritative`, `derived`, or `estimated`.
- **Freshness** — `live`, `recent`, `cached`, or `immutable`.
- **Importance** — provider-declared priority score (0–100).

### Context type taxonomy

CPP uses a 3-level MIME hierarchy:

```
application/<namespace>.<class>.<type>
```

- **Protocol types** (`application/cpp.*`): Standardized types defined in the RFC. Examples: `application/cpp.document.file`, `application/cpp.entity.commit`, `application/cpp.temporal.timestamp`.
- **Vendor extensions** (`application/<vendor>.*`): Third-party types that require no central approval. Examples: `application/github.entity.pull_request`, `application/notion.document.database`, `application/docker.entity.container`.

### Context Query Language (CQL)

Clients query context using structured filters:

- **Goal intents**: `goal.code`, `goal.project`, `goal.document`, `goal.calendar`.
- **Budget constraints**: `maxBytes`, `maxObjects`, `prefer` (quality vs. quantity).
- **Filters**: by provider, context type, minimum certainty, freshness.

### Source-side budget solver

This is CPP's key architectural idea. Instead of dumping everything to the LLM and hoping it fits, CPP **ranks and trims context at the source** before transmission.

```
  Without CPP:
  100 files → 122 KB raw text → LLM prompt (~30,000 tokens)

  With CPP:
  100 files → Budget solver ranks & filters → 232 bytes → LLM prompt (~58 tokens)
```

The solver scores each candidate object:

$$\text{Score}(u) = w_i \cdot \text{Importance}(u) + w_r \cdot \text{Relevance}(u) + w_c \cdot \text{Certainty}(u) + w_f \cdot \text{Freshness}(u)$$

subject to:

$$\sum_{u \in S} \text{bytes}(u) \le \text{maxBytes}, \quad |S| \le \text{maxObjects}$$

### Relational context graph

CPP doesn't return a flat list of results. It returns a **graph** of typed relationships between context objects:

```
[Branch: main] ──(references)──▶ [Issue: AUTH-104]
[Issue: AUTH-104] ──(associated_with)──▶ [Slack: #dev]
[Commit: f4a291] ──(modifies)──▶ [File: auth.rs]
```

This lets agents understand *why* files, issues, and messages are connected — not just that they matched a search query.

---

## Example response

A real `cpp/query` response from the running server:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "totalCount": 2,
    "resolutionTimeMs": 4,
    "providers": ["git", "filesystem"],
    "objects": [
      {
        "uri": "cpp://git/branch/main",
        "id": "sco_9f8a12",
        "contextType": "application/cpp.entity.branch",
        "providerId": "git",
        "certainty": "authoritative",
        "freshness": { "kind": "live" },
        "importance": 90,
        "title": "main",
        "relations": [
          { "relationType": "references", "targetUri": "cpp://jira/issue/AUTH-104" }
        ]
      },
      {
        "uri": "cpp://filesystem/file/src/auth.rs",
        "id": "sco_3b7c41",
        "contextType": "application/cpp.document.file",
        "providerId": "filesystem",
        "certainty": "authoritative",
        "freshness": { "kind": "live" },
        "importance": 85,
        "title": "auth.rs",
        "summary": "Authentication token verification module"
      }
    ],
    "graph": {
      "nodes": ["sco_9f8a12", "sco_3b7c41"],
      "edges": [
        { "source": "sco_9f8a12", "target": "sco_3b7c41", "edgeType": "references" }
      ],
      "cycleDetected": false
    }
  }
}
```

---

## Benchmark

Measured on the CPP codebase workspace, comparing an unbudgeted file scan against CPP's budget solver with a 4 KB budget:

| Metric | Raw file scan | CPP budget-solved |
|:--|--:|--:|
| Context volume | 122,044 bytes | 232 bytes |
| Estimated LLM tokens | ~30,511 | ~58 |
| Volume reduction | — | 99.81% |
| Resolution time | 450–1,200 ms (shell) | 2.1 ms (in-memory) |

> **Note:** These numbers are from a single workspace. Actual savings depend on workspace size, query goal, and budget configuration.

```bash
cargo run --bin benchmark -- "/path/to/workspace"
```

---

## Feature comparison

| Feature | Traditional RAG | MCP | CPP |
|:--|:--:|:--:|:--:|
| Executes system tools | ❌ | ✅ Primary role | ❌ |
| Structured context perception | ❌ | Partial (Resources) | ✅ Primary role |
| Source-side token budgeting | ❌ | ❌ | ✅ Built-in |
| Relational context graph | ❌ | ❌ | ✅ Typed edges |
| Semantic type taxonomy | ❌ | Partial | ✅ 36 standard types |
| Real-time push notifications | ❌ | Partial (Subscriptions) | ✅ WebSocket event bus |

---

## Non-goals

CPP has clear architectural boundaries. It does **not**:

- **Execute actions.** No file writes, terminal commands, or API mutations. That's MCP's job.
- **Manage agents.** No loops, prompt templates, or tool selection. Use LangChain, AutoGen, or custom agents.
- **Store embeddings.** CPP is a real-time resolution protocol, not a vector database.
- **Lock you to a language.** The spec is transport-agnostic (JSON-RPC 2.0 over HTTP, WebSocket, or stdio). This repo's Rust/Python implementation is a reference, not the standard.

---

## Specification vs. implementation

CPP separates the **protocol specification** (what any implementation must do) from this **reference implementation** (one way to do it):

**Protocol specification** (defined in RFCs):
- Semantic Context Object (SCO) schema and lifecycle
- Context Query Language (CQL) and goal registry
- 3-level MIME taxonomy (`application/cpp.<class>.<type>`)
- Context budget model (`maxBytes`, `maxObjects`, `prefer`)
- Relational context graph (nodes, edges, weights)
- JSON-RPC 2.0 method schemas and error codes (`-32000` to `-32009`)

**Reference implementation** (this repository):
- Rust core engine (`cpp-core`, `cpp-protocol`, `cpp-runtime`, `cpp-server`)
- Python async SDK (`cpp_sdk`) and MCP-to-CPP bridge (`mcp_bridge.py`)
- Built-in providers: Filesystem, Git, Datetime, GitHub, Jira, Slack

### Conformance requirements

For an independent implementation to be CPP-compliant:

- **Must** serialize the exact SCO JSON fields and 3-level MIME taxonomy.
- **Must** enforce `maxBytes` and `maxObjects` budget limits before transmitting bundles.
- **Must** use the standard JSON-RPC 2.0 error codes (`-32000` to `-32009`).
- **May** freely choose internal indexing, storage, and ranking algorithms.

---

## For platform integrators

If you maintain a developer platform (GitHub, Jira, Slack, Linear, Notion), building a CPP provider adapter gives you:

1. **Write once, connect everywhere.** One adapter works with every CPP-compliant AI client.
2. **Protect your API.** The budget solver limits how much data leaves your service per query.
3. **Express relationships.** Expose typed edges (`DependsOn`, `CreatedBy`, `Blocks`) instead of flat text.
4. **Push, don't poll.** Deliver updates over WebSocket instead of handling polling requests.

```
 AI Clients                       CPP Server                       Providers
┌─────────────┐                                                   ┌────────────┐
│ Cursor      │ ──┐                                         ┌──── │ GitHub     │
│ Claude Code │ ──┼──▶  CPP Engine (JSON-RPC 2.0 / WS)  ◀──┼──── │ Jira       │
│ Copilot     │ ──┘                                         └──── │ Slack      │
└─────────────┘                                                   └────────────┘
```

---

## Repository structure

```
context-provider-protocol/
├── spec/                          # Protocol specifications
│   ├── RFC-0000-Philosophy.md     #   Design principles
│   └── RFC-0001-CPP.md            #   Full protocol specification
│
├── crates/                        # Rust core engine
│   ├── cpp-core/                  #   SCO, URI, budget solver, graph, permissions
│   ├── cpp-protocol/              #   JSON-RPC 2.0 wire format and schemas
│   ├── cpp-sdk/                   #   ContextProvider trait and CppClient
│   ├── cpp-runtime/               #   ContextResolver and ContextCache
│   ├── cpp-server/                #   Axum HTTP/WebSocket server daemon
│   ├── cpp-transport-http/        #   HTTP transport layer
│   └── cpp-transport-stdio/       #   stdio transport adapter
│
├── providers/                     # Context providers
│   ├── filesystem/                #   Files and directories (Rust)
│   ├── git/                       #   Repositories, branches, commits (Rust)
│   ├── datetime/                  #   System time and temporal context (Rust)
│   ├── github/                    #   Pull requests, issues, commits (Python)
│   ├── jira/                      #   Sprint issues, epics, blockers (Python)
│   └── slack/                     #   Channels and threaded messages (Python)
│
├── sdks/python/                   # Python SDK
│   ├── cpp_sdk/                   #   Pydantic v2 models, async client, MCP bridge
│   └── tests/                     #   14 serialization round-trip tests
│
└── examples/                      # Demos and benchmarks
    ├── simple-query/              #   CLI context resolution demo
    ├── benchmark/                 #   Budget solver benchmark
    └── streaming/                 #   WebSocket event subscription demo
```

---

## Getting started

### Run the server

```bash
cargo build --release
cargo run --bin cpp-server
# Server starts on http://localhost:3030
```

### Query context

```bash
curl -s http://localhost:3030/api/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "cpp/query",
    "params": {
      "goal": "goal.code",
      "budget": { "maxBytes": 4096, "maxObjects": 10 }
    }
  }'
```

### Run tests

```bash
# Rust — 55 tests
cargo test --workspace

# Python — 14 tests
cd sdks/python && uv run python -m pytest tests/ -v
```

---

## Roadmap

- [x] **v0.1 — Core specification and engine**
  - [x] RFC-0000 Philosophy and RFC-0001 Protocol Specification
  - [x] Rust crate ecosystem (cpp-core, cpp-protocol, cpp-runtime, cpp-server)
  - [x] Local providers (Filesystem, Git, Datetime)
  - [x] Python SDK and MCP-to-CPP bridge
  - [x] SaaS providers (GitHub, Jira, Slack)
- [ ] **v0.2 — Enterprise and distributed context**
  - [ ] RFC-0002: Provider capability and relation registry
  - [ ] RFC-0003: Deterministic budget solver and scoring normalization
  - [ ] RFC-0004: Formal SCO schema and validation rules
  - [ ] RFC-0005: Vendor namespace and extension registry
  - [ ] RFC-0006: Cross-implementation conformance test suite
  - [ ] stdio transport adapter
  - [ ] Multi-tenant authentication tokens
  - [ ] Vector index provider integration (Qdrant, Pinecone, LanceDB)
  - [ ] TypeScript / Node.js SDK
- [ ] **v1.0 — Ecosystem standardization**
  - [ ] Finalized stable RFC specifications
  - [ ] Browser extension and agent plugins
  - [ ] Multi-language conformance test suite (Go, Rust, Python, TypeScript)

---

## Specifications

| Document | Description |
|:--|:--|
| [RFC-0000 — Philosophy](spec/RFC-0000-Philosophy.md) | Design principles: structured perception, budget negotiation, semantic typing |
| [RFC-0001 — Protocol Specification](spec/RFC-0001-CPP.md) | Complete spec: wire format, methods, types, transports, error codes |

---

## License

MIT © [CPP Contributors](LICENSE-MIT)
