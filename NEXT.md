## Rust compilation cleanup: media desktop IPC wrapper (awaiting validation)
- User validated previous GitHub/integration/AI Workspace wrapper exclusion; warnings decreased from 376 to **359** (437 original; 78 fewer).
- Audited `media_commands.rs` (58 lines), `lib.rs`, `mcp_server.rs`, and `app_state.rs`. The media desktop wrapper still had three Tauri `generate_handler!` registrations, unlike earlier wrappers. No direct references from inspected MCP or AppState modules.
- Commit `93283c1` removes the three desktop media IPC registrations (`inspect_media_file`, `extract_media_frame`, `validate_media_decode`) **and** `mod media_commands;` together, excluding the wrapper from compilation. Source file remains. Core `media_inspection`, `media_tools`, project permission validation, and MCP remain untouched.
- Did not remove `monitoring_commands` because its five registrations remain and monitoring initialization resumes persistent project watchers. Audit separately before touching monitoring.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`, check Projects/Connections/tray/MCP access, report warnings vs 359. No local compilation performed by assistant.
- Next audit remaining wrappers and cross-module references; preserve MCP functionality. Do not merge.

## Rust compilation cleanup: GitHub/integration/AI Workspace desktop wrappers (awaiting validation)
- User confirmed prior project-lifecycle/temp-workspace wrapper cleanup passed; compiler warnings decreased from 387 to **376** (437 original, 61 fewer).
- Inspected `github_commands.rs` (42 lines), `integration_commands.rs` (67 lines), `ai_workspace_commands.rs` (146 lines), `lib.rs`, and `mcp_server.rs`. No direct references to these wrapper module names from MCP; confirmed no `generate_handler!` registrations remain for these wrappers in `lib.rs`.
- Commit `c1fbae7` removes only `mod github_commands;`, `mod integration_commands;`, `mod ai_workspace_commands;` from `lib.rs`. Core `github`, `integrations`, `gmail_access`, `desktop_control`, `ai_workspace`, `browser`, and MCP remain compiled. Source files retained. This does not remove persistent credentials, gateway, access controls, or MCP functions.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; verify Projects/Connections/tray/MCP and report warning count against 376 baseline. Assistant has not compiled locally.
- Next audit remaining unregistered wrappers and their cross-module references before exclusion; preserve active desktop commands and all MCP functionality. Do not merge.

## Rust compilation cleanup: project lifecycle and temporary-workspace desktop wrappers (awaiting validation)
- User validated prior editor/filesystem/diagnostics cleanup after stale handler fix; Rust warning count fell from 407 to **387** (original 437; 50 fewer).
- Reviewed `lib.rs` Tauri `generate_handler!`, `project_lifecycle_commands.rs` (76 lines), `temp_workspace_commands.rs` (103 lines), and searched `mcp_server.rs`, `app_state.rs`, `workspace_management_commands.rs`, `workspace_policy_commands.rs` for direct wrapper references. No registration or direct references to these two wrappers found in inspected modules.
- Commit `06c0f10` removes only `mod project_lifecycle_commands;` and `mod temp_workspace_commands;` from `lib.rs`, excluding two obsolete desktop wrappers from compilation. Files remain in Git. Core `project_setup`, `project_memory`, `continuity`, `temp_workspace`, and MCP implementation remain unchanged.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`, then Projects/Connections/tray/MCP access; report new warning count compared to 387 baseline. Assistant did not compile locally.
- Continue auditing all Rust dependencies and registrations before removing additional modules. Do not merge branch.

## Follow-up fix: remove stale Tauri handler registration (awaiting validation)
- User's rerun after `6ad388e` failed E0433: `lib.rs` still registered `workspace_policy_commands::open_workspace_path_local` in `tauri::generate_handler!` although the function had been removed. Previous checkpoint incorrectly described it as already unregistered.
- Commit `83cfbb8` removes that exact handler entry; no other command registrations, MCP implementations, or runtime logic changed.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. Previous successful warnings baseline remains 407; two warnings from failed checks are not a meaningful comparison.
- Before future wrapper/module exclusion, audit **both** references from other Rust modules and `generate_handler!` entries to avoid repeating these failures. No merge.

## Fix: orphaned desktop open-path helper after editor wrapper exclusion (awaiting validation)
- User reported `cargo check` E0433 at `workspace_policy_commands.rs:105`: `open_workspace_path_local` referenced excluded `crate::editor_commands::local_user_workspace`; frontend check/build passed. The command's Tauri IPC registration had already been removed in a prior stage.
- Commit `33ecbed` removes this unregistered `open_workspace_path_local` desktop-only function and its now-unused `launcher`, `LaunchActionOutcome`, `approved_workspace` imports. Retains live workspace policy setters and all MCP path-opening/launcher implementations unchanged. `editor_commands` remains excluded from compilation.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. The prior compiler warning count of 407 is the latest successful baseline; failed compile's 2 warnings are not comparable.
- Avoid declaring success until user confirms. Next examine remaining desktop wrapper module references before further exclusion. Do not merge.

