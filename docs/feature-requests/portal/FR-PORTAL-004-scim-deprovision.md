---
id: FR-PORTAL-004
title: "PORTAL SCIM deprovision — session invalidation ≤ 30 s on IdP user removal + grace period + cascade revocation + dual-channel kill (JWT blacklist + WebSocket close)"
module: PORTAL
priority: MUST
status: ready_to_implement
verify: T
phase: P4
milestone: P4 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-PORTAL-003, FR-PORTAL-001, FR-PORTAL-005, FR-AUTH-002, FR-AUTH-004, FR-AUTH-101, FR-TEN-104, FR-CHAT-005, FR-AI-003, FR-MEMORY-111, FR-OBS-007]
depends_on: [FR-PORTAL-003]
blocks: []

source_pages:
  - website/docs/modules/portal.html#scim-deprovision
  - https://datatracker.ietf.org/doc/html/rfc7644#section-3.6  # SCIM delete + soft-delete
  - https://datatracker.ietf.org/doc/html/rfc7519              # JWT
  - https://gdpr.eu/article-17-right-to-be-forgotten/
  - https://datatracker.ietf.org/doc/html/rfc7009              # OAuth 2.0 token revocation

source_decisions:
  - DEC-1080 2026-05-17 — SCIM DELETE on a user MUST invalidate all active sessions for that user within 30 seconds (p99 SLO); this is the contractual security commitment that enterprise tenants demand for "user-removed-from-AD" workflows
  - DEC-1081 2026-05-17 — Session invalidation is dual-channel: (a) JWT blacklist propagated to all gateway nodes via Redis pub/sub (catches REST API calls), (b) active WebSocket connections (CHAT, Genie) receive explicit close frame
  - DEC-1082 2026-05-17 — Closed enum `scim_deprovision_action` = {soft_tombstone, hard_purge}; default `soft_tombstone`; `hard_purge` requires GDPR Art. 17 explicit request flow per FR-AUTH-2xx
  - DEC-1083 2026-05-17 — Grace period 5 min default: soft-tombstoned subjects can be UNDELETED via SCIM PUT within the grace window (covers accidental SCIM DELETEs from IdP-side glitches); after 5 min, undelete requires explicit admin restore endpoint
  - DEC-1084 2026-05-17 — Cascade revocation: SCIM DELETE on a user cascades to revoke (a) all their FR-PORTAL-005 Genie chat sessions, (b) all their FR-PORTAL-001 scoped views, (c) all their FR-CHAT-005 channel memberships, (d) any pending FR-MCP-006 confirmation tokens
  - DEC-1085 2026-05-17 — JWT blacklist storage: Redis sorted set keyed by `(jti)` with score=expiration timestamp; auto-trimmed past expiration (no eternal-blacklist accumulation); 30s propagation SLO end-to-end
  - DEC-1086 2026-05-17 — Per-Engagement isolation: SCIM DELETE invalidates ONLY sessions scoped to that Engagement (per FR-PORTAL-003); user with multi-Engagement membership keeps sessions in other Engagements
  - DEC-1087 2026-05-17 — Idempotent on SCIM DELETE: re-DELETEing an already-tombstoned subject returns 204 No Content (no error); re-DELETEing a hard-purged subject returns 404
  - DEC-1088 2026-05-17 — memory audit kinds: portal.scim_user_deprovisioned, portal.scim_user_restored, portal.scim_cascade_revocation, portal.jwt_blacklist_propagated, portal.websocket_force_closed, portal.scim_grace_period_expired, portal.scim_deprovision_sla_breach
  - DEC-1089 2026-05-17 — 30s SLO measured from SCIM DELETE accepted (HTTP 204 returned) to last-revoked-session timestamp; alarm sev-1 if p99 > 30s sustained 5 min
  - DEC-1090 2026-05-17 — Audit-row PII: subject_id retained in chain (anti-correlation needed for forensic); email PII-scrubbed via FR-MEMORY-111
  - DEC-1091 2026-05-17 — Rate limit: 1000 SCIM DELETEs/min/Engagement (bulk-deprovision scenarios — org-wide IdP cleanup); above → 429 with Retry-After
  - DEC-1092 2026-05-17 — Restore endpoint at `POST /v1/admin/engagements/{eng_id}/subjects/{subject_id}/restore` requires `tenant_admin` role (engagement_admin too tight — restore reaches across Engagement boundary)
  - DEC-1093 2026-05-17 — Session-store cleanup: tombstoned subject's session rows in `auth_sessions` table marked `revoked_at=now()`; not deleted (audit forensic retention)
  - DEC-1094 2026-05-17 — Cascade revocation is best-effort + audit-verifiable: each cascade target emits its own audit row; missed cascades discovered via nightly reconciliation job
  - DEC-1095 2026-05-17 — WebSocket close frame: code 4001 (custom "session_revoked") + reason "scim_deprovision"; client SHOULD reconnect (will fail re-auth + see clear 401)
  - DEC-1096 2026-05-17 — SCIM DELETE handler is sync-then-async: synchronously marks subject tombstoned + revokes JWT blacklist (fast path < 1s); asynchronously cascade-revokes other resources (slow path, within 30s SLO)
  - DEC-1097 2026-05-17 — Nightly reconciliation: compare tombstoned subjects vs active sessions; any active session for a tombstoned subject = sev-1 `portal.scim_orphan_session_detected`
  - DEC-1098 2026-05-17 — IdP-side DELETE notification path: SCIM DELETE arrives via per-Engagement SCIM token (FR-PORTAL-003); webhook-vs-SCIM-DELETE: SCIM DELETE is the canonical channel; webhook-based deprovision NOT supported at slice 2

