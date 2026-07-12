# FR-APP-006 — ship-workflow phase bundle (steps 1–12)

Run: 2026-07-13, ship-feature-requests v2.4.0. `queue: picked FR-APP-006 (priority=SHOULD, slice 4) — 5th of 5; FR-IMP-073/003/004/005 parked at HITL gate 1`.

## Steps 1–2 — repo context map

- **Patterns:** gated dispatch + unconditional guard/anchor job (batch convention); `gh release download` + hash re-derivation against the always-on `desktop` job's artifacts; split-pattern self-excluding guards (release-snap.yml precedent).
- **Grounding:** repo slug + `releases/download/v<ver>/<file>` URL shape confirmed from tauri.conf.json's production updater endpoint; `v<semver>` tag convention from release.yml; pre-existing submission-command matches in `.github/`+`tools/`: **0** (verified before authoring).
- **Blast radius:** 6 new files across 3 new dirs; `modified_files: []` honored. Outside-domain: 2 → **ADR skipped**. **Mocks skipped** — external monorepos are human-gated submission targets, not services this FR calls.

## Steps 5–6 — edge-case matrix

| # | Category | Case | Covered by |
|---|---|---|---|
| 1 | null/empty | either `PKGMGR_*_RELEASE` unset | independent job `if:` skips + one anchor for both AC #4 assertions |
| 2 | null/empty | release has no matching artifact | `gh release download --pattern` fails the step loudly (set -euo) |
| 3 | malformed | placeholder survives a render | explicit post-render asserts (grep computed-SHA present + placeholder absent) — proven in dry-run |
| 4 | malformed | sed pattern misses a field after manifest refactor | same asserts catch it (draft without this run's SHA fails) |
| 5 | security | submission command added under any flag | AC #3 standing guard, checks (a)+(b), split patterns so it never matches itself — proven clean including against my own new workflow |
| 6 | security | PAT provisioned prematurely / over-scoped | no PAT exists in this FR; future-FR scoping rule documented (answer sheet + spec §11) |
| 7 | bounds | tag with/without leading `v` | `VER="${TAG#v}"` normalization in both jobs |
| 8 | regression | flags coupled in future refactor | two independent `if:` conditions + per-job AC #4 assertions |
| 9 | degradation | `--clobber` re-upload staling an earlier run's hash | ecosystem-side hash verification rejects (never silently accepted); re-run-before-submit rule in answer sheet (spec §10) |
| 10 | degradation | livecheck strategy vs tag-format mismatch | Homebrew-side detection; pre-submission verify item in answer sheet |
| 11 | degradation | zap trash: wrong paths | real `brew uninstall --zap` test required before finalization (spec §1 #9) — audit tools structurally cannot catch it |
| 12 | degradation | wrong NSIS silent switch | AC #8 real-installer test recorded in the answer sheet before manifest finalization |

## Steps 9–10 — implementation plan (executed)

1. `homebrew-cask-manifest/cyberos.rb` — §3 contract + in-file hedge comments (placeholder sha, zap-candidates status). ✅
2. winget three-file set — installer per §3; version + locale authored to the schema's minimum with the same ManifestVersion hedge (spec §2 defers these to WORKER = this run). ✅
3. `release-pkgmgr-pr.yml` — §3 skeleton upgraded: placeholder echoes replaced with REAL render logic (sed re-derivation + post-render asserts), guard/anchor job added, submission-command strings kept out of all comments. ✅
4. `docs/deploy/package-manager-submission.md` — Cask quality bar (8 rows), winget pipeline checklist (6 rows), shared ops rules (clobber staleness, PAT scoping, bump-PRs-also-gated). ✅

## Steps 11–12 — observability injection

Both prep jobs echo `tag/version/sha` before rendering; every assert prints a named `::error::`; the guard prints per-check outcomes. All new error branches emit diagnostics before failing.

## Machine verification (this session)

- `ruby -c` cask manifest: **Syntax OK** (real interpreter).
- YAML parses ×4 (3 winget files + workflow): **PASS**.
- AC #3 checks (a) and (b): **PASS** (0 matches repo-wide, including this FR's own additions).
- **Render dry-run (both ecosystems) with a fake artifact: PASS** — version/URL/sha substituted, placeholders eliminated, outputs still valid ruby/YAML, asserts fire.
- AC #1 (`brew audit/style`) + AC #2 (`winget validate`): **expected-pending** — tools unavailable here; both are pre-PR requirements in the answer sheet, run against RENDERED drafts.
- AC #8 (NSIS `/S` real test): **pending-human** (needs a Windows machine + real installer).
- AC #7: manual secret assert (no scan gate yet — FR-IMP-003 draft); zero credentials referenced at all in this FR.

*End phase bundle.*
