import "./PendingApprovalsPanel.css";
import { useCallback, useEffect, useState } from "react";
import { denyPendingApprovalRequest, listPendingApprovalRequests, type PendingApprovalRequest } from "../lib/backend";
import { NavIcon } from "./AppSidebar";

/** Review and rejection only. This panel cannot grant filesystem permissions. */
export default function PendingApprovalsPanel() {
  const [requests, setRequests] = useState<PendingApprovalRequest[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);
  const refresh = useCallback(async () => {
    try { setRequests(await listPendingApprovalRequests()); setError(null); }
    catch (cause) { setError(String(cause)); }
  }, []);
  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => { void refresh(); }, 10000);
    return () => window.clearInterval(timer);
  }, [refresh]);
  async function deny(id: string) {
    setBusyId(id);
    try { await denyPendingApprovalRequest(id); await refresh(); }
    catch (cause) { setError(String(cause)); }
    finally { setBusyId(null); }
  }
  return (
    <section className="r-home-card r-approval-panel" aria-label="Pending operation requests">
      <div className="r-home-section-title">
        <div><h3><NavIcon name="shield" /> Operation requests</h3><p>Filesystem approval proposals only. Browser launches, Git operations, and queued file changes appear in Recent activity, not this list. Local approval controls are not yet implemented.</p></div>
        <button type="button" onClick={() => void refresh()}>Refresh</button>
      </div>
      {error && <p role="alert" className="r-home-attention">Unable to load requests: {error}</p>}
      {!requests.length && !error && <p className="r-home-muted">No pending requests.</p>}
      {requests.map((request) => (
        <div className="r-approval-request" key={request.id}>
          <div className="r-approval-request-info">
            <strong>{request.operation} · {request.workspaceId}</strong>
            <span title={request.path}>{request.path}</span>
          </div>
          <button type="button" disabled={busyId === request.id} onClick={() => void deny(request.id)}>
            {busyId === request.id ? "Denying…" : "Deny"}
          </button>
        </div>
      ))}
    </section>
  );
}
