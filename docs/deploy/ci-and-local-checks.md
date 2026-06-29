# CI is deploy-only; heavy checks run locally

The build model, after the 2026-06-30 simplification.

GitHub does one thing on a push to main: it runs `deploy` (build the images, push to GHCR, roll the VPS). The heavy checks - format, clippy, and tests - run on your Mac at push time, where the toolchain already is. This keeps GitHub fast and stops a failing test gate from sitting red next to a green deploy.

## One-time setup (on your Mac)

    git config core.hooksPath .githooks
    chmod +x .githooks/pre-push

After that, `git push` runs `.githooks/pre-push` first: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`. If anything fails, the push is aborted and nothing deploys. It only runs when the commits you are pushing touch `services/`, so doc- or console-only pushes stay instant.

Bypass options when you need them:

    SKIP_LOCAL_CHECKS=1 git push      # skip the hook for one push
    git push --no-verify              # same effect

The Postgres-gated `#[ignore]` integration tests are not in the hook (they need a database). Run them by hand when you touch that code:

    cd services && DATABASE_URL=postgres://... cargo test --workspace -- --ignored

## What changed in GitHub

`deploy.yml` still runs on push. The other workflows that used to run on push - `services` (lint+test), `mcp-sep986-check`, `rls-property-gate`, and `voice-and-consistency` - now run only on pull requests and on demand (`workflow_dispatch`), so they no longer fire on your direct pushes to main. The many pull-request-only gates are unchanged; they never ran on push anyway.
