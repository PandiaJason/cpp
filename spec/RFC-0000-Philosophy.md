# RFC-0000: Philosophy of the Context Provider Protocol

**Status**: Draft  
**Version**: CPP/0.1  
**Authors**: Context Provider Protocol Contributors  
**Date**: 2026-07-13

---

## 1. What Is Context?

Context is the information an intelligent system needs in order to reason about a situation *right now*.

Context is not data. Data is a fact stored somewhere. A row in a database. A file on a disk. Data has no awareness of who needs it or why.

Context is not knowledge. Knowledge is a persistent truth. The syntax of a programming language. The laws of physics. An API specification. Knowledge does not expire. It does not belong to a session.

Context is the subset of all information that is **relevant to a specific agent, at a specific moment, for a specific purpose**.

| | Data | Knowledge | Context |
|:--|:--|:--|:--|
| **Nature** | Stored fact | Persistent truth | Situated relevance |
| **Lifetime** | Until deleted | Indefinite | Session-bound |
| **Scope** | Universal | Domain-wide | Agent-specific |
| **Freshness** | N/A | Rarely changes | Constantly changing |
| **Example** | `users` table row | "Rust uses ownership" | "The PR you're reviewing has 3 unresolved comments" |

A calendar event at 3pm is data.  
That you have a meeting at 3pm about the project you're coding on — that's context.

A git commit is data.  
That the commit you just pushed broke a test that blocks a release your teammate is waiting on — that's context.

Context is data that has acquired **relevance**.

---

## 2. Why Is Context Different From Data?

Three properties distinguish context from raw data:

### 2.1 Relevance

Data exists independently of any consumer. Context exists *because* a consumer needs it. The same database row is data to the database and context to the agent that needs it right now.

### 2.2 Temporality

Data persists. Context is ephemeral. The fact that "you have a meeting in 10 minutes" is context for the next 10 minutes. After that, it becomes history. Context has a **freshness** that data does not.

### 2.3 Composition

A single piece of data is rarely useful as context. Context is composed: a project is a repository plus its open issues plus its recent commits plus the people working on it plus the upcoming deadline. No single data source contains the full context. **Context is inherently cross-system.**

This is why APIs are insufficient. An API gives you data from one system. Context requires data from many systems, composed and ranked by relevance to the current situation.

---

## 3. Why Is Context Different From Knowledge?

Knowledge is what you learn. Context is what surrounds you.

A software engineer *knows* how to write Rust. That knowledge doesn't change between Monday and Friday. It doesn't expire after a meeting. It isn't scoped to a particular repository.

But the engineer's *context* — the branch they're working on, the PR review they need to finish, the deployment that failed last night, the meeting about the feature in 30 minutes — that changes constantly.

| | Knowledge | Context |
|:--|:--|:--|
| **Temporal** | Static or slowly evolving | Dynamic, session-aware |
| **Retrieval** | Search by topic | Arrive by relevance |
| **Source** | Curated, authored | Observed, published |
| **Consumption** | Referenced when needed | Pushed when relevant |
| **Expiration** | Rarely | Always |

Knowledge answers: *"What do I need to understand?"*  
Context answers: *"What is happening around me right now?"*

A protocol that confuses the two will inevitably become a database. CPP is not a database. CPP transports **situated, temporal, relevant context** from the systems that produce it to the agents that need it.

---

## 4. Why Is CPP Different From MCP?

The Model Context Protocol (MCP) answers: *"How can an AI perform an action?"*

MCP gives AI systems **tools** — executable functions that read files, query databases, call APIs, and modify state. MCP is the hands and feet of an AI agent.

CPP answers a different question: *"What should an AI know before it reasons or acts?"*

CPP gives AI systems **context** — the situated, relevant information that informs reasoning. CPP is the eyes and ears of an AI agent.

### The Perception–Reasoning–Action Loop

Every intelligent system, biological or artificial, follows the same loop:

```
Perceive → Reason → Act
```

- A human **sees** a pull request with failing tests (perception), **thinks** about the root cause (reasoning), and **pushes** a fix (action).
- An AI agent **receives context** about the failing tests (perception via CPP), **reasons** about the fix (LLM inference), and **executes** the fix (action via MCP).

