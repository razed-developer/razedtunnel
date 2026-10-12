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
