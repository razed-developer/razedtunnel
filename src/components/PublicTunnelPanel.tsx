import { FormEvent, useEffect, useMemo, useState } from "react";
import type { PublicTunnelProvider, PublicTunnelStatus } from "../types";

type PublicTunnelPanelProps = {
  status: PublicTunnelStatus;
  gatewayRunning: boolean;
  busy: boolean;
  onConfigure: (provider: PublicTunnelProvider, credential: string, publicUrl?: string) => Promise<void>;
  onRestart: () => Promise<void>;
  onProvisionCertificate: () => Promise<void>;
  onRevoke: () => Promise<void>;
  onForget: () => Promise<void>;
};

function relativeTime(timestamp: number | null): string {
  if (!timestamp) return "No remote request seen yet";
  const diff = Math.max(0, Date.now() - timestamp);
  if (diff < 10_000) return "Remote request seen just now";
  if (diff < 60_000) return `Remote request seen ${Math.floor(diff / 1000)}s ago`;
  if (diff < 3_600_000) return `Remote request seen ${Math.floor(diff / 60_000)}m ago`;
  return `Remote request seen ${Math.floor(diff / 3_600_000)}h ago`;
}

function providerLabel(provider: PublicTunnelProvider): string {
  if (provider === "cloudflare") return "Cloudflare";
  if (provider === "direct") return "Direct HTTPS";
  return "ngrok";
}

