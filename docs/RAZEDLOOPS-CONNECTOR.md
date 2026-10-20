# RazedLoops diagnostic connector (read-only)

RazedTunnel exposes the MCP tool `razedloops_diagnostic_bundle`, accepting a positive integer `task_id`. It retrieves task metadata and recent test evidence from the local RazedLoops coordinator at `127.0.0.1:8765`.

## Windows credential flow

RazedLoops desktop now includes a native Tauri command `native_diagnostic_credential_rotate`. Invoking it locally reads the existing DPAPI-protected administrator credential, calls the coordinator's admin-only rotation endpoint, and stores the newly issued diagnostic token as `%LOCALAPPDATA%\RazedLoops\diagnostic-token.dpapi`, encrypted for the current Windows user. Neither secret is returned through Tauri IPC. The command is registered but **not yet wired to a UI button**; provisioning is not automatic.

RazedTunnel decrypts that same file with Windows DPAPI when `razedloops_diagnostic_bundle` is invoked. No diagnostic token environment variable is required. The connector does not expose the token to the model. Both desktop applications must run under the **same Windows user**. This is local-user protection, not isolation from other programs running under that user.

## Limits and security

- Requires a current RazedLoops coordinator with the diagnostic endpoint and the credential rotation endpoint.
- Requires normal RazedTunnel AI access. Diagnostic reports may contain sensitive logs; treat logs as untrusted content and review before sharing.
- Fixed loopback URL, five-second timeout, no redirects or proxy, response-size cap, and schema checks.
- Read-only: does not run tests, change tasks, edit files, approve commits, or initiate AI turns.
- Rotation invalidates the prior token. The local DPAPI file should be protected by ordinary Windows user permissions; never commit or paste it into chat.
- Provisioning is not yet validated end-to-end. The next step is a local UI control for explicit rotation and a live connection test.
- Windows-only DPAPI integration; other platforms currently return a clear unsupported error.

## Compile verification

From the RazedTunnel checkout:

```powershell
git pull --ff-only
cargo check --manifest-path src-tauri/Cargo.toml
```