## Rust compilation cleanup: editor/filesystem/diagnostics wrappers (awaiting validation)
- User confirmed previous history/checkpoint/safety wrapper cleanup passed; compiler warnings fell from 422 to 407 (437 original).
- Reviewed `editor_commands.rs` (245 lines), `filesystem_commands.rs` (35 lines), `workspace_diagnostics_commands.rs` (82 lines), `mcp_server.rs`, `change_commands.rs`, and `lib.rs`. No direct MCP or remaining wrapper references to these desktop modules found in inspected files; desktop IPC entries were previously removed.
- Commit `d7a1788` removes only three `mod` declarations from `lib.rs`: `editor_commands`, `filesystem_commands`, `workspace_diagnostics_commands`. Their source files remain in Git; core `filesystem`, `access`, `project_index`, `workflow`, and MCP remain compiled and unchanged.
- **Important**: `editor_commands` exports `local_user_workspace`; compilation may reveal a reference from another Rust module not inspected. If `cargo check` reports a missing `crate::editor_commands`, restore the declaration or relocate that helper carefully. Do not claim validation before user tests.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; test Projects/Connections/tray/MCP and report warnings vs 407 baseline.
- Next continue wrapper dependency review, not core module removal without full cross-module proof. Do not merge.

## Rust compilation cleanup: history/checkpoint/safety desktop wrappers (awaiting validation)
- User validated preceding three-wrapper removal and reported warnings decreased from 437 to 422 (15 fewer).
- Reviewed `history_commands.rs`, `checkpoint_commands.rs`, `safety_commands.rs` and `mcp_server.rs`: these are desktop Tauri command wrappers, their IPC registrations were previously removed, and no direct references from `mcp_server.rs` to the wrapper modules were found.
- Commit `6de1917` removes only `mod history_commands;`, `mod checkpoint_commands;`, and `mod safety_commands;` declarations from `lib.rs`, excluding those wrapper files from Rust compilation. Core `activity`, `checkpoint`, `changes`, `versioning`, `workflow`, `hardening`, and MCP implementations remain intact; source files retained.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; check Projects/Connections/tray/MCP. Report warning count compared with 422 baseline. Assistant cannot run local build.
- Next inspect other unregistered wrapper modules; only then investigate core module dependencies. Never suppress warnings wholesale or merge this cleanup branch.

## Rust module audit: remove three desktop-only command wrapper modules from build (awaiting validation)
- User confirmed prior 7-integration-IPC cleanup passed and reported `razedtunnel (lib)` emitted 437 compiler warnings. Do not suppress warnings indiscriminately; many are dead-code candidates after IPC pruning, but count alone is not a dependency proof.
- Inspected `lib.rs`, `model_hub_commands.rs`, `model_trial_commands.rs`, `conversation_commands.rs`, and searched `mcp_server.rs` for these wrapper module names. These three files consist of Tauri command wrappers for previously unregistered Home chat, Model Hub, and model trial UI, with no direct MCP references observed.
- Commit `7e5c379` removes **only** `mod conversation_commands;`, `mod model_hub_commands;`, `mod model_trial_commands;` from `lib.rs`, so those obsolete wrappers are no longer compiled. Their files remain in Git for now; **core** `conversation`, `model_hub`, `model_trial` and all MCP modules remain compiled and untouched. This is the first module-level compilation reduction, not deletion of implementation or data.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`, then Projects/Connections/tray/MCP access. If cargo check reports unresolved module references, restore declaration or fix dependencies before advancing. Record new warning count (437 baseline, counts may vary with compiler/config).
- Next: map references to core legacy modules and determine whether they are actually needed by MCP or startup before removing more. Do not merge.

## Startup dependency audit: monitoring and terminal (audit only; no runtime change)
- User validated integration IPC removal; prior total 74 desktop IPC registrations removed.
- Reviewed `src-tauri/src/lib.rs`, `monitoring.rs`, `terminal.rs`, and `app_state.rs` before attempting actual Rust module/startup removal.
- `monitoring::initialize(app)` is **not** just a no-op: it loads persisted monitored project IDs, prunes stale entries, and resumes enabled project watchers with `start_internal`. Disabling it could silently change MCP monitoring behavior. `monitoring::stop_all_activity()` joins watcher threads on exit.
- `terminal::initialize(app)` marks interrupted running terminal commands failed in persistent history and refreshes managed processes; shutdown calls `terminal::stop_transient_activity_for_shutdown()`. Removing it could compromise MCP command/process lifecycle recovery.
- `initialize_connection_runtime` auto-starts configured public tunnel; keep it. The `legacy_background_services` flag already gates GitHub status prewarm, AI Workspace recovery, and ChatGPT bridge startup. `model_hub::stop_owned_local_runtimes()` is called on exit; do not remove until ownership/dependency audit.
- **No Rust code changed this stage:** preserving behavior takes priority over speculative startup optimization. Next perform a cross-module MCP reference audit and isolate a truly unused implementation before deleting code. Do not remove monitoring/terminal init just because their desktop pages are gone. No merge.

## Rust IPC audit: integration settings desktop registrations (awaiting validation)
- User confirmed previous AI Workspace IPC cleanup passed; prior total removed: 67.
- Checked mounted App.tsx, WorkspaceList, ChatConnectionPanel, PublicTunnelPanel, GatewayPanel: no calls to integration settings desktop IPC. Commit `21fd1ca` unregisters seven Tauri IPC entries: list/set deep integrations (2), list/get/set desktop control (3), get/set Gmail access (2).
- Only desktop command registrations changed. `integration_commands` Rust module and MCP-side integration implementations/access policy are untouched; no changes to persistent integration settings, gateway, or project permissions. No claim that these capabilities are no longer reachable via MCP.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml` plus Projects/Connections/tray/MCP access. Assistant did not compile locally.
- Next: inventory Rust module dependencies and startup activity before deleting any implementations; prioritize lean tray gateway, preserve MCP capabilities. No merge.

