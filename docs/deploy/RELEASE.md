# CyberOS release process

The consolidated guideline. CyberOS carries ONE platform version (the root `VERSION` file, stamped into the init payload and every `.cyberos/VERSION`); module versions are internal. Two pipelines ship it:

- Continuous (`deploy.yml`): every push to `main` runs the gate, builds + pushes the service images to GHCR (auth, chat, ai-gateway, embed-sidecar; eval + memory best-effort), and rolls the VPS.
- Versioned (`release.yml`): pushing a `v*` tag builds the desktop (Tauri) and mobile (Capacitor) artifacts and creates the GitHub Release.

## Cutting a release, step by step

1. Land everything on `main` through PRs - the gates (services, awh-gate, docs-prerender-gate) must be green.
2. Bump the platform version:

   ```
   echo 0.2.0 > VERSION
   ```

3. Commit. The pre-commit hooks do the regeneration for you:
   - `cyberos-payload-build` rebuilds `dist/cyberos` whenever a vendored source (or `VERSION`) changes, so the init payload always matches the release.
   - `docs-site-build` verifies the site still builds whenever a documentation source changes.

4. Record the release in `CHANGELOG.md` (repo level; per-module history lives in each module's `CHANGELOG.md`, rendered to the site's changelog pages).
5. Push `main`; wait for `deploy.yml` to go green (images pushed, VPS rolled).
6. Tag and push the tag:

   ```
   git tag v0.2.0 && git push origin v0.2.0
   ```

   `release.yml` builds the desktop/mobile artifacts and publishes the GitHub Release.

7. Distribute the payload: projects update with `init.sh --check` (notify) and re-running `init.sh` (apply) - or from the desktop app's CyberOS Ops tab. Fleet-wide: `tools/cyberos-init/rollout.sh`.

## Docs are part of every release

The website is generated from the markdown single source of truth (FR-DOCS-002): module docs at `modules/<m>/docs/` or `services/<s>/docs/`, global docs under `docs/`. Three mechanisms keep it fresh, in order of defense:

1. Pre-commit `docs-site-build` (local, automatic).
2. `docs-prerender-gate` (CI, every PR touching doc sources): rebuilds the whole site and fails if it does not build clean.
3. Manual: `bash tools/docs-site/build.sh` (or `--docs` for the doctrine pages only).

Nothing generated is committed: the site renders into gitignored `dist/website`, so there is no generated HTML to edit by hand.

## GHCR troubleshooting

A `403 Forbidden` pushing an image means that GHCR package exists without this repo granted write access (packages created by this workflow auto-link via the `org.opencontainers.image.source` label; older ones may not). Fix in GitHub: org → Packages → the failing package → Package settings → Manage Actions access → add this repo with the Write role. Alternatively delete the stale package and let the workflow recreate it linked.

## Related runbooks

`go-live-guide.md` (first production bring-up), `cyberos-core-deploy.md` (VPS topology), `ci-and-local-checks.md` (what each gate runs), `local-dev-and-testing.md` (dev stack), and `tools/cyberos-init/GUIDE.md` (running CyberOS in other repos).
