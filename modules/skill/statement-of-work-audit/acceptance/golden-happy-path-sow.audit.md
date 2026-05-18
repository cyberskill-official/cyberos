---
audit_template_version: "sow_rubric@1.0"
audited_file:           "../statement-of-work-author/acceptance/golden-happy-path-sow.md"
audited_file_sha256:    "<computed-at-runtime>"
rubric_version:         "sow_rubric@1.0"
skill_id:               "statement-of-work-audit"
skill_version:          "1.0.0"
last_audit_at:          "2026-05-17T00:00:00+07:00"
overall_status:         "pass"
iterations:             2
issue_counts:
  total:                 2
  open:                  0
  needs_human:           0
  fixed:                 1
  wontfix:               1
trace_id:               "00000000-0000-0000-0000-000000000002"
caller_persona:         "cuo-cpo"
---

# Audit report — `Acme Corporation — Customer Portal Modernisation Phase 1`

Sibling artefact: `../statement-of-work-author/acceptance/golden-happy-path-sow.md` (sow@1).

This is the **golden happy-path fixture** for the statement-of-work-author + statement-of-work-audit pair.
Demonstrates the audit-fix loop converging on PASS in 2 iterations. The first
iteration found a minor frontmatter-typo (auto-fixed). The second iteration
flagged a borderline COND-007 trigger (operator marked wontfix with rationale).

## ISSUE blocks

```
ISSUE
id:               ISS-001
rule_id:          FM-103
status:           fixed
severity:         error
location:         frontmatter:client_legal_entity
evidence:         "Acme Corporation Inc., Delaware"
description:      Frontmatter field `client_legal_entity` must include both the full legal name AND jurisdiction. Initial draft had jurisdiction omitted.
suggestion:       Append the jurisdiction (e.g. ", Delaware USA").
auto_fix_applied: true
diff_hunk:        |
  - client_legal_entity: Acme Corporation Inc., Delaware
  + client_legal_entity: Acme Corporation Inc., Delaware USA
resolution:       null
resolved_at:      null
opened_at:        "2026-05-17T00:00:00+07:00"
updated_at:       "2026-05-17T00:00:00+07:00"
```

```
ISSUE
id:               ISS-002
rule_id:          COND-007
status:           wontfix
severity:         error
location:         §10. IP and Confidentiality
evidence:         "Three EU-based customer firms; data hosted in us-east per their DPAs"
description:      COND-007 normally requires a `### GDPR Addendum` subsection when EU residency is in scope. The brief reports three EU-based customer firms BUT all data is hosted in us-east under those customers' own pre-existing DPAs. Operator judged this as an inline DPA reference (covered in §10) plus customer-side data-residency control, not a CyberSkill-side GDPR-addendum trigger.
suggestion:       If the EU customer set grows or data residency moves to EU, re-author with a structured `### GDPR Addendum` subsection.
auto_fix_applied: false
diff_hunk:        ""
resolution:       "Operator judgement: COND-007 does not strictly trigger because data residency stays in us-east. Inline DPA reference is sufficient. wontfix."
resolved_at:      "2026-05-17T00:00:00+07:00"
opened_at:        "2026-05-17T00:00:00+07:00"
updated_at:       "2026-05-17T00:00:00+07:00"
```

```
SUMMARY
verdict:          pass
issues_total:     2
issues_open:      0
issues_human:     0
issues_fixed:     1
iterations:       2
next_action:      ship
```

## Smoke-test notes

This fixture proves end-to-end:

1. `statement-of-work-author` correctly emits all 12 SOW skeleton sections per Software Development Process.md §4.9.
2. The 12-section ordering is preserved (SEC-001..012 pass).
3. Conditional sections wire correctly: `fixed_price` → `### Fixed-Price Terms` (COND-001 pass); personal data → `Sub-processor list` + DPA reference (COND-006 pass).
4. The audit-fix loop converges: one auto-fix in iteration 1, one operator-judgement wontfix in iteration 2, PASS in iteration 2.
5. RACI in §6 covers all 9 roles (CS / EM / PO / TL / AR / DEV / QA / DO / SEC) per `QA-RACI-001`.
6. AI-use disclosure paragraph present per SDP §5 (`QA-AI-001` pass).
7. The audit report is byte-stable for the same audited_file_sha256 + rubric version (per INV-006 `deterministic_drift` invariant).

Use this fixture as the byte-equality reference for the parity harness at `skill/tests/`.
