import type { ActivityTimeline } from "../types";

export default function ActivityFeed({ timeline, loading, error, onRefresh }: { timeline: ActivityTimeline | null; loading: boolean; error: string | null; onRefresh: () => void }) {
  const events = (timeline?.groups ?? []).flatMap(group => group.events.map(event => ({ ...event, workspaceName: group.workspaceName }))).sort((a,b) => b.createdAt - a.createdAt).slice(0, 12);
  return <section className="home-card activity-feed"><div className="home-section-title"><div><h3>Recent activity</h3><p>Recorded actions across your projects</p></div><button type="button" disabled={loading} onClick={onRefresh}>{loading ? "Refreshing…" : "Refresh"}</button></div>
    {error ? <p role="alert">Activity unavailable: {error}</p> : null}
    {!error && !events.length ? <p className="activity-empty">{loading ? "Loading activity…" : "No recorded activity yet."}</p> : null}
    {!error && events.map(event => <div className="activity-entry" key={event.id}><div><strong>{event.summary}</strong><p>{event.workspaceName} · {event.kind} · {new Date(event.createdAt).toLocaleString()}</p></div><span>{event.status}</span></div>)}
    <p className="activity-disclaimer">Pending records are informational. Approval actions are not yet enabled.</p>
  </section>;
}
