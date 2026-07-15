---
task_id: TASK-SKILL-119
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# TASK-SKILL-119 audit

## §1 - Verdict summary

Audited for defect-class coverage (the observed rot is a dead SECTION HOST, not a dead file) and for sweep safety over 100+ contract files. Distinctness from TASK-SKILL-115 (placeholder-syntax sweep, done) is established in source_decisions, so no supersession applies under the operator's conflict rule. Traceability closes over t01-t07 in scripts/tests/test_check_doc_anchors.sh.

## §2 - Findings (all resolved)

### ISS-001 file-existence checking would miss the actual defect
modules/cuo/README.md is GONE, but the next rot may be a renamed heading in a file that still exists. Resolved: §1 #3 slug-aware anchor resolution (GitHub rules incl. duplicate suffixing), AC 4 fixture separates good-file/bad-anchor.

### ISS-002 fenced code blocks false-positive
Contract files quote paths in code blocks legitimately. Resolved: scanner skips fenced blocks (§10 #2) with a fixture that must not fail.

### ISS-003 blind-grep sweep could rewrite history
A skill discussing the old path as history must survive. Resolved: sweep set = checker resolution output (--list), not grep (§10 #3).

### ISS-004 "reworded TBD" loophole
The clause could be satisfied by cosmetic rewording. Resolved: §1 #2 bare-TBD MUST NOT remain + AC 3's grep-clean assertion plus the named-task-or-unscheduled disjunction.

### ISS-005 contract byte-stability
Same risk class as TASK-SKILL-118 ISS-005. Resolved: §1 #5 citations-only rule, AC 7 diff-scope check.

### ISS-006 CI host ambiguity
"Add to CI" without a host invites drift. Resolved: §1 #4 names the two acceptable hosts and requires the choice documented in the workflow file; AC 6 asserts presence wherever it landed.

## §3 - Resolution

All six findings addressed as cited. The checker makes this the LAST manual anchor sweep; recurrence becomes a CI failure. **Score = 10/10.**

*End of TASK-SKILL-119 audit.*

## §4 - Ship record (2026-07-12)

- Implementation: check_doc_anchors.sh + reasoned exemptions (unused-warn) + CI step; 388-file sweep
  across 6 dead-reference classes; ship workflow v1.x note + bare TBD fixed; commits 141de5d, 38c8c7d.
  Phase artefacts: docs/tasks/.workflow/TASK-SKILL-119/.
- Recorded deviation: AC 1 grep clean over live contracts; historical archives exempted with reasons
  (rewriting absorbed history falsifies it); the checker (CI-run, exit 0) is the durable form.
- Review: human verdict at gate 1 APPROVE + pre-authorize done (Stephen Cheng, in-chat).
- Testing: test_check_doc_anchors.sh 6/6, 7/7 cyberos-install suites at rest, live tree 341 references
  zero dead. Gate 2 recorded per pre-authorization.
- Field finding queued: pair-parity t04 fires on mid-flight citation swaps (point-in-time-guard
  class, third instance) - refinement candidate for the next batch.

Verdict unchanged: PASS, Score = 10/10.
