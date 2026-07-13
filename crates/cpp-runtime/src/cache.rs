//! Thread-safe semantic context object cache.

use std::sync::Arc;
use dashmap::DashMap;
use chrono::Utc;

use cpp_core::context::ContextObject;
use cpp_core::types::{ContextUri, FreshnessKind};

/// Standard cache for storing and validating SCO freshness.
///
/// Honoring the freshness metadata:
/// - Objects with `FreshnessKind::Immutable` are cached indefinitely.
/// - Other objects check `expires_at` or `stale_after` constraints.
#[derive(Clone)]
pub struct ContextCache {
    entries: Arc<DashMap<ContextUri, ContextObject>>,
}

impl ContextCache {
    /// Creates a new empty context cache.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
        }
    }

    /// Gets a cached object. Returns `None` if not found or if expired.
    pub fn get(&self, uri: &ContextUri) -> Option<ContextObject> {
        let entry = self.entries.get(uri)?;
        let obj = entry.value();

        if obj.is_expired() {
            drop(entry);
            self.entries.remove(uri);
            None
        } else {
            Some(obj.clone())
        }
    }

    /// Inserts or updates an SCO in the cache.
    pub fn insert(&self, object: ContextObject) {
        // Skip caching live objects since they change constantly
        if object.freshness.kind == FreshnessKind::Live {
            return;
        }
        self.entries.insert(object.uri.clone(), object);
    }

    /// Removes an object from the cache.
    pub fn invalidate(&self, uri: &ContextUri) {
        self.entries.remove(uri);
    }

    /// Prunes all expired cache entries.
    pub fn prune(&self) {
        let now = Utc::now();
        self.entries.retain(|_, obj| {
            if let Some(exp) = obj.expires_at {
                now <= exp
            } else {
                true
            }
        });
    }

    /// Clears the entire cache.
    pub fn clear(&self) {
        self.entries.clear();
    }
}

impl Default for ContextCache {
    fn default() -> Self {
        Self::new()
    }
}
