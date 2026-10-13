## UI THEME HANDOFF — 2026-10-13
- On `ui/dashboard-restoration`: added System / Light / Dark appearance selector in header, saved to localStorage under `razedtunnel.theme`, reacts to OS color-scheme changes in System mode.
- New `src/theme.css` loads after existing CSS and overrides the dashboard, shell, navigation, title bar, and common legacy surfaces for light mode. Dark mode preserves prior look. This is an incremental migration; some deep legacy Projects/Connections widgets may still have hard-coded dark colors and require further inspection.
- Validate: `git pull --ff-only && npm run build`; launch app, test all three modes and OS appearance change, reload persistence, and check Projects/Connections readability. Do not claim build pass until local result is returned.
- Next: fix any light-mode contrast defects found locally, then connect actual activity and approval data without placeholder counts.

## UI RESTORATION HANDOFF — 2026-10-13
- UI work is isolated on branch `ui/dashboard-restoration`, based on `cleanup/dead-video-view`. Do not merge the Rust cleanup branch as part of UI validation.
- First UI slice: added Home navigation and landing page with actual gateway status, remote tunnel/chat readiness, project count and permissions, MCP tool success/failure totals. No fabricated activity or approvals. Projects and Connections setup flows unchanged.
- Validate locally: `git fetch origin && git switch ui/dashboard-restoration && npm run build`. Run the desktop app and inspect Home, Projects, Connections; check narrower window behavior.
- Next UI slice after validation: theme system (light/dark/system), then real activity/approval data adapters and tray-friendly compact view. Existing `src/workspace.css` is very large; keep new CSS scoped and plan extraction, do not pile all future styles into it.
- User's visual reference: clean rounded-card dashboard, light and dark, monitoring-first, tray-first, few controls after setup.

## CURRENT HANDOFF — 2026-10-13 — Pause backend minimization; prioritize UI
- User wants to take a break from minimization and restore/redesign the UI, which now feels like a shell of original RepoTunnel. Do not continue dead-code deletion after this patch without discussing UI priorities.
- Removed disconnected FFmpeg provisioning cluster in commit 16fc130; preserved managed-data-dir-first/PATH discovery of existing FFmpeg and FFprobe, including executable naming and Unix permissions.
- Validate: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services && node scripts/audit-rust-functions.mjs`.
- Next: compare current frontend against original RepoTunnel UX, identify missing navigation, panels, status surfaces and flows, and make a UI restoration plan before editing. Keep backend cleanup branch unmerged; consider a separate UI branch from the desired baseline after validation.

## CURRENT HANDOFF — 2026-10-13 — FFmpeg wrapper
- User validated prior activity cleanup: default cargo check PASS with 132 warnings; legacy PASS with 93 warnings; audit 1,818 declarations, 20 review candidates, 310 protected.
- Inspected media_tools.rs and media_inspection.rs. Active media inspection only calls available_ffmpeg_program and available_ffprobe_program, and explicitly does not auto-install FFmpeg. Removed unreferenced ensure_ffmpeg_program wrapper in commit 2ee8edd; retained installer and security helpers pending wider dependency verification, and retained all active discovery.
- Validate: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services && node scripts/audit-rust-functions.mjs`. Expected 19 review candidates and 310 protected. Do not merge; preserve local changes.

## CURRENT HANDOFF — 2026-10-13 — activity updater cleanup
- User validated previous removal of three unused activity sync wrappers: default Cargo PASS (134 warnings), legacy PASS (95 warnings), audit 1,819 declarations / 21 review candidates / 310 protected.
- Removed now-orphaned `activity::update_source` and unused `ChangeRecord`/`CommandRecord` imports in `ddf32c8`. Kept active recorders, continuity, process reconciliation, and retention logic.
- Validate: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services && node scripts/audit-rust-functions.mjs`. Expect 20 review candidates; warning counts likely decrease by 2 each (unused imports and dead function). Do not merge; preserve local Cargo.lock, github.rs and audit files.

## CURRENT HANDOFF — 2026-10-13 — activity sync wrappers
- User validated previous launch/browser wrapper removal: default Cargo check PASS (136 warnings), legacy feature PASS (97 warnings), audit 1,822 declarations / 23 review candidates / 310 protected.
- Removed unused `activity::sync_change`, `sync_sandbox_command`, and `sync_terminal` in commit `a8ee1e4`. These called `update_source` only. Kept `update_source` itself pending separate evidence-based cleanup, along with active recorders, `sync_process(es)`, and helpers. Imports unchanged pending compiler validation.
- Run `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services && node scripts/audit-rust-functions.mjs`. Expect 20 review candidates. Do not merge; preserve local `Cargo.lock`, `github.rs`, and audit output files.

## CURRENT HANDOFF — 2026-10-13 — unused activity launch/browser sync wrappers
- User supplied local validation output after removing `activity::sync_git`: default Cargo check PASS with 138 warnings, legacy-feature Cargo check PASS with 99 warnings, audit 1,824 declarations / 335 zero refs / 25 review candidates / 310 protected.
- Inspected `activity.rs` and removed only unused `sync_launch` and `sync_browser` wrappers (commit `5ee6cad`). Both only delegated to unused `update_source`. Preserved active `record_launch_record`, `record_browser_record`, `sync_process(es)`, activity history persistence, and tests. Imports not touched to avoid removing shared types inadvertently.
- Awaiting local validation: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services && node scripts/audit-rust-functions.mjs`. Expect audit 23 review candidates; inspect any unused import warnings. Preserve local `github.rs` / `Cargo.lock`; DO NOT MERGE.

## CURRENT HANDOFF — 2026-10-13 — remove unused activity git sync wrapper
- User pulled prior docs-only FFmpeg audit. Inspected `activity.rs` (960 lines). `sync_git` is a compiler-confirmed unused wrapper calling `update_source`; removed only `sync_git` in commit `32a090f`. Preserved `record_git_record` (active recording), `update_source`, other activity reconciliation paths, and tests. No unrelated files changed.
- **Awaiting local validation**: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services && node scripts/audit-rust-functions.mjs`. Expect review candidates to drop from 26 to 25, but compiler warnings should be checked rather than assumed.
- Preserve local `github.rs` and `Cargo.lock`; DO NOT MERGE. Continue with similarly narrow compiler-confirmed unused wrappers only after checks pass.

## CURRENT HANDOFF — 2026-10-13 — FFmpeg provisioning dependency map
- User confirmed audit passes: 1,825 declarations, 336 zero textual refs, 26 review candidates, 310 protected. No runtime code change in this step.
- Inspected `media_tools.rs` and `media_inspection.rs`. Documented the active discovery path and disconnected provisioning cluster in `docs/audits/media-provisioning-dependencies.md` (commit `b558c6b`). Media inspection calls `available_ffmpeg_program` / `available_ffprobe_program`; retain these. `ensure_ffmpeg_program` is unreferenced, but the rest of the provisioning chain contains download integrity and safety controls and should be considered as one feature, not isolated dead helpers.
- Next: inspect remaining function candidates or decide whether managed FFmpeg provisioning is intentionally retained. Validate docs-only patch with `git pull --ff-only`. Preserve local `github.rs` and `Cargo.lock`; DO NOT MERGE.

## CURRENT HANDOFF — 2026-10-13 — fix audit JS syntax regression
- Previous trait classification patch accidentally inserted literal `\\n` into a JavaScript `//` comment, commenting out the function's return and causing `Object.values(undefined)` at line 56. User reported the failure after a successful pull.
- Fixed actual newlines, corrected regex escapes for `impl Deref for BrowserScope`, and included `isTraitImpl` in protected-flag output; commit `a9d8f80`. No runtime Rust code changed. **Awaiting user validation**.
- Next: `git pull --ff-only && node scripts/audit-rust-functions.mjs`; expected 26 review candidates and 310 protected. No Cargo check needed. Preserve local `github.rs` and `Cargo.lock`; DO NOT MERGE.

