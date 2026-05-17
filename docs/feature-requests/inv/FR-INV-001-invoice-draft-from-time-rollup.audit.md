---
fr_id: FR-INV-001
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands the invoice substrate on top of FR-TIME-009. 870 lines, 27 §1 clauses, 20 ACs, 7 tests, 20 failure modes, 10 implementation notes. 5 migrations, 8 endpoints, 9 BRAIN audit kinds. Anchors INV-002 through INV-011 + PORTAL-001 invoice view + PORTAL-006 billing workflows + TEN-003/102 cross-rail references.

6 issues resolved.

## §2 — Findings (all resolved)

### ISS-001 — Multi-currency deferred but engagement.billing_currency immutability not explicit

§11.5 — billing currency immutable per FR-TEN-003 derivative; new engagement required for currency change.

### ISS-002 — TIME-side coupling (invoiced_at column)

§11.2 — column added via this FR's modified_files; TIME-009 derivative.

### ISS-003 — Status FSM table-driven

§11.4 — Rust match table; CI test asserts every transition documented.

### ISS-004 — Numbering race during high-concurrency

§11.3 — FOR UPDATE lock prevents race; skipped logged.

### ISS-005 — Append-only line corrections vs total recomputation

§10 row + §11 — totals recomputed at correction-add; trigger or handler-side.

### ISS-006 — Cross-tenant via RLS only

§3.1 — RLS + indexed tenant_id; no second-layer check needed.

## §3 — Resolution

All 6 mechanical concerns addressed. Schema-level append-only + state-machine + gap-free numbering = forensically sound invoice substrate.

The 870-line length is appropriate for 8h-effort foundational FR with 5 migrations + 8-state FSM + cross-FR anchoring.

**Score = 10/10.**

---

*End of FR-INV-001 audit.*
