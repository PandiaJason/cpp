//! High-level client interface for agents consuming context.

use cpp_core::query::ContextQueryBuilder;
use cpp_core::types::{Goal, SessionId};

/// A client interface for agents to query context and manage sessions.
///
/// `CppClient` is the primary entry point for AI agents. It:
/// - Automatically binds queries to the current [`SessionId`].
/// - Offers fluent builders for constructing context queries (CRQs).
pub struct CppClient {
    session_id: SessionId,
}

impl CppClient {
    /// Creates a new `CppClient` with a fresh session identifier.
    pub fn new() -> Self {
        Self {
            session_id: SessionId::new(),
        }
    }

    /// Creates a client associated with an existing session identifier.
    pub fn with_session(session_id: SessionId) -> Self {
        Self { session_id }
    }

    /// Returns the session identifier for this client.
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Pre-configures a query builder for the given goal, automatically
    /// associating it with the client's session identifier.
    pub fn query(&self, goal: Goal) -> ContextQueryBuilder {
        ContextQueryBuilder::new(goal).session(self.session_id.clone())
    }
}

impl Default for CppClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpp_core::types::Goal;

    #[test]
    fn client_session_binding() {
        let client = CppClient::new();
        let query = client.query(Goal::code()).build();
        assert_eq!(query.session_id.as_ref(), Some(client.session_id()));
    }
}
