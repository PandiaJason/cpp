//! Permission and access control types for CPP.
//!
//! CPP uses a **capability-based** permission model inspired by IETF RFC 9237.
//! Instead of checking "who is the agent?" (identity-based), CPP checks
//! "what is the agent allowed to see?" (capability-based).
//!
//! The core abstraction is the [`CapabilityToken`] — a signed, scoped,
//! time-limited token that encodes what context an agent may access.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{ContextType, ProviderId};

// ═══════════════════════════════════════════════════════════════════════════
//  AccessLevel
// ═══════════════════════════════════════════════════════════════════════════

/// The level of access granted to a context consumer.
///
/// Access levels form an ordered hierarchy:
/// `MetadataOnly < Summarize < Read < Sensitive < Admin`
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AccessLevel {
    /// Only identifiers and metadata (title, type, timestamps).
    MetadataOnly,
    /// Summarized or redacted content.
    Summarize,
    /// Full content access — the standard level.
    Read,
    /// Includes sensitive fields (PII, credentials). Requires explicit grant.
    Sensitive,
    /// Full administrative access. Reserved for system operators.
    Admin,
}

impl AccessLevel {
    fn rank(&self) -> u8 {
        match self {
            Self::MetadataOnly => 0,
            Self::Summarize => 1,
            Self::Read => 2,
            Self::Sensitive => 3,
            Self::Admin => 4,
        }
    }

    /// Returns `true` if this level is sufficient for the `required` level.
    pub fn satisfies(&self, required: &AccessLevel) -> bool {
        self.rank() >= required.rank()
    }

    pub fn has_content_access(&self) -> bool { self.rank() >= Self::Read.rank() }
    pub fn has_summary_access(&self) -> bool { self.rank() >= Self::Summarize.rank() }
}

impl PartialOrd for AccessLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AccessLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl Default for AccessLevel {
    fn default() -> Self { Self::Read }
}

impl std::fmt::Display for AccessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MetadataOnly => write!(f, "metadata_only"),
            Self::Summarize => write!(f, "summarize"),
            Self::Read => write!(f, "read"),
            Self::Sensitive => write!(f, "sensitive"),
            Self::Admin => write!(f, "admin"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ContextCapability
// ═══════════════════════════════════════════════════════════════════════════

/// A single capability grant — what context an agent may access from
/// which provider(s) at what access level.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextCapability {
    /// The maximum access level granted.
    pub access_level: AccessLevel,

    /// Optional provider scope. `None` = all providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderId>,

    /// Context types this capability applies to. Empty = all types.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_types: Vec<ContextType>,

    /// When this capability expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    /// Custom scope identifiers for fine-grained access control.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

impl ContextCapability {
    pub fn new(access_level: AccessLevel) -> Self {
        Self {
            access_level,
            provider: None,
            context_types: Vec::new(),
            expires_at: None,
            scopes: Vec::new(),
        }
    }

    pub fn with_provider(mut self, provider: ProviderId) -> Self {
        self.provider = Some(provider); self
    }

    pub fn with_context_type(mut self, ct: ContextType) -> Self {
        self.context_types.push(ct); self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at); self
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into()); self
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| Utc::now() > exp).unwrap_or(false)
    }

    pub fn matches_provider(&self, provider_id: &ProviderId) -> bool {
        self.provider.as_ref().is_none_or(|p| p == provider_id)
    }

    pub fn matches_context_type(&self, ct: &ContextType) -> bool {
        self.context_types.is_empty() || self.context_types.contains(ct)
    }

    /// Returns `true` if this capability authorizes the request.
    pub fn authorizes(
        &self,
        provider_id: &ProviderId,
        context_type: &ContextType,
        required_level: &AccessLevel,
    ) -> bool {
        !self.is_expired()
            && self.matches_provider(provider_id)
            && self.matches_context_type(context_type)
            && self.access_level.satisfies(required_level)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  CapabilityToken
// ═══════════════════════════════════════════════════════════════════════════

/// A signed, time-limited token encoding an agent's context permissions.
///
/// Capability tokens are the primary authorization mechanism. An agent
/// presents a token and the runtime uses it to filter which SCOs the
/// agent may receive.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityToken {
    /// Unique token identifier.
    pub id: String,
    /// Capabilities granted by this token.
    pub capabilities: Vec<ContextCapability>,
    /// When this token was issued.
    pub issued_at: DateTime<Utc>,
    /// When this token expires.
    pub expires_at: DateTime<Utc>,
    /// The issuer (runtime ID or authority).
    pub issuer: String,
    /// The subject (agent ID or name).
    pub subject: String,
    /// Optional cryptographic signature (base64-encoded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl CapabilityToken {
    pub fn new(
        subject: impl Into<String>,
        issuer: impl Into<String>,
        duration: chrono::TimeDelta,
        capabilities: Vec<ContextCapability>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("tok_{}", uuid::Uuid::new_v4().simple()),
            capabilities,
            issued_at: now,
            expires_at: now + duration,
            issuer: issuer.into(),
            subject: subject.into(),
            signature: None,
        }
    }

    pub fn is_expired(&self) -> bool { Utc::now() > self.expires_at }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.capabilities.is_empty()
    }

    pub fn max_access_level(&self) -> AccessLevel {
        self.capabilities.iter()
            .filter(|c| !c.is_expired())
            .map(|c| c.access_level.clone())
            .max()
            .unwrap_or(AccessLevel::MetadataOnly)
    }

    /// Returns `true` if any capability authorizes the request.
    pub fn authorizes(
        &self,
        provider_id: &ProviderId,
        context_type: &ContextType,
        required_level: &AccessLevel,
    ) -> bool {
        self.is_valid()
            && self.capabilities.iter()
                .any(|c| c.authorizes(provider_id, context_type, required_level))
    }

    /// Creates an attenuated (restricted) delegation token.
    pub fn attenuate(
        &self,
        new_subject: impl Into<String>,
        max_level: AccessLevel,
        duration: chrono::TimeDelta,
    ) -> Self {
        let restricted_caps: Vec<ContextCapability> = self.capabilities.iter()
            .filter(|c| !c.is_expired())
            .map(|c| {
                let level = if c.access_level > max_level {
                    max_level.clone()
                } else {
                    c.access_level.clone()
                };
                ContextCapability {
                    access_level: level,
                    provider: c.provider.clone(),
                    context_types: c.context_types.clone(),
                    expires_at: c.expires_at,
                    scopes: c.scopes.clone(),
                }
            })
            .collect();

        Self::new(new_subject, &self.issuer, duration, restricted_caps)
    }
}

