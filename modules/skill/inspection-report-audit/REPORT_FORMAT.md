# `inspection-report` audit-report format

Every `inspection-report-audit` invocation writes a sibling `<report>.audit.md`.

## Frontmatter

```yaml
---
audit_template_version: "inspection_report_rubric@1.0"
audited_file:           "<path>"
audited_file_sha256:    "<64-hex>"
rubric_version:         "inspection_report_rubric@1.0"
skill_id:               "inspection-report-audit"
skill_version:          "1.2.0"
last_audit_at:          "<ISO 8601>"
overall_status:         "pass | needs_human | fail | exhausted | no_progress"
score:                  10
machine_floor:          "pass | fail"
inspect_lint_version:   "<from tool>"
iterations:             N
---
```

## Body

1. Machine-floor result (command + exit + any INSL-* errors).
2. Scored IRA-001..IRA-009 with pass/fail and evidence pointers.
3. Verdict: pass → may hand to `/harden`; else route-back list of rule ids.
