//! Transport abstraction layer for the wire protocol.

use async_trait::async_trait;
use thiserror::Error;

/// Errors returned by transport implementations.
#[derive(Error, Debug)]
pub enum TransportError {
    /// The transport connection was closed.
    #[error("connection closed")]
    ConnectionClosed,

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A serialization or parsing error occurred.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// A transport-specific error.
    #[error("{0}")]
    Other(String),
}

/// A transport-agnostic interface for sending and receiving raw messages.
///
/// Implementations (e.g., stdio, HTTP+SSE, WebSockets) must implement
/// this trait to interface with CPP clients, runtimes, and providers.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Sends a raw message.
    async fn send(&self, message: &[u8]) -> Result<(), TransportError>;

    /// Receives a raw message. Blocks asynchronously until a message is available.
    async fn receive(&self) -> Result<Vec<u8>, TransportError>;

    /// Gracefully closes the transport connection.
    async fn close(&self) -> Result<(), TransportError>;
}
