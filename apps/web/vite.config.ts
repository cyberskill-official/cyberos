import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// CyberOS web console.
//
// base "/web/" - the build is served additively at https://<origin>/web/ alongside the existing static
// console (apps/console/app.html), so shipping it never disturbs the live team. API calls are ORIGIN-relative
// (/v1/auth, /v1/chat), independent of this base, so they resolve to the same Caddy origin in production.
//
// Dev: `npm run dev` serves http://localhost:5173/web/ and proxies the API to the local services
// (auth :7700, chat :7720), including the chat websocket upgrade.
export default defineConfig({
  base: "/web/",
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      "/v1/auth": { target: "http://127.0.0.1:7700", changeOrigin: true },
      "/v1/admin": { target: "http://127.0.0.1:7700", changeOrigin: true },
      "/.well-known": { target: "http://127.0.0.1:7700", changeOrigin: true },
      "/v1/chat": { target: "http://127.0.0.1:7720", changeOrigin: true, ws: true },
    },
  },
  build: { outDir: "dist", emptyOutDir: true },
});
