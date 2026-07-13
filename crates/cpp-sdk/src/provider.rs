//! Provider-side interfaces and types.

use async_trait::async_trait;
use cpp_core::context::{ContextBundle, ContextObject};
use cpp_core::error::CppError;
use cpp_core::manifest::ProviderManifest;
use cpp_core::query::ContextQuery;
use cpp_core::types::ContextUri;

/// Every context provider MUST implement this trait to plug into a CPP runtime.
///
/// A provider is any system that exposes semantic context. Examples:
/// - A filesystem provider that serves local files.
/// - A git provider that serves local commits, branches, and changes.
/// - A calendar provider that serves upcoming meetings.
/// - A planning AI agent that serves task decompositions.
///
/// Providers leverage the Adapter pattern ([`crate::adapter::ProviderAdapter`]) to
/// map system-specific data into the standardized [`ContextObject`] format.
#[async_trait]
pub trait ContextProvider: Send + Sync {
    /// Returns this provider's manifest, declaring its capabilities.
    fn manifest(&self) -> &ProviderManifest;

    /// Queries the provider for context matching the CRQ.
    async fn query(&self, query: &ContextQuery) -> Result<ContextBundle, CppError>;

    /// Resolves a single Context Object by its URI.
    async fn resolve(&self, uri: &ContextUri) -> Result<ContextObject, CppError>;

    /// Hook called when the provider starts.
    async fn start(&mut self) -> Result<(), CppError> {
        Ok(())
    }

    /// Hook called when the provider stops.
    async fn stop(&mut self) -> Result<(), CppError> {
        Ok(())
    }
}
