//! Filesystem Context Provider for CPP.
//!
//! Provides context about files and directories on the local filesystem.
//! This is one of the three reference providers (filesystem, git, datetime)
//! used to prove protocol interoperability.
//!
//! # Context Types Provided
//!
//! - `application/cpp.document.file` — Individual files with content
//! - `application/cpp.collection.folder` — Directories with children
//!
//! # Goals Supported
//!
//! - `goal.code` — Source code files in the workspace
//! - `goal.document` — Document files (markdown, text, etc.)
//!
//! # Example
//!
//! ```text
//! Agent: "I need code context"
//! Provider: Returns SCOs for recently modified source files
//!           with content, language metadata, and freshness info
//! ```

mod provider;

pub use provider::FilesystemProvider;
