# Project Context Provider Protocol (CPP) Rule

This workspace strictly uses **Context Provider Protocol (CPP)** as its default perception layer.

1. **Perceive First**: Before inspecting or modifying files, execute CPP context resolution (`goal.code` / `goal.project`) via `simple-query` or CPP server.
2. **Budget Enforcement**: Keep context window consumption under 4KB per query turn.
3. **Structured Graph**: Rely on SCO URIs (`cpp://...`) for symbol and file paths.
