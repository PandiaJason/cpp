# Context Provider Protocol (CPP)

The Context Provider Protocol (CPP) is an open standard for AI applications to negotiate, perceive, and route dynamic, situated context.

## What is CPP?
CPP provides a standardized way for AI assistants like Claude, GPT, or Antigravity to:
* Connect to dynamic context providers and environment monitors
* Access situated, budget-negotiated, and ranked context
* Share state asynchronously across collaborative multi-agent networks
* Maintain strict token, bytes, and lifecycle controls

---

## How CPP works

### Local vs remote servers
| Type | Description | Use Case |
| :--- | :--- | :--- |
| **Local CPP** | Runs on your device | Local directory scanning, git status, system timezone |
| **Remote CPP** | Hosted on the internet | Shared context buses, collaborative team agents |

### Key components
* **Semantic Context Objects (SCOs)**: Ephemeral, typed, and permissioned envelopes carrying situated context (URI, type, content, relations).
* **Goal Registry**: Intent-based query targets (`goal.code`, `goal.project`, `goal.calendar`) that automate context routing.
* **Context Bus (Events)**: Streaming publish/subscribe channels delivering real-time environment notifications.

---

## Security & Metadata Model

### User control
* Capability-based permission tokens scope access levels (`read`, `write`, `admin`).
* Time-limited tokens with automatic expiration (TTLs) and lifecycle tracking (`created`, `updated`, `deleted`).

### Context hints
All CPP objects must declare:
* **Freshness**: Defines cache state (`live`, `recent`, `cached`, `immutable`) to prevent stale context injections.
* **Certainty**: Defines the truth level of the source (`authoritative`, `derived`, `estimated`).
This helps agents understand how reliable and current the information is.

---

## Building with CPP
The CPP documentation (located in the [spec](file:///Users/admin/Jas%20Apps/Context%20Provider%20Protocol/spec) folder) is the source of truth for building CPP servers.

### For developers
* Open specification in [RFC-0001-CPP.md](file:///Users/admin/Jas%20Apps/Context%20Provider%20Protocol/spec/RFC-0001-CPP.md).
* Rust SDK (`cpp-sdk`) and reference orchestrator runtime (`cpp-runtime`) available.
* Reference local filesystem, git, and datetime providers built-in.