## CURRENT HANDOFF — 2026-10-13 — trait dispatch audit
- User recovered local `github.rs` change using file-scoped stash/pull/pop and reports both Cargo checks PASS: 139 default warnings, 100 legacy warnings. Function audit: 1,825 declarations, 336 zero textual refs, 27 review candidates, 309 protected.
- Confirmed `browser::deref` is `impl Deref for BrowserScope`, so it is trait-dispatched rather than dead. Audit now protects that exact implementation via `isTraitImpl`, commit `b6ac0c9` (after superseding a too-broad heuristic in `49f59bc`). No runtime Rust code changed. **Not yet locally validated.**
- Next: `git pull --ff-only && node scripts/audit-rust-functions.mjs`; expect review candidates 26 and protected 310 if detection matches. If local github.rs conflicts again, stop and preserve local changes; do not reset. Continue inspecting compiler warnings and avoid removing feature-gated functionality wholesale. DO NOT MERGE.

## CURRENT HANDOFF — 2026-10-13 — unused GitHub cache accessor
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User's latest audit: 1,826 declarations, 337 zero textual references, 28 review candidates, 309 protected; `workspace_policy_commands::check_workspace_access` correctly flagged as unregistered Tauri command.
- Reviewed `github::cached_status`: compiler warns unused, textual audit confirms zero references. Removed only its 3-line accessor, commit `8a97b55`. Kept `status_cache`, `cache_status`, GitHub CLI authentication, connection lifecycle and status handling intact. **Not locally validated yet.**
- Noted `browser::deref` is a `Deref` trait method; it must not be deleted as an ordinary orphan even if no textual references. Next audit improvement could protect trait implementations rather than relying on generic name matching.
- Next validation: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services && node scripts/audit-rust-functions.mjs`. Continue small, compiler-confirmed removals; preserve active MCP/gateway/security/tunnels/terminal/browser/Projects/Connections/tray.

## CURRENT HANDOFF — 2026-10-13 — unregistered Tauri command audit
- User's checks PASSED after removing unused MCP `success_result`: default Cargo 140 warnings, legacy feature 101 warnings. Rust audit: 1,826 declarations, 337 zero textual references, 29 review candidates, 308 protected entry points.
- Inspected review candidate `workspace_policy_commands::check_workspace_access`: it has `#[tauri::command]` but is not in the registered `generate_handler!` list. Treat it as an unregistered Tauri endpoint, not a routine dead helper; no runtime code removed.
- Updated `scripts/audit-rust-functions.mjs` to classify `isUnregisteredTauri` separately from registered `isTauri`, and to protect both from automatic review-candidate labeling. Commit `5cbdf5f`. **Not yet run locally.**
- Next: `git pull --ff-only && node scripts/audit-rust-functions.mjs`. Then manually inspect other high-confidence compiler-confirmed unused functions. Preserve activity/history semantics, update handling, GitHub authentication, MCP and connection/security surfaces unless clearly disconnected and intentionally retired. DO NOT MERGE branch.

## CURRENT HANDOFF — 2026-10-13 — compiler-confirmed unused MCP wrapper
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed default and legacy-feature Cargo checks PASS after removal of five unregistered updater IPC wrappers. Rust audit now: 1,827 declarations, 338 no textual references, 308 protected, 30 review candidates. Default check produced 141 warnings; legacy feature 102 warnings. Warnings are useful evidence but don't automatically authorize broad removal.
- Inspected `mcp_server.rs`: `success_result<T: Serialize>` was flagged unused by compiler and textual audit. Removed **only** that wrapper, commit `2f4c12f`. Kept `serialize_success_payload`, `error_result`, MCP tool handlers, result schemas and other response helpers unchanged. **Not yet locally compiled.**
- Next validate: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services && node scripts/audit-rust-functions.mjs`. Continue in small targeted patches, prioritizing compiler-confirmed unused leaf helpers. Do not delete updates.rs wholesale or security/gateway/tunnels/terminal/browser/Projects/Connections/MCP functionality.

## CURRENT HANDOFF — 2026-10-13 — remove unregistered updater IPC wrappers
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User audit: 1,832 declarations, 341 zero textual refs, 308 protected, 33 review candidates.
- Checked `lib.rs` Tauri handler: only `runtime_commands::get_runtime_diagnostics` and `set_launch_at_login` are registered; the five `runtime_commands.rs` update wrappers are unregistered. Removed only `get_update_status`, `check_for_updates`, `set_auto_update_checks`, `defer_update`, `install_update_and_restart` and now-unused `State`/`AppState`/`updates` imports, commit `9710ed9`. **Retained entire `updates.rs` implementation** and all active registered Tauri commands. No other candidate deletions.
- **Not yet locally validated.** Next: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services && node scripts/audit-rust-functions.mjs`. Expect declarations down by 5 and candidates down by 5 if no other changes.
- The 0 `isTauri` among zero-textual-reference functions is expected: registered commands have literal `lib.rs` references. Avoid deleting trait methods, callbacks, or MCP entry points based on text matching.

## CURRENT HANDOFF — 2026-10-13 — function audit classification under refinement
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User audit output: 1,832 functions, 341 zero-text references, 303 protected, 38 review candidates. **Do not delete these candidates**: list includes MCP `ServerHandler` trait hooks, Tauri `#[tauri::command]` functions, and other possible macro/trait dispatch. The 0 protected Tauri count is not proof of no commands: only 2 runtime commands currently registered, while several retired update commands are unregistered.
- Updated `scripts/audit-rust-functions.mjs` commits `b67cbea`, `32652d1`: attempted immediate-attribute classification instead of broad proximity; explicitly protects MCP ServerHandler lifecycle/resource methods. **Not locally validated.** IMPORTANT: current attribute scanner may fail on multi-line `#[tool(...)]` attributes, and the Tauri handler scanner should be checked against actual registered commands. Fix before interpreting counts as authoritative. No production code changed.
- Next run `git pull --ff-only && node scripts/audit-rust-functions.mjs`, share counts/candidates. Prioritize improving multiline attribute parsing and macro/trait exposure, then inspect any remaining specific candidates. Preserve active MCP/gateway/security/tunnels/Projects/Connections/tray.

## CURRENT HANDOFF — 2026-10-13 — function audit candidate diagnostics
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User ran function audit: 1,832 declarations; 341 no textual references; 303 protected by heuristic; 38 review candidates. `load_history_settings` is still called by activity/updates/versioning, keep it.
- Added console diagnostics to `scripts/audit-rust-functions.mjs`, commit `c2d97f3`: prints all review candidate file:line/function names, counts of zero-reference functions protected by each flag, and first 8 examples per flag. No production code removed. Attribute-proximity heuristic may falsely protect adjacent methods; review output before deleting anything. **Not yet locally run.**
- Next validation: `git pull --ff-only && node scripts/audit-rust-functions.mjs`. Use output to inspect specific candidates and improve protection classification, particularly MCP attribute association, traits and callbacks. Preserve active MCP/gateway/security/tunnels/Projects/Connections/tray; do not merge.

## CURRENT HANDOFF — 2026-10-13 — Rust function audit entry-point classification
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed cargo checks PASSED after removing unused `save_history_settings`.
- Updated `scripts/audit-rust-functions.mjs` commits `4fcffb6`, `f20043b` to distinguish no-textual-reference counts from review candidates; suppress likely MCP `#[tool]` functions, Tauri registered `#[tauri::command]` functions, and tests from deletion candidates. Heuristic attribute proximity; method/trait/callback classification **not** solved. No production code changed. Not locally validated.
- Next run: `git pull --ff-only && node scripts/audit-rust-functions.mjs`. Compare `noTextualReferences`, `protectedEntryPoints`, `reviewCandidates` in printed counts and JSON; investigate a few candidate functions manually before deletion. Existing history settings loader remains active.

## CURRENT HANDOFF — 2026-10-13 — unused history-settings writer removed
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User ran function-reference audit: 1,833 declarations, 342 no textual references. `load_history_settings` is used in `activity.rs`, `updates.rs`, `versioning.rs`; `history_settings_path` has two internal references. Preserve these and `HistorySettings`/existing on-disk settings. `save_history_settings` had zero textual references outside its declaration; removed only that unused function in `storage.rs`, commit `a697e0b`. No persistence files modified; historical settings loading remains intact.
- **NOT YET COMPILED.** Validate `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services`.
- Next: improve function audit classification (MCP tools, Tauri commands, trait implementations, callbacks) before pruning more; avoid treating all 342 as dead. Preserve active MCP/gateway/security/tunnels/Projects/Connections/tray and unrelated local edits.

