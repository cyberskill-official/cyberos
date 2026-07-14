# TASK-APP-004 — ship-workflow phase bundle (steps 1–12)

Run: 2026-07-13, ship-tasks v2.4.0. `queue: picked TASK-APP-004 (priority=SHOULD, slice 2) — 3rd of 5; TASK-IMP-073 + TASK-APP-003 parked at HITL gate 1`.

## Steps 1–2 — repo context map

- **Patterns:** gated standalone workflow_dispatch channel (release-mas.yml precedent from TASK-APP-003, same batch); lint-as-second-gate; `${{ secrets.* }}`-only credentials; toolchain trio (node20 + stamper + dtolnay rust) from the desktop job.
- **Grounding checks run:** all 4 manifest-referenced tile PNGs exist in `apps/desktop/src-tauri/icons/` (plus 5 more square sizes); `Wide310x150Logo.png` confirmed ABSENT (spec §11 omission correct); Cargo package name = `cyberos-desktop` while the manifest declares `CyberOS.exe` → staging resolves either raw-binary name and stages AS `CyberOS.exe`.
- **Blast radius:** 4 new files + 1 lint script. Spec's `modified_files` listed release.yml — **untouched** (standalone workflow, same reasoning as TASK-APP-003; declared honestly).
- **Outside-domain: 2** (tools/, docs/deploy/) ≤ 3 → **ADR skipped**. **Mocks skipped** — Partner Center/Azure AD are human-prerequisite gates; the submission flow is deliberately implemented against Microsoft's live versioned docs at first real run (spec §6/§11 anti-drift decision).

## Steps 5–6 — edge-case matrix

| # | Category | Case | Covered by |
|---|---|---|---|
| 1 | null/empty | `MSSTORE_RELEASE` unset/false | job `if:` skip + unconditional anchor job (AC #4); lint inert mode proven green |
| 2 | null/empty | gate on, identity still placeholder | lint enforced mode exit 1 — proven live; runs FIRST in the gated job (fails in seconds, not after a Rust build) |
| 3 | null/empty | `MSSTORE_SIGNING_MODE` unset | signing steps skipped → Store-managed default, zero extra secrets (AC #5) |
| 4 | malformed | manifest schema-invalid | `makeappx pack` rejects at pack time + explicit `$LASTEXITCODE` throw (AC #1) |
| 5 | malformed | referenced asset missing from staging | pre-pack existence loop throws the missing filename (AC #2) |
| 6 | bounds/platform | Windows SDK path drift across runner images | dynamic SDK discovery + `Test-Path` throw (spec §10 row 1 resolution) |
| 7 | bounds/platform | Tauri version changes raw binary name | dual-name resolution (`CyberOS.exe` | `cyberos-desktop.exe`) + loud failure listing `*.exe` found |
| 8 | security | EV cert/thumbprint without import | explicit `Import-PfxCertificate` step precedes signing; PFX temp file removed |
| 9 | security | credential material outside secret manager | `${{ secrets.* }}` exclusively; token variable never echoed |
| 10 | concurrency | interrupted multi-step submission → stale PendingCommit | documented operational recovery (Partner Center UI discard) — resume-idempotency out of scope per spec §10 |
| 11 | regression | tile assets accidentally regenerated | AC #8 diff check — 0 lines (proven); FR only references assets |
| 12 | regression | release gates coupled in future refactor | independent `MSSTORE_RELEASE` + standing AC #4 assertion |

## Steps 9–10 — implementation plan (executed)

1. `AppxManifest.xml` — §3 contract with unambiguous CHANGEME identity, real icon refs, no wide tile, runFullTrust. ✅
2. `tauri.msix.conf.json` — targets narrowed to nsis. ✅
3. `tools/msix-identity-lint.sh` — §6 skeleton + default-path arg + explicit missing-manifest exit 2. ✅
4. `release-msstore.yml` — §3 skeleton + corrections (paths, anchor job, early lint, binary resolution, toolchain steps, artifact upload, `$LASTEXITCODE` guards). ✅
5. `microsoft-store-submission.md` — 4 hard blockers, 8-field answer sheet (IARC distinct from Apple/Google flagged), operational notes + manual tile-alpha QA. ✅

## Steps 11–12 — observability injection

Every failure branch throws with a named diagnostic (missing SDK, missing binary w/ directory listing, missing staged asset by filename, pack/sign exit codes, token-null throw); lint prints per-state messages (proven in 3 states). Success paths log one summary line each. Coverage of error branches in new code: 100 % of enumerated branches print before failing.

## Machine verification (this session)

- Identity lint: inert **OK** / enforced+placeholder **exit 1** / enforced+real **OK** — all really run.
- YAML/XML/JSON parses: **PASS** (workflow, manifest, overlay).
- AC #8: tile-asset diff **empty**.
- AC #1 (`makeappx pack` exit 0): **expected-pending** — requires a Windows SDK runner; the workflow step itself is the standing check once dispatched.
- AC #7: no repo secret-scan gate exists yet (TASK-IMP-003 draft) — manual assert: all credentials are `${{ secrets.* }}` references; recorded honestly.
- run-gates.sh floor: GREEN (unchanged).

*End phase bundle.*
