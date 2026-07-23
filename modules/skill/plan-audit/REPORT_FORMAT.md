# `plan` audit-report format

Every `plan-audit` invocation writes a sibling `plan.audit.md` per audited artefact (next to `docs/plans/PLAN-<slug>-<YYYYMMDD>/plan.md`). The report format below is normative.

## Frontmatter

```yaml
---
audit_template_version: "plan_rubric@1.0"
audited_file:           "./docs/plans/PLAN-example-20260723/plan.md"
audited_file_sha256:    "<64-hex>"
rubric_version:         "plan_rubric@1.0"
skill_id:               "plan-audit"
skill_version:          "1.0.0"
last_audit_at:          "2026-07-23T14:32:00Z"
overall_status:         "pass | needs_human | fail | exhausted | no_progress"
iterations:             2
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
rule_id:         PLAN-OUT-001
status:          open | needs_human | fixed | wontfix
severity:        error | warning | info
category:        <one of the skill's hitl_categories — required when status == needs_human>
location:        "## 5. Scope"    // heading or line of the offending passage; omit for whole-file issues
evidence:        "### Out of scope is present but has zero bullets"
description:     "Scope has no boundary: PLAN-OUT-001 requires a non-empty Out of scope list."
suggestion:      "plan-author re-runs the §4 Boundary question; the auditor never invents the out-list."
auto_fix_applied: false
diff_hunk:       null      // the three load-bearing PLAN rules are never auto-fixed (AUDIT_LOOP.md)
resolution:      null | "<operator's reply text>"
resolved_at:     null | "2026-07-23T15:00:00Z"
opened_at:       "2026-07-23T14:32:00Z"
updated_at:      "2026-07-23T14:35:00Z"
```

## Body — summary

After all issue blocks, the report ends with:

```
SUMMARY
verdict:         pass | needs_human | fail | exhausted | no_progress
score:           N/10        // pass requires 10/10 — plan-audit refuses below
issues_total:    N
issues_open:     N
issues_human:    N
issues_fixed:    N
iterations:      2
next_action:     "hand plan.md to /create-tasks | resume_after_hitl | re-author | manual_review"
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

- `RUBRIC.md` (bundle root) — the rubric binding that populates the `rule_id` field.
- `../rubrics/plan_rubric.md` — the canonical rule tables (`plan_rubric@1.0`).
- `AUDIT_LOOP.md` (bundle root) — the algorithm that produces the report.