## CURRENT HANDOFF — 2026-10-13 — Rust function reference audit
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed both cargo checks PASSED after checkpoint response-model removal.
- Inspected `models.rs` and `storage.rs`: `HistorySettings` and `load_history_settings`/`save_history_settings` remain. Avoid deleting until cross-file callers are verified. Added `scripts/audit-rust-functions.mjs` commit `aa53e7e`, a read-only inventory of Rust function declarations and textual references across top-level `.rs` files. Outputs `docs/audits/rust-function-references.json`, prints counts and history-setting symbol callers. It is heuristic only; never remove code based solely on zero references. **Not yet locally validated.**
- Next validation: `git pull --ff-only && node scripts/audit-rust-functions.mjs`. Inspect history-setting results and prioritize conservative dead-function candidates in non-MCP modules, guarding macros, startup hooks, traits, feature gates, and tests. Active MCP/gateway/security/tunnels/Projects/Connections/tray preserved.

## CURRENT HANDOFF — 2026-10-13 — checkpoint-only response models pruned
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed both default and `legacy_background_services` cargo checks PASSED after removal of `checkpoint.rs`.
- Removed four legacy checkpoint response types from `src-tauri/src/models.rs` in commit `b9f99f7`: `CheckpointSummary`, `CheckpointComparison`, `CheckpointRestoreResult`, `CheckpointClearResult`. These were owned by retired one-click checkpoint operations. Kept `HistorySettings` and `storage.rs` history load/save routines intact pending independent caller/compatibility analysis; no saved user data modified.
- **NOT YET VALIDATED:** run `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services`. If unexpected references arise, restore models and inspect.
- Next: inspect history settings callers and compatibility, refine graph to function-level roots; preserve active MCP, gateway, security, tunnels, Projects/Connections/tray and unrelated local edits.

## CURRENT HANDOFF — 2026-10-13 — retired checkpoint module removed, validation pending
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User ran Rust module graph: 75 declared modules, 19 direct lib roots, 73 visited, only `checkpoint` review candidate (no qualified references). Inspected `checkpoint.rs` (~18.9 KB): legacy one-click checkpoint create/list/compare/restore/delete/retention API. Only `mod checkpoint;` in `lib.rs`; no current frontend command registration or MCP tool exposing this API. Removed declaration commit `abbd3ee` and file commit `7a3b2f1`. **NOT YET COMPILED; if cargo check reveals unqualified imports/callers, restore module immediately rather than weakening functionality.**
- User validation: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml && cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services`. Checkpoint data on disk was not touched. The separate active continuation checkpoint system remains in `self_continuation` / `chatgpt_bridge`; legacy one-click workspace checkpoint implementation removed. Historical checkpoint types/settings in `models.rs`/`storage.rs` not yet pruned; inspect separately before removal.
- Next: inspect any remaining orphan-related model structs and history settings, then improve audit precision before additional removals. Preserve active MCP/gateway/security/tunnels/Projects/Connections/tray and unrelated local Cargo.lock/github.rs changes.

## CURRENT HANDOFF — 2026-10-13 — Rust dependency graph audit added
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User ran command graph: 26 registered Tauri, 26 literal frontend references, zero unreferenced Tauri, zero dynamic invokes, 125 MCP tool candidates. No Tauri IPC orphans identified by this audit. Rust warnings reported at 174.
- Added read-only `scripts/audit-rust-dependency-graph.mjs` commit `9e772e3`. Builds conservative qualified-reference module edges, marks roots directly referenced in `lib.rs`, traverses those edges, outputs `docs/audits/rust-module-graph.json` and `.mmd`. Heuristic-only: grouped imports, callbacks, macros, trait dispatch, feature flags and other entry points may produce false orphan candidates; **do not delete modules based on this alone**. Not locally run yet.
- Next user validation: `git pull --ff-only && node scripts/audit-rust-dependency-graph.mjs`; share candidate output. Next deepen targeted reachability analysis for candidates and function-level edges, with active MCP tools treated as roots.

## CURRENT HANDOFF — 2026-10-13 — command-level Mermaid graph audit
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed prior MCP session ID refactor cargo check PASSED; reports Rust warnings down to 174.
- Added read-only `scripts/audit-command-graph.mjs` commit `ac295b2`. Parses literal frontend Tauri `invoke()` calls via TypeScript AST, Tauri `generate_handler!` registrations, and candidate `#[tool]` methods in `mcp_server.rs`. Writes `docs/audits/command-graph.json` and `docs/audits/command-graph.mmd` locally, with review-only flags for registered commands lacking static frontend invokes, plus dynamic/unregistered calls. **Not yet locally validated.** Run `git pull --ff-only && node scripts/audit-command-graph.mjs`; share totals and suspects. Do not delete anything from this static audit alone; dynamic calls, aliases, macros, feature flags, and startup hooks require manual inspection.
- Next: validate output, then improve graph with module/function-level Rust call edges and confirmed entrypoints before selecting removal candidates. Preserve active MCP/gateway/security/tunnels/Projects/Connections/tray and unrelated local Cargo.lock/github.rs changes.

## CURRENT HANDOFF — 2026-10-13 — MCP AI Workspace session ID helper extracted
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed previous `mcp_task_metrics.rs` extraction PASSED cargo check.
- Added `src-tauri/src/mcp_app_session_ids.rs` commit `65fb1c9`: isolated the AtomicU64 sequence and ID generator for AI Workspace app session reservations. Replaced the one call in `mcp_server.rs` with `crate::mcp_app_session_ids::next_app_session_id()` commit `f82214c`, registered module in `lib.rs` commit `d62535a`. Format `aiwapp-mcp-{pid}-{unix_ms}-{sequence}` and Relaxed atomic ordering unchanged; no MCP tool names or permissions changed.
- **NOT YET VALIDATED.** Run `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml`. Next prioritize larger cohesive helpers such as self-continuation resource metadata/diagnostics, but preserve counter semantics and avoid macro router changes. Active MCP/gateway/security/tunnels/Projects/Connections/tray preserved; avoid unrelated local Cargo.lock/github.rs changes.

## CURRENT HANDOFF — 2026-10-13 — MCP task outcome metrics extraction
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed previous extraction of `SERVER_INSTRUCTIONS` to resource PASSED cargo check.
- Added focused `src-tauri/src/mcp_task_metrics.rs` commit `7cda772` with two AtomicU64 counters, `task_tool_outcomes()`, `record_task_outcome(bool)`. In `mcp_server.rs`, replaced old statics/recorder with `pub(crate) use crate::mcp_task_metrics::task_tool_outcomes;` and imported recorder commit `cf2f1a6`; registered `mod mcp_task_metrics;` in `lib.rs` commit `9be2c62`. Same counter behavior and external `mcp_server::task_tool_outcomes()` path retained. These metrics cover only the two task wrappers, not all MCP tools.
- **NOT YET VALIDATED.** Run `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml`; optionally feature check after. No MCP tool routing/permissions changed. Next extract a cohesive small helper with minimal dependencies; avoid moving `#[tool_router]` macro methods blindly. Preserve active MCP/gateway/security/tunnels/Projects/Connections/tray; avoid unrelated local Cargo.lock/github.rs changes.

## CURRENT HANDOFF — 2026-10-13 — MCP server instructions extracted
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed frontend tests/build and Rust default + legacy_background_services checks PASSED after retiring Home AI subsystem.
- Began conservative refactor of oversized `mcp_server.rs` (~291 KB). Moved its 22,614-character one-line `SERVER_INSTRUCTIONS` string verbatim to `src-tauri/resources/mcp_server_instructions.txt` commit `5c86a00`, and changed constant to `include_str!("../resources/mcp_server_instructions.txt")` commit `5be91ef`. Verified resource has no trailing newline and original was an unescaped Rust string literal, so text should be identical. No MCP tools, names, signatures, security logic, or handlers intentionally changed. **NOT YET LOCALLY VALIDATED.** Run `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml`; optionally check feature build again. Next: extract small, cohesive MCP helpers after checking dependencies, not large macro-generated tool impls blindly.
- Preserve active MCP/gateway/security/tunnels/Projects/Connections/tray; avoid unrelated local Cargo.lock/github.rs edits.

