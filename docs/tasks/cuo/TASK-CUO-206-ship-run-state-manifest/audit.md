---
task_id: TASK-CUO-206
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# TASK-CUO-206 audit

## §1 - Verdict summary

Audited hardest on the one danger a run-state manifest introduces: becoming a second source of truth. The revised spec keeps it strictly a cache (hash-verified on resume, gates always re-ask, deletable at zero correctness cost). Queue selection moved from prose to a total order. Traceability closes over the eight tests in modules/cuo/tests/test_ship_manifest.py (in new_files).

## §2 - Findings (all resolved)

### ISS-001 two-sources-of-truth risk
A trusted manifest could contradict the backlog. Resolved: cache-only doctrine (§2), resume re-verifies every artefact hash (§1 #3), HITL gates re-ask regardless of manifest content (§1 #8, AC 8), and deletion is always safe (§10 #4).

### ISS-002 task spec edits mid-flight were invisible
Steps proven against version N of the spec would resume against version N+1. Resolved: task_sha256 root field in the schema (§1 #1) with all-stale semantics; §3 example updated; §10 #3 cites the field instead of deferring it.

### ISS-003 mixed-workflow-version resume
2.3.0 manifest under a 2.3.1 workflow silently blends step semantics. Resolved: §1 #3 needs_human on version mismatch, AC 4.

### ISS-004 queue selection had undefined ties
"Next eligible" without a total order produces different picks per agent. Resolved: §1 #4 priority -> created -> id ordering with an operator-visible reasoning line, AC 5 determinism assertion.

### ISS-005 committed-or-ignored ambiguity
Manifests in git would churn every ship run; unstated either way invites both. Resolved: §1 #5 gitignore scaffold (repo + install.sh), AC 6.

### ISS-006 terminal-state handling
Manifests of done tasks would accumulate; route-backs would lose history. Resolved: §1 #6 delete-on-done, keep-with-incremented-count on route-back, AC 7 fixture pair.

## §3 - Resolution

All six findings addressed as cited. The task upgrades ship from restartable to resumable without moving any authority off task frontmatter. **Score = 10/10.**

*End of TASK-CUO-206 audit.*

## §10 - Ship record (2026-07-12)

- §10.1 Implementation: contract + helpers + tests + workflow v2.4.0 Resume semantics + scaffolds,
  commit f06ff65 (rebased onto PR #44 version-reset mid-flight; original hash 46911d8 superseded); phase artefacts at docs/tasks/.workflow/TASK-CUO-206/.
- §10.2 Review: clause-by-clause pass (packet in phase-bundle); human verdict at gate 1:
  APPROVE + pre-authorize done (Stephen Cheng, in-chat).
- §10.3 Testing: 8/8 AC tests, 100.0% statement coverage on modules/cuo/cuo/ship_manifest.py
  (raised from 77.3% by covering validate error branches + write_atomic failure cleanup),
  5/5 cyberos-install suites, git check-ignore proof. Gate 2 recorded per pre-authorization.
- §10.4 Field finding folded back: TASK-CUO-209 t08 temporal-scope guard amended to durable
  workflows_vendored_intact (TASK-CUO-209 §1 #8, AC 8, audit §11).

Verdict unchanged: PASS, Score = 10/10.