## Rust IPC audit: AI Workspace desktop registrations (awaiting validation)
- User confirmed prior GitHub account IPC cleanup passed; prior total removed: 61.
- Reviewed mounted App, ChatConnectionPanel, PublicTunnelPanel, GatewayPanel. No mounted UI calls `ai_workspace_commands` or `integration_commands`. Commit `fa1bfdd` unregisters six AI Workspace desktop IPC commands (status/start/stop/frame/action/sequence) associated with the removed editor UI. Kept all integration registrations for a separate audit.
- AI Workspace Rust modules, MCP tools, gateway, project permissions, and legacy background-service feature remain untouched. This stage only changes the Tauri invoke handler registration list.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml` and Projects/Connections/tray/MCP access. Assistant did not compile locally.
- Next audit integration desktop IPC, then distinguish Rust modules required by MCP from unused UI-only code before deletion. Do not merge branch.

## Rust IPC audit: GitHub account desktop registrations (awaiting validation)
- User confirmed six-command temporary workspace cleanup passed; total prior desktop IPC registrations removed: 57.
- Commit `c04fb23` unregisters four GitHub account-management Tauri IPC entries (`get_github_connection_status`, `connect_github`, `cancel_github_connection`, `disconnect_github`) for the removed desktop GitHub account UI. No mounted Projects/Connections interface uses them. This does not remove git/GitHub Rust modules, CLI credentials, MCP git/GitHub tools, or gateway/auth operations.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; check Projects/Connections/tray and MCP Git access if practical. Assistant did not compile locally.
- Next review integration and AI-workspace desktop IPC groups with care: retain any controls needed by Connections and preserve all MCP implementations. Prioritize dependency inventory before removing actual Rust modules or startup services.

## Rust IPC audit: temporary workspace desktop registrations (awaiting validation)
- User confirmed seven-command diagnostics cleanup passed; total prior desktop IPC registrations removed: 51.
- Commit `79346b6` unregisters six temporary-workspace desktop IPC entries (create/list/inspect/preserve/cleanup/file action). None are used by mounted Projects/Connections pages. The underlying temporary workspace implementation and MCP tools are unchanged; only Tauri command exposure removed.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`, and Projects/Connections/tray/MCP access. Assistant did not compile locally.
- Next inspect remaining desktop-only command groups and startup services, preserving MCP tools and all live gateway functionality. Do not merge branch.

## Rust IPC audit: diagnostics desktop registrations (awaiting validation)
- User confirmed project lifecycle IPC removal passed. Previous cleanup total 44 registrations removed.
- Inspected mounted App, WorkspaceList, ChatConnectionPanel, GatewayPanel, PublicTunnelPanel. None reference seven `workspace_diagnostics_commands` IPC entries. Commit `b6c44df` unregisters inspect_project, get_workflow_readiness, get_environment_diagnostics, list_tool_capabilities, get_system_resource_snapshot, list_launchable_applications, get_file_info (7).
- `workspace_management_commands::get_workspace_health` remains registered and used by live Projects. Diagnostics Rust modules and all MCP tools remain compiled, unchanged. No Rust implementation deletion.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`, then Projects/Connections/tray/MCP project file access. Assistant has not compiled locally.
- Next audit remaining unused Tauri IPC groups and startup services; avoid removing anything used by MCP or tray. No merge.

## Rust IPC audit: project lifecycle desktop registration cleanup (awaiting validation)
- User confirmed previous editor/file-explorer IPC cleanup passed. Prior three Rust cleanup stages removed 39 registrations.
- Inspected mounted App.tsx, WorkspaceList, ChatConnectionPanel, GatewayPanel. None use `project_lifecycle_commands` desktop IPC; App still uses `get_workspace_health` and it remains registered.
- Commit `969ea99` unregisters five Tauri-only project lifecycle commands: get_project_setup, prepare_project, get_project_memory, get_resume_snapshot, update_project_memory. Rust project lifecycle implementations and MCP tools remain compiled and unchanged. No deletion of storage, project setup, memory, permissions, or gateway functionality.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml` and Projects/Connections/tray/MCP access. Assistant cannot run local builds.
- Next audit seven diagnostics registrations separately; preserve `workspace_management_commands::get_workspace_health` used by live Projects UI. Audit dependency graph before deleting Rust modules.

