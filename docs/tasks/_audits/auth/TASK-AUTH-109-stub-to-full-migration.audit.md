---
task_id: TASK-AUTH-109
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per task-audit skill §0)
---

## §1 — Verdict summary

TASK-AUTH-109 ships the TASK-AUTH-101 stub → full migration enforcer — 30-day grace + verifier hook + refresh hook + per-tenant extension override + lazy cutover transition. Scope: 25 §1 normative clauses covering per-tenant `auth_migration_state` table with append-only history at SQL grant, 30-day default grace with env override (7-90d), per-tenant cutover_at (existing tenants seeded at ship, new via TASK-TEN-001 hook), one-extension-max per tenant via DB CHECK, root-admin-only extension handler with reason-required + max 60 additional days, refresh hook injecting rbac_v on every refresh + logging prior_rbac_v_present, lazy cutover transition on first post-grace verification (idempotent via WHERE status != 'cutover_completed'), 4 memory audit kinds with 1% sampling during grace + 100% in last 24h, clear 401 rejection body with action_required = "refresh", 60s in-memory cache for verifier hot-path performance, sev-2 alarm at > 100/h rejected, preview API for operator decision support, `auth_provisioner` SQL role split (cyberos_app blocked from UPDATE/DELETE on migration_state). 21 rationale paragraphs. §3 contains: migration 0014 (auth_migration_state with immutability trigger + role grants + seed for existing tenants), migration 0015 (append-only refresh_log), state cache with 60s TTL, verifier hook with sampling logic + lazy cutover, refresh hook injecting rbac_v + log row, extend-grace handler with role + validation. 27 ACs. 31 failure-mode rows. 21 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Grace window enforcement absent
First-pass had no enforcer for TASK-AUTH-101's grace promise. Resolved: §1 #5 + DEC-442 + verifier hook with cache-fast path; AC #3 + #4.

### ISS-002 — Refresh path didn't inject rbac_v
First-pass left refreshed tokens still missing rbac_v. Resolved: §1 #6 + DEC-449 + refresh hook + log row; AC #5.

### ISS-003 — No operator visibility into grace progress
First-pass had no preview API. Resolved: §1 #8 + DEC-447 + preview returning counts + can_extend; AC #15.

### ISS-004 — Cutover transition risk (double-emission)
First-pass had no idempotency. Resolved: §1 #10 + #22 + trigger predicate `status != 'cutover_completed'` + lazy transition; AC #13 + #14.

### ISS-005 — Extension unbounded
First-pass had no cap. Resolved: §1 #7 + DEC-445 + max 60 additional days + 1 extension max per tenant via DB CHECK; AC #6 + #7 + #8.

### ISS-006 — Tenant-admin could extend (privilege escalation)
First-pass had loose role check. Resolved: §1 #7 + root-admin-only; AC #7.

### ISS-007 — cutover_at mutable post-completion
First-pass allowed UPDATE. Resolved: §1 #9 + DEC-442 + trigger `cutover_immutable_post_completion`; AC #11 + #12.

### ISS-008 — Opaque 401 left clients guessing
First-pass returned 401 with no body. Resolved: §1 #5 + DEC-448 + body with `action_required: "refresh"`; AC #4.

## §3 — Resolution

All 8 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (verifier hook + refresh hook × per-tenant cutover_at × lazy idempotent transition × 1-extension cap × root-admin gate × cutover immutability × clear 401 body × 4 memory audit kinds × 60s cache × preview API × sev-2 alarm × append-only state + refresh log × env-bounded default), not by line targets.

---

*End of TASK-AUTH-109 audit.*
