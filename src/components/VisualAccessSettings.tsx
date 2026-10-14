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
    <section className="r-settings-card">
      <div className="r-settings-heading">
        <div>
          <h3>Ignored applications <abbr title="Applications ignored here are hidden from discovery after restarts. This does not revoke Desktop Access or block MCP control.">ⓘ</abbr></h3>
        </div>
        <button type="button" disabled={busy} onClick={() => void refresh()}>Refresh</button>
      </div>
      {error ? <p className="r-settings-error" role="alert">{error}</p> : null}
      {ignored.length === 0 ? <p className="r-settings-empty">No ignored applications.</p> : (
        <div className="r-settings-apps">
          {ignored.map((entry) => (
            <div className="r-settings-app-row" key={entry.executablePath}>
              <span><strong>{entry.name}</strong><span className="r-settings-app-path">{entry.executablePath}</span></span>
              <button type="button" disabled={busy} onClick={() => void restore(entry.executablePath)}>Restore</button>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
