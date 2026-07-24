use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Json, WebSocketUpgrade, Query,
    },
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use tower_http::cors::CorsLayer;
use chrono::Utc;
use futures::{SinkExt, StreamExt};

use cpp_core::{
    context::ContextObjectBuilder,
    permission::AccessLevel,
    types::{Certainty, Freshness, Importance, LifecycleState, ContextType, ContextUri, ProviderId, SubscriptionId},
    stream::{ContextEvent, ContextEventKind, SubscriptionFilter},
    ContextQuery,
};
use cpp_protocol::messages::{
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
};
use cpp_protocol::methods::{
    PublishParams, PublishResult, SubscribeParams, SubscribeResult, UnsubscribeParams, UnsubscribeResult,
};
use cpp_runtime::{ContextCache, ContextResolver, ProviderRegistry};
use cpp_provider_filesystem::FilesystemProvider;
use cpp_provider_git::GitProvider;
use cpp_provider_datetime::DatetimeProvider;

// Embed the HTML dashboard page
const DASHBOARD_HTML: &str = include_str!("dashboard.html");

#[derive(Clone, Debug)]
struct ClientSubscription {
    id: String,
    filter: SubscriptionFilter,
}

struct ConnectedClient {
    sender: mpsc::UnboundedSender<Message>,
    subscriptions: Vec<ClientSubscription>,
}

struct AppState {
    clients: Mutex<Vec<ConnectedClient>>,
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

// Check authorization header if CPP_AUTH_TOKEN is configured in environment
fn check_auth(headers: &HeaderMap) -> Result<(), (StatusCode, &'static str)> {
    if let Ok(expected_token) = std::env::var("CPP_AUTH_TOKEN") {
        if expected_token.is_empty() {
            return Ok(());
        }
        let auth_header = headers.get("Authorization")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        
        let expected_bearer = format!("Bearer {}", expected_token);
        if auth_header != expected_bearer {
            return Err((StatusCode::UNAUTHORIZED, "Unauthorized: Invalid or missing Bearer token"));
        }
    }
    Ok(())
}

// Params structure for cpp/query
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryParamsWrapper {
    query: ContextQuery,
}

// Handle incoming JSON-RPC 2.0 requests
async fn handle_rpc(
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    if let Err((status, msg)) = check_auth(&headers) {
        let err = JsonRpcError::new(-32001, msg, None);
        return (status, Json(JsonRpcResponse::error(payload.id, err))).into_response();
    }

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
        "cpp/publish" => {
            let params_val = match payload.params {
                Some(v) => v,
                None => {
                    let err = JsonRpcError::new(-32602, "Missing params", None);
                    return Json(JsonRpcResponse::error(payload.id, err)).into_response();
                }
            };
            let publish_params: PublishParams = match serde_json::from_value(params_val) {
                Ok(p) => p,
                Err(e) => {
                    let err = JsonRpcError::new(-32602, format!("Invalid publish params: {}", e), None);
                    return Json(JsonRpcResponse::error(payload.id, err)).into_response();
                }
            };

            // Broadcast published event to connected clients
            broadcast_event(&state, &publish_params.event).await;

            let result = PublishResult { accepted: true };
            return Json(JsonRpcResponse::success(
                payload.id,
                serde_json::to_value(result).unwrap(),
            )).into_response();
        }
        "cpp/query" => {}
        _ => {
            let err = JsonRpcError::new(-32601, "Method not found", None);
            return Json(JsonRpcResponse::error(payload.id, err)).into_response();
        }
    }

    // 2. Validate params are present for cpp/query
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
            // Apply AccessLevel field stripping (RFC-0001 Section 5)
            if query.access_level == AccessLevel::MetadataOnly {
                for obj in &mut bundle.objects {
                    obj.content = None;
                    obj.summary = None;
                }
            }

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
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    if let Ok(expected_token) = std::env::var("CPP_AUTH_TOKEN") {
        if !expected_token.is_empty() {
            let token_hdr = headers.get("Authorization").and_then(|h| h.to_str().ok()).unwrap_or("");
            let token_param = params.get("token").cloned().unwrap_or_default();
            let bearer = format!("Bearer {}", expected_token);

            if token_hdr != bearer && token_param != expected_token {
                return (StatusCode::UNAUTHORIZED, "Unauthorized: Invalid or missing token").into_response();
            }
        }
    }

    ws.on_upgrade(move |socket| ws_session(socket, state))
}

