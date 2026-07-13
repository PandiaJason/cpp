//! Datetime context provider implementation.

use async_trait::async_trait;
use chrono::{Local, Utc};
use cpp_core::context::{ContextBundle, ContextObject, ContextObjectBuilder, ContextPermissions};
use cpp_core::error::CppError;
use cpp_core::manifest::{ProviderCapabilities, ProviderManifest};
use cpp_core::query::ContextQuery;
use cpp_core::types::*;
use cpp_sdk::ContextProvider;

/// A context provider that serves local and UTC time, timezone, and temporal details.
pub struct DatetimeProvider {
    manifest: ProviderManifest,
}

impl DatetimeProvider {
    /// Creates a new DatetimeProvider.
    pub fn new() -> Self {
        let manifest = ProviderManifest::new(
            ProviderId::new("datetime"),
            "Datetime Context Provider",
            ProviderCapabilities::basic(
                vec![ContextType::temporal()],
                vec![Goal::calendar()],
            ),
        )
        .with_version("0.1.0")
        .with_description("Provides context about the current date, time, and active timezone");

        Self { manifest }
    }

    fn get_time_context(&self) -> ContextObject {
        let utc_now = Utc::now();
        let local_now = Local::now();
        let tz = iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".to_string());

        ContextObjectBuilder::new(
            ContextUri::new("datetime", "temporal", "current"),
            ContextType::temporal(),
            ProviderId::new("datetime"),
        )
        .title("Current Temporal Context")
        .summary(format!(
            "UTC: {}, Local: {}, Timezone: {}",
            utc_now.format("%Y-%m-%d %H:%M:%S"),
            local_now.format("%Y-%m-%d %H:%M:%S"),
            tz
        ))
        .certainty(Certainty::Authoritative)
        .freshness(Freshness::live())
        .importance(Importance::normal())
        .permissions(ContextPermissions::read())
        .metadata("utcTime", serde_json::json!(utc_now.to_rfc3339()))
        .metadata("localTime", serde_json::json!(local_now.to_rfc3339()))
        .metadata("timezone", serde_json::json!(tz))
        .metadata("dayOfWeek", serde_json::json!(utc_now.format("%A").to_string()))
        .build()
    }
}

#[async_trait]
impl ContextProvider for DatetimeProvider {
    fn manifest(&self) -> &ProviderManifest {
        &self.manifest
    }

    async fn query(&self, _query: &ContextQuery) -> Result<ContextBundle, CppError> {
        let objects = vec![self.get_time_context()];

        let providers = vec![ProviderId::new("datetime")];
        Ok(ContextBundle {
            total_count: objects.len() as u32,
            providers,
            resolution_time_ms: 0,
            from_cache: false,
            metadata: Default::default(),
            objects,
        })
    }

    async fn resolve(&self, uri: &ContextUri) -> Result<ContextObject, CppError> {
        let path = uri.path().unwrap_or("current");
        if path == "current" || path == "now" {
            Ok(self.get_time_context())
        } else {
            Err(CppError::context_not_found(uri.as_str()))
        }
    }
}

impl Default for DatetimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn datetime_manifest() {
        let provider = DatetimeProvider::new();
        assert_eq!(provider.manifest().id, ProviderId::new("datetime"));
        assert!(provider.manifest().supports_type(&ContextType::temporal()));
    }
}
