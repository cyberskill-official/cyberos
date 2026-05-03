---
title: "EMAIL — Stalwart Mail Server core integration (JMAP/IMAP/SMTP, MTA-STS, DKIM, ARC, BIMI, per-tenant storage)"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P1 / 2026-Q4"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Integrate **Stalwart Mail Server** (Rust, AGPL-3.0 / commercial) as the core mail substrate of the EMAIL module — the only modern all-protocol mail server that ships JMAP, IMAP, POP3, SMTP, ManageSieve, MTA-STS, DANE, DKIM, ARC, and BIMI in a single binary. The integration runs Stalwart as a sibling service alongside the platform, peer-to-peer with the Mattermost fork (FR-CHAT-001), uses CyberOS AUTH (FR-AUTH-001) for identity via JMAP-OAuth bridge, mirrors per-tenant mailbox storage to Postgres for queryable indexing alongside Stalwart's native blob store, publishes every mail event to NATS on `cyberos.{tenant}.email.{entity}.{verb}`, and exposes a CyberOS-shaped GraphQL subgraph on top. The shell of the EMAIL module — without UX, AI, or CRM-seam features — ships in this FR; subsequent batch-03 FRs layer on the Missive-style UX, CaMeL anti-injection, AI features, etc.

## Problem

Email is the second-highest-volume communication channel after CHAT and the gateway through which most prompt-injection attacks arrive (PRD §9.4). Building a mail server is hard; modern protocol coverage (MTA-STS, DANE, DKIM, ARC, BIMI) is unforgiving; deliverability is reputational and slow to recover once damaged.

The PRD §9.4.1 commits to Stalwart specifically because it is the only modern Rust-based all-protocol mail server with a single binary and built-in storage; the alternatives (Postfix + Dovecot + OpenDKIM + a separate ARC implementation + a separate JMAP server) compound operational cost in a way our 10-engineer team cannot sustain. Three properties this FR must guarantee:

- **Per-tenant residency.** A Vietnamese tenant's mailbox bytes never traverse an AWS US region; Stalwart's storage backend is configurable per tenant.
- **Deliverability from day one.** SPF + DKIM + DMARC + MTA-STS + ARC + BIMI must be correct from first send or the platform's external email reputation collapses (and every team using Gmail-fallback signatures becomes the cleanup work for months).
- **AUTH-equivalent identity.** A Member signed in via passkey at `app.cyberos.world` must reach their mailbox without a separate password ceremony; agent clients (FR-AUTH-001 §"Agent authentication") must reach the same mailbox under the same RBAC.

The P1 → P2 exit gate (PRD §14.2.3) requires "EMAIL has fully replaced Gmail for at least 21 consecutive days." That cannot happen if Stalwart's deliverability story is unproven.

## Proposed Solution

The shape of the answer is Stalwart deployed as a Kubernetes StatefulSet with per-tenant storage volumes, a CyberOS-side `email` GraphQL subgraph that proxies Stalwart's JMAP API, an AUTH bridge that converts CyberOS OAuth tokens to Stalwart sessions, NATS event publishing via Stalwart's webhook plugin, and a deployment harness for the deliverability primitives (SPF, DKIM, DMARC, MTA-STS, BIMI).

**Stalwart deployment.** A StatefulSet named `cyberos-email-stalwart` with one replica per residency region (Singapore for vn/sg tenants, Frankfurt for eu, Ohio for us). Each replica:

- Runs Stalwart Server (current stable line as of 2026-Q4; we pin a tag and update on a quarterly cadence).
- Mounts a persistent volume per tenant for the mailbox blob store; a single-replica StatefulSet keeps the storage layout simple at P1 internal scale (CyberSkill alone). At P3 multi-tenant scale, the StatefulSet is sharded by tenant residency.
- Mounts the per-tenant KMS key (FR-CP-002) for at-rest encryption of the blob store; encryption is enforced at the volume level via LUKS plus Stalwart's own encrypted-blob-store mode.
- Exposes JMAP at `https://email.cyberos.world/jmap/{tenant-slug}/`, IMAP at port 993 (TLS), SMTP submission at port 587 (STARTTLS) and 465 (TLS), inbound SMTP at port 25.

**Storage layout.** Stalwart's native storage uses RocksDB for indexes and a content-addressed blob store for messages. CyberOS adds:

- A Postgres mirror in `email.message_index` for fast cross-mailbox queries (e.g. "all messages mentioning Acme Corp across the team's shared inboxes"). The mirror is populated by a NATS consumer subscribing to `cyberos.{tenant}.email.message.received|sent|moved|deleted`.
- A bidirectional bridge: the CyberOS mirror is a *read* surface; Stalwart's blob store is the source of truth. Writes (delete, move, mark-read) go through Stalwart's JMAP and NATS-replay back into the mirror.

