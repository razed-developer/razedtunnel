import { useState, type FormEvent } from "react";
import type {
  CommandPolicy,
  Workspace,
  WorkspaceAccessMode,
  WorkspaceChangePolicy,
  WorkspaceHealth,
} from "../types";

type WorkspaceListProps = {
  workspaces: Workspace[];
  adding: boolean;
  removingId: string | null;
  updatingId: string | null;
  workspaceHealth: Record<string, WorkspaceHealth>;
  relocatingWorkspaceId: string | null;
  onAdd: () => void;
  onOpenFolder: (workspace: Workspace) => void;
  onClone: (url: string) => Promise<void>;
  onRemove: (workspace: Workspace) => void;
  onAccessChange: (workspace: Workspace, accessMode: WorkspaceAccessMode) => void;
  onChangePolicyChange: (workspace: Workspace, changePolicy: WorkspaceChangePolicy) => void;
  onCommandPolicyChange: (workspace: Workspace, commandPolicy: CommandPolicy) => void;
  onRelocate: (workspace: Workspace) => void;
  onRetryHealth: (workspaceId: string) => void;
};

function accessLabel(accessMode: WorkspaceAccessMode): string {
  return accessMode === "readWrite" ? "Read + write" : "Read only";
}

function changePolicyLabel(changePolicy: WorkspaceChangePolicy): string {
  return changePolicy === "automatic" ? "Apply edits automatically" : "Review edits first";
}

function commandPolicyLabel(commandPolicy: CommandPolicy): string {
  if (commandPolicy === "disabled") return "Commands disabled";
  return commandPolicy === "review" ? "Review commands" : "Run commands automatically";
}

