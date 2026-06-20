# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: Maintainability, Testing
- Benchmark basis: internal — formatting torture test: bold cells, escaped pipes, ragged spacing

### Benchmark table

| Metric          | Baseline       | Target                                   | Verify command |
|:----------------|:--------------:|------------------------------------------|----------------|
| **TODO count**  | **37**         | INTERNAL TARGET — no external citation   | `grep -rcE "TODO\|FIXME" src/ \| awk -F: '{s+=$2} END {print s}'`   |
| _Suite time_    | 8.4 s          | INTERNAL TARGET — no external citation   | `time pytest -q`|

```
$ grep -rcE "TODO|FIXME" src/ | awk -F: '{s+=$2} END {print s}'
37
$ time pytest -q
61 passed in 8.2s
real    0m8.4s
```

### Task table

| ID    | Sev    | Status | Vector          | Description + expected delta                                   | Verify command   |
|-------|--------|--------|-----------------|----------------------------------------------------------------|------------------|
| L1-T1 | High   | DONE   | Maintainability | Burn down stale TODOs in parser module; 37 → 21 (**-43%**)     | `grep -rcE "TODO\|FIXME" src/ \| awk -F: '{s+=$2} END {print s}'` |
| L1-T2 | Medium | OPEN   | Testing         | Split slow integration tests behind marker `-m "not slow"`     | `time pytest -q` |

```
$ grep -rcE "TODO|FIXME" src/ | awk -F: '{s+=$2} END {print s}'
21
```
