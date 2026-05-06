# Human-readable batch summary template

> Rendered into chat after `prd-author` completes a PRD.

## Template

```
📑 PRD authored — `cuo/cpo/prd-author` v{skill_version}

📄 PRD: {prd_path}
🔗 From brief: {brief_path}
🏷️ Status: {prd_status}
🔁 Iteration: {prd_iteration}

📊 Authority distribution (Goals):
   human-edited:    {n}
   human-confirmed: {n}
   llm-explicit:    {n}
   llm-implicit:    0          ← INV-002 enforces zero

🚦 Outcome: {outcome_emoji} {outcome}
   {if PRD_COMPLETE}
   ✅ All sections populated; ready for prd-audit.
   ➡️  Next: cuo/cpo/prd-audit (when v0.2.5 ships)
   {else if HALTED_HITL}
   🚧 Paused at HITL — see {hitl_request_path}; HITL category: {hitl_category}.
   {else if REFUSED_REJECTED_BRIEF}
   ⛔ Refused — brief has triage_verdict: reject.
   Address triage reasons in `requirements-discovery` and re-run.
   {else if REFUSED_REVISE_NEEDS_OVERRIDE}
   ⚠️  Refused — brief has triage_verdict: revise.
   Re-invoke with proceed_despite_revise: true OR address the triage flags.
   {else if EXHAUSTED}
   🛑 Reached max amendment iterations ({iterations}). PRD frozen at current state.
   {else if USER_ABORTED}
   ✋ User aborted; PRD draft saved at iteration {prd_iteration}.
   {end}

❓ Open questions ({open_count}):
   {for each: - <question> [needs: <persona|human>]}

📐 Quality bars (raw):
   Performance:    {perf_summary}
   Availability:   {avail_target}
   Privacy:        {privacy_summary}
   Security:       {security_summary}

🔬 EU AI Act class: {eu_ai_act_risk_class}
{if class ∈ {limited, high}}
   ⚠️ AI Risk section populated; CLO sign-off required before approval.
{end}

🔐 Confidentiality: {confidentiality}
   {if regulated}
   ⚠️ Compliance Implementation Plan populated; CSecO sign-off required.
   {end}

📊 Trace: {trace_id}  |  Audit row: {audit_row_id}
```

## Outcome emoji

| Outcome | Emoji |
| --- | --- |
| `PRD_COMPLETE` | ✅ |
| `HALTED_HITL` | 🚧 |
| `REFUSED_REJECTED_BRIEF` | ⛔ |
| `REFUSED_REVISE_NEEDS_OVERRIDE` | ⚠️ |
| `EXHAUSTED` | 🛑 |
| `USER_ABORTED` | ✋ |

## Localisation

Same policy as siblings: `manifest.languages[0]`; ASCII visual markers for CLI compatibility.

## Citations

- Pattern source — sibling skills' HUMAN_SUMMARY.md files.
- Outcome enum source — `envelopes/prd-author.output.json`.
