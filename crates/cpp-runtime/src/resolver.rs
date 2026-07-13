//! Reference context query resolution and orchestration engine.

use std::collections::HashSet;

use chrono::Utc;
use cpp_core::context::{ContextBundle, ContextObject};
use cpp_core::error::CppError;
use cpp_core::query::ContextQuery;
use cpp_core::types::{ContextUri, ProviderId};

use crate::cache::ContextCache;
use crate::registry::ProviderRegistry;

/// Orchestrates routing, merging, caching, ranking, and graph traversal.
pub struct ContextResolver {
    registry: ProviderRegistry,
    cache: ContextCache,
}

impl ContextResolver {
    /// Creates a new resolver.
    pub fn new(registry: ProviderRegistry, cache: ContextCache) -> Self {
        Self { registry, cache }
    }

    /// Resolves a context request query (CRQ) across registered providers.
    pub async fn resolve_query(&self, query: &ContextQuery) -> Result<ContextBundle, CppError> {
        let start_time = Utc::now();
        let providers = self.registry.find_by_goal(&query.goal);

        if providers.is_empty() {
            return Err(CppError::new(
                cpp_core::error::ErrorCode::ProviderNotFound,
                format!("No providers registered for goal '{}'", query.goal),
            ));
        }

        let mut all_objects = Vec::new();
        let mut contributing_providers = Vec::new();

        // 1. Fetch from matching providers
        for provider in providers {
            contributing_providers.push(provider.manifest().id.clone());
            match provider.query(query).await {
                Ok(bundle) => {
                    for obj in bundle.objects {
                        // Cache it if appropriate
                        self.cache.insert(obj.clone());
                        all_objects.push(obj);
                    }
                }
                Err(e) => {
                    tracing::warn!("Provider query failed: {}", e);
                }
            }
        }

        // 2. Traversal/Graph Expansion
        if query.depth > 0 && !all_objects.is_empty() {
            let mut visited = HashSet::new();
            for obj in &all_objects {
                visited.insert(obj.uri.clone());
            }

            let mut current_level = all_objects.clone();
            for _ in 0..query.depth {
                let mut next_level = Vec::new();
                for obj in &current_level {
                    for relation in &obj.relations {
                        if !visited.contains(&relation.target_uri) {
                            visited.insert(relation.target_uri.clone());
                            // Try resolving target URI (cache first, then registry)
                            if let Ok(target_obj) = self.resolve_uri(&relation.target_uri, query.depth - 1, query.access_level.clone()).await {
                                next_level.push(target_obj);
                            }
                        }
                    }
                }
                if next_level.is_empty() {
                    break;
                }
                all_objects.extend(next_level.clone());
                current_level = next_level;
            }
        }

        // 3. Filter by include/exclude types
        if !query.include.is_empty() {
            all_objects.retain(|obj| query.include.contains(&obj.context_type));
        }
        if !query.exclude.is_empty() {
            all_objects.retain(|obj| !query.exclude.contains(&obj.context_type));
        }

        // 4. Ranking / Ordering
        // Sort by Certainty, then Importance, then Recency
        all_objects.sort_by(|a, b| {
            let cert_cmp = b.certainty.cmp(&a.certainty);
            if cert_cmp != std::cmp::Ordering::Equal {
                return cert_cmp;
            }
            let imp_cmp = b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal);
            if imp_cmp != std::cmp::Ordering::Equal {
                return imp_cmp;
            }
            b.updated_at.cmp(&a.updated_at)
        });

        // 5. Apply context window negotiation budget limits
        if let Some(ref budget) = query.budget {
            let mut budget_objects = Vec::new();
            let mut total_bytes = 0;

            for obj in all_objects {
                if let Some(max_objects) = budget.max_objects {
                    if budget_objects.len() >= max_objects as usize {
                        break;
                    }
                }

                // Compute size of content if present
                let size = obj.content.as_ref().map(|c| c.len() as u64).unwrap_or(0);
                if let Some(max_bytes) = budget.max_bytes {
                    if total_bytes + size > max_bytes {
                        // Skip objects that exceed remaining byte budget
                        continue;
                    }
                }

                total_bytes += size;
                budget_objects.push(obj);
            }
            all_objects = budget_objects;
        } else {
            // Apply simple max results limit
            all_objects.truncate(query.max_results as usize);
        }

