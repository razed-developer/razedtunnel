import type {
  ChatConnectionStatus,
  GatewayStatus,
  HomeContextErrorInput,
  HomeContextTextInput,
  ModelHubSnapshot,
  PublicTunnelStatus,
  Workspace,
} from "../types";
import type { AppView, IconName } from "./AppSidebar";
import { NavIcon } from "./AppSidebar";
import HomeChat from "./HomeChat";
import { useState } from "react";

type HomeWorkspaceProps = {
  gateway: GatewayStatus;
  connection: ChatConnectionStatus;
  publicTunnel: PublicTunnelStatus;
  workspaces: Workspace[];
  selectedWorkspace: Workspace | null;
  modelHub: ModelHubSnapshot | null;
  currentFile: HomeContextTextInput | null;
  selection: HomeContextTextInput | null;
  errors: HomeContextErrorInput[];
  gatewayBusy: boolean;
  adding: boolean;
  checkpointBusy: boolean;
  safetyBusy: boolean;
  aiAccessBusy: boolean;
  aiAccessPaused: boolean;
  onModelHubChange: (snapshot: ModelHubSnapshot) => void;
  onToggleGateway: () => void;
  onAddProject: () => void;
  onCreateProject: () => void;
  onCreateCheckpoint: () => void;
  onSafetyScan: () => void;
  onToggleAiAccess: () => void;
  onNavigate: (view: AppView) => void;
};

function QuickAction({
  icon,
  title,
  caption,
  onClick,
  disabled = false,
  active = false,
}: {
  icon: IconName;
  title: string;
  caption: string;
  onClick: () => void;
  disabled?: boolean;
  active?: boolean;
}) {
  return (
    <button
      className={`home-quick-action ${active ? "active-safety-action" : ""}`}
      type="button"
      onClick={onClick}
      disabled={disabled}
    >
      <span className="home-quick-action-icon"><NavIcon name={icon} size={18} /></span>
      <span className="home-quick-action-copy">
        <strong>{title}</strong>
        <small>{caption}</small>
      </span>
      <span className="home-quick-action-arrow" aria-hidden="true">›</span>
    </button>
  );
}

