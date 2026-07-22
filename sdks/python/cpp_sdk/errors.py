"""CPP SDK error hierarchy.

Mirrors the Rust ``CppError`` variants so that Python consumers can
catch granular exception types while still having a single base class
for catch-all handling.
"""

from __future__ import annotations


class CppError(Exception):
    """Base exception for all CPP SDK errors."""

    def __init__(self, message: str, *, code: int | None = None) -> None:
        self.code = code
        super().__init__(message)


class CppConnectionError(CppError):
    """Raised when the SDK cannot reach the CPP server."""


class CppTimeoutError(CppError):
    """Raised when a request exceeds ``max_latency_ms`` or the HTTP timeout."""


class CppProtocolError(CppError):
    """Raised when the server returns a malformed or unexpected JSON-RPC response."""

    def __init__(
        self,
        message: str,
        *,
        code: int | None = None,
        data: object | None = None,
    ) -> None:
        self.data = data
        super().__init__(message, code=code)


class CppAuthenticationError(CppError):
    """Raised when the server rejects the client's credentials."""


class CppProviderError(CppError):
    """Raised inside a provider implementation when query or resolve fails."""
