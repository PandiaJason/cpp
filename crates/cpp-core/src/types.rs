//! Shared primitive types for the Context Provider Protocol.
//!
//! This module defines the foundational newtypes used throughout CPP.
//! Each type wraps a primitive value with domain-specific validation
//! and semantics, ensuring type safety across the protocol.
//!
//! # Key Design Decisions
//!
//! - **ContextUri** (`cpp://provider/class.type/path`) — Every context object
//!   is globally addressable, like HTTP URIs for documents.
//! - **ContextType** (`application/cpp.class.type`) — MIME-like registry
//!   prevents namespace collisions across providers.
//! - **ContextClass** — Base taxonomy (Entity, Document, Event, Collection,
//!   Reference) giving every provider the same mental model.
//! - **Certainty** — Replaces floating-point confidence scores with semantic
//!   categories (Authoritative, Derived, Estimated).
//! - **Freshness** — Every object carries freshness metadata so agents know
//!   how current the data is.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════
//  IDENTIFIERS
// ═══════════════════════════════════════════════════════════════════════════

/// Unique identifier for a context object, prefixed with `ctx_`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContextId(String);

impl ContextId {
    pub fn new() -> Self {
        Self(format!("ctx_{}", Uuid::new_v4().simple()))
    }
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ContextId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ContextId {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for ContextId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for a context provider (e.g., `"github"`, `"filesystem"`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProviderId(String);

impl ProviderId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl From<String> for ProviderId {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for ProviderId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for a session, prefixed with `ses_`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    pub fn new() -> Self {
        Self(format!("ses_{}", Uuid::new_v4().simple()))
    }
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for a subscription, prefixed with `sub_`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubscriptionId(String);

impl SubscriptionId {
    pub fn new() -> Self {
        Self(format!("sub_{}", Uuid::new_v4().simple()))
    }
    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SubscriptionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl From<String> for SubscriptionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for SubscriptionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  CONTEXT URI
// ═══════════════════════════════════════════════════════════════════════════

/// A globally-addressable URI for a context object.
///
/// Every context object in CPP is addressable via a URI following the
/// scheme `cpp://<provider>/<type>/<path>`.
///
/// # Examples
///
/// ```text
/// cpp://github/repository/openai-mcp
/// cpp://calendar/event/2930
/// cpp://gmail/thread/83291
/// cpp://filesystem/file/src/main.rs
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContextUri(String);

impl ContextUri {
    /// Creates a new `ContextUri` from components.
    pub fn new(
        provider: impl AsRef<str>,
        context_type: impl AsRef<str>,
        path: impl AsRef<str>,
    ) -> Self {
        Self(format!(
            "cpp://{}/{}/{}",
            provider.as_ref(),
            context_type.as_ref(),
            path.as_ref()
        ))
    }

    /// Creates a `ContextUri` from a raw string.
    pub fn from_string(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Parses into (provider, type, path). Returns `None` if not `cpp://`.
    pub fn parse(&self) -> Option<(&str, &str, &str)> {
        let rest = self.0.strip_prefix("cpp://")?;
        let mut parts = rest.splitn(3, '/');
        let provider = parts.next()?;
        let context_type = parts.next()?;
        let path = parts.next().unwrap_or("");
        Some((provider, context_type, path))
    }

    pub fn provider(&self) -> Option<&str> {
        self.parse().map(|(p, _, _)| p)
    }
    pub fn context_type(&self) -> Option<&str> {
        self.parse().map(|(_, t, _)| t)
    }
    pub fn path(&self) -> Option<&str> {
        self.parse().map(|(_, _, p)| p)
    }
}

impl fmt::Display for ContextUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl From<String> for ContextUri {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for ContextUri {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  CONTEXT CLASS (base taxonomy)
// ═══════════════════════════════════════════════════════════════════════════

/// The base class of a context object.
///
/// Every context type inherits from exactly one class. This gives all
/// providers the same mental model:
///
/// | Class | Examples |
/// |:------|:---------|
/// | `Entity` | Person, Project, Company, Repository, Team |
/// | `Document` | File, Note, Page, Email, Message |
/// | `Event` | Meeting, Commit, TaskUpdate, Notification |
/// | `Collection` | Folder, Board, Channel, Workspace, Sprint |
/// | `Reference` | Link, Bookmark, Citation, URL |
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ContextClass {
    /// A discrete, identifiable thing (person, project, repo).
    Entity,
    /// A piece of content (file, email, note, page).
    Document,
    /// Something that happened or is happening (meeting, commit).
    Event,
    /// A grouping of other objects (folder, board, channel).
    Collection,
    /// A pointer to another resource (link, bookmark, citation).
    Reference,
}

impl fmt::Display for ContextClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entity => write!(f, "entity"),
            Self::Document => write!(f, "document"),
            Self::Event => write!(f, "event"),
            Self::Collection => write!(f, "collection"),
            Self::Reference => write!(f, "reference"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  CONTEXT TYPE (MIME-like registry)
// ═══════════════════════════════════════════════════════════════════════════

/// The semantic type of a context object, using a MIME-like registry format.
///
/// Three-level hierarchy: `application/<namespace>.<class>.<type>`
///
/// **Protocol-defined**: `application/cpp.<class>.<type>`
/// ```text
/// application/cpp.entity.person
/// application/cpp.entity.project
/// application/cpp.entity.repository
/// application/cpp.document.file
/// application/cpp.document.email
/// application/cpp.event.meeting
/// application/cpp.event.commit
/// application/cpp.collection.folder
/// ```
///
/// **Provider-defined**: `application/<provider>.<class>.<type>`
/// ```text
/// application/github.entity.pull_request
/// application/notion.document.page
/// application/slack.collection.channel
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContextType(String);

impl ContextType {
    /// Creates from a full MIME-like string.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Creates a protocol-defined type: `application/cpp.<class>.<type>`.
    pub fn cpp(class: ContextClass, type_name: impl AsRef<str>) -> Self {
        Self(format!("application/cpp.{}.{}", class, type_name.as_ref()))
    }

    /// Creates a provider-defined type: `application/<provider>.<class>.<type>`.
    pub fn provider(
        provider: impl AsRef<str>,
        class: ContextClass,
        type_name: impl AsRef<str>,
    ) -> Self {
        Self(format!(
            "application/{}.{}.{}",
            provider.as_ref(),
            class,
            type_name.as_ref()
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the short type name (after the last dot).
    pub fn short_name(&self) -> &str {
        self.0.rsplit_once('.').map(|(_, n)| n).unwrap_or(&self.0)
    }

    /// Returns the namespace (e.g., `"cpp"`, `"github"`).
    pub fn namespace(&self) -> Option<&str> {
        let after = self.0.strip_prefix("application/")?;
        after.split('.').next()
    }

    /// Returns the context class, if parseable.
    pub fn class(&self) -> Option<ContextClass> {
        let after = self.0.strip_prefix("application/")?;
        let mut parts = after.split('.');
        let _ns = parts.next()?;
        let class_str = parts.next()?;
        match class_str {
            "entity" => Some(ContextClass::Entity),
            "document" => Some(ContextClass::Document),
            "event" => Some(ContextClass::Event),
            "collection" => Some(ContextClass::Collection),
            "reference" => Some(ContextClass::Reference),
            _ => None,
        }
    }

    /// Returns `true` if this is a protocol-defined type (`application/cpp.*`).
    pub fn is_protocol_type(&self) -> bool {
        self.0.starts_with("application/cpp.")
    }

    // ── Well-known protocol types ──

    // Entities
    pub fn person() -> Self { Self::cpp(ContextClass::Entity, "person") }
    pub fn project() -> Self { Self::cpp(ContextClass::Entity, "project") }
    pub fn repository() -> Self { Self::cpp(ContextClass::Entity, "repository") }
    pub fn team() -> Self { Self::cpp(ContextClass::Entity, "team") }
    pub fn company() -> Self { Self::cpp(ContextClass::Entity, "company") }
    pub fn branch() -> Self { Self::cpp(ContextClass::Entity, "branch") }

    // Documents
    pub fn file() -> Self { Self::cpp(ContextClass::Document, "file") }
    pub fn email() -> Self { Self::cpp(ContextClass::Document, "email") }
    pub fn note() -> Self { Self::cpp(ContextClass::Document, "note") }
    pub fn page() -> Self { Self::cpp(ContextClass::Document, "page") }
    pub fn message() -> Self { Self::cpp(ContextClass::Document, "message") }

    // Events
    pub fn meeting() -> Self { Self::cpp(ContextClass::Event, "meeting") }
    pub fn commit() -> Self { Self::cpp(ContextClass::Event, "commit") }
    pub fn task_update() -> Self { Self::cpp(ContextClass::Event, "task_update") }
    pub fn notification() -> Self { Self::cpp(ContextClass::Event, "notification") }
    pub fn temporal() -> Self { Self::cpp(ContextClass::Event, "temporal") }

    // Collections
    pub fn folder() -> Self { Self::cpp(ContextClass::Collection, "folder") }
    pub fn channel() -> Self { Self::cpp(ContextClass::Collection, "channel") }
    pub fn board() -> Self { Self::cpp(ContextClass::Collection, "board") }
    pub fn workspace() -> Self { Self::cpp(ContextClass::Collection, "workspace") }

    // References
    pub fn link() -> Self { Self::cpp(ContextClass::Reference, "link") }
    pub fn bookmark() -> Self { Self::cpp(ContextClass::Reference, "bookmark") }
}

impl fmt::Display for ContextType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl From<String> for ContextType {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for ContextType {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  GOAL (registry-based, not freeform strings)
// ═══════════════════════════════════════════════════════════════════════════

/// A registered goal that an agent requests context for.
///
/// Goals follow the same MIME-like registry pattern as context types.
/// Protocol-defined goals use `goal.*`, providers use `<provider>.*`.
///
/// ```text
/// goal.project          # "I need project context"
/// goal.person           # "I need people context"
/// goal.code             # "I need coding context"
/// goal.calendar         # "I need calendar context"
/// goal.task             # "I need task context"
/// goal.email            # "I need email context"
/// goal.communication    # "I need communication context"
/// goal.document         # "I need document context"
///
/// github.pull_requests  # Provider-specific goal
/// jira.sprint           # Provider-specific goal
/// ```
///
/// This prevents fragmentation — there's ONE way to ask for "project context"
/// instead of `project`, `current_project`, `active_project`, `workspace`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Goal(String);

impl Goal {
    /// Creates a new `Goal` from a registry string.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns `true` if this is a protocol-defined goal (`goal.*`).
    pub fn is_protocol_goal(&self) -> bool {
        self.0.starts_with("goal.")
    }

    /// Returns the namespace (e.g., `"goal"`, `"github"`).
    pub fn namespace(&self) -> Option<&str> {
        self.0.split('.').next()
    }

    // ── Well-known protocol goals ──

    pub fn project() -> Self { Self("goal.project".into()) }
    pub fn person() -> Self { Self("goal.person".into()) }
    pub fn code() -> Self { Self("goal.code".into()) }
    pub fn calendar() -> Self { Self("goal.calendar".into()) }
    pub fn task() -> Self { Self("goal.task".into()) }
    pub fn email() -> Self { Self("goal.email".into()) }
    pub fn communication() -> Self { Self("goal.communication".into()) }
    pub fn document() -> Self { Self("goal.document".into()) }
}

impl fmt::Display for Goal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl From<String> for Goal {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for Goal {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  CERTAINTY (replaces confidence floats)
// ═══════════════════════════════════════════════════════════════════════════

/// How certain the provider is about this context object.
///
/// Replaces floating-point confidence scores with semantic categories.
/// A GitHub repository's existence isn't "0.84 confident" — it's
/// **authoritative**. An AI-generated summary is **derived**.
///
/// | Certainty | Meaning | Example |
/// |:----------|:--------|:--------|
/// | `Authoritative` | Provider IS the system of record | GitHub repo metadata |
/// | `Derived` | Computed from authoritative data | AI summary of a document |
/// | `Estimated` | Prediction, inference, heuristic | "You might want this PR" |
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Certainty {
    /// Source of truth. The provider IS the system of record.
    #[default]
    Authoritative,
    /// Computed or derived from authoritative data (summaries, aggregations).
    Derived,
    /// Prediction, inference, or heuristic guess.
    Estimated,
}

impl Certainty {
    /// Returns a numeric rank for ordering (higher = more certain).
    pub fn rank(&self) -> u8 {
        match self {
            Self::Estimated => 0,
            Self::Derived => 1,
            Self::Authoritative => 2,
        }
    }
}

impl PartialOrd for Certainty {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Certainty {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}



impl fmt::Display for Certainty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authoritative => write!(f, "authoritative"),
            Self::Derived => write!(f, "derived"),
            Self::Estimated => write!(f, "estimated"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  FRESHNESS
// ═══════════════════════════════════════════════════════════════════════════

/// How fresh this context object's data is.
///
/// Every context object SHOULD carry freshness metadata so that agents
/// and runtimes know how current the data is.
///
/// - A live calendar event: `kind=Live, staleAfter=5m`
/// - A git commit: `kind=Immutable` (never changes)
/// - A cached email thread: `kind=Recent, staleAfter=1h, lastVerified=...`
/// - Static documentation: out of scope for CPP (that's knowledge, not context)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Freshness {
    /// The freshness category.
    pub kind: FreshnessKind,

    /// Duration after which this object should be considered stale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_after: Option<Duration>,

    /// Hard time-to-live. After this, the object MUST be re-fetched or discarded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<Duration>,

    /// When the data was last verified against the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_verified: Option<DateTime<Utc>>,
}

impl Freshness {
    /// Creates a `Live` freshness (real-time data).
    pub fn live() -> Self {
        Self {
            kind: FreshnessKind::Live,
            stale_after: Some(Duration::minutes(5)),
            ttl: None,
            last_verified: Some(Utc::now()),
        }
    }

    /// Creates a `Recent` freshness with the given stale-after duration.
    pub fn recent(stale_after: Duration) -> Self {
        Self {
            kind: FreshnessKind::Recent,
            stale_after: Some(stale_after),
            ttl: None,
            last_verified: Some(Utc::now()),
        }
    }

    /// Creates a `Cached` freshness with stale-after and TTL.
    pub fn cached(stale_after: Duration, ttl: Duration) -> Self {
        Self {
            kind: FreshnessKind::Cached,
            stale_after: Some(stale_after),
            ttl: Some(ttl),
            last_verified: Some(Utc::now()),
        }
    }

    /// Creates an `Immutable` freshness (data never changes).
    pub fn immutable() -> Self {
        Self {
            kind: FreshnessKind::Immutable,
            stale_after: None,
            ttl: None,
            last_verified: None,
        }
    }
}

impl Default for Freshness {
    fn default() -> Self {
        Self::recent(Duration::hours(1))
    }
}

/// The freshness category of a context object.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FreshnessKind {
    /// Real-time data, always current (active meeting, live dashboard).
    Live,
    /// Recently fetched, may become stale (email thread, recent commits).
    Recent,
    /// Served from cache, check `stale_after` (previously fetched data).
    Cached,
    /// Never changes (git commit SHA, published RFC, archived event).
    Immutable,
}

impl fmt::Display for FreshnessKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Live => write!(f, "live"),
            Self::Recent => write!(f, "recent"),
            Self::Cached => write!(f, "cached"),
            Self::Immutable => write!(f, "immutable"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  IMPORTANCE
// ═══════════════════════════════════════════════════════════════════════════

/// An importance score in the range `[0.0, 1.0]`.
///
/// Importance represents how significant a piece of context is relative
/// to other context objects. Providers assign importance based on their
/// domain-specific heuristics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Importance(f64);

impl Importance {
    pub fn new(value: f64) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&value) {
            return Err(format!("Importance must be in [0.0, 1.0], got {}", value));
        }
        Ok(Self(value))
    }
    pub fn critical() -> Self { Self(1.0) }
    pub fn high() -> Self { Self(0.8) }
    pub fn normal() -> Self { Self(0.5) }
    pub fn low() -> Self { Self(0.2) }
    pub fn trivial() -> Self { Self(0.0) }
    pub fn value(&self) -> f64 { self.0 }
}

impl Default for Importance {
    fn default() -> Self { Self(0.5) }
}

impl fmt::Display for Importance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl PartialOrd for Importance {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  DURATION
// ═══════════════════════════════════════════════════════════════════════════

/// A human-friendly duration: `"30s"`, `"15m"`, `"24h"`, `"7d"`, `"4w"`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Duration(String);

impl Duration {
    pub fn new(spec: impl Into<String>) -> Self { Self(spec.into()) }
    pub fn seconds(n: u64) -> Self { Self(format!("{}s", n)) }
    pub fn minutes(n: u64) -> Self { Self(format!("{}m", n)) }
    pub fn hours(n: u64) -> Self { Self(format!("{}h", n)) }
    pub fn days(n: u64) -> Self { Self(format!("{}d", n)) }
    pub fn weeks(n: u64) -> Self { Self(format!("{}w", n)) }
    pub fn as_str(&self) -> &str { &self.0 }

    /// Converts to `chrono::TimeDelta`. Supports single-unit durations.
    pub fn to_chrono(&self) -> Option<chrono::TimeDelta> {
        let s = self.0.trim();
        if s.is_empty() { return None; }
        let (num_str, unit) = s.split_at(s.len() - 1);
        let n: i64 = num_str.parse().ok()?;
        match unit {
            "s" => chrono::TimeDelta::try_seconds(n),
            "m" => chrono::TimeDelta::try_minutes(n),
            "h" => chrono::TimeDelta::try_hours(n),
            "d" => chrono::TimeDelta::try_days(n),
            "w" => chrono::TimeDelta::try_weeks(n),
            _ => None,
        }
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl From<String> for Duration {
    fn from(s: String) -> Self { Self(s) }
}
impl From<&str> for Duration {
    fn from(s: &str) -> Self { Self(s.to_string()) }
}

// ═══════════════════════════════════════════════════════════════════════════
//  CONTEXT BUDGET (window negotiation)
// ═══════════════════════════════════════════════════════════════════════════

/// Budget constraints for context window negotiation.
///
/// The agent tells the provider/runtime: "I can handle at most this much."
/// The provider returns the **best subset** within the budget.
///
/// ```json
/// {
///   "maxBytes": 51200,
///   "maxObjects": 200,
///   "maxLatencyMs": 100,
///   "prefer": "quality"
/// }
/// ```
///
/// This is the AI-native killer feature — agents operate under token
/// limits and latency budgets, and CPP respects that.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextBudget {
    /// Maximum total bytes of context content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<u64>,

    /// Maximum number of context objects to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_objects: Option<u32>,

    /// Maximum acceptable latency in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_latency_ms: Option<u32>,

    /// What to optimize for when the budget is tight.
    #[serde(default)]
    pub prefer: BudgetPreference,
}

impl ContextBudget {
    /// Creates a budget with no constraints (unlimited).
    pub fn unlimited() -> Self {
        Self {
            max_bytes: None,
            max_objects: None,
            max_latency_ms: None,
            prefer: BudgetPreference::Quality,
        }
    }

    /// Creates a budget with typical AI agent constraints.
    pub fn standard() -> Self {
        Self {
            max_bytes: Some(128_000),   // ~128 KB
            max_objects: Some(100),
            max_latency_ms: Some(5_000), // 5 seconds
            prefer: BudgetPreference::Quality,
        }
    }
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self::standard()
    }
}

/// What to optimize for when the context budget is tight.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BudgetPreference {
    /// Fewer, more relevant objects (default for reasoning).
    #[default]
    Quality,
    /// Fastest response, may sacrifice depth.
    Speed,
    /// Broadest coverage across providers and types.
    Coverage,
}

impl fmt::Display for BudgetPreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Quality => write!(f, "quality"),
            Self::Speed => write!(f, "speed"),
            Self::Coverage => write!(f, "coverage"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  LIFECYCLE STATE
// ═══════════════════════════════════════════════════════════════════════════

/// The lifecycle state of a context object.
///
/// Every context object has a lifecycle state that the runtime
/// uses for cache consistency and event dispatch.
///
/// ```text
/// Created → Updated → Archived → Expired → Deleted
///                    ↘ Merged
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum LifecycleState {
    /// Object has been created and is active.
    #[default]
    Created,
    /// Object has been modified since creation.
    Updated,
    /// Object has been merged with another object.
    Merged,
    /// Object has been archived (no longer active but preserved).
    Archived,
    /// Object's TTL has elapsed.
    Expired,
    /// Object has been permanently removed.
    Deleted,
}

impl fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Updated => write!(f, "updated"),
            Self::Merged => write!(f, "merged"),
            Self::Archived => write!(f, "archived"),
            Self::Expired => write!(f, "expired"),
            Self::Deleted => write!(f, "deleted"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  PROTOCOL VERSION
// ═══════════════════════════════════════════════════════════════════════════

/// Protocol version in `CPP/major.minor` format, like `HTTP/1.1`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolVersion {
    pub major: u32,
    pub minor: u32,
}

impl ProtocolVersion {
    /// The current protocol version: CPP/0.1 (Draft).
    pub fn current() -> Self {
        Self { major: 0, minor: 1 }
    }

    /// Checks if this version is compatible with another.
    /// Compatible if same major version and minor >= other's minor.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CPP/{}.{}", self.major, self.minor)
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::current()
    }
}

/// The current protocol version string.
pub const PROTOCOL_VERSION: &str = "CPP/0.1";
/// The protocol name.
pub const PROTOCOL_NAME: &str = "Context Provider Protocol";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_id_has_prefix() {
        assert!(ContextId::new().as_str().starts_with("ctx_"));
    }

    #[test]
    fn context_uri_construction_and_parsing() {
        let uri = ContextUri::new("github", "repository", "openai/mcp");
        assert_eq!(uri.as_str(), "cpp://github/repository/openai/mcp");

        let (p, t, path) = uri.parse().unwrap();
        assert_eq!(p, "github");
        assert_eq!(t, "repository");
        assert_eq!(path, "openai/mcp");
    }

    #[test]
    fn context_type_three_level_hierarchy() {
        let t = ContextType::person();
        assert_eq!(t.as_str(), "application/cpp.entity.person");
        assert_eq!(t.namespace(), Some("cpp"));
        assert_eq!(t.class(), Some(ContextClass::Entity));
        assert_eq!(t.short_name(), "person");
        assert!(t.is_protocol_type());
    }

    #[test]
    fn context_type_provider_defined() {
        let t = ContextType::provider("github", ContextClass::Entity, "pull_request");
        assert_eq!(t.as_str(), "application/github.entity.pull_request");
        assert!(!t.is_protocol_type());
        assert_eq!(t.namespace(), Some("github"));
    }

    #[test]
    fn goal_registry() {
        assert_eq!(Goal::project().as_str(), "goal.project");
        assert!(Goal::project().is_protocol_goal());
        assert!(!Goal::new("github.pull_requests").is_protocol_goal());
    }

    #[test]
    fn certainty_ordering() {
        assert!(Certainty::Authoritative > Certainty::Derived);
        assert!(Certainty::Derived > Certainty::Estimated);
    }

    #[test]
    fn freshness_kinds() {
        let live = Freshness::live();
        assert_eq!(live.kind, FreshnessKind::Live);

        let immut = Freshness::immutable();
        assert_eq!(immut.kind, FreshnessKind::Immutable);
        assert!(immut.stale_after.is_none());
    }

    #[test]
    fn context_budget_standard() {
        let budget = ContextBudget::standard();
        assert_eq!(budget.max_bytes, Some(128_000));
        assert_eq!(budget.max_objects, Some(100));
    }

    #[test]
    fn lifecycle_state_default() {
        assert_eq!(LifecycleState::default(), LifecycleState::Created);
    }

    #[test]
    fn protocol_version() {
        let v = ProtocolVersion::current();
        assert_eq!(v.to_string(), "CPP/0.1");
        assert!(v.is_compatible_with(&ProtocolVersion { major: 0, minor: 0 }));
        assert!(v.is_compatible_with(&ProtocolVersion { major: 0, minor: 1 }));
        assert!(!v.is_compatible_with(&ProtocolVersion { major: 1, minor: 0 }));
    }

    #[test]
    fn importance_validation() {
        assert!(Importance::new(0.5).is_ok());
        assert!(Importance::new(-0.1).is_err());
        assert!(Importance::new(1.1).is_err());
    }

    #[test]
    fn serialization_roundtrips() {
        let uri = ContextUri::new("github", "repository", "test");
        let json = serde_json::to_string(&uri).unwrap();
        assert_eq!(json, "\"cpp://github/repository/test\"");

        let ct = ContextType::file();
        let json = serde_json::to_string(&ct).unwrap();
        assert_eq!(json, "\"application/cpp.document.file\"");

        let goal = Goal::project();
        let json = serde_json::to_string(&goal).unwrap();
        assert_eq!(json, "\"goal.project\"");

        let cert = Certainty::Authoritative;
        let json = serde_json::to_string(&cert).unwrap();
        assert_eq!(json, "\"authoritative\"");
    }
}
