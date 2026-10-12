# RazedTunnel module boundaries — refactor plan

Status: design and dependency mapping only; no runtime changes.

## Product boundary

RazedTunnel is the independent secure MCP gateway for explicitly approved local projects. Askew owns editing, terminal UX, AI workspace, and workflow orchestration. A module belongs in the default gateway only if it serves project approval, secure filesystem access, Git, MCP transport/auth, connection lifecycle, or essential diagnostics.

## Current hotspots (2026-10-12)

| File | Lines | Current concern | Intended role |
| --- | ---: | --- | --- |
| `src-tauri/src/mcp_server.rs` | 12,064 | Protocol, parameters, task helpers, domain implementations, legacy tools mixed together | Thin MCP router and protocol adapters |
| `src-tauri/src/commands.rs` | 4,256 | Tauri commands and domain logic mixed together | Thin Tauri command registration adapters |
| `src-tauri/src/lib.rs` | 450 | Module declarations, startup and command registration | Application composition root |
| `src-tauri/src/media_tools.rs` | 346 | Shared FFmpeg utility extracted from video | Review whether media belongs in default gateway |

Line counts are an inspection snapshot, not a quality metric. Prefer cohesive files around 100–400 lines; review files over 500, but do not split merely to satisfy a number.

## Target ownership

| Domain | Core implementation | MCP adapter | Tauri adapter |
| --- | --- | --- | --- |
| Approved projects and policies | `projects/` using `access`, `storage`, `models` | `mcp/projects.rs` | `commands/projects.rs` |
| Filesystem and changes | `filesystem`, `changes`, `hardening` | `mcp/filesystem.rs` | `commands/filesystem.rs` |
| Git and repository | `git`, `repository` | `mcp/git.rs` | `commands/git.rs` |
| Connections and OAuth | `gateway`, `connection`, `mcp_auth`, `public_tunnel` | `mcp/connections.rs` if needed | `commands/connections.rs` |
| Settings and diagnostics | `storage`, `app_state`, `monitoring` (core subset) | `mcp/diagnostics.rs` if needed | `commands/settings.rs` |
| Inherited optional features | `legacy/` or feature-gated modules | Opt-in legacy router | No default command registration |

These are target boundaries, not instructions to rename modules immediately. Extract business logic first; retain external MCP tool names, Tauri command names, serialized payloads, and authorization behavior.

## First extraction: approved project registry

The opening section of `commands.rs` currently combines project path validation, ID allocation, approved-workspace lookup, status, and commands including `select_workspace`, `list_workspaces`, `get_workspace_health`, `relocate_workspace`, `register_workspace_path`, `add_workspace`, `clone_git_workspace`, `create_project`, `remove_workspace`, and workspace access/change/command policies.

1. Inventory each project's call sites and shared helper visibility before moving definitions.
2. Move registry and validation logic into a `projects` service, preserving storage and policy semantics; keep Tauri command wrappers unchanged initially.
3. Move cohesive Tauri project commands into `commands/projects.rs` with stable re-exports or registration paths.
4. Extract MCP project tools into a whole `#[tool_router]` impl/router group, not per-method cfg flags; preserve tool names and authorization checks.
5. After each patch, validate `cargo check --manifest-path src-tauri/Cargo.toml`, then check the UI's project list and project approval flows.

## Safety and size guardrails

- **Never** bypass approved-project checks or broaden access, including during refactor.
- Preserve existing project IDs, persistence paths, serialized API payloads, OAuth behavior, and Git safeguards.
- Preserve the original source until the extracted implementation compiles and the app works.
- Avoid feature flags on individual `#[tool]` methods in rmcp `#[tool_router]` impls; gate whole router impls or compose routers explicitly.
- Prefer small commits with one Rust check per Rust patch. Keep test-executable Windows loader issue separate from compile checks.
- After moving a feature, remove unused imports, duplicate implementations, and dependencies rather than leaving forwarding layers indefinitely.
- Check both default and legacy-feature builds before claiming reversible legacy support; default build alone is insufficient.
- Track **source reduction**, **default compiled surface**, **module coupling**, and **unchanged external behavior** separately.

## Exit criteria

- `mcp_server.rs` owns protocol setup and router composition, not unrelated domain logic.
- `commands.rs` owns registration and thin adapters, not business logic.
- Core services have one owner and can be used from both MCP and Tauri.
- Legacy modules and dependencies are either explicitly supported behind validated flags or removed after migration.
- Project approval, filesystem authorization, Git, OAuth, connections, and tray/runtime behavior still work.
