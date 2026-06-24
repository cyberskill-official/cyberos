---
id: NFR-GAM-002
title: "Supply-chain and update-signing integrity"
module: GAM
status: done
fidelity: as-built
shipped: 2026-06-23
owner: Stephen Cheng
source_repo: zintaen/gam @ f55d97c
related: [FR-GAM-008, NFR-GAM-003]
---

## §1 — Statement (BCP-14 normative)

gam MUST gate its dependencies and MUST sign its updates.

1. The build MUST run `cargo-deny` over advisories, licenses, bans, and sources.
2. The build MUST resolve against committed lockfiles (`cargo ... --locked`, frozen pnpm lockfile), so CI builds the same graph that was reviewed.
3. Updates MUST be cryptographically signed with minisign and verified against a pinned public key (see FR-GAM-008).
4. The signing key MUST be rotatable, and any rotation MUST be accompanied by a release that re-pins the new public key.
5. No private signing material may be committed to the repository.

## §2 — Why this matters

Two distinct supply-chain risks meet here: pulling in a vulnerable or wrongly-licensed dependency, and shipping an unsigned or wrongly-signed update. cargo-deny plus locked resolution addresses the first; minisign signing with a pinned key addresses the second. Treating the signing key as rotatable (and never committing it) means a key compromise is recoverable without abandoning the update channel.

## §3 — Implementation

- `src-tauri/deny.toml` — cargo-deny policy: advisories, licenses (CDLA-Permissive-2.0 allowed), bans, sources; `unmaintained = "workspace"` scopes out transitive gtk-rs/unic crates.
- `.github/workflows/gam-gate.yml` — installs and runs `cargo deny check`; all cargo steps use `--locked`.
- `src-tauri/tauri.conf.json` — pinned updater public key (rotated 2026-06-23) + endpoints.
- Signing private key + passphrase live only in CI secrets (upstream repo); the working tree carries `.env.example` only, never a real `.env` or key.

## §4 — Verification

- `src-tauri/deny.toml` present; `cargo deny check` is a gate step.
- A broad `cargo update` during hardening cleared six transitive advisories (webpki TLS, tar) via patched semver; the gate is green.
- Updater pubkey rotated 2026-06-23; old key retired; old weak passphrase scrubbed from docs.
- Repository scan confirms no `.env`, `*.key`, `*.pem`, or minisign secret material under `apps/gam`.

## §5 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| New advisory in a dependency | `cargo deny check` | gate fails until updated/patched |
| Disallowed license pulled in | cargo-deny licenses | gate fails |
| Lockfile drift | `--locked` | build fails rather than silently re-resolving |
| Signing key compromise | operator | rotate + re-pin + reinstall release (FR-GAM-008) |

## §6 — Notes

The 2026-06-23 rotation is the concrete exercise of clause #4: the old key (weak passphrase) was retired, the new public key re-pinned, and the matching private key reset in CI secrets.

*End of NFR-GAM-002. Fidelity: as-built (10/10 target).*
