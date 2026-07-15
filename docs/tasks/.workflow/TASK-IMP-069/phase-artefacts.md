---
artefacts: repo-context-map@1 + edge-case-matrix@1 + implementation-plan@1 + observability-injection@1 (bundled)
task_id: TASK-IMP-069
created: 2026-07-12
verdicts: all pass (respective audit skills)
---
# Phase artefacts - TASK-IMP-069

## Repo context map
Patterns: exit 0/10/2 + `cyberos-init:` prefix (TASK-IMP-068 lineage); release.yml = tag-driven job set (desktop/android/ios), top-level permissions contents: write; bootstrap legacy env CYBEROS_PACK_URL + top-level-dir tarballs; rollout takes payload dir as $1.
files_outside_immediate_domain: 2 (.github/workflows/release.yml, docs/deploy/RELEASE.md) -> <= 3, no ADR (channel decision recorded in task source_decisions + operator plan approval).
has_external_dependency: false (GitHub Releases is the deploy target, not a runtime dependency; tests run on file:// fixtures) -> steps 7-8 skip.

## Edge-case matrix (10 rows)
| # | category | trigger | expected | covered by |
|---|---|---|---|---|
| 1 | null/empty | payload without VERSION or cyberos.plugin | exit 2, nothing written | harness |
| 2 | null/empty | out-dir nonexistent | created by script | t01 |
| 3 | bounds | versioned vs stable asset names | byte-identical twins | t02 |
| 4 | malformed | tarball corrupted post-publish | sha256sum -c fails; bootstrap aborts pre-install | t03, t07 |
| 5 | malformed | legacy tarball with top-level dir | bootstrap unpacks via fallback, still works | code path (legacy branch) |
| 6 | concurrency | payload job races installer jobs on release creation | create-or-upload idempotent (`|| true` + --clobber) | t05 |
| 7 | SECURITY | missing SHA256SUMS beside tarball | bootstrap/rollout refuse to install unverified bits | t07 family |
| 8 | SECURITY | tag != VERSION at tag commit | job fails before any upload | t04, t05 |
| 9 | DEGRADATION | non-GNU tar host | detection: tar --version probe; recovery: exit 2 "run on ubuntu/CI"; test SKIPs visibly | t01 skip path |
| 10 | DEGRADATION | download failure mid-bootstrap | curl -f fails, set -e aborts, no partial .cyberos | t07 assertion |

## Implementation plan (estimate 3 pts)
1. release-assets.sh (§1 #1,#2; rows 1-3,8,9) 2. release.yml payload job (§1 #3,#4; rows 6,8)
3. bootstrap.sh rewrite w/ checksum + default release URL + legacy compat (§1 #5; rows 4,5,7,10)
4. rollout.sh --from-release (§1 #6; row 7) 5. README + RELEASE.md real URLs (§1 #7).

## Observability
Every script announces download URL, verification source, and asset summary; failures name the refusing step; Actions steps named per intent. PII: none.