impl std::fmt::Display for CapabilityToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "CapabilityToken({} for {} — {} caps, expires {})",
            self.id, self.subject, self.capabilities.len(), self.expires_at
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    #[test]
    fn access_level_ordering() {
        assert!(AccessLevel::Admin > AccessLevel::Sensitive);
        assert!(AccessLevel::Read > AccessLevel::Summarize);
        assert!(AccessLevel::Summarize > AccessLevel::MetadataOnly);
    }

    #[test]
    fn access_level_satisfies() {
        assert!(AccessLevel::Read.satisfies(&AccessLevel::Read));
        assert!(AccessLevel::Read.satisfies(&AccessLevel::MetadataOnly));
        assert!(!AccessLevel::Read.satisfies(&AccessLevel::Sensitive));
    }

    #[test]
    fn capability_authorization() {
        let cap = ContextCapability::new(AccessLevel::Read)
            .with_provider(ProviderId::new("github"))
            .with_context_type(ContextType::repository());

        assert!(cap.authorizes(
            &ProviderId::new("github"),
            &ContextType::repository(),
            &AccessLevel::Read,
        ));

        // Wrong provider
        assert!(!cap.authorizes(
            &ProviderId::new("gitlab"),
            &ContextType::repository(),
            &AccessLevel::Read,
        ));

        // Wrong type
        assert!(!cap.authorizes(
            &ProviderId::new("github"),
            &ContextType::email(),
            &AccessLevel::Read,
        ));
    }

    #[test]
    fn token_creation_and_validity() {
        let token = CapabilityToken::new(
            "agent-1", "runtime",
            TimeDelta::try_hours(1).unwrap(),
            vec![ContextCapability::new(AccessLevel::Read)],
        );

        assert!(token.is_valid());
        assert!(token.id.starts_with("tok_"));
        assert_eq!(token.max_access_level(), AccessLevel::Read);
    }

    #[test]
    fn token_attenuation() {
        let token = CapabilityToken::new(
            "agent-1", "runtime",
            TimeDelta::try_hours(1).unwrap(),
            vec![ContextCapability::new(AccessLevel::Sensitive)],
        );

        let restricted = token.attenuate("sub-agent", AccessLevel::Read, TimeDelta::try_minutes(30).unwrap());
        assert_eq!(restricted.max_access_level(), AccessLevel::Read);
    }
}
