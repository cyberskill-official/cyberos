# `hardening-record` audit-report format

Every `harden-record-audit` invocation writes a sibling `<record>.audit.md`.

## Frontmatter

```yaml
---
audit_template_version: "hardening_record_rubric@1.0"
audited_file:           "<path>"
audited_file_sha256:    "<64-hex>"
rubric_version:         "hardening_record_rubric@1.0"
skill_id:               "harden-record-audit"
skill_version:          "1.0.0"
last_audit_at:          "<ISO 8601>"
overall_status:         "pass | needs_human | fail | exhausted | no_progress"
score:                  10
iterations:             N
---
```

## Body

ISSUE blocks for each failing HRA-* rule, then a one-line verdict. Pass does not imply the defect is gone — only that the session record is honest.
