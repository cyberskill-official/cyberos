# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: (fixture)

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | DONE | Performance | Memoized invoice totals in src/billing/invoice.ts | `npm test` |

```
$ npm test
all 12 tests passed
```

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Suite wall time | UNMEASURED (fixture repo has no test runner) | No external benchmark applicable | — |
