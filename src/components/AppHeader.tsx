import type { ReactNode } from "react";
import type { AppView } from "./AppSidebar";
import { NavIcon } from "./AppSidebar";

type AppHeaderProps = {
  view: AppView;
  themeControl: ReactNode;
  gatewayRunning: boolean;
  connectionReady: boolean;
  aiAccessPaused: boolean;
};

function AppHeader({ view, themeControl, gatewayRunning, connectionReady, aiAccessPaused }: AppHeaderProps) {
  return (
    <header className="r-page-header">
      <div className="r-page-tab active">
        <NavIcon name={view === "home" ? "home" : view === "connections" ? "link" : view === "settings" ? "settings" : "folder"} size={18} />
        <h1>{view === "home" ? "Home" : view === "connections" ? "Connections" : view === "settings" ? "Settings" : "Projects"}</h1>
      </div>
      <div className="r-page-statuses" aria-label="Connection status">
        {themeControl}
        {aiAccessPaused ? (
          <span className="r-toolbar-paused" title="MCP project access is blocked until you resume AI access">
            <NavIcon name="pause" size={14} />
            <span>AI access paused</span>
          </span>
        ) : null}
        <span className="r-toolbar-status" title={gatewayRunning ? "Local MCP gateway online" : "Local MCP gateway offline"}>
          <NavIcon name="gateway" size={15} />
          <span>Gateway</span>
          <span className={`r-toolbar-dot ${gatewayRunning ? "online" : "offline"}`} />
        </span>
        <span className="r-toolbar-status" title={connectionReady ? "Remote connection active" : "No managed remote connection"}>
          <NavIcon name="remote" size={15} />
          <span>Remote</span>
          <span className={`r-toolbar-dot ${connectionReady ? "online" : ""}`} />
        </span>
      </div>
    </header>
  );
}

export default AppHeader;
