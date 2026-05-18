# `crisis-communications-playbook-author` — human summary format

After each batch, the skill emits a short human-readable summary in chat (in addition to the structured output envelope).

## Per-batch summary

```
CRISIS-COMMS-PLAYBOOK batch <batch_run_id> complete

Wrote N CRISIS-COMMS-PLAYBOOK(s):
  - CRISIS-COMMS-PLAYBOOK-001: <slug> — PASS (audit verdict: pass, 0 open issues)
  - CRISIS-COMMS-PLAYBOOK-002: <slug> — HITL_PAUSE (2 blocking issues — see HITL_BATCH_REQUEST below)
  - CRISIS-COMMS-PLAYBOOK-003: <slug> — PASS (audit verdict: pass, 1 warning)

Manifest:    <output_dir>/manifest.json
Total time:  <seconds>s
Next:        crisis-communications-playbook-audit will run automatically on PASS items. HITL items wait for your reply.
```

## On HITL pause

After the per-batch summary, the skill emits the standard `HITL_BATCH_REQUEST` block per `references/HITL_PROTOCOL.md`. The block is the LAST thing in the response so the user's reply lands cleanly.

## On amendment request

When a `PLAN_AMENDMENT_REQUEST` fires (high-risk amendment that breaks the current batch), the request block appears BEFORE the per-batch summary. Operator MUST approve or revise before the batch can continue.

## On refinement proposal

When the self-audit invariants breach, the skill emits a `REFINEMENT_PROPOSAL` block describing the anomaly signal that fired and the operator action needed.

## Token budget transparency

The summary SHOULD include input + output token cost vs the configured limit, when known.

```
Token budget: 12,400 / 50,000 (24.8%)
```
