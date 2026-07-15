---
artefacts: repo-context-map@1 + edge-case-matrix@1 + implementation-plan@1 + observability-injection@1 (bundled - single-module task)
task_id: TASK-SKILL-116
created: 2026-07-12
verdicts: all pass (respective audit skills)
---
# Phase artefacts - TASK-SKILL-116

## Repo context map
Patterns: bash set -uo pipefail + exit 0/10/2 contract + `cyberos-install:` prefix (pinned_in: check-version-sync.sh, TASK-IMP-068); vendored set = single $skills string at build.sh:28; chain doc = modules/cuo/.../ship-tasks.md skill_chain (`skill: <name>` entries); command docs name skills as backticked tokens.
files_outside_immediate_domain: 0 (all under tools/cyberos-install/) -> no ADR (steps 3-4 skip).
has_external_dependency: false -> steps 7-8 skip.
module_placement_warning: null.

## Edge-case matrix (9 rows)
| # | category | trigger | expected | covered by |
|---|---|---|---|---|
| 1 | null/empty | payload dir missing | exit 2 | t06 harness |
| 2 | null/empty | 0 skill refs extracted (doc format changed) | exit 2, never vacuous pass | t02b |
| 3 | bounds | skill name at doc line boundaries / duplicated refs | dedup via sort -u, single verdict | t02 |
| 4 | malformed | workflow doc missing from payload | exit 2 naming the doc | t06 harness |
| 5 | malformed | allowlist entry with only a comment | ignored, no phantom allow | t04 |
| 6 | concurrency | check run while payload rebuilt | read-only fail-closed (missing file -> 2) | t06 |
| 7 | SECURITY | allowlist typo cannot silently skip a real miss | typo name never matches -> MISSING still reported | t04b |
| 8 | DEGRADATION | grep/awk absent | coreutils presumed (same floor as build.sh); missing doc/tool -> exit 2 loud | t06 harness |
| 9 | null/empty | payload with author but no audit twin | exit 10 UNPAIRED | t05 |

## Implementation plan (estimate 2 pts)
1. check-chain-coverage.sh: extract refs (chain doc `skill:` keys + command-doc backticked `*-author|-audit` tokens), allowlist (env-overridable path, comment-stripping), MISSING + UNPAIRED rules, read-only, exit 0/10/2. (§1 #2,#3,#4,#6; rows 1-9)
2. chain-allowlist.txt: awh-gate, caf-gate with reasons. (§1 #3)
3. build.sh: add debugging-cycle pair to the set; run the check as the final build step. (§1 #1,#5)
4. tests t01-t06. (§5)

## Observability
Success line `chain OK: N referenced, M vendored, K allowlisted`; every failure branch prints MISSING/UNPAIRED lines (stdout) or `cyberos-install: ERROR:` (stderr, exit 2); allowlist rot warns on stderr. No silent branch. PII: none.
