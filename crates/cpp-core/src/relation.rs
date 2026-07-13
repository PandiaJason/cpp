//! Relationship types for the CPP context graph.
//!
//! Context objects are connected through typed, directed relationships
//! forming a knowledge graph. The [`RelationType`] enum defines the
//! protocol's standard relationship vocabulary, which is extensible
//! via the [`RelationType::Custom`] variant.
//!
//! Each [`Relation`] is a weighted, directed edge from a context object
//! to a target context URI, optionally carrying metadata.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::types::ContextUri;

// ---------------------------------------------------------------------------
// RelationType
// ---------------------------------------------------------------------------

/// The type of relationship between two context objects.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RelationType {
    /// The source contains the target (e.g., directory contains file, project contains task).
    Contains,
    /// The source is contained by the target (inverse of Contains).
    ContainedBy,
    /// The source references the target (e.g., commit message references issue).
    References,
    /// The source is referenced by the target (inverse of References).
    ReferencedBy,
    /// The source was created by the target (e.g., file created by person).
    CreatedBy,
    /// The source created the target (inverse of CreatedBy).
    Created,
    /// The source is owned by the target.
    OwnedBy,
    /// The source owns the target (inverse of OwnedBy).
    Owns,
    /// The source depends on the target (e.g., package depends on library).
    DependsOn,
    /// The source is depended on by the target (inverse of DependsOn).
    DependedBy,
    /// The source is preceded in time/sequence by the target (e.g., task B preceded by task A).
    PrecededBy,
    /// The source is followed in time/sequence by the target (inverse of PrecededBy).
    FollowedBy,
    /// The source is a part of the target (meronymy, e.g., team is part of organization).
    PartOf,
    /// The source has part the target (holonymy, inverse of PartOf).
    HasPart,
    /// The source is loosely associated with the target.
    AssociatedWith,
    /// The source is derived from the target (e.g., build artifact derived from source code).
    DerivedFrom,
    /// The source derives into the target (inverse of DerivedFrom).
    DerivedTo,
    /// Generic fallback relationship.
    RelatedTo,
    /// A custom relationship type defined by a provider.
    #[serde(untagged)]
    Custom(String),
}

impl RelationType {
    /// Returns the inverse relationship type, if one is defined.
    ///
    /// For custom relationship types, returns a custom relation with a
    /// suffix (e.g., `Custom("my_rel")` -> `Custom("inverse_my_rel")`).
    pub fn inverse(&self) -> Self {
        match self {
            Self::Contains => Self::ContainedBy,
            Self::ContainedBy => Self::Contains,
            Self::References => Self::ReferencedBy,
            Self::ReferencedBy => Self::References,
            Self::CreatedBy => Self::Created,
            Self::Created => Self::CreatedBy,
            Self::OwnedBy => Self::Owns,
            Self::Owns => Self::OwnedBy,
            Self::DependsOn => Self::DependedBy,
            Self::DependedBy => Self::DependsOn,
            Self::PrecededBy => Self::FollowedBy,
            Self::FollowedBy => Self::PrecededBy,
            Self::PartOf => Self::HasPart,
            Self::HasPart => Self::PartOf,
            Self::AssociatedWith => Self::AssociatedWith,
            Self::DerivedFrom => Self::DerivedTo,
            Self::DerivedTo => Self::DerivedFrom,
            Self::RelatedTo => Self::RelatedTo,
            Self::Custom(name) => {
                if let Some(stripped) = name.strip_prefix("inverse_") {
                    Self::Custom(stripped.to_string())
                } else {
                    Self::Custom(format!("inverse_{}", name))
                }
            }
        }
    }