```
┌─────────────────────────────────────────┐
│           The AI Agent Loop             │
│                                         │
│   CPP (Perceive)                        │
│     ↓                                   │
│   LLM (Reason)                          │
│     ↓                                   │
│   MCP (Act)                             │
│     ↓                                   │
│   CPP (Perceive the result)             │
│     ↓                                   │
│   ...                                   │
└─────────────────────────────────────────┘
```

MCP without CPP is an agent that acts blindly.  
CPP without MCP is an agent that observes but cannot act.  
Together, they form the complete agent loop.

### Comparison

| | MCP | CPP |
|:--|:--|:--|
| **Question** | How can I act? | What should I know? |
| **Primitives** | Tools, Resources, Prompts | Semantic Context Objects, Events, Sessions |
| **Direction** | Agent → System (imperative) | System → Agent (declarative / published) |
| **Trigger** | Agent decides to call a tool | Context arrives or is requested |
| **Scope** | Single system per server | Cross-system, composed |
| **Model** | Request/Response | Publish/Subscribe + Request |
| **Analogy** | Hands and feet | Eyes and ears |

They are complementary. Not competitive.

---

## 5. What Problems Does CPP Solve That Existing Protocols Don't?

### Problem 1: The Context Assembly Problem

Today, when an AI agent needs to understand a project, it must:

1. Call the GitHub API to get repository information
2. Call the Jira API to get open issues
3. Call the Google Calendar API to get upcoming meetings
4. Call the Slack API to get recent conversations
5. Read local files for the current code state
6. Somehow merge all of this into a coherent picture

Every agent framework solves this differently. Every integration is bespoke. There is no standard way to say "give me the context about this project."

**CPP solves this.** An agent says:

```
Need: goal.project
Scope: current
Include: repositories, tasks, meetings, conversations
```

The protocol handles the rest — routing to capable providers, composing the results, respecting permissions, managing freshness.

### Problem 2: The Context Freshness Problem

Data retrieved from an API five minutes ago may already be stale. A meeting that just started. A PR that just merged. A deployment that just failed. Today, agents have no way to know whether their context is current.

**CPP solves this.** Every Semantic Context Object carries freshness metadata:

- `live` — real-time, always current
- `recent` — fetched recently, may become stale
- `cached` — served from cache, check staleness
- `immutable` — never changes (a git commit hash)

Agents and runtimes use this to decide whether to re-fetch or trust the cached context.

### Problem 3: The Context Permission Problem

When an AI agent accesses Gmail, GitHub, and Notion, who controls what it sees? Today, each integration has its own auth model. There is no unified way to say "this agent may see project metadata but not email content."

**CPP solves this.** Capability-based permission tokens scope what context an agent may receive — by provider, by context type, by access level — with time-limited, delegatable, attenuatable tokens.

### Problem 4: The Context Composition Problem

Context is rarely one thing. A "coding context" is the current branch + recent commits + open PR + related issue + team discussion + upcoming deadline. These objects have **relationships** — the PR references the issue, the issue is owned by a person, the person has a meeting about the project.

Today, the agent has to manually connect these dots.

**CPP solves this.** Semantic Context Objects carry typed relationships. The protocol defines relationship semantics (contains, references, created_by, depends_on). Agents can request context with depth traversal — "give me the project, and everything one hop away."

### Problem 5: The Context Bus Problem

Today, context is **pulled** — the agent asks for it. But much of the most valuable context is **pushed** — a file changed, a meeting started, a PR was reviewed, a message arrived.

**CPP solves this.** Providers **publish** context events to the protocol bus. Agents **subscribe** to context they care about. This is not just request/response — it's a publish/subscribe context bus, like Kafka for semantic context.

```
GitHub Provider  ──publishes──→  "Repository Updated"    ──→ ┐
Calendar Provider ─publishes──→  "Meeting Starting"      ──→ ├──→ Agent
Filesystem Provider publishes─→  "File Modified"         ──→ ┘
```

The agent doesn't poll. It receives relevant context as it happens.

### Problem 6: The Multi-Agent Context Problem

When multiple AI agents collaborate, how do they share context? Today, there is no standard. Agent A's knowledge of the project is invisible to Agent B.

**CPP solves this.** Any agent can be a context provider. A vision AI that detects objects can publish context. A planning agent that decomposes tasks can publish context. A coding agent that understands a codebase can publish context. Other agents subscribe.

```
Vision AI ──publishes──→ "Object Detected: whiteboard diagram"
                              ↓
Planning Agent ──subscribes──→ receives context
                              ↓
               ──publishes──→ "Task Decomposition: 5 subtasks"
                              ↓
Coding Agent ──subscribes──→ receives context
```

