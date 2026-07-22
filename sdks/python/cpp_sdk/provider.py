"""Abstract base classes for CPP context providers.

These mirror the Rust ``ContextProvider`` and ``ProviderAdapter``
traits so that Python developers can write new data-source plugins
using the same interface.
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any, Generic, TypeVar

from pydantic import BaseModel, ConfigDict, Field
from pydantic.alias_generators import to_camel

from .context import ContextBundle, ContextObject
from .errors import CppProviderError
from .query import ContextQuery
from .types import ContextType, Goal

_CAMEL_CONFIG = ConfigDict(
    populate_by_name=True,
    alias_generator=to_camel,
    use_enum_values=True,
)


class ProviderCapabilities(BaseModel):
    """Declares what context types and goals a provider supports."""

    model_config = _CAMEL_CONFIG

    context_types: list[str] = Field(default_factory=list)
    goals: list[str] = Field(default_factory=list)
    supports_streaming: bool = False
    supports_subscriptions: bool = False

    @classmethod
    def basic(
        cls,
        context_types: list[ContextType],
        goals: list[Goal],
    ) -> ProviderCapabilities:
        return cls(
            context_types=[ct.value for ct in context_types],
            goals=[g.intent for g in goals],
        )


class ProviderManifest(BaseModel):
    """Registration manifest for a context provider."""

    model_config = _CAMEL_CONFIG

    id: str
    name: str
    description: str = ""
    version: str = "0.1.0"
    capabilities: ProviderCapabilities = Field(default_factory=ProviderCapabilities)


class ContextProvider(ABC):
    """Abstract base class for CPP context providers.

    Subclass this to create a new data-source plugin (e.g. GitHub,
    Jira, Slack, database, etc.).

    Example::

        class MyProvider(ContextProvider):
            def __init__(self):
                self._manifest = ProviderManifest(
                    id="my-provider",
                    name="My Custom Provider",
                    capabilities=ProviderCapabilities.basic(
                        context_types=[ContextType.file()],
                        goals=[Goal.code()],
                    ),
                )

            @property
            def manifest(self) -> ProviderManifest:
                return self._manifest

            async def query(self, query: ContextQuery) -> ContextBundle:
                ...

            async def resolve(self, uri: str) -> ContextObject:
                ...
    """

    @property
    @abstractmethod
    def manifest(self) -> ProviderManifest:
        """Return this provider's manifest declaring its capabilities."""
        ...

    @abstractmethod
    async def query(self, query: ContextQuery) -> ContextBundle:
        """Query the provider for context matching the given query."""
        ...

    @abstractmethod
    async def resolve(self, uri: str) -> ContextObject:
        """Resolve a single context object by its CPP URI."""
        ...

    async def start(self) -> None:
        """Optional lifecycle hook called when the provider is registered."""

    async def stop(self) -> None:
        """Optional lifecycle hook called when the server shuts down."""


# ---------------------------------------------------------------------------
# Provider adapter (for translating external records → SCOs)
# ---------------------------------------------------------------------------

T = TypeVar("T")


class ProviderAdapter(ABC, Generic[T]):
    """Abstract adapter for translating external records into SCOs.

    Type parameter ``T`` is the native external record type (e.g. a
    GitHub PR dict, a Jira issue JSON, a Slack message payload).

    Example::

        class GitHubPRAdapter(ProviderAdapter[dict]):
            def adapt(self, record: dict) -> ContextObject:
                return (
                    ContextObjectBuilder(
                        f"cpp://github/pull_request/{record['number']}",
                        ContextType.pull_request(),
                        "github",
                    )
                    .title(record["title"])
                    .summary(record["body"][:200])
                    .build()
                )

            def resolve_uri(self, uri: str) -> str:
                # Extract PR number from CPP URI
                return uri.split("/")[-1]
    """

    @abstractmethod
    def adapt(self, record: T) -> ContextObject:
        """Convert an external record into a ``ContextObject``."""
        ...

    @abstractmethod
    def resolve_uri(self, uri: str) -> str:
        """Extract the external identifier from a CPP URI.

        For example, ``"cpp://github/pull_request/42"`` → ``"42"``.
        """
        ...
