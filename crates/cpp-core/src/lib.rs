//! # CPP Core
//!
//! Core types and abstractions for the **Context Provider Protocol (CPP)**.
//!
//! CPP is the semantic context layer for AI. This crate defines the
//! protocol-level types that all implementations build upon.
//!
//! ## Core Abstraction: The Semantic Context Object (SCO)
//!
//! Every piece of context in CPP is represented as a [`ContextObject`] —
//! a unit of situated, relevant, typed, permissioned, fresh context that
//! an intelligent system can reason about.
//!
//! ## Module Overview
//!
//! - [`types`] — Primitive types: IDs, URIs, classes, goals, certainty, freshness
//! - [`context`] — Semantic Context Object (SCO), bundles, sessions
//! - [`query`] — Context Request Query (CRQ): goal, scope, constraints, budget
//! - [`relation`] — Typed relationship edges between SCOs
//! - [`permission`] — Capability-based access control
//! - [`stream`] — Context events, subscriptions, publish/subscribe
//! - [`manifest`] — Provider capability declarations
//! - [`error`] — Protocol error codes

pub mod context;
pub mod error;
pub mod manifest;
pub mod permission;
pub mod query;
pub mod relation;
pub mod stream;
pub mod types;

// Re-export primary types for convenience
pub use context::{
    ContextBundle, ContextObject, ContextObjectBuilder, ContextPermissions, ContextSession,
    Reference,
};
pub use error::{CppError, ErrorCode};
pub use manifest::{ProviderCapabilities, ProviderManifest};
pub use permission::{AccessLevel, CapabilityToken, ContextCapability};
pub use query::{ContextQuery, ContextQueryBuilder, QueryConstraints, QueryScope};
pub use relation::{Relation, RelationType};
pub use stream::{ContextEvent, ContextEventKind, Subscription, SubscriptionFilter};
pub use types::{
    BudgetPreference, Certainty, ContextBudget, ContextClass, ContextId, ContextType, ContextUri,
    Duration, Freshness, FreshnessKind, Goal, Importance, LifecycleState, ProviderId,
    ProtocolVersion, SessionId, SubscriptionId, PROTOCOL_NAME, PROTOCOL_VERSION,
};
