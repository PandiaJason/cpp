# Context Provider Protocol (CPP) — Draft 0.1

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

**Context Provider Protocol (CPP)** is a new, open standard that defines the semantic context layer for AI. 

Just as the Model Context Protocol (MCP) answers:
> *"How can an AI perform an action? (Tools)"*

CPP answers a different, more fundamental question:
> *"What context should an AI know before reasoning or taking action? (Context)"*

```text
  HTTP  ──────> Documents
  OAuth ──────> Identity
  MCP   ──────> Tools (Action)
  CPP   ──────> Context (Perception)
```

---

## 1. Core Philosophy

CPP is built on the premise that raw data and static knowledge are not context.

*   **Data** is a stored fact (e.g., a database row). It has no situational awareness.
*   **Knowledge** is a persistent truth (e.g., Rust programming rules, static documentation). It does not belong to a session and does not expire.
*   **Context** is a dynamic, session-aware subset of information that is **relevant to a specific agent, at a specific moment, for a specific purpose**.

### The Perception–Reasoning–Action Loop

Every intelligent agent operates in a continuous loop:

```text
┌─────────────────────────────────────────┐
│           The AI Agent Loop             │
│                                         │
│   CPP (Perceive)                        │
│     │                                   │
│     ▼                                   │
│   LLM (Reason)                          │
│     │                                   │
│     ▼                                   │
│   MCP (Act)                             │
└─────────────────────────────────────────┘
```

1.  **Perceive (CPP)**: The agent gathers situated, temporal, and relevant context about its environment.
2.  **Reason (LLM)**: The agent processes the context and decides what to do.
3.  **Act (MCP)**: The agent executes tools to read/write state and perform actions.

---

## 2. Key Abstractions

### 2.1 The Semantic Context Object (SCO)
The fundamental atomic unit of the protocol. An SCO is an envelope carrying:
*   **URI**: Globally addressable reference (`cpp://provider/class.type/path`).
*   **Type**: MIME-like registry type preventing namespace collisions (`application/cpp.document.file`).
*   **Certainty**: Replaces float confidence scores with semantic categories (`Authoritative`, `Derived`, `Estimated`).
*   **Freshness**: Declarative metadata telling the agent how current the data is (`Live`, `Recent`, `Cached`, `Immutable`).
*   **Lifecycle**: Active state tracker (`Created`, `Updated`, `Merged`, `Archived`, `Expired`, `Deleted`).

### 2.2 Goal Registry
Frees queries from custom string fragmentation. The agent requests context by specifying registered goals (e.g., `goal.code`, `goal.project`) rather than vendor-specific paths.

### 2.3 Context Window Negotiation
Allows agents to request context under strict token and latency budgets:
```json
{
  "maxBytes": 128000,
  "maxObjects": 50,
  "prefer": "quality"
}
```
The runtime automatically downsamples and returns the most relevant subset within the budget limits.

### 2.4 Graph Relationships & Cross-System Edges
SCOs define directed, weighted relationships. Because targets are specified as `ContextUri`s, the relationship graph can traverse across different providers (e.g., a Google Calendar meeting referencing a GitHub repository).

---

## 3. Crate Architecture

The workspace is organized to keep the **protocol specification** completely isolated from any **runtime implementation details**:

```text
/
├── spec/                        # PROTOCOL SPECIFICATION (spec only)
│   ├── RFC-0000-Philosophy.md
│   └── RFC-0001-CPP.md
├── crates/
│   ├── cpp-core/                # Primitives & Types (spec-compliant)
│   ├── cpp-protocol/            # JSON-RPC 2.0 Wire Messages & Transport
│   ├── cpp-sdk/                 # Client/Provider traits & Adapter interfaces
│   └── cpp-runtime/             # REFERENCE ORCHESTRATOR RUNTIME (not spec)
├── providers/                   # Reference Providers
│   ├── filesystem/              # Directory and file scanning
│   ├── git/                     # Repository status, branches, commits
│   └── datetime/                # System timezone, local, and UTC times
└── examples/
    ├── simple-query/            # Resolution CLI demo
    └── streaming/               # Streaming event loop stub
```

---

## 4. Running the Demo

The `simple-query` binary demonstrates the complete perception resolution pipeline. It registers all three reference providers and executes a Context Request Query (CRQ) targeting your workspace.

To run it on the CPP workspace itself:
```bash
cargo run --bin simple-query
```

To run it on any other directory (e.g., another project codebase or workspace):
```bash
cargo run --bin simple-query -- "/absolute/path/to/project"
```

### Running Tests
To verify all crates, unit tests, and doc-tests:
```bash
cargo test --workspace
```

---

## 5. License

This project is licensed under the MIT License - see the [LICENSE-MIT](LICENSE-MIT) file for details.
Copyright © 2026.
