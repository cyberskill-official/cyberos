import React from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { AuthProvider } from "./lib/auth";
import "./styles.css";

const el = document.getElementById("root");
if (!el) throw new Error("#root not found");

createRoot(el).render(
  <React.StrictMode>
    <AuthProvider>
      <App />
    </AuthProvider>
  </React.StrictMode>,
);
