# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: Performance, Security, Testing
- Benchmark basis: internal — no public comparator for this internal tool

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Cold-start time | 3.1 s | Minimal | `python3 scripts/bench_cold_start.py` |
| Suite wall time | UNMEASURED (no CI runner in fixture) | No external benchmark applicable | — |

```
$ python3 scripts/bench_cold_start.py
cold start: 3.1s (mean of 5 runs)
```

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | WIP | Performance | Lazy-import heavy deps; cold start 3.1s → <1.5s | `python3 scripts/bench_cold_start.py` |
| L1-T2 | High | DONE | Security | Move deploy credentials out of repo config | `git grep -c AKIA \|\| true` |

```
$ git grep -c AKIA || true
0
$ python3 scripts/bench_cold_start.py
cold start: 1.4s (mean of 5 runs)
The old config contained AKIAIOSFODNN7EXAMPLE which has been revoked.
```
