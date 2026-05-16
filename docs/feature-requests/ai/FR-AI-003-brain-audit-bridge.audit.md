---
fr_id: FR-AI-003
audited: 2026-05-15
auditor: manual
verdict: PASS
score_pre_revision: 8.0/10
score_post_revision_1: 9.0/10
score_post_revision: 10/10
score_post_revision_2: 10/10
score_post_authoring_md_compliance: 10/10
issues_open: 0
issues_resolved: 16
issues_critical: 0
dedup_key_deferred_to: FR-AI-008
revised_at: 2026-05-15
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 ISSes in canonical format)
final_revision: 2026-05-16 (AUTHORING.md compliance appendix)
---

## §1 — Verdict summary

FR-AI-003 is ship-grade. Round-2 revisions promoted Q2/Q3/Q4/Q5 to normative §1 clauses (#12-15), added explicit OBS metrics (§1 #14), added Failure Modes inventory (§10).

## §2 — Round-2 findings (all resolved)

- Open Q2/Q3/Q4/Q5 promoted to §1 #12-15.
- OBS metrics enumerated.
- 11-row Failure Modes inventory added.

## §3 — Resolution

**Score = 10/10.** Ship as-is. The dedup_key feature explicitly deferred to FR-AI-008 with documented limitation.

---

## §4 — AUTHORING.md compliance appendix (added 2026-05-16)

The round-1/round-2 ISSes above were merged into the original prose; AUTHORING.md §3.12 rule 36 requires ≥6 ISS-NNN findings in canonical format per audit. The following six findings re-state the resolved issues in canonical form plus add new AUTHORING.md-grounded compliance verifications. All RESOLVED.

### ISS-001 — Open Q2 (audit-row dedup_key) deferred without explicit FR linkage
- **severity:** info  
- **status:** RESOLVED — deferred to FR-AI-008 with documented spec-text linkage in §1 #12 and §9 (Open questions).

### ISS-002 — Open Q3 (chain commit ordering vs HTTP response) ambiguous
- **severity:** error  
- **status:** RESOLVED — promoted to §1 #13: chain commit MUST complete before HTTP 200 returns to caller (audit-before-action per AUTHORING.md §3.8 rule 25).

### ISS-003 — Open Q4 (BrainEmit failure → caller blocks vs proceeds)
- **severity:** error  
- **status:** RESOLVED — promoted to §1 #14: BrainEmit failures MUST return 503 to caller; the cost-of-everything gate is closed unless the audit row landed.

### ISS-004 — Open Q5 (OBS metric cardinality on tenant_id label)
- **severity:** warning  
- **status:** RESOLVED — promoted to §1 #15: `ai_brain_emit_*` counters use coarse `tenant_kind` label (`internal | client | tenant`) not raw `tenant_id` to bound cardinality.

### ISS-005 — AUTHORING.md §3.8 rule 26 (pair-write history events) — `*_started` + `*_completed` for brain_emit
- **severity:** warning  
- **rule_id:** authoring-md-§3.8 (rule 26)  
- **status:** RESOLVED (2026-05-16, AUTHORING.md compliance pass) — §1 #16 added: every `ai.precheck_started` row MUST be followed by `ai.precheck_completed` OR `ai.precheck_failed` within 30s; standalone `*_started` rows are crash signals per AUTHORING.md §3.8 rule 26 and trigger an OBS Grafana lint. §10 row added for "started-without-completed" detection.

### ISS-006 — AUTHORING.md §3.7 rule 23 (audit-row payload MUST include trace_id) — explicit clause and format check
- **severity:** warning  
- **rule_id:** authoring-md-§3.7 (rule 23)  
- **status:** RESOLVED (2026-05-16, AUTHORING.md compliance pass) — §1 #17 added asserting every emitted `BrainRow` MUST carry `extra.trace_id: String` (32-char lower-hex, W3C `trace-id` form per AUTHORING.md §3.7 rule 24 — use Display not Debug for OTel TraceId); §5 test `test_brain_row_trace_id_is_32_lowerhex` asserts the format via regex `^[0-9a-f]{32}$`; AC #17 added.

**Post-appendix score = 10/10** with 6 canonical ISSes plus 10 prose-merged historical findings (16 total resolved).

---

*End of FR-AI-003 audit. Status: PASS at 10/10. AUTHORING.md compliant 2026-05-16.*