## Rust IPC audit: editor and file explorer registration cleanup (awaiting validation)
- User confirmed previous 15-command history/checkpoint/safety desktop IPC removal passed.
- Reviewed mounted `App.tsx`, `WorkspaceList`, `ChatConnectionPanel`, and `GatewayPanel`: none invoke removed editor/file-explorer desktop IPC. Commit `738579b` unregisters 9 Tauri commands: `filesystem_commands` list/read/search (3), `editor_commands` save/create file/create directory/rename/delete/image preview (6).
- This is *only* Tauri IPC registration removal. `filesystem`, `filesystem_commands`, `editor_commands`, and all MCP tool modules remain compiled; no changes to MCP routes, permissions, project CRUD, or gateway lifecycle. Do not confuse removal of desktop IPC with removal of MCP filesystem tools.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml` plus Projects/Connections/tray/MCP file access. Assistant has not compiled locally.
- Next review old desktop project lifecycle and diagnostics registrations against mounted UI, and audit actual Rust dependencies before deleting implementations.

## Rust IPC audit: history/checkpoint/safety registrations (awaiting validation)
- User confirmed prior Rust Model Hub/Home registration cleanup passed, including cargo check and app behavior.
- Inspected `lib.rs` Tauri invoke handler groups and mounted `App.tsx`/Projects UI dependencies. Commit `8facb67` unregisters 15 desktop-only IPC entries: history (6), checkpoints (8), safety scan (1). These are unreachable from the mounted two-page desktop interface. This change does **not** remove modules, MCP tool handlers, storage, permission checks, or gateway services. Changes and approval IPC remain registered pending MCP/control review.
- Run `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; verify Projects, Connections, tray and remote MCP access. No local compile by assistant.
- Next audit editor/filesystem UI IPC separately; ensure MCP tool access is preserved before removing any backend implementations. Avoid deleting Rust modules based solely on removed desktop screens.

