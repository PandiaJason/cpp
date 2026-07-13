//! Git Context Provider for CPP.
//!
//! Exposes git repository context including commits, branch name, and status.

mod provider;

pub use provider::GitProvider;
