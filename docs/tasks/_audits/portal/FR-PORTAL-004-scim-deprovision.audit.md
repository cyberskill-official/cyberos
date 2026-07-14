---
task_id: TASK-PORTAL-004
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands SCIM 2.0 DELETE deprovision with 30s end-to-end session-invalidation SLO on top of TASK-PORTAL-003. Final form: 1,090 lines, 24 §1 normative clauses, 20 acceptance criteria, 10 verification tests, 23 failure-mode rows, 21 implementation notes. 3 migrations, 4 REST endpoints, 7 + 1 memory audit kinds, dual-channel kill (JWT blacklist via Redis sorted-set + WebSocket close frame 4001), cascade revocation across 4 targets, 5-min grace period + tenant_admin restore, nightly reconciliation, per-Engagement isolation.

7 issues caught by self-audit, all resolved.

## §2 — Findings (all resolved)

### ISS-001 — Sync phase vs SCIM-client timeout mismatch

§1 #1 says SCIM DELETE returns 204; §1 #3 requires sync phase < 1s. But Okta + Azure SCIM clients have 30s default timeout. If we returned 202 Accepted (async pattern), clients might handle differently than 204. Resolved: 204 is the right SCIM idiom per RFC 7644 §3.6 — the spec's sync phase guarantee makes 204 truthful (subject IS tombstoned + blacklisted by return time). §11.14 clarifies "204 truthful because sync phase committed the critical state".

### ISS-002 — Dual-channel kill atomicity

§1 #5 + §1 #7 invalidate via blacklist AND WebSocket close. But ordering matters: if WS closes first, a quick HTTP-API call between phases gets through. Resolved: §6.1 orchestrator runs sync phase (which includes blacklist add BEFORE cascade async phase containing WS close). API protection is in place before WS close starts. AC #2 verifies JWT rejection within 30s; AC #3 verifies WS close — both independently testable.

### ISS-003 — Reconciliation auto-revoke creates audit-row duplication risk

§1 #16 auto-revokes orphan sessions + emits audit. But this could double-count if the original cascade just took longer than expected (race between cascade completion and reconciliation). Resolved: §11.12 dedup by `(subject_id, target)` last-30d window — reconciliation skips targets already revoked recently by the original cascade.

### ISS-004 — Grace period + concurrent admin restore race

§1 #8 + §1 #9 — within 5 min, BOTH a SCIM PUT and an admin POST restore can land simultaneously. Resolved: §10 row covers — tx isolation: last writer wins; both emit audit rows; idempotent on subject status. No state corruption.

### ISS-005 — Multi-node JWT blacklist propagation under Redis pub/sub failure

§1 #12 + §6.2 use Redis pub/sub for cross-node propagation. If Redis is down, propagation fails. Resolved: §10 row covers — Redis-only blacklist works best-effort; sev-1 alert; nightly reconciliation re-syncs Postgres → Redis. Postgres is the durable source of truth (each node falls back to Postgres SISMEMBER on cache miss).

### ISS-006 — Hard purge query param vs hyperlink

§1 #21 returns 501 for `?action=hard_purge` at slice 2. But this isn't standard SCIM (RFC 7644 doesn't define action query params on DELETE). Resolved: §11.15 documents this as our CyberOS extension; slice 3 GDPR flow will surface via a separate admin endpoint, not query param. Slice 2 501 is the placeholder signalling.

### ISS-007 — Per-Engagement isolation when subject in many Engagements

§1 #11 says per-Engagement isolation. But subject A in Engagements X, Y, Z — when X SCIM DELETEs, do we revoke X-scoped JWTs only? Resolved: §11.10 confirms each JWT carries `engagement_id` claim (TASK-PORTAL-003 derivative); blacklist scoped to `(tenant_id, engagement_id)`; AC #12 verifies. Y + Z JWTs unaffected.

## §3 — Resolution

All 7 mechanical concerns addressed. SCIM idiom correct (204); dual-channel ordering sound; reconciliation dedup logic explicit; concurrent grace/restore race acceptable; Redis-failure fallback path documented; hard-purge query param scoped to CyberOS extension; per-Engagement scoping validated end-to-end.

The 1,090-line length is justified by 3 migrations + 4 endpoints + 7 memory kinds + dual-channel kill mechanism + 4 cascade targets + reconciliation job + grace period + admin restore + per-Engagement isolation + 23 failure modes covering distributed-system pitfalls. Density matches peer FRs at similar scope.

**Score = 10/10.**

---

*End of TASK-PORTAL-004 audit.*
