//! Context Request Query (CRQ) — the query model for CPP.
//!
//! CRQ replaces traditional query languages (SQL, GraphQL) with an
//! intent-based model. The agent expresses **what** context it needs
//! using registered goals, scoping, and constraints — not **how** to
//! retrieve it.
//!
//! # Design Philosophy
//!
//! ```text
//! Need:     goal.project
//! Scope:    current
//! Include:  repositories, meetings, tasks
//! Budget:   50KB, 100 objects, <2s
//! ```
//!
//! The runtime determines which providers can fulfill the goal,
//! respects the budget, and returns the best subset.
//!
//! # CRQ vs SQL/GraphQL
//!
//! | | SQL | GraphQL | CRQ |
//! |:--|:--|:--|:--|
//! | **Knows schema** | Yes | Yes | No |
//! | **Knows source** | Yes | Yes | No |
//! | **Expresses** | Structure | Shape | Intent |
//! | **Routing** | Manual | Manual | Automatic |

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::permission::AccessLevel;
use crate::relation::RelationType;
use crate::types::{ContextBudget, ContextType, Duration, Goal, ProviderId, SessionId};

// ═══════════════════════════════════════════════════════════════════════════
//  ContextQuery (CRQ)
// ═══════════════════════════════════════════════════════════════════════════

/// A Context Request Query (CRQ).
///
/// This is the primary request type for the `cpp/query` method. An agent
/// sends a CRQ to the runtime, which routes it to capable providers.
///
/// # Example (JSON)
///
/// ```json
/// {
///   "goal": "goal.project",
///   "scope": {
///     "current": true,
///     "recent": "7d",
///     "providers": ["github", "jira"]
///   },
///   "include": ["application/cpp.entity.repository", "application/cpp.event.meeting"],
///   "exclude": ["application/cpp.document.email"],
///   "constraints": {
///     "importance": { "min": 0.5 },
///     "tags": ["backend", "api"]
///   },
///   "depth": 2,
///   "maxResults": 50,
///   "accessLevel": "read",
///   "budget": {
///     "maxBytes": 51200,
///     "maxObjects": 100,
///     "prefer": "quality"
///   }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextQuery {
    /// The registered goal this query is requesting (e.g., `goal.project`).
    pub goal: Goal,

    /// Scoping constraints that narrow the search space.
    #[serde(default)]
    pub scope: QueryScope,

    /// Context types to include in results.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<ContextType>,

    /// Context types to exclude from results.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<ContextType>,

    /// Additional filtering constraints.
    #[serde(default)]
    pub constraints: QueryConstraints,

    /// How many relationship hops to traverse (0 = direct only).
    #[serde(default)]
    pub depth: u32,

    /// Which relationship types to follow during traversal.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub follow_relations: Vec<RelationType>,

    /// Maximum number of SCOs to return.
    #[serde(default = "default_max_results")]
    pub max_results: u32,

    /// Pagination offset.
    #[serde(default)]
    pub offset: u32,

    /// Required access level.
    #[serde(default)]
    pub access_level: AccessLevel,

    /// Context window budget constraints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<ContextBudget>,

    /// Session to associate this query with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,

    /// Provider-specific hints (non-binding optimization suggestions).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub hints: IndexMap<String, serde_json::Value>,
}

fn default_max_results() -> u32 {
    50
}

// ═══════════════════════════════════════════════════════════════════════════
//  QueryScope
// ═══════════════════════════════════════════════════════════════════════════

/// Scoping constraints that narrow the search space of a CRQ.
///
/// ```json
/// {
///   "current": true,
///   "recent": "7d",
///   "providers": ["github", "jira"],
///   "uriPatterns": ["cpp://github/*"]
/// }
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryScope {
    /// If true, restrict to context relevant to the current activity/session.
    #[serde(default)]
    pub current: bool,

    /// Only include context updated within this recency window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent: Option<Duration>,

    /// Restrict to specific providers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<ProviderId>,

    /// URI glob patterns to match (e.g., `"cpp://github/*"`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uri_patterns: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  QueryConstraints
// ═══════════════════════════════════════════════════════════════════════════

/// Filtering constraints within a CRQ.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryConstraints {
    /// Filter by importance range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<RangeConstraint>,

    /// Only include SCOs with these tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Attribute-based filters (provider-specific).
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub attributes: IndexMap<String, serde_json::Value>,

    /// Only include SCOs with this certainty level or higher.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_certainty: Option<crate::types::Certainty>,

    /// Only include SCOs with this freshness kind or fresher.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshness_kind: Option<crate::types::FreshnessKind>,
}

/// A numeric range constraint with optional min and max bounds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeConstraint {
    /// Minimum value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Maximum value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

