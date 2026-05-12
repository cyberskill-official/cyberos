# Human-readable batch summary template

> When `fr-with-tasks` finishes a batch in standalone mode (or completes the WORKER phase in chained mode), the runtime renders this template into chat.

## Template

```
📋 FR-with-tasks batch complete — `cuo/cpo/fr-with-tasks` v{skill_version}

📦 FRs written: {count_written}
🚧 FRs paused at HITL gate: {count_hitl}
🔁 FRs that exhausted iterations: {count_exhausted}
🛑 FRs refused (input validation failed): {count_refused}

{per-FR block — repeat for each FR}
─────────────────────────────────────────────
📄 {fr_id}: {status_emoji} {status_text}
   Title:         {title}
   Tasks emitted: {task_count}  ({S}S / {M}M / {L}L / {XL}XL)
   Assignable:    {n_human_only} human-only · {n_ai_only} AI-only · {n_either} either
   Parallelisable: {n_parallel} of {task_count}
   Est. effort:   {total_human_hours}h human · {total_ai_tokens} tokens AI
   Open questions: {open_count}
   Path: {fr_path}
   {if HITL: HITL category — {category}; details: {hitl_summary}}
   {if EXHAUSTED: reason — {reason}}
─────────────────────────────────────────────

📊 Trace: {trace_id}  |  Audit row: {audit_row_id}

{if any_hitl}
⚠️  HITL pause active — see {hitl_request_path}.

{if next_skill_recommendation}
➡️  Next skill in chain: {next_skill_recommendation}  (typically fr-audit)
{else}
✅ End of chain. Run `cyberos proj sync FR-NNN` per FR to create tickets.
```

## Status emoji mapping

| Status | Emoji | When |
| --- | --- | --- |
| `PASS`         | ✅ | FR + all tasks emitted, INV-001 through INV-014 clean |
| `HITL_PAUSE`   | 🚧 | One or more tasks have acceptance_test unclear or risk-tier ambiguous |
| `EXHAUSTED`    | 🛑 | Reached `max_iterations_per_fr` without convergence |
| `REFUSED`      | ⛔ | Input failed boot validation |

## Localisation

Renders in the project's `manifest.languages[0]`. Vietnamese rendering at v0.2.0+ when the i18n pipeline lands. ASCII separators on purpose.

## Citations

- Pattern source — `cuo/cpo/fr-author/HUMAN_SUMMARY.md`
- Status semantics — INVARIANTS.md INV-001 through INV-014
- HITL category list — SKILL.md HITL-gates section
