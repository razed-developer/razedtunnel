# Visual Access: Windows implementation plan

## Current verified behavior

The existing `desktop_control::screenshot` enforces `require_enabled(app, workspace_id, application_id)` before dispatch. Linux captures approved application windows. The Windows branch currently returns a not-implemented error; Windows UI Automation supports inspection and semantic interaction, **not yet image capture**. Managed browser and isolated AI Workspace screenshots are separate features.

## Milestone 1: approved-window capture on Windows

- Reuse the existing workspace/application permission check. Never accept arbitrary HWND or PID from MCP callers.
- Resolve an approved `uia-pid-...` application ID and optional short-lived `uia-window-...` identity against current UI Automation windows. Revalidate ownership immediately before capture; reject stale window IDs, protected RazedTunnel windows, invisible windows and cross-process targets.
- Capture only the selected app-owned window, not the full desktop. Prefer Windows Graphics Capture or an equivalent window-scoped API; fail closed if capture would include other windows, protected content or the screen.
- Enforce dimensions, output byte limits, and timeouts. Return the existing `DesktopScreenshot` image payload as PNG via the established MCP image response path.
- Do not persist frames, stream images, or expose the output to other projects. Keep screenshot request logging without logging image contents.
- Add Windows-only tests for window selection, permission denial, stale IDs, and capture errors; validate on the user's Windows PC.

## Milestone 2: explicit temporary observation

- Separate opt-in, named-window session with clear visible recording indicator and immediate Pause/Stop.
- 60-second RAM-only bounded ring buffer; expire by time and memory cap; purge on permission revocation, session end, window change, or app exit.
- Capture rate and resolution bounded; no default background recording and no persistence. No access to other windows, system desktop, credentials, or notification overlays.
- Only an authorized tool invocation can retrieve selected recent frames; no automatic transmission to ChatGPT.

## UI

Add a small Visual Access panel after the Windows capture path is functional. It should show the selected approved window, current permission, capture indicator, and Capture Now. Do not show enabled capture controls on Windows before actual support exists.
