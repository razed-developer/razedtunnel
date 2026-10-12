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


  return (
    <div className="home-workspace home-workspace-chat">
      <section className="home-center-stage">
        <div className="home-center-content razed-home-start">
          <div className="razed-home-intro">
            <span className="section-kicker">RazedTunnel</span>
            <h1>Ready to work</h1>
            <p>{selectedWorkspace ? `Continue with ${selectedWorkspace.name}, or choose another project from the drawer.` : "Select a project from the drawer to begin, or add a new one."}</p>
          </div>
          <div className="razed-home-primary-actions">
            <button type="button" className="primary-button" onClick={() => onNavigate("projects")} disabled={!hasProject}>
              <NavIcon name="folder" size={16} /> {selectedWorkspace ? "Open selected project" : "Browse projects"}
            </button>
            <button type="button" className="secondary-button" onClick={onAddProject} disabled={adding}>
              <NavIcon name="plus" size={16} /> {adding ? "Opening…" : "Add project"}
            </button>
            <button type="button" className="secondary-button" onClick={onCreateProject} disabled={adding}>New project</button>
          </div>
          <div className="razed-home-summary">
            <span>{workspaces.length} approved project{workspaces.length === 1 ? "" : "s"}</span>
            <span>{gateway.running ? "Gateway online" : "Gateway offline"}</span>
            <span>{aiAccessPaused ? "AI access paused" : remoteReady ? "Remote ready" : "Remote disconnected"}</span>
          </div>
          <div className="razed-home-chat-toggle">
            <button type="button" className="secondary-button" aria-expanded={showLocalChat} onClick={() => setShowLocalChat((value) => !value)}>
              <NavIcon name="model" size={16} /> {showLocalChat ? "Hide local AI chat" : "Local AI chat"}
            </button>
            {showLocalChat ? <div className="razed-home-chat-panel">
              <HomeChat workspace={selectedWorkspace} modelHub={modelHub} currentFile={currentFile} selection={selection} errors={errors} onModelHubChange={onModelHubChange} />
            </div> : null}
          </div>
        </div>

      </section>

      <aside className="home-right-rail">
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
