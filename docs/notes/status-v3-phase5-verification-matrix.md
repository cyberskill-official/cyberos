# Status v3 — Phase 5 verification matrix

Date: 2026-07-27 (updated after Batch B closeout)  
Branch: `main` @ `51041133` (+ closeout branch regen)  
Tasks: TASK-DOCS-022, TASK-DOCS-023  
Scope: verify packaging/channels **without** publish/deploy/tag.

| Channel | Artifact | Result | Evidence |
| --- | --- | --- | --- |
| Git installer (`build.sh` → `install.sh`) | `.cyberos/` vendored tree | PASS | `bash tools/install/build.sh` + offline cert suite 9/9 |
| Payload stamps | `check-version-sync.sh` | PASS | sync OK across payload artifacts (1.11.0 line) |
| Payload SBOM | `emit-payload-sbom.sh` | PASS | CycloneDX emit (prior Phase 5 pass) |
| npm pack (dry) | `@cyberskill/cyberos` | PASS | prior dry-run included docs-tools + checker |
| Plugin `/cyberos:status` + `/help` | command text | PASS | tabless status-hub@3 wording |
| Desktop / MAS / MS Store / Snap / flathub / homebrew / winget / updater / android / ios | version-stamped apps | N/A | apps do not embed `docs/status`; record only |
| GHCR service images | deploy.yml services | N/A | Unaffected; version stamp only |
| Offline scratch certification | payload-only install | PASS | `test_offline_status_cert.sh` **9/9** (re-run 2026-07-27) |
| Live docs curl | served v3 page | PASS (read-only) | `https://os.cyberskill.world/docs/reference/status.html` → `data-template-id="status-hub@3"`, VERSION 1.11.0, legacy URL 200. No deploy performed this session. |
| Fleet install test | `fleet-install-test.sh` | SKIPPED (intentional) | Script runs `install.sh` across ~23 consumer repos and dirties worktrees — P7-adjacent. Offline cert + live curl cover packaging truth. **Re-run under P7 GO** before/with pilots. |

## Defaults recorded (P0 #3)

`traceability.scaffold_ci` defaults to **false (opt-in)**.

## Related closeouts

- P0 decisions: [`status-v3-p0-decisions.md`](status-v3-p0-decisions.md)
- DOCS-021 triage: [`status-v3-do021-violation-triage.md`](status-v3-do021-violation-triage.md) — recommend accept 148 as pre-cutoff history
- P2 review package: [`status-v3-p2-review/README.md`](status-v3-p2-review/README.md)

## Not done here (operator HITL / later phases)

- P2: operator page review (paper + night, file:// + served)
- P6: v2.0.0 tag; flip release-range preflight to blocking
- P7: fleet rollout (`commit: all cleared`, `push: none`; skip `cyberos-12g-clone`); includes full `fleet-install-test.sh`
- Publish/tag of any channel
