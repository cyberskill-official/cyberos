# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: Performance
- Benchmark basis: internal

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| LCP (homepage) | 1.8 s (verified via static code analysis) | INTERNAL TARGET — no external citation | `npx lighthouse-ci autorun` |

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | OPEN | Performance | Inline critical CSS; LCP 1.8s→1.2s | `npx lighthouse-ci autorun` |
