# Context Provider Protocol (CPP)

**An open protocol that gives AI coding agents the right context — without wasting tokens.**

[![Protocol](https://img.shields.io/badge/Protocol-v0.1.0-8B5CF6.svg)](spec/RFC-0001-CPP.md)
[![License](https://img.shields.io/badge/License-MIT-22C55E.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-F74C00.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/Python-3.10%2B-3776AB.svg?logo=python&logoColor=white)](https://python.org/)
[![Tests](https://img.shields.io/badge/Tests-69%20passing-22C55E.svg)]()

---

## The problem

AI coding agents (Cursor, Claude Code, Copilot, Gemini CLI) need to understand your codebase before they can help you. Today, most agents do this by dumping raw file contents into the prompt — thousands of lines the model has to read, most of which are irrelevant.

This causes three real problems:

- **Wasted tokens.** You pay for the model to process code it doesn't need.
- **Slower responses.** More input tokens = longer time-to-first-token.
- **Worse answers.** Important context gets buried in noise ("lost in the middle" effect).

## The solution

CPP is a lightweight local server that sits between your data sources and the AI model. When an agent needs context, it sends a query to CPP instead of reading files directly. CPP resolves context from multiple sources, ranks it, trims it to a token budget, and returns only what matters.

```
  Your code, Git history,         CPP Server               AI Model
  Jira tickets, Slack msgs       (localhost:3030)          (GPT, Claude, Gemini)
  ─────────────────────────  →   Rank & Filter   →   Clean, budgeted context
        100+ sources                 2 ms                   ~58 tokens
```

---

## Try it in 60 seconds

```bash
# 1. Clone and build
git clone https://github.com/PandiaJason/cpp.git
cd cpp
cargo build --release

# 2. Start the server
cargo run --bin cpp-server
# → Server running on http://localhost:3030

# 3. Query your workspace context (in another terminal)
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
  }' | python3 -m json.tool
```

You'll get back a structured JSON response with the most relevant files, branches, and commits — ranked by importance, trimmed to your budget.

### What the output looks like

Here's real output from a single `cpp/query` against this repository — **one request, two providers, smart-ranked results**:

```
Resolution: 426 ms | Objects: 13 | Providers: ['filesystem', 'git']

  #  Provider     Imp  Lines  Title
  1  filesystem  1.00    354  provider.rs          ← recently edited, large source file
  2  filesystem  0.85    367  types.py             ← substantial source code
  3  filesystem  0.85    286  client.py            ← substantial source code
  4  filesystem  0.85    232  provider.rs          ← core provider implementation
  5  filesystem  0.85    331  resolver.rs          ← the context resolution engine
  6  filesystem  0.85    305  query.py             ← query logic
  7  git         0.80      -  main                 ← current branch
  8  git         0.80      -  cpp                  ← repository name
  9  git         0.50      -  feat: smart importance scoring...
 10  git         0.50      -  docs: add live multi-source demo...
```

Source code files rank above config files. Recently edited files rank highest. Files with more logic (more lines) get boosted. The budget solver ranked all objects from both providers together and returned only the best.

### CPP vs. standard shell commands

The same context gathered with standard tools requires five separate commands:

```bash
git log --oneline -5                    # commits (raw text)
git branch --show-current               # branch name
find . -name "*.rs" -exec stat ...      # recent Rust files
find . -name "*.py" -exec stat ...      # recent Python files (leaks .venv junk)
find . -exec wc -l ... | sort -rn      # largest files (returns pip packages as "biggest")
```

| | Standard tools | CPP |
|:--|:--|:--|
| **Tool calls** | 5 piped shell commands | 1 JSON request |
| **Time** | ~11 seconds | 426 ms |
| **Caught .venv junk?** | ❌ Returned pip packages as "top source files" | ✅ Automatically skipped |
| **Source code ranked first?** | ❌ Separate outputs, no ranking | ✅ `provider.rs` (1.00) ranked #1 |
| **Git + files unified?** | ❌ Three separate outputs | ✅ Single ranked list |
| **Config files polluting results?** | N/A (manually filtered) | ✅ Scored 0.3, never appeared |

---

## How it fits into the AI stack

CPP works alongside [MCP (Model Context Protocol)](https://modelcontextprotocol.io), not against it. They handle different jobs:

```
  CPP    →  Perceive    "What do I need to know?"
  LLM    →  Reason      "What should I do?"
  MCP    →  Act          "Execute this action."
```

| | MCP | CPP |
|:--|:--|:--|
| **Job** | Run tools, edit files, execute commands | Gather and filter context |
| **Core method** | `tools/call` | `cpp/query` |
| **Question it answers** | *"Do this."* | *"What should I know first?"* |

MCP already solved tool execution. CPP solves the step before it — giving the model the right information so it makes better decisions about what tools to call.

---

## What you get back

When you query CPP, you get structured **Semantic Context Objects (SCOs)** — not raw text dumps:

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

Every object has:
- A **URI** — globally unique address (`cpp://provider/type/path`). No ambiguous file paths.
- **Importance** (0–100) — how relevant this is, set by the provider.
- **Certainty** — `authoritative` (read from source), `derived`, or `estimated`.
- **Freshness** — `live`, `recent`, `cached`, or `immutable`.
- **Relations** — typed links to other objects (e.g., "this commit modifies this file").

CPP also returns a **context graph** showing how objects relate to each other:

```
[Branch: main] ──(references)──▶ [Issue: AUTH-104]
[Commit: f4a291] ──(modifies)──▶ [File: auth.rs]
[Issue: AUTH-104] ──(associated_with)──▶ [Slack: #dev]
```

This lets the agent understand *connections* — not just matching files.

---

## How the budget solver works

This is the core idea. Instead of sending everything to the model, CPP **filters at the source**.

The filesystem provider scans up to 200 candidate files, scores each one using smart heuristics, then keeps only the top-ranked objects that fit within your byte and object count limits:

```
Score = TypeBase + RecencyBoost + SizeBoost - DepthPenalty

TypeBase:      .rs/.py/.ts = 0.70    .toml/.yaml = 0.30    .md/.txt = 0.20
RecencyBoost:  < 1 hour = +0.25     < 1 day = +0.15       < 1 week = +0.05
SizeBoost:     > 200 lines = +0.10  > 50 lines = +0.05
DepthPenalty:  > 5 levels deep = -0.05
```

Source code always outranks config files. Recently edited files always outrank stale ones. Substantial implementation files outrank trivial stubs.

```
Without CPP:  100 files  →  122 KB raw text    →  ~30,000 tokens to the model
With CPP:     100 files  →  solver picks top 8  →  232 bytes / ~58 tokens to the model
```

> **Note:** The ranking uses heuristic scoring, not LLM-based semantic understanding. It can't tell you which file contains the authentication bug — but it will reliably surface source code over config, recent edits over stale files, and substantial modules over one-line stubs. For most coding tasks, that's the right 80% of the answer.

---

## Built-in context providers

CPP ships with providers that scan real data sources on every query (nothing is pre-indexed):

| Provider | What it resolves | Language |
|:--|:--|:--|
| **Filesystem** | Files and directories in your workspace | Rust |
| **Git** | Branches, commits, diffs, repo metadata | Rust |
| **Datetime** | System time and timezone | Rust |
| **GitHub** | Pull requests, issues, commits | Python |
| **Jira** | Sprint issues, epics, blockers | Python |
| **Slack** | Channel messages and threads | Python |

### Writing your own provider

Any service can become a CPP provider. You implement two methods:

```rust
// Rust
#[async_trait]
impl ContextProvider for MyProvider {
    fn manifest(&self) -> &ProviderManifest { /* describe your capabilities */ }
    async fn query(&self, q: &ContextQuery) -> Result<ContextBundle> { /* return SCOs */ }
    async fn resolve(&self, uri: &ContextUri) -> Result<ContextObject> { /* resolve one SCO */ }
}
```

```python
# Python
class MyProvider:
    async def query(self, goal: str, budget: ContextBudget) -> ContextBundle:
        # Return structured context objects
        ...
```

Your provider plugs into the CPP server and is immediately available to every connected AI client.

---

## Python SDK

```python
from cpp_sdk import CppClient, ContextQuery, Goal, ContextBudget

async with CppClient("http://localhost:3030") as client:
    # Initialize session
    await client.initialize()

    # Query context with a 4KB budget
    bundle = await client.query(ContextQuery(
        goal=Goal.code(),
        budget=ContextBudget(max_bytes=4096, max_objects=10)
    ))

    for obj in bundle.objects:
        print(f"{obj.uri} — {obj.title} (importance: {obj.importance})")

    # Subscribe to real-time file change events
    await client.subscribe(filters=["filesystem", "git"])
```

### MCP bridge

CPP integrates with MCP-compatible tools via `mcp_bridge.py`, exposing three perception tools:

| MCP Tool | What it does |
|:--|:--|
| `cpp_query` | Query context with goal intent and budget |
| `cpp_resolve` | Fetch full details for a single `cpp://` URI |
| `cpp_capabilities` | List active providers |

---

## For platform teams: why build a CPP adapter?

If you run a developer platform (GitHub, Jira, Slack, Linear, Notion, GitLab), here's why CPP matters to you:

**Today:** Every AI tool that integrates with your API builds its own custom retrieval logic. You get hit with redundant API calls, inconsistent data representations, and no control over how much data leaves your service.

**With CPP:** You build one adapter. It works with every CPP-compliant AI client. The budget solver limits data egress per query. You express rich relationships (`Blocks`, `DependsOn`, `CreatedBy`) instead of dumping flat JSON.

```
 AI Clients                       CPP Server                       Your Platform
┌─────────────┐                                                   ┌────────────┐
│ Cursor      │ ──┐                                         ┌──── │ GitHub     │
│ Claude Code │ ──┼──▶  CPP Engine (JSON-RPC 2.0 / WS)  ◀──┼──── │ Jira       │
│ Copilot     │ ──┘                                         └──── │ Slack      │
└─────────────┘                                                   └────────────┘
                    Build one adapter. Connect all AI clients.
```

---

## Protocol details

### API methods (JSON-RPC 2.0)

| Method | Type | Purpose |
|:--|:--|:--|
| `cpp/initialize` | Request | Session handshake |
| `cpp/initialized` | Notification | Client confirms ready |
| `cpp/query` | Request | Query context with filters and budget |
| `cpp/resolve` | Request | Fetch one object by `cpp://` URI |
| `cpp/capabilities` | Request | List server capabilities |
| `cpp/providers/list` | Request | List registered providers |
| `cpp/subscribe` | Request | Subscribe to WebSocket events |
| `cpp/unsubscribe` | Request | Cancel subscription |
| `cpp/publish` | Request | Publish event to the bus |
| `cpp/event` | Notification | Server → client push |
| `cpp/shutdown` | Request | Graceful teardown |
| `cpp/exit` | Notification | Final termination |

### Context type taxonomy

Three-level MIME hierarchy. Protocol types are standardized; vendor types need no approval:

```
application/<namespace>.<class>.<type>

# Protocol types (standardized)
application/cpp.document.file
application/cpp.entity.commit
application/cpp.temporal.timestamp

# Vendor extensions (no approval needed)
application/github.entity.pull_request
application/notion.document.database
application/docker.entity.container
```

### Feature comparison

| Feature | Traditional RAG | MCP | CPP |
|:--|:--:|:--:|:--:|
| Execute tools | ❌ | ✅ | ❌ |
| Structured context | ❌ | Partial | ✅ |
| Source-side budgeting | ❌ | ❌ | ✅ |
| Context graph | ❌ | ❌ | ✅ |
| Type taxonomy | ❌ | Partial | ✅ |
| Real-time push | ❌ | Partial | ✅ |

---

## Specification vs. implementation

CPP cleanly separates the **protocol** (what any implementation must follow) from the **reference code** (this repo):

**The protocol** defines: SCO schema, query language (CQL), 3-level MIME types, budget model, graph structure, JSON-RPC methods, and error codes. See [RFC-0001](spec/RFC-0001-CPP.md).

**This repository** is one implementation: a Rust server, Python SDK, and six built-in providers. You can build a compliant CPP server in any language.

### Conformance rules

To be CPP-compliant, an implementation:

- **Must** serialize the exact SCO JSON fields and MIME taxonomy.
- **Must** enforce `maxBytes` and `maxObjects` limits before sending bundles.
- **Must** use error codes `-32000` to `-32009`.
- **May** use any internal storage, indexing, or ranking strategy.

---

## Non-goals

CPP intentionally does **not**:

- **Execute actions.** No file writes, no terminal commands. Use MCP.
- **Manage agents.** No prompt chains, no tool orchestration. Use LangChain or AutoGen.
- **Store embeddings.** Not a vector database. Context is resolved on the fly.
- **Lock you to a language.** JSON-RPC 2.0 over HTTP/WebSocket/stdio. Implement in anything.

---

## Benchmark

Measured on this project's workspace (single run, 4 KB budget):

| Metric | Raw file scan | CPP budget-solved |
|:--|--:|--:|
| Context volume | 122,044 bytes | 232 bytes |
| Estimated LLM tokens | ~30,511 | ~58 |
| Volume reduction | — | 99.81% |
| Resolution time | 450–1,200 ms (shell) | 2.1 ms (in-memory) |

> These numbers are from one workspace. Actual savings depend on project size, query goal, and budget settings.

---

## Repository structure

```
context-provider-protocol/
├── spec/                          # RFC specifications
│   ├── RFC-0000-Philosophy.md
│   └── RFC-0001-CPP.md
├── crates/                        # Rust core
│   ├── cpp-core/                  #   Types, budget solver, graph
│   ├── cpp-protocol/              #   JSON-RPC wire format
│   ├── cpp-sdk/                   #   Provider trait, client
│   ├── cpp-runtime/               #   Resolver, cache
│   ├── cpp-server/                #   HTTP/WebSocket server
│   ├── cpp-transport-http/
│   └── cpp-transport-stdio/
├── providers/                     # Data source adapters
│   ├── filesystem/                #   Rust
│   ├── git/                       #   Rust
│   ├── datetime/                  #   Rust
│   ├── github/                    #   Python
│   ├── jira/                      #   Python
│   └── slack/                     #   Python
├── sdks/python/                   # Python SDK + MCP bridge
└── examples/                      # Demos + benchmark
```

---

## Testing

```bash
# Rust — 55 tests
cargo test --workspace

# Python — 14 tests
cd sdks/python && uv run python -m pytest tests/ -v
```

---

## Roadmap

- [x] **v0.1 — Core protocol** — RFC specs, Rust engine, Python SDK, 6 providers
- [ ] **v0.2 — Enterprise** — stdio transport, multi-tenant auth, vector index providers, TypeScript SDK, conformance test suite
- [ ] **v1.0 — Ecosystem** — stable RFCs, browser extension, multi-language conformance (Go, Rust, Python, TypeScript)

---

## Specifications

| Document | Focus |
|:--|:--|
| [RFC-0000 — Philosophy](spec/RFC-0000-Philosophy.md) | Design principles and motivation |
| [RFC-0001 — Protocol Specification](spec/RFC-0001-CPP.md) | Complete wire format, methods, types, error codes |

---

## License

MIT © [CPP Contributors](LICENSE-MIT)
