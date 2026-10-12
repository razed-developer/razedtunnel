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

function nextCommandPolicy(commandPolicy: CommandPolicy): CommandPolicy {
  if (commandPolicy === "review") return "automatic";
  if (commandPolicy === "automatic") return "disabled";
  return "review";
}

function WorkspaceList({
  workspaces,
  adding,
  removingId,
  updatingId,
  workspaceHealth,
  relocatingWorkspaceId,
  onAdd,
  onRemove,
  onAccessChange,
  onChangePolicyChange,
  onCommandPolicyChange,
  onRelocate,
  onRetryHealth,
}: WorkspaceListProps) {
  return (
    <section className="workspace-section" aria-labelledby="workspace-title">
      <div className="section-heading">
        <div>
          <span className="section-kicker">Projects</span>
          <h2 id="workspace-title">Approved workspaces</h2>
          <p>
            Only approved local folders are exposed through RazedTunnel. To connect a Git repository,
            select its local checkout. Access and execution permissions are configured per project.
          </p>
        </div>
        <button className="secondary-button" type="button" onClick={onAdd} disabled={adding}>
          <span aria-hidden="true">+</span>
          {adding ? "Adding…" : "Add local folder"}
        </button>
      </div>

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
            const nextMode: WorkspaceAccessMode =
              workspace.accessMode === "readWrite" ? "readOnly" : "readWrite";

            return (
              <article className={`workspace-card ${unavailable ? "workspace-card-missing" : ""}`} key={workspace.id}>
                <div className="workspace-icon" aria-hidden="true">&lt;/&gt;</div>
                <div className="workspace-details">
                  <div className="workspace-title-row">
                    <h3>{workspace.name}</h3>
                    <span className={`access-badge ${unavailable ? "missing" : ""}`}>{unavailable ? "Path missing" : "Approved"}</span>
                  </div>
                  <p title={workspace.path}>{workspace.path}</p>
                  {unavailable ? <div className="workspace-path-warning">{health?.message ?? "This project folder cannot be reached."}</div> : null}
                  <div className="workspace-security">
                    <span>{accessLabel(workspace.accessMode)}</span>
                    <span aria-hidden="true">•</span>
                    <span>{changePolicyLabel(workspace.changePolicy)}</span>
                    <span aria-hidden="true">•</span>
                    <span>{commandPolicyLabel(workspace.commandPolicy)}</span>
                    <span aria-hidden="true">•</span>
                    <span>Secrets blocked</span>
                  </div>
                </div>
                <div className="workspace-actions">
                  {unavailable ? (
                    <>
                      <button className="access-button recovery" type="button" onClick={() => onRelocate(workspace)} disabled={relocatingWorkspaceId === workspace.id}>
                        {relocatingWorkspaceId === workspace.id ? "Locating…" : "Locate again"}
                      </button>
                      <button className="policy-button" type="button" onClick={() => onRetryHealth(workspace.id)}>Recheck</button>
                    </>
                  ) : (<>
                  <button
                    className={`policy-button ${workspace.changePolicy === "automatic" ? "automatic" : ""}`}
                    type="button"
                    disabled={updating}
                    onClick={() =>
                      onChangePolicyChange(
                        workspace,
                        workspace.changePolicy === "automatic" ? "review" : "automatic",
                      )
                    }
                    title="Toggle whether MCP file changes require local approval"
                  >
                    {changePolicyLabel(workspace.changePolicy)}
                  </button>
                  <button
                    className={`policy-button ${workspace.commandPolicy === "automatic" ? "automatic" : ""}`}
                    type="button"
                    disabled={updating || workspace.changePolicy === "automatic"}
                    onClick={() => onCommandPolicyChange(workspace, nextCommandPolicy(workspace.commandPolicy))}
                    title={workspace.changePolicy === "automatic"
                      ? "Automatic edit mode also enables automatic command execution"
                      : "Cycle command policy: review, automatic execution, or disabled"}
                  >
                    {commandPolicyLabel(workspace.commandPolicy)}
                  </button>
                  <button
                    className="access-button"
                    type="button"
                    disabled={updating}
                    onClick={() => onAccessChange(workspace, nextMode)}
                    title={`Switch ${workspace.name} to ${accessLabel(nextMode).toLowerCase()}`}
                  >
                    {updating ? "Updating…" : accessLabel(workspace.accessMode)}
                  </button>
                  </>)}
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
