//! Provider registry for tracking and matching available context providers.

use std::sync::Arc;
use dashmap::DashMap;

use cpp_core::types::{Goal, ContextType, ProviderId};
use cpp_sdk::ContextProvider;

/// Holds and queries active context providers.
#[derive(Clone)]
pub struct ProviderRegistry {
    providers: Arc<DashMap<ProviderId, Arc<dyn ContextProvider>>>,
}

impl ProviderRegistry {
    /// Creates a new empty provider registry.
    pub fn new() -> Self {
        Self {
            providers: Arc::new(DashMap::new()),
        }
    }

    /// Registers a provider with the runtime.
    pub fn register(&self, provider: Arc<dyn ContextProvider>) {
        let id = provider.manifest().id.clone();
        self.providers.insert(id, provider);
    }

    /// Removes a provider from the registry.
    pub fn unregister(&self, id: &ProviderId) {
        self.providers.remove(id);
    }

    /// Returns a provider by its identifier, if registered.
    pub fn get(&self, id: &ProviderId) -> Option<Arc<dyn ContextProvider>> {
        self.providers.get(id).map(|r| r.value().clone())
    }

    /// Finds all providers that declare capability to fulfill the given goal.
    pub fn find_by_goal(&self, goal: &Goal) -> Vec<Arc<dyn ContextProvider>> {
        self.providers
            .iter()
            .filter(|r| r.value().manifest().supports_goal(goal))
            .map(|r| r.value().clone())
            .collect()
        }

    /// Finds all providers that declare capability to produce the given context type.
    pub fn find_by_type(&self, ct: &ContextType) -> Vec<Arc<dyn ContextProvider>> {
        self.providers
            .iter()
            .filter(|r| r.value().manifest().supports_type(ct))
            .map(|r| r.value().clone())
            .collect()
    }

    /// Lists manifests of all registered providers.
    pub fn manifests(&self) -> Vec<cpp_core::manifest::ProviderManifest> {
        self.providers
            .iter()
            .map(|r| r.value().manifest().clone())
            .collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