function PublicTunnelPanel({
  status,
  gatewayRunning,
  busy,
  onConfigure,
  onRestart,
  onProvisionCertificate,
  onRevoke,
  onForget,
}: PublicTunnelPanelProps) {
  const [provider, setProvider] = useState<PublicTunnelProvider>(status.provider);
  const [credential, setCredential] = useState("");
  const [publicUrl, setPublicUrl] = useState("");
  const [copied, setCopied] = useState(false);
  const activity = useMemo(() => relativeTime(status.lastRemoteRequestAt), [status.lastRemoteRequestAt]);
  const reconnecting = status.running && !status.ready && /reconnect|interrupt/i.test(status.message ?? "");
  const publicStateLabel = status.ready
    ? "Ready"
    : status.running
      ? (status.provider === "direct" ? "Local listener on" : reconnecting ? "Reconnecting" : "Connecting")
      : status.configured
        ? "Offline"
        : "Setup needed";
  const showingConfiguredProvider = status.configured && provider === status.provider;

  useEffect(() => {
    if (status.configured) setProvider(status.provider);
  }, [status.configured, status.provider]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    try {
      await onConfigure(
        provider,
        provider === "direct" ? "" : credential.trim(),
        provider === "cloudflare" || provider === "direct" ? publicUrl.trim() : undefined,
      );
      setCredential("");
      setPublicUrl("");
    } catch {
      // Keep fields so a failed setup can be corrected without retyping them.
    }
  }

  async function copyMcpUrl() {
    if (!status.mcpUrl) return;
    try {
      await navigator.clipboard.writeText(status.mcpUrl);
    } catch {
      const textarea = document.createElement("textarea");
      textarea.value = status.mcpUrl;
      textarea.style.position = "fixed";
      textarea.style.opacity = "0";
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      textarea.remove();
    }
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  }

  const credentialValid = credential.trim().length >= 20;
  const publicUrlValid = /^https:\/\/[^/\s]+\/?$/i.test(publicUrl.trim());
  const canSubmit = provider === "direct"
    ? publicUrlValid
    : credentialValid && (provider === "ngrok" || publicUrlValid);

  return (
    <section className="connection-panel public-tunnel-panel home-card public-overview" aria-labelledby="public-tunnel-title">
      <div className="connection-heading">
        <div>
          
          <h2 id="public-tunnel-title">Public MCP Endpoint <abbr title="Secure remote access to the localhost MCP gateway. Direct HTTPS avoids a relay; ngrok and Cloudflare are alternatives.">ⓘ</abbr></h2>

        </div>
        <span className={`connection-state ${status.ready ? "ready" : status.running ? "starting" : ""}`}>
          <span className="status-dot" aria-hidden="true" />
          {showingConfiguredProvider ? publicStateLabel : "Switch provider"}
        </span>
      </div>

      <div className="r-public-provider-switch" role="group" aria-label="Public connection provider">
        {(["direct", "ngrok", "cloudflare"] as const).map((option) => (
          <button
            key={option}
            type="button"
            className={provider === option ? "active" : ""}
            onClick={() => setProvider(option)}
            disabled={busy}
          >
            {providerLabel(option)}
            {status.configured && status.provider === option ? <span>Current</span> : null}
          </button>
        ))}
      </div>

      {!showingConfiguredProvider ? (
        <form className="r-public-provider-form" onSubmit={handleSubmit}>
          {provider === "direct" ? (
            <>
              <div className="r-public-callout">
                <strong>Direct HTTPS · no relay provider</strong>
                <p>
                  RepoTunnel terminates HTTPS on this computer and proxies only to its loopback MCP gateway.
                  It never needs root: map router public <code>443</code> to local <code>{status.directHttpsPort}</code>.
                  For Let&apos;s Encrypt HTTP validation, map public <code>80</code> to local <code>{status.directHttpChallengePort}</code>.
                </p>
                <p>
                  You can enable the local stack now even behind CGNAT. Public access becomes usable as soon as
                  your ISP supplies a routable IPv4/IPv6 and the router/firewall path is open.
                </p>
              </div>
              <label className="r-public-field" htmlFor="direct-public-url">
                Public HTTPS address
                <input
                  id="direct-public-url"
                  type="url"
                  autoComplete="off"
                  spellCheck={false}
                  placeholder="https://203.0.113.10"
                  value={publicUrl}
                  onChange={(event) => setPublicUrl(event.target.value)}
                  disabled={busy}
                  required
                />
                <small>Use your routable IPv4, IPv6, or hostname. No RepoTunnel relay credential is needed.</small>
              </label>
            </>
          ) : provider === "ngrok" ? (
            <>
              <div className="r-public-callout">
                <strong>ngrok · fastest hosted fallback</strong>
                <p>
                  Paste your own ngrok authtoken once. RepoTunnel embeds the connection and reuses the assigned
                  public address across normal restarts.
                </p>
                <a href="https://dashboard.ngrok.com/get-started/your-authtoken" target="_blank" rel="noreferrer">
                  Open ngrok authtoken page
                </a>
              </div>
              <label className="r-public-field" htmlFor="public-provider-credential">
                ngrok authtoken
                <input
                  id="public-provider-credential"
                  type="password"
                  autoComplete="off"
                  spellCheck={false}
                  placeholder="Paste your ngrok authtoken"
                  value={credential}
                  onChange={(event) => setCredential(event.target.value)}
                  disabled={busy}
                  required
                />
              </label>
            </>
          ) : (
            <>
              <div className="r-public-callout">
                <strong>Cloudflare Tunnel · stable named fallback</strong>
                <p>
                  Create a named Cloudflare Tunnel and published hostname, then point its Service URL to
                  <code> http://localhost:{status.cloudflareOriginPort}</code>. RepoTunnel runs <code>cloudflared</code>
                  with the token in its environment, never on the command line.
                </p>
                <div className="r-public-provider-links">
                  <a href="https://developers.cloudflare.com/tunnel/downloads/" target="_blank" rel="noreferrer">Install cloudflared</a>
                  <a href="https://developers.cloudflare.com/tunnel/setup/" target="_blank" rel="noreferrer">Cloudflare setup guide</a>
                  <a href="https://dash.cloudflare.com/" target="_blank" rel="noreferrer">Cloudflare dashboard</a>
                </div>
                <small className={status.cloudflaredAvailable ? "r-public-provider-ready" : "r-public-provider-warning"}>
                  {status.cloudflaredAvailable
                    ? "cloudflared detected on this computer."
                    : "cloudflared is not detected yet. Install it before connecting."}
                </small>
              </div>
              <label className="r-public-field" htmlFor="public-provider-credential">
                Cloudflare Tunnel token
                <input
                  id="public-provider-credential"
                  type="password"
                  autoComplete="off"
                  spellCheck={false}
                  placeholder="Paste the tunnel token"
                  value={credential}
                  onChange={(event) => setCredential(event.target.value)}
                  disabled={busy}
                  required
                />
              </label>
              <label className="r-public-field" htmlFor="cloudflare-public-url">
                Public hostname
                <input
                  id="cloudflare-public-url"
                  type="url"
                  autoComplete="off"
                  spellCheck={false}
                  placeholder="https://repotunnel.example.com"
                  value={publicUrl}
                  onChange={(event) => setPublicUrl(event.target.value)}
                  disabled={busy}
                  required
                />
              </label>
            </>
          )}

          <div className="r-public-form-footer">
            <span>
              {status.configured
                ? `Switch from ${providerLabel(status.provider)} only after the new local connection starts.`
                : provider === "direct"
                  ? "No tunnel account or relay credential required."
                  : "No RepoTunnel developer credential is bundled."}
            </span>
            <button
              className="r-public-submit"
              type="submit"
              disabled={busy || !canSubmit || (provider === "cloudflare" && !status.cloudflaredAvailable)}
            >
              {busy ? "Connecting…" : status.configured ? `Switch to ${providerLabel(provider)}` : "Set up & connect"}
            </button>
          </div>
        </form>
      ) : (
        <div className="r-public-connected">
          <div className="r-public-health-grid">
            <div><span>Provider</span><strong>{providerLabel(status.provider)}</strong></div>
            <div><span>Local gateway</span><strong>{gatewayRunning ? "Online" : "Offline"}</strong></div>
            {status.provider === "direct" ? (
              <>
                <div><span>HTTPS listener</span><strong>{status.localReady ? `:${status.directHttpsPort} online` : status.running ? "Starting / unavailable" : "Offline"}</strong></div>
                <div><span>TLS certificate</span><strong>{status.tlsTrusted ? "Trusted" : "Self-signed test"}</strong></div>
                <div><span>Public route</span><strong>{status.publicReachable ? "Reachable" : "Not confirmed"}</strong></div>
              </>
            ) : (
              <div><span>Public tunnel</span><strong>{status.ready ? "Verified online" : status.running ? (reconnecting ? "Reconnecting" : "Connecting") : "Offline"}</strong></div>
            )}
            <div><span>Remote MCP requests</span><strong>{status.requestCount}</strong></div>
          </div>

          {status.mcpUrl ? (
            <div className="r-public-mcp-endpoint">
              <div>
                <span>{status.provider === "direct" ? "Direct MCP URL" : "Stable MCP URL"}</span>
                <code>{status.mcpUrl}</code>
                <small>{activity}</small>
              </div>
              <button type="button" className="r-public-copy" onClick={() => void copyMcpUrl()}>
                {copied ? "Copied" : "Copy"}
              </button>
            </div>
          ) : null}

          {status.provider === "direct" ? (
            <details className="public-overview-details">
              <summary>Direct HTTPS setup and certificate</summary>
              <div className="r-public-callout">
              <strong>Direct network path</strong>
              <p>
                Router mappings: public <code>443 → {status.directHttpsPort}</code> and public <code>80 → {status.directHttpChallengePort}</code>
                on this computer. RepoTunnel&apos;s actual MCP gateway remains localhost-only on <code>{status.originPort ?? status.cloudflareOriginPort}</code>.
              </p>
              <small className={status.tlsTrusted ? "r-public-provider-ready" : "r-public-provider-warning"}>
                {status.tlsTrusted
                  ? "A trusted certificate is loaded."
                  : status.certbotAvailable
                    ? "Local TLS is working with a self-signed test certificate. Get the trusted IP certificate only after public port 80 reaches this computer."
                    : `Local TLS is working with a self-signed test certificate. Certbot 5.4+ is not detected${status.certbotVersion ? ` (${status.certbotVersion})` : ""}; install it before requesting a Let's Encrypt IP certificate.`}
              </small>
              {!status.tlsTrusted && status.certbotAvailable ? (
                <div className="r-public-provider-links">
                  <button className="secondary-button" type="button" onClick={() => void onProvisionCertificate()} disabled={busy || !status.localReady}>
                    {busy ? "Working…" : "Get trusted IP certificate"}
                  </button>
                </div>
              ) : null}
              </div>
            </details>
          ) : null}

          <div className="public-usage-strip">
            <div>
              <span>{status.provider === "direct" ? "Direct usage" : "Usage status"}</span>
              <strong>{status.usageLabel}</strong>
            </div>
            <a href={status.usageUrl} target="_blank" rel="noreferrer">
              {status.provider === "direct" ? "Let's Encrypt" : "Open provider usage"}
            </a>
          </div>

          <details className="public-overview-details">
            <summary>Connection details <abbr title="Ready does not prove an external client connected. Request counts do not confirm tool execution.">ⓘ</abbr></summary>
            <div className="connection-next-step">
            <strong>
              {status.provider === "direct"
                ? status.ready ? "Direct HTTPS is confirmed" : "Direct HTTPS is prepared locally"
                : status.ready ? "Ready for ChatGPT or another MCP client" : "Connection needs attention"}
            </strong>
            <p>
              {status.message ?? (status.ready
                ? `RepoTunnel will reuse this ${providerLabel(status.provider)} endpoint on normal restarts.`
                : "Use Restart connection if setup needs attention.")}
            </p>
          </div>
          </details>

          <div className="public-connection-actions public-dashboard-actions">
            <button className="public-action-button public-action-restart" type="button" onClick={() => void onRestart()} disabled={busy}>
              {busy ? "Working…" : "Restart connection"}
            </button>
            <button className="public-action-button public-action-danger" type="button" onClick={() => void onRevoke()} disabled={busy}>
              Revoke MCP access
            </button>
            <button className="public-action-button public-action-danger" type="button" onClick={() => void onForget()} disabled={busy}>
              Forget public setup
            </button>
          </div>
        </div>
      )}
    </section>
  );
}

export default PublicTunnelPanel;
