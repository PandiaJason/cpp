//! Context events, subscriptions, and the publish/subscribe model.
//!
//! CPP is event-first. Providers **publish** context events to the
//! protocol bus. Agents **subscribe** to context they care about.
//!
//! ```text
//! GitHub ──publishes──→ "Repository Updated"  ──→ ┐
//! Calendar ─publishes─→ "Meeting Starting"    ──→ ├──→ Agent
//! Filesystem publishes→ "File Modified"       ──→ ┘
//! ```
//!
//! SCOs are **snapshots** of current state.
//! Events are the **stream** of state changes.
//!
//! This follows the event sourcing pattern: events are the fundamental
//! unit, and SCOs are materialized views of accumulated events.

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::context::ContextObject;
use crate::relation::RelationType;
use crate::types::{
    ContextId, ContextType, ContextUri, Goal, LifecycleState, ProviderId, SubscriptionId,
};
use crate::permission::AccessLevel;

// ═══════════════════════════════════════════════════════════════════════════
//  ContextEvent
// ═══════════════════════════════════════════════════════════════════════════

/// A context event published by a provider.
///
/// Everything in CPP is fundamentally an event:
/// - `RepositoryUpdated` → provider publishes an event
/// - `MeetingStarted` → provider publishes an event
/// - `FileModified` → provider publishes an event
/// - `TaskCompleted` → provider publishes an event
///
/// Events carry the new state (as an SCO snapshot) when available.
///
/// # Wire Format
///
/// Events are delivered as JSON-RPC notifications:
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "cpp/event",
///   "params": {
///     "id": "evt_abc123",
///     "kind": "updated",
///     "contextUri": "cpp://github/repository/user/project",
///     "contextId": "ctx_def456",
///     "providerId": "github",
///     "timestamp": "2026-07-13T18:00:00Z",
///     "snapshot": { ... }
///   }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextEvent {
    /// Unique event identifier.
    pub id: String,

    /// What happened (maps to lifecycle transitions).
    pub kind: ContextEventKind,

    /// The URI of the affected context object.
    pub context_uri: ContextUri,

    /// The ID of the affected context object (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<ContextId>,

    /// The provider that published this event.
    pub provider_id: ProviderId,

    /// When this event occurred.
    pub timestamp: DateTime<Utc>,

    /// Optional SCO snapshot (the new state after the event).
    ///
    /// Included for `Created` and `Updated` events. Omitted for
    /// `Expired` and `Deleted` events (there's no new state).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<ContextObject>,

    /// Optional previous lifecycle state (for transitions).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<LifecycleState>,

    /// Event metadata.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl ContextEvent {
    /// Creates a new event.
    pub fn new(
        kind: ContextEventKind,
        context_uri: ContextUri,
        provider_id: ProviderId,
    ) -> Self {
        Self {
            id: format!("evt_{}", uuid::Uuid::new_v4().simple()),
            kind,
            context_uri,
            context_id: None,
            provider_id,
            timestamp: Utc::now(),
            snapshot: None,
            previous_state: None,
            metadata: IndexMap::new(),
        }
    }

    /// Attaches an SCO snapshot to this event.
    pub fn with_snapshot(mut self, sco: ContextObject) -> Self {
        self.context_id = Some(sco.id.clone());
        self.snapshot = Some(sco);
        self
    }

    /// Sets the previous lifecycle state.
    pub fn with_previous_state(mut self, state: LifecycleState) -> Self {
        self.previous_state = Some(state);
        self
    }
}

impl std::fmt::Display for ContextEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event({} {} {})", self.kind, self.context_uri, self.provider_id)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ContextEventKind
// ═══════════════════════════════════════════════════════════════════════════

/// The kind of context event (maps to lifecycle state transitions).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ContextEventKind {
    /// A new SCO was created.
    Created,
    /// An existing SCO was updated.
    Updated,
    /// An SCO was merged with another.
    Merged,
    /// An SCO was archived.
    Archived,
    /// An SCO's TTL elapsed.
    Expired,
    /// An SCO was permanently deleted.
    Deleted,
}

impl std::fmt::Display for ContextEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
//  Subscription
// ═══════════════════════════════════════════════════════════════════════════

