# Context Provider Protocol (CPP)

> **The open-standard perception layer for AI systems.**
>
> *CPP standardizes how AI systems discover, filter, relate, and deliver context before reasoning, complementing MCP’s execution layer.*

[![Protocol](https://img.shields.io/badge/Protocol-v0.1.0-8B5CF6.svg)](spec/RFC-0001-CPP.md)
[![RFC-0000](https://img.shields.io/badge/RFC-Philosophy-blue.svg)](spec/RFC-0000-Philosophy.md)
[![License](https://img.shields.io/badge/License-MIT-22C55E.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-F74C00.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10%2B-3776AB.svg?logo=python&logoColor=white)](https://python.org/)
[![Tests](https://img.shields.io/badge/Tests-69%20passing-22C55E.svg)]()

CPP is an open protocol that **standardizes how AI systems discover, prioritize, and exchange structured context before reasoning**. It provides AI assistants, autonomous coding agents, and IDEs (Cursor, Claude Code, GitHub Copilot, OpenAI Codex, Gemini CLI) with a budget-aware perception layer.

CPP complements the **Model Context Protocol (MCP)**: while MCP standardizes how AI systems **act** (tool execution, file edits, mutations), CPP standardizes how AI systems **perceive** (situated context resolution, token-budget enforcement, and relational graphs).

---

## 🏛️ The AI Execution Stack

```
             HUMAN                          AI SYSTEM
        ┌─────────────┐                 ┌──────────────┐
        │    Sees     │                 │     CPP      │  Perceives (Context & Graph)
        ├─────────────┤                 │ (Perception) │  "What should I know?"
        │ Understands │                 ├──────────────┤
        ├─────────────┤  ─────────────▶ │     LLM      │  Reasons (Plan & Strategy)
        │    Acts     │                 │ (Reasoning)  │  "What should I plan?"
        └─────────────┘                 ├──────────────┤
                                        │     MCP      │  Acts (Tools & Edits)
                                        │ (Execution)  │  "Do this action."
                                        └──────────────┘
```

### MCP vs. CPP Responsibilities

| Axis | Model Context Protocol (MCP) | Context Provider Protocol (CPP) |
|:-----|:-----------------------------|:--------------------------------|
| **Primary Role** | Executes tool capabilities & mutations | Resolves structured context & graph relations |
| **Core Paradigm** | Tool invocation & action execution | Context discovery & budget negotiation |
| **RPC Framing** | RPC for actions (`tools/call`) | RPC for context (`cpp/query`, `cpp/resolve`) |
| **Domain Prompt** | *"Do this action."* | *"What should I know before doing this?"* |

---

## 🔁 Protocol Sequence & Event Flow

```
  AI Client                   CPP Server Daemon                Context Providers
  (Cursor/Claude)             (Axum / Router)                 (Git / Filesystem / Jira)
     │                               │                                │
     │ ────── cpp/initialize ──────▶ │                                │
     │ ◀───── Session & Specs ────── │                                │
     │                               │                                │
     │ ────── cpp/query (CRQ) ─────▶ │ ───── Resolve Query (CRQ) ───▶ │
     │                               │ ◀──── Raw Context Objects ──── │
     │                               │                                │
     │                               │ ─── [ Source-Side Solver ] ─── │
     │                               │     Ranks by Score & Budget    │
     │ ◀───── ContextBundle (SCOs) ─ │                                │
     │                               │                                │
     │ ────── cpp/subscribe ───────▶ │ ────── Register Event WS ────▶ │
     │ ◀───── cpp/event (Push) ───── │ ◀───── File/Git Event Push ─── │
     │                               │                                │
```

---

## ❓ Why Existing Approaches Fail

Before CPP, AI tools gathered workspace context using ad-hoc mechanisms that struggle at scale:

1. **Prompt Stuffing:** Reading whole files and terminal outputs fills the context window, triggers "lost in the middle" degradation, and inflates API costs.
2. **Traditional Vector RAG:** Vector-based RAG primarily ranks by text similarity and requires custom application logic to preserve complex software relationships (`Branch` $\rightarrow$ `Commit` $\rightarrow$ `Issue` $\rightarrow$ `PR` $\rightarrow$ `File`).
3. **Unstructured Tool Output:** MCP tool execution returns raw text blobs that must be parsed repeatedly by the LLM.
4. **Passive Polling Loops:** Agents run continuous `while true` polling loops to detect workspace changes, wasting time and compute.

> **How does CPP differ from RAG?**
>
> *CPP addresses a different layer than traditional text retrieval. It standardizes structured context exchange, budget negotiation, and graph relations across tools, whereas RAG techniques focus primarily on retrieving text snippets by similarity.*

---

## ⚔️ Protocol Feature Matrix

| Feature | Traditional RAG | Model Context Protocol (MCP) | Context Provider Protocol (CPP) |
|:--------|:---------------:|:---------------------------:|:-------------------------------:|
| **Executes System Tools** | ❌ | **✅ Primary Role** | ❌ *(Delegated to MCP)* |
| **Structured Context Perception** | ❌ | Partial *(Resources)* | **✅ Primary Role** |
| **Source-Side Token Budgeting** | ❌ | ❌ | **✅ Built-in Solver** |
| **Relational Context Graph** | ❌ | ❌ | **✅ Typed Nodes & Edges** |
| **Semantic Type Taxonomy** | ❌ | Partial | **✅ 36 Standard MIME Types** |
| **Real-time Push Notifications** | ❌ *(CDC/Polling)* | Partial *(Resource Subscriptions)* | **✅ WebSocket Event Bus** |

---

## 🧮 The Source-Side Budget Solver

CPP's primary architectural innovation is **source-side budget enforcement**:

```
               TRADITIONAL APPROACH (CLIENT-SIDE TRUNCATION)
┌─────────────────┐       Raw Dump (100 Files)        ┌─────────────────┐
│   Data Sources  │ ────────────────────────────────▶ │    LLM Model    │
└─────────────────┘       122 KB / 30,000+ Tokens     └─────────────────┘

                 CPP APPROACH (SOURCE-SIDE SOLVER)
┌─────────────────┐    Source-Side Solver     ┌─────────────┐    Top 8 SCOs    ┌──────────┐
│   Data Sources  │ ──▶ [Ranks & Downsamples] ──▶ │ CPP Server  │ ─────────────▶ │ LLM Model│
└─────────────────┘     Max Budget: 4,096 B   └─────────────┘    232 Bytes     └──────────┘
```

### Objective Optimization Function

The budget solver evaluates each candidate object $u$ using a multi-attribute utility score:

$$\text{Score}(u) = w_i \cdot \text{Importance}(u) + w_r \cdot \text{Relevance}(u) + w_c \cdot \text{Certainty}(u) + w_f \cdot \text{Freshness}(u)$$

$$\text{subject to } \sum_{u \in S} \text{bytes}(u) \le \text{maxBytes}, \quad |S| \le \text{maxObjects}, \quad \text{Permissions}(u) \ge \text{AccessLevel}$$

* **Importance ($w_i$):** Domain priority declared by provider (0..100).
* **Relevance ($w_r$):** Semantic relevance to requested goal intent.
* **Certainty ($w_c$):** Trust classification (`Authoritative`, `Derived`, `Estimated`).
* **Freshness ($w_f$):** Temporal currency (`Live`, `Recent`, `Cached`, `Immutable`).

---

## 🔗 End-to-End Query & Real JSON Payload

When an AI tool asks *"Where is the authentication bug?"*, CPP resolves a connected multi-provider graph:

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
```
*(Illustrative Graph Trace across GitHub, Jira, and Slack providers)*

### Real JSON Response Payload (`cpp/query`)
*(Captured live from running `cpp-server` daemon via cURL)*

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

## 📈 Empirical Repository Benchmark (Methodology & Results)

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

> **Methodology Note:** Benchmark run on the CPP codebase workspace comparing an unbudgeted file scan against CPP's budget-enforced context solver with a 4 KB budget. Actual savings vary based on workspace size, goal intent, and client budget preferences.

```bash
cargo run --bin benchmark -- "/path/to/workspace"
```

---

## 🧩 Protocol Specification vs. Reference Implementation

To maintain standard neutrality, CPP clearly separates specification from implementations:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        PROTOCOL SPECIFICATION                          │
├────────────────────────────────────────────────────────────────────────┤
│  • Semantic Context Object (SCO) Schema & Lifecycle                    │
│  • Context Query Language (CQL) & Goal Registry                        │
│  • 3-Level MIME Taxonomy (application/cpp.<class>.<type>)             │
│  • Context Budget Model (maxBytes, maxObjects, prefer)                 │
│  • Relational Context Graph Engine (Nodes, Edges, Weights)             │
│  • JSON-RPC 2.0 API Schemas & Error Codes                              │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │ Implemented By
                                   ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        REFERENCE IMPLEMENTATION                        │
├────────────────────────────────────────────────────────────────────────┤
│  • Rust Core Engine (cpp-core, cpp-protocol, cpp-runtime, cpp-server)  │
│  • Python Async SDK (cpp_sdk) & MCP-to-CPP Bridge (mcp_bridge.py)      │
│  • Built-in Providers (Filesystem, Git, Datetime, GitHub, Jira, Slack) │
└────────────────────────────────────────────┬───────────────────────────┘
                                             │
### Interoperability & Conformance Guarantees

For any independent CPP server or client implementation to achieve protocol compliance:

- **Mandatory Wire & Schemas:** Must serialize the exact `ContextObject` (SCO) JSON fields, 3-level MIME taxonomy (`application/cpp.<class>.<type>`), and JSON-RPC 2.0 error codes (`-32000` to `-32009`).
- **Mandatory Budget Enforcement:** Servers must strictly enforce `maxBytes` and `maxObjects` upper bounds before transmitting context bundles to clients.
- **Implementation Freedom:** Internal indexing algorithms, vector embeddings, provider search strategies, and storage backends are left to server implementation freedom.

---

## 🚫 Non-Goals

To maintain clean architectural boundaries, CPP explicitly declares what it does **not** do:

* **Not an LLM Agent Framework:** CPP does not execute loops, select tools, or manage prompt templates (use LangChain, AutoGen, or custom agents).
* **Not an Execution Layer:** CPP does not perform file writes, terminal execution, or API mutations (use **MCP**).
* **Not a Vector Database:** CPP is a real-time semantic context resolution protocol, not a static vector embedding store.
* **Not Tied to a Single Language:** The spec is transport-agnostic (JSON-RPC 2.0 over HTTP/WS/stdio) and not locked to Rust or Python.

---

## ⚡ Benefits for Context Providers & SaaS Platforms

Why should platforms (GitHub, Jira, Slack, Linear, Notion) build a CPP adapter?

```
 AI Clients                   Universal Perception Protocol               Providers
┌─────────────┐                                                          ┌────────────┐
│ Cursor      │ ──────┐                                          ┌────── │ GitHub     │
├─────────────┤       │          ┌──────────────────────┐        │       ├────────────┤
│ Claude Code │ ──────┼────────▶ │       CPP Engine     │ ───────┼────── │ Jira       │
├─────────────┤       │          │ (JSON-RPC 2.0 / WS)  │        │       ├────────────┤
│ Copilot /   │ ──────┘          └──────────────────────┘        └────── │ Slack      │
│ Codex       │                                                          └────────────┘
└─────────────┘
```

1. **Implement Once, Work Everywhere:** One CPP adapter connects your data source to all compliant AI IDEs, CLI tools, and agents.
2. **Protect API Rate Limits:** Source-side budget solver downsamples context before payload transmission.
3. **Rich Context Graph:** Expose typed relationships (`DependsOn`, `CreatedBy`, `PrecededBy`) rather than flat text.
4. **Real-time Push Notifications:** Push updates over WebSockets (`cpp/event`) instead of handling client polling requests.

---

### 🌐 Extensibility & Vendor Namespaces

To ensure long-term stability without central bottlenecking, CPP uses a 3-level taxonomy with vendor namespaces:

```
application/<namespace>.<class>.<type>
```

- **Protocol Reserved (`application/cpp.*`):** Reserved strictly for standardized RFC types (e.g., `application/cpp.document.file`, `application/cpp.entity.commit`).
- **Vendor Extensions (`application/<vendor>.*`):** Third-party platforms can introduce proprietary types following the 3-level hierarchy without central protocol approval (e.g., `application/github.entity.pull_request`, `application/notion.document.database`, `application/gitlab.entity.merge_request`, `application/docker.entity.container`).

---

## 🗺️ Project Roadmap

- [x] **v0.1 — Core Specification & Engine**
  - [x] RFC-0000 Philosophy & RFC-0001 Protocol Specifications
  - [x] Core Rust Crate Ecosystem (`cpp-core`, `cpp-protocol`, `cpp-runtime`, `cpp-server`)
  - [x] Local Providers (Filesystem, Git, Datetime)
  - [x] Python SDK (`cpp_sdk`) & MCP-to-CPP Bridge (`mcp_bridge.py`)
  - [x] SaaS Providers (GitHub, Jira, Slack)
- [ ] **v0.2 — Enterprise & Distributed Context**
  - [ ] RFC-0002 Provider Capability & Relation Registry Specification
  - [ ] RFC-0003 Deterministic Budget Solver & Scoring Normalization
  - [ ] RFC-0004 Formal SCO Schema & Validation Rules
  - [ ] RFC-0005 Vendor Namespace & Extension Registry
  - [ ] RFC-0006 Cross-Implementation Conformance Test Suite
  - [ ] stdio Transport Adapter (`cpp-transport-stdio`)
  - [ ] Multi-tenant Authentication Tokens (`CapabilityToken` verification)
  - [ ] Vector Index Provider Integration (Qdrant, Pinecone, LanceDB)
  - [ ] TypeScript / Node.js SDK
- [ ] **v1.0 — Ecosystem Standardization**
  - [ ] Finalized Stable RFC Specifications
  - [ ] Official Browser Extension & Agent Plugins
  - [ ] Multi-Language Conformance Test Suite (Go, Rust, Python, TS)

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
