use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use cpp_core::{
    ContextObjectBuilder,
    Certainty, Freshness, Importance, LifecycleState,
    ContextType, ContextUri, ProviderId,
    ContextEvent, ContextEventKind, Subscription, SubscriptionFilter,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("===========================================================");
    println!("=== CPP Streaming Event Bus & Push Context Demonstration ===");
    println!("===========================================================");

    // 1. Initialize our communication channels (simulating the Protocol Event Bus)
    let (event_tx, mut event_rx) = mpsc::channel::<ContextEvent>(32);

    // 2. Define the Agent's Subscription Filter
    // In the real world, the AI agent wants to subscribe to:
    // - Any updates from the 'filesystem' provider involving '.rs' file changes
    // - Any updates from the 'compiler' provider (such as build status or diagnostics)
    let filter = SubscriptionFilter {
        uri_patterns: vec![
            "cpp://filesystem/file/crates/cpp-sdk/*".to_string(),
            "cpp://compiler/*".to_string(),
        ],
        ..Default::default()
    };
    
    let subscription = Subscription::new(filter);
    println!("\n[Agent] Subscribed to live context stream:");
    println!(" - Subscription ID: {}", subscription.id);
    println!(" - Filter Pattern:  crates/cpp-sdk/* AND compiler/*");

    // 3. Spawn a background thread/task simulating active external system events
    // (e.g. a developer saving a file, and a background watcher compiling it)
    let producer_tx = event_tx.clone();
    tokio::spawn(async move {
        // Event A: Developer modifies client.rs
        sleep(Duration::from_millis(800)).await;
        
        let path = "crates/cpp-sdk/src/client.rs";
        let client_sco = ContextObjectBuilder::new(
            ContextUri::new("filesystem", "file", path),
            ContextType::file(),
            ProviderId::new("filesystem"),
        )
        .title("client.rs")
        .certainty(Certainty::Authoritative)
        .freshness(Freshness::live())
        .importance(Importance::high())
        .lifecycle(LifecycleState::Updated)
        .content("pub struct CppClient { session_id: String }")
        .summary("client.rs updated with new struct definition")
        .build();

        let event1 = ContextEvent::new(
            ContextEventKind::Updated,
            ContextUri::new("filesystem", "file", path),
            ProviderId::new("filesystem"),
        ).with_snapshot(client_sco);

        println!("\n[System Watcher] File saved: {}", path);
        let _ = producer_tx.send(event1).await;

        // Event B: Background compiler runs 'cargo check' and finds a warning
        sleep(Duration::from_millis(1500)).await;

        let warning_sco = ContextObjectBuilder::new(
            ContextUri::new("compiler", "diagnostic", "warning_unused_import"),
            ContextType::new("application/cpp.diagnostic.compiler"),
            ProviderId::new("compiler"),
        )
        .title("Unused import warning")
        .certainty(Certainty::Authoritative)
        .freshness(Freshness::live())
        .importance(Importance::high())
        .lifecycle(LifecycleState::Created)
        .content("warning: unused import: `std::sync::Arc` in client.rs:12")
        .summary("Compiler warning in crates/cpp-sdk/src/client.rs")
        .build();

        let event2 = ContextEvent::new(
            ContextEventKind::Created,
            ContextUri::new("compiler", "diagnostic", "warning_unused_import"),
            ProviderId::new("compiler"),
        ).with_snapshot(warning_sco);

        println!("\n[Compiler Service] Build complete (with warnings)");
        let _ = producer_tx.send(event2).await;

        // Event C: File change in an unrelated path (e.g. documentation files)
        // This should be filtered out by the agent's subscription rules!
        sleep(Duration::from_millis(1000)).await;

        let doc_sco = ContextObjectBuilder::new(
            ContextUri::new("filesystem", "file", "docs/README.md"),
            ContextType::file(),
            ProviderId::new("filesystem"),
        )
        .title("README.md")
        .certainty(Certainty::Authoritative)
        .freshness(Freshness::live())
        .importance(Importance::low())
        .lifecycle(LifecycleState::Updated)
        .summary("Documentation updated")
        .build();

        let event3 = ContextEvent::new(
            ContextEventKind::Updated,
            ContextUri::new("filesystem", "file", "docs/README.md"),
            ProviderId::new("filesystem"),
        ).with_snapshot(doc_sco);

        println!("\n[System Watcher] File saved: docs/README.md");
        let _ = producer_tx.send(event3).await;
    });

    // 4. Main Event loop - the Agent consumes the stream of context events
    println!("\n[Agent] Actively listening for context events on the CPP bus...");
    let mut received_count = 0;

    // We will run the loop for a few seconds to let all events arrive
    let timeout = sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                // Check if the event matches the agent's subscription criteria
                if subscription.matches(&event) {
                    received_count += 1;
                    println!(" -> [Agent INGESTED] Matching Context Event Received!");
                    println!("    | Kind:      {}", event.kind);
                    println!("    | URI:       {}", event.context_uri);
                    println!("    | Provider:  {}", event.provider_id);
                    if let Some(ref snapshot) = event.snapshot {
                        println!("    | Title:     {}", snapshot.title);
                        if let Some(ref content) = snapshot.content {
                            println!("    | Content:   \"{}\"", content);
                        }
                    }
                } else {
                    println!(" -> [Agent IGNORED] Event did not match subscription criteria (filtered out).");
                    println!("    | URI:       {}", event.context_uri);
                }
            }
            _ = &mut timeout => {
                println!("\n[Agent] Stream listener timed out. Shutting down.");
                break;
            }
        }
    }

    println!("\n=== Demo Completed ===");
    println!("Agent received & processed {} live context events.", received_count);
    assert_eq!(received_count, 2, "Agent should have received exactly 2 matching events (client.rs update & compiler warning), and filtered out the third.");
    Ok(())
}