A **provider** in CPP is not limited to SaaS APIs and databases. A provider is **anything that can produce semantic context** — including other AI systems.

---

## 6. The Fundamental Abstraction

CPP introduces one new abstraction into the computing stack:

**The Semantic Context Object (SCO).**

An SCO is not a database row. It is not an API response. It is not a vector embedding. It is not a document.

An SCO is a unit of **situated, relevant, typed, permissioned, fresh context** that an intelligent system can reason about.

Every SCO carries:

| Property | Purpose |
|:--|:--|
| **URI** | Globally addressable (`cpp://provider/type/path`) |
| **Type** | MIME-like classification (`application/cpp.event.meeting`) |
| **Class** | Base taxonomy (Entity, Document, Event, Collection, Reference) |
| **Certainty** | Authoritative, Derived, or Estimated |
| **Freshness** | Live, Recent, Cached, or Immutable |
| **Lifecycle** | Created, Updated, Merged, Archived, Expired, Deleted |
| **Relationships** | Typed edges to other SCOs |
| **Permissions** | Who may see this context |
| **Importance** | How significant this is relative to other context |

SCOs are not stored by the protocol. They are **transported** by the protocol. How they are stored, indexed, cached, or ranked is an implementation concern — not a protocol concern.

---

## 7. The Protocol Stack

CPP occupies a specific layer in the emerging AI infrastructure stack:

```
┌───────────────────────────┐
│     Applications          │   User-facing products
├───────────────────────────┤
│     AI Models (LLMs)      │   Reasoning
├───────────────────────────┤
│     MCP (Action)          │   Tool execution
├───────────────────────────┤
│     CPP (Perception)      │   Semantic context
├───────────────────────────┤
│     Adapters              │   API translation
├───────────────────────────┤
│     Providers             │   GitHub, Gmail, Calendar, AI...
├───────────────────────────┤
│     Systems               │   Databases, APIs, filesystems
└───────────────────────────┘
```

Like HTTP, CPP does not define what happens above or below it. It defines the **interface between systems that produce context and systems that consume it**.

---

## 8. Design Principles

From the above philosophy, the following design principles follow:

1. **CPP is a protocol, not a runtime.** It defines messages, semantics, and guarantees. Not caching strategies, ranking algorithms, or graph databases.

2. **Context is published, not just requested.** The primary flow is provider → bus → agent, not agent → query → provider.

3. **Everything is an event.** SCOs are snapshots. Events are the stream. The protocol is event-first.

4. **The protocol must be small.** If it can't fit in a developer's head, it's too big. HTTP succeeded because it was simple. CPP must be simple.

5. **Providers are anything.** A database, a SaaS API, a filesystem, an IoT sensor, another AI agent. The protocol doesn't discriminate.

6. **Relationships are protocol-level.** Graph storage is not. The protocol defines typed edges. How they're stored is implementation-specific.

7. **Freshness is mandatory.** Every SCO must declare how fresh it is. An agent that can't trust its context can't reason correctly.

8. **Permissions are capability-based.** Not identity-based. Tokens encode what context is allowed, not who is asking.

9. **The scope is context, not knowledge.** CPP transports situated, temporal, relevant information. Not encyclopedic facts, static documentation, or training data.

10. **Interoperability is the measure of success.** If two independent implementations can't communicate without special-case code, the protocol has failed.

---

## 9. What CPP Is Not

- CPP is **not** a vector database.
- CPP is **not** a RAG framework.
- CPP is **not** an embedding service.
- CPP is **not** a memory system.
- CPP is **not** a knowledge graph.
- CPP is **not** a database abstraction.
- CPP is **not** MCP.

CPP is the **semantic context layer** — the standard interface between systems that produce context and intelligent systems that consume it.

---

## 10. The Test

The protocol is successful if and only if:

1. Two teams, working independently, can build two different CPP runtimes that interoperate without coordination.

2. A provider written for runtime A works unmodified with runtime B.

3. An agent built against runtime A can consume context from runtime B.

4. The protocol specification fits in a single document that an engineer can read in an afternoon.

If those four conditions are met, CPP is a protocol.  
If any of them fail, CPP is merely a framework.

---

*This document is the philosophical foundation of the Context Provider Protocol. The technical specification (RFC-0001) builds on these principles.*
