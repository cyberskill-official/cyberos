# AGENTS.md

This repository runs **CyberOS**. Canonical agent instructions: `.cyberos/AGENT-ENTRY.md`.

Work is tasks; HITL is required at the two human-acceptance gates; run gates with `bash .cyberos/cuo/gates/run-gates.sh`. Never push, deploy, or merge without an explicit operator instruction.

Memory (BRAIN): protocol at `.cyberos/memory/AGENTS.md` (normative source on this platform: `modules/memory/cyberos/data/AGENTS.md`); store at `.cyberos/memory/store/`.

<!-- cyberos-agent-spine (managed by cyberos install; edit above/below this marker) -->

## Cursor Cloud specific instructions

This is the CyberOS source monorepo with three runnable stacks. Standard commands are documented in `docs/reference/getting-started.md` and `docs/deploy/local-dev-and-testing.md`; `services/Makefile` has the Rust lint/build/test targets and `scripts/local_verify.sh` is the full DB-backed CI mirror. The notes below are the non-obvious environment gotchas.

Prereqs already baked into the VM snapshot (do not reinstall): Python `.venv` at repo root, Node 24.18.0 via nvm, Rust 1.88.0 via rustup, Docker engine + compose, `libssl-dev`/`pkg-config`. The update script only refreshes repo dependencies.

### Python modules (`modules/memory`, `modules/cuo`)
- Use the repo-root venv: `.venv/bin/python`, `.venv/bin/cyberos`, `.venv/bin/pytest`.
- `modules/memory` installs editable (`cyberos` CLI). Run its suite from `modules/memory`: `.venv/bin/python -m pytest`.
- `modules/cuo`'s editable install FAILS (its `pyproject.toml` sets `readme = "README.md"` but that file does not exist; CI tolerates this with `|| true`). Run it via source path instead: `PYTHONPATH=modules/cuo .venv/bin/python -m cuo.cli list-personas`, and its suite with `cd modules/cuo && PYTHONPATH=. /workspace/.venv/bin/python -m pytest`.
- `cyberos doctor` flags a store on `/tmp` (ephemeral) and non-canonical top-level folders as errors — use a store dir under the repo for a clean run.

### Web app (`apps/web`) — Vite + React SPA
- Requires Node 24 (`engines: >=24 <25`). The VM has a `/exec-daemon/node` (v22) shim EARLIER on PATH than nvm, so plain `node`/`npm` resolve to v22. Prepend nvm first: `export PATH="$HOME/.nvm/versions/node/v24.18.0/bin:$PATH"` (or call the absolute `$HOME/.nvm/versions/node/v24.18.0/bin/npm`).
- Dev server: `npm run dev` serves on port 5173 but binds `localhost` only — reach it at `http://localhost:5173/` (NOT `http://127.0.0.1:5173/`).
- Vite proxies `/v1/auth`,`/v1/admin` → `127.0.0.1:7700` (auth service) and `/v1/chat` → `127.0.0.1:7720` (chat service). Login and chat only work when those Rust services are running.
- `npm run build` writes into `apps/console/web` (a tracked path) via `emptyOutDir`. For a non-destructive check use `npx tsc --noEmit` (the typecheck gate) instead of a full build.

### Rust services (`services/`) — need Postgres + Redis
- `cargo` in `services/` auto-selects Rust 1.88 via `rust-toolchain.toml`.
- Docker is NOT auto-started on VM boot. Start it once per session: `sudo dockerd &` (daemon.json already sets `fuse-overlayfs` + disables the containerd-snapshotter for Docker 29). All `docker` commands need `sudo`.
- Bring up infra + migrations: `cd services && sudo docker compose -f dev/docker-compose.yml up -d --build`, then apply migrations in order `auth mcp-gateway memory eval chat ai-gateway email proj` (see `scripts/local_verify.sh` for the exact loop — the raw-psql loop re-runs safely, treating "already exists" as applied).
- Always export `DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos` and `REDIS_URL=redis://127.0.0.1:6379` before building/testing/running.
- DB-backed tests are `#[ignore]`d; run with `-- --include-ignored --test-threads=1` (they share one DB, not parallel-safe). `cyberos-ai-gateway` cost-hold tests shell out to the memory Writer, so set `PYTHONPATH=/workspace/modules/memory`.
- ai-gateway: the `cyberos-gateway` binary uses `RouterBackend` (the docs' "echo backend" is outdated). `POST /v1/chat` for `chat.smart` routes to a local OpenAI-compatible model at `http://localhost:1234` (LM Studio/Ollama); with no model running it returns 502 (`/healthz` still returns 200). For a local smoke without a real model, run any OpenAI-compatible stub on `:1234`.
- Full-stack web login: build+run `cyberos-auth` (`AUTH_LISTEN_ADDR=127.0.0.1:7700`), bootstrap an admin (`AUTH_BOOTSTRAP_EMAIL=… AUTH_BOOTSTRAP_PASSWORD=…` ≥12 chars via `cyberos-auth-bootstrap`), then log into the web console with tenant `root`, handle `@root`. Auth needs only `DATABASE_URL` (plus `CYBEROS_DEPLOYMENT_TIER=development`).
