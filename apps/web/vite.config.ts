import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { foglampAiPlugin } from "./vite-plugin-foglamp-ai";

// CyberOS web console.
//
// base "/" - the app IS the site root (https://os.cyberskill.world/); the legacy /web/ prefix 308-redirects
// here (Caddyfile.p0). Module views live at /chat, /dashboard, ... (client routes with an SPA fallback in
// Caddy). API calls are ORIGIN-relative (/v1/auth, /v1/chat), so they resolve to the same Caddy origin.
//
// Dev: `npm run dev` serves http://localhost:5173/ and proxies the API to the local services
// (auth :7700, chat :7720), including the chat websocket upgrade. Chat AI routes are intercepted by
// foglampAiPlugin and run through the Vercel AI SDK + Foglamp (HUD + traces) before the proxy.
export default defineConfig({
  base: "/",
  // Monorepo root holds FOGLAMP_API_KEY / AI_GATEWAY_URL from `foglamp login`.
  envDir: "../..",
  plugins: [react(), foglampAiPlugin()],
  server: {
    port: 5173,
    proxy: {
      "/v1/auth": { target: "http://127.0.0.1:7700", changeOrigin: true },
      "/v1/admin": { target: "http://127.0.0.1:7700", changeOrigin: true },
      "/.well-known": { target: "http://127.0.0.1:7700", changeOrigin: true },
      "/v1/chat": { target: "http://127.0.0.1:7720", changeOrigin: true, ws: true },
    },
  },
  // Build INTO the console folder (apps/console/web) so the existing /srv/console Caddy mount serves it -
  // no separate volume, and it dodges the repo's global `dist/` .gitignore (apps/web/dist was being ignored,
  // so it never reached the VPS). emptyOutDir clears apps/console/web on each build.
  build: { outDir: "../console/web", emptyOutDir: true },
});
