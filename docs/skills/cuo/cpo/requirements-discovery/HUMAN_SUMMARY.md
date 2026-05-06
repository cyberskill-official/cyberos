# Human-readable batch summary template

> Rendered into chat after `requirements-discovery` completes a brief.

## Template

```
🎯 Project brief complete — `cuo/cpo/requirements-discovery` v{skill_version}

📄 Brief: {brief_path}
🏷️ Project kind: {project_kind}
🚦 Triage verdict: {triage_emoji} {triage_verdict}{ — {triage_reason} if revise/reject}

📊 Coverage:
   Questions answered: {answered}/20
   BRAIN reads: {brain_query_count} queries → {brain_memory_count} memories
   Open questions: {open_count}
   Iteration: {discovery_iteration}

{if triage_verdict == proceed}
✅ Triage passed all 5 gates. Brief is ready for prd-author.
➡️  Next skill: cuo/cpo/prd-author
{else if triage_verdict == revise}
⚠️  Triage flagged: {amber_categories}. Brief recorded with reservations.
   Choose:  (a) amend now via amendment-batch
            (b) proceed to prd-author anyway (reservations recorded)
            (c) stop
{else if triage_verdict == reject}
🛑 Triage rejected: {red_categories}. Brief recorded; downstream skills will refuse to consume.
   To override, address the reasons in `## Triage Reasoning` and re-run discovery.
{end}

{if open_questions > 0}
❓ Open questions ({open_count}):
{for each: - <question> [needs: <persona|human>]}
{end}

🔬 Authority distribution in Goals:
   human-edited:    {n}
   human-confirmed: {n}
   llm-explicit:    {n}
   llm-implicit:    {n}    {warn if > 0; goals should be at least llm-explicit}

📊 Trace: {trace_id}  |  Audit row: {audit_row_id}
```

## Triage emoji

| Verdict | Emoji |
| --- | --- |
| proceed | ✅ |
| revise | ⚠️ |
| reject | 🛑 |

## Localisation

Renders in `manifest.languages[0]` (typically `en`). Vietnamese rendering at v0.2.0+ when the i18n pipeline lands. ASCII visual markers used for CLI compatibility.

## Citations

- Pattern source — `cuo/cpo/fr-author/HUMAN_SUMMARY.md` and `cuo/cpo/fr-audit/HUMAN_SUMMARY.md`.
- Triage emoji palette is intentionally simple (✅/⚠️/🛑); colour blind users can rely on the text labels.
