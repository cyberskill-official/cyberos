---
fr_id: FR-EMAIL-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 11
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per AUTHORING.md §0)
---

## §1 — Verdict summary

FR-EMAIL-001 ships the Stalwart mail-server deployment + Postgres metadata mirror + S3+KMS body storage + per-tenant residency routing + DKIM per-tenant keystore. Scope: 26 §1 normative clauses covering Stalwart v0.10.x deployment + protocol endpoints (SMTP 25/465/587, IMAP 143/993, ManageSieve 4190, JMAP /jmap), Postgres backend, S3 blob store, residency-pinned per-tenant routing, message_metadata + thread_metadata + bounce_log + dkim_keys tables with RLS + append-only SQL grant, 2 closed Postgres enums (message_direction 3, message_status 6), 5 BRAIN audit kinds with PII scrubbing of addresses, MTA-STS + DANE outbound enforcement, spam quarantine at score ≥ 5.0, per-tenant DKIM RSA-2048 keys with rotation history, Bcc separate column, 25MB body cap, graceful shutdown 30s drain, slice-1 CLI provisioning. 22 rationale paragraphs. §3 contains: Stalwart TOML config, 3 migrations (messages + bounce_log + dkim_keys), residency resolver, inbound adapter. 27 ACs. 35 failure-mode rows. 25 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Embedded RocksDB backend at production
First-pass used Stalwart default. Resolved: §1 #2 + DEC-301 + Postgres backend forced; RLS-integrated.

### ISS-002 — Bodies in Postgres
First-pass stored bodies in metadata. Resolved: §1 #1 + DEC-302 + DEC-311 + S3+KMS only; PII isolation.

### ISS-003 — Single bucket for all tenants
First-pass had no residency routing. Resolved: §1 #12 + DEC-306 + residency-pinned per-tenant bucket; fail-closed cross-region.

### ISS-004 — Single DKIM key shared
First-pass shared a domain key. Resolved: §1 #5 + DEC-304 + per-tenant `dkim_keys` table + rotation history; AC #7 + #8.

### ISS-005 — Outbound MTA-STS not enforced
First-pass STARTTLS opportunistic only. Resolved: §1 #6 + DEC-305 + MTA-STS enforce mode; DANE opportunistic; AC #15 + #16.

### ISS-006 — Metadata mutable
Resolved: §1 #11 + AUTHORING.md rule 12 + `REVOKE UPDATE, DELETE ON message_metadata, bounce_log FROM cyberos_app`; AC #11 + #12.

### ISS-007 — PII in BRAIN audit (raw addresses)
First-pass logged from/to addresses unhashed. Resolved: §1 #14 + DEC-310 + SHA-256[..16] hash; FR-BRAIN-111 scrubbing.

### ISS-008 — Body integrity unverified
First-pass trusted Stalwart blob storage. Resolved: §1 #25 + `body_sha256_hex` recorded + S3 ETag comparison at write.

### ISS-009 — Bcc leaked into to/cc arrays
Resolved: §1 #26 + separate `bcc_addresses TEXT[]` column with RLS additional clause (visible only to sender's view).

### ISS-010 — No graceful shutdown
First-pass abrupt termination dropped in-flight SMTP. Resolved: §1 #24 + 30s drain + 421 transient on new connections.

### ISS-011 — Bounce rate not monitored
First-pass dropped bounces silently. Resolved: §1 #17 + DEC-309 + bounce_log append-only + sustained > 1% alarm.

## §3 — Resolution

All 11 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (Stalwart deployment × Postgres backend × S3+KMS bodies × per-tenant residency × per-tenant DKIM × MTA-STS+DANE enforcement × append-only metadata + bounce_log × spam quarantine × 5 BRAIN audit kinds × Bcc privacy × body integrity × graceful shutdown × CLI provisioning), not by line targets.

---

*End of FR-EMAIL-001 audit.*
