import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Vite config for Tauri 2.x. Port 1420 is fixed + strict so the tauri dev
// process can connect deterministically. HMR overlay is disabled because the
// Tauri devtools panel already surfaces errors with the native window context.
export default defineConfig(async () => ({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: false,
    hmr: {
      overlay: false,
    },
    watch: {
      // Tauri reloads on Rust changes; we don't need Vite watching src-tauri/.
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "esnext",
    sourcemap: false,
    minify: "esbuild",
    outDir: "dist",
    emptyOutDir: true,
  },
}));