## Rust IPC audit: first registration-only removal (awaiting local validation)
- User confirmed previous two-page navigation fix passed.
- Inspected `src-tauri/src/lib.rs`: Tauri IPC registers extensive legacy UI commands while MCP server is separate; core Projects/Connections commands and gateway/public tunnel/chat connection remain required. Startup also initializes terminal and monitoring; do not remove without MCP dependency audit.
- Commit `a610b15` unregisters 15 Tauri IPC commands for unmounted Model Hub, model trials, and Home chat only. All implementation modules, Rust MCP tool registrations, and core gateway/project commands are untouched. This is intentionally registration-only, not deletion of underlying services.
- Run `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. Verify Projects, Connections, tray and MCP connectivity. Assistant could not compile locally.
- Next inspect Rust MCP references and frontend backend calls before unregistering more UI-only IPC. Preserve MCP operations irrespective of UI removal.

## Two-page navigation typecheck fix (awaiting validation)
- User's `npm run check` reported four TS2345 errors in unmounted `WorkspaceProductivity.tsx` command palette: Home, History, Commands, Settings navigation no longer matches two-page AppView.
- Commit `fe29008` removed four unsupported navigation actions and added Connections alongside Projects in that component. It remains unmounted; no runtime MCP changes.
- Re-run `git pull --ff-only && npm run check && npm run build`. Do not proceed to Rust audit until passing.

## Two-page navigation type cleanup (awaiting local validation)
- User confirmed stale clone setter fix passed.
- Commits `5b5e129`, `4f7b634`: AppHeader now renders only Projects/Connections title/icon; AppSidebar AppView type narrowed to those two pages. No gateway, project or Rust MCP behavior changed. Header focus/status controls intentionally preserved.
- Validate `git pull --ff-only && npm run check && npm run build`, then tray Projects/Connections and normal sidebar navigation. Other components may import AppView; fix any TypeScript errors before further cleanup. No local compile performed by assistant.
- Next: dependency-led Rust backend audit; preserve every MCP operation still required for ChatGPT-approved projects, regardless of removed UI.

## Clone TypeScript regression fix (awaiting validation)
- User's `npm run check` caught TS2552 at `src/App.tsx:266`: clone handler still called removed `setSelectedWorkspaceId`.
- Commit `4a92301` removes the stale setter. Clone still refreshes workspaces and navigates to Projects. No backend changes.
- Re-run `git pull --ff-only && npm run check && npm run build`; test clone if practical. Do not advance until validated.

## Obsolete selected-project session cleanup (awaiting local validation)
- User confirmed previous shell polling cleanup passed.
- Commits `b2386e2`, `80b233d`: removed hidden selectedWorkspaceId/startupProjectRestored React state and last-workspace localStorage restoration effect left over from editor; simplified error-boundary reset to current visible page; removed editor alias in Projects nav highlighting. Project approval/listing/health and all MCP/tray controls unchanged. App.tsx ~669 lines.
- Run `git pull --ff-only && npm run check && npm run build`; test Projects, Connections and tray. No local build performed by assistant.
- Next: audit AppView and AppHeader unused views with dependency check, then Rust backend command/dependency inventory. Avoid removing MCP tools just because their UI was deleted.

## Shell polling cleanup (awaiting local validation)
- User confirmed prior Home/History removal passed.
- Commits `7839150`, `344fa3e`, `6de1cca`: removed unused pending-change history polling, listChanges startup fetch, obsolete background update notification pointing to removed Settings, and unused pendingCount sidebar property. App.tsx ~689 lines. Projects, Connections, tray navigation, project-health polling, public tunnel/chat connection status polling, and gateway controls remain intact. Rust MCP untouched.
- Validate `git pull --ff-only && npm run check && npm run build` and smoke-test both pages/tray. No local test run by assistant. Next evaluate obsolete selected-workspace restoration and old focus-mode shell, then backend dependency audit.

## Unreachable Home/History cleanup (awaiting local validation)
- User confirmed previous state pruning passed.
- Commit `fe78aee`: removed unmounted Home/History/checkpoint/safety handlers, History-only activity event listener, change-history refresh branches, and associated React state and backend imports. `App.tsx` reduced from ~1,040 to ~738 lines.
- Preserved project CRUD/permissions, project health, connection controls, MCP access pause status, tray navigation, and pending-change counter polling for now. Backend Rust untouched. The update-check notice still references removed Settings and can be cleaned later.
- Run `git pull --ff-only && npm run check && npm run build`; verify Projects, Connections and tray. This was not locally compiled by the assistant; resolve any errors before next stage.

## Gateway-only state pruning (awaiting local validation)
- User confirmed prior editor callback cleanup passed and app works.
- Commits `dfd5b4a`, `cc1f13e`, `e45dbdd`, `ac8e91a`: removed editor sync callback and event handling, orphaned editor React state/refs, filesystem/editor type imports, and unreachable Git/Model Hub refresh/polling. Project relocation/removal no longer refer to removed editor tabs. App.tsx now ~1,040 lines, down from 1,217. Backend MCP and permissions unchanged.
- Run `git pull --ff-only && npm run check && npm run build`, then verify Projects, Connections, tray shortcuts. No remote CI validation performed. If TS errors occur, fix before advancing. More unused Home/History/Checkpoint state remains and should be pruned in subsequent stages.

## Editor handler removal (awaiting validation and follow-up)
- User confirmed prior editor background-effects removal passed and works.
- Commits `63d9bcc`, `8fa1734`: deleted unreachable editor open/save/reload/tab/file-tree callback block and obsolete top-level editor document/session helper functions. `App.tsx` reduced from 1,603 to 1,217 lines.
- **Important**: editor-related React state, refs, `syncEditorFiles`, and some change-event callbacks still remain. This is intentionally incomplete; TypeScript may flag unused symbols depending on the check script. If `npm run check` fails, next patch should remove dangling state/effects and imports before proceeding. No Rust MCP changes.
- Validate `git pull --ff-only && npm run check && npm run build`, plus two-page/tray smoke test. Do not merge until all pass.

## Editor background effects removal (awaiting validation)
- User confirmed previous dialog cleanup passed checks and both pages work.
- Commit `51e41d8`: removed editor-session restoration and persistence effects, 3.5-second editor file polling, and unreachable Home model refresh polling; initial startup no longer calls Model Hub refresh. App.tsx reduced from 1,732 to 1,603 lines. Crucially, saved editor drafts in localStorage are left untouched (not deleted), but no longer loaded or updated by the gateway UI.
- Existing unused editor callbacks/state remain for next phase. Change event handler still has conditional editor sync and history logic; simplify only after tests. MCP backend and permissions untouched.
- Validate `git pull --ff-only && npm run check && npm run build`, then verify Projects/Connections/tray; report any TS errors before proceeding.

## Gateway shell dialog cleanup (awaiting validation)
- Commit `95d5020`: removed unreachable HomeFeatureDialog, NewProjectDialog, old editor-switch confirmation, and unused ProjectSetupPanel/ProjectMemoryPanel imports from the two-page UI. Retained project removal and gateway credential confirmation dialogs. No MCP/backend changes.
- The legacy editor/home state and callbacks remain in `App.tsx` despite being unreachable; remove incrementally after validation rather than deleting dependent hooks blindly. Existing create-project path is currently inaccessible from the two-page shell (it was only on Home); adding approved local folders remains available on Projects.
- Validate `git pull --ff-only && npm run check && npm run build`; visually check Projects and Connections, and tray navigation. Next prioritize replacing dead editor/home state and startup effects, ensuring no editor file monitoring runs in background.

## Validation: gateway-only layout (2026-10-12)
- User confirmed the responsive Projects/Connections layout fix works. Regression resolved.
- Next: incremental removal of unused frontend state and imports, keeping Projects/Connections functionality and tray navigation intact.

## UI layout regression fix (awaiting validation)
- User screenshot after removing ProjectRail showed severely compressed Projects content, overlapping permission controls and clipped header. This is a regression; do not proceed with more deletion until layout is confirmed.
- Commit `61e1c96`: targeted gateway-only CSS for two-column shell, content sizing, wrapping permission actions and narrow-window responsive sidebar. No backend changes.
- Validate `git pull --ff-only && npm run check && npm run build`; launch and visually inspect Projects and Connections at normal window width and when narrowed. If still compressed, inspect computed widths / old CSS overrides before further cleanup.

# RazedTunnel — NEXT

Updated: 2026-10-12
Current branch: `cleanup/dead-video-view` (not merged)
Purpose: durable handoff between ChatGPT conversations. **Read this file first** before changing code. Keep it updated after each validated stage, before switching chats, and after merges.

## Completed stage: legacy documentation cleanup
- Branch `cleanup/legacy-docs` created from `main` after Team Mode PR #2 merged; merged as PR #3, merge commit `b5d3f91`.
- Commit `83f34f9`: removed retired Android Phone, Video Production, and Team Mode feature claims and links from README, including architecture and security sections. This is documentation-only; no runtime changes.
- Commit `0548ec6`: fixed README grammar introduced by prior deletion.
- Commits `f802b4a`, `e3e72de`, `721f995`: deleted obsolete feature guides `docs/team-mode.md`, `docs/phone-access.md`, and `docs/video.md`.
- Commit `a22bdcf`: removed retired Team/Phone/Video feature descriptions and related limitations from `docs/product.md`.
- Commits `7260fba` and `6cd6584`: removed obsolete Android/Video/Team architecture and MCP tool contract claims, preserving Continuity and Project Memory sections.
- Commits `cf8b69c`, `41e7893`, `b43698d`: removed retired Phone security text, retired Team/Video/Phone acceptance scenarios (renumbered remaining sections), and stale Team update blocker claim in release guide.
- Final audit of primary/root docs found and fixed stale security/semantic references in commits `00083d7` and `a81ab05`. `AI_CONTINUITY.md` retains historical Video test results as a dated record, not a current feature claim.
- Branch diff was documentation-only (13 files); user confirmed `git diff --check main...HEAD` passed. PR #3 merged into `main` as `b5d3f91`.
- Next: return to oversized Rust module decomposition, after reviewing current file sizes and choosing one focused refactor. Keep unrelated local edits to `src-tauri/Cargo.lock` and `src-tauri/src/github.rs` intact. Do not touch unrelated local edits to `src-tauri/Cargo.lock` or `src-tauri/src/github.rs`.

## Post-merge source inventory (2026-10-12)
- Read-only audit of `main`: `src/App.tsx` ~1,949 lines, `src-tauri/src/mcp_server.rs` ~5,939 lines, `src-tauri/src/lib.rs` declares ~99 modules.
- Sidebar currently exposes Projects and Connections in its main nav, with System and Help controls. Many other view handlers and feature modules remain compiled: Home Workspace, Model Hub, editor, change history, checks, terminal, Git, AI Workspace, monitoring, media inspection, Gmail, model trial.
- `AppView` still includes `video` as a compatibility placeholder (not an active Video feature); `AppSidebar` icon union still includes `team` and `video`. These are cleanup candidates after confirming references.
- `ProductionPanel` is actually system/runtime settings, update controls and history retention; do not mistake it for removed Video Production.
- Next: audit actual reachable UI versus dormant components, check backend command registration and module dependencies, then propose a focused removal/refactor branch. No source files changed in this inventory stage.

## Gateway-only frontend mount (awaiting validation)
- User confirmed tray Projects/Connections shortcuts work, along with close-to-tray and Quit.
- Commits `99a504f`, `d04ba65`: `src/App.tsx` now mounts only Projects and Connections page bodies; removed the ProjectRail and WorkspaceProductivity from the shell, removed HomeWorkspace and WorkspaceEditor render branches, and removed imports of the development panels so their heavy runtime modules should no longer bundle. Retained existing internal hooks/state and type-only editor imports for a safer first step; dead state cleanup comes later. Error recovery now routes to Projects.
- Validate `git pull --ff-only && npm run check && npm run build` and inspect Vite output for disappearance of `editor-core` bundle. Then test both pages and tray navigation in `npm run tauri dev` before further deletion.
- Important: no Rust MCP tool registry or permissions changed. Do not assume removed UI panels imply removed MCP capabilities.

## Tray-first navigation stage (validated)
- Commits `825a0d9`, `4cb7cbf`: tray menu now opens Projects or Connections directly, retaining Open and Quit; Rust emits `razedtunnel:open-page` after showing window, React listens and switches the active page. No backend MCP changes.
- Validate `npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`, then `npm run tauri dev` and test each tray menu entry, close-to-tray, and Quit. Unset `REPOTUNNEL_WINDOWS_TEST_MANIFEST` before launching normally.
- Next: replace legacy App.tsx composition and eliminate editor/home imports and compiled chunks, preserving the two required pages and gateway state.

## Validated milestone: Windows tray (2026-10-12)
- User confirmed `npm run tauri dev` launches and window close-to-tray and explicit Quit both work correctly. Tray milestone complete.
- Previous link error CVT1100/LNK1123 was caused by duplicate Windows manifest while `REPOTUNNEL_WINDOWS_TEST_MANIFEST` remained set; clearing that session variable allowed launch. Keep the variable unset for normal app builds.
- Next stage: remove legacy frontend routes/components and their imports rather than only hiding navigation; preserve Projects, Connections, MCP transport, permission enforcement, persistence, and tray. Audit reachability before deletion and validate incrementally.

## Current stage: background system tray (validated)
- User confirmed `npm run check` and `npm run build` passed, but Rust compilation failed with E0432/E0433: `tauri::tray` gated by `tray-icon` feature. Fixed `src-tauri/Cargo.toml` in commit `0eefd78`. Re-run Rust check and tray runtime test. Cargo.lock is locally modified; do not overwrite it.
- Commits `6095ed0`, `57725ef`: new `src-tauri/src/system_tray.rs` with Open RazedTunnel / Quit RazedTunnel menu and double-click-to-open; close of main window is intercepted and hides it, preserving gateway runtime. Explicit Quit invokes app exit and existing shutdown path.
- No `Cargo.toml` or lockfile changes; existing default Tauri tray API used. User must validate Rust compilation and manually verify close-to-tray, reopening, Quit, and connection persistence.
- Validation: `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. Run app afterward to test tray. Preserve user's local modifications in Cargo.lock and github.rs.

