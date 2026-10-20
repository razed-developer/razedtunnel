# RazedLoops diagnostic connector (initial read-only integration)

RepoTunnel exposes an opt-in MCP tool named `razedloops_diagnostic_bundle` with a positive integer `task_id`. It fetches the diagnostic bundle from a **local** RazedLoops coordinator at `127.0.0.1:8765`.

## Current limitations
- Requires RazedLoops coordinator with `GET /tasks/{id}/diagnostic-bundle` (newer than the 9-test milestone).
- Disabled unless the RepoTunnel desktop **process** is launched with `RAZEDTUNNEL_RAZEDLOOPS_ADMIN_TOKEN` set to the coordinator's existing administrator token. The token is **not** supplied as an MCP tool parameter, returned to the model, or logged by this tool.
- Do not place the token in the repository, scripts committed to Git, screenshots, or chat. Environment variables are an interim opt-in mechanism, **not** the final credential storage solution. Prefer OS-backed secret storage before enabling this in routine use.
- The tool requires normal RepoTunnel AI access and returns at most the coordinator's five most recent reports; reports may include sensitive output. Review logs before sharing.
- Fixed loopback URL, 5-second timeout, no HTTP redirects, no proxy, response size cap, and version/schema check.
- Read-only: cannot trigger tests, change task state, edit files, or commit. Does not initiate ChatGPT turns.
- No live runtime verification has yet been performed. Rebuild RepoTunnel and refresh MCP tool definitions after verification.

## Validation
```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

Run only after pulling the changes in the RazedTunnel checkout. A future revision should replace environment configuration with OS-protected credential retrieval, and preferably a scoped read-only coordinator credential instead of the administrator token.
