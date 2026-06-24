---
id: FR-GAM-008
title: "Signed auto-update"
module: GAM
priority: MUST
status: done
fidelity: as-built
shipped: 2026-06-23
owner: Stephen Cheng
source_repo: zintaen/gam @ f55d97c
related_frs: [NFR-GAM-002]
---

## §1 — Description (BCP-14 normative)

gam MUST only apply updates that are cryptographically signed by the project.

1. The app MUST verify each update against a pinned minisign public key before applying it.
2. An update whose signature does not verify against the pinned key MUST be rejected.
3. The pinned public key MUST live in `src-tauri/tauri.conf.json`.
4. The signing key MUST be rotatable, and a rotation MUST be shipped in a release that re-pins the new public key (see NFR-GAM-002).
5. Because the embedded key changes on rotation, a rotation release MUST tell existing users they need one manual reinstall before auto-update resumes.

## §2 — Why this design

Auto-update is a remote-code-execution channel: whatever the updater accepts, it runs. Signature verification against a pinned key is what makes that channel safe, because only artifacts signed with the matching private key are accepted. Pinning the public key in the app binary (via `tauri.conf.json`) means a compromised update server cannot substitute its own key. Rotation has to re-pin and force a reinstall because the old installed binaries trust only the old key.

## §3 — Implementation

- `tauri-plugin-updater` configured in `src-tauri/tauri.conf.json`: `plugins.updater.pubkey` (the pinned minisign public key) and `endpoints` (the update manifest URLs), with `createUpdaterArtifacts` on.
- The public key was rotated on 2026-06-23 (new key `A55DB9ED5AE4C0D1`, old `B128E25D1D5AF1C3` retired) after the prior key's passphrase was found weak.
- Release signing is wired in the upstream release workflow, which reads the private key and passphrase from CI secrets (see NFR-GAM-002 and apps/gam/README "Open decisions").

## §4 — Acceptance criteria

1. The updater is configured with a pinned `pubkey` and `endpoints` in `tauri.conf.json`.
2. An update signed by the matching private key verifies and can apply.
3. An update not signed by the matching key is rejected by the updater.
4. The currently pinned key is the rotated 2026-06-23 key.

## §5 — Verification

Verification is configuration- and process-level, not unit-test-level (the updater verifies at runtime against the live manifest):

- `src-tauri/tauri.conf.json` carries the pinned `pubkey` and `endpoints`.
- The pinned key matches the private key now held in the upstream repo's GitHub secrets (reset 2026-06-23); confirmed by `gh secret list` showing both `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` with fresh timestamps.
- The new public key compiles into the app on all three OSes in the upstream green CI run.

## §6 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| Update signed by wrong/old key | minisign verify in updater | update rejected, not applied |
| Pinned key does not match signer | verify failure | no update applies (releases must use the matching key) |
| Rotation without reinstall note | operator process | existing installs stop auto-updating until reinstalled once |

## §7 — Notes

This FR is the app-facing half of the security posture; NFR-GAM-002 covers the supply-chain and signing-integrity half. The open decision "configure signing secrets in CyberOS before releasing gam from here" is recorded in apps/gam/README.

*End of FR-GAM-008. Fidelity: as-built (10/10 target).*
