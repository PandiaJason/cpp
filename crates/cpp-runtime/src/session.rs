//! Context session tracker.

use std::sync::Arc;
use dashmap::DashMap;

use cpp_core::context::ContextSession;
use cpp_core::types::SessionId;

/// Manages and tracks active semantic context sessions.
#[derive(Clone)]
pub struct SessionTracker {
    sessions: Arc<DashMap<SessionId, ContextSession>>,
}

impl SessionTracker {
    /// Creates a new session tracker.
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
        }
    }

    /// Creates and registers a new context session.
    pub fn create_session(&self, name: impl Into<String>) -> ContextSession {
        let session = ContextSession::new(name);
        self.sessions.insert(session.id.clone(), session.clone());
        session
    }

    /// Retrieves a session by its identifier.
    pub fn get(&self, id: &SessionId) -> Option<ContextSession> {
        self.sessions.get(id).map(|s| s.value().clone())
    }

    /// Updates or inserts a session.
    pub fn update(&self, session: ContextSession) {
        self.sessions.insert(session.id.clone(), session);
    }

    /// Removes a session.
    pub fn remove(&self, id: &SessionId) {
        self.sessions.remove(id);
    }

    /// Removes expired sessions from tracking.
    pub fn prune_expired(&self) {
        self.sessions.retain(|_, s| !s.is_expired());
    }
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new()
    }
}
