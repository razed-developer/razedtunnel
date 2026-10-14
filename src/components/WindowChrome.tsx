import { getCurrentWindow } from "@tauri-apps/api/window";

const appWindow = getCurrentWindow();

function WindowChrome() {
  return (
    <header className="r-window-chrome" data-tauri-drag-region>
      <div className="r-window-title" data-tauri-drag-region>RazedTunnel</div>
      <div className="r-window-controls">
        <button type="button" aria-label="Minimize" title="Minimize" onClick={() => appWindow.minimize()}>
          <span className="r-window-minimize" />
        </button>
        <button type="button" aria-label="Maximize" title="Maximize" onClick={() => appWindow.toggleMaximize()}>
          <span className="r-window-maximize" />
        </button>
        <button className="r-window-close" type="button" aria-label="Close" title="Close" onClick={() => appWindow.close()}>
          <span>×</span>
        </button>
      </div>
    </header>
  );
}

export default WindowChrome;
