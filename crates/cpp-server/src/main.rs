use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Json, WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use tower_http::cors::CorsLayer;
use chrono::Utc;
use futures::{SinkExt, StreamExt};

use cpp_core::{
    context::ContextObjectBuilder,
    types::{Certainty, Freshness, Importance, LifecycleState, ContextType, ContextUri, ProviderId},
    stream::{ContextEvent, ContextEventKind},
    ContextQuery,
};
use cpp_protocol::messages::{
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
};
use cpp_runtime::{ContextCache, ContextResolver, ProviderRegistry};
use cpp_provider_filesystem::FilesystemProvider;
use cpp_provider_git::GitProvider;
use cpp_provider_datetime::DatetimeProvider;

// Embed the HTML dashboard page
const DASHBOARD_HTML: &str = include_str!("dashboard.html");

struct AppState {
    clients: Mutex<Vec<mpsc::UnboundedSender<Message>>>,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let port = 3030;
    println!("===========================================================");
    println!("=== CPP Semantic Context Daemon starting on port {} ===", port);
    println!("===========================================================");

    // Initialize global application state for tracking WebSocket clients
    let state = Arc::new(AppState {
        clients: Mutex::new(Vec::new()),
    });

    // Start background context event publisher to simulate real-time environment changes
    let event_state = state.clone();
    tokio::spawn(async move {
        start_background_events(event_state).await;
    });

    // Configure the Axum HTTP routing server
    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/api/rpc", post(handle_rpc))
        .route("/api/events", get(handle_websocket))
        .layer(CorsLayer::permissive())
        .layer(Extension(state));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    println!("CPP Server dashboard available at http://localhost:{}", port);
    axum::serve(listener, app).await.unwrap();
}

// Serve the single page UI dashboard
async fn serve_dashboard() -> impl IntoResponse {
    Html(DASHBOARD_HTML)
}

// Params structure for cpp/query
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryParamsWrapper {
    query: ContextQuery,
}

// Handle incoming JSON-RPC 2.0 requests
async fn handle_rpc(
    Json(payload): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    match payload.method.as_str() {
        "cpp/initialize" => {
            let res = serde_json::json!({
                "protocolVersion": "0.1.0",
                "runtimeInfo": {
                    "name": "cpp-server-axum",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "streaming": true,
                    "subscriptions": true,
                    "budgetNegotiation": true
                },
                "providers": [
                    { "id": "filesystem", "name": "Filesystem Context Provider", "contextTypes": ["application/cpp.document.file"], "goals": ["code", "document"] },
                    { "id": "git", "name": "Git Context Provider", "contextTypes": ["application/cpp.repository", "application/cpp.commit", "application/cpp.branch"], "goals": ["code", "project"] },
                    { "id": "datetime", "name": "Datetime Context Provider", "contextTypes": ["application/cpp.datetime"], "goals": ["calendar"] }
                ]
            });
            return Json(JsonRpcResponse::success(payload.id, res)).into_response();
        }
        "cpp/capabilities" => {
            let res = serde_json::json!({
                "providers": [
                    { "id": "filesystem", "name": "Filesystem Context Provider", "contextTypes": ["application/cpp.document.file"], "goals": ["code", "document"] },
                    { "id": "git", "name": "Git Context Provider", "contextTypes": ["application/cpp.repository", "application/cpp.commit", "application/cpp.branch"], "goals": ["code", "project"] },
                    { "id": "datetime", "name": "Datetime Context Provider", "contextTypes": ["application/cpp.datetime"], "goals": ["calendar"] }
                ]
            });
            return Json(JsonRpcResponse::success(payload.id, res)).into_response();
        }
        "cpp/resolve" => {
            let params_val = match payload.params {
                Some(v) => v,
                None => {
                    let err = JsonRpcError::new(-32602, "Missing params", None);
                    return Json(JsonRpcResponse::error(payload.id, err)).into_response();
                }
            };
            let uri_str = params_val.get("uri").and_then(|u| u.as_str()).unwrap_or("");
            let res = serde_json::json!({
                "object": {
                    "uri": uri_str,
                    "id": uuid::Uuid::new_v4().to_string(),
                    "contextType": "application/cpp.document.file",
                    "providerId": "filesystem",
                    "title": uri_str,
                    "certainty": "authoritative",
                    "freshness": { "kind": "live" },
                    "importance": { "priority": 0.8 },
                    "content": format!("Resolved content for {}", uri_str)
                }
            });
            return Json(JsonRpcResponse::success(payload.id, res)).into_response();
        }
        "cpp/query" => {}
        _ => {
            let err = JsonRpcError::new(-32601, "Method not found", None);
            return Json(JsonRpcResponse::error(payload.id, err)).into_response();
        }
    }

    // 2. Validate params are present
    let params_val = match payload.params {
        Some(v) => v,
        None => {
            let err = JsonRpcError::new(-32602, "Missing params", None);
            return Json(JsonRpcResponse::error(payload.id, err)).into_response();
        }
    };

    // 3. Deserialize target query
    let wrapper: QueryParamsWrapper = match serde_json::from_value(params_val) {
        Ok(w) => w,
        Err(e) => {
            let err = JsonRpcError::new(-32602, format!("Invalid query params: {}", e), None);
            return Json(JsonRpcResponse::error(payload.id, err)).into_response();
        }
    };

    let query = wrapper.query;

    // 4. Resolve workspace directory from hints (defaulting to current directory)
    let workspace_path = query.hints.get("workspacePath")
        .and_then(|v| v.as_str())
        .map(|s| std::path::PathBuf::from(s))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let canonical_path = workspace_path.canonicalize().unwrap_or(workspace_path);

    // 5. Initialize the Registry and Providers dynamically for this workspace
    let registry = ProviderRegistry::new();
    
    let fs_provider = Arc::new(FilesystemProvider::new(&canonical_path));
    registry.register(fs_provider);
    
    let git_provider = Arc::new(GitProvider::new(&canonical_path));
    registry.register(git_provider);
    
    let dt_provider = Arc::new(DatetimeProvider::new());
    registry.register(dt_provider);

    let cache = ContextCache::new();
    let resolver = ContextResolver::new(registry, cache);

    // 6. Execute context resolution
    match resolver.resolve_query(&query).await {
        Ok(mut bundle) => {
            // Attach a session ID in metadata so the client has tracking context
            let session_id = format!("ses_{}", uuid::Uuid::new_v4().simple());
            bundle.metadata.insert("sessionId".to_string(), serde_json::json!(session_id));

            Json(JsonRpcResponse::success(
                payload.id,
                serde_json::to_value(bundle).unwrap(),
            )).into_response()
        }
        Err(e) => {
            let err = JsonRpcError::new(-32000, format!("Query resolution failed: {}", e), None);
            Json(JsonRpcResponse::error(payload.id, err)).into_response()
        }
    }
}

