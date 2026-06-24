#!/usr/bin/env sh
# Stage-0 evidence gate for gam (TypeScript/Tauri). The Stop/SubagentStop hook runs
# this and BLOCKS turn-end on a non-zero exit. Keep it fast + deterministic.
# Native `tauri build` and Playwright e2e are intentionally excluded (host/display).
set -e
echo "[awh-gate] pnpm lint…"
pnpm lint
echo "[awh-gate] pnpm test (vitest run)…"
pnpm test
echo "[awh-gate] tsc --noEmit…"
pnpm exec tsc --noEmit -p tsconfig.json
# Alternative single command if you prefer your canonical check: pnpm ready
