# `inspection-report-author` — pipeline

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| (none — standalone / slash command) | User runs `/inspect` or `/harden` or invokes directly | Operator provides target / report path via input envelope. |
| `inspection-report-audit` (PASS) | `/harden` chain only | Passes lint-clean `inspection-report@1` path. |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| `inspection-report-audit` | Default after every successful write | `next_skill_recommendation: inspection-report-audit` in output envelope. |
| (none — terminal) | User opts out of chaining | `chain_to: []` in input envelope. |

## Event emission

| subject | when |
|---|---|
| `inspection_report_author.artefact_written` | After every successful write of `inspection-report@1`. |
| `inspection_report_author.hitl_pause` | When a human gate blocks. |

## Halting and resuming

Halts on HITL (plan / review gates for harden; platform / credential halts for inspect), self-audit invariant breach, or operator interrupt. Resume from the last recorded manifest / record state; never skip a recorded human gate.
