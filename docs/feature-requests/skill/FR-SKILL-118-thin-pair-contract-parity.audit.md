---
fr_id: FR-SKILL-118
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# FR-SKILL-118 audit

## §1 - Verdict summary

Audited with special attention to scope integrity (six pairs, additive-only) and to the honesty of an abbreviated new_files list against a 72-file real matrix. The decisive property - rubrics ENCODE existing prose gates rather than inventing policy - is enforced by the prose->rule mapping requirement. Traceability closes over t01-t06 in tools/cyberos-init/tests/test_pair_parity.sh.

## §2 - Findings (all resolved)

### ISS-001 rubrics could silently raise the bar
Nothing stopped a rubric from being stricter than the SKILL.md prose it encodes. Resolved: AC 2 requires a prose->rule mapping table per rubric; unsourced rules become review findings (§10 #1).

### ISS-002 new_files understated the change surface
13 listed files vs 72 real ones reads as evasion. Resolved: §6 states the full matrix explicitly and names the parity checker as the completeness authority gating AC 1 - the abbreviated list cannot hide a missing file.

### ISS-003 coverage threshold hardcode
Encoding 90 as a literal would collide with FR-CUO-207's config override a wave later. Resolved: §1 #4 named constants + the override hook cited in the rubric header contract (§3).

### ISS-004 artefact stability unguarded
"Additive only" needed a check, not a promise. Resolved: AC 4 diff-scope guard over each pair's artefact-spec section.

### ISS-005 trigger contracts at risk
Rewriting SKILL.md files invites description drift that would break FR-SKILL-111/112 trigger tests. Resolved: §1 #7 byte-stability rule + AC 6 sha256 assertion on the six TRIGGER_TESTS.md.

### ISS-006 checker rigidity
Skills legitimately differ (backlog-state-update needs no references/ tree in the same shape). Resolved: the file-class arrays at the top of the checker ARE the policy, changeable only by editing this FR's clauses (§10 #3) - no per-skill exceptions smuggled in code.

## §3 - Resolution

All six findings addressed as cited. Sequencing note (land before FR-CUO-205's @2 bump) is recorded on both FRs. **Score = 10/10.**

*End of FR-SKILL-118 audit.*
