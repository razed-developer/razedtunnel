import { useMemo, useState } from "react";
import type { ActivityTimeline } from "../types";

type Props = { timeline: ActivityTimeline | null; loading: boolean; error: string | null; onRefresh: () => void };

export default function ActivityFeed({ timeline, loading, error, onRefresh }: Props) {
  const [expanded, setExpanded] = useState(false);
  const [workspace, setWorkspace] = useState("all");
  const [kind, setKind] = useState("all");
  const [status, setStatus] = useState("all");
  const [search, setSearch] = useState("");
  const [limit, setLimit] = useState(50);
  const groups = timeline?.groups ?? [];
  const workspaces = useMemo(() => Array.from(new Map(groups.map(group => [group.workspaceId, group.workspaceName])).entries()).sort((a, b) => a[1].localeCompare(b[1])), [groups]);
  const events = useMemo(() => groups.flatMap(group => group.events.map(event => ({
    ...event, workspaceId: group.workspaceId, workspaceName: group.workspaceName,
  }))).sort((a, b) => b.createdAt - a.createdAt), [groups]);
  const filtered = useMemo(() => events.filter(event =>
    (workspace === "all" || event.workspaceId === workspace) &&
    (kind === "all" || event.kind === kind) &&
    (status === "all" || event.status === status) &&
    (!search.trim() || [event.summary, event.detail ?? "", event.action, event.workspaceName].join(" ").toLowerCase().includes(search.trim().toLowerCase()))
  ), [events, workspace, kind, status, search]);
  const visible = expanded ? filtered.slice(0, limit) : events.slice(0, 12);
  return <section className="r-home-card r-activity-feed">
    <div className="r-home-section-title">
      <div><h3>{expanded ? "Activity log" : "Recent activity"}</h3><p>{expanded ? "Browse recorded project activity with filters" : "Latest recorded actions across your projects"}</p></div>
      <div style={{ display: "flex", gap: 8 }}>
        <button type="button" onClick={() => { setExpanded(!expanded); setLimit(50); }}>{expanded ? "Show recent" : "View activity log →"}</button>
        <button type="button" disabled={loading} onClick={onRefresh}>{loading ? "Refreshing…" : "Refresh"}</button>
      </div>
    </div>
    {expanded && <div className="r-activity-filters" style={{ display: "flex", flexWrap: "wrap", gap: 8, marginBottom: 16 }}>
      <label>Project <select aria-label="Filter by project" value={workspace} onChange={e => { setWorkspace(e.target.value); setLimit(50); }}><option value="all">All projects</option>{workspaces.map(([id, name]) => <option key={id} value={id}>{name}</option>)}</select></label>
      <label>Type <select aria-label="Filter by activity type" value={kind} onChange={e => { setKind(e.target.value); setLimit(50); }}><option value="all">All types</option>{["files", "terminal", "process", "launcher", "browser", "git", "monitoring", "verification", "team"].map(value => <option key={value} value={value}>{value}</option>)}</select></label>
      <label>Status <select aria-label="Filter by status" value={status} onChange={e => { setStatus(e.target.value); setLimit(50); }}><option value="all">All statuses</option>{["pending", "running", "succeeded", "failed", "rejected", "stopped", "observed"].map(value => <option key={value} value={value}>{value}</option>)}</select></label>
      <label>Search <input aria-label="Search activity" value={search} onChange={e => { setSearch(e.target.value); setLimit(50); }} placeholder="Action, path, details…" /></label>
      <button type="button" onClick={() => { setWorkspace("all"); setKind("all"); setStatus("all"); setSearch(""); setLimit(50); }}>Clear filters</button>
    </div>}
    {error && <p role="alert">Activity unavailable: {error}</p>}
    {!error && !visible.length && <p className="r-activity-empty">{loading ? "Loading activity…" : "No matching recorded activity."}</p>}
    {!error && visible.map(event => <div className="r-activity-entry" key={event.id}>
      <div><strong>{event.summary}</strong><p>{event.workspaceName} · {event.kind} · {new Date(event.createdAt).toLocaleString()}</p>
      {expanded && event.detail && <p style={{ whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}>{event.detail}</p>}</div>
      <span className={`r-activity-status r-activity-status-${event.status}`}>{event.status}</span>
    </div>)}
    {expanded && !error && <div style={{ display: "flex", alignItems: "center", gap: 12, marginTop: 12 }}>
      <span>{visible.length} of {filtered.length} matching recorded events</span>
      {visible.length < filtered.length && <button type="button" onClick={() => setLimit(value => value + 50)}>Load 50 more</button>}
    </div>}
    <p className="r-activity-disclaimer">This log shows retained activity records, not necessarily every action ever performed. Pending entries are informational; they cannot be approved here.</p>
  </section>;
}
