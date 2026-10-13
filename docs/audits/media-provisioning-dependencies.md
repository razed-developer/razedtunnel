# FFmpeg provisioning dependency review (2026-10-13)

Scope: `src-tauri/src/media_tools.rs` and its current `media_inspection.rs` caller. Static source review, not proof of whole-repository reachability.

## Active discovery path

`media_inspection.rs` calls `media_tools::available_ffprobe_program` once and `media_tools::available_ffmpeg_program` twice. Both call `discover_program`, which looks for executables under the application's private `video-tools` data directory first, then on `PATH`. The shared `executable_name` helper adds `.exe` on Windows. **Preserve this path.**

## Disconnected provisioning path (compiler-reported unused)

```text
ensure_ffmpeg_program
  -> available_ffmpeg_program
  -> install_ffmpeg
       -> available_ffmpeg_program
       -> download_client -> trusted_helper_host (redirect policy)
       -> download_bytes -> trusted_helper_host (URL validation)
       -> wheel_matches_platform
       -> sha256_hex
       -> extract_ffmpeg_wheel_binary
       -> managed_tools_root
       -> executable_name
       -> private_write
       -> program_version
  -> available_ffmpeg_program
```

`install_ffmpeg` fetches PyPI `imageio-ffmpeg` metadata and a matching wheel, enforces HTTPS and host restrictions, checks SHA-256, extracts an FFmpeg executable, writes it to private app data, and verifies execution. Its support functions and constants appear unused because the **entry** `ensure_ffmpeg_program` is unused, not because they are independent dead leaves. The code contains meaningful safety checks; do not strip individual safeguards.

## Boundary and recommendation

Current media inspection **discovers** FFmpeg/FFprobe but does not invoke the provisioning entry point. Do not remove `media_tools.rs` or the discovery helpers. Before deleting or relocating provisioning, check all module references, any future/feature-gated phone and media workflows, tests, and whether managed installation is a desired capability. Prefer an explicit decision to retain, move behind a feature, or remove the *whole provisioning cluster* with its dependencies. No runtime change is authorized by this report.

Audit limitation: the textual function audit can show `ensure_ffmpeg_program` as a candidate even while its internal helpers have references. That is an entry-point signal, not a declaration that the helper cluster is live.