function HomeWorkspace({
  gateway,
  connection,
  publicTunnel,
  workspaces,
  selectedWorkspace,
  modelHub,
  currentFile,
  selection,
  errors,
  gatewayBusy,
  adding,
  checkpointBusy,
  safetyBusy,
  aiAccessBusy,
  aiAccessPaused,
  onModelHubChange,
  onToggleGateway,
  onAddProject,
  onCreateProject,
  onCreateCheckpoint,
  onSafetyScan,
  onToggleAiAccess,
  onNavigate,
}: HomeWorkspaceProps) {
  const [showLocalChat, setShowLocalChat] = useState(false);
  const hasProject = workspaces.length > 0;
  const remoteReady = publicTunnel.ready || connection.ready;
  const remoteTitle = publicTunnel.ready ? "Public MCP ready" : connection.ready ? "Connected" : "Disconnected";
  const remoteDetail = aiAccessPaused
    ? "MCP workspace access paused"
    : publicTunnel.ready
      ? (publicTunnel.lastRemoteRequestAt ? "Remote MCP traffic detected" : "Stable public endpoint online")
      : connection.ready
        ? (connection.clientVersion ?? "Managed client connected")
        : "External MCP access";

  return (
    <div className="home-workspace home-workspace-chat">
      <section className="home-center-stage">
        <div className="home-center-content home-chat-center-content" style={{ overflowY: "auto", justifyContent: "flex-start", padding: "32px", gap: "20px" }}>
          <div style={{ width: "100%", maxWidth: 760, margin: "0 auto", display: "grid", gap: 20 }}>
            <div>
              <span className="section-kicker">RazedTunnel workspace</span>
              <h1 style={{ margin: "8px 0" }}>Connection &amp; projects</h1>
              <p style={{ color: "var(--text-muted, #8995a6)" }}>Your local development bridge at a glance.</p>
            </div>
            <section className="rail-card connection-card">
              <div className="rail-card-title"><NavIcon name="gateway" size={16} /><strong>Connection status</strong></div>
              <p>{gateway.running ? "Local MCP gateway running" : "Local MCP gateway stopped"} · {remoteReady ? "Remote endpoint ready" : "Remote endpoint not ready"}</p>
              <p>{aiAccessPaused ? "AI workspace access is paused." : "AI workspace access is enabled."}</p>
              <button type="button" onClick={() => onNavigate("connections")}>Manage connections</button>
            </section>
            <section className="rail-card connection-card">
              <div className="rail-card-title"><NavIcon name="folder" size={16} /><strong>Approved projects ({workspaces.length})</strong></div>
              {workspaces.length === 0 ? <p>No projects approved yet.</p> : (
                <div style={{ display: "grid", gap: 10 }}>
                  {workspaces.map((workspace) => (
                    <div key={workspace.id} style={{ display: "flex", justifyContent: "space-between", gap: 12, alignItems: "center", flexWrap: "wrap" }}>
                      <span><strong>{workspace.name}</strong>{selectedWorkspace?.id === workspace.id ? " · Selected" : ""}</span>
                      <small>{workspace.accessMode === "readOnly" ? "Read only" : "Read/write"} · Changes: {workspace.changePolicy} · Commands: {workspace.commandPolicy}</small>
                    </div>
                  ))}
                </div>
              )}
              <button type="button" onClick={() => onNavigate("projects")}>Manage projects</button>
            </section>
            <section className="rail-card connection-card">
              <div className="rail-card-title"><NavIcon name="checks" size={16} /><strong>Development tools</strong></div>
              <p>Review pending changes, run checks, or inspect Git for the selected project.</p>
              <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                <button type="button" onClick={() => onNavigate("changes")}>History</button>
                <button type="button" onClick={() => onNavigate("checks")}>Checks</button>
                <button type="button" onClick={() => onNavigate("git")}>Git</button>
              </div>
            </section>
            <section className="rail-card connection-card">
              <button type="button" aria-expanded={showLocalChat} onClick={() => setShowLocalChat((value) => !value)}>
                <NavIcon name="model" size={16} /> {showLocalChat ? "Hide local AI chat" : "Show local AI chat"}
              </button>
              {showLocalChat ? (
                <div style={{ minHeight: 480, position: "relative" }}>
                  <HomeChat workspace={selectedWorkspace} modelHub={modelHub} currentFile={currentFile} selection={selection} errors={errors} onModelHubChange={onModelHubChange} />
                </div>
              ) : null}
            </section>
          </div>
        </div>

      </section>

      <aside className="home-right-rail">
        <section className="rail-card connection-card">
          <div className="rail-card-title"><NavIcon name="gateway" size={16} /><strong>Gateway</strong></div>
          <div className={`rail-status ${gateway.running ? "online" : "offline"}`}>
            <span className="rail-status-dot" />
            <strong>{gateway.running ? "Online" : "Offline"}</strong>
          </div>
          <small>{gateway.running && gateway.port ? `127.0.0.1:${gateway.port}` : "Local MCP endpoint"}</small>
          <button type="button" onClick={onToggleGateway} disabled={gatewayBusy}>
            <NavIcon name={gateway.running ? "stop" : "play"} size={16} />
            {gatewayBusy ? "Working..." : gateway.running ? "Stop gateway" : "Start gateway"}
          </button>
        </section>

        <section className="rail-card connection-card">
          <div className="rail-card-title"><NavIcon name="remote" size={16} /><strong>Remote</strong></div>
          <div className={`rail-status ${remoteReady ? "online" : "neutral"}`}>
            <span className="rail-status-dot" />
            <strong>{remoteTitle}</strong>
          </div>
          <small>{remoteDetail}</small>
          <button type="button" onClick={() => onNavigate("connections")}>
            <NavIcon name="link" size={16} />
            {remoteReady ? "Manage remote" : "Connect remote"}
          </button>
        </section>

        <section className="rail-card connection-card">
          <div className="rail-card-title"><NavIcon name="folder" size={16} /><strong>Projects</strong></div>
          <div className="rail-status neutral">
            <strong>{workspaces.length} approved workspace{workspaces.length === 1 ? "" : "s"}</strong>
          </div>
          <small>
            {selectedWorkspace
              ? `Selected: ${selectedWorkspace.name} · ${selectedWorkspace.accessMode === "readOnly" ? "Read only" : "Read/write"}`
              : "No project selected"}
          </small>
          <button type="button" onClick={() => onNavigate("projects")}>
            <NavIcon name="folder" size={16} />
            Manage projects
          </button>
        </section>

        <section className="rail-card quick-actions-card">
          <div className="rail-card-heading-row">
            <strong>Quick Actions</strong>
            <span>Existing tools</span>
          </div>
          <div className="home-quick-actions">
            <QuickAction icon="folder" title={adding ? "Opening..." : "Open project"} caption="Browse workspace" onClick={onAddProject} disabled={adding} />
            <QuickAction icon="plus" title="New project" caption="Create from scratch" onClick={onCreateProject} disabled={adding} />
            <QuickAction icon="checkpoint" title={checkpointBusy ? "Saving..." : "Create checkpoint"} caption="Save project state" onClick={onCreateCheckpoint} disabled={!hasProject || checkpointBusy} />
            <QuickAction icon="shield" title={safetyBusy ? "Scanning..." : "Safety scan"} caption="Check protection" onClick={onSafetyScan} disabled={!hasProject || safetyBusy} />
            <QuickAction
              icon={aiAccessPaused ? "resume" : "pause"}
              title={aiAccessBusy ? "Updating..." : aiAccessPaused ? "Resume AI access" : "Pause AI access"}
              caption={aiAccessPaused ? "Unlock MCP clients" : "Temporarily lock AI"}
              onClick={onToggleAiAccess}
              disabled={aiAccessBusy}
              active={aiAccessPaused}
            />
          </div>
        </section>

      </aside>
    </div>
  );
}

export default HomeWorkspace;
