import "./PendingApprovalsPanel.css";
import { useCallback, useEffect, useState } from "react";
import { approvePendingBrowserAction, approvePendingFileChange, denyPendingApprovalRequest, listPendingApprovalRequests, rejectPendingBrowserAction, rejectPendingFileChange, type PendingApprovalRequest } from "../lib/backend";
import type { ActivityTimeline } from "../types";
import { NavIcon } from "./AppSidebar";

type Props = { activity?: ActivityTimeline | null; onRefreshActivity: () => void };

/** Pending operational records are informational until trusted local confirmation is implemented. */
export default function PendingApprovalsPanel({ activity, onRefreshActivity }: Props) {
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
  async function approveBrowser(id: string) {
    setBusyId(id);
    try { await approvePendingBrowserAction(id); await refresh(); onRefreshActivity(); }
    catch (cause) { setError(String(cause)); }
    finally { setBusyId(null); }
  }
  async function rejectBrowser(id: string) {
    setBusyId(id);
    try { await rejectPendingBrowserAction(id); await refresh(); onRefreshActivity(); }
    catch (cause) { setError(String(cause)); }
    finally { setBusyId(null); }
  }
  async function approveFileChange(id: string) {
    setBusyId(id);
    try { await approvePendingFileChange(id); await refresh(); onRefreshActivity(); }
    catch (cause) { setError(String(cause)); }
    finally { setBusyId(null); }
  }
  async function rejectFileChange(id: string) {
    setBusyId(id);
    try { await rejectPendingFileChange(id); await refresh(); onRefreshActivity(); }
    catch (cause) { setError(String(cause)); }
    finally { setBusyId(null); }
  }
  const pendingOperations = (activity?.groups ?? [])
    .flatMap(group => group.events
      .filter(event => event.status === "pending")
      .map(event => ({ ...event, workspaceName: group.workspaceName, workspaceId: group.workspaceId })))
    .sort((a, b) => b.createdAt - a.createdAt);
  return (
    <section className="r-home-card r-approval-panel" aria-label="Pending operation requests">
      <div className="r-home-section-title">
        <div><h3><NavIcon name="shield" /> Operation requests</h3><p>Pending operations across projects. Browser requests require native confirmation before approval. File changes require a reviewable diff and native confirmation.</p></div>
        <button type="button" onClick={() => void refresh()}>Refresh requests</button>
      </div>
      {error && <p role="alert" className="r-home-attention">Operation request error: {error}</p>}
      {!requests.length && !pendingOperations.length && !error && <p className="r-home-muted">No pending requests.</p>}
      {pendingOperations.map(event => (
        <div className="r-approval-request" key={event.id}>
          <div className="r-approval-request-info">
            <strong>{event.summary}</strong>
            <span>{event.workspaceName} · {event.kind} · {new Date(event.createdAt).toLocaleString()}</span>
            {event.detail && <span style={{ whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}>{event.detail}</span>}
          </div>
          <div>
            <span className="r-activity-status r-activity-status-pending">Pending</span>
            {event.kind === "browser" && event.sourceId && <button type="button" disabled={busyId === event.sourceId} onClick={() => void approveBrowser(event.sourceId!)}>Accept…</button>}
            {event.kind === "browser" && event.sourceId && <button type="button" disabled={busyId === event.sourceId} onClick={() => void rejectBrowser(event.sourceId!)}>Reject</button>}
            {event.kind === "files" && event.sourceId && <button type="button" disabled={busyId === event.sourceId} onClick={() => void approveFileChange(event.sourceId!)}>Review &amp; Accept…</button>}
            {event.kind === "files" && event.sourceId && <button type="button" disabled={busyId === event.sourceId} onClick={() => void rejectFileChange(event.sourceId!)}>Reject</button>}
          </div>
        </div>
      ))}
      {requests.map(request => (
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
      <p className="r-activity-disclaimer">Pending activity is read from retained audit records. Browser Accept requires an explicit native confirmation; file changes require diff review and native confirmation.</p>
    </section>
  );
}
