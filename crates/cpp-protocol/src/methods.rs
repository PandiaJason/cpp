//! Typed parameters and results for all CPP protocol methods.

use serde::{Deserialize, Serialize};

use cpp_core::{
    AccessLevel, ContextBundle, ContextEvent, ContextObject, ContextQuery, ContextUri, Goal,
    ProviderManifest, ProtocolVersion, SubscriptionFilter, SubscriptionId,
};

// ═══════════════════════════════════════════════════════════════════════════
//  METHOD NAMES
// ═══════════════════════════════════════════════════════════════════════════

pub const INITIALIZE: &str = "cpp/initialize";
pub const INITIALIZED: &str = "cpp/initialized";
pub const QUERY: &str = "cpp/query";
pub const RESOLVE: &str = "cpp/resolve";
pub const CAPABILITIES: &str = "cpp/capabilities";
pub const SUBSCRIBE: &str = "cpp/subscribe";
pub const UNSUBSCRIBE: &str = "cpp/unsubscribe";
pub const EVENT: &str = "cpp/event";
pub const PUBLISH: &str = "cpp/publish";
pub const SHUTDOWN: &str = "cpp/shutdown";
pub const EXIT: &str = "cpp/exit";

// ═══════════════════════════════════════════════════════════════════════════
//  CPP/INITIALIZE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    #[serde(default)]
    pub supports_streaming: bool,
    #[serde(default)]
    pub supports_subscriptions: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: ProtocolVersion,
    pub client_info: ClientInfo,
    pub capabilities: ClientCapabilities,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCapabilities {
    #[serde(default)]
    pub supports_streaming: bool,
    #[serde(default)]
    pub supports_subscriptions: bool,
    #[serde(default)]
    pub supports_publish: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: ProtocolVersion,
    pub runtime_info: RuntimeInfo,
    pub capabilities: RuntimeCapabilities,
    pub providers: Vec<ProviderManifest>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  CPP/QUERY
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub query: ContextQuery,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub bundle: ContextBundle,
}

// ═══════════════════════════════════════════════════════════════════════════
//  CPP/RESOLVE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveParams {
    pub uri: ContextUri,
    #[serde(default)]
    pub depth: u32,
    #[serde(default)]
    pub access_level: AccessLevel,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveResult {
    pub object: ContextObject,
}

// ═══════════════════════════════════════════════════════════════════════════
//  CPP/CAPABILITIES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<Goal>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesResult {
    pub providers: Vec<ProviderManifest>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  CPP/SUBSCRIBE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeParams {
    pub filter: SubscriptionFilter,
    #[serde(default)]
    pub access_level: AccessLevel,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeResult {
    pub subscription_id: SubscriptionId,
}

// ═══════════════════════════════════════════════════════════════════════════
//  CPP/UNSUBSCRIBE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsubscribeParams {
    pub subscription_id: SubscriptionId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsubscribeResult {
    pub success: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
//  CPP/EVENT (Notification)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventParams {
    pub event: ContextEvent,
    pub subscription_id: SubscriptionId,
}

// ═══════════════════════════════════════════════════════════════════════════
//  CPP/PUBLISH
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishParams {
    pub event: ContextEvent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishResult {
    pub accepted: bool,
}
