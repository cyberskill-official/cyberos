# `observability-injection` audit-report format

Every `observability-injection-audit` invocation writes a sibling `.audit.md` per audited artefact.

## Frontmatter

```yaml
---
audit_template_version: "observability_injection_rubric@1.0"
audited_file:           "<path>"
audited_file_sha256:    "<64-hex>"
rubric_version:         "observability_injection_rubric@1.0"
skill_id:               "observability-injection-audit"
overall_status:         "pass | needs_human | fail | exhausted | no_progress"
iterations:             N
issue_counts: {total: N, open: N, resolved: N}
score:                  "N/10"
---
```

## Body

`## §1 Verdict summary`, `## §2 Findings` (one `### <rule_id>` block each: quote, why it fails the gate, fix), `## §3 Resolution`.
Findings cite rule IDs from RUBRIC.md - never paraphrased prose (that is the point of observability_injection_rubric@1.0).
