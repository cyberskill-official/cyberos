# `backlog-state-update` audit-report format

Every `backlog-state-update-audit` invocation writes a sibling `.audit.md` per audited artefact.

## Frontmatter

```yaml
---
audit_template_version: "backlog_state_update_rubric@2.0"
audited_file:           "<path>"
audited_file_sha256:    "<64-hex>"
rubric_version:         "backlog_state_update_rubric@2.0"
skill_id:               "backlog-state-update-audit"
overall_status:         "pass | needs_human | fail | exhausted | no_progress"
iterations:             N
issue_counts: {total: N, open: N, resolved: N}
score:                  "N/10"
---
```

## Body

`## §1 Verdict summary`, `## §2 Findings` (one `### <rule_id>` block each: quote, why it fails the gate, fix), `## §3 Resolution`.
Findings cite rule IDs from RUBRIC.md - never paraphrased prose (that is the point of backlog_state_update_rubric@2.0).
