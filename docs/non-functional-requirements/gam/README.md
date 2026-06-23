# GAM module — non-functional requirements

As-built NFRs for the absorbed gam desktop app. Normative keywords follow BCP-14.

| NFR | Title |
|---|---|
| NFR-GAM-001 | Least-privilege desktop surface |
| NFR-GAM-002 | Supply-chain and update-signing integrity |
| NFR-GAM-003 | Green-gate quality bar |

## NFR-GAM-001 — Least-privilege desktop surface

gam MUST request only the Tauri capabilities it actually uses. The shell and opener plugins were removed because no code path needed them; opening a browser or folder goes through the `open` crate instead. New capabilities MUST be justified by a real call site before they are added back.

Verification: src-tauri/capabilities/default.json (trimmed grants); src-tauri/Cargo.toml (no shell/opener plugins); cargo clippy/test green after removal.

## NFR-GAM-002 — Supply-chain and update-signing integrity

gam MUST gate its dependencies: cargo-deny checks advisories, licenses, bans, and sources, and the build resolves against committed lockfiles (`--locked`). Updates MUST be cryptographically signed (minisign) and verified against a pinned public key; the signing key MUST be rotatable, and a rotation MUST be accompanied by a release that re-pins the new public key.

Verification: src-tauri/deny.toml; cargo-deny step in .github/workflows/gam-gate.yml; updater pubkey in src-tauri/tauri.conf.json (rotated 2026-06-23, old key retired).

## NFR-GAM-003 — Green-gate quality bar

Every change under apps/gam MUST pass the gam gate before merge: frontend lint, type-check, unit tests, a build check, Rust clippy with warnings denied, Rust tests, and cargo-deny. The gate MUST be self-contained (no dependency on external shared-action repositories) so it runs from a clean checkout.

Verification: .github/workflows/gam-gate.yml; the upstream repo's green CI run on the same checks.
