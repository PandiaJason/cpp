//! The Semantic Context Object (SCO) and related types.
//!
//! Every piece of context flowing through CPP is a [`ContextObject`] — the
//! protocol's fundamental unit. An SCO is not a database row or an API
//! response. It is a unit of **situated, relevant, typed, permissioned, fresh
//! context** that an intelligent system can reason about.
//!
//! SCOs are **transported** by the protocol, not stored by it.
//!
//! A [`ContextBundle`] groups SCOs returned from a query.
//! A [`ContextSession`] groups SCOs that belong to a coherent working context.

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::permission::AccessLevel;
use crate::relation::Relation;
use crate::types::{
    Certainty, ContextId, ContextType, ContextUri, Freshness, Importance, LifecycleState,
    ProviderId, SessionId,
};

// ═══════════════════════════════════════════════════════════════════════════
//  Reference
// ═══════════════════════════════════════════════════════════════════════════

/// A reference to an external resource associated with a context object.
///
/// References link SCOs back to their original sources for provenance
/// tracking and direct access.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    /// The URI of the referenced resource.
    pub uri: String,
    /// The type of reference (`"source"`, `"documentation"`, `"related"`).
    #[serde(rename = "type")]
    pub ref_type: String,
    /// Optional human-readable label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl Reference {
    pub fn new(uri: impl Into<String>, ref_type: impl Into<String>) -> Self {
        Self { uri: uri.into(), ref_type: ref_type.into(), label: None }
    }
    pub fn source(uri: impl Into<String>) -> Self { Self::new(uri, "source") }
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ContextPermissions
// ═══════════════════════════════════════════════════════════════════════════

/// Permission metadata attached to an SCO.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextPermissions {
    /// The access level at which this context was provided.
    pub level: AccessLevel,
    /// Scopes authorized to access this context (empty = all).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

impl ContextPermissions {
    pub fn new(level: AccessLevel) -> Self {
        Self { level, scopes: Vec::new() }
    }
    pub fn read() -> Self { Self::new(AccessLevel::Read) }
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }
}

impl Default for ContextPermissions {
    fn default() -> Self { Self::read() }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ContextObject (Semantic Context Object / SCO)
// ═══════════════════════════════════════════════════════════════════════════

/// The Semantic Context Object (SCO) — the fundamental unit of CPP.
///
/// An SCO is not data. It is not an API response. It is a unit of
/// **situated, relevant, typed, permissioned, fresh context**.
///
/// # Required Fields (MUST per RFC-0000)
///
/// | Field | Purpose |
/// |:------|:--------|
/// | `uri` | Globally addressable (`cpp://provider/type/path`) |
/// | `id` | Unique identifier within the runtime |
/// | `context_type` | MIME-like classification |
/// | `provider_id` | Which provider produced this |
/// | `certainty` | Authoritative, Derived, or Estimated |
/// | `freshness` | How current this data is |
/// | `lifecycle` | Created, Updated, Merged, Archived, Expired, Deleted |
/// | `permissions` | Who may see this |
/// | `created_at` | When first created |
/// | `updated_at` | When last modified |
///
/// # Example
///
/// ```rust
/// use cpp_core::context::ContextObjectBuilder;
/// use cpp_core::types::*;
///
/// let sco = ContextObjectBuilder::new(
///     ContextUri::new("github", "repository", "user/project"),
///     ContextType::repository(),
///     ProviderId::new("github"),
/// )
/// .title("my-project")
/// .certainty(Certainty::Authoritative)
/// .freshness(Freshness::recent(Duration::hours(1)))
/// .importance(Importance::high())
/// .build();
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextObject {
    // ── Identity ──

    /// Globally addressable URI (`cpp://provider/type/path`).
    pub uri: ContextUri,
    /// Unique identifier within the runtime.
    pub id: ContextId,
    /// Version number, incremented on each update.
    pub version: u32,
    /// MIME-like semantic type classification.
    pub context_type: ContextType,
    /// The provider that produced this SCO.
    pub provider_id: ProviderId,

    // ── Temporal ──

    /// When this context was first created.
    pub created_at: DateTime<Utc>,
    /// When this context was last updated.
    pub updated_at: DateTime<Utc>,
    /// When this context expires and should be discarded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    // ── Semantic Properties ──

    /// How certain the provider is about this context.
    pub certainty: Certainty,
    /// How fresh this context's data is.
    pub freshness: Freshness,
    /// Current lifecycle state.
    pub lifecycle: LifecycleState,
    /// Importance relative to other SCOs.
    pub importance: Importance,

    // ── Content ──

    /// Human-readable title.
    pub title: String,
    /// Brief summary of the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Full content (omitted at MetadataOnly/Summarize access levels).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    // ── Access Control ──

    /// Permission metadata.
    pub permissions: ContextPermissions,

    // ── Graph ──

    /// Typed relationships to other SCOs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<Relation>,
    /// References to external resources.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<Reference>,

    // ── Extension ──

    /// Provider-specific metadata (e.g., `{"language": "rust"}`).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: IndexMap<String, serde_json::Value>,
    /// Extensibility for forward compatibility.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, serde_json::Value>,
}

impl ContextObject {
    /// Returns `true` if this SCO has expired.
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }

