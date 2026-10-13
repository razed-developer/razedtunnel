import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import AppErrorBoundary from "./components/AppErrorBoundary";
import "./styles.css";
import "./workspace.css";
import "./theme.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("Application root element was not found.");
}

createRoot(root).render(
  <StrictMode>
    <AppErrorBoundary compact>
      <App />
    </AppErrorBoundary>
  </StrictMode>,
);
