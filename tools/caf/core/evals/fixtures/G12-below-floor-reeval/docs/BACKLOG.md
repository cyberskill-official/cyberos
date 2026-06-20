# BACKLOG

## Loop 1 — 2026-06-13

### Scope & method
- Protocol: v1.4.0 | Mode: gated | Depth: standard | Severity floor: High | Vectors: Architecture, Security, Testing
- Benchmark basis: internal — single-tenant internal API, no credible external comparator

Approved: L1-T1

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Protected routes | 0 of 7 routes require a token | INTERNAL TARGET — no external citation | `rg -c "require_auth" src/api` |

```
$ rg -c "require_auth" src/api
0
```

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L1-T1 | High | DONE | Security | Add bearer-token auth to every route; protected routes 0→7 | `pytest -q tests/test_auth.py` |

```
$ pytest -q tests/test_auth.py
7 passed in 0.4s
```

### Below-floor watch (re-evaluate at Phase 4 if a task changes the premise)
- Permissive CORS (allow_origins=["*"]) is **Low** while the API is unauthenticated — there is no session or credential for a cross-origin page to abuse. It escalates to **High the moment auth is added**, because the wildcard would then expose authenticated calls cross-origin. Premise owner: L1-T1.

## Loop 2 — 2026-06-13

### Scope & method
- Protocol: v1.4.0 | Mode: gated | Depth: standard | Severity floor: High | Vectors: Architecture, Security, Testing
- Benchmark basis: internal — single-tenant internal API, no credible external comparator
- Phase 4 re-evaluation of Loop 1: L1-T1 added auth, which changed the premise of the below-floor CORS watch; it now crosses the High floor, so it is promoted to L2-T1 below rather than left closed.

Approved: L2-T1

### Benchmark table

| Metric | Baseline | Target | Verify command |
|---|---|---|---|
| Wildcard CORS origins | UNMEASURED (config value, not a runtime metric) | No external benchmark applicable | — |

### Task table

| ID | Sev | Status | Vector | Description + expected delta | Verify command |
|---|---|---|---|---|---|
| L2-T1 | High | DONE | Security | Replace the allow_origins=["*"] wildcard with the env allowlist now that auth is live (premise changed by L1-T1); wildcard origins 1→0 | `pytest -q tests/test_cors.py` |

```
$ pytest -q tests/test_cors.py
5 passed in 0.3s
```
