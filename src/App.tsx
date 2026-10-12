import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import AppHeader from "./components/AppHeader";
import AppErrorBoundary from "./components/AppErrorBoundary";
import AppSidebar, { type AppView } from "./components/AppSidebar";
import ChatConnectionPanel from "./components/ChatConnectionPanel";
import ConfirmationDialog from "./components/ConfirmationDialog";
import GatewayPanel from "./components/GatewayPanel";

import WindowChrome from "./components/WindowChrome";
import PublicTunnelPanel from "./components/PublicTunnelPanel";
import WorkspaceList from "./components/WorkspaceList";
import {
  addWorkspace,
  cloneGitWorkspace,
  checkForUpdates,
  createProject,
  approveChange,
  createCheckpoint,
  clearVersionHistory,
  getAiAccessStatus,
  getChatConnectionStatus,
  getGatewayStatus,
  getWorkspaceHealth,
  getActivityTimeline,
  getPublicTunnelStatus,
  getVersionTimeline,
  listChanges,
  listWorkspaces,
  restoreVersion,
  runSafetyScan,
  rejectChange,
  removeWorkspace,
  relocateWorkspace,
  restartPublicTunnel,
  provisionDirectHttpsCertificate,
  clearPublicTunnel,
  revokeMcpAccess,
  configurePublicTunnel,
  selectWorkspace,
  startChatConnection,
  startGateway,
  stopChatConnection,
  stopGateway,
  setAiAccessPaused,
  updateWorkspaceAccess,
  updateWorkspaceChangePolicy,
  updateWorkspaceCommandPolicy,
} from "./lib/backend";
import type {
  ActivityTimeline,
  ChangeRecord,
  CheckpointSummary,
  ChatConnectionStatus,
  CommandPolicy,
  DirectoryEntry,
  GatewayStatus,
  HomeContextErrorInput,
  HomeContextTextInput,
  PublicTunnelProvider,
  PublicTunnelStatus,
  SafetyScanResult,
  Workspace,
  WorkspaceHealth,
  WorkspaceAccessMode,
  WorkspaceChangePolicy,
  VersionTimeline,
} from "./types";

const initialStatus: GatewayStatus = {
  taskToolSuccesses: 0,
  taskToolFailures: 0,
  running: false,
  port: null,
  workspaceCount: 0,
};


const initialPublicTunnelStatus: PublicTunnelStatus = {
  configured: false,
  provider: "ngrok",
  providerAvailable: true,
  cloudflaredAvailable: false,
  cloudflareOriginPort: 43182,
  directHttpsPort: 43183,
  directHttpChallengePort: 43184,
  certbotAvailable: false,
  certbotVersion: null,
  tlsTrusted: false,
  publicReachable: false,
  localReady: false,
  running: false,
  ready: false,
  publicUrl: null,
  mcpUrl: null,
  autoStart: false,
  requestCount: 0,
  lastRemoteRequestAt: null,
  usageLabel: "0 authenticated MCP requests this launch",
  usageUrl: "https://dashboard.ngrok.com/usage",
  originPort: null,
  message: null,
};

