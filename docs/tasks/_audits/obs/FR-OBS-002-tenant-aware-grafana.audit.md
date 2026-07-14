---
task_id: TASK-OBS-002
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
---

## §1 — Verdict summary

TASK-OBS-002 expanded from 166 lines to ~870. Added 6 §1 clauses (#9 memory audit row per query, #10 OTel metrics, #11 root-admin unfiltered exception, #12 JWKS cache, #13 supported endpoint list; expanded #4 with sev-1 audit; expanded #6 with overhead measurement). Full Rust types + per-language injection skeletons + handler + cross-tenant property test in §3 + §5. 17 ACs. 4 test files (proxy + 3 inject + property). 21 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — String-concat injection vs AST-based not specified; bypass class undocumented
First-pass §6 used `inject_promql_label(req.body(), &tenant_id)` with no specification. String concat would be bypass-able. Resolved: §1 #2 + DEC-146 mandate AST-based; §3 shows promql-parser usage; §2 rationale paragraph; AC #15 malformed query parse-fail.

### ISS-002 — Cross-tenant attempt not audit-emitted
First-pass §10 row "Caller attempts tenant_id=other → 400 (cross-tenant attempt logged sev-1)" but no audit-row spec. Resolved: §1 #4 + #9 mandate `obs.cross_tenant_query_attempt` memory row; §3 emit; AC #6 + §5 test.

### ISS-003 — Root-admin cross-tenant query path unspecified
Root-admin legitimately needs cross-tenant queries (compliance reports). First-pass had no exception. Resolved: §1 #11 normative special case + audit row `outcome: root_admin_unfiltered` + sev-2 informational; AC #12 + §5 test.

### ISS-004 — Per-query audit log not memory-emitted
First-pass §1 #8 said "log every query (with tenant_id) for compliance audit" without specifying memory. Compliance lookback needs durable audit. Resolved: §1 #9 mandates `obs.query_proxied` memory row with query_sha256 (privacy-preserving); AC #11 + §5 test.

### ISS-005 — Property test for cross-tenant data not specified
First-pass §4 AC #6 said "Cross-tenant attempt → 400" but tested only the rejected path. Property test needed for "data doesn't leak through legitimate query." Resolved: §5 `cross_tenant_property_test.rs` mirrors TASK-AI-018 — 1000 random queries × tenant pairs assert no leak.

### ISS-006 — JWT verification (TASK-AUTH-004 JWKS) not specified
First-pass §1 #1 said "Grafana's bearer token; the token MUST carry tenant_id claim (Lumi identity from TASK-AUTH-108)" — but TASK-AUTH-108 is future. Slice 1 needs working auth. Resolved: §1 #1 uses TASK-AUTH-004 JWT; JWKS verification with 5min cache; auth.rs module; depends_on includes TASK-AUTH-004.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of TASK-OBS-002 audit.*
