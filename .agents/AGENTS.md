# Project Context Provider Protocol (CPP) Rule

This workspace strictly uses **Context Provider Protocol (CPP)** as its default perception layer, managed internally by Antigravity.

1. **Auto-Server Management**: Ensure `cpp-server` is running on `http://localhost:3030`. If down, execute `.agents/scripts/ensure_cpp_daemon.sh`.
2. **Perceive First**: Before inspecting or modifying files, execute CPP context resolution (`goal.code` / `goal.project`) via `CppClient` or RPC query against `http://localhost:3030`.
3. **Budget Enforcement**: Keep context window consumption under 4KB per query turn.
4. **Structured Graph**: Rely on SCO URIs (`cpp://...`) for symbol and file paths.
