# Run the core modules locally and verify them one by one

This is the step after the CAF absorption and before building the remaining modules: stand the current modules up locally in Docker, then run each one through the real verification chain, one at a time. It is the local-dev loop, not the VPS go-live (that is `docs/deploy/cyberos-core-deploy.md`, for later).

## How CyberOS "does work" (read these two first)

- `modules/cuo/chief-technology-officer/workflows/ship-tasks.md` - the ~30-step author/audit chain that drives one task from `ready_to_implement` to `done`. The last gates are step 28 `awh-gate` (rerun the tests) and step 28.5 `caf-gate` (rerun the target's own build/lint/test + audit). The `testing -> done` flip requires `awh GREEN AND caf CLEAN`.
- `website/docs/architecture/verification-gate.html` - why the gate exists (separate the grader from the author) and how it is wired (CUO workflow, pre-commit, CI, merge gate).

"Test a module" here means: run those same two gates against that module by hand. That is exactly what the workflow does automatically at 28 / 28.5.

## Step 0 - prerequisites (once)

```bash
# Toolchains
rustup toolchain install 1.88.0          # workspace floor (services/Cargo.toml)
# Python env for modules/cuo and modules/memory (use your venv of choice)

# The two gates, installed from the vendored copies (no external repos needed)
pip install -e tools/awh                  # provides `awh`
pip install -e tools/caf                  # provides `code-audit-validate` (caf_gate.sh also runs from source)
```

## Step 1 - local infrastructure in Docker

```bash
cd services/dev && docker compose up -d && cd -     # Postgres 16 (Apache AGE + pgvector) :5432, Redis :6379
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
```

The `ai` module's tests also use this Redis at `127.0.0.1:6379`. Create the per-service databases and run each service's sqlx migrations (the names mirror the `*_DB` keys in `deploy/vps/.env.local`):

```bash
for db in cyberos_auth cyberos_memory cyberos_proj; do
  psql "$DATABASE_URL" -c "CREATE DATABASE \"$db\";" 2>/dev/null || true
done
# cargo install sqlx-cli --no-default-features --features rustls,postgres
DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos_auth   sqlx migrate run --source services/auth/migrations
DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos_memory sqlx migrate run --source services/memory/migrations
DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos_proj   sqlx migrate run --source services/proj/migrations
```

There is no single all-services compose in the repo (only the infra compose above and `services/chat/Dockerfile`). Run the Rust services with `cargo run`, chat from its Dockerfile, and cuo from Python - see Step 2.

## Step 2 - run each core module locally

Order follows the dependencies: AUTH (identity) first, MEMORY (the audit chain everything writes to), then PROJ and SKILL, then CHAT, then CUO (it orchestrates the rest).

```bash
# AUTH - HTTP service (services/auth/src/main.rs), binds AUTH_LISTEN_ADDR
AUTH_DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos_auth \
AUTH_LISTEN_ADDR=0.0.0.0:8090 AUTH_JWT_ISSUER=http://localhost:8090 \
AUTH_WEBAUTHN_RP_ID=localhost AUTH_WEBAUTHN_RP_ORIGIN=http://localhost:8090 \
AUTH_CURSOR_SIGNING_SECRET=$(openssl rand -base64 48) \
AUTH_BOOTSTRAP_EMAIL=admin@local AUTH_BOOTSTRAP_PASSWORD=$(openssl rand -base64 18) \
  cargo run -p cyberos-auth

# MEMORY - HTTP service (services/memory/src/main.rs); needs an embeddings endpoint
#          (MEMORY_EMBED_URL -> services/embed-sidecar, run it too)
MEMORY_DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos_memory \
MEMORY_LISTEN_ADDR=0.0.0.0:7700 MEMORY_EMBED_URL=http://localhost:5050 \
  cargo run -p cyberos-memory

# SKILL - skill-broker has a [[bin]]
cargo run -p cyberos-skill-broker

# PROJ - library (cyberos_proj, no main.rs): its HTTP surface is served by a host binary.
#        Confirm which binary embeds it; for verification you exercise it through its tests (Step 3).

# CHAT - separate service, build + run from its Dockerfile
docker build -t cyberos-chat services/chat && docker run --rm -p 7800:7800 cyberos-chat
#   or: cd services && make chat-build

# CUO - Python orchestration runtime (modules/cuo); install then run its console script
pip install -e modules/cuo    # exposes the [project.scripts] entrypoint (read pyproject.toml for the name)
```

## Step 3 - verify each module, one at a time

For each module, run the same two gates the workflow runs at step 28 and 28.5. If a module has no sealed awh baseline yet, capture and lock it once (the bootstrap script does every module in one go).

```bash
# One-time: capture + seal every module's awh baseline (green/red report per module)
bash scripts/awh_bootstrap_waves.sh

# Then, per module - the loop you repeat for AUTH, MEMORY, SKILL, PROJ, CHAT, CUO:
awh eval modules/<m>/.awh/goldenset.yaml --base-dir . --seeds 1 \
    --baseline modules/<m>/.awh/eval-baseline.json --max-regression 0.0      # must be GREEN
bash scripts/caf_gate.sh <m>                                                 # must be CLEAN
```

GREEN + CLEAN means that module passes both gates exactly as it would inside a real `ship-tasks` run. RED or a dirty gate is the signal to fix that module before moving on.

| Module | Run locally | awh golden set | caf profile | Infra |
|---|---|---|---|---|
| AUTH | `cargo run -p cyberos-auth` | `modules/auth/.awh/goldenset.yaml` (`cargo test -p cyberos-auth`) | `modules/auth/audit-profile.yaml` | Postgres |
| MEMORY | `cargo run -p cyberos-memory` (+ embed-sidecar) | `modules/memory/.awh/goldenset.yaml` (pytest + `cargo test -p cyberos-memory`) | `modules/memory/audit-profile.yaml` | Postgres |
| SKILL | `cargo run -p cyberos-skill-broker` | `modules/skill/.awh/goldenset.yaml` (`cargo test -p cyberos-skill-broker`) | `modules/skill/audit-profile.yaml` | - |
| PROJ | served via a host binary (confirm) | `modules/proj/.awh/goldenset.yaml` (`cargo test -p cyberos-proj`) | `modules/proj/audit-profile.yaml` | Postgres |
| CHAT | `docker build/run services/chat` | `modules/chat/.awh/goldenset.yaml` (`make chat-verify`) | `modules/chat/audit-profile.yaml` | own Postgres |
| CUO | `pip install -e modules/cuo` + console script | `modules/cuo/.awh/goldenset.yaml` (pytest) | `modules/cuo/audit-profile.yaml` | - |

## When this loop is done

Once AUTH, MEMORY, SKILL, PROJ, CHAT, and CUO each pass GREEN + CLEAN locally, the platform is verified end to end against both gates, and you can move to implementing the remaining modules (obs is the unblocked next one). Each new module then ships through the same `ship-tasks` chain, so the gates apply to it automatically.

## Notes and the small gaps to confirm

- The caf gate currently runs target-health only (no sealed `.caf/` audit per module yet); that already catches build/lint/test breaks. Add the LLM audit half later via `tools/caf/core/evals/run-audit.sh`.
- PROJ is a library, so "run PROJ" means running the binary that embeds `cyberos_proj` - confirm which.
- CUO's console-script name is in `modules/cuo/pyproject.toml` `[project.scripts]`.
- ai is not in the core six but its suite needs the same Redis; it runs serial (`--test-threads=1`).
