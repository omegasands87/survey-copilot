import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";

async function bootstrap() {
  const rootElement = document.getElementById("root");
  if (!rootElement) throw new Error("Root element was not found.");

  const root = ReactDOM.createRoot(rootElement);
  const isUiPreview = new URLSearchParams(window.location.search).get("preview") === "1";

  if (isUiPreview) {
    // Keep the web preview isolated: it loads no desktop app/services/IPC module.
    document.title = "Survey Copilot — UI Preview";
    const { PreviewApp } = await import("./preview/PreviewApp");
    root.render(
      <React.StrictMode>
        <PreviewApp />
      </React.StrictMode>,
    );
    return;
  }

  const { default: App } = await import("./App");
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}

void bootstrap();
