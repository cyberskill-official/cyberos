---
task_id: TASK-SKILL-118
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# TASK-SKILL-118 audit

## §1 - Verdict summary

Audited with special attention to scope integrity (six pairs, additive-only) and to the honesty of an abbreviated new_files list against a 72-file real matrix. The decisive property - rubrics ENCODE existing prose gates rather than inventing policy - is enforced by the prose->rule mapping requirement. Traceability closes over t01-t06 in tools/cyberos-install/tests/test_pair_parity.sh.

## §2 - Findings (all resolved)

### ISS-001 rubrics could silently raise the bar
Nothing stopped a rubric from being stricter than the SKILL.md prose it encodes. Resolved: AC 2 requires a prose->rule mapping table per rubric; unsourced rules become review findings (§10 #1).

### ISS-002 new_files understated the change surface
13 listed files vs 72 real ones reads as evasion. Resolved: §6 states the full matrix explicitly and names the parity checker as the completeness authority gating AC 1 - the abbreviated list cannot hide a missing file.

### ISS-003 coverage threshold hardcode
Encoding 90 as a literal would collide with TASK-CUO-207's config override a wave later. Resolved: §1 #4 named constants + the override hook cited in the rubric header contract (§3).

### ISS-004 artefact stability unguarded
"Additive only" needed a check, not a promise. Resolved: AC 4 diff-scope guard over each pair's artefact-spec section.

### ISS-005 trigger contracts at risk
Rewriting SKILL.md files invites description drift that would break TASK-SKILL-111/112 trigger tests. Resolved: §1 #7 byte-stability rule + AC 6 sha256 assertion on the six TRIGGER_TESTS.md.

### ISS-006 checker rigidity
Skills legitimately differ (backlog-state-update needs no references/ tree in the same shape). Resolved: the file-class arrays at the top of the checker ARE the policy, changeable only by editing this task's clauses (§10 #3) - no per-skill exceptions smuggled in code.

## §3 - Resolution

All six findings addressed as cited. Sequencing note (land before TASK-CUO-205's @2 bump) is recorded on both tasks. **Score = 10/10.**

*End of TASK-SKILL-118 audit.*

## §4 - Ship record (2026-07-12)

- Implementation: 86 files across 8 pairs (six §1 pairs + debugging-cycle full-raise + spike acceptance
  READMEs), check-pair-parity.sh + build.sh hookup + test_pair_parity.sh; commits 247f021, e63f0fd.
  Phase artefacts: docs/tasks/.workflow/TASK-SKILL-118/.
- Recorded deviation (newest wins): BSU rubric versioned @2.0 - TASK-CUO-205 landed first; its §7
  migration path followed. ISS-003's override hook present in every rubric header.
- Review: human verdict at gate 1 APPROVE + pre-authorize done (Stephen Cheng, in-chat).
- Testing: t01-t06 6/6, all 6 cyberos-install suites, full-profile build green with parity gate live
  (52 skills, plugin 1.09 MB < 2 MB). Gate 2 recorded per pre-authorization.
- Field findings folded back: t04B repo-VERSION dependence fixed (TASK-IMP-070 audit note); first
  manifest-tracked ship run (TASK-CUO-206 dogfood) - manifest created, stepped, finalized-deleted at done.

Verdict unchanged: PASS, Score = 10/10.

- 2026-07-12: t04 amended to at-rest semantics (warn+defer on dirty worktree) after three mid-flight false-fires. Verdict unchanged: PASS, Score = 10/10.
