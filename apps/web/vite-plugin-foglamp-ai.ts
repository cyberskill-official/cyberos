import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadEnv, type Plugin } from "vite";

const PLUGIN_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(PLUGIN_DIR, "../..");

function applyEnvFile(filePath: string): void {
  if (!existsSync(filePath)) return;
  for (const line of readFileSync(filePath, "utf8").split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const eq = trimmed.indexOf("=");
    if (eq <= 0) continue;
    const key = trimmed.slice(0, eq).trim();
    let val = trimmed.slice(eq + 1).trim();
    if (
      (val.startsWith('"') && val.endsWith('"')) ||
      (val.startsWith("'") && val.endsWith("'"))
    ) {
      val = val.slice(1, -1);
    }
    if (process.env[key] === undefined) process.env[key] = val;
  }
}

function loadFoglampEnv(mode: string): void {
  // Prefer an explicit root .env from `foglamp login`, then apps/web/.env.
  applyEnvFile(path.join(REPO_ROOT, ".env"));
  applyEnvFile(path.join(REPO_ROOT, `.env.${mode}`));
  applyEnvFile(path.join(PLUGIN_DIR, ".env"));
  applyEnvFile(path.join(PLUGIN_DIR, `.env.${mode}`));
  // Vite's loader as a backstop (also picks up .env.local).
  for (const dir of [REPO_ROOT, PLUGIN_DIR]) {
    const env = loadEnv(mode, dir, "");
    for (const [k, v] of Object.entries(env)) {
      if (process.env[k] === undefined) process.env[k] = v;
    }
  }
}

/**
 * Dev-only: serve chat AI routes (`/v1/chat/.../ai/*`, `/v1/chat/translate`)
 * through the Vercel AI SDK + Foglamp so local runs produce traces and the HUD
 * broker starts. Production still hits the Rust chat → ai-gateway path via Caddy.
 */
export function foglampAiPlugin(): Plugin {
  return {
    name: "cyberos-foglamp-ai",
    config(_userConfig, { mode }) {
      loadFoglampEnv(mode);
    },
    async configureServer(server) {
      // Dynamic import so foglamp() runs only after env is loaded above.
      const { attachFoglampAiMiddleware } = await import("./server/ai-dev-middleware");
      attachFoglampAiMiddleware(server);
    },
  };
}
