---
artefacts: repo-context-map@1 + edge-case-matrix@1 + implementation-plan@1 + observability-injection@1 (bundled)
task_id: TASK-SKILL-117
created: 2026-07-12
verdicts: all pass (respective audit skills)
---
# Phase artefacts - TASK-SKILL-117

## Repo context map
Patterns: skill contracts under modules/skill/<name>/ with the Identity / Scope / Inputs / Triggers frontmatter blocks (pinned_in: debugging-cycle-author, implementation-plan-author); TRIGGER_TESTS header (skill_id, min_confidence 0.7, classifier_version 3.0.0-a4) with positive/negative sections (TASK-SKILL-112 lineage); descriptions < 1024 chars (host limit, TASK-SKILL-111/plugin parity fix lineage); audit twins carry RUBRIC/AUDIT_LOOP/REPORT_FORMAT.
files_outside_immediate_domain: 1 (architecture-decision-record-author/SKILL.md input wiring) -> <= 3, no ADR step.
has_external_dependency: false -> mock steps skip. NOT vendored here (TASK-CUO-209 owns build.sh expansion per task §11).

## Edge-case matrix (8 rows)
| # | category | trigger | expected | covered by |
|---|---|---|---|---|
| 1 | null/empty | spike with zero probed options | SPK-EVID-001 fail | audit TRIGGER_TESTS case table |
| 2 | null/empty | empty discard log with rejected options | SPK-DISC-001 fail | case table |
| 3 | bounds | actual_hours > 1.5x timebox, no HALT recorded | SPK-BOX-003 fail | case table |
| 4 | bounds | recommendation names an unprobed option | SPK-STRUCT-003 fail | case table |
| 5 | malformed | evidence entry is an uncited assertion | SPK-EVID-002 fail | case table (AC 4 fixture) |
| 6 | malformed | spike_id not SPIKE-<task-ID>-<n> | SPK-STRUCT-004 fail | case table |
| 7 | SECURITY | evidence citing a file that does not resolve at audit time | SPK-EVID-002 fail (citation must be checkable) | case table |
| 8 | DEGRADATION | no spike exists for an ADR (lean profile) | ADR proceeds with evidence inline (fallback wired in ADR-author) | AC 6 grep |

## Implementation plan (estimate 2 pts)
1. author: SKILL.md (artefact schema, evidence rule, timebox HALT), PIPELINE, INVARIANTS, envelopes, FAILURE_MODES, TRIGGER_TESTS (>= 6 cases). (§1 #1,#2,#3,#6)
2. audit: SKILL.md, RUBRIC (SPK-STRUCT/EVID/BOX/DISC, 10/10), AUDIT_LOOP, REPORT_FORMAT, envelopes, TRIGGER_TESTS + fixture case table. (§1 #4,#6)
3. ADR-author wiring: architectural-spike@1 named as spike input + lean fallback. (§1 #5)
4. Layout parity per new_files. (§1 #7)

## Observability
Author records plan/actual hours + halted flag in the artefact and audit row (architectural_spike_authored); audit emits score + per-rule findings (architectural_spike_audited). All refusals name their rule id. PII: none.