**Postgres mirror schema.**

```sql
CREATE SCHEMA email;

CREATE TABLE email.message_index (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  stalwart_blob_id TEXT NOT NULL,                -- the canonical content-addressed ID in Stalwart
  jmap_id TEXT NOT NULL,                          -- Stalwart's JMAP message ID
  mailbox_id UUID NOT NULL,                       -- references email.mailbox
  thread_id TEXT NOT NULL,                        -- JMAP thread ID
  message_id_header TEXT,                         -- RFC 5322 Message-ID
  in_reply_to TEXT,
  refs TEXT[],
  from_addresses TEXT[] NOT NULL,
  to_addresses TEXT[],
  cc_addresses TEXT[],
  bcc_addresses TEXT[],
  reply_to_addresses TEXT[],
  subject TEXT,
  preview TEXT,                                   -- the first ~256 chars of the plaintext body
  body_pgrn TSVECTOR_TYPE NOT NULL,               -- PGroonga-indexed Vietnamese-aware FTS
  size_bytes BIGINT NOT NULL,
  is_received BOOLEAN NOT NULL,                   -- true for inbound, false for outbound
  is_draft BOOLEAN NOT NULL DEFAULT false,
  has_attachments BOOLEAN NOT NULL DEFAULT false,
  occurred_at TIMESTAMPTZ NOT NULL,
  flags TEXT[],                                   -- "seen", "answered", "flagged", custom labels
  spam_score REAL,                                -- populated by Stalwart's milter
  spf_pass BOOLEAN,
  dkim_pass BOOLEAN,
  dmarc_pass BOOLEAN,
  arc_chain_validates BOOLEAN,
  layer3_doc_id UUID,                             -- optional FK to brain.l3_doc once ingested
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX email_msg_tenant_thread_idx ON email.message_index (tenant_id, thread_id, occurred_at DESC);
CREATE INDEX email_msg_pgrn_idx          ON email.message_index USING pgroonga (body_pgrn);
CREATE INDEX email_msg_from_idx          ON email.message_index USING gin (from_addresses);
CREATE INDEX email_msg_to_idx            ON email.message_index USING gin (to_addresses);

CREATE TABLE email.mailbox (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  jmap_id TEXT NOT NULL,
  member_id UUID,                                 -- null for shared inboxes
  name TEXT NOT NULL,                             -- "INBOX", "Sent", "Drafts", "support@", "sales@", etc.
  mailbox_kind TEXT NOT NULL,                     -- "personal" | "shared" | "system"
  parent_id UUID,
  message_count INT NOT NULL DEFAULT 0,
  unread_count INT NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**AUTH ↔ Stalwart bridge.** A small `cyberos-email-auth-bridge` service translates CyberOS OAuth tokens to Stalwart sessions:

1. Member signs in to `app.cyberos.world` via passkey (FR-AUTH-001).
2. Member opens the EMAIL Module-Federation remote at `/email`.
3. The remote calls `cyberos-email-auth-bridge.exchange(access_token)` which validates the access token, resolves Member + Tenant, and asks Stalwart to create a JMAP session bound to the Member's mailbox set.
4. Stalwart returns a JMAP session token; the bridge proxies it back to the remote's GraphQL subgraph; the remote uses the GraphQL subgraph for all reads, with the JMAP session held server-side.

For IMAP/SMTP clients (the founder's mobile mail app, e.g.), an OAuth 2.1 token-exchange flow lets the client authenticate via passkey on `app.cyberos.world` once and receive an IMAP/SMTP-specific bearer token; the token is rotated on the same 30-day refresh cadence as the rest of AUTH (FR-AUTH-003).

**GraphQL subgraph (`email`).** Mirrors a curated subset of JMAP:

```graphql
type Query {
  emailMailboxes(memberOnly: Boolean = false): [EmailMailbox!]!
  emailThreads(mailboxId: ID!, after: String, first: Int = 50): EmailThreadConnection!
  emailThread(id: ID!): EmailThread
  emailMessages(threadId: ID!): [EmailMessage!]!
  emailMessage(id: ID!): EmailMessage
  emailSearch(query: String!, mailboxIds: [ID!], from: String, to: String,
              hasAttachments: Boolean, since: DateTime, until: DateTime,
              first: Int = 50): EmailSearchHits!
}