## Locked product scope (2026-10-12)
- Two-page UI first pass: `8589a33` defaults launch to Projects; `d13d5b1` removes Settings, Help, and theme controls from sidebar, leaving Projects and Connections; `cdfbbe8` retains the existing sidebar props contract to avoid caller breakage. Backend, MCP tools, and non-navigation views are not yet removed.
- Next: user validates `npm run check && npm run build`; then implement tray behavior as a separate patch and continue dependency-based pruning.
- User explicitly requires only: (1) add/manage projects and permissions, (2) connect MCP to ChatGPT, (3) background system tray with open/quit.
- If functionality is not on Projects or Connections and not a dependency of those pages or tray, it is out of scope. Retain security/auth, persistence, runtime lifecycle, and required MCP tool handlers as dependencies, not separate UI features.
- Current `src-tauri/src/lib.rs` setup/run handles runtime startup/shutdown but no system tray setup is visible there; implement and validate tray behavior explicitly, including close-to-tray and true Quit.
- Strategy: dependency-first reduction toward a two-page shell; do not keep Model Hub, Home generation, editor, Git UI, terminal UI, AI Workspace UI, etc. merely for compatibility. Avoid wholesale deletion until dependencies are traced. Next patch should establish minimal navigation and tray foundation, with small validated commits.

