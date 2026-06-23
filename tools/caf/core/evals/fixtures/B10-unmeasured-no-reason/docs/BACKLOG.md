# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: (fixture)

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Cold-start time | UNMEASURED | INTERNAL TARGET — no external citation | `hyperfine './bin/app --help'` |

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | Medium | OPEN | Maintainability | Deflake seed handling in test fixtures; CI variance -50% | `pytest -q tests/test_seed.py` |
