//! Provider manifest and capability declarations.
//!
//! Every provider publishes a **manifest** that declares what context
//! it can provide. This enables **Context Capability Discovery** — agents
//! ask "what context can you provide?" and providers respond with their
//! capabilities.
//!
//! This is fundamentally different from "discover providers." An agent
//! can dynamically plan based on what context is *available*, not what
//! providers are *registered*.
//!
//! # Adapter Pattern
//!
//! Providers use the adapter pattern to isolate API-specific details
//! from the protocol:
//!
//! ```text
//! GitHub REST API v4  →  GitHubAdapter  →  CPP SCO
//! Gmail API v1        →  GmailAdapter   →  CPP SCO
//! Notion API 2023-08  →  NotionAdapter  →  CPP SCO
//! ```
//!
//! When GitHub changes its API, only the adapter changes.

use serde::{Deserialize, Serialize};

use crate::types::{ContextType, Goal, ProviderId, ProtocolVersion};

// ═══════════════════════════════════════════════════════════════════════════
//  ProviderManifest
// ═══════════════════════════════════════════════════════════════════════════

/// A provider's self-description and capability declaration.
///
/// Manifests are how providers register with a runtime and advertise
/// what context they can produce. This powers capability discovery.
///
/// # Example (JSON)
///
/// ```json
/// {
///   "id": "github",
///   "name": "GitHub Context Provider",
///   "version": "1.0.0",
///   "protocolVersion": { "major": 0, "minor": 1 },
///   "capabilities": {
///     "contextTypes": [
///       "application/cpp.entity.repository",
///       "application/cpp.entity.project",
///       "application/github.entity.pull_request",
///       "application/github.entity.issue",
///       "application/cpp.event.commit"
///     ],
///     "goals": ["goal.project", "goal.code"],
///     "supportsSubscriptions": true,
///     "supportsStreaming": true
///   }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderManifest {
    /// Unique provider identifier.
    pub id: ProviderId,

    /// Human-readable provider name.
    pub name: String,

    /// Provider implementation version.
    pub version: String,

    /// Optional description of what this provider does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Protocol version this provider implements.
    pub protocol_version: ProtocolVersion,

    /// What this provider can do.
    pub capabilities: ProviderCapabilities,

    /// Contact/source URL for this provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl ProviderManifest {
    /// Creates a new manifest with minimal required fields.
    pub fn new(
        id: ProviderId,
        name: impl Into<String>,
        capabilities: ProviderCapabilities,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            version: "0.1.0".into(),
            description: None,
            protocol_version: ProtocolVersion::current(),
            capabilities,
            url: None,
        }
    }

    /// Sets the provider version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Sets the URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Returns `true` if this provider can serve the requested goal.
    pub fn supports_goal(&self, goal: &Goal) -> bool {
        self.capabilities.goals.contains(goal)
    }

    /// Returns `true` if this provider can produce the requested type.
    pub fn supports_type(&self, ct: &ContextType) -> bool {
        self.capabilities.context_types.contains(ct)
    }
}

impl std::fmt::Display for ProviderManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Provider({} — \"{}\" v{} — {} types, {} goals)",
            self.id,
            self.name,
            self.version,
            self.capabilities.context_types.len(),
            self.capabilities.goals.len(),
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ProviderCapabilities
// ═══════════════════════════════════════════════════════════════════════════

/// What a provider can do — the response to "What context can you provide?"
///
/// This powers Context Capability Discovery. An agent doesn't need to
/// know about specific providers. It asks the runtime "who can give me
/// project context?" and the runtime checks manifests.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapabilities {
    /// Context types this provider can produce.
    ///
    /// e.g., `["application/cpp.entity.repository", "application/cpp.event.commit"]`
    pub context_types: Vec<ContextType>,

    /// Goals this provider can help fulfill.
    ///
    /// e.g., `["goal.project", "goal.code"]`
    pub goals: Vec<Goal>,

    /// Whether this provider supports real-time event subscriptions.
    #[serde(default)]
    pub supports_subscriptions: bool,

    /// Whether this provider supports streaming responses.
    #[serde(default)]
    pub supports_streaming: bool,

    /// Whether this provider supports resolving individual URIs.
    #[serde(default = "default_true")]
    pub supports_resolve: bool,

    /// Whether this provider can publish events proactively.
    #[serde(default)]
    pub supports_publish: bool,

    /// Whether this provider can negotiate context types.
    #[serde(default)]
    pub supports_negotiation: bool,
}

fn default_true() -> bool { true }

impl ProviderCapabilities {
    /// Creates capabilities for a basic read-only provider.
    pub fn basic(context_types: Vec<ContextType>, goals: Vec<Goal>) -> Self {
        Self {
            context_types,
            goals,
            supports_subscriptions: false,
            supports_streaming: false,
            supports_resolve: true,
            supports_publish: false,
            supports_negotiation: false,
        }
    }

    /// Creates capabilities for a full-featured provider.
    pub fn full(context_types: Vec<ContextType>, goals: Vec<Goal>) -> Self {
        Self {
            context_types,
            goals,
            supports_subscriptions: true,
            supports_streaming: true,
            supports_resolve: true,
            supports_publish: true,
            supports_negotiation: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_creation() {
        let manifest = ProviderManifest::new(
            ProviderId::new("github"),
            "GitHub Context Provider",
            ProviderCapabilities::full(
                vec![ContextType::repository(), ContextType::commit()],
                vec![Goal::project(), Goal::code()],
            ),
        )
        .with_version("1.0.0")
        .with_description("Provides GitHub repository context");

        assert_eq!(manifest.id, ProviderId::new("github"));
        assert!(manifest.supports_goal(&Goal::project()));
        assert!(manifest.supports_type(&ContextType::repository()));
        assert!(!manifest.supports_goal(&Goal::email()));
    }

    #[test]
    fn manifest_serialization() {
        let manifest = ProviderManifest::new(
            ProviderId::new("filesystem"),
            "Filesystem Provider",
            ProviderCapabilities::basic(
                vec![ContextType::file(), ContextType::folder()],
                vec![Goal::code(), Goal::document()],
            ),
        );

        let json = serde_json::to_value(&manifest).unwrap();
        assert_eq!(json["id"], "filesystem");
        assert_eq!(json["capabilities"]["goals"].as_array().unwrap().len(), 2);
    }
}
