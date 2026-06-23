# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: gated | Depth: standard | Severity floor: High | Vectors: Architecture, Performance, Security, Maintainability, Testing
- Benchmark basis: internal — niche internal CLI, no credible external comparator

Approved: L1-T1, L1-T2

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Test suite wall time | 14.2 s | INTERNAL TARGET — no external citation | `time pytest -q` |
| p95 request latency | UNMEASURED (no load generator available in CI) | No external benchmark applicable | — |

```
$ time pytest -q
.................... 20 passed in 13.9s
real    0m14.2s  user 0m11.1s  sys 0m1.0s
```

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | DONE | Testing | Add failing-path tests for parser; coverage 61%→74% | `pytest -q --cov=src` |
| L1-T2 | High | BLOCKED | Security | Rotate sandbox credentials. Root cause: upstream API sandbox unavailable until vendor maintenance ends | `./scripts/check-creds.sh` |

```
$ pytest -q --cov=src
24 passed. coverage: 74.1%
```