## CURRENT HANDOFF — 2026-10-13 — legacy Home AI Rust subsystem retired; VALIDATION REQUIRED
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed prior frontend removal of ~198 legacy Home AI TypeScript type lines PASSED.
- After Rust and frontend reference audits, retired legacy Home AI group: removed `mod conversation`, `mod project_context`, `mod model_trial`, `mod model_hub` and legacy `model_hub::stop_owned_local_runtimes()` shutdown hook from `src-tauri/src/lib.rs` commit `d770fde`. Removed Home AI/model-trial-only activity checks and imports from `updates.rs` commit `0da6ebe`; remaining updater guards unchanged. Deleted `conversation.rs` `4fb014d`, `project_context.rs` `990459e`, `model_trial.rs` `17f3f1a`, `model_hub.rs` `760b055`. Total ~190 KB. No mounted desktop Tauri commands or MCP direct calls found for these modules; no active entrypoint to launch their background jobs identified. This is a behavioral retirement, not a mechanical dead-file removal.
- **NOT YET VALIDATED.** Run `git pull --ff-only && npm run check && npm run test:frontend && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. Also strongly recommend `cargo test --manifest-path src-tauri/Cargo.toml --lib` and `cargo check --manifest-path src-tauri/Cargo.toml --features legacy_background_services` to catch optional-feature imports. If failures occur, fix actual cross-module dependencies, preserve active protections; do not silently weaken security. Check update safety behavior and gateway startup.
- Earlier `docs/LEGACY_HOME_AI_AUDIT.md` describes pre-retirement state; treat as historical. Preserve active MCP, security, gateway, tunnels, Projects/Connections/tray. Avoid unrelated local Cargo.lock/github.rs changes.

## CURRENT HANDOFF — 2026-10-13 — legacy Home AI TypeScript types removed
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User ran `node scripts/audit-legacy-home-ai.mjs` and shared output: all legacy Home AI/local model TS type references occurred within `src/types.ts`; Rust module references were in the legacy subsystem and updater/lib lifecycle. This supports removing the TS-only declarations, not yet the Rust services.
- Commit `8091372` removes the contiguous legacy model provider/capabilities/selection/runtime/trial/Home context/conversation TS type block from `src/types.ts` (~198 lines), retaining `AccessCheck` and following types. **Not locally validated yet.** Run `git pull --ff-only && npm run check && npm run test:frontend && npm run build` (check package scripts if missing).
- Preserve Rust `updates.rs` safety checks `conversation::has_active_generation()`, `model_trial::is_running()` and `lib.rs` owned Ollama shutdown hook until coordinated retirement design; no Rust changes in this batch. Active MCP/gateway/security/tunnels/Projects/Connections/tray untouched.
- After validation, consider evaluating Rust Home AI retirement as a deliberate feature removal, not piecemeal dead_code pruning. Preserve unrelated local Cargo.lock/github.rs changes.

## CURRENT HANDOFF — 2026-10-13 — cross-language Home AI reference inventory
- Branch `cleanup/dead-video-view`; DO NOT MERGE. Added read-only `scripts/audit-legacy-home-ai.mjs` commit `df10931`. It scans `src/` and `src-tauri/src/` for selected Home AI Rust modules, legacy TS types, and feature flag references; prints file:line and names. **Not yet validated locally**. Run `git pull --ff-only && node scripts/audit-legacy-home-ai.mjs`, paste output for targeted cleanup. No production code changed; no Cargo build needed.
- Inspected `src/App.tsx` and `src/lib/backend.ts`: no current Home AI invoke paths. `src/types.ts` still declares Home context/conversation and model-trial types. `src-tauri/src/lib.rs` has no registered Home AI commands, but retains `model_hub::stop_owned_local_runtimes()` and feature-gated legacy initialization. `updates.rs` still guards active Home responses and model trials. Preserve safety guards until coordinated retirement is proven safe. The current audit is not proof of full reachability.
- Previous audit `docs/LEGACY_HOME_AI_AUDIT.md` commit `3c560e5`; earlier Rust inventory `scripts/audit-rust-modules.mjs`. Preserve active MCP, security, gateway, tunnels, Projects/Connections/tray; avoid unrelated local Cargo.lock/github.rs edits.

## CURRENT HANDOFF — 2026-10-13 — legacy Home AI dependency audit
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User ran `node scripts/audit-rust-modules.mjs` successfully and shared complete inventory. Every declared Rust module has textual references; this does not establish actual runtime reachability.
- Added `docs/LEGACY_HOME_AI_AUDIT.md` commit `3c560e5` (documentation only; no Rust changes). Legacy Home AI group ~190 KB: model_hub.rs (~84K), model_trial.rs (~39K), conversation.rs (~35K), project_context.rs (~32K). `updates.rs` checks `conversation::has_active_generation()` and `model_trial::is_running()`; `lib.rs` calls `model_hub::stop_owned_local_runtimes()` on shutdown. State lives in ACTIVE_GENERATIONS, ACTIVE_TRIAL, OWNED_OLLAMA. Preserve these protections; do not replace checks with false.
- Inspected direct qualified calls in mcp_server.rs/chatgpt_bridge.rs; none for these four modules, but indirect imports and feature-gated use remain to be checked. Next: check entire repo frontend references, Rust imports and feature flags, then decide if Home AI feature can be retired as one coherent dependency group. Avoid arbitrary dead_code pruning. No build required for docs-only patch.
- Preserve active MCP, security, gateway, tunnels, Projects/Connections/tray. Avoid unrelated local Cargo.lock/github.rs changes.

## CURRENT HANDOFF — 2026-10-13 — Rust dependency audit tooling
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed previous deletion of retired `https_setup.rs` PASSED.
- Added `scripts/audit-rust-modules.mjs` in commit `5338962`. Run `node scripts/audit-rust-modules.mjs` from repo root to inventory declared Rust modules and textual references from other sibling Rust files. This is conservative, NOT a compiler reachability graph: false positives include comments/tests; inspect `include!`, `#[path]`, feature gates, build scripts, imports, and runtime safety before deletion. **Script not yet validated on user's local machine.**
- Confirmed from source: `updates.rs` calls `conversation::has_active_generation()` and `model_trial::is_running()`; `conversation.rs` imports `model_hub` and `project_context`; `lib.rs` invokes model_hub shutdown; `mcp_server.rs` references self_continuation and chatgpt_bridge. Do not remove these as stand-alone dead modules.
- Next: user runs `git pull --ff-only && node scripts/audit-rust-modules.mjs`, share relevant output. Use results to select a small safe batch; preserve active MCP, access/security, gateway/tunnels, Projects/Connections/tray. Rust warnings baseline ~481 after import fix, exact latest count not reported. Preserve unrelated local Cargo.lock/github.rs changes.

