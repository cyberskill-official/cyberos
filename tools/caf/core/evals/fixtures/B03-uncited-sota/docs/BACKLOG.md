# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: (fixture)

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| TTFB | UNMEASURED (no staging environment) | <0.5s Palantir-grade enterprise standard | `curl -w '%{time_starttransfer}' -o /dev/null -s https://app.example.com` |

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | Medium | OPEN | Maintainability | Deflake seed handling in test fixtures; CI variance -50% | `pytest -q tests/test_seed.py` |