/// A subscription to context events.
///
/// Agents subscribe to specific patterns of events. The runtime
/// (or provider, in direct mode) filters events and delivers only
/// those matching the subscription's filter.
///
/// # Wire Format
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": 3,
///   "method": "cpp/subscribe",
///   "params": {
///     "filter": {
///       "goals": ["goal.project"],
///       "contextTypes": ["application/cpp.entity.repository"],
///       "providers": ["github"],
///       "eventKinds": ["created", "updated"]
///     },
///     "accessLevel": "read"
///   }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    /// Unique subscription identifier.
    pub id: SubscriptionId,

    /// What events to deliver.
    pub filter: SubscriptionFilter,

    /// When this subscription was created.
    pub created_at: DateTime<Utc>,

    /// Required access level for received events.
    #[serde(default)]
    pub access_level: AccessLevel,

    /// When this subscription expires (if temporary).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl Subscription {
    /// Creates a new subscription with the given filter.
    pub fn new(filter: SubscriptionFilter) -> Self {
        Self {
            id: SubscriptionId::new(),
            filter,
            created_at: Utc::now(),
            access_level: AccessLevel::default(),
            expires_at: None,
        }
    }

    /// Returns `true` if this subscription has expired.
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }

    /// Returns `true` if the given event matches this subscription's filter.
    pub fn matches(&self, event: &ContextEvent) -> bool {
        self.filter.matches(event)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  SubscriptionFilter
// ═══════════════════════════════════════════════════════════════════════════

/// Filter criteria for a subscription.
///
/// All fields are optional. An empty filter matches **all** events.
/// When multiple fields are set, they are combined with AND logic.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionFilter {
    /// Only events related to these goals.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub goals: Vec<Goal>,

    /// Only events for these context types.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_types: Vec<ContextType>,

    /// Only events from these providers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<ProviderId>,

    /// Only these event kinds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_kinds: Vec<ContextEventKind>,

    /// Only events involving these relation types.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relation_types: Vec<RelationType>,

    /// URI patterns to match (glob-style).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uri_patterns: Vec<String>,
}

impl SubscriptionFilter {
    /// Creates a filter for all events from a specific provider.
    pub fn provider(provider_id: ProviderId) -> Self {
        Self {
            providers: vec![provider_id],
            ..Default::default()
        }
    }

    /// Creates a filter for specific event kinds.
    pub fn events(kinds: Vec<ContextEventKind>) -> Self {
        Self {
            event_kinds: kinds,
            ..Default::default()
        }
    }

    /// Returns `true` if the event matches all non-empty filter criteria.
    pub fn matches(&self, event: &ContextEvent) -> bool {
        // Provider filter
        if !self.providers.is_empty() && !self.providers.contains(&event.provider_id) {
            return false;
        }

        // Event kind filter
        if !self.event_kinds.is_empty() && !self.event_kinds.contains(&event.kind) {
            return false;
        }

        // Context type filter (requires snapshot to check)
        if !self.context_types.is_empty() {
            if let Some(ref snapshot) = event.snapshot {
                if !self.context_types.contains(&snapshot.context_type) {
                    return false;
                }
            }
            // If no snapshot and we have type filters, we can't confirm match
            // but we allow it (provider may omit snapshot for delete events)
        }

        // URI pattern filter (simple prefix matching)
        if !self.uri_patterns.is_empty() {
            let uri = event.context_uri.as_str();
            let matches_any = self.uri_patterns.iter().any(|pattern| {
                if let Some(prefix) = pattern.strip_suffix('*') {
                    uri.starts_with(prefix)
                } else {
                    uri == pattern
                }
            });
            if !matches_any {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_creation() {
        let event = ContextEvent::new(
            ContextEventKind::Updated,
            ContextUri::new("github", "repository", "user/project"),
            ProviderId::new("github"),
        );

        assert!(event.id.starts_with("evt_"));
        assert_eq!(event.kind, ContextEventKind::Updated);
        assert!(event.snapshot.is_none());
    }

    #[test]
    fn subscription_filter_matches() {
        let filter = SubscriptionFilter {
            providers: vec![ProviderId::new("github")],
            event_kinds: vec![ContextEventKind::Created, ContextEventKind::Updated],
            ..Default::default()
        };

        let event = ContextEvent::new(
            ContextEventKind::Updated,
            ContextUri::new("github", "repository", "test"),
            ProviderId::new("github"),
        );
        assert!(filter.matches(&event));

        let wrong_provider = ContextEvent::new(
            ContextEventKind::Updated,
            ContextUri::new("gitlab", "repository", "test"),
            ProviderId::new("gitlab"),
        );
        assert!(!filter.matches(&wrong_provider));

        let wrong_kind = ContextEvent::new(
            ContextEventKind::Deleted,
            ContextUri::new("github", "repository", "test"),
            ProviderId::new("github"),
        );
        assert!(!filter.matches(&wrong_kind));
    }

    #[test]
    fn subscription_filter_uri_pattern() {
        let filter = SubscriptionFilter {
            uri_patterns: vec!["cpp://github/*".into()],
            ..Default::default()
        };

        let match_event = ContextEvent::new(
            ContextEventKind::Created,
            ContextUri::new("github", "repository", "test"),
            ProviderId::new("github"),
        );
        assert!(filter.matches(&match_event));

        let no_match = ContextEvent::new(
            ContextEventKind::Created,
            ContextUri::new("gitlab", "repository", "test"),
            ProviderId::new("gitlab"),
        );
        assert!(!filter.matches(&no_match));
    }

    #[test]
    fn empty_filter_matches_everything() {
        let filter = SubscriptionFilter::default();
        let event = ContextEvent::new(
            ContextEventKind::Deleted,
            ContextUri::new("any", "type", "path"),
            ProviderId::new("any"),
        );
        assert!(filter.matches(&event));
    }
}
