---
id: NFR-GAM-003
title: "Green-gate quality bar"
module: GAM
status: done
fidelity: as-built
shipped: 2026-06-23
owner: Stephen Cheng
source_repo: zintaen/gam @ f55d97c
related: [NFR-GAM-001, NFR-GAM-002]
---

## §1 — Statement (BCP-14 normative)

Every change under `apps/gam` MUST pass the gam gate before merge.

1. The gate MUST run, on a clean checkout: frontend lint, type-check, unit tests, a frontend build check, Rust clippy with warnings denied, Rust tests, and cargo-deny.
2. The gate MUST be self-contained: it MUST NOT depend on any external shared-action repository, so it runs from a clean checkout of this repo alone.
3. The gate MUST install the Linux Tauri system libraries before the cargo steps, because the crate compiles the Tauri stack.
4. The gate MUST be scoped to changes under `apps/gam`.

## §2 — Why this matters

A quality bar only protects a branch if it actually runs everywhere from a clean checkout. The upstream gam repo's first CI run failed precisely because the workflow referenced a shared-actions repository that did not exist; the fix was to make the gate self-contained. The Linux system-library step matters because the gam crate pulls in webkit2gtk/glib, which are not present on a bare runner, so clippy and tests cannot even compile without them.

## §3 — Implementation

- `.github/workflows/gam-gate.yml` — triggers on changes under `apps/gam`; runs lint, type-check, frontend tests, build check, `cargo clippy --locked -- -D warnings`, `cargo test --lib --locked`, and `cargo deny check`.
- Uses only standard public actions (checkout, pnpm/action-setup, setup-node, dtolnay/rust-toolchain, swatinem/rust-cache); no external org action.
- Installs `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev`, `patchelf` on Linux before the cargo steps.

## §4 — Verification

- `.github/workflows/gam-gate.yml` passes `actionlint`.
- The workflow contains no `cyberskill-world/.github` (or any external org action) reference.
- The same checks ran green in the upstream repo's CI on ubuntu, macOS, and windows after the self-contained and Linux-deps fixes.

## §5 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| Missing external action | resolves to nothing | avoided: gate is self-contained |
| Linux Tauri libs absent | `glib-sys` build fails | avoided: apt step installs them first |
| clippy regression on newer toolchain | `-D warnings` | gate fails (caught the `unnecessary_sort_by` lint) |
| Lockfile drift | `--locked` | gate fails |

## §6 — Notes

This NFR is the CyberOS-side restatement of the upstream gam CI, which now passes green on all three OSes. Mark `gam-gate.yml` required in branch protection once the absorption lands.

*End of NFR-GAM-003. Fidelity: as-built (10/10 target).*