type Mutation {
  emailComposeDraft(to: [String!]!, cc: [String!], bcc: [String!],
                    subject: String!, body: EmailBodyInput!,
                    inReplyTo: ID, attachments: [EmailAttachmentInput!]): EmailMessage!
  emailUpdateDraft(id: ID!, ...): EmailMessage!
  emailSendMessage(draftId: ID!): EmailMessage!
  emailMoveThread(threadId: ID!, mailboxId: ID!): EmailThread!
  emailMarkRead(messageIds: [ID!]!, read: Boolean!): Int!
  emailDeleteThread(threadId: ID!): Boolean!
  emailFlagThread(threadId: ID!, flag: String!, value: Boolean!): EmailThread!
}

type Subscription {
  emailMailboxStream(mailboxId: ID!): EmailEvent!
}
```

Persisted-queries discipline applies (FR-INFRA-001 §"Apollo Federation v2 supergraph").

**Deliverability primitives.** The EMAIL module ships pre-configured infrastructure for outbound deliverability; deployment is automated via Terraform module `cyberskill/email-deliverability`:

- **SPF.** TXT record `v=spf1 include:_spf.cyberos.world ~all` set on every tenant sending domain. The `_spf.cyberos.world` record enumerates the Stalwart cluster's egress IPs by region.
- **DKIM.** Stalwart signs with `cyberskill._domainkey.{tenant-domain}` (selector `cyberskill`). Each tenant gets a 2048-bit RSA key generated at provisioning; the public key TXT record is published in the tenant's DNS by the deployment automation. Annual rotation is automated.
- **DMARC.** TXT record at `_dmarc.{tenant-domain}` with policy `v=DMARC1; p=quarantine; rua=mailto:dmarc-reports@cyberos.world; ruf=mailto:dmarc-forensic@cyberos.world; pct=100`. Reports are aggregated and surfaced in the Compliance Cockpit; spike alerts route to OBS.
- **MTA-STS.** Stalwart serves the MTA-STS policy at `https://mta-sts.{tenant-domain}/.well-known/mta-sts.txt` with `mode: enforce`; the corresponding `_mta-sts.{tenant-domain}` TXT record is published by deployment.
- **DANE.** TLSA records for the Stalwart cluster are published per region; downstream MTAs that support DANE prefer the cryptographic binding to PKI.
- **ARC.** Stalwart's ARC implementation seals every relayed message; receiving servers can verify the upstream chain.
- **BIMI.** A tenant configures their logo (SVG-Tiny PS) + Verified Mark Certificate; the platform publishes the BIMI TXT record. Optional but improves trust signals in Gmail/Apple Mail.

**Bounce + reputation handling.** Stalwart's bounce-processor classifies bounces; CyberOS adds:

- A suppression list per tenant: hard bounces auto-suppress; soft bounces increment a counter; persistent soft bounces auto-suppress with manual override.
- A daily report of suppression-list growth; spikes trigger a sev-1 alert.
- Reputation monitoring via the major postmaster-tools APIs (Google Postmaster Tools, Microsoft SNDS) with alerts on reputation degradation.

**Rate limits.** Per-tenant outbound caps: 600 messages/hour and 2,000/day at P1 internal scale; configurable per plan from P3+. Excess is queued; queue-depth alerts route to the founder if a tenant approaches the cap.

**Audit integration.** Every send, every delete, every move, every mailbox creation writes an audit row in scope `email.{tenant}` with `from_addresses`, `to_addresses`, subject hash (the subject itself is potentially personal data and is denylisted from raw audit-row payload), and the JMAP message ID.

**MCP tool surface (read-mostly).**

- `cyberos.email.list_mailboxes` (read).
- `cyberos.email.search(query, ...)` (read; same parameters as GraphQL).
- `cyberos.email.list_threads(mailbox_id, ...)` (read).
- `cyberos.email.get_message(id)` (read).
- `cyberos.email.compose_draft(to, cc, bcc, subject, body)` (`destructive: false`; produces a draft, not a send).
- `cyberos.email.send_message(draft_id)` (`destructive: true; requires_confirmation: true; sensitivity: high` — step-up auth required per FR-AUTH-003).

The `send_message` tool requires both the destructive-confirmation gate *and* a step-up token because outbound email is the highest-leverage external action a Member can take through CyberOS.

## Alternatives Considered