## CURRENT HANDOFF — 2026-10-13 — retired HTTPS setup removed, validation pending
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed previous browser/change/terminal wrapper deletion batch PASSED. Warning count not reported.
- Removed `mod https_setup;` from `src-tauri/src/lib.rs` commit `9580d8a` and deleted `src-tauri/src/https_setup.rs` (~26.9 KB) commit `7130872`. This was the old desktop HTTPS setup assistant; active `direct_https.rs`, `public_tunnel.rs`, gateway, MCP, and tunnel commands retained. Inspected references in lib.rs, direct_https.rs, public_tunnel.rs, gateway.rs, gateway_commands.rs, public_tunnel_commands.rs, external_access.rs, launcher.rs, connection.rs, mcp_server.rs, app_state.rs; none referenced https_setup except its declaration. **Not a full repo-wide cross-module search; validation pending.**
- Next: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml`. If unresolved import arises, restore dependency or module rather than weakening active code. Track warning count.
- Important prior dependency: `updates.rs` calls `model_trial::is_running()`; model_trial deletion was reverted. `conversation.rs` uses `project_context` and `model_hub`; model_hub has shutdown hook; MCP references chatgpt_bridge. Preserve safety/security, MCP, gateway, tunnels, Projects/Connections/tray; avoid local Cargo.lock/github.rs unrelated edits.

## CURRENT HANDOFF — 2026-10-13 — retired desktop IPC wrappers removed, validation pending
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed previous `cargo check` PASSED after unused import cleanup commit `b0734d9` (warning count not supplied; prior baseline 483).
- Removed `mod browser_commands;`, `mod change_commands;`, `mod terminal_commands;` from `src-tauri/src/lib.rs` commit `4013f67`. Deleted the three now-unmounted wrapper files in commits `0088069`, `f02dbd2`, `ba6fb11` (~23.4 KB). Their desktop Tauri invoke registrations were removed in earlier validated patches; underlying `browser.rs`, `terminal.rs`, `changes.rs` MCP engines retained. **Current batch not yet validated.**
- Next run `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml`; report pass and warnings. Consider `cargo test --manifest-path src-tauri/Cargo.toml --lib` after this batch to protect security/IPC regression tests if feasible.
- Do NOT remove `model_trial.rs`: `updates.rs` depends on `model_trial::is_running()` as an update safety guard. An attempted removal was reverted and user validated restoration. `conversation.rs` uses `project_context.rs` and `model_hub.rs`; `lib.rs` calls model_hub shutdown hook; MCP uses `chatgpt_bridge`. Full dependency graph is still not completed; audit cross-module references before compiled engine deletion. Keep active MCP, gateway, access/security, tunnels, Projects/Connections/tray. Preserve unrelated local Cargo.lock/github.rs changes.

## CURRENT HANDOFF — 2026-10-13 — compiled Rust cleanup safety-first
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed **cargo check PASSED, 483 warnings** after restoring `model_trial.rs` and its module declaration. The prior deletion broke `updates.rs` import because the update safety guard calls `model_trial::is_running()`; preserve until update dependency is handled explicitly. The failed attempt was fully reverted in commits `e4a2969` and `d2e5507`.
- Warning log contains overwhelmingly unused-code warnings and 2 unused imports. Commit `b0734d9` removes the two unused imports from `mcp_server.rs`, keeping the browser launch helper imported within its test module. **Validation pending** for this change.
- An attempted broad cross-module reference audit exceeded GitHub tool-call budget; do not claim it completed. Audit modules across all Rust sources before deleting compiled implementations, including safety checks and startup/shutdown. `conversation.rs` uses `model_hub` and `project_context`; `updates.rs` uses `conversation` and `model_trial`; `lib.rs` uses `model_hub::stop_owned_local_runtimes()`; MCP uses `chatgpt_bridge`. Preserve active MCP, security, gateway, tunnels, Projects, Connections, tray.
- Next: `git pull --ff-only && cargo check --manifest-path src-tauri/Cargo.toml` (and Rust tests when appropriate), then dependency-map smaller modules before deletion. Do not delete code merely to reduce warnings. Previous 20 unmounted Rust files already removed and validated. Avoid unrelated local edits Cargo.lock/github.rs.

## CURRENT HANDOFF — 2026-10-13 — warning audit and first compiled module removal
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User validated prior batch: all checks passed with **483 Rust warnings**. User uploaded warning-only log, overwhelmingly dead_code; two unused imports. This warning count is the baseline before current change.
- Audited compiled modules: `model_trial.rs` was declared in `lib.rs` but no active MCP dispatch references; deleted module declaration commit `bd1e7ba` and file commit `e03f336`. **Not yet locally validated**. Expect many model-trial warnings to disappear. Run one check after a useful batch.
- `conversation.rs` references `project_context` and `model_hub`; `model_hub::stop_owned_local_runtimes()` is called on app shutdown. Preserve these until auditing startup/shutdown, other modules and tests. `chatgpt_bridge` is referenced by active MCP server; DO NOT delete just because its legacy startup feature is off.
- Previous cleanup removed 19 unmounted *_commands.rs and unmounted review_council_core.rs, now validated. No remaining undeclared Rust source files were found then. Protect MCP browser/terminal, gateway, security/access, public tunnel, Projects/Connections/tray. Avoid unrelated local edits Cargo.lock/github.rs.
- Next: audit and remove truly orphaned compiled modules in dependency groups, then `git pull --ff-only && npm run check && npm run test:frontend && npm run build && cargo check --manifest-path src-tauri/Cargo.toml` once. Track warning count. Keep NEXT.md current.

## CURRENT HANDOFF — 2026-10-13 — unmounted Rust modules removed; validate batch
- Branch `cleanup/dead-video-view`; DO NOT MERGE. Last user-confirmed validation after desktop IPC pruning passed with **422 Rust warnings** (previous 400). Frontend checks/build passed.
- Deleted **19 unmounted `*_commands.rs` files** across 19 small commits: model_trial, filesystem, conversation, media, github, ai_workspace, checkpoint, editor, execution, git, history, integration, launcher, model_hub, monitoring, project_lifecycle, safety, temp_workspace, workspace_diagnostics. None was declared by `mod` in `src-tauri/src/lib.rs`; their underlying Rust implementations were retained. Also deleted unmounted `review_council_core.rs` commit `be2ddde`. Latest command-wrapper deletion commit `ab34429`.
- A directory-to-`lib.rs` module declaration audit now finds no remaining unmounted `.rs` files under `src-tauri/src` apart from Rust entrypoints. Caveat: other Rust modules can reference files via `#[path]` or `include!`; do not extrapolate this check into full dependency reachability.
- **Next action: one combined validation** `git pull --ff-only && npm run check && npm run test:frontend && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. Report pass/fail and warning count. Expect warnings roughly unchanged since removed files were not in compiled module tree.
- Then audit remaining **compiled** Rust modules and MCP dispatch carefully; do not delete Rust code on warning count alone. Preserve gateway, terminal, browser, MCP, access/security, public tunnel, Projects/Connections/tray. Avoid overwriting unrelated local changes `src-tauri/Cargo.lock`, `src-tauri/src/github.rs`.
- `src/lib/applicationFamilies.ts` still orphaned; deletion blocked repeatedly by tool safety checks. `NEXT.md` maintained for cross-chat handoff. Prefer batching safe deletions to reduce multi-minute Rust checks.

## CURRENT HANDOFF — 2026-10-13 — Rust desktop IPC pruning, validation pending
- Branch `cleanup/dead-video-view`; DO NOT MERGE. User confirmed frontend reduction/dependency cleanup. CodeMirror packages removed in user-pushed commit `547c4360`; frontend test script now uses `vitest run --passWithNoTests`.
- Rust Tauri command registry cleanup: browser desktop commands (25) commit `5a2406e` user validated, terminal desktop commands (14) commit `f10b55b` and remaining retired desktop commands (22) commit `8aa4f8b` are **pending combined validation**. The 22 cover file-change (11), workspace project creation (1), workspace access check (1), HTTPS setup UI (4), and update UI (5). Underlying Rust and MCP services remain compiled; no Rust implementation deleted.
- Latest confirmed Rust warning count after browser registration removal: ~400, up from ~369. Warning growth can follow removal of Tauri reachability; do not equate warnings with runtime failure. Confirm combined batch compiles and smoke-test MCP operations before deeper Rust pruning.
- Validation once for this batch: `git pull --ff-only && npm run check && npm run test:frontend && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. User finds repeated Rust checks slow; batch safe registry changes, but check implementation/security/MCP changes individually.
- Remaining active frontend `src/lib/backend.ts` is ~139 lines (26 wrappers) after `1a8d502`, user validated. `src/lib/applicationFamilies.ts` remains orphaned (64 lines); deletion repeatedly blocked by tool safety checks. Do not claim removed.
- Next: assess warnings and command references, then plan Rust module cleanup based on Tauri registration, MCP dispatch, and actual call graph; preserve Projects/Connections/tray, gateway, HTTPS/public tunnel, security/access controls and MCP. Do not overwrite unrelated local edits to Cargo.lock/github.rs. Keep NEXT.md current.