impl RangeConstraint {
    /// Creates a constraint requiring at least `min`.
    pub fn at_least(min: f64) -> Self {
        Self { min: Some(min), max: None }
    }
    /// Creates a constraint requiring at most `max`.
    pub fn at_most(max: f64) -> Self {
        Self { min: None, max: Some(max) }
    }
    /// Creates a constraint requiring a value in `[min, max]`.
    pub fn between(min: f64, max: f64) -> Self {
        Self { min: Some(min), max: Some(max) }
    }
    /// Returns `true` if the value satisfies this constraint.
    pub fn matches(&self, value: f64) -> bool {
        let above_min = self.min.map_or(true, |m| value >= m);
        let below_max = self.max.map_or(true, |m| value <= m);
        above_min && below_max
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ContextQueryBuilder
// ═══════════════════════════════════════════════════════════════════════════

/// Fluent builder for constructing CRQ queries.
///
/// # Example
///
/// ```rust
/// use cpp_core::query::ContextQueryBuilder;
/// use cpp_core::types::*;
///
/// let query = ContextQueryBuilder::new(Goal::project())
///     .scope_current()
///     .scope_recent(Duration::days(7))
///     .include(ContextType::repository())
///     .include(ContextType::meeting())
///     .importance_at_least(0.5)
///     .depth(2)
///     .budget(ContextBudget::standard())
///     .build();
/// ```
pub struct ContextQueryBuilder {
    goal: Goal,
    scope: QueryScope,
    include: Vec<ContextType>,
    exclude: Vec<ContextType>,
    constraints: QueryConstraints,
    depth: u32,
    follow_relations: Vec<RelationType>,
    max_results: u32,
    offset: u32,
    access_level: AccessLevel,
    budget: Option<ContextBudget>,
    session_id: Option<SessionId>,
    hints: IndexMap<String, serde_json::Value>,
}

impl ContextQueryBuilder {
    /// Creates a new query builder with the specified goal.
    pub fn new(goal: Goal) -> Self {
        Self {
            goal,
            scope: QueryScope::default(),
            include: Vec::new(),
            exclude: Vec::new(),
            constraints: QueryConstraints::default(),
            depth: 0,
            follow_relations: Vec::new(),
            max_results: 50,
            offset: 0,
            access_level: AccessLevel::default(),
            budget: None,
            session_id: None,
            hints: IndexMap::new(),
        }
    }

    /// Scopes to current activity/session.
    pub fn scope_current(mut self) -> Self {
        self.scope.current = true;
        self
    }

    /// Scopes to a recency window.
    pub fn scope_recent(mut self, d: Duration) -> Self {
        self.scope.recent = Some(d);
        self
    }

    /// Restricts to a specific provider.
    pub fn scope_provider(mut self, p: ProviderId) -> Self {
        self.scope.providers.push(p);
        self
    }

    /// Adds a URI pattern to scope.
    pub fn scope_uri_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.scope.uri_patterns.push(pattern.into());
        self
    }

    /// Includes a context type in results.
    pub fn include(mut self, ct: ContextType) -> Self {
        self.include.push(ct);
        self
    }

    /// Excludes a context type from results.
    pub fn exclude(mut self, ct: ContextType) -> Self {
        self.exclude.push(ct);
        self
    }

    /// Sets minimum importance.
    pub fn importance_at_least(mut self, min: f64) -> Self {
        self.constraints.importance = Some(RangeConstraint::at_least(min));
        self
    }

    /// Adds a tag filter.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.constraints.tags.push(tag.into());
        self
    }

    /// Sets relationship traversal depth.
    pub fn depth(mut self, d: u32) -> Self {
        self.depth = d;
        self
    }

    /// Adds a relation type to follow during traversal.
    pub fn follow(mut self, rt: RelationType) -> Self {
        self.follow_relations.push(rt);
        self
    }

    /// Sets maximum result count.
    pub fn max_results(mut self, n: u32) -> Self {
        self.max_results = n;
        self
    }

    /// Sets the pagination offset.
    pub fn offset(mut self, n: u32) -> Self {
        self.offset = n;
        self
    }

    /// Sets the required access level.
    pub fn access_level(mut self, level: AccessLevel) -> Self {
        self.access_level = level;
        self
    }

    /// Sets the context window budget.
    pub fn budget(mut self, b: ContextBudget) -> Self {
        self.budget = Some(b);
        self
    }

    /// Associates this query with a session.
    pub fn session(mut self, id: SessionId) -> Self {
        self.session_id = Some(id);
        self
    }

    /// Adds a provider-specific hint.
    pub fn hint(mut self, k: impl Into<String>, v: serde_json::Value) -> Self {
        self.hints.insert(k.into(), v);
        self
    }

    /// Builds the CRQ.
    pub fn build(self) -> ContextQuery {
        ContextQuery {
            goal: self.goal,
            scope: self.scope,
            include: self.include,
            exclude: self.exclude,
            constraints: self.constraints,
            depth: self.depth,
            follow_relations: self.follow_relations,
            max_results: self.max_results,
            offset: self.offset,
            access_level: self.access_level,
            budget: self.budget,
            session_id: self.session_id,
            hints: self.hints,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn query_builder_basic() {
        let query = ContextQueryBuilder::new(Goal::project())
            .scope_current()
            .scope_recent(Duration::days(7))
            .include(ContextType::repository())
            .include(ContextType::meeting())
            .importance_at_least(0.5)
            .depth(2)
            .max_results(20)
            .build();

        assert_eq!(query.goal, Goal::project());
        assert!(query.scope.current);
        assert_eq!(query.include.len(), 2);
        assert_eq!(query.depth, 2);
        assert_eq!(query.max_results, 20);
    }

    #[test]
    fn query_with_budget() {
        let query = ContextQueryBuilder::new(Goal::code())
            .budget(ContextBudget::standard())
            .build();

        let budget = query.budget.unwrap();
        assert_eq!(budget.max_bytes, Some(128_000));
        assert_eq!(budget.prefer, BudgetPreference::Quality);
    }

    #[test]
    fn query_serialization() {
        let query = ContextQueryBuilder::new(Goal::project())
            .scope_current()
            .include(ContextType::repository())
            .depth(1)
            .build();

        let json = serde_json::to_value(&query).unwrap();
        assert_eq!(json["goal"], "goal.project");
        assert_eq!(json["scope"]["current"], true);
        assert_eq!(json["depth"], 1);
    }

    #[test]
    fn range_constraint() {
        let c = RangeConstraint::at_least(0.5);
        assert!(c.matches(0.5));
        assert!(c.matches(0.8));
        assert!(!c.matches(0.3));

        let c2 = RangeConstraint::between(0.2, 0.8);
        assert!(c2.matches(0.5));
        assert!(!c2.matches(0.1));
        assert!(!c2.matches(0.9));
    }
}
