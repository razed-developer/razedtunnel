import { invoke } from "@tauri-apps/api/core";
import type {
  ActivityTimeline,
  AiAccessStatus,
  ChatConnectionStatus,
  CommandPolicy,
  GatewayStatus,
  PublicTunnelProvider,
  PublicTunnelStatus,
  Workspace,
  WorkspaceHealth,
  WorkspaceAccessMode,
  WorkspaceChangePolicy,
  RuntimeDiagnostics,
} from "../types";

export async function selectWorkspace(): Promise<string | null> {
  return invoke<string | null>("select_workspace");
}

export async function listWorkspaces(): Promise<Workspace[]> {
  return invoke<Workspace[]>("list_workspaces");
}

export async function getWorkspaceHealth(workspaceId: string): Promise<WorkspaceHealth> {
  return invoke<WorkspaceHealth>("get_workspace_health", { workspaceId });
}

export async function relocateWorkspace(workspaceId: string, path: string): Promise<Workspace> {
  return invoke<Workspace>("relocate_workspace", { workspaceId, path });
}

export async function addWorkspace(path: string): Promise<Workspace> {
  return invoke<Workspace>("add_workspace", { path });
}

export async function cloneGitWorkspace(url: string, parentPath: string): Promise<Workspace> {
  return invoke<Workspace>("clone_git_workspace", { url, parentPath });
}

export async function removeWorkspace(id: string): Promise<Workspace[]> {
  return invoke<Workspace[]>("remove_workspace", { id });
}

export async function updateWorkspaceAccess(
  id: string,
  accessMode: WorkspaceAccessMode,
): Promise<Workspace> {
  return invoke<Workspace>("update_workspace_access", { id, accessMode });
}

export async function updateWorkspaceChangePolicy(
  id: string,
  changePolicy: WorkspaceChangePolicy,
): Promise<Workspace> {
  return invoke<Workspace>("update_workspace_change_policy", { id, changePolicy });
}

export async function updateWorkspaceCommandPolicy(
  id: string,
  commandPolicy: CommandPolicy,
): Promise<Workspace> {
  return invoke<Workspace>("update_workspace_command_policy", { id, commandPolicy });
}

export async function getAiAccessStatus(): Promise<AiAccessStatus> {
  return invoke<AiAccessStatus>("get_ai_access_status");
}

export async function setAiAccessPaused(paused: boolean): Promise<AiAccessStatus> {
  return invoke<AiAccessStatus>("set_ai_access_paused", { paused });
}

export async function getGatewayStatus(): Promise<GatewayStatus> {
  return invoke<GatewayStatus>("get_gateway_status");
}

export async function startGateway(): Promise<GatewayStatus> {
  return invoke<GatewayStatus>("start_gateway");
}

export async function stopGateway(): Promise<GatewayStatus> {
  return invoke<GatewayStatus>("stop_gateway");
}

export async function getPublicTunnelStatus(): Promise<PublicTunnelStatus> {
  return invoke<PublicTunnelStatus>("get_public_tunnel_status");
}

export async function configurePublicTunnel(
  provider: PublicTunnelProvider,
  credential: string,
  publicUrl?: string,
): Promise<PublicTunnelStatus> {
  return invoke<PublicTunnelStatus>("configure_public_tunnel", {
    provider,
    credential,
    publicUrl: publicUrl?.trim() ? publicUrl.trim() : null,
  });
}

export async function restartPublicTunnel(): Promise<PublicTunnelStatus> {
  return invoke<PublicTunnelStatus>("restart_public_tunnel");
}

export async function provisionDirectHttpsCertificate(staging = false): Promise<PublicTunnelStatus> {
  return invoke<PublicTunnelStatus>("provision_direct_https_certificate", { staging });
}

export async function clearPublicTunnel(): Promise<PublicTunnelStatus> {
  return invoke<PublicTunnelStatus>("clear_public_tunnel");
}

export async function revokeMcpAccess(): Promise<void> {
  return invoke<void>("revoke_mcp_access");
}

export async function getChatConnectionStatus(): Promise<ChatConnectionStatus> {
  return invoke<ChatConnectionStatus>("get_chat_connection_status");
}

export async function startChatConnection(
  tunnelId: string,
  apiKey: string,
): Promise<ChatConnectionStatus> {
  return invoke<ChatConnectionStatus>("start_chat_connection", { tunnelId, apiKey });
}

export async function stopChatConnection(): Promise<ChatConnectionStatus> {
  return invoke<ChatConnectionStatus>("stop_chat_connection");
}

export async function getRuntimeDiagnostics(): Promise<RuntimeDiagnostics> {
  return invoke<RuntimeDiagnostics>("get_runtime_diagnostics");
}

export async function setLaunchAtLogin(enabled: boolean): Promise<RuntimeDiagnostics> {
  return invoke<RuntimeDiagnostics>("set_launch_at_login", { enabled });
}

/** Read-only history for the monitoring dashboard. */
export async function getActivityTimeline(): Promise<ActivityTimeline> {
  return invoke<ActivityTimeline>("get_activity_timeline");
}
