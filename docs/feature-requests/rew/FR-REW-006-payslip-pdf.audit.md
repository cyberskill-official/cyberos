---
fr_id: FR-REW-006
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

REW byte-identical payslip PDF render via Tectonic + pinned fonts + SHA256 verification. 220 lines, 10 §1 clauses, 20 ACs, 3 tests, 10 failure modes, 5 notes. 6 issues resolved (pdf_render_status enum cardinality 5, Tectonic deterministic compilation, pinned fonts in template, SHA256 verification with mismatch → sev-1, UNIQUE(payslip_id) idempotency, content never in BRAIN chain (sha256 only)). **Score = 10/10.**

*End of FR-REW-006 audit.*
