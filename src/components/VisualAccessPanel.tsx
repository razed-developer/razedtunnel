import { useCallback, useEffect, useState } from "react";
import {
  captureVisualAccessWindow,
  type VisualAccessScreenshot,
  getVisualAccessApplicationIdentity,
  getVisualAccessStatus,
  listIgnoredVisualApplications,
  listVisualAccessWindows,
  type VisualAccessWindow,
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
  const [windows, setWindows] = useState<VisualAccessWindow[]>([]);
  const [expandedApp, setExpandedApp] = useState<string | null>(null);
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
      const [status, discovered, ignored, discoveredWindows] = await Promise.all([
        getVisualAccessStatus(),
        listVisualAccessApplications(),
        listIgnoredVisualApplications(),
        listVisualAccessWindows(),
      ]);
      setEnabled(status);
      setApps(discovered);
      setIgnoredApps(ignored);
      setWindows(discoveredWindows);
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
    <section className="r-visual-panel">
      <div className="r-home-section-title">
        <div>
          <h3>Visual Access <abbr title="Desktop Access is global for approved projects and allows interaction as well as screenshots. Captures are on demand, with no background recording or desktop fallback.">ⓘ</abbr></h3>
          
        </div>
        <button type="button" disabled={busy} onClick={() => void refresh()}>Refresh apps</button>
      </div>
      
      <div className="r-home-detail-row">
        <span>Desktop Access <abbr title="Enabling Desktop Access permits control as well as screenshots for approved projects.">ⓘ</abbr></span>
        <button type="button" disabled={busy} onClick={() => void toggle()}
          aria-pressed={enabled}>{busy ? "Working…" : enabled ? "Enabled — turn off" : "Disabled — enable"}</button>
      </div>
      {error ? <p role="alert">{error}</p> : null}
      <h4>Detected applications ({apps.length})</h4>
      {apps.length === 0 ? <p>No applications detected. Open Notepad and select Refresh apps.</p> : (
        <div className="r-visual-apps">
          {apps.map((app) => (
            <div className="r-visual-app-row" key={app.id}>
            <div className="r-home-detail-row">
              <span>{app.name}</span>
              <span className="r-home-muted">{app.windowCount} window{app.windowCount === 1 ? "" : "s"} · {app.accessibility ? "Accessible" : "Limited accessibility"}</span>
              <button type="button" aria-expanded={expandedApp === app.id} onClick={() => setExpandedApp(expandedApp === app.id ? null : app.id)}>Windows ({windows.filter((entry) => entry.applicationId === app.id).length})</button>
              <button type="button" disabled={busy || capturingId !== null} onClick={() => void changeIgnored({ applicationId: app.id, name: app.name, ignored: true })}>Ignore</button>
              <button type="button" disabled={busy || capturingId !== null} onClick={() => void inspectIdentity(app)}>Identify</button>
              <button type="button" disabled={!enabled || busy || capturingId !== null} onClick={() => void capture(app)}>{capturingId === app.id ? "Capturing…" : "Capture Now"}</button>
            </div>
            {expandedApp === app.id ? <div className="r-visual-app-windows">{windows.filter((entry) => entry.applicationId === app.id).map((entry, index) => <p key={`${entry.applicationId}-${index}`}>{entry.title}</p>)}</div> : null}
            </div>
          ))}
        </div>
      )}
      <div className="visual-access-ignored-settings">
        <button type="button" aria-expanded={showIgnored} onClick={() => setShowIgnored(!showIgnored)}>{showIgnored ? "Hide" : "Manage"} ignored apps ({ignoredApps.length})</button>
        {showIgnored ? <div><h4>Ignored applications <abbr title="Ignored applications stay hidden after restarts; this does not revoke Desktop Access.">ⓘ</abbr></h4>{ignoredApps.length === 0 ? <p>No ignored applications.</p> : ignoredApps.map((entry) => <div className="r-home-detail-row" key={entry.executablePath}><span>{entry.name} <small style={{ overflowWrap: "anywhere" }}>{entry.executablePath}</small></span><button type="button" disabled={busy} onClick={() => void changeIgnored({ executablePath: entry.executablePath, ignored: false })}>Restore</button></div>)}</div> : null}
      </div>
      {identity ? <p><strong>Executable identity for {identity.name}:</strong> <code style={{ overflowWrap: "anywhere" }}>{identity.path}</code></p> : null}
      {preview ? <div className="visual-access-preview"><h4>Preview: {previewName}</h4><p>{preview.width} × {preview.height} · {(preview.sizeBytes / 1024).toFixed(0)} KB · Local window capture</p><img src={`data:image/png;base64,${preview.dataBase64}`} alt={`Captured window: ${previewName}`} style={{ display: "block", width: "100%", maxWidth: 900, height: "auto", borderRadius: 8, border: "1px solid var(--border, #d1d5db)" }} /><button type="button" onClick={() => setPreview(null)}>Clear preview</button></div> : null}
    </section>
  );
}
