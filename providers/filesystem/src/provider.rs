//! Filesystem context provider implementation.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use chrono::Utc;
use cpp_core::context::{ContextBundle, ContextObject, ContextObjectBuilder, ContextPermissions, Reference};
use cpp_core::error::CppError;
use cpp_core::manifest::{ProviderCapabilities, ProviderManifest};
use cpp_core::query::ContextQuery;
use cpp_core::types::*;
use cpp_sdk::ContextProvider;

/// A context provider that serves file and directory context from
/// the local filesystem.
pub struct FilesystemProvider {
    /// Root directory to serve context from.
    root: PathBuf,
    /// Provider manifest.
    manifest: ProviderManifest,
}

impl FilesystemProvider {
    /// Creates a new filesystem provider rooted at the given path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let manifest = ProviderManifest::new(
            ProviderId::new("filesystem"),
            "Filesystem Context Provider",
            ProviderCapabilities::basic(
                vec![ContextType::file(), ContextType::folder()],
                vec![Goal::code(), Goal::document()],
            ),
        )
        .with_version("0.1.0")
        .with_description("Provides context about local files and directories");

        Self { root, manifest }
    }

    /// Returns the root directory this provider serves.
    pub fn root(&self) -> &Path {
        &self.root
    }

    // ── Private helpers ──

    fn scan_directory(
        &self,
        dir: &Path,
        extensions: &[&str],
        objects: &mut Vec<ContextObject>,
        max: usize,
        depth: usize,
        max_depth: usize,
    ) -> Result<(), CppError> {
        if objects.len() >= max || depth > max_depth {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir).map_err(|e| {
            CppError::new(
                cpp_core::error::ErrorCode::ContextError,
                format!("Failed to read directory {}: {}", dir.display(), e),
            )
        })?;

        for entry in entries.flatten() {
            if objects.len() >= max {
                break;
            }

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files and common ignored directories
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        if let Ok(sco) = self.file_to_sco(&path) {
                            objects.push(sco);
                        }
                    }
                }
            } else if path.is_dir() {
                self.scan_directory(&path, extensions, objects, max, depth + 1, max_depth)?;
            }
        }

        Ok(())
    }

    fn file_to_sco(&self, path: &Path) -> Result<ContextObject, CppError> {
        let relative = path.strip_prefix(&self.root).unwrap_or(path);
        let relative_str = relative.to_string_lossy();

        let metadata = std::fs::metadata(path).map_err(|e| {
            CppError::new(
                cpp_core::error::ErrorCode::ContextError,
                format!("Failed to read metadata for {}: {}", path.display(), e),
            )
        })?;

        let modified: chrono::DateTime<Utc> = metadata
            .modified()
            .map(|t| t.into())
            .unwrap_or_else(|_| Utc::now());

        // Read file content (limit to 128KB for context budget)
        let content = std::fs::read_to_string(path).ok().and_then(|s| {
            if s.len() > 128_000 {
                Some(s[..128_000].to_string())
            } else {
                Some(s)
            }
        });

        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let extension = path.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        let language = match extension.as_str() {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "go" => "go",
            "java" => "java",
            "c" | "h" => "c",
            "cpp" | "hpp" => "cpp",
            "md" => "markdown",
            "toml" => "toml",
            "yaml" | "yml" => "yaml",
            "json" => "json",
            _ => "text",
        };

        let mut builder = ContextObjectBuilder::new(
            ContextUri::new("filesystem", "file", &*relative_str),
            ContextType::file(),
            ProviderId::new("filesystem"),
        )
        .title(&file_name)
        .certainty(Certainty::Authoritative)
        .freshness(Freshness::live())
        .importance(Importance::normal())
        .lifecycle(LifecycleState::Updated)
        .created_at(modified)
        .updated_at(modified)
        .permissions(ContextPermissions::read())
        .reference(Reference::source(format!("file://{}", path.display())))
        .metadata("language", serde_json::json!(language))
        .metadata("extension", serde_json::json!(extension))
        .metadata("sizeBytes", serde_json::json!(metadata.len()));

        if let Some(ref c) = content {
            let line_count = c.lines().count();
            builder = builder
                .content(c.clone())
                .summary(format!("{} ({} lines, {} bytes)", file_name, line_count, metadata.len()))
                .metadata("lineCount", serde_json::json!(line_count));
        }

        Ok(builder.build())
    }

    fn dir_to_sco(&self, path: &Path) -> Result<ContextObject, CppError> {
        let relative = path.strip_prefix(&self.root).unwrap_or(path);
        let relative_str = relative.to_string_lossy();
        let dir_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| relative_str.to_string());

        let child_count = std::fs::read_dir(path)
            .map(|entries| entries.count())
            .unwrap_or(0);

        Ok(ContextObjectBuilder::new(
            ContextUri::new("filesystem", "folder", &*relative_str),
            ContextType::folder(),
            ProviderId::new("filesystem"),
        )
        .title(&dir_name)
        .summary(format!("Directory with {} entries", child_count))
        .certainty(Certainty::Authoritative)
        .freshness(Freshness::live())
        .importance(Importance::normal())
        .permissions(ContextPermissions::read())
        .metadata("childCount", serde_json::json!(child_count))
        .build())
    }
}

