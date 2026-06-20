# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: Testing
- Benchmark basis: none — internal tool

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Coverage gate report | 81 % | INTERNAL TARGET — no external citation | `python3 tools/coverage_report.py --markdown` |

```
$ python3 tools/coverage_report.py --markdown
| ID | Sev | Status | Description |
|---|---|---|---|
| COV-1 | Blocker | FAILING | apps/api uncovered branches |
```

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | OPEN | Testing | Raise branch coverage on apps/api 81%→90% | `python3 tools/coverage_report.py --markdown` |
