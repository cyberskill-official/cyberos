# `*.audit.md` report format

> Audit reports are sibling files of the PRDs they audit. Format mirrors fr-audit's report structure with PRD-specific field renames.

## Frontmatter

```yaml
---
audit_template_version: "1.0"
audited_prd_path: <relative path to the PRD>
audited_prd_sha256: sha256:<64 hex>
rubric_version: prd_rubric@1.0
overall_status: pass | needs_human | fail | stale
exit_code: 0 | 1 | 2 | 3
first_audit_at: <ISO 8601>
last_audit_at: <ISO 8601>
audit_iteration_count: <integer ≥ 1>
counts:
  passed: <int>
  failed: <int>
  needs_human: <int>
  warnings: <int>
upstream_skill: cuo/cpo/prd-author | null  # set when chained
upstream_manifest: <path | null>
genie_action_log_row_id: <evt_…>
---
```

## Body

```markdown
# Audit report for <PRD title>

## Summary

`overall_status: <verdict>` after <N> iterations.
- Passed: <count>
- Failed: <count>
- Needs human: <count>
- Warnings: <count>

## Failed rules

### ISS-001 — <rule_id> at line <N>
**Severity:** <error|warning>
**What was flagged:** "<exact substring from PRD>"
**Why:** <one-sentence rule statement>
**Suggested fix:** <auto-fix description OR "needs human input">

[repeated per failed rule]

## Needs-human issues

### ISS-N — <rule_id> at line <N>
**HITL category:** <category from prd-audit/SKILL.md hitl_categories>
**HITL question:** "<question for the human>"
**Resolution:** <null until answered; populated on resume>

## Passed rules

<list of rule_ids that passed; one per line>

## Skipped rules (with reason)

<list of rule_ids skipped; e.g., COND-002 skipped because eu_ai_act_risk_class != high>

## Confidence summary

<table of LLM-judgement rules' confidence values; mechanical rules collapsed to "confidence ≥ 0.95">
```

## Determinism contract

The body of an audit report is byte-stable for a given `audited_prd_sha256` + `rubric_version`, modulo:

1. `last_audit_at` (always advances on re-run).
2. The `confidence` floats for LLM-judgement rules (per RUBRIC §15.10's band-reproducible contract).

Anything else differing between two runs against identical input is INV-001 breach.

## Citations

- Pattern source — `cuo/cpo/fr-audit/REPORT_FORMAT.md`.
- INV-001 — `cuo/cpo/prd-audit/INVARIANTS.md` §"INV-001 — verdict reproducibility on mechanical rules".
