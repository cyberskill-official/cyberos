---
id: FR-EMAIL-001
title: "EMAIL Stalwart Rust mail server deployment — JMAP + IMAP + SMTP + ManageSieve + MTA-STS + DANE + per-tenant residency + S3+KMS body storage + Postgres metadata"
module: EMAIL
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (interim CCO)
created: 2026-05-16
shipped: 2026-05-19
memory_chain_hash: pending
related_frs: [FR-AI-016, FR-AI-003, FR-MEMORY-101, FR-OBS-001, FR-EMAIL-002, FR-EMAIL-003, FR-EMAIL-004, FR-EMAIL-005, FR-EMAIL-006, FR-EMAIL-007, FR-EMAIL-009, FR-EMAIL-011]
depends_on: []
blocks: [FR-EMAIL-002, FR-EMAIL-003, FR-EMAIL-004, FR-EMAIL-005, FR-EMAIL-006, FR-EMAIL-007, FR-EMAIL-008, FR-EMAIL-011]   # 8 downstream consumers of substrate

source_pages:
  - website/docs/modules/email.html#what
  - website/docs/modules/email.html#architecture
  - https://stalw.art (Stalwart project; current production = 0.10.x)
source_decisions:
  - DEC-300 (Stalwart Rust mail server pinned at v0.10.x; AGPL-3.0; single binary serving JMAP, IMAP, SMTP, POP3, ManageSieve, MTA-STS, DANE, DKIM, ARC, BIMI)
  - DEC-301 (Postgres backend for Stalwart metadata — pinned via `data.store = "pg"` in stalwart.toml; the spec ships Stalwart with our cluster, never the embedded RocksDB store at production)
  - DEC-302 (S3 + KMS bodies — Stalwart's blob store points at residency-pinned S3 buckets per FR-AI-016; bucket per residency tag; KMS key per residency tag)
  - DEC-303 (SMTP listener on ports 25 (relay-in), 465 (TLS implicit), 587 (TLS STARTTLS); IMAP on 143/993; JMAP on /jmap over HTTPS 443; ManageSieve on 4190)
  - DEC-304 (DKIM keys per-tenant via Stalwart's signing keystore; 2048-bit RSA at slice 1; Ed25519 deferred to slice 2)
  - DEC-305 (MTA-STS policy fetched at remote-relay time; DANE TLSA records consulted when present; STARTTLS opportunistic for legacy peers)
  - DEC-306 (residency routing: tenant residency tag maps to (storage bucket, KMS key, Postgres schema namespace) at Stalwart adapter layer; cross-residency leakage is fail-closed)
  - DEC-307 (auth at slice 1 is Stalwart's internal user store — operators provisioned via `cyberos-email-cli`; FR-EMAIL-002 ships the JWT-bridge plugin to delegate to AUTH)
  - DEC-308 (spam triage via Stalwart's built-in SpamAssassin-equivalent + Rspamd-compatible filters; quarantined messages → Trash with 30-day retention)
  - DEC-309 (bounce + reputation handling: bounces go into `mail.bounce_log`; reputation per-tenant tracked via OTel; sev-3 alarm at sustained > 1% bounce rate)
  - DEC-310 (memory audit kinds: email.message_received, email.message_sent, email.message_bounced, email.message_quarantined, email.dkim_key_rotated)
  - DEC-311 (message bodies are stored encrypted at rest in S3 with KMS keyspace separate from DOC + KB; messages cannot be exported plaintext without an audit row)
  - DEC-312 (deployment target: Stalwart runs as a separate container; the gateway/adapter layer at services/email/ talks JMAP from the SPA + REST internally for module integration)
  - DEC-313 (DSAR export at message level — FR-EMAIL-011 ships the per-subject export; this FR ships the queryable metadata that the export reads)
  - Decree 53/2022 (VN data localisation — VN-tenant mail must reside in VN-region storage)
  - RFC 8620 (JMAP Core); RFC 8621 (JMAP for Mail); RFC 5321 (SMTP); RFC 9051 (IMAP4rev2); RFC 6376 (DKIM); RFC 8617 (ARC)
  - RFC 8460 (MTA-STS); RFC 7672 (SMTP + DANE); RFC 7489 (DMARC)
  - GDPR Art. 15 (right to access — DSAR export); PDPL Art. 14 (data subject rights)

language: rust 1.81 + sql + docker
service: cyberos/services/email/
new_files:
  - services/email/docker/Dockerfile                                  # Stalwart container image (FROM stalwartlabs/mail-server:0.10)
  - services/email/docker/stalwart.toml                               # Stalwart config — data store, blob store, network listeners, DKIM, MTA-STS
  - services/email/docker/compose.yml                                 # local-dev compose; Postgres + Stalwart + minio for S3
  - services/email/migrations/0001_messages.sql                       # message_metadata + thread_metadata + message_envelope tables (gateway-side mirror)
  - services/email/migrations/0002_bounce_log.sql                     # append-only bounce + reputation log
  - services/email/migrations/0003_dkim_keys.sql                      # per-tenant DKIM key registry + rotation history
  - services/email/migrations/0004_residency_routing.sql              # per-tenant residency → S3 bucket + KMS key + Postgres schema mapping
  - services/email/src/lib.rs                                         # crate root
  - services/email/src/types.rs                                       # EmailMessage, EmailThread, BounceEvent, DkimKey, MessageStatus enum
  - services/email/src/stalwart_adapter/mod.rs                        # Stalwart HTTP API client; sync metadata into our Postgres
  - services/email/src/stalwart_adapter/inbound.rs                    # consume Stalwart inbound events → write metadata + emit memory row
  - services/email/src/stalwart_adapter/outbound.rs                   # outbound send via Stalwart SMTP queue; tracks delivery state
  - services/email/src/dkim/keystore.rs                               # per-tenant DKIM key generation + rotation + Stalwart sync
  - services/email/src/residency.rs                                   # per-tenant residency lookup (FR-AI-016 contract)
  - services/email/src/repo/messages.rs                               # message metadata CRUD + thread linkage
  - services/email/src/repo/bounce_log.rs                             # append-only bounce writer
  - services/email/src/audit/email_events.rs                          # canonical email.* memory row builders (5 kinds per DEC-310)
  - services/email/src/handlers/status.rs                             # GET /v1/email/healthz + GET /v1/email/messages/{id}/status
  - services/email/src/cli/provision.rs                               # cyberos-email-cli provision (slice-1 user provisioning until FR-EMAIL-002 ships)
  - services/email/Cargo.toml                                         # +tokio, +sqlx, +uuid, +serde, +reqwest, +aws-sdk-s3, +cyberos-cli-exit
  - services/email/tests/inbound_quarantine_test.rs                     # mock Stalwart inbound; verify metadata + memory row
  - services/email/tests/stalwart_outbound_test.rs                    # mock SMTP queue; verify DKIM applied
  - services/email/tests/residency_pin_test.rs                        # VN tenant → vn-1 storage; EU → eu-1; assert no cross-region leakage
  - services/email/tests/dkim_per_tenant_test.rs                      # each tenant has its own key; rotation produces new key + new row
  - services/email/tests/bounce_log_append_only_test.rs               # UPDATE/DELETE rejected by SQL grant
  - services/email/tests/inbound_quarantine_test.rs                      # known-spam patterns → Trash; memory row email.message_quarantined
  - services/email/tests/protocol_endpoints_test.rs                   # SMTP 25/465/587 + IMAP 143/993 + JMAP 443 + ManageSieve 4190 all listening
  - services/email/tests/mta_sts_dane_test.rs                         # outbound peer with MTA-STS policy → enforces TLS; peer with DANE TLSA → uses certificate pin
  - services/email/tests/audit_row_test.rs                           # per-subject message list returns all messages where subject was sender or recipient
  - services/email/tests/audit_row_test.rs                       # received + sent + bounced + quarantined each emit exactly one memory row
modified_files:
  - services/auth/src/rls/templates.rs                                # add message_metadata, thread_metadata, bounce_log, dkim_keys to TENANT_SCOPED_TABLES

allowed_tools:
  - file_read: services/email/**
  - file_read: services/auth/src/rls/**
  - file_write: services/email/{src,tests,migrations,docker,cli}/**
  - bash: cd services/email && docker compose up -d (local dev)
  - bash: cd services/email && cargo test
  - bash: cd services/email && cyberos-email-cli provision (slice-1 user mgmt)

disallowed_tools:
  - allow plaintext message bodies in Postgres (per DEC-311 — bodies live in S3+KMS only)
  - skip DKIM signing on outbound (per DEC-304 + DEC-305 — mail must be authenticated)
  - allow cross-residency body storage (per DEC-306 + Decree 53/2022)
  - allow UPDATE on bounce_log or message_metadata (per feature-request-audit skill rule 12; append-only enforced)
  - implement CaMeL quarantine here (FR-EMAIL-005 ships)
  - implement shared-inbox UX here (FR-EMAIL-003 ships)
  - implement JMAP JWT bridge here (FR-EMAIL-002 ships)

effort_hours: 12
sub_tasks:
  - "1.5h: Dockerfile + stalwart.toml — Stalwart container with Postgres backend + S3 blob store + DKIM keystore + MTA-STS"
  - "0.5h: compose.yml — local-dev with Postgres + Stalwart + minio"
  - "1.0h: 0001_messages.sql — message_metadata + thread_metadata + message_envelope + RLS + REVOKE writes"
  - "0.4h: 0002_bounce_log.sql — append-only + REVOKE"
  - "0.4h: 0003_dkim_keys.sql — per-tenant keystore + rotation history"
  - "0.4h: 0004_residency_routing.sql — tenant residency → (bucket, kms_key, schema)"
  - "0.5h: types.rs — EmailMessage, EmailThread, BounceEvent, DkimKey, MessageStatus"
  - "1.0h: stalwart_adapter/inbound.rs — consume Stalwart inbound webhook events; write metadata; emit memory row"
  - "1.0h: stalwart_adapter/outbound.rs — send via Stalwart SMTP queue; track delivery"
  - "0.7h: dkim/keystore.rs — per-tenant key generation + rotation + Stalwart sync"
  - "0.5h: residency.rs — FR-AI-016 lookup"
  - "0.5h: repo/messages.rs — metadata CRUD + thread linkage"
  - "0.3h: repo/bounce_log.rs — append-only writer"
  - "0.4h: audit/email_events.rs — 5 row builders"
  - "0.4h: handlers/status.rs — health + per-message status"
  - "0.5h: cli/provision.rs — slice-1 user provisioning CLI"
  - "2.5h: tests — 10 test files covering inbound, outbound, residency, DKIM, bounce, spam, protocol endpoints, MTA-STS, DSAR query, audit emission"

risk_if_skipped: "EMAIL is the second-highest-volume communication channel after CHAT and one of the highest-risk attack surfaces (EchoLeak 2025 + most prompt-injection CVEs entered through email). Every downstream EMAIL FR (FR-EMAIL-002 JWT bridge, FR-EMAIL-003 shared-inbox UX, FR-EMAIL-004 DKIM/ARC/BIMI hardening, FR-EMAIL-005 CaMeL quarantine, FR-EMAIL-006 CRM auto-log, FR-EMAIL-007 thread-to-issue) needs the protocol stack and the metadata mirror. Without DEC-300's Stalwart deployment, we depend on hosted SaaS (Gmail/M365) which (a) cannot accommodate CaMeL, (b) lacks shared-inbox primitives, (c) violates Decree 53/2022 for VN tenants. Without DEC-306's residency routing, VN tenant mail bodies cross border. Without DEC-304's per-tenant DKIM keys, outbound legitimacy depends on shared keys (operationally fragile). Without DEC-311's S3+KMS body storage separate from Postgres, message bodies create a PII-everywhere problem. The 12h effort lands the foundational mail-server + adapter layer so every email FR has a known-good substrate."
---

## §1 — Description (BCP-14 normative)

The EMAIL service **MUST** deploy Stalwart as the canonical mail server with Postgres metadata mirror + S3+KMS body storage + per-tenant residency routing. Each requirement:

1. **MUST** deploy Stalwart v0.10.x as a containerised service (per DEC-300). Image: `stalwartlabs/mail-server:0.10` plus our config layer at `services/email/docker/stalwart.toml`. The container listens on ports 25, 465, 587 (SMTP), 143, 993 (IMAP), 4190 (ManageSieve); JMAP is reverse-proxied at `/jmap` on the application HTTPS endpoint.

2. **MUST** configure Stalwart's metadata backend as **Postgres** (per DEC-301): `[storage.data]\ntype = "pg"\nhost = "..."\ndatabase = "stalwart_metadata"\n`. The embedded RocksDB store is forbidden at production (per DEC-301); the Postgres backend is the single source of truth that integrates with our cluster's RLS + backup strategy.

3. **MUST** configure Stalwart's blob store as **S3 with KMS encryption** (per DEC-302), residency-pinned per tenant. Per-tenant routing is via Stalwart's directory-tag mechanism: a Stalwart user's `tenant_id` directs blob writes to the per-tenant bucket. Bucket naming: `cyberos-email-<residency>-bodies` (e.g. `cyberos-email-vn-1-bodies`).

4. **MUST** open the following protocol endpoints (per DEC-303):
    - **SMTP** port 25 (relay-in only; receives mail from external MTAs).
    - **SMTP** port 465 (TLS implicit; outbound submission).
    - **SMTP** port 587 (STARTTLS; outbound submission; preferred for clients).
    - **IMAP** port 143 (STARTTLS-required).
    - **IMAP** port 993 (TLS implicit).
    - **ManageSieve** port 4190 (filter rule management).
    - **JMAP** at `/jmap` on the application HTTPS endpoint (RFC 8620 + 8621).

5. **MUST** ship per-tenant DKIM key registry at `dkim_keys` table (per DEC-304). Schema: `(tenant_id, key_id, dkim_selector, key_algorithm, public_key, private_key_encrypted_kms_id, created_at, status active|rotated|revoked)`. Slice 1 generates RSA-2048 keys; Ed25519 (RFC 8463) deferred to slice 2. Stalwart's signing keystore is synced from this registry at start + on rotation.

6. **MUST** support MTA-STS policy fetch for outbound mail (per DEC-305). Stalwart fetches `https://mta-sts.<peer>.com/.well-known/mta-sts.txt` and enforces the policy (`mode=enforce` → drop on TLS fail; `mode=testing` → log only). DANE TLSA records consulted when the peer publishes them; STARTTLS opportunistic for legacy peers.

7. **MUST** ship the `message_metadata` Postgres table mirroring Stalwart's message envelope (per DEC-301). Schema: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `stalwart_message_id BIGINT NOT NULL` (Stalwart's internal id; for join), `thread_id TEXT NOT NULL` (RFC 8621 JMAP threadId), `direction message_direction NOT NULL` (inbound | outbound | internal), `from_address TEXT NOT NULL`, `to_addresses TEXT[] NOT NULL`, `cc_addresses TEXT[] NOT NULL DEFAULT '{}'`, `subject TEXT`, `received_at TIMESTAMPTZ NOT NULL`, `s3_body_key TEXT NOT NULL`, `s3_body_kms_key_id TEXT NOT NULL`, `body_sha256_hex CHAR(64) NOT NULL`, `byte_size BIGINT NOT NULL`, `status message_status NOT NULL` (received | quarantined | delivered | sent | bounced | dropped), `spam_score REAL`, `dkim_pass BOOLEAN`, `spf_pass BOOLEAN`, `dmarc_pass BOOLEAN`, `bimi_present BOOLEAN`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`. RLS-protected.

8. **MUST** ship the `thread_metadata` table for JMAP-native threading: `(thread_id TEXT, tenant_id UUID, subject_normalised TEXT, last_message_at TIMESTAMPTZ, message_count INT, participant_addresses TEXT[])`. Inbound messages are merged into existing threads by JMAP's `threadId` algorithm; thread_metadata is the materialised view for fast list queries.

9. **MUST** declare 2 closed Postgres enums:
    - `message_direction`: 3 values (`inbound`, `outbound`, `internal`).
    - `message_status`: 6 values (`received`, `quarantined`, `delivered`, `sent`, `bounced`, `dropped`).

10. **MUST** enforce RLS with both `USING` and `WITH CHECK` clauses on `message_metadata`, `thread_metadata`, `bounce_log`, `dkim_keys`. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`.

11. **MUST** be **append-only** on `message_metadata` AND `bounce_log` at the SQL-grant layer (per feature-request-audit skill rule 12). `REVOKE UPDATE, DELETE ON message_metadata, bounce_log FROM cyberos_app;`. Status changes (received → quarantined → delivered) create new rows linked via `prior_message_id` self-FK. Bounce events are pure inserts.

12. **MUST** route messages to per-tenant storage per FR-AI-016 (per DEC-306). The Stalwart inbound handler:
    - Resolves the recipient's tenant via the local-part-to-tenant directory.
    - Looks up tenant residency via `residency::resolve(tenant_id)`.
    - Writes the body to the residency-pinned S3 bucket with the residency-pinned KMS key.
    - Records `s3_body_key` + `s3_body_kms_key_id` in `message_metadata`.
   Cross-residency leakage (e.g. VN tenant body accidentally written to eu-1 bucket) is **fail-closed**: the handler asserts residency match before write.

13. **MUST** emit memory audit row `email.message_received` on every inbound delivery; `email.message_sent` on every outbound; `email.message_bounced` on every bounce; `email.message_quarantined` on spam classification; `email.dkim_key_rotated` on key rotation (per DEC-310). All rows carry `{tenant_id, message_id, thread_id, direction, from_hash16, to_hash16, body_sha256_hex, status, ts_ns}`.

14. **MUST** PII-scrub sensitive headers + body summary via FR-MEMORY-111 BEFORE chain commit. The PostgreSQL row retains the raw values (tenant-scoped + RLS-protected); the memory audit chain holds PII-stripped form. `from_hash16`/`to_hash16` = SHA-256[..16] of normalised email address.

15. **MUST** sign outbound messages with the tenant's active DKIM key (per DEC-304). Stalwart's signing keystore is synchronised from `dkim_keys` at start + on rotation event. Outbound message lacking DKIM signature → dropped + `email.message_dropped` memory row + sev-2 alarm via FR-OBS-007.

16. **MUST** ship `cyberos-email-cli provision --tenant-id <uuid> --local-part <name> --display-name <text>` as the slice-1 user-provisioning entry point. The CLI calls Stalwart's admin API to create the user, generates the per-tenant DKIM key if absent, and emits `email.user_provisioned` memory row. FR-EMAIL-002 replaces this with the JWT bridge plugin.

17. **MUST** track bounces in `bounce_log` (per DEC-309). Schema: `id BIGSERIAL PRIMARY KEY, tenant_id UUID, message_id UUID REFERENCES message_metadata(id), bounce_kind TEXT (hard | soft | transient), bounce_reason TEXT, bounce_code TEXT (SMTP status), remote_peer TEXT, ts TIMESTAMPTZ`. Bounce rate > 1% per tenant over rolling 24h → sev-3 alarm.

18. **MUST** quarantine spam per Stalwart's built-in classifier (per DEC-308). Spam score threshold: 5.0 (Stalwart default). Above → `status = quarantined`, message routed to per-user `Trash` folder with 30-day retention.

19. **MUST** expose internal REST handlers:
    - `GET /v1/email/healthz` — returns `{stalwart_status, last_message_received_at, last_message_sent_at, postgres_status, s3_status, registered_tenants}`.
    - `GET /v1/email/messages/{id}/status` — returns delivery status, DKIM/SPF/DMARC pass flags, bounce events.
    - `GET /v1/email/messages?tenant_id=<>&subject_id=<>&from=<ts>&to=<ts>` — list with cursor pagination (used by FR-EMAIL-003 + FR-EMAIL-011).

20. **MUST** complete inbound metadata write + memory audit emit in ≤ 200 ms p95 (excluding body upload to S3, which is async). `inbound_perf_test` asserts.

21. **MUST** complete `cyberos-email-cli provision` in ≤ 5 seconds (includes DKIM key generation if needed; RSA-2048 generation typically 1-2s).

22. **MUST** emit OTel span `email.{inbound,outbound,bounce,quarantine,dkim_rotate}` with attributes: `tenant_id`, `message_id`, `direction`, `outcome` (success | residency_mismatch | dkim_missing | body_too_large | s3_write_failed | postgres_write_failed).

23. **MUST** emit OTel metrics:
    - `email_message_received_total{tenant_id, outcome}` (counter).
    - `email_message_sent_total{tenant_id, outcome}` (counter).
    - `email_message_bounced_total{tenant_id, bounce_kind}` (counter).
    - `email_message_quarantined_total{tenant_id, spam_score_band}` (counter; bands: 5-7, 7-10, 10+).
    - `email_dkim_key_rotated_total{tenant_id}` (counter).
    - `email_stalwart_up{instance}` (gauge 0/1; scraped from /healthz).

24. **MUST** support graceful shutdown: SIGTERM triggers (a) refuse new SMTP connections with 421 temporary failure, (b) drain in-flight messages with 30s budget, (c) close IMAP/JMAP sessions cleanly, (d) shut down. Shutdown deadline configurable via env `EMAIL_SHUTDOWN_BUDGET_SECONDS`.

25. **MUST** ensure body integrity at storage: `body_sha256_hex` recorded in metadata MUST match the actual SHA-256 of the S3 object. A reconciliation job (out of scope here; FR-EMAIL-2xx) cross-checks; this FR enforces at write time via S3 ETag comparison.

26. **MUST** support `Bcc` recipients without leaking in `to_addresses` or `cc_addresses` arrays. Per RFC 5322, Bcc is preserved at the SMTP envelope level but stripped from the message body. The Stalwart adapter records Bcc in a separate `bcc_addresses TEXT[]` column visible only to the sender's view (RLS additional clause).

---

## §2 — Why this design (rationale for humans)

**Why Stalwart instead of building a mail server (DEC-300)?** Building a production-grade mail server is years of work (RFC compliance + spam filtering + reputation handling + DKIM + MTA-STS + DANE all need to be correct or mail breaks). Stalwart is a credible Rust implementation of the full stack: actively developed, AGPL-licensed (commercial deployment is OK for our use), single binary (operational simplicity). The cost is "we don't own every line"; the benefit is "we ship a working mail server in 12 hours instead of 12 months."

**Why Postgres backend instead of RocksDB (DEC-301)?** Stalwart's default RocksDB is single-machine and lacks RLS. Pointing at Postgres lets us (a) share the cluster (one operational surface), (b) apply RLS (tenant isolation by construction), (c) use existing backup tooling. The Postgres backend is officially supported by Stalwart; the only cost is slightly higher write latency (5ms vs 0.5ms typical), which is invisible against SMTP's ~100ms baseline.

**Why S3 + KMS for bodies (DEC-302, DEC-311)?** Bodies are large + PII-heavy + retention-bound. Storing in Postgres would bloat the cluster + create a PII-everywhere problem (every backup carries every body). S3 + KMS isolates bodies from the metadata cluster, encryption at rest is per-tenant (residency + KMS keyspace), retention can be enforced via S3 lifecycle. The cost is a hop on read (50-100ms typical); the benefit is operational + compliance separation.

**Why per-tenant residency routing (DEC-306, §1 #12)?** Decree 53/2022 (VN) + GDPR (EU) + DORA (EU finance) impose data-residency obligations. A single global mail server violates all three for cross-jurisdictional tenants. The residency router maps tenant → bucket + KMS at body-write time; cross-region writes are fail-closed. The implementation cost is one lookup per message; the compliance benefit is mandatory.

**Why per-tenant DKIM keys (DEC-304, §1 #5)?** A shared DKIM key for all tenants means any tenant's compromised key affects all. Per-tenant keys isolate compromise + give each tenant a stable signing identity (matters for reputation + BIMI). RSA-2048 is the universal-support floor; Ed25519 (smaller + faster) deferred to slice 2 because not all remote MTAs support it yet.

**Why MTA-STS + DANE (DEC-305, §1 #6)?** STARTTLS without policy is "best-effort encryption" — a MITM downgrade attack succeeds. MTA-STS publishes a policy ("require TLS for my domain"); DANE pins the TLS certificate via DNSSEC. Together they make MTA-to-MTA TLS verifiable. Stalwart supports both; we just need to enable.

**Why append-only message metadata (§1 #11)?** Mail is legally significant — operators tracing "did we receive this message?" need the original recorded fact. UPDATE-in-place would let a malicious or buggy handler rewrite history. Status transitions write new rows linked via `prior_message_id`; the original `received` row stays. Bounce events are pure inserts.

**Why ports 25/465/587 all open (§1 #4)?** 25 is the standard inbound-relay port (legacy MTAs use it). 465 is the SMTPS submission port (TLS implicit). 587 is the modern submission port (STARTTLS-required). Modern clients prefer 587; legacy clients use 465. Both submission ports require auth; port 25 is recipient-domain accepted only.

**Why JMAP on application HTTPS endpoint, not direct (DEC-303)?** JMAP-over-HTTPS uses standard HTTP/2 paths; reverse-proxying through our ingress lets us apply standard rate-limiting + WAF + auth (FR-EMAIL-002 JWT bridge). The cost is one extra hop; the benefit is unified observability + security.

**Why CaMeL quarantine deferred to FR-EMAIL-005 (disallowed_tools)?** CaMeL is a deliberate isolation pattern (quarantine LLM with no tools + no memory parses untrusted body; privileged CUO only sees sanitised extraction). Implementing it correctly is its own work; shipping FR-EMAIL-001 + FR-EMAIL-005 in parallel slows both. Split: this FR ships the mail server; FR-EMAIL-005 wires CaMeL between the inbound handler and any CUO-bound consumer.

**Why shared-inbox UX deferred to FR-EMAIL-003?** Shared-inbox UX (assignment, internal comments, snooze, tagging) is a TypeScript SPA + backend handlers — substantial. Splitting lets this FR focus on the protocol stack + storage + residency; the UX builds on top.

**Why JWT bridge deferred to FR-EMAIL-002 (DEC-307)?** Stalwart has its own user store; integrating with our AUTH JWT (FR-AUTH-004) is a Stalwart plugin. The plugin is a focused work item (~6h); shipping it together would mean blocking on AUTH-004 fully. Slice 1 uses Stalwart's internal store via the CLI; slice 2 cuts over.

**Why bounces in append-only log (§1 #17, DEC-309)?** Bounce reasons are diagnostic + reputation-relevant. A hard bounce (mailbox doesn't exist) means future sends to that address should be suppressed; a soft bounce (mailbox full) is transient. Tracking both with timestamps lets us build the suppression list + monitor reputation. Append-only because reputation reports are forensic — they need the original sequence.

**Why spam quarantine at threshold 5.0 (§1 #18)?** Stalwart's classifier uses the SpamAssassin-style additive scoring (each rule adds a small number). 5.0 is the well-established threshold; below = ham; above = spam. Quarantined → Trash with 30-day retention gives users time to recover false positives.

**Why fail-closed residency mismatch (§1 #12)?** A cross-region body write is a Decree 53/2022 violation per message — a single mistake creates real regulatory exposure. Fail-closed means the handler refuses to commit; the message gets retried (transient SMTP failure 421) until the residency lookup is fixed. The cost is occasional delivery delay; the benefit is "compliance is enforced by code, not by review."

**Why no DELETE on metadata or bounce_log (§1 #11)?** Both are forensic records. DELETE would let operators "make a mistake go away"; legal/compliance reviews of mail flows need the complete history. Retention policies are S3-side (lifecycle rules); metadata persists until explicit GDPR/PDPL erasure (FR-EMAIL-2xx ships handler that anonymises rather than deletes).

**Why hashed addresses in memory audit (§1 #14)?** Email addresses are PII. memory audit chain is forensically queryable but kept lean; embedding raw addresses would create everywhere-PII. SHA-256[..16] is collision-safe at our scale (~10⁹) + lets forensic queries join.

**Why ManageSieve port 4190 (§1 #4)?** Sieve is the standard mail-filter language; ManageSieve is the protocol for clients to upload/manage filter rules. Members may set per-mailbox auto-filters (e.g. "label as project-alpha if from:client-x@example.com"); the standard port is 4190 per RFC 5804.

**Why 30s shutdown drain (§1 #24)?** SMTP receivers can hold a connection for several seconds during DATA phase; 30s gives all in-flight messages a chance to finish. Past 30s the receiver gets 421 (try again later) — the sending MTA retries.

**Why `Bcc` separate column (§1 #26)?** RFC 5322 keeps Bcc out of the delivered message body to preserve recipient privacy (other recipients shouldn't see Bcc'd parties). Our `to_addresses + cc_addresses` arrays mirror the delivered message; `bcc_addresses` captures the SMTP envelope (visible only to the sender via RLS-extended policy).

**Why FR-EMAIL-001 has zero upstream deps (`depends_on: []`)?** The mail server itself is a leaf service — it doesn't depend on RBAC at slice 1 (per DEC-307 — Stalwart internal user store), it doesn't depend on RLS (Stalwart handles its own auth), it doesn't depend on FR-AI-016 logically (residency routing is a config lookup; the residency tag table can ship alongside this FR with a default `vn-1` tag). Slice 2 wires JWT (FR-EMAIL-002 = FR-AUTH-004) + slice 3 wires CUO (FR-EMAIL-005 = FR-CUO-101). Slice 1 is foundational and independent.

**Why slice 1 uses Stalwart internal user store (DEC-307)?** Two reasons. (1) Stalwart's directory layer already exists; reusing it is fastest path to working mail. (2) Mail server deployment is risky enough; adding JWT-bridge complexity at the same slice multiplies failure modes. FR-EMAIL-002 cuts over in slice 2, after slice 1 is proven stable.

**Why DKIM keys in their own table not as Stalwart config (§1 #5)?** Two reasons. (1) Stalwart's keystore is per-server; if we run multiple Stalwart instances (HA), we need a shared key store. (2) Rotation history is forensically relevant — knowing "this message was signed with key v3 (rotated 2026-04-15)" matters. Postgres table is the source of truth; Stalwart's keystore is a synced cache.

---

## §3 — API contract

### 3.1 — Stalwart config

```toml
# services/email/docker/stalwart.toml

[storage]
data         = "pg"
blob         = "s3"
fts          = "pg"

[storage.data]
type     = "pg"
host     = "${POSTGRES_HOST}"
port     = 5432
database = "stalwart_metadata"
user     = "${POSTGRES_USER}"
password = "${POSTGRES_PASSWORD}"

[storage.blob]
type        = "s3"
region      = "${S3_REGION_DEFAULT}"   # adapter override per-tenant via directory tag
bucket      = "${S3_BUCKET_DEFAULT}"
kms-key-arn = "${KMS_KEY_DEFAULT}"
sse         = "aws:kms"

# SMTP listeners
[server.listener."smtp.relay-in"]
bind     = ["[::]:25"]
protocol = "smtp"
tls.implicit = false

[server.listener."smtp.submission-implicit"]
bind     = ["[::]:465"]
protocol = "smtp"
tls.implicit = true

[server.listener."smtp.submission-starttls"]
bind     = ["[::]:587"]
protocol = "smtp"
tls.implicit = false

# IMAP listeners
[server.listener."imap.starttls"]
bind     = ["[::]:143"]
protocol = "imap"
tls.implicit = false

[server.listener."imap.implicit"]
bind     = ["[::]:993"]
protocol = "imap"
tls.implicit = true

# ManageSieve
[server.listener."managesieve"]
bind     = ["[::]:4190"]
protocol = "managesieve"

# JMAP — reverse-proxied at application layer
[server.listener."jmap"]
bind     = ["[::]:8080"]
protocol = "jmap"

# DKIM signing keys (synced from Postgres dkim_keys table at boot)
[signature."default"]
type   = "dkim"
domain = "%{tenant.primary_domain}%"
selector = "cyberos"
algorithm = "rsa-sha256"
private-key = "file:///etc/stalwart/keys/%{tenant.id}%/cyberos.pem"

# MTA-STS + DANE
[security.tls.outbound]
mta-sts = "enforce"
dane = "opportunistic"
fallback = "starttls"
```

### 3.2 — Migration 0001 — message_metadata + thread_metadata

```sql
-- services/email/migrations/0001_messages.sql

BEGIN;

CREATE TYPE message_direction AS ENUM ('inbound', 'outbound', 'internal');
CREATE TYPE message_status    AS ENUM ('received', 'quarantined', 'delivered', 'sent', 'bounced', 'dropped');

CREATE TABLE thread_metadata (
    thread_id              TEXT         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    subject_normalised     TEXT,
    last_message_at        TIMESTAMPTZ  NOT NULL,
    message_count          INT          NOT NULL DEFAULT 0,
    participant_addresses  TEXT[]       NOT NULL DEFAULT '{}'
);

CREATE TABLE message_metadata (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    stalwart_message_id    BIGINT       NOT NULL,
    thread_id              TEXT         NOT NULL REFERENCES thread_metadata(thread_id) ON DELETE RESTRICT,
    direction              message_direction NOT NULL,
    from_address           TEXT         NOT NULL,
    to_addresses           TEXT[]       NOT NULL,
    cc_addresses           TEXT[]       NOT NULL DEFAULT '{}',
    bcc_addresses          TEXT[]       NOT NULL DEFAULT '{}',
    subject                TEXT,
    received_at            TIMESTAMPTZ  NOT NULL,
    s3_body_key            TEXT         NOT NULL,
    s3_body_kms_key_id     TEXT         NOT NULL,
    body_sha256_hex        CHAR(64)     NOT NULL CHECK (body_sha256_hex ~ '^[0-9a-f]{64}$'),
    byte_size              BIGINT       NOT NULL CHECK (byte_size BETWEEN 1 AND 26214400),   -- 25 MB attachment cap
    status                 message_status NOT NULL,
    prior_message_id       UUID         REFERENCES message_metadata(id),
    spam_score             REAL,
    dkim_pass              BOOLEAN,
    spf_pass               BOOLEAN,
    dmarc_pass             BOOLEAN,
    bimi_present           BOOLEAN,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX message_metadata_tenant_received_idx ON message_metadata (tenant_id, received_at DESC);
CREATE INDEX message_metadata_thread_idx ON message_metadata (thread_id, received_at ASC);
CREATE INDEX message_metadata_from_idx ON message_metadata (tenant_id, from_address);
CREATE INDEX message_metadata_status_idx ON message_metadata (tenant_id, status);
CREATE INDEX thread_metadata_tenant_last_idx ON thread_metadata (tenant_id, last_message_at DESC);

ALTER TABLE message_metadata ENABLE ROW LEVEL SECURITY;
ALTER TABLE thread_metadata ENABLE ROW LEVEL SECURITY;

CREATE POLICY message_metadata_tenant_isolation ON message_metadata
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

CREATE POLICY thread_metadata_tenant_isolation ON thread_metadata
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Append-only enforcement (§1 #11)
REVOKE UPDATE, DELETE ON message_metadata FROM cyberos_app;

COMMIT;
```

### 3.3 — Migration 0002 — bounce_log

```sql
-- services/email/migrations/0002_bounce_log.sql

BEGIN;

CREATE TABLE bounce_log (
    id              BIGSERIAL    PRIMARY KEY,
    tenant_id       UUID         NOT NULL,
    message_id      UUID         NOT NULL REFERENCES message_metadata(id) ON DELETE RESTRICT,
    bounce_kind     TEXT         NOT NULL CHECK (bounce_kind IN ('hard', 'soft', 'transient')),
    bounce_reason   TEXT         NOT NULL,
    bounce_code     TEXT,                                       -- SMTP enhanced status
    remote_peer     TEXT,
    ts              TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX bounce_log_tenant_ts_idx ON bounce_log (tenant_id, ts DESC);
CREATE INDEX bounce_log_message_idx ON bounce_log (message_id);

ALTER TABLE bounce_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY bounce_log_tenant_isolation ON bounce_log
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON bounce_log FROM cyberos_app;

COMMIT;
```

### 3.4 — Migration 0003 — DKIM keystore

```sql
-- services/email/migrations/0003_dkim_keys.sql

BEGIN;

CREATE TABLE dkim_keys (
    id                              UUID         PRIMARY KEY,
    tenant_id                       UUID         NOT NULL,
    dkim_selector                   TEXT         NOT NULL DEFAULT 'cyberos',
    key_algorithm                   TEXT         NOT NULL CHECK (key_algorithm IN ('rsa-2048', 'ed25519')),
    public_key_pem                  TEXT         NOT NULL,
    private_key_kms_encrypted_blob  BYTEA        NOT NULL,
    kms_key_id                      TEXT         NOT NULL,
    status                          TEXT         NOT NULL CHECK (status IN ('active', 'rotated', 'revoked')) DEFAULT 'active',
    created_at                      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    rotated_at                      TIMESTAMPTZ
);

CREATE UNIQUE INDEX uniq_active_dkim_key ON dkim_keys (tenant_id, dkim_selector) WHERE status = 'active';
CREATE INDEX dkim_keys_tenant_idx ON dkim_keys (tenant_id, created_at DESC);

ALTER TABLE dkim_keys ENABLE ROW LEVEL SECURITY;
CREATE POLICY dkim_keys_tenant_isolation ON dkim_keys
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON dkim_keys FROM cyberos_app;

COMMIT;
```

### 3.5 — Residency resolver

```rust
// services/email/src/residency.rs
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EmailStorageBinding {
    pub region: String,
    pub bucket: String,
    pub kms_key_id: String,
}

pub async fn resolve(tenant_id: Uuid, db: &sqlx::PgPool) -> anyhow::Result<EmailStorageBinding> {
    let residency: String = sqlx::query_scalar("SELECT residency FROM tenant_residency WHERE tenant_id = $1")
        .bind(tenant_id).fetch_one(db).await?;

    let (region, bucket_prefix) = match residency.as_str() {
        "vn-1" => ("ap-southeast-1", "cyberos-email-vn-1-bodies"),
        "sg-1" => ("ap-southeast-1", "cyberos-email-sg-1-bodies"),
        "eu-1" => ("eu-west-1",      "cyberos-email-eu-1-bodies"),
        "us-1" => ("us-east-1",      "cyberos-email-us-1-bodies"),
        _ => anyhow::bail!("unknown_residency: {residency}"),
    };

    Ok(EmailStorageBinding {
        region: region.into(),
        bucket: bucket_prefix.into(),
        kms_key_id: format!("alias/{bucket_prefix}"),
    })
}
```

### 3.6 — Inbound adapter

```rust
// services/email/src/stalwart_adapter/inbound.rs
use axum::{Json, extract::State};
use crate::types::*;
use crate::audit::email_events;
use crate::residency;

#[derive(Deserialize)]
pub struct StalwartInboundEvent {
    pub stalwart_message_id: i64,
    pub thread_id: String,
    pub tenant_id: Uuid,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub cc_addresses: Vec<String>,
    pub bcc_addresses: Vec<String>,
    pub subject: Option<String>,
    pub received_at: DateTime<Utc>,
    pub body_bytes: Vec<u8>,
    pub spam_score: f32,
    pub dkim_pass: bool,
    pub spf_pass: bool,
    pub dmarc_pass: bool,
    pub bimi_present: bool,
}

pub async fn on_inbound(
    State(state): State<AppState>,
    Json(evt): Json<StalwartInboundEvent>,
) -> Result<StatusCode, ApiError> {
    // Validate body size at adapter boundary
    if evt.body_bytes.is_empty() || evt.body_bytes.len() > 26_214_400 {
        return Err(ApiError::BodyTooLarge);
    }
    let body_sha = hex::encode(sha2::Sha256::digest(&evt.body_bytes));

    // Residency-pin write to S3
    let binding = residency::resolve(evt.tenant_id, &state.db).await?;
    let s3_key = format!("{tenant_id}/{stalwart_id}/{sha}",
        tenant_id = evt.tenant_id, stalwart_id = evt.stalwart_message_id, sha = body_sha);
    state.s3.put_object()
        .bucket(&binding.bucket)
        .key(&s3_key)
        .body(evt.body_bytes.clone().into())
        .ssekms_key_id(&binding.kms_key_id)
        .server_side_encryption(aws_sdk_s3::types::ServerSideEncryption::AwsKms)
        .send().await?;

    // Insert thread (upsert) + message metadata
    let mut tx = state.db.begin().await?;
    sqlx::query(r#"
        INSERT INTO thread_metadata (thread_id, tenant_id, subject_normalised, last_message_at, message_count, participant_addresses)
        VALUES ($1, $2, $3, $4, 1, $5)
        ON CONFLICT (thread_id) DO UPDATE
        SET last_message_at = EXCLUDED.last_message_at,
            message_count   = thread_metadata.message_count + 1,
            participant_addresses = thread_metadata.participant_addresses || EXCLUDED.participant_addresses
    "#).bind(&evt.thread_id).bind(evt.tenant_id).bind(normalise_subject(&evt.subject))
       .bind(evt.received_at).bind(&[evt.from_address.clone()][..])
       .execute(&mut *tx).await?;

    let status = if evt.spam_score >= 5.0 { MessageStatus::Quarantined } else { MessageStatus::Received };

    let msg_id = Uuid::new_v4();
    sqlx::query(r#"
        INSERT INTO message_metadata (id, tenant_id, stalwart_message_id, thread_id, direction,
            from_address, to_addresses, cc_addresses, bcc_addresses, subject, received_at,
            s3_body_key, s3_body_kms_key_id, body_sha256_hex, byte_size, status,
            spam_score, dkim_pass, spf_pass, dmarc_pass, bimi_present)
        VALUES ($1, $2, $3, $4, 'inbound'::message_direction, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15::message_status, $16, $17, $18, $19, $20)
    "#).bind(msg_id).bind(evt.tenant_id).bind(evt.stalwart_message_id).bind(&evt.thread_id)
       .bind(&evt.from_address).bind(&evt.to_addresses).bind(&evt.cc_addresses).bind(&evt.bcc_addresses)
       .bind(&evt.subject).bind(evt.received_at)
       .bind(&s3_key).bind(&binding.kms_key_id).bind(&body_sha).bind(evt.body_bytes.len() as i64)
       .bind(status as MessageStatus)
       .bind(evt.spam_score).bind(evt.dkim_pass).bind(evt.spf_pass).bind(evt.dmarc_pass).bind(evt.bimi_present)
       .execute(&mut *tx).await?;

    email_events::emit_message_received(&mut tx, evt.tenant_id, msg_id, &evt, status).await?;
    tx.commit().await?;

    Ok(StatusCode::CREATED)
}
```

---

## §4 — Acceptance criteria

1. **Stalwart container starts** — `docker compose up` brings up Stalwart + Postgres + minio; healthcheck passes within 30s.
2. **All protocol ports listen** — 25, 465, 587 (SMTP), 143, 993 (IMAP), 4190 (ManageSieve), 8080 (JMAP); `protocol_endpoints_test` asserts.
3. **Stalwart uses Postgres backend** — `[storage.data].type == "pg"`; rocksdb forbidden via env check.
4. **Stalwart uses S3 blob store** — `[storage.blob].type == "s3"`; per-tenant bucket override applied via residency resolver.
5. **Inbound mail creates message_metadata row** — mock Stalwart event → 1 row in `message_metadata` + 1 in `thread_metadata` (or message_count += 1) + 1 memory `email.message_received` row.
6. **Outbound mail signed with tenant DKIM** — mock outbound → message DKIM-Signature header carries tenant's `cyberos` selector.
7. **Per-tenant DKIM keys distinct** — two tenants get distinct keys; `dkim_per_tenant_test` asserts.
8. **DKIM key rotation creates new row** — `cyberos-email-cli rotate-dkim --tenant <uuid>` → new `dkim_keys` row with `status=active`; prior row → `status=rotated`; memory `email.dkim_key_rotated` row emitted.
9. **Residency pinning: VN tenant → vn-1 bucket** — body written to `cyberos-email-vn-1-bodies`; metadata records `s3_body_kms_key_id = alias/cyberos-email-vn-1-bodies`.
10. **Cross-residency leak prevented** — handler refuses to write body to wrong-region bucket; returns SMTP 421 transient failure.
11. **Append-only metadata** — `UPDATE message_metadata SET status='delivered' WHERE id=$1` as cyberos_app → permission denied.
12. **Append-only bounce log** — same.
13. **Spam quarantined at score ≥ 5.0** — mock event with `spam_score=5.5` → `status=quarantined`; `email.message_quarantined` row.
14. **DSAR query returns all messages for subject** — `GET /v1/email/messages?subject_id=<uuid>` returns all messages where subject is sender or in to/cc.
15. **MTA-STS policy enforcement** — outbound to peer with `mode=enforce` MTA-STS + TLS handshake failure → message dropped + `email.message_dropped` memory row.
16. **DANE TLSA pinning** — peer with DANE TLSA + matching certificate → delivered; mismatch → dropped.
17. **Bcc not in to_addresses/cc_addresses** — incoming message with Bcc → metadata has Bcc in separate `bcc_addresses` column.
18. **Body integrity verified at write** — `body_sha256_hex` matches actual S3 ETag.
19. **Body > 25 MB rejected** — DB CHECK + handler validation → 413.
20. **Bounce rate alarm** — > 1% rolling 24h per tenant → sev-3.
21. **Graceful shutdown** — SIGTERM → SMTP returns 421; in-flight messages drained ≤ 30 s.
22. **Stalwart up gauge** — `email_stalwart_up{instance=...} == 1`; scraped from /healthz.
23. **OTel span emitted per inbound** — `email.inbound` with `outcome=success`.
24. **Counter `email_message_received_total{tenant_id, outcome=success}` increments** — per inbound.
25. **Counter `email_message_quarantined_total{tenant_id, spam_score_band=5-7}` increments** — per quarantine.
26. **Inbound perf < 200ms p95** — `inbound_perf_test` (1k iters).
27. **provision CLI < 5s** — RSA-2048 generation + Stalwart admin call.

---

## §5 — Verification

```rust
// services/email/tests/protocol_endpoints_test.rs
#[tokio::test]
async fn all_protocol_ports_listening() {
    let ctx = TestEmailStack::up().await;
    for (port, label) in [
        (25, "smtp-relay-in"), (465, "smtp-implicit"), (587, "smtp-starttls"),
        (143, "imap-starttls"), (993, "imap-implicit"),
        (4190, "managesieve"), (8080, "jmap"),
    ] {
        let conn = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")).await;
        assert!(conn.is_ok(), "{label} port {port} not listening: {:?}", conn.err());
    }
}
```

```rust
// services/email/tests/residency_pin_test.rs
#[tokio::test]
async fn vn_tenant_body_lands_in_vn_bucket() {
    let ctx = TestEmailStack::up().await;
    let tenant_id = ctx.create_tenant_with_residency("vn-1").await;
    let evt = mock_inbound_event(tenant_id);
    ctx.simulate_stalwart_inbound(evt.clone()).await.unwrap();
    let row = ctx.fetch_message_metadata_for_message(evt.stalwart_message_id).await;
    assert!(row.s3_body_key.starts_with(&format!("{tenant_id}/")));
    assert_eq!(row.s3_body_kms_key_id, "alias/cyberos-email-vn-1-bodies");
    let obj_exists = ctx.s3_head_object("cyberos-email-vn-1-bodies", &row.s3_body_key).await;
    assert!(obj_exists);
    let obj_in_wrong_bucket = ctx.s3_head_object("cyberos-email-eu-1-bodies", &row.s3_body_key).await;
    assert!(!obj_in_wrong_bucket);
}
```

```rust
// services/email/tests/dkim_per_tenant_test.rs
#[tokio::test]
async fn each_tenant_has_distinct_active_key() {
    let ctx = TestEmailStack::up().await;
    let t1 = ctx.create_tenant_with_residency("vn-1").await;
    let t2 = ctx.create_tenant_with_residency("vn-1").await;
    ctx.run_cli(&["provision", "--tenant-id", &t1.to_string(), "--local-part", "ops", "--display-name", "Ops"]).await;
    ctx.run_cli(&["provision", "--tenant-id", &t2.to_string(), "--local-part", "ops", "--display-name", "Ops"]).await;
    let k1: String = ctx.fetch_active_dkim_public(t1).await;
    let k2: String = ctx.fetch_active_dkim_public(t2).await;
    assert_ne!(k1, k2);
}
```

```rust
// services/email/tests/audit_row_test.rs
#[sqlx::test]
async fn metadata_update_blocked(pool: sqlx::PgPool) {
    set_role_app(&pool).await;
    let id = seed_message_metadata(&pool).await;
    let err = sqlx::query("UPDATE message_metadata SET status = 'delivered'::message_status WHERE id = $1")
        .bind(id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("permission denied"));
}

#[sqlx::test]
async fn metadata_delete_blocked(pool: sqlx::PgPool) {
    set_role_app(&pool).await;
    let id = seed_message_metadata(&pool).await;
    let err = sqlx::query("DELETE FROM message_metadata WHERE id = $1").bind(id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("permission denied"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; outbound, bounce handlers, and the DKIM keystore writer follow the same pattern.)

---

## §7 — Dependencies

**Upstream:** none — this FR is a leaf service. Residency lookup consumes FR-AI-016's policy table; FR-AI-016 ships independently (its dependency on this FR is for the email-side residency tag).

**Downstream (6 placeholders):**
- **FR-EMAIL-002** — `cyberos-email-authbridge` plugin (Stalwart JMAP auth delegates to AUTH JWT).
- **FR-EMAIL-003** — Missive-style shared-inbox UX.
- **FR-EMAIL-004** — DKIM signing + ARC chain forward + BIMI brand indicator.
- **FR-EMAIL-006** — tracked-domain auto-log to CRM activity feed.
- **FR-EMAIL-007** — "Convert to issue" thread → PROJ Issue.
- **FR-EMAIL-011** — DSAR export per subject.

**Cross-module:**
- **FR-AI-016** — residency policy; consumed by `residency::resolve()`.
- **FR-AI-003** — memory audit bridge; receives 5 `email.*` audit row kinds.
- **FR-MEMORY-111** — PII scrubbing on `from_address`, `to_addresses`, `subject` before chain commit.

---

## §8 — Example payloads

### 8.1 — Stalwart inbound webhook event

```json
{
  "stalwart_message_id": 8472,
  "thread_id": "<CABcd123ef.tenant.cyberos@mail.cyberos.com>",
  "tenant_id": "5e8f1d2a-...",
  "from_address": "alice@example.com",
  "to_addresses": ["support@cyberskill.world"],
  "cc_addresses": [],
  "bcc_addresses": [],
  "subject": "Question about Q4 invoice",
  "received_at": "2026-05-16T10:00:00Z",
  "body_bytes": "<base64-encoded MIME message>",
  "spam_score": 0.5,
  "dkim_pass": true,
  "spf_pass": true,
  "dmarc_pass": true,
  "bimi_present": false
}
```

### 8.2 — email.message_received memory row

```json
{
  "kind": "email.message_received",
  "tenant_id": "5e8f1d2a-...",
  "message_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "thread_id": "<CABcd123ef.tenant.cyberos@mail.cyberos.com>",
  "direction": "inbound",
  "from_hash16": "a47c8e0c5f8a8e8e",
  "to_hash16": "b58d9f1d6e9b9f9f",
  "body_sha256_hex": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
  "byte_size": 12340,
  "status": "received",
  "spam_score": 0.5,
  "dkim_pass": true,
  "spf_pass": true,
  "dmarc_pass": true,
  "ts_ns": 1747920731000000000
}
```

### 8.3 — email.message_quarantined memory row

```json
{
  "kind": "email.message_quarantined",
  "tenant_id": "5e8f1d2a-...",
  "message_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8N",
  "thread_id": "<...>",
  "from_hash16": "spammyhash01234",
  "spam_score": 7.3,
  "spam_score_band": "7-10",
  "ts_ns": 1747920731000000000
}
```

### 8.4 — email.dkim_key_rotated memory row

```json
{
  "kind": "email.dkim_key_rotated",
  "tenant_id": "5e8f1d2a-...",
  "old_key_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "new_key_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8N",
  "dkim_selector": "cyberos",
  "key_algorithm": "rsa-2048",
  "rotated_by_subject_id_hash16": "9b1deb4d3b7d4bad",
  "ts_ns": 1747920731000000000
}
```

### 8.5 — cyberos-email-cli provision

```bash
$ cyberos-email-cli provision \
    --tenant-id 5e8f1d2a-... \
    --local-part support \
    --display-name "Support Team"
✓ Stalwart user created: support@cyberskill.world
✓ DKIM key generated (RSA-2048, selector=cyberos)
✓ memory audit row email.user_provisioned written
```

---

## §9 — Open questions

Deferred:
- **JMAP JWT bridge** — FR-EMAIL-002.
- **Shared-inbox UX (assignment, comments, snooze, tag)** — FR-EMAIL-003.
- **DKIM + ARC + BIMI hardening** — FR-EMAIL-004.
- **CaMeL quarantine LLM** — FR-EMAIL-005.
- **Tracked-domain auto-log to CRM** — FR-EMAIL-006.
- **Convert thread to PROJ issue** — FR-EMAIL-007.
- **Outbound 1:1 send with AM confirm** — FR-EMAIL-009.
- **DSAR export** — FR-EMAIL-011.
- **Ed25519 DKIM (RFC 8463)** — slice 2.
- **Vietnamese-aware PGroonga search** — slice 2.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Stalwart container fails to start | docker healthcheck | Service unavailable | Investigate logs; common causes: bad config, DB unreachable, S3 perm |
| Postgres backend connection lost | Stalwart loses metadata | SMTP returns 451 (temporary) | DB restored |
| S3 write failure for body | adapter logs + S3 SDK error | SMTP returns 421 | S3 outage recovery |
| KMS key disabled | aws-sdk error on PUT | SMTP returns 421 + sev-1 | Re-enable; rotate per FR-AUTH-2xx |
| Residency mismatch attempt | handler check | SMTP returns 421 | Operator fixes residency tag |
| Body > 25MB | DB CHECK + handler | SMTP returns 552 size limit | Caller chunks |
| MTA-STS policy fetch fails | Stalwart fallback to STARTTLS | Outbound proceeds opportunistic | Designed |
| DANE TLSA mismatch | TLS verify fail | Outbound dropped + sev-3 alarm | Investigate peer cert change |
| DKIM signing key missing for tenant | outbound handler check | Outbound dropped + sev-2 alarm | Provision key via CLI |
| DKIM private-key KMS decrypt fail | aws-sdk error | Outbound dropped + sev-2 | KMS investigation |
| Spam score above threshold | Stalwart classifier | Quarantined to Trash; memory row | None — designed |
| Hard bounce | bounce report → bounce_log | Tracked; reputation gauge update | None — designed |
| Sustained > 1% bounce rate | OTel alarm | sev-3 | Investigate sender reputation |
| Thread merge race (two messages same thread_id) | UPSERT idempotent | None | Designed |
| Cross-tenant FK | RLS | 0 rows | Designed |
| RLS bypass attempt | `USING` predicate | 0 rows | Designed |
| Bcc accidentally exposed in to_addresses | handler test | CI fails | Fix adapter |
| Audit row commit fails | tx rollback | metadata not persisted; SMTP 421 | memory_writer diagnosis |
| body_sha256 mismatch S3 ETag | reconciliation job (FR-EMAIL-2xx) | sev-2 alarm | Investigate; possible corruption |
| Stalwart admin API unauthorised | CLI errors out | Service unavailable | Re-credential per docker secret |
| ManageSieve port blocked at firewall | client cannot upload filters | Filters not updated | Operator opens port |
| JMAP HTTP/2 frame too large | proxy returns 431 | Caller chunks | None — designed |
| DKIM key rotation in progress + outbound queued | active key sync atomic | Old key still valid | Designed (no flap) |
| User mailbox quota exceeded | Stalwart 552 | Sender retries / drop | Quota mgmt at FR-EMAIL-2xx |
| Stalwart upgrade requires migration | upgrade docs | Operator runs migration | docs |
| Concurrent SMTP listener crash | tokio task supervisor | Restart; ~5s gap | Designed |
| TLS cert expiring | OBS alarm at 14 days | Operator rotates per FR-AUTH-2xx | Standard rotation |
| Postgres `tenant_residency` row missing | residency lookup fails | SMTP 421 | Provision tenant |
| OTel exporter down | metrics buffered locally | None visible | OTel restore |
| Bounce_log row mid-tx fails | tx rollback | None | Designed |
| Graceful shutdown drain timeout | force-abort after 30s | in-flight messages drop | Designed |
| Stalwart container OOM | docker kill | Stalwart restarts; ~5s gap | Resource sizing review |
| message_metadata.thread_id orphaned (thread_metadata missing) | FK ON DELETE RESTRICT | UPSERT keeps thread alive | Designed |

---

## §11 — Implementation notes

- **Stalwart is the leaf service** — adapter layer reads webhook events + writes metadata. Don't reimplement protocol code; rely on Stalwart for RFC compliance.
- **Postgres backend mandatory at production** — RocksDB is for dev only; cluster + RLS + backups depend on Postgres.
- **S3 + KMS bodies separate from Postgres** — keeps PII-heavy bodies out of the cluster's backup envelope.
- **Per-tenant residency routed at adapter layer** — Stalwart sees a single bucket config; the adapter overrides per write via the tenant directory tag.
- **Per-tenant DKIM keys** — table is source of truth; Stalwart's keystore is a synced cache. Keystore filesystem path: `/etc/stalwart/keys/<tenant-uuid>/cyberos.pem`.
- **Append-only metadata** — status transitions create new rows linked via `prior_message_id`; bounce events are pure inserts.
- **SHA-256 body integrity verified at write** — `body_sha256_hex` computed by adapter from raw bytes; compared against S3 ETag (S3 computes ETag as MD5 by default; we compute SHA-256 separately for integrity).
- **`stalwart_message_id` BIGINT** — Stalwart's internal id, useful for joining adapter-side metadata back to Stalwart's own message store.
- **JMAP `threadId`** — RFC 8621 algorithm; Stalwart computes; we mirror as `thread_id TEXT`.
- **Adapter-side validation duplicates DB CHECK** — defence in depth; both layers reject body > 25 MB.
- **MTA-STS = "enforce" outbound** — sender-side strict mode; receiver-side is the peer's responsibility.
- **DANE = "opportunistic" outbound** — many peers lack DNSSEC; opportunistic uses DANE when available, falls back to STARTTLS otherwise.
- **Spam classifier threshold 5.0** — Stalwart default; tunable via env if a tenant has unusually high false-positive rate.
- **Quarantine → Trash with 30-day retention** — gives users time to recover; standard mail-UX pattern.
- **Bounce_kind enum (hard | soft | transient)** — RFC 3463 enhanced status maps: 5.x.x = hard, 4.x.x = soft/transient (depending on remote code).
- **`bcc_addresses` separate column** — RFC 5322 mandates Bcc be stripped from delivered body; adapter records envelope-level Bcc in this column for sender's view.
- **`spam_score_band` cardinality bounded** — Prometheus labels: 5-7, 7-10, 10+ (only spam-classified messages get banded; ham messages have no spam_score_band label).
- **`participant_addresses` array on thread_metadata** — append-only growing list; useful for "who has been on this thread?" queries.
- **`normalise_subject`** — strips `Re: `, `Fwd: `, leading/trailing whitespace; for thread-merge heuristics (slice 2; this FR uses Stalwart's threadId).
- **`cyberos-email-cli provision` is slice-1 only** — FR-EMAIL-002 replaces with JWT-driven JIT provisioning.
- **`status: dropped` covers outbound drop (DKIM missing, MTA-STS fail, etc.)** — distinct from `bounced` (peer rejected) and `delivered` (peer accepted).
- **`MessageStatus::Sent` transitions to `Delivered` on bounce-window-passed** — at slice 1, sent stays as sent (we don't implement the bounce window timer); FR-EMAIL-2xx ships the transition.
- **Stalwart healthcheck**: `curl http://localhost:8080/admin/health` returns 200 when ready; our `email_stalwart_up` gauge scrapes.
- **Docker compose includes minio** for local-dev S3; production uses real S3.
- **`Bcc` privacy concerns**: per-recipient view should NOT show other Bcc'd addresses; this FR ships the column; UI (FR-EMAIL-003) enforces visibility.

---

*End of FR-EMAIL-001.*
