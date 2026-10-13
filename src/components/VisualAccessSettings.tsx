import { useCallback, useEffect, useState } from "react";
import {
  listIgnoredVisualApplications,
  setVisualApplicationIgnored,
  type IgnoredVisualApplication,
} from "../lib/backend";

export default function VisualAccessSettings() {
  const [ignored, setIgnored] = useState<IgnoredVisualApplication[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setIgnored(await listIgnoredVisualApplications());
      setError(null);
    } catch (cause) {
      setError(String(cause));
    }
  }, []);

  useEffect(() => { void refresh(); }, [refresh]);

  async function restore(path: string) {
    setBusy(true);
    try {
      setIgnored(await setVisualApplicationIgnored({ executablePath: path, ignored: false }));
      setError(null);
    } catch (cause) {
      setError(String(cause));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="home-card">
      <div className="home-section-title">
        <div>
          <h3>Ignored applications <abbr title="Applications ignored here are hidden from discovery after restarts. This does not revoke Desktop Access or block MCP control.">ⓘ</abbr></h3>
        </div>
        <button type="button" disabled={busy} onClick={() => void refresh()}>Refresh</button>
      </div>
      {error ? <p role="alert">{error}</p> : null}
      {ignored.length === 0 ? <p>No ignored applications.</p> : (
        <div className="visual-access-apps">
          {ignored.map((entry) => (
            <div className="home-detail-row" key={entry.executablePath}>
              <span><strong>{entry.name}</strong><span className="home-muted" style={{ display: "block", overflowWrap: "anywhere" }}>{entry.executablePath}</span></span>
              <button type="button" disabled={busy} onClick={() => void restore(entry.executablePath)}>Restore</button>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