## CURRENT HANDOFF — 2026-10-13 — frontend component cleanup complete, validation pending
- Branch `cleanup/dead-video-view`; DO NOT merge. User validates locally: `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`.
- Last user-confirmed validation passed after deleting `HttpsSetupGuide.test.tsx` and `HttpsSetupGuide.tsx` (commits `f608987`, `b93e3d1`).
- Just removed final unreachable component `src/components/ProductionPanel.tsx` (339 lines) in commit `3f01cc2`; **await local validation**. Earlier removal was blocked, retry succeeded.
- Previous full `node scripts/audit-frontend.mjs` inventory: 27 source files, 18 components, 11 reachable, 7 unreachable. All seven unreachable component/test files have now been deleted across commits: ProjectSetupPanel `390acf7`, PendingChangeReview `8121fef`, GitPanel `f2c56a5`, WorkspaceProductivity `8183269`, HttpsSetupGuide.test `f608987`, HttpsSetupGuide `b93e3d1`, ProductionPanel `3f01cc2`.
- Next after validation: rerun `node scripts/audit-frontend.mjs` to verify 11 reachable/0 unreachable components. Audit other `src/lib` modules, unused backend wrappers, frontend dependencies, and associated tests before Rust cleanup. Keep active Projects/Connections/tray, gateway, HTTPS/public tunnel, MCP and security/access controls intact. Rust warning baseline 369; do not delete merely because of warnings.
- User explicitly requested NEXT.md stay updated across chat length limits. Older handoff sections below are historical and may say awaiting validation incorrectly.

## CURRENT HANDOFF — 2026-10-13 — frontend cleanup (latest)
- Branch: `cleanup/dead-video-view`, DO NOT merge. User has approved small, sequential commits and validates locally with `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. Last validation: commit `f2c56a5` passed.
- Fresh `node scripts/audit-frontend.mjs` output after prior cleanup: 27 source files, 18 components, 11 reachable, 7 unreachable. Preserve reachable: AppErrorBoundary, AppHeader, AppSidebar, ChatConnectionPanel, ConfirmationDialog, GatewayPanel, PublicTunnelPanel, RepoTunnelLogo, StartupPanel, WindowChrome, WorkspaceList.
- Seven unreachable at audit: GitPanel (447 lines), HttpsSetupGuide (509) + HttpsSetupGuide.test (134), PendingChangeReview (108), ProductionPanel (339), ProjectSetupPanel (80), WorkspaceProductivity (339). Subsequent commits: `390acf7` removed ProjectSetupPanel, `8121fef` removed PendingChangeReview, `f2c56a5` removed GitPanel; all user-validated.
- Remaining four unreachable component/test files: `src/components/HttpsSetupGuide.tsx`, `src/components/HttpsSetupGuide.test.tsx`, `src/components/ProductionPanel.tsx`, `src/components/WorkspaceProductivity.tsx`. Audit confirmed no incoming static imports except test -> HttpsSetupGuide. WorkspaceProductivity imports active AppSidebar but not vice versa. Remove in safe batches, include associated test, then validate.
- User explicitly requested NEXT.md stay up-to-date to survive chat context loss. Update it after meaningful batches or phase transitions; supersede older 'awaiting validation' entries below rather than trusting stale status.
- Then audit frontend lib helpers/backend wrappers/dependencies before Rust. Rust warnings baseline 369; do not assume warnings mean safe deletion. Preserve active MCP gateway, security/access controls, Projects/Connections/tray UI, and Rust services until call-site checks. No local Windows build access. Do not overwrite unrelated uncommitted local changes.

## Frontend audit results and video UI cleanup (awaiting validation)
- User ran `node scripts/audit-frontend.mjs`: 50 source files, 41 component TS/TSX files, 11 reachable from `src/main.tsx`, 30 unreachable. Inventory is static graph only; 11 reachable include AppErrorBoundary, AppHeader, AppSidebar, ChatConnectionPanel, ConfirmationDialog, GatewayPanel, PublicTunnelPanel, RepoTunnelLogo, StartupPanel, WindowChrome, WorkspaceList. Preserve those.
- Removed four mutually related, unreachable video UI files including their exclusive tests: `VideoPanel.tsx` (521 lines), `VideoProductionPanel.tsx` (1180), `VideoPanel.test.tsx` (97), `VideoProductionPanel.test.tsx` (432). Commits d5c9d94, f6f70af, a7bd950, cd0c024. About 2,230 lines. MCP video services and Rust unchanged.
- Await local validation `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. Optionally run `npm run test:frontend` to detect stale test dependencies. Do not claim validated until user confirms.
- Next: remove other unreachable component clusters in controlled batches, including tests; then prune exclusive lib helpers, backend wrappers and unused npm dependencies with separate checks. Keep cleanup/dead-video-view unmerged.

## Frontend component inventory (awaiting local output)
- User validated previous model-trial helper/test cleanup.
- Added `scripts/audit-frontend.mjs` (d2d32a7, corrected escape sequences 33548ca): read-only TypeScript AST import/export graph from `src/main.tsx`; lists all `src/components/*.ts(x)` files, line counts, inbound static imports, and reachable vs unreachable from the active entrypoint. It also scans literal dynamic imports and CSS @import. Uses existing `typescript` dev dependency. No app runtime, Rust, MCP, or dependencies changed.
- Ask user to run `git pull --ff-only && node scripts/audit-frontend.mjs` and share output. Do not delete further files until complete inventory reviewed. Caveats: runtime computed imports, non-main entrypoints and tests need manual inspection; tsconfig includes all src.
- Next classify unreachable groups and their tests/exclusive helpers, then remove in validation batches. Keep cleanup/dead-video-view unmerged.

## Model trial cleanup test dependency fix
- User's `npm run check` failed after deleting `src/lib/modelTrial.ts` because `src/lib/modelTrial.test.ts` still imported it. Reviewed test: it exclusively exercises removed `modelTrialKey`/`sameModelIdentity` helpers. Deleted orphan test in commit 7f24cb3. This does not remove tests for active code.
- Await validation: `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`.
- Future cleanup must inspect associated `*.test.ts(x)` files before deleting exclusive helpers; full component inventory still outstanding.

## Frontend cleanup batch 3 — model trial (awaiting validation)
- User confirmed prior editor-cluster deletion passed local check/build/cargo check.
- `tsconfig.json` includes all of `src`, so unmounted components still need valid imports. Inspected `src/components/ModelTrial.tsx` (97 lines) and `src/lib/modelTrial.ts` (15 lines). ModelTrial was only a child of deleted ModelHub UI; its helper contains only selection comparison utilities. Deleted both (commits be48b6a, 6842931). No Rust/MCP changes.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`.
- Still pending: comprehensive `src/components` directory inventory and transitive import reachability graph. `tsconfig` includes src, so validate every deletion batch. Do not assume that unmounted means safe to delete if another legacy component imports it. Keep cleanup/dead-video-view unmerged.

## Frontend cleanup batch 2 — editor dependency cluster (awaiting validation)
- User confirmed prior restored-DeveloperDock state passes check/build/cargo check.
- Inspected current `WorkspaceEditor.tsx` (510 lines), `CodeMirrorSurface.tsx` (427 lines), `DeveloperDock.tsx` (894 lines): WorkspaceEditor imports both CodeMirrorSurface and DeveloperDock; CodeMirrorSurface imports the EditorProblem type from DeveloperDock. None is imported by current `src/App.tsx` or `src/main.tsx` entrypoint. Deleted all three as a single legacy editor cluster (commits 5f6e8d9, 622c483, 216c651), rather than leaving broken internal imports. `ModelHub.tsx` remains deleted from prior batch. No Rust/MCP changes.
- Local validation needed: `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. If TypeScript reports imports from another legacy component, audit and remove its dead cluster or restore as necessary. Assistant cannot run user's Windows build.
- Still need comprehensive component directory inventory and reachable import graph; do not claim exhaustive audit. Next consider unused `ModelTrial.tsx` and its exclusive helpers, after validating and checking dependents. Keep branch cleanup/dead-video-view unmerged.

