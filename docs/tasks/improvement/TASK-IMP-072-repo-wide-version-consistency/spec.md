---
id: TASK-IMP-072
title: "Repo-wide version consistency - every version-bearing file moves with VERSION, enforced at bump, gate, and hook"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: improvement
priority: p0
status: done
verify: T
phase: Wave E - 1.0.0 hardening closeout
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-IMP-068, TASK-IMP-071]
depends_on: []
blocks: []
source_pages:
  - scripts/stamp-release-version.mjs
  - .github/workflows/version.yml
source_decisions:
  - "2026-07-12 operator requirement for 1.0.0: the version must be consistent across the WHOLE codebase and submittable across all platform stores. Audit found apps/web + android + ios + tauri + Cargo stuck at 0.1.0/10701 (stamped only at release-build time)."
language: javascript (node stdlib) + yaml + bash
service: scripts/
new_files: []
modified_files:
  - scripts/stamp-release-version.mjs
  - .github/workflows/version.yml
  - .github/workflows/payload-gate.yml
  - .githooks/pre-commit
---

# TASK-IMP-072: Repo-wide version consistency

## §1 - Description

1. The stamper's coverage MUST be total: tauri.conf.json, src-tauri/Cargo.toml (about-dialog/crate metadata), apps/web/package.json, Android versionName+versionCode, iOS MARKETING_VERSION+CURRENT_PROJECT_VERSION, and the repo mcp/package.json (build.sh's payload stamp becomes a no-op).
2. version.yml's bump commit MUST run `stamp --apply` and commit every stamped file - a release commit carries the entire codebase to the new version, never just VERSION+CHANGELOG.
3. payload-gate MUST run `stamp --check --exit-code` so any manual drift is a red build.
4. The pre-commit hook MUST refuse a staged VERSION change whose artifacts drift, printing the exact one-line fix (`stamp --apply && git add -u`).
5. Store-side invariants MUST hold: versionCode/CURRENT_PROJECT_VERSION derive from the monotonic BUILD_NUMBER (never VERSION), and the Play high-water guard (>10700) stays.

## §2 - Why this design

Stamp-at-build kept binaries honest but let the repo lie between releases - a 1.0.0 that greps as 0.1.0 in five files is not "1.0.0 across the whole codebase". Three enforcement points (bump, gate, hook) close every path a stale stamp could survive.

## §3 - Contract

`node scripts/stamp-release-version.mjs [--apply|--check --exit-code]`; exit 10 on drift in check mode. Bump commit file set fixed in version.yml.

## §4 - Acceptance criteria

1. **Coverage total** (§1 #1) - `--check` at a fresh VERSION lists exactly the seven files; Cargo + mcp included.
2. **Bump carries all** (§1 #2) - version.yml's git add names every stamped file after the apply step.
3. **Gate red on drift** (§1 #3) - the payload-gate step exists; drift exits 10.
4. **Hook refuses drifting VERSION commits** (§1 #4) - staged VERSION + drift = commit rejected with the fix line.
5. **Monotonic store counters** (§1 #5) - versionCode==CURRENT_PROJECT_VERSION==BUILD_NUMBER; high-water guard intact.

## §5 - Verification

Live: `--check` output (7-file drift list at 0.4.0), hook rejection then acceptance around the 1.0.0 commit, gate step grep, version.yml git-add grep. Store counters asserted by the stamper's own output line.

## §6 - Implementation skeleton

stampCargo helper + two stampJson targets; three wiring blocks.

## §7 - Dependencies

Rides TASK-IMP-071 (bump commits now visible to CI, so the gate actually runs on them).

## §8 - Example payloads

`VERSION=1.0.0 BUILD_NUMBER=10706 (androidVersionCode + iosBuildNumber)` + `all release artifacts already match VERSION`.

## §9 - Open questions

None blocking. apps/console carries no version field (nothing to stamp); services' Cargo.tomls version internal crates, deliberately out of scope (they are not store artifacts).

## §10 - Failure modes inventory

1. New version-bearing file added later - the gate misses it until listed; the stamper header documents "add here + version.yml git add".
2. BUILD_NUMBER reused after a failed upload - Play/ASC reject; bump BUILD_NUMBER, never reuse.
3. Hook bypassed with --no-verify - payload-gate catches on push.
4. Stamp regex misses a reformatted pbxproj - check mode reports non-match as drift, loudly.
5. mcp double-stamp (repo + build.sh) - idempotent by construction.

## §11 - Implementation notes

The 1.0.0 release commit itself is the first full-codebase stamp (operator-prepared, hook-verified).

*End of TASK-IMP-072.*
