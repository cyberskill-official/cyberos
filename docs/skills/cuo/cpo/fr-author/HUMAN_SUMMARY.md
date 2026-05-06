# `fr-author` human-summary template

> Rendered to chat at the end of every batch when the skill runs in **standalone** mode. Chained-mode invocations skip this — the upstream supervisor handles user-facing surfacing through its own conventions.

## When it renders

After every `BATCH_COMPLETE`, `BATCH_COMPLETE_WITH_AMENDMENTS`, or `HALTED_HITL` event in standalone mode. NOT rendered on `EXHAUSTED` (the supervisor has its own exhaustion announcement).

## Template

````
✅ **Batch complete** — {batch_outcome_label}

Wrote {len(frs_written)} feature request(s) to `{output_dir}`:

{for fr in frs_written:}
  - **{fr.id}** — {fr.title}  ·  status: `{fr.status}`  ·  {fr.fr_hash[:12]}…
{end-for}

{if amendments_pending:}
🔄 **Amendments pending review** ({len(amendments_pending)}):

{for amd in amendments_pending:}
  - **{amd.id}** — {amd.summary}  ·  risk: `{amd.risk_class}`
{end-for}

Reply `APPROVE`, `REVISE: <edits>`, or `REJECT` to dispatch.
{end-if}

{if hitl_pending:}
⏸️ **Paused — waiting on you**

{render: references/HITL_PROTOCOL.md HITL_BATCH_REQUEST format}
{end-if}

📋 **What I changed in BRAIN**:

{audit_block_per_AGENTS_md_§14}

📊 **Trace**: `{trace_id}`  ·  invocation #{invocation_counter}
🔗 **Next**: `{next_skill_recommendation or "—"}`

> Type `audit FR-NNN` to QA any of these now (chains into `cuo/cpo/fr-audit`).
> Type `more` to claim another batch from the same backlog.
> Type `pause` to stop here; manifest state is preserved for resume.
````

## Tone rules (deltas from PRD §6.2 base voice)

- **Compact.** Each FR is one line. No long descriptions in the summary; the user can open the file if they want detail.
- **Status-first.** The user wants to see at a glance: did anything PASS, did anything HITL_PAUSE, did anything EXHAUST. Counts before prose.
- **Action verbs.** "Wrote", "Paused", "Awaiting" — not "I have generated", "It would seem that", "If you wouldn't mind".
- **No celebratory noise.** No "Great job!" or "Perfect!" or "All done!". The user dispatched work; the work is done; report.
- **Hash prefix only.** Never paste full SHA-256 in chat. First 12 hex chars + ellipsis. The full hash is in the audit row.

## Standalone-mode-specific UX touches

- The "Type `audit FR-NNN`" hint is the chain-discovery affordance. Users who don't know `fr-audit` exists learn about it from this prompt.
- The `📋 What I changed in BRAIN` block uses the AGENTS.md §14 verbatim format so the user gets a consistent view of memory mutations across every standalone skill.
- Emojis are deliberately limited to 4 status icons (✅ 🔄 ⏸️ 📋 📊 🔗). Per registry README §0.3, emoji vocabulary is fixed; no free-form emoji.

## Chained-mode equivalent

When `fr-author` chains into `fr-audit`, this summary is NOT rendered. The supervisor instead emits a single-line trace:

```
fr-author → fr-audit  ·  3 FRs handed off  ·  trace_id={…}
```

The next skill's `HUMAN_SUMMARY.md` takes over from there.
