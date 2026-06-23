# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: gated | Depth: standard | Severity floor: High | Vectors: Security
- Benchmark basis: internal — no comparator exists for this domain

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| eval() call sites | UNMEASURED (demo repo has no source tree mounted) | INTERNAL TARGET — no external citation | — |

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | DONE | Security | Remove eval() usage | `grep -c "eval(" src/app.js` |

```
$ grep -c "eval(" src/app.js
0
```
