import { useEffect, useState } from "react";
import { getRuntimeDiagnostics, setLaunchAtLogin } from "../lib/backend";

export default function StartupPanel() {
  const [enabled, setEnabled] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let mounted = true;
    getRuntimeDiagnostics()
      .then((diagnostics) => { if (mounted) setEnabled(diagnostics.launchAtLogin); })
      .catch((cause: unknown) => { if (mounted) setError(String(cause)); });
    return () => { mounted = false; };
  }, []);

  async function change(enabledNext: boolean) {
    setBusy(true);
    setError(null);
    try {
      const diagnostics = await setLaunchAtLogin(enabledNext);
      setEnabled(diagnostics.launchAtLogin);
    } catch (cause) {
      setError(String(cause));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="r-settings-card">
      <div className="r-settings-heading"><div><h3>Startup <abbr title="Launch at sign-in starts quietly in the tray and requires an installed release build.">ⓘ</abbr></h3></div></div>
      <label className="r-settings-toggle">
        <input
          type="checkbox"
          checked={enabled ?? false}
          disabled={busy || enabled === null}
          onChange={(event) => void change(event.target.checked)}
        />
        {" "}Launch RazedTunnel when I sign in
      </label>
      {error ? <p className="r-settings-error" role="alert">{error}</p> : null}
    </section>
  );
}