## Frontend cleanup build correction
- User's `npm run check` caught six errors after deleting DeveloperDock: CodeMirrorSurface imports EditorProblem type; WorkspaceEditor imports DeveloperDock and EditorProblem. Type inference errors cascade from missing module. These legacy files are still included by TypeScript even if not mounted in App.
- Commit 6494cab restores DeveloperDock unchanged as immediate safe fix; ModelHub remains removed. Do not claim 1,236 lines removed; net deletion currently only ModelHub (340 lines). User should pull and rerun checks.
- Next: audit full TypeScript inclusion and import graph; remove dead WorkspaceEditor/CodeMirrorSurface/DeveloperDock together only after verifying no active dependents, or extract shared types if any active component needs them. Avoid deleting files individually solely because App does not import them.

## Frontend legacy component cleanup — batch 1 (awaiting validation)
- User requested inventory/deletion of leftover components after tray-first simplification. Active `src/App.tsx` imports AppHeader, AppErrorBoundary, AppSidebar, ChatConnectionPanel, ConfirmationDialog, GatewayPanel, StartupPanel, WindowChrome, PublicTunnelPanel, WorkspaceList; `src/main.tsx` imports App and AppErrorBoundary. Do not remove those or their transitive dependencies.
- Removed `src/components/ModelHub.tsx` (341 lines) and `src/components/DeveloperDock.tsx` (895 lines) in commits dfc74e3 and 58ebf40. Neither is imported by active App/main entrypoints; no Rust backend or MCP tools changed. This is a targeted first batch, not a complete repository-wide reachability proof.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; TypeScript build will catch any surviving static imports. If it fails, fix dependents rather than restoring obsolete UI wholesale.
- Next: obtain complete `src/components` and frontend import graph (prefer local inventory script or repository tree) and classify direct/transitive reachability, then remove dead components/helpers/styles in batches. Keep cleanup/dead-video-view unmerged. Preserve MCP backend.

## Long-outage connection recovery (awaiting local validation)
- User confirmed prior finished-worker cleanup passed local checks. Live MCP was available while hidden.
- Commits d056cb4 and e0ae1b5 add in-memory `public_connection_desired` and `shutting_down` flags to AppState. Explicit stop_gateway and clear_public_tunnel clear desired; configure_public_tunnel sets desired. Startup initializes desired from saved auto_start. ExitRequested marks shutdown before stopping services.
- Commit 6ecc297 adds a 15-second low-frequency connection recovery thread. While desired and not shutting down, it checks public tunnel status, which already cleans up/restarts unexpectedly finished workers. If status reports not running, it tries start_public_tunnel. Errors are logged, with next attempt on next interval. The thread stays idle when connection is deliberately stopped, and terminates after exit signal. Healthy runtime untouched; no frontend polling or extra MCP permissions.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. Then verify MCP while hidden, intentional Stop gateway stays stopped after 30+ seconds, restart/configure restores connectivity, and Quit exits cleanly. Test longer network outage if practical. Assistant did not compile locally.
- Follow-up audit: startup retry worker and recovery thread may both try start around startup, but AppState mutex serializes tunnel spawn; examine interactions with provider reconfiguration and `public_tunnel_status` auto-restart under concurrent calls. Avoid broad changes until user validates. Keep cleanup/dead-video-view unmerged.

## Reconnect lifecycle safety (awaiting validation)
- Live MCP list_workspaces confirmed working while RazedTunnel is hidden; previous startup retry patch was not explicitly build-validated in latest user message.
- Audit found `AppState::start_public_tunnel` returned Ok whenever a runtime object existed, even if its worker had already exited. Commit c285c4f fixes this: stop/join a finished worker before checking for an active runtime, allowing explicit/startup reconnect attempts to spawn a new worker. Healthy running workers remain untouched. This is a small prerequisite for safe recovery, not a new background supervisor.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; then check MCP while hidden. Assistant did not compile locally.
- Next: design background watchdog that distinguishes intentional Stop from unexpected failure, supports cancellation on exit, avoids races with provider reconfiguration, and retries long outages. Do not introduce an unconditional poll that restarts intentionally stopped connections. Keep cleanup/dead-video-view unmerged.

## Auto-connect retries after login (awaiting validation)
- User reported previous Windows startup toggle build passed. ChatGPT MCP was confirmed accessible with UI hidden.
- Audited `lib.rs` auto-connect and `app_state.rs`/`public_tunnel.rs` recovery: startup auto-connect previously attempted only once. Cloudflare worker itself has restart/backoff; ngrok session handles network interruptions; public_tunnel_status can restart a finished worker when queried. These do not guarantee recovery after an initial login-time network failure.
- Commit db34ad3: when saved tunnel config has auto_start enabled, startup worker attempts `start_public_tunnel` up to three times, waiting 10 seconds after failed attempts, logging each failure. Successful connection ends loop. No new background service or frontend polling, no change to MCP tools/permissions. Existing behavior when auto_start disabled is unchanged.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. Then fully restart and verify MCP with UI hidden. To exercise retries, test an intentional temporary network outage during startup only when convenient; watch logs for attempt 1/3 etc. The assistant did not compile locally.
- Remaining limitation: bounded retries can still fail if network unavailable longer than retry window; worker-exit recovery is currently status-query-driven in app_state.rs. Future work: carefully scoped lifecycle supervisor with shutdown cancellation and no duplicate tunnels. Keep cleanup/dead-video-view unmerged.

## Windows startup toggle on Connections (awaiting validation)
- User confirmed previous Windows autostart backend change compiled and MCP remains available while window hidden.
- Commit fd91119: Windows `set_launch_at_login(enabled=true)` rejects debug builds, preventing a development executable from being registered for login startup. Disabling remains allowed. This guards the registry path; it does not auto-enable anything.
- Commit 4e99116: new `src/components/StartupPanel.tsx` reads existing `getRuntimeDiagnostics` and shows opt-in checkbox, busy/error handling, and installed-release requirement. Commit 1b4e8df mounts it on Connections. Existing `setLaunchAtLogin` backend wrapper and `runtime_commands` Tauri registrations are reused.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; open Connections and check Startup panel. In `tauri dev`, checkbox enable should show an explanatory error and not create registry startup entry. Installed release build can opt in. Check that toggling off removes the entry and that login startup opens only in tray. Assistant did not compile locally.
- Important follow-up: Windows registry launch command quoting and installed-executable path should be verified on an installed build, not just `cargo check`; consider inspecting HKCU Run value after opt-in. Do not merge cleanup/dead-video-view.

## Opt-in Windows login startup (awaiting validation)
- User confirmed ChatGPT MCP `list_workspaces` works while main window hidden; tray-first startup validated. Idle baseline 47.30 MB working set / 7.70 MB private.
- Audited `hardening.rs`: existing `runtime_commands::set_launch_at_login` supported Linux only; Windows returned unsupported. Added Windows HKCU Run-key handling with `reg.exe` in commits `79bc58e` and `0aa9509`; existing Linux path unchanged. Detection uses `reg query`, opt-in enable uses `reg add`, disable uses `reg delete`. Executable path is quoted. Does NOT auto-enable on startup or modify registry until existing Tauri command is explicitly invoked.
- **Important:** existing Projects/Connections UI does not expose this control yet. Do not invoke enable from a development executable: the registered path would point to the debug binary. Next stage should add a deliberate UI control in Connections or tray and guard dev builds/confirm correct installed executable before enabling.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; verify tray hidden start and MCP. No registry change expected from just building/running. Assistant cannot compile locally. Warning baseline 368.
- Next: add safe opt-in UI and guard development startup paths, then test Windows login behavior. Keep cleanup/dead-video-view unmerged.

## Tray-first startup (awaiting validation)
- Idle baseline: 47.30 MB working set, 7.70 MB private memory for RazedTunnel process; WebView2 separate.
- Commit a88fe12 adds `visible: false` to the main Tauri window config. Existing tray Open, Projects, Connections, Quit, double-click, and close-to-hide logic remains unchanged. MCP initialization remains unchanged.
- Validate: `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`. Fully quit/relaunch. Verify starts hidden with tray icon, Open/Projects/Connections show UI, closing hides, Quit exits, MCP works hidden. Warning baseline 368. No local compilation by assistant.
- Next: audit autostart and reconnect behavior. Keep cleanup/dead-video-view branch unmerged.

