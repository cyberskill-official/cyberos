# TASK-APP-003 — code-review packet (steps 17–18)

Status: `reviewing`. **HALTED at HITL gate 1 (review acceptance).** Diff under review: commit `e5b61de` (5 new files + 1 lint script; zero modifications to tauri.conf.json or release.yml).

## §1 clause → evidence map (all 8 clauses)

| §1 clause | Requirement | Evidence / named check | Verdict |
|---|---|---|---|
| 1 | Independent MAS overlay target; Developer ID config unmutated | `tauri.mas.conf.json` = §3 contract verbatim; `git diff HEAD~1 -- tauri.conf.json` empty (AC #3) | ✅ |
| 2 | App Sandbox on; every fs/network/IPC surface enumerated + dispositioned | 12-row audit table in `mac-app-store-submission.md` covering all 8 IPC commands, updater, webview, (absent) child procs, (absent) fs — from real source read, line references included | ✅ |
| 3 | inherit.plist exists, applied to child processes | File exists; workflow's helper-signing loop applies it to any non-main binary; today defensive-only (**no child processes exist** — spec §9 Q2 resolved with evidence) | ✅ |
| 4 | Signed App Store `.pkg` via productbuild | `release-mas.yml` "Build pkg installer" step + `pkgutil --check-signature` evidence (AC #4, exercised when gate flips) | ✅ contract |
| 5 | Gated behind new `MAS_RELEASE` var, default off | `if: vars.MAS_RELEASE == 'true'`; unconditional `assert-mas-gate-inert` job for AC #6 | ✅ |
| 6 | 3rd-Party cert pair, distinct from Developer ID; two distinct secrets | `MAS_APP_SIGNING_IDENTITY` (codesign) + `MAS_INSTALLER_SIGNING_IDENTITY` (productbuild) — never collapsed (spec §11 warning) | ✅ |
| 7 | No enrollment/credential acquisition by the agent | Nothing acquired; blockers table lists Stephen's 5 prerequisites; secrets referenced only as `${{ secrets.* }}` | ✅ |
| 8 | Answer sheet mirrors play-store pattern | 10-field ASC table + export compliance + privacy strings (not-applicable, audit-grounded) + age rating | ✅ (fields `pending-human` for your confirmation — AC #7 fully closes when you confirm them) |

## Findings the review should weigh (implementer-disclosed)

1. **Spec-skeleton bug fixed:** §3's child-signing step signed `Contents/MacOS/*` including the MAIN binary with the inherit plist — that would clobber the app's real entitlements. Implemented loop skips the main executable and re-seals the bundle only if a helper was signed. 
2. **Real blocker discovered (AC #1 audit):** `tauri-plugin-updater` is active in the base config → MAS bundle would self-update = sandbox + policy violation. Recorded as **hard blocker #1**; needs a small `mas` cargo-feature follow-up task (out of TASK-APP-003's scope per its own §11). `MAS_RELEASE` must stay off until then.
3. **Entitlement narrowed vs spec sketch:** `files.user-selected.read-write` dropped — zero fs access exists; keeping it would fail our own AC #2 lint and invite Apple scope-creep questions.
4. **`modified_files` ended up empty:** spec predicted tauri.conf.json + release.yml edits; neither was needed (AC #3 forbids the first; standalone workflow avoids the second).
5. AC #5 (`codesign`/`spctl` local run) is **expected-pending** — impossible without macOS + real certs; spec §5 explicitly allows documenting this state.

## Machine gates

Lint positive **OK (2/2)** + negative **exit 1** (both really run); YAML/XML/JSON parses all PASS; run-gates.sh floor GREEN (unchanged config); line coverage N/A (YAML/plist/docs/shell — declared, not fabricated).

## Reviewer verdict needed

**"TASK-APP-003 review: approved"** → ready_to_test, or **"TASK-APP-003 review: rejected — <reason>"** → routed back.

*End review packet.*
