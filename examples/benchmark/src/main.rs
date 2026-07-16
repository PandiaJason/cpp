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
    // BENCHMARK 1: Filesystem Provider (Code Files)
    // ==========================================
    println!("--- Filesystem Context Benchmark ---");
    let query_fs_unbudgeted = client.query(Goal::code())
        .scope_current()
        .include(ContextType::file())
        .max_results(50)
        .build();

    let start_fs_unbudgeted = Instant::now();
    let bundle_fs_unbudgeted = resolver.resolve_query(&query_fs_unbudgeted).await?;
    let duration_fs_unbudgeted = start_fs_unbudgeted.elapsed();

    let mut size_fs_unbudgeted = 0;
    for obj in &bundle_fs_unbudgeted.objects {
        if let Some(content) = obj.get_metadata("sizeBytes") {
            if let Some(bytes) = content.as_u64() {
                size_fs_unbudgeted += bytes;
            }
        }
    }

    let budget_fs = ContextBudget {
        max_bytes: Some(2000),
        max_objects: Some(5),
        max_latency_ms: None,
        prefer: BudgetPreference::Quality,
    };

    let query_fs_budgeted = client.query(Goal::code())
        .scope_current()
        .include(ContextType::file())
        .max_results(50)
        .budget(budget_fs)
        .build();

    let start_fs_budgeted = Instant::now();
    let bundle_fs_budgeted = resolver.resolve_query(&query_fs_budgeted).await?;
    let duration_fs_budgeted = start_fs_budgeted.elapsed();

    let mut size_fs_budgeted = 0;
    for obj in &bundle_fs_budgeted.objects {
        if let Some(content) = obj.get_metadata("sizeBytes") {
            if let Some(bytes) = content.as_u64() {
                size_fs_budgeted += bytes;
            }
        }
    }

    println!("-----------------------------------------------------");
    println!("| Metric             | Unbudgeted       | Budgeted        |");
    println!("-----------------------------------------------------");
    println!("| File Count         | {:<16} | {:<15} |", bundle_fs_unbudgeted.objects.len(), bundle_fs_budgeted.objects.len());
    println!("| Total Content Size | {:<11} bytes | {:<10} bytes |", size_fs_unbudgeted, size_fs_budgeted);
    println!("| Resolution Time    | {:<13?} | {:<15?} |", duration_fs_unbudgeted, duration_fs_budgeted);
    println!("-----------------------------------------------------");

    // ==========================================
    // BENCHMARK 2: Git Provider (Commits & History)
    // ==========================================
    println!("\n--- Git History Context Benchmark ---");
    let query_git_unbudgeted = client.query(Goal::project())
        .scope_current()
        .include(ContextType::commit())
        .max_results(50)
        .build();

    let start_git_unbudgeted = Instant::now();
    let bundle_git_unbudgeted = resolver.resolve_query(&query_git_unbudgeted).await?;
    let duration_git_unbudgeted = start_git_unbudgeted.elapsed();

    // Calculate raw size of commit descriptions/summaries
    let mut size_git_unbudgeted = 0;
    for obj in &bundle_git_unbudgeted.objects {
        size_git_unbudgeted += obj.title.len() as u64;
        if let Some(ref summary) = obj.summary {
            size_git_unbudgeted += summary.len() as u64;
        }
    }

    let budget_git = ContextBudget {
        max_bytes: Some(300), // Very strict budget for commit summaries
        max_objects: Some(3),
        max_latency_ms: None,
        prefer: BudgetPreference::Quality,
    };

    let query_git_budgeted = client.query(Goal::project())
        .scope_current()
        .include(ContextType::commit())
        .max_results(50)
        .budget(budget_git)
        .build();

    let start_git_budgeted = Instant::now();
    let bundle_git_budgeted = resolver.resolve_query(&query_git_budgeted).await?;
    let duration_git_budgeted = start_git_budgeted.elapsed();

    let mut size_git_budgeted = 0;
    for obj in &bundle_git_budgeted.objects {
        size_git_budgeted += obj.title.len() as u64;
        if let Some(ref summary) = obj.summary {
            size_git_budgeted += summary.len() as u64;
        }
    }

    println!("-----------------------------------------------------");
    println!("| Metric             | Unbudgeted       | Budgeted        |");
    println!("-----------------------------------------------------");
    println!("| Commit Count       | {:<16} | {:<15} |", bundle_git_unbudgeted.objects.len(), bundle_git_budgeted.objects.len());
    println!("| Metadata Text Size | {:<11} bytes | {:<10} bytes |", size_git_unbudgeted, size_git_budgeted);
    println!("| Resolution Time    | {:<13?} | {:<15?} |", duration_git_unbudgeted, duration_git_budgeted);
    println!("-----------------------------------------------------");

    // ==========================================
    // SUMMARY ANALYSIS
    // ==========================================
    let total_unbudgeted = size_fs_unbudgeted + size_git_unbudgeted;
    let total_budgeted = size_fs_budgeted + size_git_budgeted;

    if total_unbudgeted > 0 {
        let savings = (1.0 - (total_budgeted as f64 / total_unbudgeted as f64)) * 100.0;
        println!("\n>>> Combined context volume reduced by {:.2}% at source!", savings);
        println!(">>> Prompt cost for LLM is {:.2}% cheaper!", savings);
    }

    Ok(())
}
