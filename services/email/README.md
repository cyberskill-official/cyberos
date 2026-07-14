# CyberOS EMAIL — Stalwart adapter + per-tenant DKIM + residency-pinned bodies

**Status:** TASK-EMAIL-001 + TASK-EMAIL-004/005/009/011 shipped as service slices — Stalwart container config, per-tenant DKIM keystore, residency-pinned S3+KMS body storage, delivery-auth hardening, CaMeL quarantine gate, outbound confirm-before-send queue, DSAR export jobs, HTTP routes, and memory audit row builders.
**Mail server:** [Stalwart Mail Server](https://stalw.art) v0.10.x (AGPL-3.0; container pinned in `docker/stalwart.toml`).
**Protocols:** JMAP + IMAP + SMTP + POP3 + ManageSieve + MTA-STS + DANE + DKIM + ARC + BIMI.

---

## §1 — Architecture

```
                ┌─────────────────────────────────────────────────────┐
                │  Stalwart container (mail server)                   │
                │   smtp 25/465/587 · imap 143/993 · jmap 8080        │
                │   managesieve 4190                                  │
                │   storage.data = pg  storage.blob = s3+kms          │
                │   spam quarantine threshold = 5.0                   │
                └────────────┬──────────────────────────────┬─────────┘
                             │  webhook                     │
                             ▼                              ▼
                ┌───────────────────────┐         ┌─────────────────┐
                │ cyberos-email gateway │         │ Postgres        │
                │  (this Rust crate)    │         │  stalwart_meta  │
                │                       │         │  message_meta   │
                │  - inbound adapter    │         │  thread_meta    │
                │  - outbound adapter   │         │  bounce_log     │
                │  - DKIM keystore sync │         │  dkim_keys      │
                │  - residency resolver │         │  tenant_resid.  │
                │  - memory audit emit   │         └─────────────────┘
                │  - REST /v1/email/*   │
                └────────────┬──────────┘
                             ▼
                ┌────────────────────────────────┐
                │ S3 buckets (per residency)     │
                │  cyberos-email-vn-1-bodies     │
                │  cyberos-email-sg-1-bodies     │
                │  cyberos-email-eu-1-bodies     │
                │  cyberos-email-us-1-bodies     │
                │ KMS keys per residency         │
                └────────────────────────────────┘
```

---

## §2 — What ships

| Concern | Status | Where |
|---|---|---|
| Stalwart container + config | ✅ | `docker/Dockerfile` + `docker/stalwart.toml` |
| Postgres metadata schema (messages/threads) | ✅ | `migrations/0001_messages.sql` |
| Append-only bounce_log | ✅ | `migrations/0002_bounce_log.sql` |
| Per-tenant DKIM keystore | ✅ | `migrations/0003_dkim_keys.sql` + `src/dkim/keystore.rs` |
| Residency routing | ✅ | `migrations/0004_residency_routing.sql` + `src/residency.rs` |
| Inbound webhook adapter | ✅ | `src/stalwart_adapter/inbound.rs` |
| Outbound submission adapter | ✅ | `src/stalwart_adapter/outbound.rs` |
| 5 memory audit row builders | ✅ | `src/audit/email_events.rs` |
| Health + per-message status REST | ✅ | `src/handlers/status.rs` |
| `cyberos-email-cli provision` | ✅ | `src/bin/cli.rs` |
| Tests — types + residency + DKIM + audit + adapter + subject normalisation | ✅ | `tests/` + inline `#[cfg(test)]` |
| DKIM+ARC+BIMI delivery hardening | ✅ | `migrations/0005_delivery_auth.sql` + `src/delivery_auth.rs` + `src/handlers/delivery_auth.rs` |
| Outbound 1:1 confirm-before-send queue | ✅ | `migrations/0006_outbound_messages.sql` + `src/outbound.rs` + `src/handlers/outbound.rs` |
| DSAR message export aggregation | ✅ | `migrations/0007_dsar_export_jobs.sql` + `src/dsar.rs` + `src/handlers/dsar.rs` |
| CaMeL dual-LLM quarantine gate | ✅ | `migrations/0011_camel_audit.sql` + `src/camel.rs` + `src/handlers/camel.rs` |

---

## §3 — What remains deferred (per FR §9)

| Concern | FR | Notes |
|---|---|---|
| JMAP / IMAP / SMTP JWT bridge | TASK-EMAIL-002 | Stalwart's auth delegates to AUTH JWT validator |
| Shared-inbox UX (assignment, comments, snooze) | TASK-EMAIL-003 | TypeScript SPA + backend handlers |
| Tracked-domain auto-log to CRM | TASK-EMAIL-006 | CRM activity-feed integration |
| Convert thread → PROJ issue | TASK-EMAIL-007 | One-click issue derivation |
| Bulk send approval | TASK-EMAIL-010 | CFO/marketing dual-token |

---

## §4 — Build + test

```bash
# Compile the gateway crate (uses workspace deps).
cd services && cargo build -p cyberos-email

# Run the unit tests (no DB required).
cd services && cargo test -p cyberos-email --lib

# Bring up the full local stack — Stalwart + Postgres + Minio.
docker compose -f services/email/docker/compose.yml up
```

The protocol-endpoint integration tests under `tests/protocol_endpoints_test.rs`
require the compose stack to be running.

---

## §5 — Operator commands

```bash
# Provision a local-part + ensure DKIM key is active for the tenant.
DATABASE_URL=postgres://... cyberos-email-cli provision \
    --tenant-id 5e8f1d2a-... \
    --local-part support \
    --display-name "Support Team"

# Rotate the active DKIM key for a tenant (atomic; emits memory row).
DATABASE_URL=postgres://... cyberos-email-cli rotate-dkim \
    --tenant-id 5e8f1d2a-... \
    --selector cyberos

# Debug residency routing (useful when investigating cross-residency suspicions).
DATABASE_URL=postgres://... cyberos-email-cli resolve-residency \
    --tenant-id 5e8f1d2a-...
```

---

## §6 — memory audit row kinds emitted

TASK-EMAIL-001 defines the core message row kinds; TASK-EMAIL-004/005/009/011 add service-specific helper rows:

| Kind | Trigger | PII handling |
|---|---|---|
| `email.message_received` | Stalwart inbound webhook → `on_inbound` | `from_hash16` + `to_hash16` (SHA-256[..16]) |
| `email.message_sent` | `on_outbound` after DKIM verify | Same |
| `email.message_bounced` | bounce_log append | No address PII |
| `email.message_quarantined` | spam_score ≥ 5.0 | Same as received, plus `spam_score_band` |
| `email.dkim_key_rotated` | `rotate_key` | `rotated_by_subject_id_hash16` |
| `email.delivery_auth_*` | DKIM/ARC/BIMI DNS, signing, and verification decisions | Tenant/domain metadata only |
| `email.camel_*` | CaMeL quarantine parse, trust-list, and execute decisions | Sanitised variables only |
| `email.outbound_*` | Draft, confirm, send, bounce, complaint, suppression events | Recipient address hashed where emitted |
| `email.dsar_*` | DSAR export request/completion | Attachment refs only; no raw body in memory |

The core row body is defined by `src/audit/email_events.rs` (`EmailAuditRow`).
Postgres `message_metadata` carries raw addresses (RLS-scoped); memory holds
only the 16-char hash prefix per §1 #14.

---

## §7 — Spec divergences

See `docs/tasks/email/TASK-EMAIL-001-stalwart-deployment.audit.md`
§10.6 for the GUC-name divergence (`app.current_tenant_id` from
TASK-AUTH-003 §10.6 vs. the spec's `auth.tenant_id`).

---

## §8 — Layout

```
services/email/
├── Cargo.toml
├── README.md                       this file
├── AGENTS.md                       module-level agent instructions
├── docker/
│   ├── Dockerfile                  Stalwart container
│   ├── stalwart.toml               Stalwart config (data+blob+listeners+DKIM+MTA-STS)
│   └── compose.yml                 local-dev compose (Postgres + Minio + Stalwart + gateway)
├── migrations/
│   ├── 0001_messages.sql           message_metadata + thread_metadata + enums + RLS
│   ├── 0002_bounce_log.sql         append-only bounce log + RLS
│   ├── 0003_dkim_keys.sql          per-tenant keystore + rotation history + RLS
│   ├── 0004_residency_routing.sql  tenant_residency table + RLS
│   ├── 0005_delivery_auth.sql      tenant DNS setup + delivery auth event rows
│   ├── 0006_outbound_messages.sql  outbound queue + suppression + delivery events
│   ├── 0007_dsar_export_jobs.sql   DSAR jobs + subject/attachment refs
│   └── 0011_camel_audit.sql        CaMeL variable/trust/audit tables
├── src/
│   ├── lib.rs                      crate root
│   ├── types.rs                    EmailMessage, EmailThread, BounceEvent, DkimKey, enums
│   ├── errors.rs                   EmailError + structured codes
│   ├── residency.rs                TASK-AI-016 resolver + fail-closed assert
│   ├── delivery_auth.rs            TASK-EMAIL-004 DKIM/ARC/BIMI hardening helpers
│   ├── camel.rs                    TASK-EMAIL-005 CaMeL quarantine gate
│   ├── outbound.rs                 TASK-EMAIL-009 outbound confirm/send queue
│   ├── dsar.rs                     TASK-EMAIL-011 DSAR export helpers
│   ├── dkim/
│   │   ├── mod.rs
│   │   └── keystore.rs             provision + rotate + KmsEncryptor trait + MockKmsEncryptor
│   ├── stalwart_adapter/
│   │   ├── mod.rs
│   │   ├── inbound.rs              webhook adapter + BlobStore trait + MemoryBlobStore
│   │   └── outbound.rs             submission adapter
│   ├── repo/
│   │   ├── mod.rs
│   │   ├── messages.rs             upsert_thread + insert_message + list + normalise_subject
│   │   └── bounce_log.rs           append-only writer + 24h rate
│   ├── audit/
│   │   ├── mod.rs
│   │   └── email_events.rs         5 row builders + hash16 + spam_band
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── status.rs               healthz + message_status REST
│   │   ├── delivery_auth.rs        DNS setup / verify / BIMI handlers
│   │   ├── outbound.rs             compose / send / suppression handlers
│   │   ├── dsar.rs                 export / job status handlers
│   │   └── camel.rs                execute / trust-list / audit-log handlers
│   └── bin/
│       ├── server.rs               cyberos-email HTTP server
│       └── cli.rs                  cyberos-email-cli operator entry
└── tests/
    ├── residency_pin_test.rs       TASK-EMAIL-001 §4 #9 + #10
    ├── audit_row_test.rs           TASK-EMAIL-001 §4 #5 + #13 + #25
    ├── inbound_quarantine_test.rs  TASK-EMAIL-001 §4 #13
    └── subject_normalisation_test.rs
```
