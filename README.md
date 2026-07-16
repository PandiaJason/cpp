# Introducing the Context Provider Protocol (CPP)

**13 July 2026**

*An open-source standard for AI systems to perceive, route, and negotiate situated context in real time.*

Today, we are open-sourcing the **Context Provider Protocol (CPP)**, a new standard for connecting AI assistants and collaborative agents to the dynamic environments where they operate. Its aim is to serve as the unified **perception layer** for AI, helping frontier models receive better, more situated, and token-efficient context.

As AI assistants gain mainstream adoption, the industry has achieved rapid advances in reasoning (via LLMs) and action (via tool-calling protocols like MCP). Yet even the most sophisticated models remain constrained by their lack of environmental perception—relying on brute-force prompt stuffing, static RAG databases, and ad-hoc shell commands to understand their workspace. Every new context source requires its own custom implementation, making context window management and token budgeting difficult to scale.

CPP addresses this challenge. It provides a universal, open standard for connecting AI systems with dynamic context sources, replacing fragmented integrations with a single, event-first protocol. The result is a simpler, more reliable way to give AI systems access to the exact context they need, right when they need it.

---

## What is the Context Provider Protocol?

The Context Provider Protocol is an open standard that enables developers to build secure, two-way connections between dynamic context sources (providers) and AI-powered tools (clients). The architecture is straightforward: developers can either expose their workspace data through CPP servers or build AI applications (CPP clients) that connect to these servers.

Today, we are introducing three major components of the Context Provider Protocol for developers:

1. **The CPP Specification and SDKs**: Open, transport-agnostic standards supporting JSON-RPC 2.0 wire messages, capability negotiation, and context budgeting.
2. **Local CPP Daemon & Dashboard (`cpp-server`)**: A reference orchestrator runtime that hosts query resolution, serves an interactive graphical console, and broadcasts live environment events.
3. **Reference Providers**: Built-in providers for Filesystem scanning, Git repository history tracking, and Datetime timezone reporting.

To help developers start exploring, we have shared pre-built binaries and examples showing how to query workspace metadata, filter objects by certainty and recency, and stream updates over WebSockets.

---

## Why CPP is Genuinely Better Than Existing Approaches

Traditional AI systems gather context by "prompt stuffing" (dumping files into prompts) or running semantic searches over a vector database (RAG). CPP is fundamentally better for three reasons:

* **Budget Enforcement at the Source**: RAG searches and direct file loads have no concept of context limits. If a folder contains 100MB of code, a direct tool will overload your model's context window. CPP negotiates a strict `ContextBudget` (in bytes or objects) before sending data, dynamically ranking and downsampling files at the provider level to guarantee a low token count.
* **Relational Context Graphs**: Traditional tools return flat, isolated lists of data. CPP builds a directed, weighted semantic graph. It maps relationships between objects (e.g. `Branch` -> `references` -> `Commit` -> `modifies` -> `File` -> `fails` -> `Compiler Error`), allowing the AI to understand the structural context of the codebase in a single structured schema.
* **Passive Event Streams vs. Active Polling**: To stay up-to-date with your workspace, traditional systems must poll command execution (e.g. running `git status` every 5 seconds). CPP uses WebSockets to stream real-time push events (`cpp/event`). The agent stays idle, waking up only when a file is saved or a build fails.

---

## The Paradigm Shift: Perception vs. Action

While tools allow AI to act, they do not help them perceive. CPP completes the agent loop by formalizing the **Perceive → Reason → Act** cycle. Instead of relying on active polling, agents subscribe to the CPP bus and receive live context updates passively as they happen in the environment.

Rather than maintaining separate, bespoke connectors for each dataset, developers can now build against a standard protocol. As the ecosystem matures, AI systems will maintain precise context as they transition between different tools and codebases, replacing today's fragmented integrations with a more sustainable, budget-compliant architecture.

---

## Getting Started

Developers can start building and testing CPP connectors today.

To start exploring:
1. **Launch the Daemon & Dashboard**: Run `cargo run --bin cpp-server` and navigate to `http://localhost:3030` to visual the live context relationship graph.
2. **Run a Context Query**: Execute `cargo run --bin simple-query` to test local directory resolution and Git status checking.
3. **Explore the Specifications**: Read [RFC-0000: Philosophy](file:///Users/admin/Jas%20Apps/Context%20Provider%20Protocol/spec/RFC-0000-Philosophy.md) and [RFC-0001: Spec](file:///Users/admin/Jas%20Apps/Context%20Provider%20Protocol/spec/RFC-0001-CPP.md) in the specification folder.

---

## An Open Community

CPP was created to resolve the context assembly fragmentation facing modern AI agents. We are committed to building CPP as a collaborative, open-source project and ecosystem. Whether you are an AI tool developer, an enterprise looking to leverage existing workspace metadata, or an early adopter exploring the agentic frontier, we invite you to build the future of context-aware AI together.
