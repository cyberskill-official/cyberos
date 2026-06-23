# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: (fixture)

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Source line count | 4,812 | INTERNAL TARGET — no external citation | `wc -l src/*.py` |
| Bundle size | 3.1 MB | INTERNAL TARGET — no external citation | `du -sh dist/` |

```
$ wc -l src/*.py
 4812 total
```

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | Medium | OPEN | Maintainability | Deflake seed handling in test fixtures; CI variance -50% | `pytest -q tests/test_seed.py` |
