import { NavIcon } from "./AppSidebar";
import ActivityFeed from "./ActivityFeed";
import type { ActivityTimeline, ChatConnectionStatus, GatewayStatus, PublicTunnelStatus, Workspace } from "../types";

type Props = {
  gateway: GatewayStatus;
  tunnel: PublicTunnelStatus;
  chat: ChatConnectionStatus;
  workspaces: Workspace[];
  aiAccessPaused: boolean;
  activity: ActivityTimeline | null;
  activityLoading: boolean;
  activityError: string | null;
  onRefreshActivity: () => void;
  onNavigate: (view: "projects" | "connections") => void;
};

export default function HomeDashboard({ gateway, tunnel, chat, workspaces, aiAccessPaused, activity, activityLoading, activityError, onRefreshActivity, onNavigate }: Props) {
  const remoteReady = tunnel.ready || chat.ready;
  const remoteLabel = tunnel.ready ? "Public tunnel ready" : chat.ready ? "Secure connection ready" : "Not connected";
  const totalCalls = gateway.taskToolSuccesses + gateway.taskToolFailures;
  return (
    <div className="home-dashboard">
      <div className="home-intro"><div><span className="home-eyebrow">OVERVIEW</span><h2>RazedTunnel at a glance</h2><p>Monitor your local gateway, remote access, and project permissions.</p></div><span className={`home-runtime ${gateway.running ? "is-running" : ""}`}><span className="home-status-dot" />{gateway.running ? "Gateway running" : "Gateway stopped"}</span></div>
      <div className="home-metrics">
        <section className="home-card home-metric"><div className="home-card-label"><NavIcon name="gateway" />Local gateway</div><strong>{gateway.running ? "Online" : "Offline"}</strong><p>{gateway.running && gateway.port ? `Listening on port ${gateway.port}` : "Not accepting local requests"}</p></section>
        <section className="home-card home-metric"><div className="home-card-label"><NavIcon name="remote" />Remote access</div><strong>{remoteReady ? "Connected" : "Disconnected"}</strong><p>{remoteLabel}</p></section>
        <section className="home-card home-metric"><div className="home-card-label"><NavIcon name="folder" />Projects</div><strong>{workspaces.length}</strong><p>{workspaces.filter((w) => w.accessMode === "readWrite").length} with write access</p></section>
        <section className="home-card home-metric"><div className="home-card-label"><NavIcon name="checks" />MCP tool calls</div><strong>{totalCalls.toLocaleString()}</strong><p>{gateway.taskToolSuccesses} succeeded · {gateway.taskToolFailures} failed</p></section>
      </div>
      <div className="home-detail-grid">
        <section className="home-card home-detail"><div className="home-section-title"><div><h3>Connection health</h3><p>Live configuration and availability</p></div><button type="button" onClick={() => onNavigate("connections")}>Manage connections →</button></div>
          <div className="home-detail-row"><span><NavIcon name="gateway" />Local MCP gateway</span><span className={gateway.running ? "home-good" : "home-muted"}>{gateway.running ? "Running" : "Stopped"}</span></div>
          <div className="home-detail-row"><span><NavIcon name="remote" />Public tunnel</span><span className={tunnel.ready ? "home-good" : "home-muted"}>{tunnel.ready ? "Ready" : tunnel.configured ? "Not ready" : "Not configured"}</span></div>
          <div className="home-detail-row"><span><NavIcon name="shield" />AI project access</span><span className={aiAccessPaused ? "home-attention" : "home-good"}>{aiAccessPaused ? "Paused" : "Enabled"}</span></div>
          <div className="home-detail-row"><span><NavIcon name="remote" />Secure connection</span><span className={chat.ready ? "home-good" : "home-muted"}>{chat.ready ? "Ready" : "Inactive"}</span></div>
        </section>
        <section className="home-card home-detail"><div className="home-section-title"><div><h3>Projects</h3><p>Workspace access overview</p></div><button type="button" onClick={() => onNavigate("projects")}>View projects →</button></div>
          {workspaces.length ? workspaces.slice(0, 5).map((workspace) => <div className="home-detail-row" key={workspace.id}><span><NavIcon name="folder" />{workspace.name}</span><span className="home-muted">{workspace.accessMode === "readWrite" ? "Read & write" : "Read only"}</span></div>) : <div className="home-empty"><NavIcon name="folder" size={24} /><strong>No projects yet</strong><p>Add a project to make it available to connected AI clients.</p><button type="button" onClick={() => onNavigate("projects")}>Set up projects →</button></div>}
        </section>
      </div>
      <ActivityFeed timeline={activity} loading={activityLoading} error={activityError} onRefresh={onRefreshActivity} />
    </div>
  );
}