#[async_trait]
impl ContextProvider for FilesystemProvider {
    fn manifest(&self) -> &ProviderManifest {
        &self.manifest
    }

    async fn query(&self, query: &ContextQuery) -> Result<ContextBundle, CppError> {
        let mut objects = Vec::new();
        let max = query.max_results as usize;

        let extensions = if query.goal == Goal::code() {
            vec!["rs", "py", "js", "ts", "go", "java", "c", "cpp", "h", "toml", "yaml", "json"]
        } else {
            vec!["md", "txt", "rst", "adoc", "html"]
        };

        self.scan_directory(&self.root, &extensions, &mut objects, max, 0, 3)?;
        objects.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        objects.truncate(max);

        let providers = vec![ProviderId::new("filesystem")];
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
        let path_str = uri.path().ok_or_else(|| {
            CppError::context_not_found(uri.as_str())
        })?;

        let path = self.root.join(path_str);
        if !path.exists() {
            return Err(CppError::context_not_found(uri.as_str()));
        }

        if path.is_file() {
            self.file_to_sco(&path)
        } else {
            self.dir_to_sco(&path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn provider_creation() {
        let provider = FilesystemProvider::new("/tmp");
        assert_eq!(provider.manifest().id, ProviderId::new("filesystem"));
        assert!(provider.manifest().supports_goal(&Goal::code()));
        assert!(provider.manifest().supports_type(&ContextType::file()));
    }

    #[tokio::test]
    async fn query_code_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("lib.rs"), "pub mod foo;").unwrap();
        fs::write(dir.path().join("readme.md"), "# Hello").unwrap();
        fs::write(dir.path().join("notes.txt"), "some notes").unwrap();

        let provider = FilesystemProvider::new(dir.path());

        let query = cpp_core::query::ContextQueryBuilder::new(Goal::code())
            .max_results(10)
            .build();

        let bundle = provider.query(&query).await.unwrap();

        assert!(bundle.len() >= 2);
        assert!(bundle.objects.iter().all(|o| o.context_type == ContextType::file()));
        assert!(bundle.objects.iter().all(|o| o.certainty == Certainty::Authoritative));
    }

    #[tokio::test]
    async fn resolve_file() {
        let dir = tempfile::tempdir().unwrap();
        let content = "fn main() { println!(\"hello\"); }";
        fs::write(dir.path().join("main.rs"), content).unwrap();

        let provider = FilesystemProvider::new(dir.path());
        let uri = ContextUri::new("filesystem", "file", "main.rs");

        let sco = provider.resolve(&uri).await.unwrap();
        assert_eq!(sco.title, "main.rs");
        assert_eq!(sco.content.as_deref(), Some(content));
        assert_eq!(sco.certainty, Certainty::Authoritative);
        assert_eq!(sco.freshness.kind, FreshnessKind::Live);
    }
}