- **Postfix + Dovecot + OpenDKIM + Cyrus.** Rejected: four moving parts to operate; ARC support is a separate add-on; JMAP is via a third-party bridge. Operational cost is multiples of Stalwart.
- **Mailgun / Postmark / SendGrid for transactional + Gmail for personal.** Rejected: residency story breaks; the platform's own UX cannot embed a hosted provider's surface; the lock-in posture conflicts with Bet 1.
- **Use Mattermost's email integration only (no separate EMAIL module).** Rejected: Mattermost's email is for notification delivery, not a full mail client; PRD §9.4 explicitly scopes EMAIL as its own module.
- **Defer Stalwart adoption to P3 in favour of a thin wrapper around Gmail.** Rejected: PRD §14.1.2 explicitly defers EMAIL to P1 *because* the team will continue using Gmail through P0; deferring further past P1 contradicts the gate.
- **Run Stalwart as a single shared instance for all tenants.** Rejected: per-tenant volume isolation is the path to verifiable per-tenant erasure; shared instances would leak deletion semantics across tenants.

## Success Metrics

- **Primary metric.** P1 mid-sprint demo passes: (1) a Member signs in via passkey and reads their mailbox at `/email` end-to-end; (2) outbound message from `stephen@cyberskill.world` reaches a synthetic external mailbox with SPF + DKIM + DMARC all `pass`; (3) inbound message to `info@cyberskill.world` lands in the shared inbox and surfaces in the GraphQL subgraph; (4) Stalwart cluster passes a synthetic load test of 100 messages/min for 30 minutes with zero loss.
- **Deliverability metric.** Outbound DMARC pass rate ≥ 99% on a 14-day window; reputation green at Google Postmaster Tools and Microsoft SNDS by P1 → P2 exit.
- **Reliability metric.** Stalwart cluster uptime ≥ 99.9% over 30 days.
- **Latency metric.** JMAP message-list query p95 ≤ 400 ms over a 50,000-message mailbox.

## Scope

**In-scope (P1 sprint cluster S1-1 to S1-2).**
- Stalwart StatefulSet deployed in the canonical residency region (Singapore for CyberSkill).
- Per-tenant storage volume + LUKS + KMS-key encryption.
- AUTH bridge for JMAP session minting.
- Postgres mirror with the `message_index` and `mailbox` tables, populated via NATS consumer.
- GraphQL `email` subgraph with the queries + mutations + subscription above.
- Deliverability automation: SPF, DKIM, DMARC, MTA-STS, DANE, ARC, BIMI.
- Bounce-processing + suppression list.
- Reputation monitoring via Postmaster Tools + SNDS.
- Per-tenant rate limits.
- Audit integration in scope `email.{tenant}`.
- MCP tools.
- The `cyberskill.world` and `cyberos.world` mail domains active and live for the team.

**Out-of-scope (deferred to subsequent batch-03 FRs).**
- Missive-style shared inbox UX (FR-EMAIL-002).
- CaMeL anti-injection on inbound (FR-EMAIL-003).
- AI features: thread summarisation, suggested replies, auto-categorisation (FR-EMAIL-004).
- Vietnamese-aware composition (FR-EMAIL-005).
- CRM bidirectional integration (FR-EMAIL-006; CRM lands in batch-05).
- PROJ promote-to-task (FR-EMAIL-007; PROJ lands in batch-04).
- Gmail migration (FR-EMAIL-009).
- Attachment scanning + S/MIME (FR-EMAIL-010).

## Dependencies

- FR-INFRA-001 (Postgres + NATS + K8s + Cloudflare DNS).
- FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 (identity, audit, step-up auth on send).
- FR-MCP-001 (destructive-confirmation gate + step-up integration).
- FR-CP-002 (per-tenant KMS keys).
- FR-OBS-001 / FR-OBS-002 (Stalwart metrics, alerts, dashboards, runbooks).
- DNS control over `cyberskill.world` and `cyberos.world` for SPF/DKIM/DMARC/MTA-STS/DANE/BIMI records.
- A Verified Mark Certificate provider (Entrust, DigiCert) for BIMI — optional but on the roadmap.
- Compliance: PDPL Decree 13 (mail content is personal data); EU AI Act (no AI in this FR — see FR-EMAIL-003 + EMAIL-004 for the AI-classified surfaces); SOC 2 CC6 + CC7.
- Locked decisions referenced: DEC-077 (Stalwart as the mail core), DEC-078 (per-tenant volume + LUKS + KMS encryption at rest), DEC-079 (JMAP for native + IMAP/SMTP for legacy clients).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The Stalwart core integration is deterministic mail infrastructure. AI-related EMAIL surfaces (CaMeL ingestion, suggested replies, thread summaries) ship in dedicated FRs (FR-EMAIL-003, FR-EMAIL-004) where they are correctly classified.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
