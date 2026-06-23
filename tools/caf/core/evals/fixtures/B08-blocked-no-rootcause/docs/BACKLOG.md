# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: (fixture)

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | BLOCKED | Security | Upgrade vulnerable transitive dep | `npm audit --audit-level=high` |

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Suite wall time | UNMEASURED (fixture repo has no test runner) | No external benchmark applicable | — |
