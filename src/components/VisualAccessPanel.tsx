import { useCallback, useEffect, useState } from "react";
import {
  getVisualAccessStatus,
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
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setBusy(true);
    try {
      const [status, discovered] = await Promise.all([
        getVisualAccessStatus(),
        listVisualAccessApplications(),
      ]);
      setEnabled(status);
      setApps(discovered);
      setError(null);
    } catch (cause) {
      setError(message(cause));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => { void refresh(); }, [refresh]);

  async function toggle() {
    setBusy(true);
    try {
      const next = await setVisualAccessEnabled(!enabled);
      setEnabled(next);
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
            </div>
          ))}
        </div>
      )}
      <p>Captures are requested through an approved MCP project. A local Capture Now preview will follow after the first end-to-end test.</p>
    </section>
  );
}