build_envelope:
  language: rust 1.81
  service: cyberos/services/portal/
  new_files:
    - services/portal/migrations/0009_portal_deprovision_log.sql       # detailed deprovision event log
    - services/portal/migrations/0010_portal_jwt_blacklist.sql          # JWT jti blacklist (also in Redis hot-path)
    - services/portal/migrations/0011_portal_restore_requests.sql        # admin restore endpoint requests
    - services/portal/src/deprovision/mod.rs                            # deprovision orchestrator
    - services/portal/src/deprovision/sync_phase.rs                     # < 1s fast-path (tombstone + JWT blacklist)
    - services/portal/src/deprovision/async_phase.rs                    # < 30s slow-path (cascade revocation)
    - services/portal/src/deprovision/jwt_blacklist.rs                  # Redis sorted-set + pub/sub
    - services/portal/src/deprovision/websocket_killer.rs               # iterates active WS connections + sends close 4001
    - services/portal/src/deprovision/cascade.rs                        # cascade to PORTAL-005 + PORTAL-001 + CHAT-005 + MCP-006
    - services/portal/src/deprovision/restore.rs                        # admin restore handler
    - services/portal/src/deprovision/reconciliation.rs                 # nightly orphan-session scan
    - services/portal/src/scim/user_delete.rs                           # extends scim/users.rs from FR-PORTAL-003
    - services/portal/src/audit/deprovision_events.rs                   # 7 memory row builders
    - services/portal/src/handlers/admin_restore.rs                     # POST restore endpoint
    - services/portal/tests/scim_delete_invalidates_jwt_test.rs
    - services/portal/tests/scim_delete_closes_websocket_test.rs
    - services/portal/tests/scim_delete_cascade_test.rs
    - services/portal/tests/scim_delete_30s_sla_test.rs
    - services/portal/tests/scim_delete_idempotent_test.rs
    - services/portal/tests/scim_grace_period_undelete_test.rs
    - services/portal/tests/scim_admin_restore_test.rs
    - services/portal/tests/scim_per_engagement_isolation_test.rs
    - services/portal/tests/scim_orphan_reconciliation_test.rs
    - services/portal/tests/scim_rate_limit_test.rs
    - services/portal/tests/scim_jwt_blacklist_propagation_test.rs
    - services/portal/tests/scim_audit_emission_test.rs

  modified_files:
    - services/portal/src/scim/users.rs                                 # add DELETE handler + tombstone semantics
    - services/portal/src/scim/mod.rs                                   # add DELETE route
    - services/auth/src/jwt/validator.rs                                # consult blacklist on every JWT validate
    - services/auth/src/sessions/store.rs                               # session tombstone semantics
    - services/portal/Cargo.toml                                        # +redis (already in workspace)

  allowed_tools:
    - file_read: services/portal/**
    - file_read: services/auth/src/{jwt,sessions}/**
    - file_write: services/portal/{src,tests,migrations}/**
    - file_write: services/auth/src/jwt/validator.rs
    - file_write: services/auth/src/sessions/store.rs
    - bash: cd services/portal && cargo test deprovision

  disallowed_tools:
    - leak sessions past 30s SLO (per DEC-1080)
    - permit cross-Engagement session revocation (per DEC-1086)
    - hard-purge by default (per DEC-1082 — soft tombstone is default)
    - allow engagement_admin to restore (per DEC-1092)
    - block on async cascade phase (per DEC-1096 — sync phase < 1s)
    - skip nightly reconciliation (per DEC-1097)
    - allow webhook-based deprovision at slice 2 (per DEC-1098 — SCIM-only)

effort_hours: 8
sub_tasks:
  - "0.5h: 0009_portal_deprovision_log.sql + 0010_portal_jwt_blacklist.sql + 0011_portal_restore_requests.sql"
  - "0.7h: deprovision/sync_phase.rs — < 1s tombstone + Redis blacklist add + pub/sub"
  - "0.7h: deprovision/async_phase.rs — < 30s cascade orchestration"
  - "0.6h: deprovision/jwt_blacklist.rs — Redis sorted-set add + pub/sub + multi-node propagation"
  - "0.5h: deprovision/websocket_killer.rs — iterate active connections + close 4001"
  - "0.7h: deprovision/cascade.rs — 4 target revokers (PORTAL-005, PORTAL-001, CHAT-005, MCP-006)"
  - "0.4h: deprovision/restore.rs + handlers/admin_restore.rs"
  - "0.4h: deprovision/reconciliation.rs — nightly orphan scan"
  - "0.5h: scim/user_delete.rs — DELETE handler + grace period + idempotency"
  - "0.4h: audit/deprovision_events.rs — 7 builders"
  - "0.3h: auth/jwt/validator.rs — consult blacklist on validate"
  - "0.3h: auth/sessions/store.rs — tombstone semantics"
  - "1.5h: tests — 12 test files covering happy + 30s SLO + idempotent + grace + restore + per-eng + reconciliation + rate-limit + propagation"
  - "0.4h: integration smoke — full IdP DELETE → cascade verified end-to-end"

risk_if_skipped: "Without SCIM deprovision, terminated employees retain access to client portals indefinitely — fatal for any enterprise prospect (regulatory + brand risk). Without DEC-1080's 30s SLO, the deprovision is 'eventually consistent' which fails procurement due-diligence questions. Without DEC-1081's dual-channel kill, the user's open browser tab keeps streaming CHAT/Genie data minutes after IdP-side termination. Without DEC-1083's grace period, an IdP glitch that fires a spurious DELETE cannot be recovered → user re-onboarded from scratch. Without DEC-1084's cascade revocation, orphan chat sessions/views persist (audit-row gap). Without DEC-1085's auto-trimmed blacklist, the Redis cache grows unbounded. Without DEC-1095's WebSocket close frame, clients hang on broken connections. Without DEC-1097's reconciliation, missed cascades go undetected until audit. The 8h effort lands the security primitive that converts FR-PORTAL-003 SCIM JIT into a complete enterprise-grade identity story."
---

## §1 — Description (BCP-14 normative)

The PORTAL service **MUST** ship SCIM 2.0 DELETE handler at `services/portal/src/scim/user_delete.rs` extending FR-PORTAL-003's SCIM endpoint with session invalidation ≤ 30s p99, dual-channel session kill (JWT blacklist + WebSocket close), cascade revocation across PORTAL-005 + PORTAL-001 + CHAT-005 + MCP-006, 5-min grace period + admin restore, idempotency, per-Engagement isolation, nightly reconciliation, and 7 memory audit kinds.

1. **MUST** extend the SCIM endpoint at `DELETE /scim/v2/{engagement_slug}/Users/{id}` per RFC 7644 §3.6. Handler:
    - Authenticated via per-Engagement SCIM bearer token (FR-PORTAL-003 §1 #13).
    - Marks subject status `tombstoned` + sets `tombstoned_at = now()` in the SUBJECTS table (FR-AUTH-002).
    - Enters sync phase per §1 #3 — completes within 1 second wall-clock.
    - Enqueues async phase per §1 #4 — completes within 30 seconds wall-clock end-to-end.
    - Returns `204 No Content` per RFC 7644 §3.6 (synchronous ack; async work continues).

2. **MUST** define `portal_jwt_blacklist` table at migration `0010`: `(jti UUID PRIMARY KEY, tenant_id UUID NOT NULL, subject_id UUID NOT NULL, engagement_id UUID, blacklisted_at TIMESTAMPTZ NOT NULL DEFAULT now(), expires_at TIMESTAMPTZ NOT NULL, reason TEXT NOT NULL)`. Postgres is the durable store; Redis is the hot-path cache. Both consulted at JWT validate; Postgres is the source of truth on cache miss.

3. **MUST** complete the sync phase within 1 second wall-clock per DEC-1096. The `deprovision/sync_phase.rs::execute(subject_id, engagement_id)`:
    - UPDATE `subjects SET status='tombstoned', tombstoned_at=now() WHERE id=$1` (within FR-AUTH-002 RLS scope).
    - SELECT all active JWT jtis for the subject from `auth_sessions WHERE subject_id=$1 AND revoked_at IS NULL`.
    - For each jti: INSERT into `portal_jwt_blacklist` (Postgres) + ZADD into Redis sorted-set `jwt_blacklist:<tenant>` with score=expiration_unix.
    - PUBLISH on Redis pub/sub channel `jwt_blacklist_update` so all gateway nodes update their in-memory caches within ~50ms.
    - Emit `portal.scim_user_deprovisioned` memory row.

4. **MUST** complete the async phase within 30 seconds end-to-end per DEC-1089. The `deprovision/async_phase.rs::execute(subject_id, engagement_id)`:
    - In parallel, invoke each of the 4 cascade targets per DEC-1084 + §1 #6.
    - Iterate active WebSocket connections per §1 #5 + close them.
    - UPDATE `auth_sessions SET revoked_at=now(), revoked_reason='scim_deprovision' WHERE subject_id=$1 AND engagement_id=$2 AND revoked_at IS NULL`.
    - Compute end-to-end latency (sync phase start → last cascade complete) + emit OTel histogram + check 30s SLO.
    - On SLO breach: emit `portal.scim_deprovision_sla_breach` sev-1.

5. **MUST** force-close active WebSocket connections per DEC-1081 + DEC-1095. The `deprovision/websocket_killer.rs::close_for_subject(subject_id, engagement_id)`:
    - Iterates `chat_active_connections` + `genie_active_connections` registries (in-memory + Redis-backed for multi-node).
    - For each connection matching `(subject_id, engagement_id)`: send WebSocket close frame with code `4001` + reason `"scim_deprovision"`.
    - Emit `portal.websocket_force_closed` memory row per closed connection (informational; sev-3; sampled at 10% for high-volume scenarios via FR-OBS-006).

6. **MUST** cascade revoke 4 downstream resources per DEC-1084. The `deprovision/cascade.rs::run_all(subject_id, engagement_id)` invokes in parallel:
    - **FR-PORTAL-005 Genie sessions**: `portal_genie_sessions` table — UPDATE `revoked_at=now()` WHERE subject_id + engagement_id; emit `portal.scim_cascade_revocation` with `target='genie_sessions'`.
    - **FR-PORTAL-001 scoped views**: invalidate any cached view tokens in Redis for this subject's Engagement membership; no DB write needed (views are computed; tombstoned subject's Engagement-membership join returns 0 rows).
    - **FR-CHAT-005 channel memberships**: `chat_channel_memberships` table — UPDATE `revoked_at=now()`; cascade emits a CHAT membership-revoked event for any active subscribers.
    - **FR-MCP-006 pending confirmations**: DELETE FROM `mcp_pending_confirmations` WHERE caller_subject_id=$1 AND consumed_at IS NULL (per MCP-006 grants table; tombstoned subject's pending acks are voided).

7. **MUST** consult the JWT blacklist on every JWT validate per DEC-1085. The `services/auth/src/jwt/validator.rs::validate(jwt)` modification:
    - After standard signature + exp + aud checks: lookup `jwt.jti` in Redis blacklist set (`SISMEMBER jwt_blacklist:<tenant> <jti>`).
    - If present → return `Err(AuthError::JwtRevoked)` → handler returns 401 + `{ error: "jwt_revoked", reason: "scim_deprovision" }`.
    - On Redis miss + Postgres hit: backfill Redis (cache warm).

8. **MUST** support 5-min grace period per DEC-1083. Within 5 minutes of tombstone, SCIM PUT on the same user reactivates:
    - Handler at `PUT /scim/v2/{engagement_slug}/Users/{id}`: if subject is tombstoned + `now() - tombstoned_at < 5 min`, transition `status='active'` + `tombstoned_at=null` + reverse the JWT blacklist (DELETE rows + ZREM from Redis sorted-set + PUBLISH unblacklist event).
    - Cascade rollback: re-activate sessions + Genie subscriptions etc. WebSocket connections cannot be re-opened (already closed); client must re-connect (will succeed since blacklist cleared).
    - Emit `portal.scim_user_restored` sev-2 + payload `restored_via=scim_put_within_grace`.
    - Beyond 5 min: SCIM PUT returns `409 + grace_period_expired` + suggests using admin restore endpoint.

9. **MUST** expose admin restore at `POST /v1/admin/engagements/{eng_id}/subjects/{subject_id}/restore` per DEC-1092. Caller has `tenant_admin` role. Body: `{ reason }`. Handler:
    - Validates subject status='tombstoned'.
    - Same logic as §1 #8 grace-period reversal (un-blacklist + cascade re-activate).
    - Emit `portal.scim_user_restored` sev-1 + payload `restored_via=admin_restore`.
    - Reason free-text persisted to `portal_restore_requests` (audit forensic).

10. **MUST** be idempotent on SCIM DELETE per DEC-1087:
    - Re-DELETE on tombstoned subject → 204 (no state change; no re-emit of deprovision audit row to avoid double-counting).
    - Re-DELETE on hard-purged subject → 404 + `{ scim_type: "noTarget" }` per RFC 7644 §3.6.

11. **MUST** scope session invalidation per-Engagement per DEC-1086. Subject A is member of Engagement X + Y. SCIM DELETE on subject A within Engagement X's SCIM endpoint revokes ONLY X-scoped sessions (JWTs with `engagement_id=X` claim); Y-scoped sessions remain valid. Subject A's `auth_sessions` rows filtered `WHERE engagement_id = $X`.

12. **MUST** propagate JWT blacklist updates across gateway nodes within 50ms p95 per DEC-1085. The Redis pub/sub mechanism:
    - Sync-phase publish: `PUBLISH jwt_blacklist_update <jti_list_json>`.
    - Each gateway node subscribes; on receive, updates in-memory cache.
    - Cache miss falls back to Redis SISMEMBER (~1ms) then Postgres SELECT (~5ms).
    - End-to-end visibility: < 100ms p99 for blacklist effect.

13. **MUST** maintain `portal_deprovision_log` table at migration `0009`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, engagement_id UUID NOT NULL, subject_id UUID NOT NULL, action TEXT NOT NULL CHECK (action IN ('soft_tombstone','hard_purge','restore','grace_undelete')), initiated_at TIMESTAMPTZ NOT NULL DEFAULT now(), completed_at TIMESTAMPTZ, sync_phase_duration_ms INT, async_phase_duration_ms INT, cascade_targets JSONB NOT NULL, sla_breach BOOLEAN NOT NULL DEFAULT false)`. Append-only. RLS scoped to `(tenant_id, engagement_id)`.

14. **MUST** maintain `portal_restore_requests` table at migration `0011`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, engagement_id UUID NOT NULL, subject_id UUID NOT NULL, requested_by_subject_id UUID NOT NULL, reason TEXT NOT NULL, requested_at TIMESTAMPTZ NOT NULL DEFAULT now())`. Append-only.

15. **MUST** rate-limit SCIM DELETE at 1000/min/Engagement per DEC-1091 — enterprise org-wide cleanup scenarios. Excess returns `429 + Retry-After: 60`.

16. **MUST** run nightly reconciliation per DEC-1097 + DEC-1094. The `reconciliation.rs::run_nightly()`:
    - SELECT subjects WHERE status='tombstoned' AND tombstoned_at < now() - interval '1 day'.
    - For each: check `auth_sessions WHERE subject_id=$1 AND revoked_at IS NULL` should return 0 rows.
    - Any active session for a tombstoned subject = sev-1 `portal.scim_orphan_session_detected` (not in 7-kind core list per DEC-1088; informational+forensic).
    - Auto-revoke the orphan + audit; emit one row per orphan.

17. **MUST** emit 7 memory audit row kinds per DEC-1088:
    - `portal.scim_user_deprovisioned` (sev-1 — material identity event)
    - `portal.scim_user_restored` (sev-1 — counter-event)
    - `portal.scim_cascade_revocation` (sev-2 — one per cascade target)
    - `portal.jwt_blacklist_propagated` (sev-3 — high-volume; sampled at 1%)
    - `portal.websocket_force_closed` (sev-3 — high-volume; sampled at 10%)
    - `portal.scim_grace_period_expired` (sev-3 — informational)
    - `portal.scim_deprovision_sla_breach` (sev-1 — SLO miss)

18. **MUST** PII-scrub audit rows per DEC-1090 + feature-request-audit skill rule 18. `subject_id` UUID retained (forensic anti-correlation needed); `email` PII-scrubbed via FR-MEMORY-111 → `email_hash16`.

19. **MUST** thread W3C `traceparent` across SCIM DELETE → sync phase → async phase → cascade targets → reconciliation (feature-request-audit skill rule 22-24). Single trace_id per deprovision operation.

20. **MUST** measure deprovision SLO via OTel histogram `portal_scim_deprovision_duration_seconds`. Alarm sev-1 if `p99 > 30s sustained 5 min`.

21. **MUST NOT** hard-purge by default per DEC-1082. The `?action=hard_purge` query param is reserved for slice 3 GDPR Art. 17 explicit erasure flow (FR-AUTH-2xx); slice 2 returns 501 `not_implemented` on hard_purge.

22. **MUST NOT** block sync phase on cascade completion per DEC-1096. Sync phase MUST return 204 within 1s; cascade async via `tokio::spawn` with deadline.

23. **MUST NOT** support webhook-based deprovision at slice 2 per DEC-1098. SCIM DELETE is the canonical deprovision channel; webhook-fan-in deferred.

24. **SHOULD** observe per-Engagement deprovision-volume metric `portal_scim_deprovision_total{engagement_id}`. Sudden spike (>100/min sustained) may indicate IdP-side misconfig + warrant operator attention.

---

## §2 — Why this design (rationale for humans)

**Why 30s SLO (§1 #1, DEC-1080)?** Enterprise procurement asks "how quickly does a terminated employee lose access?" The industry-standard answer is "≤ 60 seconds" (Okta + Azure benchmarks); 30s is competitive + achievable with the dual-channel architecture (JWT blacklist + WebSocket close). Anything slower = procurement objection.

**Why dual-channel kill (§1 #5 + §1 #7, DEC-1081)?** Stateless REST APIs use JWT bearer auth — JWT blacklist catches those. Long-lived WebSocket connections (CHAT, Genie) don't re-validate JWTs every message — they need explicit close frames. Both channels must be killed; missing either leaks access.

**Why sync-then-async phase split (§1 #3 + §1 #4, DEC-1096)?** SCIM clients (Okta, Azure) expect SCIM DELETE to return 204 quickly. Doing all the work (blacklist + cascade + WS close) synchronously would push response time to 5-10 seconds — SCIM clients time out at 30s default + Okta retries on timeout → duplicate deprovisions. Sync phase does the critical 1s work (tombstone + blacklist add); async phase handles the rest within the 30s SLO end-to-end.

**Why grace period 5 min (§1 #8, DEC-1083)?** IdP-side glitches happen — Okta + Azure both have documented incidents where SCIM DELETE fires spuriously due to integration bugs. Without a grace, the affected user is locked out + must be re-onboarded from scratch (loses CHAT history, Genie context, etc.). 5 min is enough to catch + reverse the glitch; short enough that an attacker who tricked IdP into firing DELETE can't undo from the user's side (user lacks SCIM PUT access; only IdP can re-add).

**Why admin restore endpoint (§1 #9, DEC-1092)?** Grace period covers minutes-after-DELETE. Beyond grace, restore requires admin intent (tenant_admin role). Engagement_admin too narrow (restore reaches across Engagement scope — Genie subscriptions span Engagements).

**Why per-Engagement isolation (§1 #11, DEC-1086)?** A consulting user may belong to Engagement X (Client A) + Engagement Y (Client B). Client A firing user via their SCIM endpoint must NOT terminate user's access to Client B. Per-Engagement scoping is the only correct semantic.

**Why Redis blacklist + Postgres durable store (§1 #2 + §1 #7, DEC-1085)?** Redis = fast (1ms lookups at scale); Postgres = durable (Redis is in-memory + can lose data on restart). Redis is the hot cache; Postgres is the source of truth. Cache-miss-recovery is fast (~5ms Postgres query); Redis pub/sub propagates new blacklist entries across all gateway nodes within ~50ms.

**Why nightly reconciliation (§1 #16, DEC-1097)?** Cascade revocation is best-effort — distributed system, partial failures possible. Reconciliation closes the loop: any tombstoned subject with still-active sessions = bug, auto-fix + alert.

**Why WebSocket close code 4001 (§1 #5, DEC-1095)?** RFC 6455 reserves 4000-4999 for application-defined codes. 4001 is our convention for "session revoked"; clients SHOULD reconnect (will fail re-auth + see clear 401). Custom code lets client UI surface a "you've been signed out by your admin" message rather than a generic "connection lost".

**Why subject_id NOT scrubbed in audit (§1 #18, DEC-1090)?** Identity events are forensic-critical. Without subject_id, "who was deprovisioned at 09:14?" is unanswerable. UUID is non-PII (random + non-correlatable to external identity); email IS PII (correlatable to person + service). Differential scrub.

**Why soft tombstone default, hard purge gated (§1 #21, DEC-1082)?** GDPR Art. 17 erasure is a heavyweight legal flow (verify request, document, propagate to processors, attest). Most deprovisions are operational ("employee left") not legal ("erase me"); soft tombstone preserves audit history without violating retention obligations. Hard purge is reserved for the explicit GDPR flow (FR-AUTH-2xx slice 3).

**Why SCIM-only at slice 2 (§1 #23, DEC-1098)?** Multi-channel deprovision (SCIM + webhook + admin UI) compounds correctness risk (which channel wins on simultaneous events?). SCIM is the standard; we stick to it at slice 2. Webhook deprovision deferred.

---

## §3 — API contract

### 3.1 Postgres schema

```sql
-- 0009_portal_deprovision_log.sql
CREATE TABLE portal_deprovision_log (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  subject_id UUID NOT NULL,
  action TEXT NOT NULL CHECK (action IN ('soft_tombstone','hard_purge','restore','grace_undelete')),
  initiated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  completed_at TIMESTAMPTZ,
  sync_phase_duration_ms INT,
  async_phase_duration_ms INT,
  cascade_targets JSONB NOT NULL DEFAULT '[]'::jsonb,
  sla_breach BOOLEAN NOT NULL DEFAULT false,
  trace_id CHAR(32)
);
CREATE INDEX idx_deprovision_subject ON portal_deprovision_log(subject_id, initiated_at DESC);
ALTER TABLE portal_deprovision_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_deprovision_log_rls ON portal_deprovision_log
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_deprovision_log FROM cyberos_app;
GRANT UPDATE (completed_at, sync_phase_duration_ms, async_phase_duration_ms, cascade_targets, sla_breach)
  ON portal_deprovision_log TO cyberos_app;

-- 0010_portal_jwt_blacklist.sql
CREATE TABLE portal_jwt_blacklist (
  jti UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  subject_id UUID NOT NULL,
  engagement_id UUID,
  blacklisted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL,
  reason TEXT NOT NULL
);
CREATE INDEX idx_jwt_blacklist_subject ON portal_jwt_blacklist(subject_id, blacklisted_at DESC);
CREATE INDEX idx_jwt_blacklist_expiry ON portal_jwt_blacklist(expires_at);
ALTER TABLE portal_jwt_blacklist ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_jwt_blacklist_rls ON portal_jwt_blacklist
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_jwt_blacklist FROM cyberos_app;
GRANT DELETE ON portal_jwt_blacklist TO cyberos_pruner;  -- auto-trim past expires_at

-- 0011_portal_restore_requests.sql
CREATE TABLE portal_restore_requests (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  subject_id UUID NOT NULL,
  requested_by_subject_id UUID NOT NULL,
  reason TEXT NOT NULL,
  requested_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE portal_restore_requests ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_restore_requests_rls ON portal_restore_requests
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_restore_requests FROM cyberos_app;
```

### 3.2 Rust types

```rust
// services/portal/src/deprovision/mod.rs
#[derive(Debug, serde::Serialize)]
pub struct DeprovisionResult {
    pub subject_id: Uuid,
    pub engagement_id: Uuid,
    pub sync_phase_completed_at: DateTime<Utc>,
    pub async_phase_completed_at: Option<DateTime<Utc>>,
    pub sla_breach: bool,
    pub cascade_targets_completed: Vec<CascadeTarget>,
}

#[derive(Debug, serde::Serialize)]
pub enum CascadeTarget { GenieSessions, ScopedViews, ChannelMemberships, McpConfirmations }
```

### 3.3 REST endpoints

```text
DELETE /scim/v2/{engagement_slug}/Users/{id}                        (SCIM bearer)
PUT    /scim/v2/{engagement_slug}/Users/{id}                        (SCIM bearer; grace-period reactivate)
POST   /v1/admin/engagements/{eng_id}/subjects/{subject_id}/restore (tenant_admin)
GET    /v1/admin/tenants/{tenant_id}/deprovision-log                (tenant_admin or cfo)
```

---

## §4 — Acceptance criteria

1. **SCIM DELETE returns 204 < 1s** — DELETE handler sync phase completes within 1 second wall-clock.
2. **JWT invalidated within 30s** — post-DELETE, REST API call with the user's prior JWT returns 401 `jwt_revoked` within 30 seconds end-to-end.
3. **WebSocket force-closed** — active WS connection receives close frame code 4001 + reason `scim_deprovision`.
4. **Cascade to Genie sessions** — user's `portal_genie_sessions` rows updated with `revoked_at`.
5. **Cascade to channel memberships** — user's `chat_channel_memberships` updated with `revoked_at`.
6. **Cascade to MCP pending confirmations** — user's `mcp_pending_confirmations` deleted.
7. **Idempotent re-DELETE** — DELETE on already-tombstoned subject returns 204 + no audit re-emit.
8. **Idempotent re-DELETE post-purge** — DELETE on hard-purged subject returns 404 `noTarget`.
9. **Grace-period SCIM PUT reactivates** — PUT within 5 min restores subject + reverses blacklist + emits `portal.scim_user_restored`.
10. **Beyond-grace SCIM PUT rejected** — PUT 6 min post-DELETE returns 409 `grace_period_expired`.
11. **Admin restore tenant_admin only** — engagement_admin POST restore returns 403; tenant_admin returns 200 + audit row.
12. **Per-Engagement isolation** — user member of Eng X + Y; DELETE in X revokes only X's sessions; Y sessions still valid.
13. **Rate-limit 1000/min/Engagement** — 1001st DELETE in 60s returns 429.
14. **JWT blacklist propagation < 50ms p95** — multi-node test: blacklist add on node A is visible to node B within 50ms.
15. **30s SLO breach alerted** — fixture-injected delay > 30s triggers `portal.scim_deprovision_sla_breach` sev-1.
16. **Nightly reconciliation catches orphans** — fixture-tombstoned subject with active session → reconciliation auto-revokes + emits sev-1 orphan row.
17. **7 memory audit kinds emitted** — full lifecycle covers all 7 (deprovisioned + restored + cascade × 4 targets + propagated + force_closed + grace_expired + sla_breach).
18. **PII scrub** — audit row carries `email_hash16` not raw email; `subject_id` UUID retained.
19. **Hard purge returns 501 at slice 2** — `?action=hard_purge` query param returns 501 `not_implemented`.
20. **Trace_id end-to-end** — single trace_id across SCIM DELETE + sync phase + async phase + cascade row + reconciliation row.

---

## §5 — Verification

### 5.1 `scim_delete_invalidates_jwt_test.rs`

```rust
#[tokio::test]
async fn jwt_invalidated_within_30s() {
    let ctx = TestContext::with_engagement_subject_with_jwt("alice@acme.com").await;
    let jwt = ctx.subject_jwt.clone();

    // pre-DELETE: JWT works
    let r0 = ctx.get("/v1/portal/profile").bearer_auth(&jwt).send().await.unwrap();
    assert_eq!(r0.status(), 200);

    // SCIM DELETE
    let start = Instant::now();
    let r1 = ctx.scim_delete_user(ctx.subject_id).await;
    assert_eq!(r1.status(), 204);
    assert!(start.elapsed() < Duration::from_secs(1));

    // poll until JWT rejected (max 30s)
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let r = ctx.get("/v1/portal/profile").bearer_auth(&jwt).send().await.unwrap();
        if r.status() == 401 { break; }
        if Instant::now() >= deadline { panic!("JWT still valid after 30s"); }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

### 5.2 `scim_delete_closes_websocket_test.rs`

```rust
#[tokio::test]
async fn websocket_force_closed_on_deprovision() {
    let ctx = TestContext::with_engagement_subject_with_jwt("alice@acme.com").await;
    let mut ws = ctx.open_chat_websocket(&ctx.subject_jwt).await;
    ctx.scim_delete_user(ctx.subject_id).await;
    let close_event = tokio::time::timeout(Duration::from_secs(30), ws.next_close_frame()).await
        .expect("WS close frame not received within 30s");
    assert_eq!(close_event.code, 4001);
    assert_eq!(close_event.reason, "scim_deprovision");
}
```

### 5.3 `scim_delete_cascade_test.rs`

```rust
#[tokio::test]
async fn cascade_revokes_all_4_targets() {
    let ctx = TestContext::with_full_session("alice@acme.com").await;  // creates Genie + CHAT + MCP confirms
    ctx.scim_delete_user(ctx.subject_id).await;
    tokio::time::sleep(Duration::from_secs(2)).await;  // allow async phase

    let genie_revoked: Option<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT revoked_at FROM portal_genie_sessions WHERE subject_id=$1"
    ).bind(ctx.subject_id).fetch_one(&ctx.pool).await.unwrap();
    assert!(genie_revoked.is_some());

    let chat_memberships: Vec<Option<DateTime<Utc>>> = sqlx::query_scalar(
        "SELECT revoked_at FROM chat_channel_memberships WHERE subject_id=$1"
    ).bind(ctx.subject_id).fetch_all(&ctx.pool).await.unwrap();
    assert!(chat_memberships.iter().all(|r| r.is_some()));

    let mcp_confirms: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM mcp_pending_confirmations WHERE caller_subject_id=$1"
    ).bind(ctx.subject_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(mcp_confirms, 0);
}
```

### 5.4 `scim_delete_30s_sla_test.rs`

```rust
#[tokio::test]
async fn 30s_sla_p99_under_load() {
    let ctx = TestContext::with_n_subjects(100).await;
    let mut durations = Vec::new();
    for subject_id in ctx.subject_ids.iter() {
        let start = Instant::now();
        ctx.scim_delete_user(*subject_id).await;
        ctx.wait_until_fully_revoked(*subject_id).await;
        durations.push(start.elapsed());
    }
    durations.sort();
    let p99 = durations[(durations.len() as f64 * 0.99) as usize];
    assert!(p99 < Duration::from_secs(30), "actual p99 {:?}", p99);
}
```

### 5.5 `scim_grace_period_undelete_test.rs`

```rust
#[tokio::test]
async fn put_within_grace_reactivates() {
    let ctx = TestContext::with_engagement_subject_with_jwt("alice@acme.com").await;
    ctx.scim_delete_user(ctx.subject_id).await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    let r = ctx.scim_put_user(ctx.subject_id, json!({"userName": "alice"})).await;
    assert_eq!(r.status(), 200);

    let status: String = sqlx::query_scalar("SELECT status FROM subjects WHERE id=$1")
        .bind(ctx.subject_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(status, "active");

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "portal.scim_user_restored"
        && r.payload["restored_via"] == "scim_put_within_grace"));
}

#[tokio::test]
async fn put_beyond_grace_rejected() {
    let ctx = TestContext::with_engagement_subject_with_jwt("alice@acme.com").await;
    ctx.scim_delete_user(ctx.subject_id).await;
    ctx.travel_clock_forward(Duration::from_secs(360)).await;  // 6 min

    let r = ctx.scim_put_user(ctx.subject_id, json!({"userName": "alice"})).await;
    assert_eq!(r.status(), 409);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "grace_period_expired");
}
```

### 5.6 `scim_admin_restore_test.rs`

```rust
#[tokio::test]
async fn admin_restore_tenant_admin_only() {
    let ctx = TestContext::with_engagement_subject_with_jwt("alice@acme.com").await;
    ctx.scim_delete_user(ctx.subject_id).await;
    ctx.travel_clock_forward(Duration::from_secs(360)).await;

    let eng_admin_jwt = ctx.mint_jwt_with_role(ctx.tenant, "engagement_admin");
    let r = ctx.post(&format!("/v1/admin/engagements/{}/subjects/{}/restore", ctx.eng_id, ctx.subject_id))
        .bearer_auth(eng_admin_jwt).json(&json!({"reason": "reinstated"})).send().await.unwrap();
    assert_eq!(r.status(), 403);

    let tenant_admin_jwt = ctx.mint_jwt_with_role(ctx.tenant, "tenant_admin");
    let r = ctx.post(&format!("/v1/admin/engagements/{}/subjects/{}/restore", ctx.eng_id, ctx.subject_id))
        .bearer_auth(tenant_admin_jwt).json(&json!({"reason": "reinstated"})).send().await.unwrap();
    assert_eq!(r.status(), 200);
}
```

### 5.7 `scim_per_engagement_isolation_test.rs`

```rust
#[tokio::test]
async fn delete_in_one_engagement_preserves_other() {
    let ctx = TestContext::with_subject_in_two_engagements("alice@acme.com", "eng-x", "eng-y").await;
    let jwt_x = ctx.mint_jwt_for(ctx.subject_id, ctx.eng_x_id).await;
    let jwt_y = ctx.mint_jwt_for(ctx.subject_id, ctx.eng_y_id).await;

    ctx.scim_delete_user_in_engagement(ctx.subject_id, "eng-x").await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    let r_x = ctx.get("/v1/portal/profile").bearer_auth(&jwt_x).send().await.unwrap();
    assert_eq!(r_x.status(), 401);  // X-scoped JWT revoked
    let r_y = ctx.get("/v1/portal/profile").bearer_auth(&jwt_y).send().await.unwrap();
    assert_eq!(r_y.status(), 200);  // Y-scoped JWT still valid
}
```

### 5.8 `scim_orphan_reconciliation_test.rs`

```rust
#[tokio::test]
async fn nightly_reconciliation_catches_orphan_session() {
    let ctx = TestContext::with_engagement_subject_with_jwt("alice@acme.com").await;
    // Tombstone subject without going through normal SCIM (simulate cascade failure)
    sqlx::query("UPDATE subjects SET status='tombstoned', tombstoned_at=now() WHERE id=$1")
        .bind(ctx.subject_id).execute(&ctx.pool).await.unwrap();
    // Session row remains active (orphan)

    ctx.travel_clock_forward(Duration::from_days(1)).await;
    ctx.run_reconciliation_job().await;

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "portal.scim_orphan_session_detected" && r.severity == 1));
    let session_revoked: Option<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT revoked_at FROM auth_sessions WHERE subject_id=$1"
    ).bind(ctx.subject_id).fetch_one(&ctx.pool).await.unwrap();
    assert!(session_revoked.is_some());
}
```

### 5.9 `scim_jwt_blacklist_propagation_test.rs`

```rust
#[tokio::test]
async fn blacklist_propagates_under_50ms_p95() {
    let ctx = TestContext::with_two_gateway_nodes().await;
    let subject = ctx.provision_subject("alice@acme.com").await;
    let jwt = ctx.mint_jwt_for(subject, ctx.eng_id).await;

    let mut latencies = Vec::new();
    for _ in 0..100 {
        ctx.add_to_blacklist_via_node_a(&jwt).await;
        let start = Instant::now();
        loop {
            let r = ctx.get_via_node_b("/v1/portal/profile").bearer_auth(&jwt).send().await.unwrap();
            if r.status() == 401 { latencies.push(start.elapsed()); break; }
            if start.elapsed() > Duration::from_millis(500) { panic!("propagation > 500ms"); }
        }
        ctx.remove_from_blacklist(&jwt).await;
    }
    latencies.sort();
    let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];
    assert!(p95 < Duration::from_millis(50));
}
```

### 5.10 `scim_audit_emission_test.rs`

```rust
#[tokio::test]
async fn full_lifecycle_emits_7_kinds() {
    let ctx = TestContext::with_full_session("alice@acme.com").await;
    ctx.scim_delete_user(ctx.subject_id).await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    ctx.travel_clock_forward(Duration::from_secs(360)).await;
    let _ = ctx.scim_put_user(ctx.subject_id, json!({"userName":"alice"})).await;  // grace expired
    ctx.admin_restore(ctx.subject_id).await;

    let kinds: Vec<&str> = ctx.memory_rows().await.iter().map(|r| r.kind.as_str()).collect();
    for expected in &[
        "portal.scim_user_deprovisioned", "portal.scim_user_restored",
        "portal.scim_cascade_revocation", "portal.jwt_blacklist_propagated",
        "portal.websocket_force_closed", "portal.scim_grace_period_expired",
    ] {
        assert!(kinds.contains(expected), "missing {expected}");
    }
}
```

---

## §6 — Implementation skeleton

### 6.1 Deprovision orchestrator (sync + async split)

```rust
// services/portal/src/deprovision/mod.rs
pub async fn run(ctx: AppCtx, engagement_id: Uuid, subject_id: Uuid) -> Result<(), DeprovError> {
    let start = Instant::now();
    let trace_id = otel_trace_id();

    // Sync phase < 1s
    let sync_start = Instant::now();
    sync_phase::execute(&ctx, subject_id, engagement_id, &trace_id).await?;
    let sync_ms = sync_start.elapsed().as_millis() as i32;

    // Insert deprovision log row at sync-phase completion
    let log_id = ctx.repo.deprovision_log.insert(
        engagement_id, subject_id, "soft_tombstone", sync_ms, &trace_id
    ).await?;

    // Async phase via tokio::spawn with 30s deadline
    let async_ctx = ctx.clone();
    tokio::spawn(async move {
        let async_start = Instant::now();
        let result = tokio::time::timeout(
            Duration::from_secs(30),
            async_phase::execute(&async_ctx, subject_id, engagement_id, &trace_id),
        ).await;
        let async_ms = async_start.elapsed().as_millis() as i32;
        let breach = result.is_err() || async_start.elapsed() > Duration::from_secs(30);
        async_ctx.repo.deprovision_log.complete(log_id, async_ms, breach).await.ok();
        if breach {
            emit_audit(&async_ctx, "portal.scim_deprovision_sla_breach", json!({
                "subject_id": subject_id, "engagement_id": engagement_id,
                "duration_ms": async_ms,
            })).await;
        }
    });

    // Caller (SCIM handler) returns 204 immediately after sync phase
    Ok(())
}
```

### 6.2 JWT blacklist (Redis + Postgres)

```rust
// services/portal/src/deprovision/jwt_blacklist.rs
pub async fn add(ctx: &AppCtx, jti: Uuid, tenant: Uuid, subject: Uuid, eng: Uuid, expires_at: DateTime<Utc>) -> Result<(), Err> {
    sqlx::query("INSERT INTO portal_jwt_blacklist (jti, tenant_id, subject_id, engagement_id, expires_at, reason)
                 VALUES ($1, $2, $3, $4, $5, 'scim_deprovision')
                 ON CONFLICT (jti) DO NOTHING")
        .bind(jti).bind(tenant).bind(subject).bind(eng).bind(expires_at)
        .execute(&ctx.pool).await?;

    let mut conn = ctx.redis.get().await?;
    let score = expires_at.timestamp() as f64;
    conn.zadd::<_, _, _, ()>(format!("jwt_blacklist:{tenant}"), jti.to_string(), score).await?;
    conn.publish::<_, _, ()>("jwt_blacklist_update", serde_json::to_string(&jti)?).await?;
    Ok(())
}

pub async fn is_blacklisted(ctx: &AppCtx, jwt_claims: &JwtClaims) -> Result<bool, Err> {
    let mut conn = ctx.redis.get().await?;
    let in_redis: bool = conn.zscore::<_, _, Option<f64>>(
        format!("jwt_blacklist:{}", jwt_claims.tenant_id),
        jwt_claims.jti.to_string()
    ).await?.is_some();
    if in_redis { return Ok(true); }
    // Postgres fallback (Redis cold cache)
    let pg_hit: Option<(Uuid,)> = sqlx::query_as(
        "SELECT jti FROM portal_jwt_blacklist WHERE jti=$1 AND expires_at > now()"
    ).bind(jwt_claims.jti).fetch_optional(&ctx.pool).await?;
    if pg_hit.is_some() {
        // backfill Redis
        let _ = conn.zadd::<_, _, _, ()>(
            format!("jwt_blacklist:{}", jwt_claims.tenant_id),
            jwt_claims.jti.to_string(), jwt_claims.exp as f64
        ).await;
    }
    Ok(pg_hit.is_some())
}
```

### 6.3 WebSocket killer

```rust
// services/portal/src/deprovision/websocket_killer.rs
pub async fn close_for_subject(ctx: &AppCtx, subject_id: Uuid, engagement_id: Uuid) -> Vec<Uuid> {
    let connections = ctx.ws_registry.list_for_subject(subject_id, engagement_id).await;
    let mut closed = Vec::new();
    for conn in connections {
        let close = CloseFrame { code: 4001, reason: "scim_deprovision".into() };
        if conn.send_close(close).await.is_ok() {
            closed.push(conn.id);
        }
    }
    closed
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **FR-PORTAL-003** External IdP + SCIM JIT — SCIM Users endpoint extended here for DELETE.

**Cross-module (related_frs):**
- **FR-PORTAL-001** Scoped views — cascade target.
- **FR-PORTAL-005** Branded Genie chat — cascade target.
- **FR-AUTH-002** Subject create — subject tombstone semantics.
- **FR-AUTH-004** JWT mint + validate — blacklist consulted at validate.
- **FR-AUTH-101** RBAC — role gates.
- **FR-TEN-104** Lifecycle — deprovision may trigger if last admin lost.
- **FR-CHAT-005** CHAT membership — cascade target.
- **FR-AI-003** memory audit-row bridge — 7 new kinds.
- **FR-MEMORY-111** PII scrubbing — email hash in chain.
- **FR-OBS-007** Auto-runbook — sev-1 SLA-breach + orphan-session alerts route to CHAT.

**Downstream (blocks):** None.

---

## §8 — Example payloads

### 8.1 `portal.scim_user_deprovisioned` memory row

```json
{
  "kind": "portal.scim_user_deprovisioned",
  "severity": 1,
  "tenant_id": "8a2f...",
  "actor_id": "system.portal.scim",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "engagement_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001",
    "subject_id": "7c4e9a2b-1d3f-4e6a-bccc-000000000042",
    "email_hash16": "f8a1b2c3d4e5f607",
    "external_id_hash16": "9c4e7a8b6d2f1e3a",
    "action": "soft_tombstone",
    "sync_phase_duration_ms": 87,
    "active_jwts_blacklisted_count": 3,
    "active_websockets_to_close_count": 2,
    "grace_period_until": "2026-05-17T09:19:32.847Z"
  }
}
```

### 8.2 `portal.scim_deprovision_sla_breach` memory row

```json
{
  "kind": "portal.scim_deprovision_sla_breach",
  "severity": 1,
  "tenant_id": "8a2f...",
  "actor_id": "system.portal.scim",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:15:05.118Z",
  "payload": {
    "subject_id": "7c4e...",
    "engagement_id": "0190...",
    "actual_duration_ms": 34218,
    "sla_ms": 30000,
    "cascade_targets_pending": ["genie_sessions"],
    "remediation": "nightly_reconciliation_will_clean_up"
  }
}
```

### 8.3 Admin restore request

```json
{
  "reason": "Wrong user deprovisioned by Azure AD glitch; user verified active employee"
}
```

### 8.4 WebSocket close frame

```text
WebSocket Close frame:
  Code: 4001
  Reason: "scim_deprovision"

Client receives + should reconnect; re-auth fails with 401 jwt_revoked.
Client UI: "You've been signed out by your administrator."
```

---

## §9 — Open questions

All resolved for slice 2. Deferred:

- **Deferred:** Hard purge (GDPR Art. 17) flow — slice 3, FR-AUTH-2xx (placeholder).
- **Deferred:** Webhook-based deprovision (non-SCIM channel) — slice 3.
- **Deferred:** Multi-actor deprovision approval (M-of-N for executive accounts) — slice 3.
- **Deferred:** Deprovision dry-run mode (preview cascade impact) — slice 3.
- **Deferred:** SCIM Bulk endpoint (RFC 7644 §3.7) — slice 3, FR-PORTAL-3xx.
- **Deferred:** Per-Engagement grace period override (some Engagements want 0 min, some want 30 min) — slice 3.
- **Deferred:** Deprovision report export (PDF for compliance) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Sync phase exceeds 1s | OTel histogram alert | 204 still returned; sev-2 alert | Operator investigates DB/Redis latency |
| Async phase exceeds 30s | deadline timeout | `sla_breach=true` in log row; sev-1 audit | Nightly reconciliation cleans up; investigate cause |
| Redis pub/sub propagation > 50ms | latency metric | Gateway nodes catch up via SISMEMBER fallback; sev-2 alert | Redis cluster investigation |
| Postgres unavailable during blacklist write | sqlx error | Redis-only blacklist works (best-effort); sev-1 alert; nightly reconciliation re-syncs to Postgres | DB recovery |
| Cascade target throws (e.g. CHAT service down) | per-target error caught | Other targets proceed; failed target retried via reconciliation | CHAT recovers; reconciliation re-attempts |
| WebSocket close fails (connection already broken) | send_close returns Err | Logged; counts as success (no-op) | Inherent |
| SCIM PUT within grace race with concurrent admin restore | tx isolation | Last writer wins; both emit audit rows; idempotent | Inherent |
| Hard_purge requested at slice 2 | query param check | 501 + `not_implemented_slice_2` | Tenant_admin uses GDPR flow when shipped |
| Re-DELETE on tombstoned subject | status check | 204 + no re-emit | Inherent idempotency |
| Re-DELETE on purged subject | status check | 404 + `noTarget` | Inherent |
| Per-Engagement isolation bypass (cross-engagement blacklist) | scope check | Sev-1 alert; engagement-other JWTs unaffected | Investigate handler; fix |
| Orphan session detected at reconciliation | nightly job | Auto-revoke + sev-1 audit | Inherent |
| Reconciliation job itself fails | OBS alert | Sev-1; manual reconciliation via CLI | Operator runs reconciliation manually |
| Rate-limit hit (1000/min/Eng) | counter | 429 + Retry-After | Caller backs off; legitimate org-wide cleanup spread |
| Multi-node propagation race (node A blacklists, node B serves old JWT briefly) | latency window | < 50ms p95 window; acceptable | Inherent |
| Restore endpoint called on non-tombstoned subject | status check | 409 + `not_tombstoned` | Caller verifies state first |
| Restore endpoint called on hard-purged subject | status check | 404 + `noTarget` | Hard purge is one-way |
| WebSocket close code 4001 misinterpreted by client | client-side bug | Client may retry; will fail 401 + see clearer error | Client UI improvements |
| Grace-period clock skew across nodes | NTP-bounded | < 1s skew; 5-min grace tolerates | Inherent |
| MCP confirmation cascade race (concurrent confirm + delete) | atomic SELECT FOR UPDATE | First writer wins; second blocked | Inherent |
| `cyberos_pruner` role not present for blacklist auto-trim | migration gate | Trimming via cyberos_app role at degraded mode | Operator adds role |
| Forced WS close on a paused connection | TCP-level retransmit | Eventually delivered; close still effective | Inherent |
| Audit row insert fails post-deprovision | Postgres error | Subject still tombstoned (sync phase committed); sev-2 alert; OBS log retains forensic trace | Operator backfills audit if needed |
| Foreign-Engagement JWT presented during deprovision window | scope check at validator | Correct rejection (foreign JWT was already invalid in this Engagement) | Inherent |

---

## §11 — Implementation notes

**§11.1** Redis sorted-set storage for blacklist — auto-pruning past expires_at via `ZREMRANGEBYSCORE` scheduled task hourly.

**§11.2** WebSocket close code 4001 — chosen from RFC 6455 application-reserved range (4000-4999); documented in our public API docs so client libraries can handle it.

**§11.3** Tombstone vs delete: subjects table has a `status` column with `tombstoned` value; row remains for audit forensic (and possible grace-period restore).

**§11.4** Grace period clock-skew tolerance: 5-min window with up to 1s NTP skew across nodes is safe; clients shouldn't be cutting it close.

**§11.5** The async cascade phase uses `futures::future::join_all` for parallel target invocation; failures per-target are logged but don't abort other targets.

**§11.6** Reconciliation job runs nightly at 03:00 UTC; light query (subject status='tombstoned' + active session count); typically < 30s for 10k tombstoned subjects.

**§11.7** PII hash function: `email_hash16 = encode(substring(digest(lower(email) || global_salt, 'sha256') from 1 for 8), 'hex')` — matches FR-TEN-101 + FR-PORTAL-003 patterns.

**§11.8** The `auth.subjects.status` enum is shared with FR-AUTH-002; this FR adds `tombstoned` value via migration referenced in modified_files.

**§11.9** `auth_sessions` table also shared with FR-AUTH-004; column `revoked_at` + `revoked_reason` added via migration; FR-AUTH-004 modified to set them.

**§11.10** Per-Engagement JWT scoping: each JWT carries `engagement_id` claim (FR-PORTAL-003 derivative); blacklist scoped to `(tenant_id, engagement_id)` partition.

**§11.11** Cascade target ordering: parallel (no order); each target is idempotent; safe to retry on partial failure.

**§11.12** The reconciliation job's missing-cascade auto-fix triggers another audit row (`portal.scim_cascade_revocation` via reconciliation path); deduped by `(subject_id, target)` last-30d.

**§11.13** Restore endpoint reactivates subject status + clears blacklist + restores cascade — but does NOT re-open closed WebSocket connections (impossible; client must re-connect).

**§11.14** The 30s SLO is end-to-end (SCIM DELETE accept → last cascade complete); intermediate phases have their own SLOs (sync 1s, blacklist propagation 50ms p95).

**§11.15** Hard_purge query param reserved for slice 3 GDPR flow; rejecting with 501 makes the missing capability visible (vs silent fall-through to soft tombstone).

**§11.16** Restore via admin endpoint is rate-limited at 10/min/tenant (default; tenant_admin can tune in slice 3) — unusual operation, should not be high-volume.

**§11.17** Per-Engagement audit-log RLS prevents one Engagement's deprovision history from being read by another Engagement's tenant_admin (cross-Engagement read leak is a regulatory concern).

**§11.18** WebSocket killer iterates Redis-backed connection registry (multi-node) for completeness; missing one node's connections = blacklist still catches at next message (JWT validate hits blacklist).

**§11.19** Trace_id propagation: SCIM DELETE handler generates trace_id; passes via `tracing::instrument` context into sync phase + async spawn + cascade targets; all audit rows carry the same trace_id.

**§11.20** The 7-kind core list (§1 #17) plus `portal.scim_orphan_session_detected` (§1 #16) totals 8 memory audit kinds for this FR; FR-AI-003 closed-set extension covers all 8.

**§11.21** Per-PSP-like multi-tenant abstraction not needed here — SCIM is the single canonical channel; webhook-based deprovision deferred per DEC-1098.

---

*End of FR-PORTAL-004 spec.*
