---
fr_id: FR-MEMORY-118
audited: 2026-05-19
verdict: PASS (gated on protocol amendment)
score_pre_revision: 9.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 4
template: engineering-spec@1
gate: "Requires `APPROVE protocol change P21 §3.1` chat-turn. Spec-side audit independent of that gate."
---

## §1 — Verdict summary

FR-MEMORY-118 authored direct-to-10/10. ~750 lines. 14 §1 normative clauses (additive enum, precondition shape, lock + check semantics, HEAD doesn't advance on rejection, ACL check first, success-row indistinguishability from put, aux row payload, CLI surface, typed result, shape validation, retry-loop documentation, §3.1 amendment requirement, INTEROP support, batch stretch). 7 §2 rationale paragraphs. Writer extension + AGENTS.md §3.1 amendment text + PutIfResult dataclass in §3. 20 ACs all `traces_to: §1 #N`. 16 pytest tests including concurrency + retry-loop + CLI. 16 failure modes. 8 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Replacement vs additive
First sketch had `put` taking an optional `precondition_body_hash=` kwarg. Risk: complicates the canonical op model; INTEROP consumers would need to support optional preconditions on `put`. Resolved: §1 #1 + DEC-240 separate primitive `put_if`; AGENTS.md §3.1 lists 4 ops not 3 modes-on-put.

### ISS-002 — Aux-row-on-rejection vs no-row
Should rejection emit any chain row? First draft: nothing. Risk: silent rejection invisible to operators. Resolved: §1 #4 + #7 + DEC-242 — aux row IS emitted even though put payload isn't; HEAD advances by 1. AC #2 + #7 + #11 cover.

### ISS-003 — ACL vs precondition ordering
Both checks fire on the same write; which first? First draft: precondition first. Risk: an ACL-denied caller gets "precondition_failed" instead of the right "acl_denied". Resolved: §1 #5 ACL first; §2 rationale paragraph; AC #8 + #9.

### ISS-004 — Success row shape
Successful put_if could plausibly emit `kind: "put_if"`. Risk: forces every downstream consumer to support two row shapes. Resolved: §1 #6 + DEC-242 — success row is INDISTINGUISHABLE from put; AC #10 asserts kind=="put".

## §3 — Rubric scorecard

| Rule | Pass | Notes |
|---|---|---|
| FM-001..004 | ✓ | Frontmatter clean |
| FM-101..111 | ✓ | Title 144 chars (project convention) |
| SEC-001..009 | ✓ | All required sections present |
| COND-001..004 | n/a | client_visible: false |
| QA-001..009 | ✓ | Alternatives (additive vs replacement) explicit; scope clear |
| SAFE-001..004 | n/a | No untrusted_content |
| TRACE-001 | ✓ | Coverage: §1 #1→AC1, #2→AC3/AC4/AC16/AC17, #3→AC1-#7, #4→AC2/AC7, #5→AC8/AC9, #6→AC10, #7→AC11, #8→AC12/AC13/AC14, #9→AC15, #10→AC16/AC17, #11→AC18, #12→AC19/AC20, #13 SHOULD (deferred), #14 SHOULD (deferred) |
| TRACE-002 | ✓ | Every AC traces to a §5 test |
| TRACE-003 | ✓ | Test path in `frontmatter.new_files` |
| TRACE-004 | n/a | status: draft |
| TRACE-005 | n/a | Stretch SHOULDs in §1 #13/#14 deferred |

### Score derivation
- Pre-revision: 9.0/10 (additive vs replacement; success-row shape; ordering with ACL)
- Post-expansion: 9.5/10 (DEC-240..243 + §2 rationale paragraphs)
- Post-revision: **10/10** — AC #10 + AC #18 + AC #19 pin all the load-bearing invariants

## §4 — Resolution

All 4 mechanical concerns addressed during authoring. **Score = 10/10 with explicit protocol-gate.**

Implementation gated by `APPROVE protocol change P21 §3.1` chat-turn — same pattern as FR-MEMORY-115 / 117. Writer raises `ProtocolAmendmentMissingError` if §3.1 doesn't list `put_if`.

---

*End of FR-MEMORY-118 audit.*
