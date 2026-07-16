use std::sync::Arc;
use std::time::Instant;

use cpp_core::types::{Goal, ContextBudget, BudgetPreference, ContextType};
use cpp_runtime::{ContextCache, ContextResolver, ProviderRegistry};
use cpp_provider_filesystem::FilesystemProvider;
use cpp_provider_git::GitProvider;
use cpp_provider_datetime::DatetimeProvider;
use cpp_sdk::CppClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=====================================================");
    println!("=== CPP Context Window Budget Benchmark Run ===");
    println!("=====================================================");

    // Use current directory as workspace root
    let target_path = std::env::current_dir()?;
    println!("Workspace Root: {}\n", target_path.display());

    // 1. Initialize Orchestrator components
    let registry = ProviderRegistry::new();
    let cache = ContextCache::new();

    // 2. Register local providers
    let fs_provider = Arc::new(FilesystemProvider::new(&target_path));
    registry.register(fs_provider);

    let git_provider = Arc::new(GitProvider::new(&target_path));
    registry.register(git_provider);

    let dt_provider = Arc::new(DatetimeProvider::new());
    registry.register(dt_provider);

    let resolver = ContextResolver::new(registry, cache);
    let client = CppClient::new();

    // ==========================================
    // TEST 1: Unbudgeted query (Request all workspace files)
    // ==========================================
    let query_unbudgeted = client.query(Goal::code())
        .scope_current()
        .include(ContextType::file())
        .max_results(50)
        .build();

    let start_unbudgeted = Instant::now();
    let bundle_unbudgeted = resolver.resolve_query(&query_unbudgeted).await?;
    let duration_unbudgeted = start_unbudgeted.elapsed();

    // Calculate content size of unbudgeted files
    let mut size_unbudgeted = 0;
    for obj in &bundle_unbudgeted.objects {
        if let Some(content) = obj.get_metadata("sizeBytes") {
            if let Some(bytes) = content.as_u64() {
                size_unbudgeted += bytes;
            }
        }
    }

    // ==========================================
    // TEST 2: Budgeted query (Request files with a strict 2KB limit)
    // ==========================================
    let budget = ContextBudget {
        max_bytes: Some(2000),
        max_objects: Some(5),
        max_latency_ms: None,
        prefer: BudgetPreference::Quality,
    };

    let query_budgeted = client.query(Goal::code())
        .scope_current()
        .include(ContextType::file())
        .max_results(50)
        .budget(budget)
        .build();

    let start_budgeted = Instant::now();
    let bundle_budgeted = resolver.resolve_query(&query_budgeted).await?;
    let duration_budgeted = start_budgeted.elapsed();

    let mut size_budgeted = 0;
    for obj in &bundle_budgeted.objects {
        if let Some(content) = obj.get_metadata("sizeBytes") {
            if let Some(bytes) = content.as_u64() {
                size_budgeted += bytes;
            }
        }
    }

    // ==========================================
    // DISPLAY RESULTS
    // ==========================================
    println!("-----------------------------------------------------");
    println!("| Metric             | Unbudgeted       | Budgeted        |");
    println!("-----------------------------------------------------");
    println!("| Object Count       | {:<16} | {:<15} |", bundle_unbudgeted.objects.len(), bundle_budgeted.objects.len());
    println!("| Total Content Size | {:<11} bytes | {:<10} bytes |", size_unbudgeted, size_budgeted);
    println!("| Resolution Time    | {:<13?} | {:<15?} |", duration_unbudgeted, duration_budgeted);
    println!("-----------------------------------------------------");

    if size_unbudgeted > 0 {
        let savings = (1.0 - (size_budgeted as f64 / size_unbudgeted as f64)) * 100.0;
        println!("\n>>> Token/Bytes volume reduced by {:.2}% at source!", savings);
        println!(">>> Prompt cost for LLM is {:.2}% cheaper!", savings);
    }

    Ok(())
}
