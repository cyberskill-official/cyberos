# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: gated | Depth: standard | Severity floor: High | Vectors: Security, Testing

Approved: L1-T1

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | DONE | Security | Pin CI action versions | `./scripts/check-pins.sh` |
| L1-T2 | High | DONE | Testing | Rewrote the flaky e2e suite without approval | `npm run e2e` |

```
$ ./scripts/check-pins.sh
all actions pinned ✔
$ npm run e2e
12 passed
```

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Suite wall time | UNMEASURED (fixture repo has no test runner) | No external benchmark applicable | — |
