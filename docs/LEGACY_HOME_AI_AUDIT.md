# Legacy Home AI dependency and retirement audit

Date: 2026-10-13. Branch: `cleanup/dead-video-view`. **Analysis only; no removal approved by this document.**

## Confirmed production entry points

| Module | Production caller | Purpose |
| --- | --- | --- |
| `conversation.rs` | `updates.rs:active_work_reason` | `conversation::has_active_generation()` prevents an update while a Home response is active |
| `model_trial.rs` | `updates.rs:active_work_reason` | `model_trial::is_running()` prevents an update while a model trial is active |
| `model_hub.rs` | `lib.rs` shutdown | `model_hub::stop_owned_local_runtimes()` terminates locally owned Ollama runtime processes |
| `project_context.rs` | `conversation.rs` | Builds Home conversation project context |

`conversation.rs` also calls `model_hub.rs`. `model_trial.rs` also calls `model_hub.rs`. This forms a legacy Home AI dependency group rather than independent orphaned files.

## State ownership and safety constraints

- Home generation activity is tracked by `ACTIVE_GENERATIONS: OnceLock<Mutex<HashMap<String, ActiveGeneration>>>` in `conversation.rs`.
- Model trial activity is tracked by `ACTIVE_TRIAL: OnceLock<Mutex<Option<watch::Sender<bool>>>>` in `model_trial.rs`.
- Owned Ollama process state is tracked by `OWNED_OLLAMA: OnceLock<Mutex<Option<OwnedOllama>>>` in `model_hub.rs`.
- Simply deleting either `conversation.rs` or `model_trial.rs` breaks `updates.rs`. Replacing these checks with unconditional `false` would discard safety behavior; do not do that.
- Removing `model_hub.rs` without preserving its owned-process shutdown behavior risks orphaning a runtime.

## Findings and next implementation boundary

The direct `conversation::` and `model_trial::` calls found in `updates.rs` are activity checks. `mcp_server.rs` and `chatgpt_bridge.rs` showed no direct qualified calls to `conversation::`, `model_trial::`, `model_hub::`, or `project_context::` during this audit. This does **not** prove that all indirect or feature-gated references are absent.

The next safe step is to decide whether to **retire the Home AI subsystem as a feature**, rather than trimming arbitrary dead-code warnings. If retiring it, first check all Rust modules, `src/` frontend imports, updater behavior, feature-gated paths, and tests. Design a small dedicated activity/owned-runtime lifecycle module before deleting the larger implementation. Do not weaken updater guards or MCP authorization.

No production code changes were made as part of this audit.
