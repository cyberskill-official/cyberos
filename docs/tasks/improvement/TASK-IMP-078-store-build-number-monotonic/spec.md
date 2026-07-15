---
id: TASK-IMP-078
title: "Store build-number monotonicity — release-time floor decouples re-tags from BUILD_NUMBER bumps (Play versionCode 10706 collision)"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-13T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: improvement
priority: p0
status: done
verify: T
phase: "Wave 6 - go-live (Track B: mobile shells)"
owner: Stephen Cheng (CTO)
created: 2026-07-13
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-IMP-073, TASK-IMP-077]
depends_on: []
blocks: []
source_pages:
  - "v1.0.0 re-tag android run log 2026-07-13: r0adkll/upload-google-play 'Error: Version code 10706 has already been used.' - build green, upload rejected"
  - "scripts/cyberos-version.mjs apply(): BUILD_NUMBER increments ONLY when VERSION changes; Release-As: 1.0.0 pins VERSION, so every pinned re-tag rebuilt the same 10706"
  - "same run, iOS lane GREEN: ASC accepted CFBundleVersion 10706 for 1.0.0 - meaning the NEXT re-tag would have failed the iOS lane with the identical duplicate-build rejection"
source_decisions:
  - "2026-07-13 Stephen: android run screenshot (Publish to Google Play failed, version code 10706 already used)."
language: node (one stamper flag), YAML (two step args), markdown
service: .github/workflows + scripts
new_files: []
modified_files:
  - scripts/stamp-release-version.mjs
  - .github/workflows/release.yml
  - docs/deploy/RELEASE.md
  - docs/deploy/GO-LIVE-CHECKLIST.md
effort_hours: 1
subtasks:
  - "stamp-release-version.mjs: --store-monotonic flag, effective code = max(BUILD_NUMBER, minutes-since-epoch) - DONE, live-verified (29731417 stamped, baseline restored)"
  - "release.yml: android + iOS stamp steps pass the flag - DONE, YAML parses"
  - "RELEASE.md + GO-LIVE-CHECKLIST: document the floor, refresh android/iOS status - DONE"
risk_if_skipped: "Every re-tag of a pinned version re-offers a consumed build number: android fails today (10706 at Play), and the moment android is fixed any other way, the next run fails the iOS lane instead (10706 already at ASC for 1.0.0). Release-by-re-tag stays permanently one-shot per version bump."
---
## §1
1. `stamp-release-version.mjs` **MUST** accept `--store-monotonic`: effective build number = `max(BUILD_NUMBER, floor(now/60s))`. Without the flag, behavior is byte-identical to today (committed baseline, `--check` drift detection, version.yml bump commits all unchanged).
2. `release.yml`'s android and iOS stamp steps **MUST** pass the flag; no other call site does. The high-water-mark guard keeps validating the FILE value (the committed floor), and the effective value can only be >= it.
3. `BUILD_NUMBER` keeps its existing role and is NOT bumped here: one mechanism owns re-tag safety (the wall-clock floor), one owns the committed baseline (version bumps). A second incrementer was considered and rejected - it re-introduces the operator-memory dependency this task removes.
4. Wall-clock minutes, not commit timestamps: a re-tag of the SAME commit must still get a fresh number (commit-time would collide; run-time cannot). ~29.7M in 2026 vs Play's 2100000000 cap - headroom measured in millennia.

*Lean profile: one flag + two step args; defect, fix, and both-mode behavior machine-verified in-session; store acceptance proven by the next tag run.*

## §5 (run 2026-07-13)
- `node --check` PASS; plain `--check` run: "all release artifacts already match VERSION" (baseline untouched). PASS
- `--apply --store-monotonic`: stamped versionCode/CURRENT_PROJECT_VERSION 10706 -> 29731417 (> 10706, < 2.1e9), both pbxproj occurrences; files then restored to committed baseline. PASS
- release.yml YAML parses; both stamp steps carry the flag (`grep -c 'store-monotonic' .github/workflows/release.yml` == 2 run-lines + comments). PASS
- Testing pass 2026-07-13 (post gate-1 "approve all"): full battery re-run green; fresh floor stamped 29731447 (> the earlier test's 29731417 - the clock floor visibly advances between runs), baseline restored.

## §9
- Two jobs in one run stamp slightly different numbers (independent clocks). Cosmetic only - Play and ASC never compare; recorded, not fixed.
- Should `cyberos-version.mjs` fast-forward BUILD_NUMBER to the last shipped effective number at the next real version bump? Not required for correctness (the floor always wins at release time); revisit only if a human-readable committed number regains value.

## §10
| Failure | Detection | Recovery |
|---|---|---|
| two release runs inside the same minute | store rejects the second upload, loud | re-run: next minute, next number |
| runner clock skew backwards | floor still >= committed BUILD_NUMBER; worst case equals a consumed number -> loud store rejection | re-run after NTP settles |
| flag typo'd/dropped in a future workflow edit | next re-tag reproduces the 10706-class failure loudly at upload | re-add flag; this spec's §5 grep pins the 2 call sites |
| someone passes the flag in version.yml | committed files would carry a wall-clock number - visible in the bump commit diff, and `--check` flags drift on the next run | revert; flag is release.yml-only by §1 clause 2 |
*End of TASK-IMP-078.*
