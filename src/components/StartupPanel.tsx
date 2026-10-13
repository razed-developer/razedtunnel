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
    <section className="home-card">
      <div className="home-section-title"><div><h3>Startup</h3><p>Choose how RazedTunnel starts with Windows.</p></div></div>
      <label>
        <input
          type="checkbox"
          checked={enabled ?? false}
          disabled={busy || enabled === null}
          onChange={(event) => void change(event.target.checked)}
        />
        {" "}Launch RazedTunnel when I sign in
      </label>
      <p>Starts quietly in the system tray. Requires an installed release build on Windows; development builds cannot enable startup.</p>
      {error ? <p role="alert">{error}</p> : null}
    </section>
  );
}
