---
fr_id: FR-MEMORY-120
audited: 2026-05-19
verdict: PASS
score_pre_revision: 9.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 3
template: engineering-spec@1
---

## §1 — Verdict summary

FR-MEMORY-120 authored direct-to-10/10. ~750 lines. 15 §1 normative clauses (read-only, CLI surface, HistoryEntry shape, extras inline annotation, follow-moves default, since filter, unified diff, all chain kinds rendered, tombstone+purge handling, REST endpoint, latency budget, --all-paths, never-existed handling, filter grammar stretch, --follow stretch). 7 §2 rationale paragraphs. Python module + CLI + REST integration scaffold in §3. 21 ACs all `traces_to: §1 #N`. 15 + 4 = 19 pytest tests. 18 failure modes. 8 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Move-following default
First sketch defaulted `--no-follow-moves`. Risk: operators surprised when history is incomplete after a rename. Resolved: §1 #5 + DEC-263 + §2 rationale paragraph; default follows moves, `--no-follow-moves` is the explicit opt-out.

### ISS-002 — Annotation rendering grammar
The audit chain has rich `extra` data (dream_id, session_id, etc.) but no convention for surfacing it. Risk: human output is sparse. Resolved: §1 #4 — defines recognised annotation patterns with inline rendering; AC #10/#11 cover.

### ISS-003 — Aux rows are part of history
First sketch only included put/move/delete in history. Risk: invisible importance scoring, dream applications, etc. Resolved: §1 #8 includes all rows touching the path with structured kind; ACs #12 + #16 + #21 cover.

## §3 — Rubric scorecard

| Rule | Pass | Notes |
|---|---|---|
| FM-001..004 | ✓ | Frontmatter clean |
| FM-101..111 | ✓ | Title 153 chars (project convention) |
| SEC-001..009 | ✓ | All required sections present |
| COND-001..004 | n/a | client_visible: false |
| QA-001..009 | ✓ | Alternatives discussed; scope clear |
| SAFE-001..004 | n/a | No untrusted_content blocks |
| TRACE-001 | ✓ | Coverage: §1 #1→AC21, #2→AC1-AC9/AC19, #3→AC1/AC2/AC3, #4→AC10/AC11/AC12, #5→AC8/AC9, #6→AC5, #7→AC6/AC15, #8→AC12/AC16, #9→AC13/AC14, #10→AC17, #11→AC18, #12→AC19, #13→AC20, #14/#15 SHOULD (deferred) |
| TRACE-002 | ✓ | Every AC has a §5 test fn |
| TRACE-003 | ✓ | Test paths in `frontmatter.new_files` |
| TRACE-004 | n/a | status: draft |
| TRACE-005 | n/a | Stretch SHOULDs deferred to slice 4 |

### Score derivation
- Pre-revision: 9.0/10 (move-following ambiguity + aux row inclusion)
- Post-expansion: 9.5/10 (DEC-260..263 + §2 rationale + AC traceability)
- Post-revision: **10/10** — AC #17 REST/CLI parity + AC #21 read-only assertion close the loop

## §4 — Resolution

All 3 mechanical concerns addressed during authoring. **Score = 10/10.** No protocol amendment required — pure projection over existing data. Ready for implementation as soon as Stephen approves.

---

*End of FR-MEMORY-120 audit.*
