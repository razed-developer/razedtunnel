import { useCallback, useEffect, useState } from "react";
import {
  captureVisualAccessWindow,
  type VisualAccessScreenshot,
  getVisualAccessApplicationIdentity,
  getVisualAccessStatus,
  listIgnoredVisualApplications,
  setVisualApplicationIgnored,
  type IgnoredVisualApplication,
  listVisualAccessApplications,
  setVisualAccessEnabled,
  type VisualAccessApplication,
} from "../lib/backend";

function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export default function VisualAccessPanel() {
  const [enabled, setEnabled] = useState(false);
  const [apps, setApps] = useState<VisualAccessApplication[]>([]);
  const [ignoredApps, setIgnoredApps] = useState<IgnoredVisualApplication[]>([]);
  const [showIgnored, setShowIgnored] = useState(false);
  const [busy, setBusy] = useState(false);
  const [capturingId, setCapturingId] = useState<string | null>(null);
  const [preview, setPreview] = useState<VisualAccessScreenshot | null>(null);
  const [previewName, setPreviewName] = useState("");
  const [identity, setIdentity] = useState<{ name: string; path: string } | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setBusy(true);
    try {
      const [status, discovered, ignored] = await Promise.all([
        getVisualAccessStatus(),
        listVisualAccessApplications(),
        listIgnoredVisualApplications(),
      ]);
      setEnabled(status);
      setApps(discovered);
      setIgnoredApps(ignored);
      setError(null);
    } catch (cause) {
      setError(message(cause));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => { void refresh(); }, [refresh]);

  async function changeIgnored(args: { applicationId?: string; executablePath?: string; name?: string; ignored: boolean }) {
    setBusy(true);
    try {
      const updated = await setVisualApplicationIgnored(args);
      setIgnoredApps(updated);
      setApps(await listVisualAccessApplications());
      setError(null);
    } catch (cause) {
      setError(message(cause));
    } finally {
      setBusy(false);
    }
  }

  async function inspectIdentity(app: VisualAccessApplication) {
    setBusy(true);
    setError(null);
    try {
      const path = await getVisualAccessApplicationIdentity(app.id);
      setIdentity({ name: app.name, path });
    } catch (cause) {
      setError(`Could not identify ${app.name}: ${message(cause)}`);
    } finally {
      setBusy(false);
    }
  }

  async function capture(app: VisualAccessApplication) {
    setCapturingId(app.id);
    setPreview(null);
    setError(null);
    try {
      const result = await captureVisualAccessWindow(app.id);
      if (result.mimeType !== "image/png") throw new Error("Unexpected capture format.");
      setPreview(result);
      setPreviewName(app.name);
    } catch (cause) {
      setError(`Could not capture ${app.name}: ${message(cause)}`);
    } finally {
      setCapturingId(null);
    }
  }

  async function toggle() {
    setBusy(true);
    try {
      const next = await setVisualAccessEnabled(!enabled);
      setEnabled(next);
      if (!next) setPreview(null);
      setApps(await listVisualAccessApplications());
      setError(null);
    } catch (cause) {
      setError(message(cause));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="home-card visual-access-panel">
      <div className="home-section-title">
        <div>
          <h3>Visual Access</h3>
          <p>Windows application discovery and screenshot permission</p>
        </div>
        <button type="button" disabled={busy} onClick={() => void refresh()}>Refresh apps</button>
      </div>
      <p>Desktop Access is currently a global permission for all approved projects.
        Enabling it permits application interaction as well as screenshots; it is not view-only.</p>
      <div className="home-detail-row">
        <span>Desktop Access</span>
        <button type="button" disabled={busy} onClick={() => void toggle()}
          aria-pressed={enabled}>{busy ? "Working…" : enabled ? "Enabled — turn off" : "Disabled — enable"}</button>
      </div>
      {error ? <p role="alert">{error}</p> : null}
      <h4>Detected applications ({apps.length})</h4>
      {apps.length === 0 ? <p>No applications detected. Open Notepad and select Refresh apps.</p> : (
        <div className="visual-access-apps">
          {apps.map((app) => (
            <div className="home-detail-row" key={app.id}>
              <span>{app.name}</span>
              <span className="home-muted">{app.windowCount} window{app.windowCount === 1 ? "" : "s"} · {app.accessibility ? "Accessible" : "Limited accessibility"}</span>
              <button type="button" disabled={busy || capturingId !== null} onClick={() => void changeIgnored({ applicationId: app.id, name: app.name, ignored: true })}>Ignore</button>
              <button type="button" disabled={busy || capturingId !== null} onClick={() => void inspectIdentity(app)}>Identify</button>
              <button type="button" disabled={!enabled || busy || capturingId !== null} onClick={() => void capture(app)}>{capturingId === app.id ? "Capturing…" : "Capture Now"}</button>
            </div>
          ))}
        </div>
      )}
      <div className="visual-access-ignored-settings">
        <button type="button" aria-expanded={showIgnored} onClick={() => setShowIgnored(!showIgnored)}>{showIgnored ? "Hide" : "Manage"} ignored apps ({ignoredApps.length})</button>
        {showIgnored ? <div><h4>Ignored applications</h4><p>Hidden from this Visual Access list, including after application restarts. This does not revoke Desktop Access permissions.</p>{ignoredApps.length === 0 ? <p>No ignored applications.</p> : ignoredApps.map((entry) => <div className="home-detail-row" key={entry.executablePath}><span>{entry.name} <small style={{ overflowWrap: "anywhere" }}>{entry.executablePath}</small></span><button type="button" disabled={busy} onClick={() => void changeIgnored({ executablePath: entry.executablePath, ignored: false })}>Restore</button></div>)}</div> : null}
      </div>
      {identity ? <p><strong>Executable identity for {identity.name}:</strong> <code style={{ overflowWrap: "anywhere" }}>{identity.path}</code></p> : null}
      {preview ? <div className="visual-access-preview"><h4>Preview: {previewName}</h4><p>{preview.width} × {preview.height} · {(preview.sizeBytes / 1024).toFixed(0)} KB · Local window capture</p><img src={`data:image/png;base64,${preview.dataBase64}`} alt={`Captured window: ${previewName}`} style={{ display: "block", width: "100%", maxWidth: 900, height: "auto", borderRadius: 8, border: "1px solid var(--border, #d1d5db)" }} /><button type="button" onClick={() => setPreview(null)}>Clear preview</button></div> : null}
      <p>Captures are on demand and shown only in this local preview. No background recording or desktop fallback.</p>
    </section>
  );
}
