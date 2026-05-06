# Human-readable batch summary template

```
🔍 PRD audit complete — `cuo/cpo/prd-audit` v{skill_version}

📊 PRDs audited: {count}
   ✅ pass: {pass_count}
   🚧 needs_human: {hitl_count}
   ❌ fail: {fail_count}
   ⏰ stale: {stale_count}

{per-PRD block}
─────────────────────────────────────
📄 {prd_path}: {status_emoji} {status}
   Iterations: {iteration_count}
   Errors: {err_count}; warnings: {warn_count}; needs-human: {hitl_count}
   Audit report: {audit_path}
{if HITL: HITL questions:
   {for each: - <category>: <question>}}
─────────────────────────────────────

📊 Trace: {trace_id}  |  Rubric: prd_rubric@{rubric_version}

{if any_hitl}
⚠️  HITL pause — answer questions in {hitl_request_path} and re-invoke to resume.
{end}

{if any_fail}
❌ Failures need attention. Most common rule_ids: {top_rule_ids}.
{end}
```

## Status emoji

| Status | Emoji |
| --- | --- |
| pass | ✅ |
| needs_human | 🚧 |
| fail | ❌ |
| stale | ⏰ |

## Localisation

`manifest.languages[0]`; ASCII visual markers.

## Citations

- Pattern source — `cuo/cpo/fr-audit/HUMAN_SUMMARY.md`.