function WorkspaceList({
  workspaces,
  adding,
  removingId,
  updatingId,
  workspaceHealth,
  relocatingWorkspaceId,
  onAdd,
  onOpenFolder,
  onClone,
  onRemove,
  onAccessChange,
  onChangePolicyChange,
  onCommandPolicyChange,
  onRelocate,
  onRetryHealth,
}: WorkspaceListProps) {
  const [showGitInstructions, setShowGitInstructions] = useState(false);
  const [gitUrl, setGitUrl] = useState("");
  const [cloning, setCloning] = useState(false);
  const [cloneError, setCloneError] = useState<string | null>(null);
  async function submitClone(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setCloning(true);
    setCloneError(null);
    try {
      await onClone(gitUrl.trim());
      setGitUrl("");
      setShowGitInstructions(false);
    } catch (error) {
      setCloneError(error instanceof Error ? error.message : String(error));
    } finally {
      setCloning(false);
    }
  }
  return (
    <section className="workspace-section" aria-labelledby="workspace-title">
      <div className="section-heading">
        <div>
          <span className="section-kicker">Projects</span>
          <h2 id="workspace-title">Approved workspaces</h2>
          <p>Manage approved folders and their access permissions.</p>
        </div>
        <div className="workspace-actions">
          <button className="secondary-button" type="button" onClick={onAdd} disabled={adding}>
            <span aria-hidden="true">+</span>
            {adding ? "Adding…" : "Add local folder"}
          </button>
          <button className="secondary-button" type="button" aria-expanded={showGitInstructions} onClick={() => setShowGitInstructions((value) => !value)}>
            Git repository
          </button>
        </div>
      </div>

      {showGitInstructions ? (
        <div className="security-note" role="region" aria-label="Connect a Git repository">
          <div>
            <strong>Clone and connect a Git repository</strong>
            <p>Enter an HTTPS clone URL, then select a parent folder on this computer. RazedTunnel will clone into a new subfolder and register it with read-only access, edit review, and commands disabled by default. You can change permissions afterward. Git must be installed; private repositories need non-interactive credentials already configured.</p>
            <form onSubmit={(event) => void submitClone(event)}>
              <label htmlFor="git-clone-url">Repository URL</label>
              <input id="git-clone-url" type="url" required placeholder="https://github.com/owner/repository.git" value={gitUrl} onChange={(event) => setGitUrl(event.target.value)} disabled={cloning} style={{ width: "100%", margin: "8px 0" }} />
              <button className="secondary-button" type="submit" disabled={cloning || !gitUrl.trim()}>{cloning ? "Cloning…" : "Choose destination and clone"}</button>
            </form>
            {cloneError ? <p role="alert">{cloneError}</p> : null}
            <p>Already cloned? <button className="secondary-button" type="button" onClick={onAdd} disabled={adding || cloning}>Select existing checkout</button></p>
          </div>
        </div>
      ) : null}

      {workspaces.length === 0 ? (
        <div className="empty-state">
          <div className="empty-icon" aria-hidden="true">&lt;/&gt;</div>
          <h3>No projects added yet</h3>
          <p>Choose an existing local folder or a checked-out Git repository to make it available through MCP.</p>
          <button className="primary-button" type="button" onClick={onAdd} disabled={adding}>
            {adding ? "Opening…" : "Choose project folder"}
          </button>
        </div>
      ) : (
        <div className="workspace-list">
          {workspaces.map((workspace) => {
            const updating = updatingId === workspace.id;
            const health = workspaceHealth[workspace.id];
            const unavailable = health?.available === false;

            return (
              <article className={`workspace-card ${unavailable ? "workspace-card-missing" : ""}`} key={workspace.id}>
                <button className="workspace-icon workspace-folder-button" type="button" onClick={() => onOpenFolder(workspace)} disabled={unavailable} aria-label={`Open ${workspace.name} folder`} title={unavailable ? "Folder unavailable" : "Open folder in file manager"}>
                  <svg viewBox="0 0 24 24" width="19" height="19" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>
                </button>
                <div className="workspace-details">
                  <div className="workspace-title-row">
                    <h3>{workspace.name}</h3>
                    <span className={`access-badge ${unavailable ? "missing" : ""}`}>{unavailable ? "Path missing" : "Approved"}</span>
                  </div>
                  <p title={workspace.path}>{workspace.path}</p>
                  {unavailable ? <div className="workspace-path-warning">{health?.message ?? "This project folder cannot be reached."}</div> : null}
                  <div className="workspace-security" aria-label="Project security status">
                    <span>Secrets blocked</span>
                  </div>
                </div>
                <div className="workspace-actions workspace-card-actions">
                  {unavailable ? (
                    <>
                      <button className="access-button recovery" type="button" onClick={() => onRelocate(workspace)} disabled={relocatingWorkspaceId === workspace.id}>
                        {relocatingWorkspaceId === workspace.id ? "Locating…" : "Locate again"}
                      </button>
                      <button className="policy-button" type="button" onClick={() => onRetryHealth(workspace.id)}>Recheck</button>
                    </>
                  ) : (<div className="workspace-policy-controls" aria-label={`Permissions for ${workspace.name}`}>
                  <div className="workspace-permission-field">
                    <span>Edit approval</span>
                    <div className="workspace-segmented" role="group" aria-label={`Edit approval for ${workspace.name}`}>
                      <button type="button" className={workspace.changePolicy === "review" ? "selected" : ""} aria-pressed={workspace.changePolicy === "review"} disabled={updating} onClick={() => onChangePolicyChange(workspace, "review")}>Review</button>
                      <button type="button" className={workspace.changePolicy === "automatic" ? "selected" : ""} aria-pressed={workspace.changePolicy === "automatic"} disabled={updating} onClick={() => onChangePolicyChange(workspace, "automatic")}>Automatic</button>
                    </div>
                  </div>
                  <div className="workspace-permission-field">
                    <span>Commands</span>
                    <div className="workspace-segmented" role="group" aria-label={`Command execution for ${workspace.name}`}>
                      {(["disabled", "review", "automatic"] as const).map((policy) => (
                        <button key={policy} type="button" className={workspace.commandPolicy === policy ? "selected" : ""} aria-pressed={workspace.commandPolicy === policy} disabled={updating || workspace.changePolicy === "automatic"} title={workspace.changePolicy === "automatic" ? "Automatic edits also enable automatic command execution" : undefined} onClick={() => onCommandPolicyChange(workspace, policy)}>{policy === "disabled" ? "Off" : policy === "review" ? "Review" : "Auto"}</button>
                      ))}
                    </div>
                  </div>
                  <div className="workspace-permission-field">
                    <span>File access</span>
                    <div className="workspace-segmented" role="group" aria-label={`File access for ${workspace.name}`}>
                      <button type="button" className={workspace.accessMode === "readOnly" ? "selected" : ""} aria-pressed={workspace.accessMode === "readOnly"} disabled={updating} onClick={() => onAccessChange(workspace, "readOnly")}>Read only</button>
                      <button type="button" className={workspace.accessMode === "readWrite" ? "selected" : ""} aria-pressed={workspace.accessMode === "readWrite"} disabled={updating} onClick={() => onAccessChange(workspace, "readWrite")}>Read + write</button>
                    </div>
                  </div>
                  </div>)}
                  <button
                    className="icon-button"
                    type="button"
                    aria-label={`Remove ${workspace.name}`}
                    title="Remove project"
                    disabled={removingId === workspace.id}
                    onClick={() => onRemove(workspace)}
                  >
                    {removingId === workspace.id ? "…" : "×"}
                  </button>
                </div>
              </article>
            );
          })}
        </div>
      )}

      <div className="security-note">
        <span className="security-note-icon" aria-hidden="true">✓</span>
        <div>
          <strong>Workspace boundary enforced locally</strong>
          <p>
            File access stays inside the approved root. Applied AI code edits are saved into local
            version history. Automatic command execution can run live development commands. Enable it only for trusted projects.
          </p>
        </div>
      </div>
    </section>
  );
}

export default WorkspaceList;
