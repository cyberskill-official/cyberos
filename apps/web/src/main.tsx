import React from "react";
import { createRoot } from "react-dom/client";
import { FoglampHUD } from "foglamp/hud";
import { App } from "./App";
import { UpdateBanner } from "./components/UpdateBanner";
import { AuthProvider } from "./lib/auth";
import "./styles.css";

const el = document.getElementById("root");
if (!el) throw new Error("#root not found");

createRoot(el).render(
  <React.StrictMode>
    <AuthProvider>
      <App />
      {/* Cross-surface "new build available" prompt; mounted at the root so it shows on any page. */}
      <UpdateBanner />
      {/* Dev-only overlay: inert unless the Vite Foglamp broker is running. */}
      <FoglampHUD />
    </AuthProvider>
  </React.StrictMode>,
);