    /// Returns `true` if content is populated.
    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }

    /// Returns a metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

impl std::fmt::Display for ContextObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SCO({} [{}] — \"{}\" [{}])",
            self.uri, self.context_type, self.title, self.certainty
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ContextObjectBuilder
// ═══════════════════════════════════════════════════════════════════════════

/// Builder for constructing Semantic Context Objects.
pub struct ContextObjectBuilder {
    uri: ContextUri,
    context_type: ContextType,
    provider_id: ProviderId,
    id: Option<ContextId>,
    version: u32,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    certainty: Certainty,
    freshness: Freshness,
    lifecycle: LifecycleState,
    importance: Importance,
    title: String,
    summary: Option<String>,
    content: Option<String>,
    permissions: ContextPermissions,
    relations: Vec<Relation>,
    references: Vec<Reference>,
    metadata: IndexMap<String, serde_json::Value>,
    extensions: IndexMap<String, serde_json::Value>,
}

impl ContextObjectBuilder {
    /// Creates a new builder with the three required fields.
    pub fn new(uri: ContextUri, context_type: ContextType, provider_id: ProviderId) -> Self {
        Self {
            uri,
            context_type,
            provider_id,
            id: None,
            version: 1,
            created_at: None,
            updated_at: None,
            expires_at: None,
            certainty: Certainty::default(),
            freshness: Freshness::default(),
            lifecycle: LifecycleState::default(),
            importance: Importance::default(),
            title: String::new(),
            summary: None,
            content: None,
            permissions: ContextPermissions::default(),
            relations: Vec::new(),
            references: Vec::new(),
            metadata: IndexMap::new(),
            extensions: IndexMap::new(),
        }
    }

    pub fn id(mut self, id: ContextId) -> Self { self.id = Some(id); self }
    pub fn version(mut self, v: u32) -> Self { self.version = v; self }
    pub fn created_at(mut self, ts: DateTime<Utc>) -> Self { self.created_at = Some(ts); self }
    pub fn updated_at(mut self, ts: DateTime<Utc>) -> Self { self.updated_at = Some(ts); self }
    pub fn expires_at(mut self, ts: DateTime<Utc>) -> Self { self.expires_at = Some(ts); self }
    pub fn certainty(mut self, c: Certainty) -> Self { self.certainty = c; self }
    pub fn freshness(mut self, f: Freshness) -> Self { self.freshness = f; self }
    pub fn lifecycle(mut self, l: LifecycleState) -> Self { self.lifecycle = l; self }
    pub fn importance(mut self, i: Importance) -> Self { self.importance = i; self }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn summary(mut self, s: impl Into<String>) -> Self { self.summary = Some(s.into()); self }
    pub fn content(mut self, c: impl Into<String>) -> Self { self.content = Some(c.into()); self }
    pub fn permissions(mut self, p: ContextPermissions) -> Self { self.permissions = p; self }

    pub fn relation(mut self, r: Relation) -> Self { self.relations.push(r); self }
    pub fn reference(mut self, r: Reference) -> Self { self.references.push(r); self }
    pub fn metadata(mut self, k: impl Into<String>, v: serde_json::Value) -> Self {
        self.metadata.insert(k.into(), v); self
    }
    pub fn extension(mut self, k: impl Into<String>, v: serde_json::Value) -> Self {
        self.extensions.insert(k.into(), v); self
    }

    /// Builds the Semantic Context Object.
    pub fn build(self) -> ContextObject {
        let now = Utc::now();
        ContextObject {
            uri: self.uri,
            id: self.id.unwrap_or_default(),
            version: self.version,
            context_type: self.context_type,
            provider_id: self.provider_id,
            created_at: self.created_at.unwrap_or(now),
            updated_at: self.updated_at.unwrap_or(now),
            expires_at: self.expires_at,
            certainty: self.certainty,
            freshness: self.freshness,
            lifecycle: self.lifecycle,
            importance: self.importance,
            title: self.title,
            summary: self.summary,
            content: self.content,
            permissions: self.permissions,
            relations: self.relations,
            references: self.references,
            metadata: self.metadata,
            extensions: self.extensions,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ContextBundle
// ═══════════════════════════════════════════════════════════════════════════

/// A collection of SCOs returned from a query.
///
/// This is the response type for `cpp/query` requests. It contains the
/// resolved SCOs along with metadata about the resolution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextBundle {
    /// The SCOs in this bundle, ordered by relevance.
    pub objects: Vec<ContextObject>,
    /// Total matching count (may exceed `objects.len()` if budget-limited).
    pub total_count: u32,
    /// Providers that contributed to this bundle.
    pub providers: Vec<ProviderId>,
    /// Resolution time in milliseconds.
    pub resolution_time_ms: u64,
    /// Whether any results were served from cache.
    pub from_cache: bool,
    /// Additional resolution metadata.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl ContextBundle {
    pub fn empty() -> Self {
        Self {
            objects: Vec::new(),
            total_count: 0,
            providers: Vec::new(),
            resolution_time_ms: 0,
            from_cache: false,
            metadata: IndexMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool { self.objects.is_empty() }
    pub fn len(&self) -> usize { self.objects.len() }
    pub fn iter(&self) -> std::slice::Iter<'_, ContextObject> { self.objects.iter() }
}

impl IntoIterator for ContextBundle {
    type Item = ContextObject;
    type IntoIter = std::vec::IntoIter<ContextObject>;
    fn into_iter(self) -> Self::IntoIter { self.objects.into_iter() }
}

impl<'a> IntoIterator for &'a ContextBundle {
    type Item = &'a ContextObject;
    type IntoIter = std::slice::Iter<'a, ContextObject>;
    fn into_iter(self) -> Self::IntoIter { self.objects.iter() }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ContextSession
// ═══════════════════════════════════════════════════════════════════════════

/// A coherent grouping of related SCOs representing a working context.
///
/// Instead of retrieving five separate objects, an agent retrieves one
/// session that contains everything relevant to what it's doing.
///
/// # Example: A Coding Session
///
/// ```text
/// ContextSession("coding-session-42")
///   ├── SCO: current branch (application/cpp.entity.repository)
///   ├── SCO: open PR (application/github.entity.pull_request)
///   ├── SCO: related issue (application/cpp.entity.issue)
///   ├── SCO: modified files (application/cpp.document.file × 3)
///   ├── SCO: Slack thread (application/slack.document.message)
///   └── SCO: upcoming standup (application/cpp.event.meeting)
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSession {
    /// Unique session identifier.
    pub id: SessionId,
    /// Human-readable name for this session.
    pub name: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The SCOs in this session, ordered by relevance.
    pub objects: Vec<ContextObject>,
    /// When this session was created.
    pub created_at: DateTime<Utc>,
    /// When this session was last updated.
    pub updated_at: DateTime<Utc>,
    /// When this session expires (if ephemeral).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Session-level metadata.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl ContextSession {
    /// Creates a new empty session.
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            name: name.into(),
            description: None,
            objects: Vec::new(),
            created_at: now,
            updated_at: now,
            expires_at: None,
            metadata: IndexMap::new(),
        }
    }

    /// Adds an SCO to this session.
    pub fn add(&mut self, object: ContextObject) {
        self.objects.push(object);
        self.updated_at = Utc::now();
    }

    /// Returns the number of SCOs in this session.
    pub fn len(&self) -> usize { self.objects.len() }

    /// Returns `true` if this session contains no SCOs.
    pub fn is_empty(&self) -> bool { self.objects.is_empty() }

    /// Returns `true` if this session has expired.
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }
}

impl std::fmt::Display for ContextSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Session({} — \"{}\" — {} objects)", self.id, self.name, self.objects.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn test_sco() -> ContextObject {
        ContextObjectBuilder::new(
            ContextUri::new("filesystem", "file", "src/main.rs"),
            ContextType::file(),
            ProviderId::new("filesystem"),
        )
        .title("main.rs")
        .summary("Application entry point")
        .content("fn main() {}")
        .certainty(Certainty::Authoritative)
        .freshness(Freshness::recent(Duration::minutes(5)))
        .importance(Importance::high())
        .metadata("language", serde_json::json!("rust"))
        .reference(Reference::source("file:///src/main.rs"))
        .build()
    }

    #[test]
    fn builder_creates_valid_sco() {
        let sco = test_sco();
        assert_eq!(sco.title, "main.rs");
        assert_eq!(sco.context_type, ContextType::file());
        assert_eq!(sco.certainty, Certainty::Authoritative);
        assert!(sco.id.as_str().starts_with("ctx_"));
        assert!(sco.has_content());
        assert!(!sco.is_expired());
    }

    #[test]
    fn sco_serialization_roundtrip() {
        let sco = test_sco();
        let json = serde_json::to_string_pretty(&sco).unwrap();
        let deserialized: ContextObject = serde_json::from_str(&json).unwrap();
        assert_eq!(sco.title, deserialized.title);
        assert_eq!(sco.uri, deserialized.uri);
        assert_eq!(sco.certainty, deserialized.certainty);
    }

    #[test]
    fn sco_display() {
        let sco = test_sco();
        let display = format!("{}", sco);
        assert!(display.contains("main.rs"));
        assert!(display.contains("authoritative"));
    }

    #[test]
    fn context_session() {
        let mut session = ContextSession::new("coding-session");
        assert!(session.is_empty());

        session.add(test_sco());
        assert_eq!(session.len(), 1);
        assert!(!session.is_empty());
        assert!(!session.is_expired());
    }

    #[test]
    fn context_bundle() {
        let mut bundle = ContextBundle::empty();
        assert!(bundle.is_empty());
        bundle.objects.push(test_sco());
        assert_eq!(bundle.len(), 1);
    }
}
