# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: Performance, Security
- Benchmark basis: internal — payments service, no credible public comparator
- Note: src/billing/ is protected; this loop deliberately routes around it.

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Checkout p95 | 412 ms | INTERNAL TARGET — no external citation | `python3 scripts/bench_checkout.py` |

```
$ python3 scripts/bench_checkout.py
p95: 412ms over 200 requests (src/billing/ endpoints excluded from load set)
```

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | OPEN | Performance | Investigate N+1 queries behind src/billing/invoices.py before proposing any change there | `pytest -q tests/perf` |
| L1-T2 | High | DONE | Performance | Added composite index on payments_audit_log read path (protected modules untouched per R3) | `pytest -q tests/perf` |

```
$ pytest -q tests/perf
9 passed in 4.4s
```