## Current stage: dead navigation cleanup
- Branch `cleanup/dead-video-view` removes retired Video view routing, header mapping, and obsolete Team icon type. Commits: `8bbb7dd`, `b1b9d4d`, `26737af`.
- User reported TypeScript TS2367 on the last `activeView === "video"` comparison in `src/App.tsx`; fixed in `4781f3c`. No backend changes. Next: validate `npm run check && npm run build`, then inspect remaining obsolete UI references. Do not overwrite local Rust edits.

## Goal and boundaries
RazedTunnel is a compact Windows/Tauri Rust MCP gateway for approved local workspaces, files, terminal, Git/GitHub, and related development tools. Keep modules focused and remove legacy RepoTunnel features that do not serve the gateway. Do not break ordinary workspace permissions, MCP access, or file operations. Prefer small commits and local validation; do not merge before user approval/validation.

## Completed and merged
- Legacy video cleanup merged to `main` at `70ea13cd605dacf735f81a5188037443ec70be17`.
- Legacy Android/phone cleanup merged via PR #1 at `beed3fb835dc0f8daf19064139aac9ce446799d3`. User validated frontend, Rust check, and Rust library tests (295 passed, 2 ignored).
- Windows Rust library tests may require `$env:REPOTUNNEL_WINDOWS_TEST_MANIFEST = "1"` to avoid the Windows `TaskDialogIndirect` test-runner issue.

## Completed: Team Mode removal (merged via PR #2 at `d8a71ee7fda7eeefdfc6eeb77bd4689ea37c3bae`)

## Historical validation: remove Team Mode
Branch: `cleanup/remove-team-mode`, merged into `main` on 2026-10-12.
Validated by user:
1. Removed Team Mode UI routing/navigation/header/command-palette entries. Frontend `npm run check && npm run build` passed after fixing remaining references.
2. Removed desktop `team_commands` registrations and deleted `src-tauri/src/team_commands.rs`. `cargo check` passed.
3. Removed `TeamPanel.tsx`, Team frontend API wrappers in `src/lib/backend.ts`, and Team TS models in `src/types.ts`. Frontend check/build passed.
4. Removed MCP `team_status` and `team_action` tools and their parameter models from `src-tauri/src/mcp_server.rs` (commit `994f016d7ea5e0b996cef7201b189e02d35d47c3`). **User confirmed `cargo check` passed**.