// WebSocket session handler with subscription filtering support
async fn ws_session(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Create channel for transmitting messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let client_id = uuid::Uuid::new_v4().to_string();

    // Register client in list of active listeners
    {
        let mut clients = state.clients.lock().await;
        clients.push(ConnectedClient {
            sender: tx,
            subscriptions: Vec::new(),
        });
    }

    // Task to send outgoing messages to WebSocket connection
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Receive incoming WebSocket RPC calls (cpp/subscribe, cpp/unsubscribe)
    while let Some(Ok(Message::Text(text))) = ws_receiver.next().await {
        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&text) {
            match req.method.as_str() {
                "cpp/subscribe" => {
                    if let Some(params_val) = req.params {
                        if let Ok(sub_params) = serde_json::from_value::<SubscribeParams>(params_val) {
                            let sub_id = format!("sub_{}", uuid::Uuid::new_v4().simple());
                            let sub = ClientSubscription {
                                id: sub_id.clone(),
                                filter: sub_params.filter,
                            };

                            let mut clients = state.clients.lock().await;
                            if let Some(client) = clients.iter_mut().last() {
                                client.subscriptions.push(sub);
                            }

                            let res = SubscribeResult { subscription_id: SubscriptionId::from_string(sub_id) };
                            let resp = JsonRpcResponse::success(req.id, serde_json::to_value(res).unwrap());
                            let _ = resp;
                            let _ = text;
                        }
                    }
                }
                "cpp/unsubscribe" => {
                    if let Some(params_val) = req.params {
                        if let Ok(unsub_params) = serde_json::from_value::<UnsubscribeParams>(params_val) {
                            let mut clients = state.clients.lock().await;
                            let target_sub_id = unsub_params.subscription_id.as_str();
                            for client in clients.iter_mut() {
                                client.subscriptions.retain(|s| s.id != target_sub_id);
                            }
                            let res = UnsubscribeResult { success: true };
                            let _resp = JsonRpcResponse::success(req.id, serde_json::to_value(res).unwrap());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    send_task.abort();
    let _ = client_id;
}

// Broadcast event to connected clients checking selective subscription filters
async fn broadcast_event(state: &Arc<AppState>, event: &ContextEvent) {
    let notification = JsonRpcNotification::new(
        "cpp/event",
        Some(serde_json::to_value(event).unwrap()),
    );
    let serialized = serde_json::to_string(&notification).unwrap();

    let mut clients = state.clients.lock().await;
    clients.retain(|client| {
        let should_send = if client.subscriptions.is_empty() {
            true // Default: send all if client has no specific subscription filters
        } else {
            client.subscriptions.iter().any(|sub| matches_filter(&sub.filter, event))
        };

        if should_send {
            client.sender.send(Message::Text(serialized.clone().into())).is_ok()
        } else {
            true
        }
    });
}

// Matches an event against a subscription filter
fn matches_filter(filter: &SubscriptionFilter, event: &ContextEvent) -> bool {
    let provider_str = event.provider_id.as_str();
    if !filter.providers.is_empty() && !filter.providers.iter().any(|p| p.as_str() == provider_str) {
        return false;
    }
    if !filter.uri_patterns.is_empty() {
        let uri_str = event.context_uri.as_str();
        let matches = filter.uri_patterns.iter().any(|p| {
            if p.ends_with('*') {
                uri_str.starts_with(p.trim_end_matches('*'))
            } else {
                uri_str == *p
            }
        });
        if !matches {
            return false;
        }
    }
    true
}

// Background event generator that pushes simulated updates to the event bus
async fn start_background_events(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(4));
    let mut step = 0;

    loop {
        interval.tick().await;

        let event = match step % 4 {
            0 => {
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
                .summary("main.rs updated: added selective subscription filter & auth")
                .build();

                ContextEvent::new(
                    ContextEventKind::Updated,
                    ContextUri::new("filesystem", "file", "crates/cpp-server/src/main.rs"),
                    ProviderId::new("filesystem"),
                ).with_snapshot(sco)
            }
            1 => {
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
                .content("warning: unused import in main.rs:3")
                .summary("Compiler warning in crates/cpp-server/src/main.rs")
                .build();

                ContextEvent::new(
                    ContextEventKind::Created,
                    ContextUri::new("compiler", "diagnostic", "warning_unused_import"),
                    ProviderId::new("compiler"),
                ).with_snapshot(sco)
            }
            2 => {
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
                .summary("Git branch 'main' updated")
                .build();

                ContextEvent::new(
                    ContextEventKind::Updated,
                    ContextUri::new("git", "branch", "main"),
                    ProviderId::new("git"),
                ).with_snapshot(sco)
            }
            _ => {
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
        broadcast_event(&state, &event).await;
    }
}
