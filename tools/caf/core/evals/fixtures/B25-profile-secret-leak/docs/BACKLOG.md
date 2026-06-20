# BACKLOG

## Loop 1 — 2026-06-10

### Scope & method
- Protocol: v1.3.0 | Mode: autonomous | Depth: standard | Severity floor: High | Vectors: Security
- Benchmark basis: none — internal service

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Token rotation age | UNMEASURED (vault API offline in fixture) | No external benchmark applicable | — |

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | DONE | Security | Rotated service token acme_Ab12Cd34Ef56Gh78Ij90 across environments | `./scripts/check-token-age.sh` |

```
$ ./scripts/check-token-age.sh
all tokens < 24h old
```
