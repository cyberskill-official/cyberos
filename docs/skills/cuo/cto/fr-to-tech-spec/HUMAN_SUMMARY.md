# Human-readable batch summary template

> When `fr-to-tech-spec` finishes a batch in standalone mode (or completes the WORKER phase in chained mode), the runtime renders this template into chat. The user gets a quick, scannable overview of what was written + what needs human review.

## Template

```
🛠️ Tech-spec batch complete — `cuo/cto/fr-to-tech-spec` v{skill_version}

📦 Specs written: {count_written}
🚧 Specs paused at HITL gate: {count_hitl}
🔁 Specs that exhausted the iteration budget: {count_exhausted}
🛑 FRs refused (verdict ≠ pass): {count_refused}

{per-spec block — repeat for each spec}
─────────────────────────────────────────────
📄 {fr_id} → {spec_id}: {status_emoji} {status_text}
   FR section ↦ Spec section coverage: {coverage_percent}%
   Sizing: {S_count}S / {M_count}M / {L_count}L / {XL_count}XL
   Open questions: {open_count}
   Path: {spec_path}
   {if HITL: HITL category — {category}; details: {hitl_summary}}
   {if EXHAUSTED: reason — {reason}}
─────────────────────────────────────────────

📊 Trace: {trace_id}  |  Audit row: {audit_row_id}

{if any_hitl}
⚠️  HITL pause active — see {hitl_request_path} and answer the questions there.
The runtime will resume each paused spec when its HITL question is answered.

{if any_refusal}
🛑 Refused FRs:
{for each: FR-NNN — verdict {verdict} (e.g., needs_human, fail). Run fr-audit's resume protocol first.}

{if next_skill_recommendation}
➡️  Next skill in chain: {next_skill_recommendation}
The supervisor will route there automatically.
{else}
✅ End of chain. No follow-up skill recommended.
```

## Status emoji mapping

| Status | Emoji | When |
| --- | --- | --- |
| `PASS`         | ✅ | Spec written, all INV-* clean, ready for engineering review. |
| `HITL_PAUSE`   | 🚧 | One or more open questions need human input before spec is final. |
| `EXHAUSTED`    | 🛑 | Reached `max_iterations_per_fr` without convergence. |
| `REFUSED`      | ⛔ | FR's audit verdict was not `pass`; spec was not written. |

## Localisation

The template renders in the project's `manifest.languages[0]` (typically `en`). Vietnamese rendering at v0.2.0+ when the i18n pipeline (registry README Part 17) lands. Non-emoji visual markers (`────`) are intentionally ASCII to render correctly in CLI viewers + terminal-based chat clients.

## Citations

- Pattern source — `cuo/cpo/fr-author/HUMAN_SUMMARY.md` and `cuo/cpo/fr-audit/HUMAN_SUMMARY.md`.
- Localisation policy → registry README Part 17.
- HITL category list → this skill's CONTRACT_ECHO `hitl_categories` line.
