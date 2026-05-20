---
fr_id: FR-TEN-004
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 9
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per feature-request-audit skill §0)
---

## §1 — Verdict summary

FR-TEN-004 ships the 4-axis metering substrate (seats, api_calls, ai_tokens, storage_bytes) with append-only Postgres + memory audit dual-write + per-tenant overage policy. Scope: 27 §1 normative clauses covering closed `metering_axis` 4-value enum + closed `metering_unit` 4-value enum + (axis, unit) CHECK-paired so api_calls always counted in requests, ai_tokens in tokens, etc. — append-only via SQL grant (REVOKE UPDATE/DELETE FROM cyberos_app) + privileged `metering_writer` role, idempotency via UNIQUE(tenant, axis, key) within 24h, dual-write Postgres COMMIT + memory audit (Postgres authoritative for billing, memory tamper-evident), API-call middleware emission on response path (≤200µs p99), AI-token emission at cost_ledger postcall reconcile, seats snapshot at period close (not entry/exit), storage snapshot from S3 inventory + pg_total_relation_size, closed 3-value `metering_overage_policy` (block | warn | allow) with CFO-only mutation + sev-2 audit, overage `block` returns 402 at middleware, overage `warn` emits sev-2 at threshold crossing, materialized `metering_current_period` view refreshed every 5 min via pg_cron CONCURRENTLY, `aggregator_state.last_aggregated_seq` cross-check before period freeze, period-freeze trigger rejects late inserts with P0203, correction_to self-FK with sign-opposite trigger + state machine (active → superseded → corrected for double-correction), bounded 100k-event in-memory WAL queue with 90% back-pressure threshold, 60s active-tenant cache for terminated-tenant rejection (404), dispute log append-only with billing_disputes role + 4 KiB resolution_notes cap + 4-value closed resolution_status, 7 memory audit kinds (event_recorded sev-3, warn_threshold_crossed sev-2, overage_blocked sev-2, correction_issued sev-2, policy_changed sev-2, reconciliation_divergence sev-1, wal_queue_overflow sev-1), all reason text scrubbed via FR-MEMORY-111 before chain emission, PII-free metric rows by design, per-axis quantity range CHECK constraints, `GET /v1/usage` hits materialized view with `last_refreshed_at` staleness column, ops-only period-close handler with bookkeeper role + per-tenant per-period scope. 22 rationale paragraphs. §3 contains: 3 migrations (metering_events with all 4 closed enums + grants + RLS + correction trigger + freeze trigger; metering_periods + aggregator_state + per-tenant policy column; materialized view with pg_cron schedule), Recorder API with active-tenant cache + WAL push, period-close handler with aggregator-lag verification + memory emission + freeze. 30 ACs. 32 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Free-string axis or unit (no closed enum + no pair constraint)
First-pass had `axis text` + `unit text` allowing buggy emit like `(api_calls, minute)`. Resolved: §1 #1 + DEC-704 + closed 4-value metering_axis enum + closed 4-value metering_unit enum + (axis, unit) CHECK pair + CI cardinality test asserts exactly (4, 4, 3, 3); AC #1 + #2 + #5.

### ISS-002 — UPDATE/DELETE permitted (billing dispute hole)
First-pass relied on handler discipline. Resolved: §1 #2 + DEC-702 + REVOKE UPDATE, DELETE FROM cyberos_app + privileged metering_writer + correction_to self-FK refund path (DEC-711); AC #3 + #22 + #23.

### ISS-003 — No dual-write tamper-evidence (Postgres-only allows operator rewrite)
First-pass had Postgres-only persistence. Resolved: §1 #4 + DEC-712 + memory_chain_hash NOT NULL on every row + period-close reconciliation diff detected as sev-1; AC #8 + clause #21.

### ISS-004 — Materialized view could lag past period freeze (under-bill)
First-pass closed period from stale MV. Resolved: §1 #24 + DEC-713 + aggregator_state.last_aggregated_seq tracked + close-period verifies `seq >= max(events.seq)`; AC #20.

### ISS-005 — Correction with same-sign quantity (no sign-opposite trigger)
First-pass allowed positive correction of positive parent. Resolved: §1 #17 + DEC-711 + correction trigger with P0202 sign-opposite check + state machine transition (active → superseded → corrected); AC #23 + #24.

### ISS-006 — Hot-path metering on request-side (latency burn + bills failed-auth)
Resolved: §1 #6 + DEC-707 + middleware emits on response path post status code + failed-auth never emits + ≤200µs p99 budget; AC #9 + #10.

### ISS-007 — Cross-tenant leak via missing RLS
Resolved: §1 #16 + ENABLE ROW LEVEL SECURITY + tenant_isolation policy USING + WITH CHECK + bookkeeper role SECURITY DEFINER for legitimate cross-tenant queries (audited sev-3); AC #7 + #29.

### ISS-008 — Idempotency-key unbounded (index bloat)
Resolved: §1 #3 + DEC-715 + UNIQUE partial index WHERE occurred_at > now() - INTERVAL '24 hours' + reuse allowed after 24h; AC #4.

### ISS-009 — Overage policy free-string + non-CFO mutation
Resolved: §1 #10 + DEC-710 + closed 3-value enum + cfo role gate + reason ≥10 chars + sev-2 audit on every change; AC #14.

## §3 — Resolution

All 9 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (4 closed axes × (axis, unit) CHECK pair × append-only SQL grant × idempotent UNIQUE 24h × dual-write Postgres+memory × middleware response-path emission × postcall AI-token hook × end-of-period seats/storage snapshot × closed overage policy with CFO-only × materialized view 5-min refresh × aggregator-seq lag-check at freeze × correction_to sign-opposite trigger + state machine × bounded WAL queue back-pressure × 60s active-tenant cache for terminated-tenant 404 × dispute log append-only × 7 memory audit kinds × FR-MEMORY-111 PII scrubbing × per-axis quantity range CHECK × period-freeze trigger), not by line targets.

---

*End of FR-TEN-004 audit.*
