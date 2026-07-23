# CI gates and their local mirrors

What runs where, and how to run every gate on your own machine before pushing. The design rule: CI's heavyweight job executes the SAME script you run locally (`scripts/local_verify.sh`), so local green and CI green are the same claim - there is no emulation layer to drift.

## The gates

On every pull request:

- `services.yml` - two jobs. `lint + test (pure-Rust)`: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace` with NO database (hermetic). `integration (Postgres + Redis)`: boots pgvector Postgres + Redis and runs `bash scripts/local_verify.sh` - the full migration chain and every module suite including the DB-backed tests.
- `awh-gate.yml` - for each module changed by the PR that has a golden set (`modules/<m>/.awh/goldenset.yaml`), reruns the sealed tasks against the committed baseline; any regression blocks the merge.
- `docs-prerender-gate.yml` - rebuilds the whole docs site from the markdown sources; a broken source or missing asset fails the PR.
- Focused property/consistency gates, each owning one invariant: `rls-property-gate`, `cache-isolation-gate`, `obs-correlation-gate`, `rew-memory-exclusion`, `vn-pii-recall`, `mcp-sep986-check`, `voice-and-consistency`, `zdr-staleness-check`, `proj-a11y-gate`.

On push to `main`: `deploy.yml` (gate -> build + push images to GHCR -> roll the VPS). On a `v*` tag: `release.yml` (desktop/mobile installers + GitHub Release). Scheduled: `vn-pii-quarterly-refresh`, `memory-rebuild`.

## About all those "ignored" tests

Every test that needs Postgres is `#[ignore]`-by-default. That is a deliberate split, not missing coverage:

- In DB-less contexts - the `lint + test` job, an awh golden task's plain `cargo test -p <crate>`, or a bare `cargo test` on your machine - they print `ignored`, because no database exists there and a silent pass would be a lie.
- In the integration job (and in `scripts/local_verify.sh` locally) they ALL execute: the suites run with `--include-ignored --test-threads=1` after the full migration chain (auth -> mcp-gateway -> memory -> eval -> chat -> ai-gateway -> email -> proj).

So an `ignored` line answers "does this context have a database", never "is this test skipped by CI". Every crate that declares `#[ignore]` tests (auth, memory, eval, ai-gateway, mcp-gateway) is in `local_verify.sh`'s suite list; if you add a new crate with DB tests, add it to that list - that is the one place coverage could silently leak.

## Run the gates locally

One-time: `git config core.hooksPath .githooks` (the hooks are repo-tracked; no framework install needed).

| Gate | Local command |
|---|---|
| services: integration | `bash scripts/local_verify.sh` (Docker up; wipe first for a CI-clean run: `docker compose -f services/dev/docker-compose.yml down -v`) |
| services: one crate / one test | `bash services/dev/test-db.sh -p <crate> [--test <file>]` |
| services: pure-Rust job | `cd services && cargo fmt --all --check && cargo clippy --workspace --all-targets --no-deps -- -D warnings && cargo test --workspace` |
| awh-gate for a module | `pip install -e tools/awh && awh eval modules/<m>/.awh/goldenset.yaml --base-dir . --seeds 1 --baseline modules/<m>/.awh/eval-baseline.json --max-regression 0.0` |
| docs-prerender-gate | `bash tools/docs-site/build.sh` |
| deploy.yml's gate step | same as the pure-Rust job above |

The hooks run the important ones automatically: pre-push runs `local_verify.sh` when the push touches `services/` and Docker is up (`NO_DB_VERIFY=1` or `SKIP_LOCAL_CHECKS=1` to bypass; it falls back to the pure-Rust checks when Docker is down), and pre-commit rebuilds the install payload and verifies the docs site build when their sources are staged.

## Why not act?

[act](https://github.com/nektos/act) replays GitHub workflow YAML in local Docker containers. We evaluated it and inverted the problem instead: the expensive job (integration) IS `scripts/local_verify.sh`, so running that script locally reproduces CI by construction - no runner emulation, no image drift, and it works with the native toolchain (act cannot exercise the real macOS paths at all). What act would add on top is only the YAML glue (checkout, caches, service-container wiring), which the focused gates above cover directly and cheaply.

If you want to sanity-check workflow YAML itself, act still works as an unsupported extra: `brew install act`, then e.g. `act pull_request -W .github/workflows/docs-prerender-gate.yml`. Expect differences on jobs that need service containers or large toolchains; nothing in our process depends on it.

## A normal change, end to end

Edit, commit (pre-commit regenerates payload/docs if touched), push (pre-push runs the CI-equivalent verify when Docker is up), open the PR (the gates above run), merge on green - `deploy.yml` builds the images in Actions and rolls the VPS. Web-only changes skip the Rust gate and image build.
