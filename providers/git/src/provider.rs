//! Git context provider implementation.

use std::path::PathBuf;
use std::process::Command;

use async_trait::async_trait;
use chrono::Utc;
use cpp_core::context::{ContextBundle, ContextObject, ContextObjectBuilder, Reference};
use cpp_core::error::CppError;
use cpp_core::manifest::{ProviderCapabilities, ProviderManifest};
use cpp_core::query::ContextQuery;
use cpp_core::types::*;
use cpp_sdk::ContextProvider;

/// A context provider that serves git repository context (branches, commits).
pub struct GitProvider {
    repo_path: PathBuf,
    manifest: ProviderManifest,
}

impl GitProvider {
    /// Creates a new GitProvider for the given repository path.
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        let repo_path = repo_path.into();
        let manifest = ProviderManifest::new(
            ProviderId::new("git"),
            "Git Context Provider",
            ProviderCapabilities::basic(
                vec![
                    ContextType::repository(),
                    ContextType::commit(),
                    ContextType::branch(),
                ],
                vec![Goal::code(), Goal::project()],
            ),
        )
        .with_version("0.1.0")
        .with_description("Provides context about local git branches, commits, and changes");

        Self { repo_path, manifest }
    }

    // ── Helper methods to run git CLI or fallback to mocks ──

    fn run_git(&self, args: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(args)
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    fn get_repo_name(&self) -> String {
        if let Ok(url) = self.run_git(&["config", "--get", "remote.origin.url"]) {
            if let Some(name) = url.rsplit('/').next() {
                return name.strip_suffix(".git").unwrap_or(name).to_string();
            }
        }
        self.repo_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown-repo".to_string())
    }

    fn get_repo_context(&self) -> Result<ContextObject, CppError> {
        let repo_name = self.get_repo_name();
        let branch = self.run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_else(|_| "main".to_string());

        let mut builder = ContextObjectBuilder::new(
            ContextUri::new("git", "repository", &repo_name),
            ContextType::repository(),
            ProviderId::new("git"),
        )
        .title(&repo_name)
        .certainty(Certainty::Authoritative)
        .freshness(Freshness::live())
        .importance(Importance::high())
        .metadata("currentBranch", serde_json::json!(branch));

        if let Ok(url) = self.run_git(&["config", "--get", "remote.origin.url"]) {
            builder = builder
                .reference(Reference::source(&url))
                .metadata("remoteUrl", serde_json::json!(url));
        }

        Ok(builder.build())
    }

    fn get_branch_context(&self) -> Result<ContextObject, CppError> {
        let branch = self.run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_else(|_| "main".to_string());

        let sha = self.run_git(&["rev-parse", "HEAD"])
            .unwrap_or_else(|_| "0000000000000000000000000000000000000000".to_string());

        Ok(ContextObjectBuilder::new(
            ContextUri::new("git", "branch", &branch),
            ContextType::branch(),
            ProviderId::new("git"),
        )
        .title(&branch)
        .summary(format!("Git branch '{}' at commit {}", branch, &sha[..8]))
        .certainty(Certainty::Authoritative)
        .freshness(Freshness::live())
        .importance(Importance::high())
        .metadata("commitSha", serde_json::json!(sha))
        .build())
    }

    fn get_commit_context(&self, sha: &str) -> Result<ContextObject, CppError> {
        let commit_sha = if sha.eq_ignore_ascii_case("head") {
            self.run_git(&["rev-parse", "HEAD"])
                .unwrap_or_else(|_| "0000000000000000000000000000000000000000".to_string())
        } else {
            sha.to_string()
        };

        let author = self.run_git(&["show", "-s", "--format=%an <%ae>", &commit_sha])
            .unwrap_or_else(|_| "Author <author@example.com>".to_string());

        let date_str = self.run_git(&["show", "-s", "--format=%aI", &commit_sha])
            .unwrap_or_else(|_| Utc::now().to_rfc3339());

        let date = chrono::DateTime::parse_from_rfc3339(&date_str)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let message = self.run_git(&["show", "-s", "--format=%B", &commit_sha])
            .unwrap_or_else(|_| "Commit message placeholder".to_string());

        Ok(ContextObjectBuilder::new(
            ContextUri::new("git", "commit", &commit_sha),
            ContextType::commit(),
            ProviderId::new("git"),
        )
        .title(message.lines().next().unwrap_or("No message"))
        .summary(&message)
        .certainty(Certainty::Authoritative)
        .freshness(Freshness::immutable())
        .importance(Importance::normal())
        .created_at(date)
        .updated_at(date)
        .metadata("author", serde_json::json!(author))
        .metadata("sha", serde_json::json!(commit_sha))
        .build())
    }

    fn get_recent_commits(&self, limit: usize) -> Result<Vec<ContextObject>, CppError> {
        let log = self.run_git(&["log", &format!("-n{}", limit), "--format=%H"]);
        let shas = match log {
            Ok(stdout) => stdout.lines().map(|s| s.to_string()).collect::<Vec<_>>(),
            Err(_) => vec!["0000000000000000000000000000000000000000".to_string()],
        };

        let mut commits = Vec::new();
        for sha in shas {
            if let Ok(commit) = self.get_commit_context(&sha) {
                commits.push(commit);
            }
        }
        Ok(commits)
    }
}

#[async_trait]
impl ContextProvider for GitProvider {
    fn manifest(&self) -> &ProviderManifest {
        &self.manifest
    }

    async fn query(&self, _query: &ContextQuery) -> Result<ContextBundle, CppError> {
        let mut objects = Vec::new();

        if let Ok(repo_sco) = self.get_repo_context() {
            objects.push(repo_sco);
        }

        if let Ok(branch_sco) = self.get_branch_context() {
            objects.push(branch_sco);
        }

        if let Ok(commits) = self.get_recent_commits(5) {
            objects.extend(commits);
        }

        let providers = vec![ProviderId::new("git")];
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
        let context_type = uri.context_type().ok_or_else(|| {
            CppError::context_not_found(uri.as_str())
        })?;

        match context_type {
            "repository" => self.get_repo_context(),
            "branch" => self.get_branch_context(),
            "commit" => {
                let path = uri.path().unwrap_or("head");
                self.get_commit_context(path)
            }
            _ => Err(CppError::context_not_found(uri.as_str())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn git_manifest() {
        let provider = GitProvider::new(".");
        assert_eq!(provider.manifest().id, ProviderId::new("git"));
        assert!(provider.manifest().supports_goal(&Goal::project()));
    }
}
