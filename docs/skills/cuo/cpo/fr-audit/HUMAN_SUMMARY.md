# `fr-audit` human-summary template

> Rendered to chat after each audit batch when the skill runs in **standalone** mode. Chained-mode runs (e.g. after `fr-author`) skip this — the supervisor folds the verdict into the upstream skill's summary.

## Template

````
🔍 **Audit complete** — {N} FR(s) audited against `{rubric_version}`

**Verdicts:**
{for v in verdicts:}
  - {pass_icon} **{fr.id}** — {verdict_label}{rule_count_suffix}
{end-for}

{if any verdict is "fail":}
❌ **Failures** ({count}):

{for fail in fails:}
  - **{fr.id}** at line {line}:
    `{rule_id}` — {one_line_violation_summary}
    💡 {one_line_fix_suggestion}
{end-for}
{end-if}

{if any verdict is "needs_human":}
⏸️ **Needs human review** ({count}):

{render: references/HITL_PROTOCOL.md HITL_BATCH_REQUEST format}
{end-if}

{if all verdicts are "pass":}
✅ All {N} FR(s) pass `{rubric_version}`. Reports are at:
{for fr in frs:}
  - `{fr.audit_path}`
{end-for}
{end-if}

📋 **What I changed in BRAIN**:

{audit_block_per_AGENTS_md_§14}

📊 **Trace**: `{trace_id}`  ·  invocation #{invocation_counter}
🔗 **Next**: `{next_skill_recommendation or "—"}`

> Type `fix FR-NNN` to dispatch a fix-iteration with `cuo/cpo/fr-author`
> on a specific failing FR.
> Type `re-audit FR-NNN` to re-run after edits.
> Type `dismiss FR-NNN` to mark a failure as wontfix (logged with reason).
````

## Tone rules

Same as `fr-author/HUMAN_SUMMARY.md` §"Tone rules", plus:

- **Verdicts are first.** The user wants pass/fail/needs_human counts before per-rule details.
- **One-line violation summary, not a full rubric paste.** Link to the full report file for detail.
- **Fix suggestion per failure when confident.** Mark uncertain suggestions with "(LOW CONFIDENCE)" — the user can still take the hint without being misled.

## Pass icons (fixed vocabulary)

```
✅ pass
⚠️  pass-with-warnings
❌ fail
⏸️  needs_human
🚫 stale (audited file SHA changed since the audit ran)
```

No other icons. No skin-tone variants. No emoji combinations.

## Chained-mode equivalent

When chained from `fr-author`, the supervisor renders one line per audited FR back into the upstream skill's summary block:

```
fr-author wrote FR-007.md (artefact_write evt_…)
  └─ fr-audit: ⚠️ pass-with-warnings (2 warnings, 0 errors) → FR-007.audit.md
```

Full audit details remain in the audit report file; the supervisor doesn't re-paste them inline.