    /// Returns `true` if the relationship is symmetric (i.e., its own inverse).
    pub fn is_symmetric(&self) -> bool {
        matches!(self, Self::AssociatedWith | Self::RelatedTo)
    }
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contains => write!(f, "contains"),
            Self::ContainedBy => write!(f, "contained_by"),
            Self::References => write!(f, "references"),
            Self::ReferencedBy => write!(f, "referenced_by"),
            Self::CreatedBy => write!(f, "created_by"),
            Self::Created => write!(f, "created"),
            Self::OwnedBy => write!(f, "owned_by"),
            Self::Owns => write!(f, "owns"),
            Self::DependsOn => write!(f, "depends_on"),
            Self::DependedBy => write!(f, "depended_by"),
            Self::PrecededBy => write!(f, "preceded_by"),
            Self::FollowedBy => write!(f, "followed_by"),
            Self::PartOf => write!(f, "part_of"),
            Self::HasPart => write!(f, "has_part"),
            Self::AssociatedWith => write!(f, "associated_with"),
            Self::DerivedFrom => write!(f, "derived_from"),
            Self::DerivedTo => write!(f, "derived_to"),
            Self::RelatedTo => write!(f, "related_to"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

// ---------------------------------------------------------------------------
// Relation
// ---------------------------------------------------------------------------

/// A directed, weighted edge between two context objects in the graph.
///
/// Relations connect context objects, enabling the runtime to traverse
/// the context graph and include related objects in query results based
/// on the `depth` and `includeRelations` query parameters.
///
/// # Example
///
/// ```rust
/// use cpp_core::relation::{Relation, RelationType};
/// use cpp_core::types::ContextUri;
///
/// let relation = Relation::new(
///     RelationType::Contains,
///     ContextUri::from("cpp://filesystem/file/src/main.rs"),
/// )
/// .with_weight(0.9);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relation {
    /// The type of relationship.
    #[serde(rename = "type")]
    pub relation_type: RelationType,

    /// The URI of the target context object.
    pub target_uri: ContextUri,

    /// Optional weight/strength of the relationship in `[0.0, 1.0]`.
    ///
    /// Higher weights indicate stronger relationships. Used by the
    /// ranking engine when traversing the context graph.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,

    /// Optional metadata attached to this relationship.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: IndexMap<String, serde_json::Value>,
}

impl Relation {
    /// Creates a new relation with the given type and target URI.
    pub fn new(relation_type: RelationType, target_uri: ContextUri) -> Self {
        Self {
            relation_type,
            target_uri,
            weight: None,
            metadata: IndexMap::new(),
        }
    }

    /// Sets the weight of this relation.
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight.clamp(0.0, 1.0));
        self
    }

    /// Adds metadata to this relation.
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

impl std::fmt::Display for Relation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.weight {
            Some(w) => write!(f, "--[{} ({:.2})]-->  {}", self.relation_type, w, self.target_uri),
            None => write!(f, "--[{}]-->  {}", self.relation_type, self.target_uri),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_type_inverse() {
        assert_eq!(
            RelationType::Contains.inverse(),
            RelationType::ContainedBy
        );
        assert_eq!(
            RelationType::Custom("inverse_my_rel".to_string()).inverse(),
            RelationType::Custom("my_rel".to_string())
        );
    }

    #[test]
    fn relation_serialization() {
        let rel = Relation::new(
            RelationType::Contains,
            ContextUri::from("cpp://filesystem/file/child"),
        )
        .with_weight(0.85);

        let json = serde_json::to_string(&rel).unwrap();
        assert!(json.contains("\"type\":\"contains\""));
        assert!(json.contains("\"targetUri\":\"cpp://filesystem/file/child\""));
        assert!(json.contains("\"weight\":0.85"));

        let deserialized: Relation = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.relation_type, RelationType::Contains);
        assert_eq!(deserialized.weight, Some(0.85));
    }

    #[test]
    fn relation_type_symmetry() {
        assert!(RelationType::AssociatedWith.is_symmetric());
        assert!(!RelationType::Contains.is_symmetric());
    }

    #[test]
    fn relation_weight_clamped() {
        let rel = Relation::new(RelationType::Contains, ContextUri::from("cpp://test"))
            .with_weight(1.5);
        assert_eq!(rel.weight, Some(1.0));

        let rel2 = Relation::new(RelationType::Contains, ContextUri::from("cpp://test"))
            .with_weight(-0.5);
        assert_eq!(rel2.weight, Some(0.0));
    }
}
