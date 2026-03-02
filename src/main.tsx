import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

// Global error handlers to catch crashes before refresh
window.addEventListener('error', (event) => {
  console.error('[💥 GLOBAL ERROR]', event.message, event.error);
  if (event.error?.stack) {
    console.error('[STACK]', event.error.stack);
  }
});

window.addEventListener('unhandledrejection', (event) => {
  console.error('[💥 UNHANDLED REJECTION]', event.reason);
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
