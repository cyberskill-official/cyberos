# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: (fixture)

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Env audit findings | 1 | INTERNAL TARGET — no external citation | `grep -R "AKIA" -n .env* deploy/` |

```
$ grep -R "AKIA" -n .env* deploy/
deploy/staging.env:3:AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
```

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | Medium | OPEN | Maintainability | Deflake seed handling in test fixtures; CI variance -50% | `pytest -q tests/test_seed.py` |
