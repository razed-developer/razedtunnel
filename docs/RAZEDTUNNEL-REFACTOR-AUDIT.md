# RazedTunnel architecture audit — initial baseline

Status: discovery only. No legacy modules removed.

## Product boundary

RazedTunnel is a secure, independently running gateway exposing explicitly approved local project checkouts through MCP. Askew owns editing, terminal UX, coding assistants and project workflow.

## Core runtime to preserve

- `gateway.rs`, `mcp_server.rs`, `mcp_auth.rs`, `app_state.rs`, `connection.rs`, `public_tunnel.rs`, `direct_https.rs`: transport, MCP, authentication, connection lifecycle.
- `access.rs`, `hardening.rs`, `secret_guard.rs`, `platform_sandbox.rs`, `storage.rs`: policy, secrets and persistence.
- `filesystem.rs`, `git.rs`, `repository.rs`, `changes.rs`, `execution.rs`: project capabilities. Keep execution gated by project policy.
- `commands.rs`, `models.rs`: currently large shared interfaces; split only after dependency mapping.

## Candidate removals or relocations (NOT safe to delete yet)

- UI and features related to `model_hub`, `model_trial`, `conversation`, `ai_workspace`, `team`, `phone`, `video*`, `browser`, `desktop_control`, `media_inspection`, `video_production`, `workflow`.
- Other optional domains: `gmail_access`, `integrations`, `self_continuation`, `chatgpt_bridge`, `semantic`, `project_memory`, `project_index`, `ai_resources`.

These modules are compiled and may be referenced by MCP tools, commands, or startup/shutdown hooks even if their screens are hidden. Removal requires tracing references, preserving externally exposed tool contracts where needed, and running Windows/Rust tests.

## MCP verification: current evidence and gap

- `gateway.rs` validates OAuth bearer tokens on forwarded HTTPS MCP paths and calls `AppState::record_remote_request()` after validation.
- `app_state.rs` exposes an authenticated request counter and timestamp through `PublicTunnelStatus`. The counter is **per launch**, not a client identity or successful tool-execution counter.
- `mcp_server.rs` uses rmcp's `#[tool_handler]` router, with tools returning `CallToolResult`. A tool may return a protocol-level error result even when HTTP succeeds.
- **Do not equate HTTP 200, OAuth validation, or request count with successful MCP tool execution.** Instrument at the rmcp tool-dispatch/result boundary, count `CallToolResult` successes and failures separately, and never log credentials or tool arguments.
- Verification must also cover local MCP clients, remote OAuth clients, status refresh, and counter reset on restart.

## Refactor sequence

1. Add explicit per-launch MCP tool outcome counters and last successful tool timestamp at the actual tool result boundary; tests must cover successful result, tool error result, and transport error.
2. Freeze the essential MCP tool surface and map frontend/backend references and startup hooks.
3. Remove unrelated UI imports, branches, and dead state in small commits.
4. Extract project registry, policy, Git, gateway and connections modules from shared commands/types.
5. Remove unused Rust feature modules and dependencies in dependency-safe groups.
6. Verify Windows build, Rust tests, frontend checks, OAuth access and background/tray operation after each group.

## Guardrails

- Never silently broaden access while refactoring. Cloned projects default to read-only, review edits, commands disabled.
- Never delete user data or existing registered workspaces as a migration side effect.
- Keep commits small and reversible. Treat external MCP tool names as compatibility surface.
