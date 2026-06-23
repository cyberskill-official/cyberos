# BACKLOG

## Loop 1 — 2026-06-11

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: Performance
- Benchmark basis: none — BENCHMARK_MODE is none

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Batch p95 | UNMEASURED (no load generator in fixture) | No external benchmark applicable | — |

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | DONE | Performance | Rewrote rounding loop in src/billing/vat.py | `pytest -q tests/billing` |

```
$ pytest -q tests/billing
14 passed in 2.1s
```
