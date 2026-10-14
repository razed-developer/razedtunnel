import { useState } from "react";
import type { ChatConnectionStatus, GatewayStatus, PublicTunnelStatus } from "../types";

type GatewayPanelProps = {
  status: GatewayStatus;
  connection: ChatConnectionStatus;
  publicTunnel: PublicTunnelStatus;
  aiAccessPaused: boolean;
  busy: boolean;
  onToggle: () => void;
};

function GatewayPanel({ status, connection, publicTunnel, aiAccessPaused, busy, onToggle }: GatewayPanelProps) {
  const [copied, setCopied] = useState(false);
  const actionLabel = status.running ? "Stop gateway" : "Start gateway";
  const mcpEndpoint = status.running && status.port ? `http://127.0.0.1:${status.port}/mcp` : null;
  const remoteReady = publicTunnel.ready || connection.ready;
  const managedClientState = publicTunnel.ready ? "Endpoint ready; client not confirmed" : connection.ready ? "Tunnel ready; client not confirmed" : connection.running ? "Starting" : "Not connected";
  // The backend increments this counter only after verifying an OAuth bearer token
  // on a forwarded HTTPS MCP request. It does not identify the client or prove tool success.
  const authenticatedRequestObserved = publicTunnel.requestCount > 0;

  async function copyEndpoint() {
    if (!mcpEndpoint) return;
    try {
      await navigator.clipboard.writeText(mcpEndpoint);
    } catch {
      const textarea = document.createElement("textarea");
      textarea.value = mcpEndpoint;
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

  return (
    <section className="r-home-card r-gateway-overview" aria-labelledby="gateway-title">
      <div className="r-home-section-title r-gateway-heading">
        <div>
          <h3 id="gateway-title">Local Gateway <abbr title="The local MCP endpoint is available only on this computer. Approved workspace requests are checked against RazedTunnel's access policy.">ⓘ</abbr></h3>
        </div>
        <span className={status.running ? "r-home-good" : "r-home-muted"} role="status">
          <span aria-hidden="true">{status.running ? "● " : "○ "}</span>
          {status.running ? "Running" : "Offline"}
        </span>
      </div>

      <div className="r-gateway-endpoint">
        <div>
          <span className="r-home-muted">Local MCP endpoint</span>
          {mcpEndpoint ? <code>{mcpEndpoint}</code> : <span className="r-home-muted">Start the gateway to enable the endpoint</span>}
        </div>
        {mcpEndpoint ? <button type="button" className="r-gateway-copy" onClick={() => void copyEndpoint()}>{copied ? "Copied" : "Copy"}</button> : null}
      </div>

      <div className="r-gateway-metrics">
        <div><span className="r-home-muted">Projects</span><strong>{status.workspaceCount}</strong></div>
        <div><span className="r-home-muted">Remote endpoint <abbr title="Ready means the tunnel reports online, not that a remote MCP client has connected.">ⓘ</abbr></span><strong>{remoteReady ? "Ready" : "Not ready"}</strong></div>
        <div><span className="r-home-muted">AI access</span><strong className={aiAccessPaused ? "r-home-attention" : "r-home-good"}>{aiAccessPaused ? "Paused" : "Enabled"}</strong></div>
      </div>

      <div className="r-gateway-footer">
        <details className="r-gateway-diagnostics">
          <summary>Diagnostics</summary>
          <div className="r-home-detail-row"><span>Local gateway</span><strong>{status.running ? "Running" : "Offline"}</strong></div>
          <div className="r-home-detail-row"><span>Remote connection</span><strong>{publicTunnel.ready ? "Managed public MCP" : connection.running ? "OpenAI Secure Tunnel" : "None"}</strong></div>
          <p className="r-home-muted">{managedClientState}</p>
          <div className="r-home-detail-row"><span>Authenticated MCP access <abbr title="Counts OAuth-authenticated remote MCP requests this launch; this does not prove a tool executed successfully or identify the client.">ⓘ</abbr></span><strong>{authenticatedRequestObserved ? "Observed" : "Not observed"}</strong></div>
          <p className="r-home-muted">{publicTunnel.requestCount} authenticated remote MCP requests this launch.</p>
          <div className="r-home-detail-row"><span>Instrumented MCP tasks <abbr title="Partial coverage: filesystem and structured task wrappers only, across local and remote clients this launch.">ⓘ</abbr></span><strong>{status.taskToolSuccesses} succeeded · {status.taskToolFailures} failed</strong></div>
          <div className="r-home-detail-row"><span>Workspace access</span><strong>{aiAccessPaused ? "Paused" : "Available"}</strong></div>
          <p className="r-home-muted">Direct MCP clients can use the local endpoint. Generic HTTP clients may not provide a reliable client identity.</p>
        </details>
        <button className={`r-gateway-toggle ${status.running ? "r-gateway-stop" : ""}`} type="button" onClick={onToggle} disabled={busy}>{busy ? "Working…" : actionLabel}</button>
      </div>
    </section>
  );
}

export default GatewayPanel;