        let duration = Utc::now() - start_time;

        Ok(ContextBundle {
            total_count: all_objects.len() as u32,
            providers: contributing_providers,
            resolution_time_ms: duration.num_milliseconds() as u64,
            from_cache: false,
            metadata: Default::default(),
            objects: all_objects,
        })
    }

    /// Resolves a single SCO by URI, checking cache first.
    pub async fn resolve_uri(
        &self,
        uri: &ContextUri,
        depth: u32,
        access_level: cpp_core::AccessLevel,
    ) -> Result<ContextObject, CppError> {
        // Check cache
        if let Some(obj) = self.cache.get(uri) {
            return Ok(obj);
        }

        // Find provider from URI (cpp://<provider>/...)
        let provider_name = uri.provider().ok_or_else(|| {
            CppError::new(
                cpp_core::error::ErrorCode::InvalidQuery,
                format!("Invalid Context URI: {}", uri),
            )
        })?;

        let provider_id = ProviderId::new(provider_name);
        let provider = self.registry.get(&provider_id).ok_or_else(|| {
            CppError::provider_not_found(provider_name)
        })?;

        let obj = provider.resolve(uri).await?;

        // Context traversal for relations if depth > 0
        if depth > 0 && !obj.relations.is_empty() {
            let mut resolved_relations = Vec::new();
            for relation in &obj.relations {
                if let Ok(target_obj) = Box::pin(self.resolve_uri(&relation.target_uri, depth - 1, access_level.clone())).await {
                    resolved_relations.push(target_obj);
                }
            }
            // For now, resolved relations are kept in cache for subsequent lookup
            for target in resolved_relations {
                self.cache.insert(target);
            }
        }

        self.cache.insert(obj.clone());
        Ok(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use async_trait::async_trait;
    use cpp_core::context::ContextObjectBuilder;
    use cpp_core::manifest::{ProviderCapabilities, ProviderManifest};
    use cpp_core::types::*;
    use cpp_sdk::ContextProvider;

    struct MockProvider {
        manifest: ProviderManifest,
        objects: Vec<ContextObject>,
    }

    #[async_trait]
    impl ContextProvider for MockProvider {
        fn manifest(&self) -> &ProviderManifest { &self.manifest }
        async fn query(&self, _q: &ContextQuery) -> Result<ContextBundle, CppError> {
            Ok(ContextBundle {
                objects: self.objects.clone(),
                total_count: self.objects.len() as u32,
                providers: vec![self.manifest.id.clone()],
                resolution_time_ms: 0,
                from_cache: false,
                metadata: Default::default(),
            })
        }
        async fn resolve(&self, uri: &ContextUri) -> Result<ContextObject, CppError> {
            self.objects.iter()
                .find(|o| o.uri == *uri)
                .cloned()
                .ok_or_else(|| CppError::context_not_found(uri.as_str()))
        }
    }

    #[tokio::test]
    async fn resolution_and_ranking() {
        let registry = ProviderRegistry::new();
        let cache = ContextCache::new();

        let obj1 = ContextObjectBuilder::new(
            ContextUri::new("mock", "file", "a.txt"),
            ContextType::file(),
            ProviderId::new("mock"),
        )
        .title("a.txt")
        .certainty(Certainty::Derived)
        .build();

        let obj2 = ContextObjectBuilder::new(
            ContextUri::new("mock", "file", "b.txt"),
            ContextType::file(),
            ProviderId::new("mock"),
        )
        .title("b.txt")
        .certainty(Certainty::Authoritative) // Higher certainty, should rank first
        .build();

        let provider = Arc::new(MockProvider {
            manifest: ProviderManifest::new(
                ProviderId::new("mock"),
                "Mock Provider",
                ProviderCapabilities::basic(vec![ContextType::file()], vec![Goal::code()]),
            ),
            objects: vec![obj1, obj2],
        });

        registry.register(provider);
        let resolver = ContextResolver::new(registry, cache);

        let query = cpp_core::query::ContextQueryBuilder::new(Goal::code()).build();
        let bundle = resolver.resolve_query(&query).await.unwrap();

        assert_eq!(bundle.len(), 2);
        assert_eq!(bundle.objects[0].title, "b.txt"); // Authoritative first
    }
}