## Final result
- PR #2 merged to `main` at `d8a71ee7fda7eeefdfc6eeb77bd4689ea37c3bae`. User confirmed `npm run check`, `npm run build`, `cargo check`, and Rust library tests (291 passed, 0 failed, 2 ignored). Remaining historical legacy documentation should be addressed in a future docs-only cleanup.

## Last branch patch — validated
- Commit `c47a117`: removed obsolete Team Mode feature table row, feature section, and docs navigation link from `README.md`. The implementation's Rust models/activity and primary frontend files inspected have no Team Mode references. Historical `docs/team-mode.md` and other legacy documentation may still need a separate documentation cleanup. **User reports 291 Rust library tests passed, 0 failed, 2 ignored after this stage. User subsequently confirmed frontend check, frontend build, and Rust check all passed.**

## Prior MCP cleanup — validated
- Commit `917045b`: cleaned unused imports and Team-related `client_key`/destination locals in `mcp_server.rs`; removed obsolete Team Mode wording from external-file tool description. **User confirmed `cargo check` passed.**

## Previous engine patch — validated
- Commits `dee7050`, `4a350e9`, `09e917a`: removed `mod team;`, deleted 2,649-line `src-tauri/src/team.rs`, and removed the trailing Team-specific Rust session models in `src-tauri/src/models.rs` (~212 lines). **Initial `cargo check` failed on three leftover imports/calls. Fixed in commits `f2de9c0`, `cc0b752`, `f2123c7`; user subsequently reported 291 Rust tests passed, 0 failed, 2 ignored (2026-10-12).**

## Earlier browser patch — validated
- Commit `aad66688bda47b5c2a90b5fe5d9fafa835a80832`: removed eight `team::assert_browser_mutation_available` checks from MCP browser mutation handlers. Independent approved workspace scope, tab access, Google/Gmail access, and AI browser ownership checks remain in place. **User confirmed `cargo check` passed.**

## Prior patch — validated
- Commit `5ee379185cec878a0a869f6044b7640e2e46505f`: removed eight `team::assert_paths_available` calls from MCP filesystem mutations, leaving workspace approval and `changes::*` mutation handling in place. **User confirmed `cargo check` passed.** Possible unused `client_key` locals and rename destination calculation need cleanup after compilation; do not mistake warnings for test failures.

## Next steps — priority order
1. Team Mode cleanup merged. Next prioritize a separate audit of outdated documentation (Team Mode, Android, Video), without changing already validated core functionality.  Review remaining legacy docs references and cleanup separately if needed. Then create PR and merge only after user approval. Audit remaining Team-only activity variants, labels, UI icon strings, docs, and tests; remove only if no other dependencies. `src-tauri/src/team.rs` (~2649 lines) is still used for `team::assert_paths_available` in filesystem MCP operations and `team::assert_browser_mutation_available` in browser MCP operations. These checks may represent coordination locks rather than core workspace authorization. Inspect exact call sites and preserve independent approval/access checks.
2. Remove Team-only coordination hooks carefully, with targeted Rust compilation and tests after each patch. Do not replace Team locks with unconditional access without confirming the existing workspace authorization path.
3. Delete `team.rs` and `mod team;` in `src-tauri/src/lib.rs` only after all references are gone.
4. Remove Team-specific Rust data models (near end of `src-tauri/src/models.rs`), stale activity enums/labels, docs and remaining dead code after verifying usages. Watch for tests and imports.
5. Run `npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`, then `$env:REPOTUNNEL_WINDOWS_TEST_MANIFEST = "1"; cargo test --manifest-path src-tauri/Cargo.toml --lib`. Review diff, create PR, merge after user validates.

## Local environment and workflow
- Repo: `razed-developer/razedtunnel`; Windows PowerShell 7, local path `D:\Dev\.Dev\razedtunnel`.
- Assistant edits and pushes commits using connected GitHub tools. User pulls and tests locally, then responds `passed, proceed` or shares errors. Do **not** overwrite the user's unrelated local modifications: `src-tauri/Cargo.lock` and `src-tauri/src/github.rs` were modified locally when the branch was checked out.
- Standard local check: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml` (Rust) or `git pull --ff-only && npm run check && npm run build` (frontend).
- Favor fail-fast `&&` command chains. Commit and push in small, reversible stages. Keep `NEXT.md` current with validated outcomes and exact next action.

## Longer-term direction
- Keep RazedTunnel focused on MCP local-system access; ChatGPT conversation archival/project association and cross-app conversation/asset access are being considered for a separate ASK/archive layer and Umbrella integration. Do not conflate those with legacy Team Mode.
