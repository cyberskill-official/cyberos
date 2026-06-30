# Local-first build, auto-deploy on push

How CyberOS P0 ships. All the heavy work runs on your Mac, in the pre-push hook, BEFORE the push. GitHub does
one thing: when a push lands on main, it deploys. It never builds or tests.

## The pre-push hook does the heavy work

One-time:

    git config core.hooksPath .githooks
    chmod +x .githooks/pre-push
    docker login ghcr.io -u <github-username>    # PAT with write:packages, for the image push

When a push touches `services/`, `.githooks/pre-push` runs, in order, and aborts the push on any failure:

    cargo fmt --all --check
    cargo clippy --workspace -- -D warnings
    cargo test --workspace
    deploy/vps/build-push-images.sh              # build linux/amd64 images, push to GHCR

So nothing half-built ever reaches GitHub, and the fresh images are in GHCR before the push triggers the
deploy. A push that touches only the console or `apps/web` skips all of this - those deploy via the VPS
git-pull and need no image rebuild, so they stay instant.

Bypass: `SKIP_LOCAL_CHECKS=1 git push` (skip everything) or `SKIP_IMAGE_BUILD=1 git push` (keep the checks,
skip the image build). The VPS is x86_64, so images build `linux/amd64`; on an Apple Silicon Mac that runs
emulated and is slower than native - the trade for keeping the build off GitHub.

## GitHub deploys on push

`deploy.yml` runs on every push to main that touches `services/`, `apps/console/`, `apps/web/`, or
`deploy/vps/`. It SSHes to the VPS and runs `deploy.sh`, which git-pulls (console, Caddyfile, compose,
apps/web/dist) and docker-pulls the images the hook just pushed, then restarts. You can also trigger it by
hand (Actions -> deploy -> Run workflow).

The other workflows (`services`, `mcp-sep986-check`, `rls-property-gate`, `voice-and-consistency`) run only on
pull requests and on demand - never on push.

## A normal change, end to end

Edit, commit, push. For a service-code change the hook formats, lints, tests, and builds + pushes images
locally, then the push triggers the deploy. For a web-only change, rebuild the SPA first
(`cd apps/web && npm run build`), commit `dist`, then push - the deploy git-pulls the new build, no image
needed.
