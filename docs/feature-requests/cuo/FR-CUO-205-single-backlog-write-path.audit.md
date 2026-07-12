---
fr_id: FR-CUO-205
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# FR-CUO-205 audit

## §1 - Verdict summary

Audited for contract-evolution safety (a @1->@2 bump on a skill every ship run touches) and for exact grammar compatibility with the backlog regenerator. The decisive property - inserted rows indistinguishable from regenerated rows - is pinned by a byte-equality round-trip case. Verification is the executable case table at modules/skill/backlog-state-update-author/acceptance/INSERT_ROW_CASES.md (in new_files; TRACE-003 closed), CASE-01..08 mapping onto AC 1-8.

## §2 - Findings (all resolved)

### ISS-001 grammar drift risk vs the regenerator
Two writers to one row format eventually disagree. Resolved: §1 #3 regenerator-identical grammar + CASE-08 round-trip byte equality; §11 cross-cites regen_backlog() in both directions.

### ISS-002 concurrent-insert race unhandled
Two agents inserting the same FR row. Resolved: expected_absent pre-image gate + post-image single-occurrence check, deterministic fail-and-retry per put_if semantics (§10 #1).

### ISS-003 row could lie about status
An inserted row's status had no tie to the FR file. Resolved: BSU-INS-005 (row status == frontmatter status at write time), AC 6.

### ISS-004 whole-file discipline unstated
An insert mutation could smuggle unrelated edits. Resolved: BSU-INS-004 (no other line changed except that section's header counts), AC 5.

### ISS-005 @1 transition undefined
Existing artefacts must not fail retroactively. Resolved: §1 #1 transition window + CASE-07; sunset date recorded in the audit SKILL.md at implementation (§10 #5).

### ISS-006 ship-path regression surface
Touching this pair risks the 31-step chain's most-used skill. Resolved: §1 #6 ship-unchanged clause + AC 8 diff-clean assertion on the workflow doc.

## §3 - Resolution

All six findings addressed as cited. One audited write path to BACKLOG.md remains after this ships. **Score = 10/10.**

*End of FR-CUO-205 audit.*
