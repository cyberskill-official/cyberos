# Testing evidence — host-a (TEN-203 / TEN-204)

## Machine gates
`bash .cyberos/cuo/gates/run-gates.sh` → **GREEN** (suites pass=66 fail=0 skip=1; doctor 17/17).
Transcript: `docs/batches/batch-ten-inv-host-a-gates-transcript.txt`.

## Focused suite (re-run at testing claim, 2026-07-26)

| Suite | Result |
|---|---|
| `cyberos-auth --lib plan_admin` | 4 passed |
| `cyberos-auth --test plan_change_http_test` | 3 passed, 1 ignored (P0301 needs DATABASE_URL) |
| `cyberos-ai-gateway --test metering_ai_tokens_emit_test` | 3 passed |
| `cyberos-ten` + `cyberos-metering` | green |

Awaiting Gate-2 human ACCEPT before `testing → done`.
