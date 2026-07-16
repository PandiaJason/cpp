use std::sync::Arc;
use std::time::Instant;

use cpp_core::types::{Goal, ContextBudget, BudgetPreference};
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

    // Parse target directory from command line arguments, defaulting to current dir
    let args: Vec<String> = std::env::args().collect();
    let target_dir = if args.len() > 1 {
        args[1].clone()
    } else {
        ".".to_string()
    };

    let target_path = std::path::PathBuf::from(&target_dir).canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(&target_dir));

    println!("Workspace Target: {}\n", target_path.display());

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
    // TEST 1: Unbudgeted Context Query (Retrieve all matching context)
    // ==========================================
    let query_unbudgeted = client.query(Goal::document())
        .scope_current()
        .max_results(100)
        .build();

    let start_unbudgeted = Instant::now();
    let bundle_unbudgeted = resolver.resolve_query(&query_unbudgeted).await?;
    let duration_unbudgeted = start_unbudgeted.elapsed();

    // Calculate total serialized size of all resolved context objects
    let mut size_unbudgeted = 0;
    for obj in &bundle_unbudgeted.objects {
        // Size is either file bytes or metadata text length
        if let Some(content_size) = obj.get_metadata("sizeBytes").and_then(|v| v.as_u64()) {
            size_unbudgeted += content_size;
        } else {
            size_unbudgeted += obj.title.len() as u64;
            if let Some(ref summary) = obj.summary {
                size_unbudgeted += summary.len() as u64;
            }
        }
    }

    // ==========================================
    // TEST 2: Budget-Constrained Query (Request same query with strict limit)
    // ==========================================
    let budget = ContextBudget {
        max_bytes: Some(250), // Strict 250 bytes limit to force truncating doc2.txt
        max_objects: Some(5),
        max_latency_ms: None,
        prefer: BudgetPreference::Quality,
    };

    let query_budgeted = client.query(Goal::document())
        .scope_current()
        .max_results(100)
        .budget(budget)
        .build();

    let start_budgeted = Instant::now();
    let bundle_budgeted = resolver.resolve_query(&query_budgeted).await?;
    let duration_budgeted = start_budgeted.elapsed();

    let mut size_budgeted = 0;
    for obj in &bundle_budgeted.objects {
        if let Some(content_size) = obj.get_metadata("sizeBytes").and_then(|v| v.as_u64()) {
            size_budgeted += content_size;
        } else {
            size_budgeted += obj.title.len() as u64;
            if let Some(ref summary) = obj.summary {
                size_budgeted += summary.len() as u64;
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
    println!("| Total Context Size | {:<11} bytes | {:<10} bytes |", size_unbudgeted, size_budgeted);
    println!("| Resolution Time    | {:<13?} | {:<15?} |", duration_unbudgeted, duration_budgeted);
    println!("-----------------------------------------------------");

    // Print types breakdown in budgeted run to show diversity
    let mut types = std::collections::HashMap::new();
    for obj in &bundle_budgeted.objects {
        *types.entry(obj.context_type.short_name()).or_insert(0) += 1;
    }
    println!("\nBudgeted Context Diversity:");
    for (t, count) in types {
        println!(" - {}: {} objects", t, count);
    }

    if size_unbudgeted > 0 {
        let savings = (1.0 - (size_budgeted as f64 / size_unbudgeted as f64)) * 100.0;
        println!("\n>>> Combined context volume reduced by {:.2}% at source!", savings);
        println!(">>> Prompt cost for LLM is {:.2}% cheaper!", savings);
    }

    Ok(())
}
