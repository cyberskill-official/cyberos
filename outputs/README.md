# `outputs/` — generated artefacts + reference implementations

This folder holds files written by `cyberos` runtime tools. Mix of:

1. **Reference implementations** (committed to git — read these to understand the protocol):
   - [`brain_writer.py`](brain_writer.py) — canonical BRAIN-mutation API; the `cyberos add` family delegates to it.
   - [`apply-bundle-Q.sh`](apply-bundle-Q.sh) — atomic-write rollout helper invoked by `cyberos rollout apply`.
   - [`cleanup-host.sh`](cleanup-host.sh) — sandbox-cannot-unlink workaround; remove stale files on host.

2. **Generated dashboards** (committed; regenerable on demand):
   - [`_audit-site/`](_audit-site/) — static HTML audit dashboard. Regenerate with `cyberos audit publish`. Open `_audit-site/index.html` in a browser.

3. **Per-tool scratch directories** (mostly committed; some gitignored — see `.gitignore`):
   - [`council/`](council/) — outputs of `cyberos council run` (Tier A council voting).
   - [`doctor/`](doctor/) — `*.log` files from `cyberos doctor run` (logs gitignored).
   - [`refinements/`](refinements/) — staged refinement candidates from §11.4 dashboard (`draft-*.md` gitignored).
   - [`replan/`](replan/) — `cyberos replan` outputs.
   - [`runtime-specs/`](runtime-specs/) — generated tool specs for `cyberos doctor`.
   - [`staged-memories/`](staged-memories/) — memories awaiting `cyberos commit-stage` (`*.md` gitignored).
   - [`templates/`](templates/) — pre-baked Layer-1 starter templates (loaded by `cyberos init`).
   - [`cyberos-starter/`](cyberos-starter/) — boot-strap scaffold for new projects.

4. **Smoke-test traces** (mostly transient):
   - `_chain-smoke/`, `_chain-wire-test/`, `_runner-smoke/` — fixtures created by the `runtime/tests/` integration tests. Safe to delete; tests recreate.
   - `sync/` — placeholder for `cyberos sync` operations.

## Why is this under `outputs/` and not `runtime/`?

`runtime/` holds **source code** (Python modules, completion scripts, test fixtures). `outputs/` holds **the things runtime code writes** — generated reports, scratch state, atomic-write stages. Splitting them keeps the source tree free of generated artefacts and makes the gitignore patterns clearer.

## Quick reference

| Want to … | Run … |
| --- | --- |
| Regenerate the audit dashboard | `cyberos audit publish` |
| See current refinement candidates | `cyberos refinement dashboard` |
| List staged memories awaiting commit | `cyberos status --staged` |
| Clean transient outputs | `cyberos cleanup` (Batch 15) |
| Apply a pending bundle atomically | `bash outputs/apply-bundle-Q.sh` |
