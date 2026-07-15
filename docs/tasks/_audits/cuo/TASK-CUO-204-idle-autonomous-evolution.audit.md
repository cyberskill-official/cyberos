---
task_id: TASK-CUO-204
audited: 2026-06-22
verdict: PASS
score: 9.5/10
template: task@1
rubric: task-audit RUBRIC.md (FM / SEC / QA / COND / SAFE)
auditor: "@stephen (assisted)"
---

Idle-time autonomous evolution - the "dream loop" that runs the TASK-CUO-201/202/203 propose cycle on idle, applies only changes that pass the AWH gate AND sit inside an explicit evolution envelope AND classify low-risk, halts any security-invariant change for a human, keeps every apply reversible, lands on a review branch, and carries a kill switch.

Frontmatter (FM-001..111, FM-004): all required keys present and well-typed; title 64 chars (FM-101 ok); template literal task@1; priority p2; eu_ai_act_risk_class high; ai_authorship assisted; client_visible false. Required sections (SEC-001..008): Summary, Problem, Proposed Solution (with a Section 1 normative BCP-14 block of 10 MUST / MUST NOT clauses), Alternatives Considered (3 distinct, QA-005 ok), Success Metrics (one primary + one guardrail, each with definition / baseline / target / measurement method / source, QA-004 + QA-007 ok), Scope with Out of scope (5 items, QA-006 ok), Dependencies (TASK-CUO-200..203 plus the gate, golden sets, and scheduler, QA-008 ok). Conditional (COND-003, triggered by eu_ai_act_risk_class high): AI Risk Assessment present with Data Sources, Human Oversight, Failure Modes in that order - and the oversight section is load-bearing here, since the safety envelope is the whole point. Conditional (COND-004): AI Authorship Disclosure present with the three required bullets. No untrusted-content blocks, so SAFE rules not triggered. Heading hierarchy well-formed (SEC-009).

The safety envelope (allowlist plus denylist), the denylist-halt-for-human clause, reversibility with auto-revert, the bounded-run limits, and the kill switch together answer the high-risk classification: the loop is autonomous inside a fence and provably cannot auto-modify a security invariant. The escalation_on_breach for a high-risk task routes to cuo-clo per the contract; this audit flags that Stephen's explicit sign-off on the envelope and denylist is the gate before any code lands.

Open item (the -0.5): the envelope's denylist must be enumerated concretely in config/dream.yaml against the real protected paths before implementation, not left to the prose list here. That is an implementation-time precondition, called out in clause 4.

Verdict: PASS, with the standing condition that the envelope is owner-approved before build.

*End of TASK-CUO-204 audit.*
