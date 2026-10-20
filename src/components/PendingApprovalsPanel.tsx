import "./PendingApprovalsPanel.css";
import { useCallback, useEffect, useState } from "react";
import { approvePendingBrowserAction, approvePendingFileChange, denyPendingApprovalRequest, listPendingApprovalRequests, rejectPendingBrowserAction, rejectPendingFileChange, type PendingApprovalRequest } from "../lib/backend";
import type { ActivityTimeline } from "../types";
import { NavIcon } from "./AppSidebar";

type Props = { activity?: ActivityTimeline | null; onRefreshActivity: () => void };

/** Pending operations expose only the locally supported approval actions. */
export default function PendingApprovalsPanel({ activity, onRefreshActivity }: Props) {
  const [requests, setRequests] = useState<PendingApprovalRequest[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [reviewId, setReviewId] = useState<string | null>(null);
  const refresh = useCallback(async () => {
    try { setRequests(await listPendingApprovalRequests()); }
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
  const reviewEvent = pendingOperations.find(event => event.sourceId === reviewId);
  return (
    <section className="r-home-card r-approval-panel" aria-label="Pending operation requests">
      <div className="r-home-section-title">
        <div><h3><NavIcon name="shield" /> Operation requests</h3><p>Pending operations across projects. Browser requests require native confirmation before approval. File changes require a reviewable diff and native confirmation.</p></div>
        <button type="button" onClick={() => void refresh()}>Refresh requests</button>
      </div>
      {error && <div role="alert" className="r-home-attention"><span>Operation request error: {error}</span> <button type="button" onClick={() => setError(null)} aria-label="Dismiss operation request error">Dismiss</button></div>}
      {!requests.length && !pendingOperations.length && !error && <p className="r-home-muted">No pending requests.</p>}
      {pendingOperations.map(event => (
        <div className="r-approval-request" key={event.id}>
          <div className="r-approval-request-info">
            <strong>{event.summary}</strong>
            <span>{event.workspaceName} · {event.kind} · {new Date(event.createdAt).toLocaleString()}</span>
            {event.detail && <span style={{ whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}>{event.detail}</span>}
          </div>
          <div className="r-approval-request-actions">
            <span className="r-activity-status r-activity-status-pending">Pending</span>
            {event.kind === "browser" && event.sourceId && <button type="button" disabled={busyId === event.sourceId} onClick={() => void approveBrowser(event.sourceId!)}>Accept…</button>}
            {event.kind === "browser" && event.sourceId && <button type="button" disabled={busyId === event.sourceId} onClick={() => void rejectBrowser(event.sourceId!)}>Reject</button>}
            {event.kind === "files" && (event.action === "PatchFile" || event.action === "CreateFile") && event.sourceId && <button type="button" disabled={busyId === event.sourceId} onClick={() => setReviewId(event.sourceId!)}>Review &amp; Accept…</button>}
            {event.kind === "files" && event.action !== "PatchFile" && event.action !== "CreateFile" && <span className="r-approval-unavailable" title="This operation cannot be accepted from this panel.">Approval unavailable</span>}
            {event.kind === "files" && event.sourceId && <button type="button" disabled={busyId === event.sourceId} onClick={() => void rejectFileChange(event.sourceId!)}>Reject</button>}
          </div>
        </div>
      ))}
      {reviewEvent && (
        <div className="r-approval-review-backdrop" role="presentation">
          <section className="r-approval-review-dialog" role="dialog" aria-modal="true" aria-label="Review proposed file change">
            <h3>Review proposed file change</h3>
            <p><strong>{reviewEvent.summary}</strong> · {reviewEvent.workspaceName}</p>
            <p className="r-home-muted">The activity preview below may be abbreviated. Final approval is only possible when the backend can independently verify a complete diff.</p>
            <pre className="r-approval-review-diff">{reviewEvent.detail || "No activity preview available. Approval will be blocked if the backend cannot verify a complete diff."}</pre>
            <div className="r-approval-review-actions">
              <button type="button" onClick={() => setReviewId(null)}>Cancel</button>
              <button type="button" disabled={busyId === reviewId} onClick={() => { const id = reviewId; setReviewId(null); if (id) void approveFileChange(id); }}>Continue to native confirmation…</button>
            </div>
          </section>
        </div>
      )}
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
