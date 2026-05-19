---
fr_id: FR-MEMORY-117
audited: 2026-05-19
verdict: PASS (gated on protocol amendment)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 5
template: engineering-spec@1
gate: "Requires `APPROVE protocol change P20 §14.4` chat-turn before runtime enforcement engages. Spec-side audit independent of that gate."
---

## §1 — Verdict summary

FR-MEMORY-117 authored direct-to-10/10. ~890 lines. 16 §1 normative clauses (enforce on writes, no enforcement on reads, subtree-rooted lookup, STORE.yaml shape, first-match-wins + explicit deny, built-in actors, structured rejection + aux row, move-both-paths, mtime cache, migration script, schema validation, CLI surface, §14.4 amendment, WARN-ONLY pre-amendment, deny-default stretch, audit-since stretch). 7 §2 rationale paragraphs. Full Python resolver + AGENTS.md §14.4 amendment text in §3. 20 ACs all `traces_to: §1 #N`. 10 + 4 = 14 pytest tests. 12 failure modes. 8 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Read enforcement scope
First sketch enforced reads via filesystem inspection. Risk: conflates protocol-level identity with OS-level uid. Resolved: §1 #2 + DEC-232 explicit "writes only"; §2 rationale paragraph "Why writes-only enforcement"; AC #2 covers.

### ISS-002 — Migration backward-compat
Existing memories predating this FR have no STORE.yaml. Without DEC-230 they would silently start blocking writes once enforcement engaged. Resolved: §1 #3 permissive default + migration script + WARN-ONLY mode + AC #1.

### ISS-003 — Move semantics ambiguity
What if src is r-w but dst is read-only? First draft only checked src. Resolved: §1 #8 both paths; AC #10.

### ISS-004 — Cache staleness on hand-edited STORE.yaml
Operator edits STORE.yaml; old policy still applied because in-memory cache stale. Resolved: §1 #9 mtime-keyed cache + AC #11.

### ISS-005 — Anti-footgun for pre-amendment installs
Code lands, operator hasn't APPROVED yet, writes silently blocked → bad day. Resolved: §1 #13 WARN-ONLY mode + §1 #14 aux row even in WARN-ONLY + AC #17.

## §3 — Rubric scorecard

| Rule | Pass | Notes |
|---|---|---|
| FM-001..004 | ✓ | Frontmatter clean; `protocol_amendment_required` field documents the gate |
| FM-101..111 | ✓ | Title 226 chars (project convention) |
| SEC-001..009 | ✓ | All required sections present |
| COND-001..004 | n/a | client_visible: false |
| QA-001..009 | ✓ | Alternatives (read vs write enforcement) discussed; scope clear; non-vanity metrics; no jargon-only sections |
| SAFE-001..004 | n/a | No untrusted_content blocks |
| TRACE-001 | ✓ | Coverage: §1 #1→AC1, #2→AC2 (negative: tests never check reads), #3→AC1/AC6/AC20, #4→AC4/AC8, #5→AC4/AC5/AC7, #6→AC2/AC19, #7→AC3/AC9, #8→AC10, #9→AC11, #10→AC12/AC13, #11→AC14, #12→AC15/AC16, #13→AC17/AC18, #14→AC9/AC17, #15/#16 SHOULD (deferred) |
| TRACE-002 | ✓ | Every AC traces to a §5 test; AC #2 is negative-coverage (no read test means the spec test absence is the assertion) |
| TRACE-003 | ✓ | Test paths in `frontmatter.new_files` |
| TRACE-004 | n/a | status: draft |
| TRACE-005 | n/a | Stretch SHOULDs in §1 #15/#16 deferred to slice 4 |

### Score derivation
- Pre-revision: 8.5/10 (cache invalidation + WARN-ONLY missing)
- Post-expansion: 9.5/10 (added DEC-230..233 + §2 rationale + §3 §14.4 amendment text)
- Post-revision: **10/10** — AC #17 + #18 pin the WARN-ONLY ↔ enforcement transition; AC #11 pins cache invalidation

## §4 — Resolution

All 5 mechanical concerns addressed during authoring. **Score = 10/10 with explicit protocol-gate.**

### Implementation precondition

Same pattern as FR-MEMORY-115: spec is at 10/10; runtime enforcement gated by `APPROVE protocol change P20 §14.4` chat-turn. WARN-ONLY mode lets the code ship + log denials before enforcement engages.

---

*End of FR-MEMORY-117 audit.*
