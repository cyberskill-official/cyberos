---
task_id: TASK-CHAT-005
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 14
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per task-audit skill §0; ISS-007..014 added)
---

## §1 — Verdict summary

TASK-CHAT-005 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 25 §1 clauses (logical replication, publication, per-tenant bridge, event mapping, sync_class derivation, PII redact, trace propagation, latency tracking, lag alarm, LSN persistence, tenant scope, metrics, memory-down pause, dedup_key idempotency, pending_acks ordering, message_edited as new row, reactions/channelmembers/fileinfo rows, read-replica refusal, heartbeat row, SIGTERM drain, normalise specials before PII, pgoutput replay-safety, attachment redaction, explicit Sessions exclusion). 17 §2 rationale paragraphs. §3 contains Cargo.toml pinned set, init-bridge-publication.sql with role + view, main.rs with reconnect loop + signal handlers + replica safety check, replication.rs with exactly-once-modulo-dedup loop + pending_acks, map.rs covering all 10 event types + normalisation, lsn.rs with slot management + lag computation. 36 ACs. §5 contains 14 named test bodies covering happy/delete/edit/reactions/channelmembers/PII/sync_class/read-replica refusal/heartbeat/sigterm-drain/specials-normalisation/Sessions-excluded/dedup_key-determinism/LSN-pause/scope-isolation/WAL-bloat. §6 deepens with 11 wiring subsections (process model, slot naming, reconnect backoff, standby status, memory client interface, PII allowlist, init publication, schema drift, drain semantics, heartbeat semantics, cross-tenant isolation defense-in-depth). §8 lists 11 example payloads. §10 lists 38 failure rows. §11 lists 24 implementation notes covering crate choice rationale, LSN persistence model, SLA budget breakdown, dedup_key construction, pending_acks memory bound, lower-level vs higher-level postgres client choice, standby cadence, wal2json vs pgoutput, per-tenant Fargate cost tradeoff, replica safety scope, slot drop policy, heartbeat sizing, severity-field scoping, force-replay runbook.

## §2 — Findings (all resolved)

### ISS-001 — Sync method
Dual-write = complexity. Resolved: logical replication + DEC-460.

### ISS-002 — Sync_class derivation
Without explicit rule, drift. Resolved: §1 #5 + DEC-461 channel-privacy-driven.

### ISS-003 — LSN failure handling
Naive advance = data loss on memory down. Resolved: §1 #10 #13 pause-on-failure.

### ISS-004 — Latency SLA
Without budget, lag invisible. Resolved: §1 #8-9 + DEC-462 5s p95 + sev-2 alarm.

### ISS-005 — PII redaction pipeline
Without it, customer data lands raw. Resolved: §1 #6 TASK-MEMORY-111 ruleset.

### ISS-006 — Tenant scope
Cross-tenant emit = data leak. Resolved: §1 #11 per-process tenant_id.

### ISS-007 — Crash-mid-advance loses messages OR double-emits without dedup contract (strict-redo pass)
Original spec said "advance LSN on successful memory write" but didn't address the crash-after-ack-before-advance window. Without a dedup contract, TASK-MEMORY-107 sees the same logical event twice and stores both. Resolved: §1 #14 introduces deterministic `dedup_key = sha256(tenant_id || post_id || version)`; §1 #15 introduces `pending_acks: BTreeMap` and gates LSN advance on contiguous-acked entries; AC #19 #20 #32 + dedicated test bodies verify; TASK-MEMORY-107 dedup contract owns the read-side collapse.

### ISS-008 — Message edits silently overwrote (strict-redo pass, task-audit skill §3.8 audit-trail-integrity)
Original spec mapped UPDATE-posts to nothing OR to message_deleted. Mattermost message edits would have either been lost or mis-classified as deletions. Resolved: §1 #16 introduces `chat.message_edited` row kind with `prior_dedup_key` back-reference; map.rs distinguishes `delete_at > 0` from `message != old.message`; AC #21 + test body verify.

### ISS-009 — Reactions, channelmembers, fileinfo were absent (strict-redo pass)
The original publication only included posts + channels. TASK-CHAT-008 (mentions) and TASK-CHAT-012 (DSAR) downstream both need channelmember + reaction events to function. Without these, those tasks would have to query chat DB directly, coupling downstream to schema. Resolved: §1 #17-18-24 add row kinds; init-bridge-publication.sql adds tables to publication; AC #22-24 #30 + test bodies; map.rs handles all event types.

### ISS-010 — Read-replica DSN would silently no-op (strict-redo pass)
Postgres logical replication only works on primary. Original spec assumed primary connection but had no guard. An operator pointing the bridge at a read replica DSN would see a process that runs but emits nothing — silent data loss. Resolved: §1 #19 mandates `replica_safety_check` at startup; emits SEV-1 `chat.bridge_misconfigured` audit and exits non-zero; AC #25 + test body verify.

### ISS-011 — No bridge liveness signal in memory (strict-redo pass)
Operators investigating "is the bridge alive?" had to inspect OBS metrics OR tail logs. Neither is accessible from a memory-only context (e.g. compliance audit, CLI-only operator). Resolved: §1 #20 introduces `chat.bridge_heartbeat` row every 30s with full liveness payload; AC #26 + cadence-drift test; §11 documents the 30s ↔ 5min alarm-window calibration.

### ISS-012 — SIGTERM dropped in-flight messages (strict-redo pass)
Original spec didn't address Fargate scale-in. SIGTERM with no drain = up to N in-flight messages lost (or duplicated on restart per dedup contract — fine but noisy). Resolved: §1 #21 introduces 10s graceful drain + final-LSN persist + `chat.bridge_shutdown` audit; AC #27 + test body verify.

### ISS-013 — `@channel` triggers email regex (strict-redo pass)
The PII scrubber's regex `\w+@\w+\.\w+` would scrub the literal `@channel` Mattermost mention. Resolved: §1 #22 mandates `normalise_mattermost_specials` pre-pass that converts @channel/@here/@all and emoji codes and markdown links to non-PII-shaped placeholders before scanning; AC #28 + test body verify.

### ISS-014 — Sessions table at publication-scope risk (strict-redo pass)
A future operator running `ALTER PUBLICATION chat_bridge FOR ALL TABLES` would silently include Mattermost's Sessions table, leaking login events to memory that should be owned by TASK-CHAT-002. Resolved: §1 #25 mandates explicit table list; init-bridge-publication.sql enumerates the 5 tables explicitly; AC #31 + test body verify that an INSERT to Sessions produces no memory row; `tests/scope-isolation.py` reads `pg_stat_user_tables` to enforce.

## §3 — Resolution

All 14 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by genuine architectural surface (Postgres logical replication + PII redaction + memory exactly-once contract + 10 event types × dedup/edit/delete semantics), not by line targets.

---

*End of TASK-CHAT-005 audit.*
