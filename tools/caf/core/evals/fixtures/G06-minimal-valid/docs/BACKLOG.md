# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: quick | Severity floor: High | Vectors: Testing
- Benchmark basis: none — quick pass, internal tool

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Coverage | UNMEASURED (coverage tooling not yet installed) | No external benchmark applicable | — |

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | OPEN | Testing | Install coverage tooling and capture a real baseline | `pytest -q --cov=src` |
