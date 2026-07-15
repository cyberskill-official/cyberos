# `task-audit` — human summary format

After each batch, the skill emits a short human-readable summary in chat (in addition to the structured output envelope).

## Per-batch summary

```
Task audit batch complete

Audited N task(s):
  - TASK-001-foo.md → audit verdict: pass (0 open, 1 warning, 3 iterations)
  - TASK-002-bar.md → audit verdict: needs_human (2 blocking issues — see HITL_BATCH_REQUEST below)
  - TASK-003-baz.md → audit verdict: pass (0 open, 0 warnings, 1 iteration)

Rubric:        fr_rubric@1.0
Total time:    <seconds>s
Reports:       written alongside each artefact as <name>.audit.md
Next:          PASS items can proceed to the next stage. HITL items wait for your reply.
```

## On HITL pause

After the per-batch summary, the skill emits the standard `HITL_BATCH_REQUEST` block per `references/HITL_PROTOCOL.md`. The block is the LAST thing in the response so the user's reply lands cleanly.

## On STALE-001 fire

When source-hash drift is detected, the skill emits a `STALE_DIFF` block BEFORE the summary so the operator sees what changed before being asked to choose REVERT / OVERWRITE / WONTFIX.

## On deterministic_drift fire

This is a catastrophic invariant breach. The skill emits a `CATASTROPHIC_DRIFT` block at the TOP of the response and halts immediately. No further audits run until the operator clears the breach via `cyberos doctor --repair --reason <text>`.

## Token budget transparency

The summary SHOULD include input + output token cost vs the configured limit, when known.

```
Token budget: 8,200 / 50,000 (16.4%)
```
