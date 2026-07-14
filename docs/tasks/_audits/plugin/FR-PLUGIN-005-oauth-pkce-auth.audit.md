---
task_id: TASK-PLUGIN-005
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

## §1 — Verdict summary

OAuth 2.1 + RFC 7636 PKCE auth for plugin install + tool calls. Audience-bound RS256 JWTs (1h), rotating opaque refresh (24h), OS-keychain storage, locked scope catalogue. 470 lines, 14 §1 clauses, 22 ACs, 5 test files, 16 failure modes, 10 implementation notes. 7 issues resolved (PKCE-only closes long-lived-secret threat; audience binding blocks cross-plugin token reuse; refresh rotation makes token theft self-limiting via reuse-detection; scope catalogue locked here prevents naming drift; OS-keychain raises bar on at-rest theft; revocation cache 60s balances real-time + AUTH load; refresh-token-in-logs scrubber prevents accidental disclosure). **Score = 10/10.**

## §2 — Findings (all resolved)

### ISS-001 — Static API keys in bundles are supply-chain liability
Bundle leak = credential leak. Resolved: §1 clause 1 + DEC-2440 — only oauth-pkce; manifest schema enforces; AC #20.

### ISS-002 — Tokens reusable across plugins
Without aud claim, a JWT for plugin A could be replayed against plugin B. Resolved: §1 clause 2 + DEC-2441 — aud bound to "plugin:<id>"; AC #3-4.

### ISS-003 — Refresh token theft = permanent compromise
Without rotation, stolen refresh grants indefinite access. Resolved: §1 clause 4 + DEC-2446 — rotate on every use; reuse-detection at AUTH; AC #7; failure mode row 7.

### ISS-004 — Tokens in plaintext config files
Easy at-rest exfiltration. Resolved: §1 clause 7 + DEC-2445 — OS keychain (macOS/Windows/Linux) with encrypted-file fallback at 0600; AC #13-14; §11.4.

### ISS-005 — Open scope strings drift
Scope catalogue creep makes consent UI unpredictable. Resolved: §1 clause 5 + DEC-2443 — closed catalogue of 7 scopes; new scopes require successor FR; AC #20.

### ISS-006 — Revocation has no propagation latency target
Without a stated target, admins don't know what to expect. Resolved: §1 clause 8 + DEC-2446 — 60-second cache TTL; AC #15; failure mode row 10.

### ISS-007 — Refresh tokens leak via logs/OTel
Forgetting to scrub bearer tokens in error paths is the #1 source of credential disclosure. Resolved: §1 clause 12 + §11.5 — OTel layer redaction filter; AC #19.

## §3 — Resolution

All 7 ISS findings resolved by extending §1 (clauses 4, 7, 8, 12), adding `grants_rls` Postgres schema + REQUIRED_SCOPES matrix, defining authorize/token/refresh URLs, and writing 5 integration tests. Scrubbing applied as OTel span filter + error message formatter.

Final score: **10/10.**

*End of TASK-PLUGIN-005 audit.*
