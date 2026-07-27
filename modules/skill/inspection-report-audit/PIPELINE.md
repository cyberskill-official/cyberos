# `inspection-report-audit` — pipeline

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| `inspection-report-author` | Default chain | Passes `inspection-report@1` path. |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| `harden-record-author` | Only after inspection-report-audit PASS + operator `/harden` | Report path (inspection chain only). |
| (none — terminal) | harden-record-audit PASS | Optional re-`/inspect` by operator; never auto-invoked. |

## Machine floor

Inspection audits run `tools/inspect-lint.mjs` before judgement. Harden audits check the record against its own evidence (scope, verification, fingerprints, HITL).
