# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: Performance
- Benchmark basis: provided — comparator fixed by client contract

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Import wall time | UNMEASURED (importer not built yet) | 2x faster than LegacyVendor importer per contract SLA | — |

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | OPEN | Performance | Build importer benchmark harness first | `pytest -q tests/bench` |
