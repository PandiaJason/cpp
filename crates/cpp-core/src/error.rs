//! Protocol error codes and error types.
//!
//! CPP defines a set of error codes in the `-32000` to `-32099` range
//! (within the JSON-RPC 2.0 server error space).

use serde::{Deserialize, Serialize};
use std::fmt;

/// CPP protocol error codes.
///
/// These are in the JSON-RPC 2.0 server error range (`-32000` to `-32099`).
/// Standard JSON-RPC errors (`-32600` to `-32603`) also apply.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    /// `-32000` — Generic context error.
    ContextError = -32000,
    /// `-32001` — Provider not found or unavailable.
    ProviderNotFound = -32001,
    /// `-32002` — Context object not found at the given URI.
    ContextNotFound = -32002,
    /// `-32003` — Insufficient permissions for the requested access level.
    PermissionDenied = -32003,
    /// `-32004` — Query constraint or parameter validation failure.
    InvalidQuery = -32004,
    /// `-32005` — Provider timeout during context resolution.
    ProviderTimeout = -32005,
    /// `-32006` — Request exceeds configured rate limits.
    RateLimited = -32006,
    /// `-32007` — Subscription limit reached.
    SubscriptionLimitReached = -32007,
    /// `-32008` — Protocol version negotiation failed.
    VersionMismatch = -32008,
    /// `-32009` — Context budget exceeded (too many objects or bytes).
    BudgetExceeded = -32009,
}

impl ErrorCode {
    /// Returns the numeric JSON-RPC error code.
    pub fn code(&self) -> i32 {
        *self as i32
    }

    /// Returns the standard error message for this code.
    pub fn message(&self) -> &'static str {
        match self {
            Self::ContextError => "Context error",
            Self::ProviderNotFound => "Provider not found",
            Self::ContextNotFound => "Context not found",
            Self::PermissionDenied => "Permission denied",
            Self::InvalidQuery => "Invalid query",
            Self::ProviderTimeout => "Provider timeout",
            Self::RateLimited => "Rate limited",
            Self::SubscriptionLimitReached => "Subscription limit reached",
            Self::VersionMismatch => "Version mismatch",
            Self::BudgetExceeded => "Budget exceeded",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.message(), self.code())
    }
}

/// A structured CPP protocol error.
///
/// Follows the JSON-RPC 2.0 error object format with optional
/// structured `data` for additional details.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CppError {
    /// The error code.
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl CppError {
    /// Creates an error from a protocol error code.
    pub fn from_code(code: ErrorCode) -> Self {
        Self {
            code: code.code(),
            message: code.message().to_string(),
            data: None,
        }
    }

    /// Creates an error with a custom message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code.code(),
            message: message.into(),
            data: None,
        }
    }

    /// Attaches structured data to the error.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    // ── Convenience constructors ──

    pub fn provider_not_found(id: &str) -> Self {
        Self::new(
            ErrorCode::ProviderNotFound,
            format!("Provider '{}' not found or unavailable", id),
        )
    }

    pub fn context_not_found(uri: &str) -> Self {
        Self::new(
            ErrorCode::ContextNotFound,
            format!("No context object at URI '{}'", uri),
        )
    }

    pub fn permission_denied(detail: impl Into<String>) -> Self {
        Self::new(ErrorCode::PermissionDenied, detail)
    }

    pub fn invalid_query(detail: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidQuery, detail)
    }

    pub fn budget_exceeded(detail: impl Into<String>) -> Self {
        Self::new(ErrorCode::BudgetExceeded, detail)
    }
}

impl fmt::Display for CppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[CPP-{}] {}", self.code, self.message)
    }
}

impl std::error::Error for CppError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes() {
        assert_eq!(ErrorCode::ContextError.code(), -32000);
        assert_eq!(ErrorCode::BudgetExceeded.code(), -32009);
    }

    #[test]
    fn error_construction() {
        let err = CppError::provider_not_found("github");
        assert_eq!(err.code, -32001);
        assert!(err.message.contains("github"));
    }

    #[test]
    fn error_with_data() {
        let err = CppError::from_code(ErrorCode::RateLimited)
            .with_data(serde_json::json!({"retryAfterMs": 5000}));
        assert!(err.data.is_some());
    }

    #[test]
    fn error_display() {
        let err = CppError::from_code(ErrorCode::PermissionDenied);
        assert_eq!(format!("{}", err), "[CPP--32003] Permission denied");
    }
}
