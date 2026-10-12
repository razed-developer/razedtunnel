# RazedTunnel — NEXT

Updated: 2026-10-12
Current branch: `cleanup/remove-team-mode` (not yet merged)
Purpose: durable handoff between ChatGPT conversations. **Read this file first** before changing code. Keep it updated after each validated stage, before switching chats, and after merges.

## Goal and boundaries
RazedTunnel is a compact Windows/Tauri Rust MCP gateway for approved local workspaces, files, terminal, Git/GitHub, and related development tools. Keep modules focused and remove legacy RepoTunnel features that do not serve the gateway. Do not break ordinary workspace permissions, MCP access, or file operations. Prefer small commits and local validation; do not merge before user approval/validation.

## Completed and merged
- Legacy video cleanup merged to `main` at `70ea13cd605dacf735f81a5188037443ec70be17`.
- Legacy Android/phone cleanup merged via PR #1 at `beed3fb835dc0f8daf19064139aac9ce446799d3`. User validated frontend, Rust check, and Rust library tests (295 passed, 2 ignored).
- Windows Rust library tests may require `$env:REPOTUNNEL_WINDOWS_TEST_MANIFEST = "1"` to avoid the Windows `TaskDialogIndirect` test-runner issue.

## In progress: remove Team Mode
Branch: `cleanup/remove-team-mode`, based on `main` after Android merge.
Validated by user:
1. Removed Team Mode UI routing/navigation/header/command-palette entries. Frontend `npm run check && npm run build` passed after fixing remaining references.
2. Removed desktop `team_commands` registrations and deleted `src-tauri/src/team_commands.rs`. `cargo check` passed.
3. Removed `TeamPanel.tsx`, Team frontend API wrappers in `src/lib/backend.ts`, and Team TS models in `src/types.ts`. Frontend check/build passed.
4. Removed MCP `team_status` and `team_action` tools and their parameter models from `src-tauri/src/mcp_server.rs` (commit `994f016d7ea5e0b996cef7201b189e02d35d47c3`). **User confirmed `cargo check` passed**.

## Latest patch — awaiting user validation
- Commit `c47a117`: removed obsolete Team Mode feature table row, feature section, and docs navigation link from `README.md`. The implementation's Rust models/activity and primary frontend files inspected have no Team Mode references. Historical `docs/team-mode.md` and other legacy documentation may still need a separate documentation cleanup. **Awaiting full frontend/Rust validation.**

## Prior MCP cleanup — validated
- Commit `917045b`: cleaned unused imports and Team-related `client_key`/destination locals in `mcp_server.rs`; removed obsolete Team Mode wording from external-file tool description. **User confirmed `cargo check` passed.**

## Previous engine patch — validated
- Commits `dee7050`, `4a350e9`, `09e917a`: removed `mod team;`, deleted 2,649-line `src-tauri/src/team.rs`, and removed the trailing Team-specific Rust session models in `src-tauri/src/models.rs` (~212 lines). **Initial `cargo check` failed on three leftover imports/calls. Fixed in commits `f2de9c0`, `cc0b752`, `f2123c7`; user subsequently reported 291 Rust tests passed, 0 failed, 2 ignored (2026-10-12).**

## Earlier browser patch — validated
- Commit `aad66688bda47b5c2a90b5fe5d9fafa835a80832`: removed eight `team::assert_browser_mutation_available` checks from MCP browser mutation handlers. Independent approved workspace scope, tab access, Google/Gmail access, and AI browser ownership checks remain in place. **User confirmed `cargo check` passed.**

## Prior patch — validated
- Commit `5ee379185cec878a0a869f6044b7640e2e46505f`: removed eight `team::assert_paths_available` calls from MCP filesystem mutations, leaving workspace approval and `changes::*` mutation handling in place. **User confirmed `cargo check` passed.** Possible unused `client_key` locals and rename destination calculation need cleanup after compilation; do not mistake warnings for test failures.

## Next steps — priority order
1. Run full frontend (`npm run check && npm run build`) and Rust (`cargo check`, then library tests with Windows manifest environment variable) validation. Review remaining legacy docs references and cleanup separately if needed. Then create PR and merge only after user approval. Audit remaining Team-only activity variants, labels, UI icon strings, docs, and tests; remove only if no other dependencies. `src-tauri/src/team.rs` (~2649 lines) is still used for `team::assert_paths_available` in filesystem MCP operations and `team::assert_browser_mutation_available` in browser MCP operations. These checks may represent coordination locks rather than core workspace authorization. Inspect exact call sites and preserve independent approval/access checks.
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
