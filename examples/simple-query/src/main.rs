//! Simple query resolution example.
//!
//! This example shows how to set up the reference runtime, register providers,
//! construct a Context Request Query (CRQ), and resolve it to a ranked Context Bundle.

use std::sync::Arc;

use cpp_core::types::Goal;
use cpp_runtime::{ContextCache, ContextResolver, ProviderRegistry};
use cpp_provider_filesystem::FilesystemProvider;
use cpp_provider_git::GitProvider;
use cpp_provider_datetime::DatetimeProvider;
use cpp_sdk::CppClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CPP Semantic Context Resolution Demo ===");

    // 1. Initialize Registry and Cache (Orchestration Layer)
    let registry = ProviderRegistry::new();
    let cache = ContextCache::new();

    // 2. Register Providers (Systems of Record)
    // Filesystem provider serving current workspace
    let fs_provider = Arc::new(FilesystemProvider::new("."));
    registry.register(fs_provider);

    // Git provider serving current workspace repository
    let git_provider = Arc::new(GitProvider::new("."));
    registry.register(git_provider);

    // Datetime provider serving current temporal context
    let dt_provider = Arc::new(DatetimeProvider::new());
    registry.register(dt_provider);

    println!("Registered providers:");
    for manifest in registry.manifests() {
        println!(" - {} (Capabilities: {} goals, {} types)", manifest.name, manifest.capabilities.goals.len(), manifest.capabilities.context_types.len());
    }

    // 3. Build the Resolver
    let resolver = ContextResolver::new(registry, cache);

    // 4. Create an Agent Client & Session
    let client = CppClient::new();
    println!("\nAgent session initialized: {}", client.session_id());

    // 5. Construct a Context Request Query (CRQ)
    // "I need coding context relevant to current state, including code files and git status"
    let query = client
        .query(Goal::code())
        .scope_current()
        .include(cpp_core::types::ContextType::file())
        .include(cpp_core::types::ContextType::repository())
        .include(cpp_core::types::ContextType::branch())
        .max_results(5)
        .build();

    println!("\nSending Context Request Query (CRQ):");
    println!(" - Goal: {}", query.goal);
    println!(" - Depth: {}", query.depth);
    println!(" - Include types: {}", query.include.iter().map(|t| t.short_name()).collect::<Vec<_>>().join(", "));

    // 6. Resolve the query
    let bundle = resolver.resolve_query(&query).await?;

    println!("\nResolved Context Bundle ({} objects retrieved from {:?} in {}ms):", 
        bundle.objects.len(), 
        bundle.providers.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
        bundle.resolution_time_ms
    );

    // 7. Inspect the ranked Semantic Context Objects (SCOs)
    for (i, obj) in bundle.objects.iter().enumerate() {
        println!("\n[SCO #{}] {} ({})", i + 1, obj.title, obj.context_type);
        println!(" - URI: {}", obj.uri);
        println!(" - Certainty: {}", obj.certainty);
        println!(" - Freshness: {}", obj.freshness.kind);
        if let Some(ref summary) = obj.summary {
            println!(" - Summary: {}", summary);
        }
        if let Some(lang) = obj.get_metadata("language") {
            println!(" - Language: {}", lang);
        }
    }

    println!("\n=== Demo Completed ===");
    Ok(())
}
