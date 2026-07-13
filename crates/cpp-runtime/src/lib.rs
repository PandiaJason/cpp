//! Reference orchestration runtime for the Context Provider Protocol.
//!
//! This crate implements the core dispatch, caching, ranking, relationship traversal,
//! and session management logic of a CPP runtime.
//!
//! AI agents communicate with this runtime, which in turn orchestrates requests
//! to one or more registered [`cpp_sdk::ContextProvider`]s.

pub mod cache;
pub mod registry;
pub mod resolver;
pub mod session;

pub use cache::ContextCache;
pub use registry::ProviderRegistry;
pub use resolver::ContextResolver;
pub use session::SessionTracker;