// Upgrade HTTP to WebSocket route
async fn handle_websocket(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_session(socket, state))
}

// WebSocket session handler
async fn ws_session(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_sender, _ws_receiver) = socket.split();

    // Create channel for transmitting messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Register client in list of active listeners
    {
        let mut clients = state.clients.lock().await;
        clients.push(tx);
    }

    // Task to receive messages from the channel and send them down the WebSocket connection
    while let Some(msg) = rx.recv().await {
        if ws_sender.send(msg).await.is_err() {
            break; // Connection closed or failed
        }
    }
}

// Background event generator that pushes simulated updates to the event bus
async fn start_background_events(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(4));
    let mut step = 0;

    loop {
        interval.tick().await;

        let event = match step % 4 {
            0 => {
                // Event A: main.rs file saved (filesystem provider)
                let sco = ContextObjectBuilder::new(
                    ContextUri::new("filesystem", "file", "crates/cpp-server/src/main.rs"),
                    ContextType::file(),
                    ProviderId::new("filesystem"),
                )
                .title("main.rs")
                .certainty(Certainty::Authoritative)
                .freshness(Freshness::live())
                .importance(Importance::high())
                .lifecycle(LifecycleState::Updated)
                .content("async fn handle_rpc(...) { ... }")
                .summary("main.rs updated: added dynamic path resolution")
                .build();

                ContextEvent::new(
                    ContextEventKind::Updated,
                    ContextUri::new("filesystem", "file", "crates/cpp-server/src/main.rs"),
                    ProviderId::new("filesystem"),
                ).with_snapshot(sco)
            }
            1 => {
                // Event B: build warning discovered (compiler provider)
                let sco = ContextObjectBuilder::new(
                    ContextUri::new("compiler", "diagnostic", "warning_unused_import"),
                    ContextType::new("application/cpp.diagnostic.compiler"),
                    ProviderId::new("compiler"),
                )
                .title("Unused import warning")
                .certainty(Certainty::Authoritative)
                .freshness(Freshness::live())
                .importance(Importance::normal())
                .lifecycle(LifecycleState::Created)
                .content("warning: unused import: `std::time::Duration` in main.rs:3")
                .summary("Compiler warning in crates/cpp-server/src/main.rs")
                .build();

                ContextEvent::new(
                    ContextEventKind::Created,
                    ContextUri::new("compiler", "diagnostic", "warning_unused_import"),
                    ProviderId::new("compiler"),
                ).with_snapshot(sco)
            }
            2 => {
                // Event C: Git branch main commit pushed (git provider)
                let sco = ContextObjectBuilder::new(
                    ContextUri::new("git", "branch", "main"),
                    ContextType::new("application/cpp.entity.branch"),
                    ProviderId::new("git"),
                )
                .title("main")
                .certainty(Certainty::Authoritative)
                .freshness(Freshness::live())
                .importance(Importance::normal())
                .lifecycle(LifecycleState::Updated)
                .summary("Git branch 'main' updated to commit 5e78d91a")
                .build();

                ContextEvent::new(
                    ContextEventKind::Updated,
                    ContextUri::new("git", "branch", "main"),
                    ProviderId::new("git"),
                ).with_snapshot(sco)
            }
            _ => {
                // Event D: Datetime current temporal context (datetime provider)
                let sco = ContextObjectBuilder::new(
                    ContextUri::new("datetime", "temporal", "current"),
                    ContextType::new("application/cpp.event.temporal"),
                    ProviderId::new("datetime"),
                )
                .title("Current Temporal Context")
                .certainty(Certainty::Authoritative)
                .freshness(Freshness::live())
                .importance(Importance::normal())
                .lifecycle(LifecycleState::Updated)
                .summary(format!("UTC Time: {}", Utc::now().to_rfc3339()))
                .build();

                ContextEvent::new(
                    ContextEventKind::Updated,
                    ContextUri::new("datetime", "temporal", "current"),
                    ProviderId::new("datetime"),
                ).with_snapshot(sco)
            }
        };

        step += 1;

        // Wrap ContextEvent inside JSON-RPC 2.0 Notification: method = "cpp/event"
        let notification = JsonRpcNotification::new(
            "cpp/event",
            Some(serde_json::to_value(&event).unwrap()),
        );

        let serialized = serde_json::to_string(&notification).unwrap();

        // Broadcast notification to all active WebSocket connections
        let mut clients = state.clients.lock().await;
        clients.retain(|client| {
            client.send(Message::Text(serialized.clone().into())).is_ok()
        });
    }
}
