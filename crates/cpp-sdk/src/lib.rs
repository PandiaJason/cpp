//! SDK for implementing CPP context providers and building agent clients.

pub mod adapter;
pub mod client;
pub mod provider;

pub use adapter::ProviderAdapter;
pub use client::CppClient;
pub use provider::ContextProvider;

// Re-export core types for SDK convenience
pub use cpp_core::{
    AccessLevel, Certainty, ContextBudget, ContextBundle, ContextClass, ContextEvent,
    ContextEventKind, ContextId, ContextObject, ContextObjectBuilder, ContextPermissions,
    ContextQuery, ContextQueryBuilder, ContextSession, ContextType, ContextUri, Duration,
    ErrorCode, Freshness, FreshnessKind, Goal, Importance, LifecycleState, ProviderCapabilities,
    ProviderId, ProviderManifest, ProtocolVersion, Reference, SessionId, Subscription,
    SubscriptionFilter, SubscriptionId,
};