## Backend lifecycle/dependency audit (no runtime change; next implementation decision)
- User validated previous desktop execution wrapper cleanup: **368** compiler warnings (437 original; 69 fewer). Warnings can rise when wrappers disappear and core functions become dead-code warnings; not a reliable size metric.
- Audited `src-tauri/src/lib.rs` (342 lines), `src-tauri/Cargo.toml`, `src-tauri/src/app_state.rs`, `src-tauri/src/model_hub.rs`, `src-tauri/src/updates.rs`, `src-tauri/src/system_tray.rs`.
- Default startup: install rustls AWS-LC provider; install tray; hardening initialize; load persisted AI access pause fail-closed; updater post-update health check; auto-start configured public tunnel in worker; terminal initialize (interrupted command/process recovery); monitoring initialize (persisted watchers). Old GitHub status prewarm, AI Workspace recovery, and ChatGPT bridge are already behind disabled-by-default `legacy_background_services` feature.
- Shutdown: stop monitoring, terminal transient activity, browser activity, owned Ollama runtimes, legacy bridge/AI workspace if feature enabled, gateway, hardening. `model_hub::stop_owned_local_runtimes()` is guarded by `OWNED_OLLAMA` OnceLock and exits without action if no owned process; do not remove before tracing any MCP paths that can start owned Ollama. `runtime_commands::set_launch_at_login` remains registered and may be needed for tray-first startup.
- Major dependencies: axum/axum-server/rmcp/ngrok/reqwest/tauri/rustls/tauri-plugin-updater; no Cargo removals yet because live MCP and HTTPS paths depend on them. Tauri tray-icon feature enabled. No evidence of a continuously running Model Hub polling loop in default startup; avoid attributing memory overhead without profiling.
- User already confirmed read-only MCP Git status/log/diff successful after `git.exe` fix. Next measure release executable size and baseline idle memory/startup, inspect `terminal::initialize` and `monitoring::initialize` for unnecessary work, and map compiled modules to MCP tools before changing dependencies or runtime services. Suggested Windows baseline: `Get-Process razedtunnel | Select-Object ProcessName,Id,WorkingSet64,PrivateMemorySize64,StartTime` after launching app (actual process name may vary); `Get-Item src-tauri/target/release/*.exe | Select Name,Length` only if release build exists. Keep branch `cleanup/dead-video-view`; do not merge.

## Desktop execution IPC wrapper cleanup (awaiting validation)
- User validated previous launcher wrapper exclusion with **365** compiler warnings, up from 359. Increase can arise from newly unused underlying code; no root-cause analysis yet. Original baseline 437 (72 fewer). Do not treat warning count alone as quality metric.
- Inspected `lib.rs`, `execution_commands.rs` (64 lines), `mcp_server.rs`, `app_state.rs`, `src/App.tsx`; six desktop execution Tauri handler registrations remained but no direct references to wrapper from inspected MCP/AppState/UI modules.
- Commit `9621b63` removes six `execution_commands::*` Tauri handler registrations and `mod execution_commands;` from `lib.rs`; source file retained. Core `execution`, project access policy, approvals, and MCP tool implementations untouched.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; test Projects/Connections/tray, optionally read-only MCP execution status/presets. Report warnings vs 365. Assistant did not compile locally.
- Next: dependency graph and actual binary size/service audit; avoid removing core MCP tools, and preserve runtime launch-at-login support for tray-first design. Do not merge.

## Desktop launcher wrapper cleanup (awaiting validation)
- User confirmed live MCP Git status, log, and compact diff all passed on approved `story-gospel-according-to-ashe` after Windows `git.exe` discovery fix. Branch main, clean worktree. Last compiler warning count 359.
- Inspected `lib.rs` and `launcher_commands.rs` (67 lines). Old desktop launcher wrapper still registered six Tauri IPC commands; not mounted in current Projects/Connections interface. Core `launcher` remains separate for MCP. Also inspected `execution_commands.rs` and `runtime_commands.rs` but did not alter them: `set_launch_at_login` may be needed for tray-first startup.
- Commit `9c5c010` removes all six `launcher_commands::*` handler registrations and `mod launcher_commands;` from `lib.rs`. Source file remains in Git. Core `launcher`, access checks, activity history, MCP tools remain unchanged.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; report warning count and verify Projects/Connections/tray. If possible test read-only MCP launcher history; avoid triggering external launches just for testing. Assistant has not compiled locally.
- Next audit backend compiled dependency graph/size, rather than optimize warning count alone; preserve MCP functionality and tray startup. Do not merge.

## Fix Windows Git discovery for MCP (awaiting user rebuild/restart)
- MCP connectivity and `list_workspaces` succeeded, but MCP `git_status` returned unavailable and `git_log`/`git_diff` returned `Git was not found in PATH`. User confirmed PowerShell `Get-Command git` and `where.exe git` resolve `C:\\Program Files\\Git\\cmd\\git.exe`.
- Root cause in `src-tauri/src/git.rs`: `git_binary()` checked `directory.join("git")` with `is_file()`, which does not use Windows PATHEXT and misses `git.exe`. This was a pre-existing detection bug, not evidence that removing `git_commands` broke MCP.
- Commit `11205bf` changes Windows PATH search to explicit `git.exe`, retains Unix `git`, and adds fallback to `ProgramFiles/Git/cmd/git.exe` or `ProgramFiles(x86)/Git/cmd/git.exe` for tray processes with reduced PATH. No shell invocation or change to project access policy. Git core/MCP remains intact.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`, **restart the running Tauri/tray process** to load the updated binary, then use ChatGPT MCP `list_workspaces`, `git_status`, `git_log`, `git_diff` read-only on an approved Git project. No local compilation performed by assistant.
- Note: Git repository check requires `.git` directory inside approved workspace; git worktrees with `.git` files are currently unsupported separately. Next continue dependency/size audit after successful MCP test. Do not merge.

## Rust compilation cleanup: desktop Git wrapper (awaiting validation)
- User validated monitoring wrapper removal; warning count remained **359** for two consecutive wrapper-removal stages (437 initial, 78 fewer). Do not use warnings as the sole optimization metric.
- Audited `lib.rs`, `git_commands.rs` (96 lines), `mcp_server.rs`, `app_state.rs`, `src/App.tsx`. Desktop Git wrapper had ten Tauri handler registrations but no direct references in inspected MCP/AppState/App modules.
- Commit `d53a63a` removes all ten `git_commands::*` handler entries and `mod git_commands;` **together**. `git_commands.rs` remains in Git. Core `git`, `mcp_git_helpers`, GitHub integration, project permissions, and MCP server remain untouched; no change to Git access or credentials intended.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`, Projects/Connections/tray, and especially MCP Git status/diff if available. Report warnings vs 359. Assistant did not compile locally.
- Next pivot from warning counting to compiled dependency/size and runtime-service audits; verify MCP feature coverage before removing core modules. Do not merge.

## Rust compilation cleanup: monitoring desktop wrapper only (awaiting validation)
- User validated prior media wrapper exclusion; warning count remained **359** (baseline 437). No additional warnings eliminated by that change.
- Inspected `lib.rs`, `monitoring_commands.rs`, `mcp_server.rs`, `app_state.rs` previously. Monitoring desktop wrapper consists of five Tauri IPC commands; no mounted Projects/Connections UI uses them. Distinguish the wrapper from core `monitoring` runtime, which resumes persisted watchers and is required by MCP.
- Commit `e7faf38` removes all five `monitoring_commands::*` Tauri registrations and the `mod monitoring_commands;` declaration **together**. Source file remains. Crucially `mod monitoring;`, `monitoring::initialize(app)` and `monitoring::stop_all_activity()` remain unchanged; monitoring MCP functionality preserved at implementation level.
- Validate `git pull --ff-only && npm run check && npm run build && cargo check --manifest-path src-tauri/Cargo.toml`; test Projects, Connections, tray, MCP monitoring if available, and report warning count vs 359. Assistant did not compile locally.
- Next inspect remaining desktop-only wrappers and their registrations; prioritize eliminating genuinely unused compilation units, but warning count is not sole success metric. Do not merge.

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
