//! The Adapter pattern for isolating provider API details.

use cpp_core::context::ContextObject;
use cpp_core::error::CppError;
use cpp_core::types::ContextUri;

/// Translates external API records into standardized CPP Semantic Context Objects (SCOs).
///
/// Under the Adapter pattern, system-specific details (e.g., specific HTTP schemas,
/// database rows, JSON structures) are translated into protocol-compliant SCOs.
/// When the underlying service's API changes, only the adapter's implementation
/// needs to change. The protocol and any consuming agents remain untouched.
pub trait ProviderAdapter: Send + Sync {
    /// The source data type from the system of record.
    type ExternalRecord;

    /// Translates an external record into a ContextObject.
    fn adapt(&self, record: Self::ExternalRecord) -> Result<ContextObject, CppError>;

    /// Translates a ContextUri into an external identifier or query format.
    fn resolve_uri(&self, uri: &ContextUri) -> Result<String, CppError>;
}