const initialConnectionStatus: ChatConnectionStatus = {
  clientAvailable: false,
  clientVersion: null,
  running: false,
  ready: false,
  tunnelId: null,
  adminUrl: null,
  message: null,
};

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function App() {
  const [activeView, setActiveView] = useState<AppView>("projects");

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    void listen<string>("razedtunnel:open-page", (event) => {
      if (event.payload === "projects" || event.payload === "connections") {
        setActiveView(event.payload);
      }
    }).then((cleanup) => {
      if (disposed) cleanup();
      else unlisten = cleanup;
    }).catch((error) => console.error("Could not listen for tray navigation", error));
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  const [startupProjectRestored, setStartupProjectRestored] = useState(false);
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [selectedWorkspaceId, setSelectedWorkspaceId] = useState<string | null>(null);
  const [changes, setChanges] = useState<ChangeRecord[]>([]);
  const [versionTimeline, setVersionTimeline] = useState<VersionTimeline>({ records: [], currentVersionId: null });
  const [activityTimeline, setActivityTimeline] = useState<ActivityTimeline>({ groups: [] });
  const [versionBusy, setVersionBusy] = useState(false);
  const [gatewayStatus, setGatewayStatus] = useState<GatewayStatus>(initialStatus);
  const [connectionStatus, setConnectionStatus] = useState<ChatConnectionStatus>(
    initialConnectionStatus,
  );
  const [publicTunnelStatus, setPublicTunnelStatus] = useState<PublicTunnelStatus>(initialPublicTunnelStatus);
  const [publicTunnelBusy, setPublicTunnelBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [adding, setAdding] = useState(false);
  const [gatewayBusy, setGatewayBusy] = useState(false);
  const [connectionBusy, setConnectionBusy] = useState(false);
  const [workspaceToRemove, setWorkspaceToRemove] = useState<Workspace | null>(null);
  const [removingId, setRemovingId] = useState<string | null>(null);
  const [updatingId, setUpdatingId] = useState<string | null>(null);
  const [changeBusyId, setChangeBusyId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [focusMode, setFocusMode] = useState(false);
  const [workspaceHealth, setWorkspaceHealth] = useState<Record<string, WorkspaceHealth>>({});
  const [relocatingWorkspaceId, setRelocatingWorkspaceId] = useState<string | null>(null);
  const [pendingWorkspaceSelection, setPendingWorkspaceSelection] = useState<string | null>(null);
  const [showNewProject, setShowNewProject] = useState(false);
  const [creatingProject, setCreatingProject] = useState(false);
  const [confirmStopGateway, setConfirmStopGateway] = useState(false);
  const [confirmForgetPublicTunnel, setConfirmForgetPublicTunnel] = useState(false);
  const [confirmRevokeMcpAccess, setConfirmRevokeMcpAccess] = useState(false);
  const [aiAccessPaused, setAiAccessPausedState] = useState(false);
  const [checkpointBusy, setCheckpointBusy] = useState(false);
  const [safetyBusy, setSafetyBusy] = useState(false);
  const [aiAccessBusy, setAiAccessBusy] = useState(false);
  const [checkpointResult, setCheckpointResult] = useState<CheckpointSummary | null>(null);
  const [safetyScanResult, setSafetyScanResult] = useState<SafetyScanResult | null>(null);

  const pendingCount = useMemo(
    () => changes.filter((change) => change.status === "pending").length,
    [changes],
  );

  const handleExecutionError = useCallback((message: string) => {
    setNotice(`Command execution: ${message}`);
  }, []);

  const refreshPendingChanges = useCallback(async () => {
    setChanges(await listChanges(undefined, 100));
  }, []);

  const refreshChanges = useCallback(async () => {
    await refreshPendingChanges();
    if (selectedWorkspaceId) {
      const [versions, activities] = await Promise.all([
        getVersionTimeline(selectedWorkspaceId),
        getActivityTimeline(selectedWorkspaceId),
      ]);
      setVersionTimeline(versions);
      setActivityTimeline(activities);
    } else {
      setVersionTimeline({ records: [], currentVersionId: null });
      setActivityTimeline({ groups: [] });
    }
  }, [selectedWorkspaceId, refreshPendingChanges]);

  const refresh = useCallback(async () => {
    const [savedWorkspaces, status, publicStatus, chatStatus, savedChanges, aiStatus] = await Promise.all([
      listWorkspaces(),
      getGatewayStatus(),
      getPublicTunnelStatus(),
      getChatConnectionStatus(),
      listChanges(undefined, 100),
      getAiAccessStatus(),
    ]);
    setWorkspaces(savedWorkspaces);
    setGatewayStatus(status);
    setPublicTunnelStatus(publicStatus);
    setConnectionStatus(chatStatus);
    setChanges(savedChanges);
    setAiAccessPausedState(aiStatus.paused);
  }, []);

  const refreshWorkspaceHealth = useCallback(async () => {
    if (workspaces.length === 0) {
      setWorkspaceHealth({});
      return;
    }
    const entries = await Promise.all(workspaces.map(async (workspace) => {
      try {
        return [workspace.id, await getWorkspaceHealth(workspace.id)] as const;
      } catch (error) {
        return [workspace.id, { workspaceId: workspace.id, available: false, message: errorMessage(error) }] as const;
      }
    }));
    setWorkspaceHealth(Object.fromEntries(entries));
  }, [workspaces]);

  useEffect(() => {
    setLoading(true);
    refresh()
      .catch((error) => setNotice(`RepoTunnel could not initialize: ${errorMessage(error)}`))
      .finally(() => setLoading(false));
  }, [refresh]);

  useEffect(() => {
    let active = true;
    const check = () => {
      checkForUpdates(false)
        .then((status) => {
          if (active && status.shouldNotify && status.update) {
            setNotice(`RepoTunnel ${status.update.version} is available. Open Settings to update.`);
          }
        })
        .catch(() => undefined);
    };
    check();
    const timer = window.setInterval(check, 6 * 60 * 60 * 1000);
    return () => {
      active = false;
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    if (!notice) return;

    const timer = window.setTimeout(() => setNotice(null), 3800);
    return () => window.clearTimeout(timer);
  }, [notice]);

  useEffect(() => {
    function handleAppShortcuts(event: KeyboardEvent) {
      if (!(event.ctrlKey || event.metaKey)) return;
      if (event.key === "+" || event.key === "=" || event.key === "-" || event.key === "0") {
        event.preventDefault();
        return;
      }
      if (event.shiftKey && event.key === "Enter") {
        event.preventDefault();
        setFocusMode((current) => !current);
      }
    }
    function blockDesktopZoomWheel(event: WheelEvent) {
      if (event.ctrlKey || event.metaKey) event.preventDefault();
    }

    window.addEventListener("keydown", handleAppShortcuts, true);
    window.addEventListener("wheel", blockDesktopZoomWheel, { capture: true, passive: false });
    return () => {
      window.removeEventListener("keydown", handleAppShortcuts, true);
      window.removeEventListener("wheel", blockDesktopZoomWheel, true);
    };
  }, []);

  useEffect(() => {
    void refreshWorkspaceHealth();
    if (workspaces.length === 0) return;
    const timer = window.setInterval(() => void refreshWorkspaceHealth(), 30_000);
    return () => window.clearInterval(timer);
  }, [workspaces.length, refreshWorkspaceHealth]);

  useEffect(() => {
    if (workspaces.length === 0) {
      setSelectedWorkspaceId(null);
      if (!loading) setStartupProjectRestored(true);
      return;
    }

    setSelectedWorkspaceId((current) => {
      if (current && workspaces.some((workspace) => workspace.id === current)) return current;
      if (!startupProjectRestored) {
        const saved = window.localStorage.getItem("razedtunnel-last-workspace");
        if (saved && workspaces.some((workspace) => workspace.id === saved)) return saved;
      }
      return null;
    });
    if (!loading) setStartupProjectRestored(true);
  }, [workspaces]);

  useEffect(() => {
    if (activeView !== "changes") return;
    if (!selectedWorkspaceId) {
      setVersionTimeline({ records: [], currentVersionId: null });
      setActivityTimeline({ groups: [] });
      return;
    }
    Promise.all([getVersionTimeline(selectedWorkspaceId), getActivityTimeline(selectedWorkspaceId)])
      .then(([versions, activities]) => {
        setVersionTimeline(versions);
        setActivityTimeline(activities);
      })
      .catch((error) => setNotice(`Could not load history: ${errorMessage(error)}`));
  }, [activeView, selectedWorkspaceId]);

  useEffect(() => {
    if (!connectionStatus.running) return;

    const timer = window.setInterval(() => {
      getChatConnectionStatus().then(setConnectionStatus).catch(() => undefined);
    }, 5000);

    return () => window.clearInterval(timer);
  }, [connectionStatus.running, connectionStatus.ready]);

  useEffect(() => {
    if (!publicTunnelStatus.configured) return;
    let polling = false;
    const poll = () => {
      if (polling) return;
      polling = true;
      getPublicTunnelStatus()
        .then(setPublicTunnelStatus)
        .catch(() => undefined)
        .finally(() => { polling = false; });
    };
    const timer = window.setInterval(poll, 4000);
    return () => window.clearInterval(timer);
  }, [publicTunnelStatus.configured]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;
    let timer: number | null = null;
    let refreshRunning = false;
    let refreshQueued = false;

    const refreshAfterChange = async () => {
      if (disposed) return;
      if (refreshRunning) {
        refreshQueued = true;
        return;
      }
      refreshRunning = true;
      try {
        await refreshPendingChanges().catch(() => undefined);
        if (activeView === "changes" && selectedWorkspaceId) {
          await Promise.all([
            getVersionTimeline(selectedWorkspaceId),
            getActivityTimeline(selectedWorkspaceId),
          ]).then(([versions, activities]) => {
            setVersionTimeline(versions);
            setActivityTimeline(activities);
          }).catch(() => undefined);
        }
      } finally {
        refreshRunning = false;
        if (refreshQueued && !disposed) {
          refreshQueued = false;
          timer = window.setTimeout(() => {
            timer = null;
            void refreshAfterChange();
          }, 500);
        }
      }
    };

    listen("repotunnel://changes-updated", () => {
      if (disposed) return;
      if (timer !== null) window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        timer = null;
        void refreshAfterChange();
      }, 500);
    })
      .then((stopListening) => {
        if (disposed) stopListening();
        else unlisten = stopListening;
      })
      .catch(() => undefined);

    return () => {
      disposed = true;
      if (timer !== null) window.clearTimeout(timer);
      unlisten?.();
    };
  }, [activeView, selectedWorkspaceId, refreshPendingChanges]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;
    let timer: number | null = null;
    let refreshRunning = false;

    const refreshActivity = async () => {
      if (disposed || activeView !== "changes" || !selectedWorkspaceId || refreshRunning) return;
      refreshRunning = true;
      try {
        setActivityTimeline(await getActivityTimeline(selectedWorkspaceId));
      } catch {
        // Keep the last successful history snapshot during active AI work.
      } finally {
        refreshRunning = false;
      }
    };

    listen("repotunnel://activity-updated", () => {
      if (disposed || activeView !== "changes" || !selectedWorkspaceId) return;
      if (timer !== null) window.clearTimeout(timer);
      timer = window.setTimeout(() => {
        timer = null;
        void refreshActivity();
      }, 700);
    })
      .then((stopListening) => {
        if (disposed) stopListening();
        else unlisten = stopListening;
      })
      .catch(() => undefined);

    return () => {
      disposed = true;
      if (timer !== null) window.clearTimeout(timer);
      unlisten?.();
    };
  }, [activeView, selectedWorkspaceId]);

  useEffect(() => {
    if (!gatewayStatus.running) return;

    const timer = window.setInterval(() => {
      refreshPendingChanges().catch(() => undefined);
    }, 10_000);

    return () => window.clearInterval(timer);
  }, [gatewayStatus.running, refreshPendingChanges]);

  async function handleAddWorkspace() {
    setNotice(null);
    setAdding(true);

    try {
      const selected = await selectWorkspace();
      if (!selected) return;
      await addWorkspace(selected);
      await refresh();
      setActiveView("projects");
    } catch (error) {
      setNotice(`Could not add that project: ${errorMessage(error)}`);
    } finally {
      setAdding(false);
    }
  }

  async function handleCloneGitWorkspace(url: string) {
    const parentPath = await selectWorkspace();
    if (!parentPath) return;
    setNotice(null);
    try {
      const workspace = await cloneGitWorkspace(url, parentPath);
      await refresh();
      setSelectedWorkspaceId(workspace.id);
      setActiveView("projects");
      setNotice(`Cloned and connected ${workspace.name}.`);
    } catch (error) {
      const message = errorMessage(error);
      setNotice(`Could not clone repository: ${message}`);
      throw new Error(message);
    }
  }

  async function handleCreateProject(name: string) {
    setNotice(null);
    setCreatingProject(true);
    try {
      const workspace = await createProject(name);
      await refresh();
      setSelectedWorkspaceId(workspace.id);
      setShowNewProject(false);
      setActiveView("projects");
      setNotice(`Created ${workspace.name}. The project is ready for AI work.`);
    } catch (error) {
      setNotice(`Could not create that project: ${errorMessage(error)}`);
      throw error;
    } finally {
      setCreatingProject(false);
    }
  }

  async function handleRelocateWorkspace(workspace: Workspace) {
    setNotice(null);
    setRelocatingWorkspaceId(workspace.id);
    try {
      const selected = await selectWorkspace();
      if (!selected) return;
      const updated = await relocateWorkspace(workspace.id, selected);
      replaceWorkspace(updated);
      setNotice(`Project path repaired: ${updated.name}.`);
      await refreshWorkspaceHealth();
    } catch (error) {
      setNotice(`Could not repair that project path: ${errorMessage(error)}`);
    } finally {
      setRelocatingWorkspaceId(null);
    }
  }

  async function handleWorkspaceHealthRetry(workspaceId: string) {
    try {
      const health = await getWorkspaceHealth(workspaceId);
      setWorkspaceHealth((current) => ({ ...current, [workspaceId]: health }));
      setNotice(health.available ? "Project folder is available again." : (health.message ?? "Project folder is still unavailable."));
    } catch (error) {
      setNotice(`Could not recheck that project: ${errorMessage(error)}`);
    }
  }

  async function confirmWorkspaceRemoval() {
    if (!workspaceToRemove) return;

    setNotice(null);
    setRemovingId(workspaceToRemove.id);

    try {
      const removedWorkspaceId = workspaceToRemove.id;
      const updated = await removeWorkspace(removedWorkspaceId);
      setWorkspaces(updated);
      setGatewayStatus((current) => ({ ...current, workspaceCount: updated.length }));
      setWorkspaceToRemove(null);
    } catch (error) {
      setNotice(`Could not remove that project: ${errorMessage(error)}`);
    } finally {
      setRemovingId(null);
    }
  }

  function replaceWorkspace(updated: Workspace) {
    setWorkspaces((current) =>
      current.map((workspace) => (workspace.id === updated.id ? updated : workspace)),
    );
  }

  async function handleAccessChange(workspace: Workspace, accessMode: WorkspaceAccessMode) {
    setNotice(null);
    setUpdatingId(workspace.id);
    try {
      replaceWorkspace(await updateWorkspaceAccess(workspace.id, accessMode));
    } catch (error) {
      setNotice(`Could not change project access: ${errorMessage(error)}`);
    } finally {
      setUpdatingId(null);
    }
  }

  async function handleChangePolicyChange(
    workspace: Workspace,
    changePolicy: WorkspaceChangePolicy,
  ) {
    setNotice(null);
    setUpdatingId(workspace.id);
    try {
      replaceWorkspace(await updateWorkspaceChangePolicy(workspace.id, changePolicy));
      setNotice(
        changePolicy === "automatic"
          ? `${workspace.name}: AI edits will apply automatically with version protection.`
          : `${workspace.name}: AI edits will wait for local review.`,
      );
    } catch (error) {
      setNotice(`Could not change the AI edit policy: ${errorMessage(error)}`);
    } finally {
      setUpdatingId(null);
    }
  }

  async function handlePendingChange(
    change: ChangeRecord,
    action: "approve" | "reject",
  ) {
    setNotice(null);
    setChangeBusyId(change.id);
    try {
      if (action === "approve") {
        await approveChange(change.id);
        setNotice(`Applied ${change.primaryPath} and saved it to version history.`);
      } else {
        await rejectChange(change.id);
        setNotice(`Rejected ${change.primaryPath}.`);
      }
      await refreshChanges();
    } catch (error) {
      const verb = action === "approve" ? "apply" : "reject";
      setNotice(`Could not ${verb} that AI change: ${errorMessage(error)}`);
      await refreshChanges().catch(() => undefined);
    } finally {
      setChangeBusyId(null);
    }
  }

  async function handleCommandPolicyChange(workspace: Workspace, commandPolicy: CommandPolicy) {
    setNotice(null);
    setUpdatingId(workspace.id);
    try {
      replaceWorkspace(await updateWorkspaceCommandPolicy(workspace.id, commandPolicy));
    } catch (error) {
      setNotice(`Could not change the command policy: ${errorMessage(error)}`);
    } finally {
      setUpdatingId(null);
    }
  }

  async function handlePublicTunnelConfigure(provider: PublicTunnelProvider, credential: string, publicUrl?: string) {
    setNotice(null);
    setPublicTunnelBusy(true);
    try {
      setPublicTunnelStatus(await configurePublicTunnel(provider, credential, publicUrl));
      setGatewayStatus(await getGatewayStatus());
      setNotice(provider === "direct"
        ? "Direct HTTPS is prepared locally. The public route becomes usable after a routable IPv4/IPv6, router forwarding, and trusted TLS are in place."
        : `${provider === "cloudflare" ? "Cloudflare" : "ngrok"} public connection is ready. Copy the MCP URL and connect it in ChatGPT once.`);
    } catch (error) {
      setNotice(`Public connection failed: ${errorMessage(error)}`);
      setPublicTunnelStatus(await getPublicTunnelStatus().catch(() => initialPublicTunnelStatus));
      throw error;
    } finally {
      setPublicTunnelBusy(false);
    }
  }

  async function handleDirectCertificateProvision() {
    setNotice(null);
    setPublicTunnelBusy(true);
    try {
      setPublicTunnelStatus(await provisionDirectHttpsCertificate(false));
      setGatewayStatus(await getGatewayStatus());
      setNotice("Trusted Let’s Encrypt certificate loaded for Direct HTTPS.");
    } catch (error) {
      setNotice(`Could not provision the Direct HTTPS certificate: ${errorMessage(error)}`);
      setPublicTunnelStatus(await getPublicTunnelStatus().catch(() => initialPublicTunnelStatus));
    } finally {
      setPublicTunnelBusy(false);
    }
  }

  async function handlePublicTunnelRestart() {
    setNotice(null);
    setPublicTunnelBusy(true);
    try {
      setPublicTunnelStatus(await restartPublicTunnel());
      setGatewayStatus(await getGatewayStatus());
      setNotice("Public MCP connection restarted.");
    } catch (error) {
      setNotice(`Could not restart public connection: ${errorMessage(error)}`);
      setPublicTunnelStatus(await getPublicTunnelStatus().catch(() => initialPublicTunnelStatus));
    } finally {
      setPublicTunnelBusy(false);
    }
  }

  async function handlePublicTunnelForget() {
    setConfirmForgetPublicTunnel(true);
  }

  async function handleRevokeMcpAccess() {
    setConfirmRevokeMcpAccess(true);
  }

  async function performRevokeMcpAccess() {
    setNotice(null);
    setPublicTunnelBusy(true);
    try {
      await revokeMcpAccess();
      setNotice("MCP access revoked. ChatGPT will need to sign in with RepoTunnel again.");
    } catch (error) {
      setNotice(`Could not revoke MCP access: ${errorMessage(error)}`);
    } finally {
      setPublicTunnelBusy(false);
      setConfirmRevokeMcpAccess(false);
    }
  }

  async function performPublicTunnelForget() {
    setNotice(null);
    setPublicTunnelBusy(true);
    try {
      setPublicTunnelStatus(await clearPublicTunnel());
      setNotice("Public connection setup removed from this device.");
    } catch (error) {
      setNotice(`Could not remove public connection setup: ${errorMessage(error)}`);
    } finally {
      setPublicTunnelBusy(false);
      setConfirmForgetPublicTunnel(false);
    }
  }

  async function handleChatConnect(tunnelId: string, apiKey: string) {
    setNotice(null);
    setConnectionBusy(true);
    try {
      const status = await startChatConnection(tunnelId, apiKey);
      setConnectionStatus(status);
      setGatewayStatus(await getGatewayStatus());
    } catch (error) {
      setNotice(`ChatGPT connection failed: ${errorMessage(error)}`);
    } finally {
      setConnectionBusy(false);
    }
  }

  async function handleChatDisconnect() {
    setNotice(null);
    setConnectionBusy(true);
    try {
      setConnectionStatus(await stopChatConnection());
    } catch (error) {
      setNotice(`Could not stop the ChatGPT connection: ${errorMessage(error)}`);
    } finally {
      setConnectionBusy(false);
    }
  }

  function selectedWorkspace(): Workspace | null {
    if (!selectedWorkspaceId) return null;
    return workspaces.find((workspace) => workspace.id === selectedWorkspaceId) ?? null;
  }

  async function handleCreateCheckpoint() {
    const workspace = selectedWorkspace();
    if (!workspace) {
      setNotice("Open a project before creating a checkpoint.");
      return;
    }

    setNotice(null);
    setCheckpointBusy(true);
    try {
      const result = await createCheckpoint(workspace.id);
      setSafetyScanResult(null);
      setCheckpointResult(result);
    } catch (error) {
      setNotice(`Could not create checkpoint: ${errorMessage(error)}`);
    } finally {
      setCheckpointBusy(false);
    }
  }

  async function handleSafetyScan() {
    const workspace = selectedWorkspace();
    if (!workspace) {
      setNotice("Open a project before running a safety scan.");
      return;
    }

    setNotice(null);
    setSafetyBusy(true);
    try {
      const result = await runSafetyScan(workspace.id);
      setCheckpointResult(null);
      setSafetyScanResult(result);
    } catch (error) {
      setNotice(`Safety scan failed: ${errorMessage(error)}`);
    } finally {
      setSafetyBusy(false);
    }
  }

  async function handleAiAccessToggle() {
    setNotice(null);
    setAiAccessBusy(true);
    try {
      const result = await setAiAccessPaused(!aiAccessPaused);
      setAiAccessPausedState(result.paused);
      setNotice(result.paused
        ? "AI access paused. MCP clients cannot read or change approved project files until you resume access."
        : "AI access resumed. Connected MCP clients can use approved projects again.");
    } catch (error) {
      setNotice(`Could not update AI access: ${errorMessage(error)}`);
    } finally {
      setAiAccessBusy(false);
    }
  }

  async function handleRestoreVersion(versionId: string | null) {
    const workspace = selectedWorkspace();
    if (!workspace) {
      setNotice("Select a project before restoring version history.");
      return;
    }

    setNotice(null);
    setVersionBusy(true);
    try {
      const result = await restoreVersion(workspace.id, versionId);
      setVersionTimeline(await getVersionTimeline(workspace.id));
      await refreshChanges().catch(() => undefined);
      setNotice(`Version restored. ${result.restoredFiles} files restored${result.removedFiles ? `, ${result.removedFiles} newer files removed` : ""}. Later versions remain saved.`);
    } catch (error) {
      setNotice(`Could not restore version: ${errorMessage(error)}`);
      throw error;
    } finally {
      setVersionBusy(false);
    }
  }

  async function handleClearVersionHistory() {
    const workspace = selectedWorkspace();
    if (!workspace) {
      setNotice("Select a project before clearing version history.");
      return;
    }

    setNotice(null);
    try {
      const result = await clearVersionHistory(workspace.id);
      await refreshChanges();
      const removedRecords = result.removedChanges + result.removedActivities + result.removedOperationalRecords;
      setNotice(
        `History cleared. Removed ${result.removedVersions} saved ${result.removedVersions === 1 ? "version" : "versions"} and ${removedRecords} activity ${removedRecords === 1 ? "record" : "records"}. Current project files were not changed.`,
      );
    } catch (error) {
      setNotice(`Could not clear history: ${errorMessage(error)}`);
      throw error;
    }
  }

  // The gateway UI no longer owns an editor or project file-tree session.
  function handleGatewayToggle() {
    if (gatewayStatus.running) {
      setConfirmStopGateway(true);
      return;
    }
    void performGatewayToggle(false);
  }

  async function performGatewayToggle(stop: boolean) {
    setNotice(null);
    setGatewayBusy(true);
    try {
      const status = stop ? await stopGateway() : await startGateway();
      setGatewayStatus(status);
      setPublicTunnelStatus(await getPublicTunnelStatus());
      setConnectionStatus(await getChatConnectionStatus());
      setNotice(stop ? "Gateway stopped cleanly." : "Gateway started.");
    } catch (error) {
      setNotice(`Gateway action failed: ${errorMessage(error)}`);
    } finally {
      setGatewayBusy(false);
      setConfirmStopGateway(false);
    }
  }

  function renderPage() {
    if (activeView === "projects") {
      return (
        <div className="page-stack projects-page-stack">
          <WorkspaceList
            workspaces={workspaces}
            adding={adding}
            removingId={removingId}
            updatingId={updatingId}
            workspaceHealth={workspaceHealth}
            relocatingWorkspaceId={relocatingWorkspaceId}
            onAdd={handleAddWorkspace}
            onClone={handleCloneGitWorkspace}
            onRemove={setWorkspaceToRemove}
            onAccessChange={handleAccessChange}
            onChangePolicyChange={handleChangePolicyChange}
            onCommandPolicyChange={handleCommandPolicyChange}
            onRelocate={(workspace) => void handleRelocateWorkspace(workspace)}
            onRetryHealth={(workspaceId) => void handleWorkspaceHealthRetry(workspaceId)}
          />
          {/* Development-oriented project panels are preserved in source but not shown in the gateway UI. */}
        </div>
      );
    }

    if (activeView === "connections") {
      return (
        <div className="page-stack">
          <GatewayPanel
            status={gatewayStatus}
            connection={connectionStatus}
            publicTunnel={publicTunnelStatus}
            aiAccessPaused={aiAccessPaused}
            busy={gatewayBusy}
            onToggle={handleGatewayToggle}
          />
          <PublicTunnelPanel
            status={publicTunnelStatus}
            gatewayRunning={gatewayStatus.running}
            busy={publicTunnelBusy}
            onConfigure={handlePublicTunnelConfigure}
            onRestart={handlePublicTunnelRestart}
            onProvisionCertificate={handleDirectCertificateProvision}
            onRevoke={handleRevokeMcpAccess}
            onForget={handlePublicTunnelForget}
          />
          <details className="connection-advanced-details">
            <summary>Advanced: OpenAI Secure Tunnel</summary>
          <ChatConnectionPanel
            status={connectionStatus}
            gatewayRunning={gatewayStatus.running}
            busy={connectionBusy}
            aiAccessPaused={aiAccessPaused}
            onConnect={handleChatConnect}
            onDisconnect={handleChatDisconnect}
          />
          </details>
        </div>
      );
    }

    return null;
  }

  return (
    <div className="desktop-app">
      <WindowChrome />
      <div className={`desktop-shell ${focusMode ? "focus-mode" : ""}`}>
        <AppSidebar
          activeView={activeView}
          pendingCount={pendingCount}
          onNavigate={setActiveView}
        />


        <main className="main-view">
          <AppHeader
            view={activeView}
            gatewayRunning={gatewayStatus.running}
            connectionReady={publicTunnelStatus.ready || connectionStatus.ready}
            aiAccessPaused={aiAccessPaused}
            focusMode={focusMode}
            onToggleFocusMode={() => setFocusMode((current) => !current)}
          />

          {notice ? (
            <div className="notice desktop-notice" role="alert">
              <span>{notice}</span>
              <button type="button" onClick={() => setNotice(null)} aria-label="Dismiss message">×</button>
            </div>
          ) : null}

          <AppErrorBoundary
            resetKey={`${activeView}:${selectedWorkspaceId ?? "none"}`}
            onGoHome={() => setActiveView("projects")}
          >
          {loading ? (
            <section className="loading-state" aria-live="polite">
              <div className="loader" aria-hidden="true" />
              <span>Loading RazedTunnel…</span>
            </section>
          ) : (
            <div className="page-scroll">
              <div className="page-content">{renderPage()}</div>
            </div>
          )}
          </AppErrorBoundary>
        </main>
      </div>


      {workspaceToRemove ? (
        <ConfirmationDialog
          title="Remove project?"
          message={`RepoTunnel will forget “${workspaceToRemove.name}”. Nothing inside the project folder will be deleted.`}
          confirmLabel="Remove project"
          busy={removingId === workspaceToRemove.id}
          onCancel={() => setWorkspaceToRemove(null)}
          onConfirm={confirmWorkspaceRemoval}
        />
      ) : null}

      {confirmStopGateway ? (
        <ConfirmationDialog
          title="Stop the MCP gateway?"
          message="This cleanly stops the local MCP gateway and managed remote connections. Your projects, History, and saved public setup remain unchanged."
          confirmLabel="Stop gateway"
          busy={gatewayBusy}
          busyLabel="Stopping…"
          onCancel={() => setConfirmStopGateway(false)}
          onConfirm={() => void performGatewayToggle(true)}
        />
      ) : null}

      {confirmRevokeMcpAccess ? (
        <ConfirmationDialog
          title="Revoke MCP access?"
          message="This immediately invalidates the current MCP access and refresh credentials. Your public endpoint, tunnel-provider setup, projects, and history remain unchanged. ChatGPT will need to sign in with RepoTunnel again when it next connects."
          confirmLabel="Revoke access"
          busy={publicTunnelBusy}
          busyLabel="Revoking…"
          onCancel={() => setConfirmRevokeMcpAccess(false)}
          onConfirm={() => void performRevokeMcpAccess()}
        />
      ) : null}

      {confirmForgetPublicTunnel ? (
        <ConfirmationDialog
          title="Forget public connection setup?"
          message="This removes the saved public-tunnel credential and endpoint configuration from this device. Your project files and RepoTunnel history are not affected."
          confirmLabel="Forget setup"
          busy={publicTunnelBusy}
          busyLabel="Removing…"
          onCancel={() => setConfirmForgetPublicTunnel(false)}
          onConfirm={() => void performPublicTunnelForget()}
        />
      ) : null}

    </div>
  );
}

export default App;
