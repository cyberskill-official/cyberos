# TASK-APP-003 — ship-workflow phase bundle (steps 1–12)

Run: 2026-07-13, ship-tasks v2.4.0. Queue echo: `queue: picked TASK-APP-003 (priority=SHOULD, created=2026-07-12) — 2nd of 5 in batch; TASK-IMP-073 parked at HITL gate 1`.

## Steps 1–2 — repo context map

- **Patterns:** config-overlay convention already exists (`tauri.updater.conf.json` + `--config` flag in the desktop job) — `tauri.mas.conf.json` follows it exactly. Gated-job convention: `vars.<GATE> == 'true'`. macOS runner quirks documented in release.yml (keychain ACL flake → image pinning; notarization poll → not applicable to MAS, ASC processing replaces it).
- **Backend audit basis (AC #1):** `apps/desktop/src-tauri/src/` = 310 lines total across 5 files; 8 `#[tauri::command]` handlers; deps: tauri 2, reqwest(rustls), keyring 3, tauri-plugin-updater (desktop-only). No fs/dialog/shell plugins, no `std::fs`, no `Command::new`, no sidecars (`externalBin` absent from tauri.conf.json).
- **Blast radius:** 5 new files (2 plists, 1 overlay, 1 workflow, 1 doc) + 1 lint script. Spec's `modified_files` (tauri.conf.json, release.yml) ended up **untouched** — AC #3 requires tauri.conf.json unchanged (verified: 0-line diff), and the MAS job lives in its own `release-mas.yml` (workflow_dispatch), so release.yml needs no edit. Declared-but-untouched recorded honestly.
- **Outside-domain files: 2** (tools/ lint script, docs/deploy sheet) ≤ 3 → **ADR not triggered** (steps 3–4 skipped).
- **Mocks (steps 7–8): skipped** — external dependencies (Apple enrollment, certs, ASC record) are *human-prerequisite gates*, not services to mock; the workflow is inert until they exist (spec §1 #5/#7).

## Steps 5–6 — edge-case matrix

| # | Category | Case | Covered by |
|---|---|---|---|
| 1 | null/empty | `MAS_RELEASE` unset/false | job-level `if:` — skipped, not failed (AC #6); `assert-mas-gate-inert` job gives every dispatch a completed run to assert against |
| 2 | null/empty | `MAS_RELEASE=true` before certs exist | `security import`/`codesign` fail loudly, no partial artifact (spec §10 row 6 — operator error, correct behavior) |
| 3 | malformed | entitlement added without audit-table row | `tools/mas-entitlement-lint.sh` exit 1 — proven both directions this session (unjustified key → ERROR; full set → OK). Lint runs in the unconditional job on every dispatch |
| 4 | malformed | plist with zero entitlement keys / missing audit section | lint exit 2 explicit branches (not silent pass) |
| 5 | security | self-updater inside MAS bundle (sandbox + policy violation) | audit finding recorded; **hard blocker #1** in the answer sheet — MAS_RELEASE must stay off until the `mas` cargo-feature follow-up task lands. Not rationalized away via unverified config-merge tricks |
| 6 | security | credential material outside secret manager | workflow uses `${{ secrets.* }}` exclusively; API key materialized only to `./private_keys/` and deleted in `if: always()` cleanup along with the ephemeral keychain |
| 7 | bounds/platform | wrong signing-order: inherit plist applied to MAIN binary (spec skeleton's `MacOS/*` glob bug) | helper-signing loop skips `CyberOS`, re-seals outer bundle only if a helper was signed (inside-out order) — divergence documented for review |
| 8 | bounds/platform | arm64-only artifact excludes Intel Macs | `--target universal-apple-darwin` + both rustc targets, mirroring the desktop job |
| 9 | regression | Developer ID channel behavior drift | AC #3: tauri.conf.json 0-line diff; overlay declares no `version`/`productName` so the stamper stays authoritative (spec §10 drift row) |
| 10 | concurrency/env | keychain ACL flake (`set-key-partition-list`) on image rotation | `macos-14` pinned (spec §10 mitigation); ephemeral keychain deleted `if: always()` |

## Steps 9–10 — implementation plan (executed)

1. `Entitlements.mas.plist` — app-sandbox + network.client only (audit-minimal; §3 sketch's files.user-selected dropped with in-file rationale). ✅
2. `Entitlements.mas.inherit.plist` — defensive-only, resolves spec §9 Q2 (no child processes today) with evidence. ✅
3. `tauri.mas.conf.json` — §3 contract verbatim. ✅
4. `.github/workflows/release-mas.yml` — full workflow per §3 skeleton + 4 documented corrections. ✅
5. `tools/mas-entitlement-lint.sh` — AC #2 lint (dependency-free, section-scoped). ✅
6. `docs/deploy/mac-app-store-submission.md` — sandbox surface audit (12 rows, source-grounded), updater finding, 5 hard blockers, 10-field ASC answer sheet. ✅

## Steps 11–12 — observability injection

CI-surface observability: `::error::` on the sandbox-verification step (bundle-not-sandboxed check); loud native failures from security/codesign/productbuild/altool; lint prints per-key ERROR lines + OK summary; `pkgutil --check-signature` output in the job log satisfies AC #4's evidence requirement. Error-branch coverage: every failure path in the new shell code prints a diagnostic before exiting non-zero (lint: 3 explicit exit-2 branches + per-key errors; workflow: verify step's grep guard).

## Machine verification (this session)

- `tools/mas-entitlement-lint.sh` positive: **OK (2/2 keys justified)**; negative: **exit 1 with correct ERROR** (both run for real).
- `release-mas.yml` YAML parse: **PASS**. Both plists XML parse: **PASS**. Overlay JSON parse: **PASS**.
- AC #3: `git diff HEAD -- tauri.conf.json` = **empty**.
- AC #5 (`codesign --verify`, `spctl`): **expected-pending** — requires macOS + real MAS certs (spec §5 allows "documented as expected-fail with reason otherwise"); first real run after blockers clear.
- AC #4/#6 live CI evidence: **deferred by design** until a dispatch happens (AC #6's assert command is embedded in the workflow contract).
- AC #8: no secret-scan gate exists yet in the repo (TASK-IMP-003 still draft) — asserted manually instead: every credential reference in added files is `${{ secrets.* }}`; recorded honestly rather than citing a nonexistent gate.

*End phase bundle.*
