---
fr_id: FR-GAM-008
audited: 2026-06-24
verdict: PASS (as-built, process-verified)
score: 9.5/10
fidelity: as-built
template: as-built-verification@1
---

## §1 — Verdict summary

FR-GAM-008 is shipped and the key rotation is complete and verified. The half-point reflects that signature acceptance/rejection is enforced by the updater at runtime against a live manifest rather than by an in-repo automated test; verification here is configuration- and process-level, which is appropriate for an updater but is not a unit test.

## §2 — Clause to artefact traceability

| §1 Clause | Artefact | Verification | Status |
|---|---|---|---|
| #1/#2 verify against pinned key, reject mismatch | `tauri-plugin-updater` + `tauri.conf.json` pubkey | runtime minisign verification (plugin behavior) | OK |
| #3 key in `tauri.conf.json` | `plugins.updater.pubkey` | present; valid JSON | OK |
| #4 rotatable + re-pinned | rotated 2026-06-23 to `A55DB9ED5AE4C0D1` | `gh secret list` fresh timestamps; new key in CI build | OK |
| #5 reinstall note on rotation | release process | documented in apps/gam/README open decisions | OK (process) |

## §3 — Verification record

- `src-tauri/tauri.conf.json` contains the pinned `pubkey` and `endpoints` (valid JSON).
- New public key `A55DB9ED5AE4C0D1` in place; old `B128E25D1D5AF1C3` retired; old weak passphrase scrubbed from docs.
- GitHub secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` reset 2026-06-23; the upstream CI run built the new key into the app on ubuntu/macOS/windows.

## §4 — Open decision (carried, not a defect)

Releasing gam from CyberOS requires the signing secrets to be configured in this repo's CI as well. Recorded in apps/gam/README and NFR-GAM-002.

## §5 — Status

`accepted → shipped`. Security-critical; rotation closed the one real gap.

*End of FR-GAM-008 audit.*
