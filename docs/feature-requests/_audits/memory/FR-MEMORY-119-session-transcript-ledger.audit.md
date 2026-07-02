---
fr_id: FR-MEMORY-119
audited: 2026-05-19
verdict: PASS (gated on protocol amendment)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
gate: "Requires `APPROVE protocol change P22 §18` chat-turn. Spec-side audit independent of that gate."
---

## §1 — Verdict summary

FR-MEMORY-119 authored direct-to-10/10. ~870 lines. 16 §1 normative clauses (CLI lifecycle, storage location, default confidential, restricted-forces-encryption, lifecycle invariants, reject-after-end, read with optional decrypt, list CLI, retention, session.purged row, ACL on sessions/, §18 amendment, session_id on memory writes, PII redaction flag, export stretch, correlation stretch). 9 §2 rationale paragraphs. Full Python lifecycle module + AGENTS.md §18 amendment text + schema fragments in §3. 25 ACs all `traces_to: §1 #N`. 13 + 2 = 15 pytest tests. 18 failure modes. 8 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Sessions in main chain vs separate
First sketch inlined session.turn rows into the main audit chain. Risk: cardinality explosion + privacy mixing. Resolved: §1 #2 separate `sessions/<date>/<id>.binlog.zst` storage; summary rows on main chain for FR-MEMORY-115 discovery; §2 rationale paragraph "Why separate binlog instead of inlining."

### ISS-002 — Default classification
Stephen explicitly chose `confidential` per the 2026-05-19 question response. Resolved: §1 #3 + DEC-251; `public` and `internal` rejected to prevent misuse.

### ISS-003 — Retention purge semantics
Retention without purge = privacy leak; retention WITH eager purge = lost diagnostic data. Resolved: §1 #9 + §1 #10 + DEC-252 — body purged, summary rows + `session.purged` audit row remain; mirrors §3.6 `delete(purge)` for memory files.

### ISS-004 — Active-session pointer
How does the writer know which session id to attach to memory writes? Resolved: §1 #13 + `.active` pointer file + AC #11 + AC #12 cover.

### ISS-005 — Cross-day sessions
Date-partitioned storage with sessions spanning midnight could ambiguity which date directory the session lives in. Resolved: §1 #2 + §2 rationale "Why date-partitioned" + AC #25 — start date wins, sessions stay where they began.

### ISS-006 — Lifecycle invariant enforcement
Without walker-level enforcement, malformed session lifecycles (start without end; turn out of order; duplicate end) are detectable only at dream-runner time. Resolved: §1 #5 + #6 + DEC-253 walker invariants `session-lifecycle-well-formed`; AC #13 + #14.

## §3 — Rubric scorecard

| Rule | Pass | Notes |
|---|---|---|
| FM-001..004 | ✓ | Frontmatter clean |
| FM-101..111 | ✓ | Title 215 chars (project convention) |
| SEC-001..009 | ✓ | All required sections present |
| COND-001..004 | n/a | client_visible: false |
| QA-001..009 | ✓ | Alternatives discussed; scope clear; non-vanity metrics |
| SAFE-001..004 | n/a | No untrusted_content blocks (transcripts may contain user content but the FR doesn't quote any inline) |
| TRACE-001 | ✓ | Coverage: §1 #1→AC1/AC9/AC10, #2→AC24/AC25, #3→AC2/AC4, #4→AC3/AC15/AC16, #5→AC5/AC6/AC7/AC8/AC13/AC14, #6→AC5/AC6/AC7, #7→AC15/AC16, #8→AC17, #9→AC18/AC19, #10→AC18/AC20, #11→AC21, #12→AC22, #13→AC11/AC12, #14→AC23, #15/#16 SHOULD (deferred) |
| TRACE-002 | ✓ | Every AC named in §5 test fns |
| TRACE-003 | ✓ | Test paths in `frontmatter.new_files` |
| TRACE-004 | n/a | status: draft |
| TRACE-005 | n/a | Stretch SHOULDs in §1 #15/#16 deferred |

### Score derivation
- Pre-revision: 8.5/10 (storage layout + active pointer ambiguity)
- Post-expansion: 9.5/10 (DEC-250..253 + §2 rationale paragraphs + §3 schema + amendment text)
- Post-revision: **10/10** — AC #11 + #12 pin session_id propagation; AC #18 + #20 pin purge semantics

## §4 — Resolution

All 6 mechanical concerns addressed. **Score = 10/10 with explicit protocol-gate.**

Implementation gated by `APPROVE protocol change P22 §18`. Same pattern as FR-MEMORY-115/117/118.

---

*End of FR-MEMORY-119 audit.*
