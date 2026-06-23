# gam (Git Alias Manager)

A desktop GUI for managing Git aliases, built with Tauri v2 (Rust) and React 19 with TypeScript. Absorbed into CyberOS from the standalone repository github.com/zintaen/gam.

## Status

Absorbed 2026-06-23 from zintaen/gam at commit f55d97c (branch auto/gam-absorb upstream). The app ships to real users; this copy is its monorepo home, kept in sync with the upstream repo until that repo is retired.

## How it fits into CyberOS

gam is a standalone crate. It keeps its own Cargo.lock and pnpm-lock.yaml and is not a member of services/Cargo.toml. Do not add it to the services workspace; it builds and tests on its own.

CyberOS has no root pnpm workspace, so gam's Node toolchain stays independent automatically. Run every gam command from inside apps/gam.

The CI gate is .github/workflows/gam-gate.yml. It runs on any change under apps/gam and covers lint, types, frontend tests, a build check, Rust clippy, Rust tests, and cargo-deny. Mark it required in branch protection once this lands.

## Develop

From apps/gam:

- `pnpm install`
- `pnpm dev` to run the app
- `pnpm lint && pnpm exec tsc --noEmit && pnpm test` for the frontend gate
- `cd src-tauri && cargo test --locked && cargo clippy --locked -- -D warnings && cargo deny check` for the Rust gate

Linux builds need the Tauri system libraries (libwebkit2gtk-4.1-dev, libgtk-3-dev, librsvg2-dev, libayatana-appindicator3-dev, patchelf). The gate installs them.

## Open decisions for the merge

Version reconciliation. gam is at 1.0.11 because real installs depend on the updater seeing a monotonically increasing version. The CyberOS root VERSION is 0.1.0. Resetting gam to 0.1.0 would break auto-update for existing users, so gam keeps its own semver for now. Decide at merge time whether to unify; if you do, gam's number must keep climbing from 1.0.11, not reset.

Updater signing. The updater public key in src-tauri/tauri.conf.json was rotated on 2026-06-23. The matching private key lives in the upstream repo's GitHub secrets. Releasing gam from CyberOS needs those signing secrets configured here too.

Requirements. The gam catalog lives at docs/feature-requests/gam/ (FR-GAM-001..008) and docs/non-functional-requirements/gam/ (NFR-GAM-001..003). Each entry has a per-file spec and a paired .audit.md, authored at as-built fidelity 2026-06-24.
