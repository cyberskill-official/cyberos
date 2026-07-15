---
task_id: TASK-PLUGIN-002
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

## §1 — Verdict summary

CyberOS MCP bridge — single Rust binary supporting stdio + HTTP transports, MCP 2025-11-25 protocol, 8 tools across CUO/memory/SKILL, Tasks primitive for long-running execute_workflow + invoke_skill, 4-class error taxonomy, RLS-protected Postgres task store. 470 lines, 14 §1 clauses, 24 ACs, 5 test files, 15 failure modes, 11 implementation notes. 7 issues resolved (single binary both transports halves maintenance vs two-binary path; static 8-tool registry forces API stability; Tasks primitive prevents client request-thread blocking; Postgres-backed task state survives reconnect; 4-class error taxonomy makes host UX actionable; OTel emission completes 4-hop trace correlation; CORS-off-by-default closes browser-CSRF surface). **Score = 10/10.**

## §2 — Findings (all resolved)

### ISS-001 — Long-running tools block JSON-RPC connection
A 30-second workflow execution returned synchronously freezes the host request thread. Resolved: §1 clause 5 + DEC-2414 — Tasks primitive (handle + poll + cancel + resume); AC #4-7.

### ISS-002 — Task state in memory breaks reconnect-resume
Desktop hosts disconnect frequently; in-memory task state is lost. Resolved: §1 clause 5 + Postgres `plugin_host.tasks` table with RLS; AC #7-8.

### ISS-003 — Generic error responses kill host UX
"Something went wrong" gives the user nothing. Resolved: §1 clause 7 + DEC-2416 — 4 error classes (input_validation, authz_denied, upstream_unavailable, internal_error) + actionable hint per class; AC #9-13.

### ISS-004 — Cross-tenant data leak via task_id guess
Without RLS, task_id is a global namespace; tenant A guessing tenant B's id retrieves data. Resolved: §1 clause 6 + Postgres RLS policy `tasks_rls` + AC #8 cross-tenant test.

### ISS-005 — Stateful sessions complicate scaling
Sticky sessions tax load balancers and rolling deploys. Resolved: §1 clause 6 + DEC-2415 — stateless, JWT carries tenant_id; AC #16.

### ISS-006 — Destructive tools without confirm break TASK-MCP-006
TASK-MCP-006 requires elicit_confirm for destructive tools; bridge needs to enforce. Resolved: §1 clause 9 — bridge checks `elicit_confirm` flag, returns authz_denied if missing; AC #15.

### ISS-007 — OTel spans missing → 4-hop trace fragments
Trace correlation across host → bridge → upstream → downstream requires every hop to emit. Resolved: §1 clause 8 + DEC-2417 — required OTel emission with 6 attributes; AC #14; §11.8 implementation.

## §3 — Resolution

All 7 ISS findings resolved by extending §1 (clauses 5, 6, 7, 8, 9), adding 8 ACs (#4-8, #13-15), defining `plugin_host.tasks` schema with RLS, adding `error.rs` with 4-class taxonomy and hint registry, and wiring OTel spans through every handler. Bridge ships as static-linked musl binary for clean Fargate deploy.

Final score: **10/10.**

*End of TASK-PLUGIN-002 audit.*
