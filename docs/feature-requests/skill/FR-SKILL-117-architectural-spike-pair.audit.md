---
fr_id: FR-SKILL-117
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# FR-SKILL-117 audit

## §1 - Verdict summary

Audited for artefact-contract completeness (every field typed), rubric enforceability (rules an auditor can apply without judgment calls), and layout parity with the four full pairs. The evidence rule and the timebox both moved from aspiration to checkable invariants during revision. Verification is acceptance-driven per skill-contract conventions; both TRIGGER_TESTS.md files are in new_files (TRACE-003 closed).

## §2 - Findings (all resolved)

### ISS-001 evidence was subjective
"Options with evidence" invited opinion-spikes. Resolved: §1 #3 checkable-citation rule (file path, command+output, or URL); SPK-EVID rejects uncited claims, AC 4 pins the fixture.

### ISS-002 timebox had no mechanism
A timebox nobody records is prose. Resolved: §1 #2 plan-vs-actual recording + HALT at >1.5x; AC 3 requires it in PIPELINE.md and INVARIANTS.md.

### ISS-003 recommendation could name an unprobed option
Structural hole in the artefact contract. Resolved: SPK-STRUCT rule (recommendation must name exactly one probed option), §10 #3.

### ISS-004 no-spike fallback undefined
ADR-author consumes spike output; repos without spikes would stall. Resolved: §1 #5 lean-profile fallback (evidence inline in the ADR options table), AC 6.

### ISS-005 layout parity incomplete
First cut omitted the audit-side AUDIT_LOOP.md/REPORT_FORMAT.md from new_files, which would fail FR-SKILL-118's parity checker on arrival. Resolved: new_files completed to the full-pair file classes, AC 7.

### ISS-006 confidence inflation unchecked
`confidence: high` cost nothing. Resolved: rubric cross-check (high with single-evidence options -> SPK-EVID finding), §10 #5.

## §3 - Resolution

All six findings addressed as cited. The pair lands vendorable (FR-CUO-209) and parity-clean (FR-SKILL-118) by construction. **Score = 10/10.**

*End of FR-SKILL-117 audit.*
