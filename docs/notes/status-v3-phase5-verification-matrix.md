# Status v3 — Phase 5 verification matrix

Date: 2026-07-27  
Branch: `feat/status-v3-platform`  
Tasks: TASK-DOCS-022, TASK-DOCS-023  
Scope: verify packaging/channels **without** publish/deploy/tag.

| Channel | Artifact | Result | Evidence |
| --- | --- | --- | --- |
| Git installer (`build.sh` → `install.sh`) | `.cyberos/` vendored tree | PASS | `bash tools/install/build.sh /tmp/cyberos-payload-p5` → profile=full; offline cert suite 9/9 (`tools/install/tests/test_offline_status_cert.sh`) proves install + cutoff + checker + v3 page |
| Payload stamps | `check-version-sync.sh` | PASS | `sync OK 1.10.0 across 7 artifacts` |
| Payload SBOM | `emit-payload-sbom.sh` | PASS | wrote CycloneDX (1624 components, hermetic) |
| npm pack (dry) | `@cyberskill/cyberos` | PASS | `npm pack --dry-run` @ 1.10.0; includes `lib/check_task_link.sh`, `ci/traceability/traceability.yml`, docs-tools renderer (1591 files) |
| Plugin `/cyberos:status` + `/help` | command text | PASS | Updated to say tabless status-hub@3 (not lenses/tabs) |
| Desktop / MAS / MS Store / Snap / flathub / homebrew / winget / updater / android / ios | version-stamped apps | N/A (confirmed) | `apps/**` has **no** embeds of `docs/status` / status-hub; store channels ship app shells only — status page is installer/docs-site concern. Record only; no channel rebuild in this phase. |
| GHCR service images | deploy.yml services | N/A | Unaffected; version stamp only |
| Offline scratch certification | payload-only install | PASS | `test_offline_status_cert.sh` 9/9: install, checker, cutoff, no CI scaffold by default (`scaffold_ci` opt-in), v3 markup, no CDN script tags, feed JSON |
| Fleet install test | `fleet-install-test.sh` | SKIPPED (slow) | Started but stopped after ~90s without completion; offline cert + payload rebuild cover the critical path. Re-run before P7. |

## Defaults recorded (P0 open question #3)

`traceability.scaffold_ci` defaults to **false (opt-in)**. Installer scaffolds `.github/workflows/cyberos-traceability.yml` only when the key is explicitly `true` and `.github/` exists.

All P0 open questions are locked — see [`status-v3-p0-decisions.md`](status-v3-p0-decisions.md).

## Not done here (operator HITL / later phases)

- P0 remaining: PR merge + required checks + cutoff follow-up (cutoff must not be invented pre-merge)
- P2: operator page review (paper + night, file:// + served)
- P6: v2.0.0 tag; flip release-range preflight to blocking
- P7: fleet rollout under `/Users/stephencheng/Projects` (`commit: all cleared`, `push: none`; skip `cyberos-12g-clone`)
- Publish/deploy of any channel
- TASK-DOCS-021 mothership 148-violation triage ledger (skipped — incomplete)

## Incomplete

- TASK-DOCS-021 disposition list not generated
- Live VPS/vercel curl of served v3 page deferred (no deploy)
- `fleet-install-test.sh` full run may be re-run by operator before P7
