# `ip-strategy` audit-report format

Every `ip-strategy-audit` invocation writes a sibling `ip-strategy.audit.md` per audited artefact. The report format below is normative.

## Frontmatter

```yaml
---
audit_template_version: "ip-strategy_rubric@1.0"
audited_file:           "./ip-strategys/IP_STRATEGY-001-foo.md"
audited_file_sha256:    "<64-hex>"
rubric_version:         "ip-strategy_rubric@1.0"
skill_id:               "ip-strategy-audit"
skill_version:          "1.0.0"
last_audit_at:          "2026-05-17T14:32:00Z"
overall_status:         "pass | needs_human | fail | exhausted | no_progress"
iterations:             3
issue_counts:
  total:                 N
  open:                  N
  needs_human:           N
  fixed:                 N
  wontfix:               N
trace_id:               "<uuid>"
caller_persona:         "cuo-cpo"
---
```

## Body — issue blocks

For every issue (open, needs_human, fixed, wontfix), the report emits one fenced block:

```
ISSUE
id:              <ISS-001>  // monotonic per audit report
rule_id:         FM-101
status:          open | needs_human | fixed | wontfix
severity:        error | warning | info
category:        <one of the skill's hitl_categories — required when status == needs_human>
location:        line 12, col 5    // file location of the offending substring; omit for whole-file issues
evidence:        "title is missing"   // the offending substring or fact
description:     "Frontmatter field `title` is required per FM-101 but was not found."
suggestion:      "Add a `title:` field to the frontmatter. Suggested: `<inferred title from filename>`."
auto_fix_applied: true | false
diff_hunk:       |
  ---
  + title: Inferred from filename
  ...
  ---
resolution:      null | "<operator's reply text>"
resolved_at:     null | "2026-05-17T15:00:00Z"
opened_at:       "2026-05-17T14:32:00Z"
updated_at:      "2026-05-17T14:35:00Z"
```

## Body — summary

After all issue blocks, the report ends with:

```
SUMMARY
verdict:         pass | needs_human | fail | exhausted | no_progress
issues_total:    N
issues_open:     N
issues_human:    N
issues_fixed:    N
iterations:      3
next_action:     "ship | resume_after_hitl | re-author | manual_review"
```

## Re-entrancy

When the audit re-runs on the same artefact:

- If `audited_file_sha256` unchanged: existing issue blocks are preserved; only `last_audit_at` and `updated_at` per issue are refreshed.
- If `audited_file_sha256` differs: every `open` and `needs_human` issue resets to `open` and re-evaluates. `fixed` and `wontfix` blocks remain for diff context.

## Byte-stability

Two runs against the same artefact + same rubric version MUST produce byte-identical reports modulo:

- `last_audit_at` in the frontmatter
- `updated_at` per ISSUE block
- the order of `fixed` ISSUE blocks (sorted by `opened_at` ascending)

If determinism breaks, the `deterministic_drift` self-audit invariant fires and the skill pauses for operator review.

## Cross-references

- `RUBRIC.md` (sibling file) — the rules that populate the `rule_id` field.
- `cyberos/skill/docs/AUDIT_LOOP.md` — the algorithm that produces the report.
- `INVARIANTS.md` (sibling file) — the byte-stability invariant.
