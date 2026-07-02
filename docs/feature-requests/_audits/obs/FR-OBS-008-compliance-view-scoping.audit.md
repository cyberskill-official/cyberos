---
fr_id: FR-OBS-008
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
---

## §1 — Verdict summary

FR-OBS-008 expanded from 159 lines to ~830. Added 6 §1 clauses (#9 summary block; #10 audit-row of view access; #12 PII-placeholder defence-in-depth; #13 auditor JWT mechanism; #14 metrics; expanded #11 with full per-view content). 7 §2 rationale paragraphs. Full Rust types + 4 view modules + chain_proof + PDF/JSON exporters in §3. 17 ACs. 7 full Rust test bodies. 17 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Per-view content underspecified
First-pass §1 had high-level descriptions only. Resolved: §1 #11 enumerates exact row kinds per view; CIS DSS scope deferred.

### ISS-002 — Cross-tenant via query param not blocked
First-pass §3 example used `?tenant_id=org:cyberskill` query param. An auditor could supply different tenant_id. Resolved: §1 #3 + AC #6 + #7; tenant_id from JWT only; query param rejected with 403.

### ISS-003 — Auditor JWT mechanism unspecified (separate role? TTL?)
First-pass §1 #2 mentioned `role: external_auditor` but no issuance path. Resolved: §1 #13 + per-engagement JWT (30-day TTL) + `cyberos-auth issue-auditor-token` future command.

### ISS-004 — Audit-row of compliance view access missing
Auditor's own access should be auditable. Resolved: §1 #10 + canonical::compliance_view_accessed builder; AC #14 + §5 test.

### ISS-005 — PII-placeholder defence-in-depth missing
First-pass disallowed_tools said "redact PII from compliance view export" but no enforcement. Resolved: §1 #12 + regex scan at response time + sev-1 alarm; AC #15 + §5 test asserts 500 on raw PII.

### ISS-006 — Chain-proof verification mechanism not specified
First-pass §1 #5 said "Ed25519 signature" without canonicalisation rule. Auditor would need to know the exact serialisation. Resolved: §3 + chain_proof.rs canonical(rows + summary); §5 test asserts independent verify; §11 documents.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-OBS-008 audit.*
