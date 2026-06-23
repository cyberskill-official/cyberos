# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.0.0 | Depth: standard | Severity floor: High | Vectors: Testing
- Benchmark basis: none — internal CLI (v1.0.0-era run; the Mode echo did not exist yet)

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Suite wall time | UNMEASURED (no runner in fixture) | No external benchmark applicable | — |

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | OPEN | Testing | Add failing-path tests for parser | `pytest -q` |
