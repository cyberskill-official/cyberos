---
title: EMAIL — Capture surface · Genie draft · Thread-to-issue bridge · CyberOS
source: website/docs/modules/email/index.html
migrated: FR-DOCS-002
---

EMAIL is CyberOS's **mail server, shared inbox, and AI-native composition surface** in a single bundled module. The protocol stack is Stalwart (Rust, single binary) speaking JMAP / IMAP / SMTP / ManageSieve to the world. On top sits a Missive-style UX where a team can manage `support@cyberskill.world` together — assignment, internal comments, snooze, tagging. Every inbound body passes through a CaMeL quarantined LLM before any privileged CUO context can see it; the privileged side operates on the sanitised extraction (sender, recipient, subject, gist, action requests, entities), never on raw HTML. Outbound mail is DKIM-signed with per-tenant keys, ARC-stamped on forward, and tagged with BIMI for inbox-list branding. Threading is JMAP-native; search is PGroonga (Vietnamese-aware bigram); calendar is iCal. DSAR export bundles every message a subject participated in. 

Status

Planned

P1 · design phase

Core

Stalwart

Rust · AGPL-3.0 · single binary

UX layer

Missive-style

shared inbox · internal comments

Anti-injection

CaMeL

dual-LLM quarantine

Protocols

JMAP · IMAP · SMTP

\+ MTA-STS · DANE · DKIM · BIMI

Search

PGroonga

Vietnamese-aware bigram

Depends on

AUTH · memory · AI

\+ OBS · MCP

Est. LoC

~9,000

Rust + TS (Stalwart adapter + UI)

0

## The bigger picture — three strategic roles

EMAIL is structurally hostile territory. It is simultaneously: the second-highest-volume communication channel for any consultancy, the single most dangerous source of indirect prompt-injection attacks (EchoLeak / CVE-2025-32711 in May 2025 set the precedent), and the easiest way to log customer conversations to CRM + PROJ. The three strategic roles below are not nice-to-have features — they're the only architecture that handles those three realities at once. 

Role 1 · Capture surface

Inbound auto-log to CRM + PROJ thread-to-issue

Tracked domains auto-log every inbound + outbound message as a CRM activity. Thread-to-issue: AM clicks "Convert to issue" on a thread, the thread becomes a PROJ Issue with the email body as the issue description + reply chain as comments. Captures the consultancy memory layer that otherwise lives in nobody's inbox — and is gone the moment they leave. 

Role 2 · Genie draft

Ask Genie composes; AM approves before send

"Genie:" subject prefix OR sidebar action → CUO drafts a reply grounded in: the thread history (sanitised by CaMeL), the linked CRM account, memory memories about prior interactions with this account, and the relevant KB docs. Draft never auto-sent. AM reviews + edits + sends. The whole flow takes 30 seconds versus 5 minutes of writing. 

Role 3 · Outbound send + defence

DKIM + ARC + BIMI · CaMeL anti-injection

Outbound: per-tenant DKIM keys, ARC on forward, BIMI for inbox-list branding. Inbound: every body passes through the CaMeL quarantine LLM (no tools, no memory, produces structured extraction only) before the privileged CUO sees anything. EchoLeak-class attacks fail because the privileged agent never reads raw HTML or untrusted text. AM/CFO approval tokens required on bulk send. 

### EMAIL in the orchestration spine

flowchart LR EXT["External sender"] EMAIL["✉ EMAIL  
Stalwart · Missive UX · CaMeL quarantine"] CRM["🤝 CRM  
activity auto-log"] PROJ["📋 PROJ  
thread-to-issue"] CUO["🎯 CUO  
Genie draft skill"] AM["👤 AM  
review + send"] memory["🧠 memory  
thread history · CaMeL audit"] OBS["👁 OBS  
send latency · injection blocked"] EXT --> EMAIL EMAIL -- "CaMeL quarantine"-->|"sanitised extract"| memory EMAIL -- "tracked domain" --> CRM EMAIL -- "thread-to-issue" --> PROJ CRM --> EMAIL PROJ --> EMAIL EMAIL --> CUO CUO -- "draft" --> EMAIL EMAIL --> AM AM -- "send" --> EXT EMAIL --> OBS classDef hub fill:#e2e8f0,stroke:#334155,stroke-width:3px,color:#0f172a classDef mod fill:#e0e7ff,stroke:#3730a3 classDef memory fill:#fef6e0,stroke:#9c750a classDef ext fill:#fee2e2,stroke:#b91c1c class EMAIL hub class CRM,PROJ,CUO,AM,OBS mod class memory memory class EXT ext 

### Auto vs human-in-loop operations matrix

Operation| How it happens| Why this split  
---|---|---  
Inbound CaMeL quarantine| **Auto** on every inbound message| Protocol invariant; privileged context never sees raw body.  
Tracked-domain CRM auto-log| **Auto** by domain match| Stops the "log it in the CRM" tax; auto = no manual entry.  
Thread-to-issue conversion| **Manual** AM click| Not every thread becomes an Issue; AM intent required.  
Genie draft| **Auto** ; never auto-sent| Draft is suggestion; AM is accountable for prose + tone.  
Outbound send (1:1)| **Manual** AM action| Customer comms = relationship; human sends.  
Bulk send (≥ 10 recipients)| **Manual** AM + CFO/marketing approval| Anti-spam + brand-tone gate; never one-click bulk.  
DKIM / ARC / BIMI signing| **Auto** on outbound| Standards-driven; deterministic.  
Spam classification| **Auto** via Stalwart + ML| Trash routing; quarantine for borderline.  
Bounce + reputation handling| **Auto** \+ alarms| Email reputation is fragile; OBS surfaces dips.  
DSAR export| **Auto** bundle on request| Per-subject; chained audit; matches memory export pattern.  
  
1

## Why EMAIL exists

Three reasons to own the mail plane rather than outsource to a SaaS provider: (1) **injection defence** — most of the high-profile 2025 prompt-injection CVEs entered through email, and a hosted provider does not let you put a CaMeL quarantine between the body and the agent; (2) **shared-inbox UX** — teams managing `support@`, `sales@`, `billing@` need assignment, internal comments, and snooze, which Gmail / M365 implement only via shaky third-party add-ons; (3) **data residency** — Vietnamese tenants under Decree 53 must keep certain customer data on Vietnamese soil, which most hosted mail providers cannot promise. Stalwart gives us a credible Rust mail server, a Missive-style UX gives us the shared-inbox primitive, and CaMeL gives us a proof-shaped boundary against indirect prompt injection. 

🛡

CaMeL by default

Untrusted body → quarantined LLM (no tools, no memory) → structured extraction → privileged CUO. Never raw HTML into the agent's context window.

👥

Shared-inbox primitive

One address, many people. Assign a thread to a teammate. Comment internally without the customer seeing. Snooze until Monday. The Missive playbook.

🇻🇳

Vietnamese-first

PGroonga search handles VN bigram tokenisation; AI drafts respect Anh / Chị / Bạn salutations and TPHCM / Hà Nội regional sign-off conventions.

EchoLeak (CVE-2025-32711, May 2025) was the canonical demonstration: a malicious email tricked Microsoft 365 Copilot into exfiltrating data with zero user clicks. The attack class continued through April 2026. The lesson is not "patch the LLM" — it is "do not let the LLM see the raw body". CaMeL is the architecture that enforces that, and EMAIL is the module where that enforcement lives. 

2

## What it does — 5W1H2C5M

A structured decomposition of EMAIL's scope. Every cell traces back to and §11.2.3.

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is EMAIL?| A Stalwart-based mail server (JMAP / IMAP / SMTP / MTA-STS / DANE / DKIM / ARC / BIMI) plus a Missive-style shared-inbox UX, gated by a CaMeL dual-LLM filter on every inbound body. Single Rust deployment with a separate TypeScript SPA. Postgres for metadata; object storage (S3 + KMS) for bodies and attachments.  
**5W · Who**|  Who uses it?| **Members:** personal inboxes + assigned shared inboxes. **Customers:** external senders / recipients (never authenticate to CyberOS). **Agents:** CUO via MCP for draft-reply / categorise / snooze tools. **Owner:** CCO seat (interim CEO).  
**5W · When**|  When does it run?| Continuous SMTP listener (25 / 465 / 587). JMAP / IMAP for client sync. Inbound CaMeL scan runs on the receive path before any thread row is committed. Outbound DKIM signing happens at queue time. AI features (draft, summarise) run on user demand.  
**5W · Where**|  Where does it run?| P1: single region (SG-1) plus a VN-residency relay for Decree 53 customers. P3+: multi-region with active-active SMTP and an MX-record failover. Object storage and Postgres are tenant-tagged; `vn-residency=true` tenants land on `hanoi-1`.  
**5W · Why**|  Why own the plane?| Three reasons: (a) prompt-injection defence requires the CaMeL boundary, which a SaaS does not allow; (b) shared-inbox UX needs internal-comments and assignment primitives that Gmail does not provide natively; (c) Vietnamese data-residency obligations under Decree 53.  
**1H · How**|  How does it work?| Inbound: SMTP → DKIM / SPF / DMARC verify → spam triage → CaMeL quarantined extraction → thread merge by JMAP `threadId` → notification fan-out. Outbound: composer → CaMeL out-bound scan (block PII leaks) → DKIM sign → MTA-STS / DANE next-hop → SMTP send. AI: composer pulls draft from AI Gateway with persona-stamped JWT.  
**2C · Cost**|  Cost budget?| P1: ~$110 / month for SG-1 single-tenant pilot (Stalwart Fargate + RDS Postgres + S3 + Redis cache). 50-tenant: ~$420 / month. Per-message inbound CaMeL cost ~$0.0004 (gpt-4o-mini-equivalent extraction).  
**2C · Constraints**|  Constraints?| (a) AGPL-3.0 — Stalwart's licence means our distribution model is service-only. (b) DMARC alignment mandatory — strict policy from P0 · exit. (c) No body content into memory by default ((FR pending)); opt-in per Member. (d) CaMeL on every ingest path — non-negotiable.  
**5M · Materials**|  Stack?| Stalwart Mail Server (Rust) · axum HTTP API · sqlx · PostgreSQL 16 · PGroonga search · Redis 7 cache · S3 + KMS for bodies · CaMeL adapter on top of AI Gateway · OpenTelemetry SDK · React / TypeScript SPA · htmx + Alpine for the inbox shell.  
**5M · Methods**|  Method choices?| JMAP-native threading (no Reply-To header heuristics). PGroonga over tsvector (Vietnamese bigram quality wins). CaMeL dual-LLM (quarantined no-tools + privileged). Per-tenant DKIM keypair stored in KMS. ARC stamping on forward. Single-binary Stalwart deployment (no separate MTA / MDA / MUA processes).  
**5M · Machines**|  Deployment?| Fargate task per region for SMTP + JMAP + IMAP listeners. RDS Postgres for metadata. S3 + KMS for bodies and attachments. Redis for hot-thread cache. Per-tenant DKIM keypair in KMS.  
**5M · Manpower**|  Who maintains?| 0.5 FTE (CTO seat) at P1 launch. By P2+: CCO seat owns product; CTO owns infra; CSO consulted on every Stalwart upgrade.  
**5M · Measurement**|  How measured?| Deliverability (DMARC pass rate ≥ 98%), p95 send latency ≤ 2 s, p95 receive latency ≤ 5 s, CaMeL injection-block rate ≥ 99.5% on the test corpus, AI-draft acceptance ≥ 40% by Member.  
  
3

## Architecture

EMAIL has four planes: the protocol plane (Stalwart, talking SMTP / IMAP / JMAP outbound and inbound to the world); the policy plane (CaMeL filter + DKIM / SPF / DMARC verify + per-tenant ACL); the UX plane (the Missive-style SPA + an htmx-light fallback for slow networks); and the AI plane (draft, summarise, categorise — all routed through AI Gateway with the privileged CUO persona). The diagram shows the canonical inbound path with the CaMeL boundary highlighted. 

graph TB subgraph WORLD ["Internet"] SENDER["External sender  
(any SMTP source)"] RECIPIENT["External recipient"] end subgraph EDGE ["Edge / protocol plane"] SMTP["📨 Stalwart SMTP  
25 · 465 · 587"] JMAP["📡 Stalwart JMAP  
/jmap/"] IMAP["📬 Stalwart IMAP  
143 · 993"] MTA_STS["MTA-STS · DANE  
next-hop policy"] end subgraph POLICY ["Policy plane"] DKIM_VERIFY["DKIM / SPF / DMARC  
verify"] SPAM["Spam triage"] CAMEL["🛡 CaMeL quarantine  
no-tools LLM"] EXTRACT["Sanitised extraction:  
{from, to, subj, gist,  
actions, entities}"] DKIM_SIGN["DKIM sign  
per-tenant key (KMS)"] ARC["ARC stamp  
on forward"] end subgraph CORE ["EMAIL service (Rust)"] THREAD["Thread merger  
JMAP threadId"] INBOX["Shared-inbox  
assignment · comments"] SEARCH["PGroonga search  
(VN bigram)"] CAL["iCal · calendar"] SIEVE["ManageSieve  
rules engine"] end subgraph STORES ["Stores"] PG[("PostgreSQL  
threads · assignments  
RLS by tenant_id")] S3[("S3 + KMS  
bodies + attachments")] RED[("Redis 7  
hot threads · search cache")] KMS_K[("AWS KMS  
per-tenant DKIM keys")] end subgraph SINKS ["Sinks"] memory["🧠 memory  
summaries (opt-in) ·  
audit rows"] OBS["👁 OBS  
traces + DMARC reports"] CUO["🎯 CUO  
privileged agent"] AI["⚡ AI Gateway"] end SENDER --> SMTP SMTP --> DKIM_VERIFY DKIM_VERIFY --> SPAM SPAM --> CAMEL CAMEL --> EXTRACT EXTRACT --> THREAD THREAD --> PG THREAD --> S3 THREAD --> SEARCH THREAD --> memory THREAD --> INBOX THREAD --> RED INBOX --> CUO CUO --> AI AI --> CAMEL CAMEL --> AI JMAP --> THREAD IMAP --> THREAD SIEVE --> THREAD CAL --> THREAD THREAD --> DKIM_SIGN DKIM_SIGN --> KMS_K DKIM_SIGN --> ARC ARC --> MTA_STS MTA_STS --> RECIPIENT THREAD --> OBS classDef planned fill:#e2e8f0,stroke:#334155 classDef store fill:#f5f3ff,stroke:#7c3aed classDef sink fill:#f5ede6,stroke:#45210e classDef camel fill:#fee2e2,stroke:#b91c1c,stroke-width:3px class SMTP,JMAP,IMAP,MTA_STS,THREAD,INBOX,SEARCH,CAL,SIEVE,DKIM_VERIFY,SPAM,DKIM_SIGN,ARC planned class CAMEL,EXTRACT camel class PG,S3,RED,KMS_K store class memory,OBS,CUO,AI sink 

### Internal components

Component| Path (planned)| Responsibility  
---|---|---  
`stalwart_adapter.rs`| services/email/src/stalwart_adapter.rs| Spawns and supervises the Stalwart binary; bridges its event stream into the rest of CyberOS.  
`camel.rs`| services/email/src/camel.rs| CaMeL boundary. Calls the quarantined LLM through AI Gateway with `persona=quarantined-extractor` (no tools, no memory). Returns the sanitised structured extraction.  
`dkim_verify.rs`| services/email/src/dkim_verify.rs| Inbound DKIM / SPF / DMARC verification. Wraps mail-auth crate.  
`dkim_sign.rs`| services/email/src/dkim_sign.rs| Outbound DKIM signing with per-tenant key from KMS. ARC stamping on forward.  
`thread.rs`| services/email/src/thread.rs| Thread merge using JMAP-native `threadId`. Reply-To / In-Reply-To / References headers as inputs; never used as authoritative thread keys.  
`inbox.rs`| services/email/src/inbox.rs| Shared-inbox state — assignment, comments, snooze, tags. RLS-keyed by `inbox_id`.  
`search.rs`| services/email/src/search.rs| PGroonga query builder with Vietnamese bigram tokenisation. Falls back to tsvector for non-CJK locales.  
`sieve.rs`| services/email/src/sieve.rs| ManageSieve rules engine — per-Member auto-tag, auto-snooze, auto-route. Rules compiled to Stalwart's sieve runtime.  
`ical.rs`| services/email/src/ical.rs| iCal parser / generator. Calendar invites surface as native events in CyberOS Calendar.  
`draft_ai.rs`| services/email/src/draft_ai.rs| AI-suggested-reply pipeline. Pulls from AI Gateway; persona-stamped; CyberSkill voice; Vietnamese-aware salutation logic.  
`memory_summariser.rs`| services/email/src/memory_summariser.rs| Per-thread summary writer. Calls AI Gateway in summarise mode; writes a single memory row per thread with citations back to message IDs. Body bytes never enter memory by default ((FR pending)).  
`dmarc_report.rs`| services/email/src/dmarc_report.rs| Aggregate DMARC RUA / RUF reports per tenant; surface to Trust Center ((FR pending)).  
`dsar_export.rs`| services/email/src/dsar_export.rs| DSAR bundle: every message a subject participated in, with attachments and decisions.  
`migration_import.rs`| services/email/src/migration_import.rs| Gmail / M365 / IMAP import. Walks the source via JMAP / IMAP, replays threads in CaMeL-filtered order, writes audit row per imported message.  
`migrations/`| services/email/migrations/| sqlx migrations. All identity-touching tables RLS-keyed by `tenant_id`; shared-inbox tables additionally RLS-keyed by `inbox_id`.  
  
4

## Data model

Metadata lives in PostgreSQL with row-level security keyed by `tenant_id`; bodies and attachments live in S3 + KMS, addressed by content-hash. Threads are normalised on JMAP `threadId`, not on Reply-To header heuristics. Shared inboxes are a separate table with their own ACL; a Member can be a participant in many inboxes, and an inbox can be managed by many Members. 

erDiagram TENANT ||--o{ MAILBOX: "owns" TENANT ||--o{ SHARED_INBOX: "owns" MAILBOX ||--o{ THREAD: "contains" SHARED_INBOX ||--o{ THREAD: "routes" THREAD ||--o{ MESSAGE: "contains" MESSAGE ||--o{ ATTACHMENT: "has" MESSAGE ||--o| CAMEL_EXTRACTION: "produced" THREAD ||--o{ ASSIGNMENT: "has" THREAD ||--o{ INTERNAL_COMMENT: "has" THREAD ||--o{ TAG: "has" MAILBOX ||--o{ SIEVE_RULE: "applies" MAILBOX ||--o| DKIM_KEY: "signs with" THREAD ||--o| AI_SUMMARY: "summarised by" MESSAGE ||--o{ DMARC_VERDICT: "verified by" THREAD ||--o{ CALENDAR_EVENT: "links" TENANT { uuid id PK string slug string vn_residency "true | false" } MAILBOX { uuid id PK uuid tenant_id FK uuid owner_subject_id FK string address "person@tenant.com" string display_name string kind "personal | shared" timestamp created_at } SHARED_INBOX { uuid id PK uuid tenant_id FK string address "support@tenant.com" string display_name string default_assignee_subject_id timestamp created_at } THREAD { uuid id PK uuid tenant_id FK uuid mailbox_id FK uuid inbox_id FK "nullable" string subject string jmap_thread_id timestamp last_message_at string status "active | snoozed | done" timestamp snoozed_until int message_count } MESSAGE { uuid id PK uuid thread_id FK string message_id "RFC 5322" string from_addr string to_addrs "json array" string cc_addrs string bcc_addrs string subject timestamp received_at string body_s3_key "ciphertext" bigint body_size_bytes string body_sha256 string spf_result string dkim_result string dmarc_result string direction "inbound | outbound" } ATTACHMENT { uuid id PK uuid message_id FK string filename string mime_type bigint size_bytes string s3_key string sha256 bool scanned "antivirus clean" } CAMEL_EXTRACTION { uuid message_id PK string from_norm string to_norm string subject_norm string gist string action_requests "json" string entities "json" string sensitive_flags "json" string injection_detected_reason "nullable" timestamp produced_at string ai_model "gpt-4o-mini | …" } ASSIGNMENT { uuid id PK uuid thread_id FK uuid assignee_subject_id FK uuid assigner_subject_id FK timestamp assigned_at string status "open | resolved" } INTERNAL_COMMENT { uuid id PK uuid thread_id FK uuid author_subject_id FK string body_markdown timestamp created_at } TAG { uuid thread_id FK string tag uuid added_by FK } SIEVE_RULE { uuid id PK uuid mailbox_id FK string sieve_script int priority bool enabled } DKIM_KEY { uuid id PK uuid tenant_id FK string selector "cyberos-2026-q2" string kms_key_id string public_key_dns timestamp valid_from timestamp valid_to } AI_SUMMARY { uuid thread_id PK string summary_markdown string citations "json: [{message_id, span}]" timestamp generated_at string memory_chain "linked audit row" } DMARC_VERDICT { uuid id PK uuid message_id FK string verdict "pass | fail" string aligned_spf string aligned_dkim timestamp ts } CALENDAR_EVENT { uuid id PK uuid thread_id FK string ical_uid timestamp dtstart timestamp dtend string summary } 

### Indexing & search strategy

  * **Threads** indexed on `(tenant_id, mailbox_id, last_message_at DESC)` and `(inbox_id, status, last_message_at DESC)` for shared-inbox views.
  * **Messages** have a PGroonga index on `subject || coalesce(camel_extraction.gist, '')` — body bytes are never indexed; the CaMeL gist is the queryable surface.
  * **Attachments** are content-addressed by sha256; an antivirus daemon (ClamAV) marks `scanned=true`; unscanned attachments are quarantined and not downloadable.
  * **DKIM keys** rotate every 90 days; old selectors retained 30 days for verification of recently received forwards.
  * **RLS:** every table enforces `WHERE tenant_id = current_setting('cyberos.tenant_id')::uuid`; shared-inbox tables additionally enforce inbox-participation membership.



5

## API surface

Four surfaces: JMAP for IETF-standard mail clients, a GraphQL federated subgraph for the CyberOS SPA, MCP tools for the CUO agent, and a small REST admin surface for DKIM rotation and Trust-Center reports. 

### GraphQL subgraph (federated)
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key", "@requiresScopes"])
    
    type Mailbox @key(fields: "id") {
     id: ID!
     address: String!
     displayName: String!
     kind: MailboxKind!
     unread: Int!
     threads(status: ThreadStatus, limit: Int = 50, cursor: String): ThreadConnection!
     @requiresScopes(scopes: [["email.read"]])
    }
    
    type SharedInbox @key(fields: "id") {
     id: ID!
     address: String!
     displayName: String!
     defaultAssignee: Subject
     unassignedCount: Int!
     threads(status: ThreadStatus, assignee: ID, limit: Int = 50): ThreadConnection!
     @requiresScopes(scopes: [["email.shared.read"]])
    }
    
    type Thread @key(fields: "id") {
     id: ID!
     subject: String!
     participants: [String!]!
     messageCount: Int!
     lastMessageAt: DateTime!
     status: ThreadStatus!
     assignment: Assignment
     tags: [String!]!
     comments: [InternalComment!]!
     summary: AISummary
     messages(limit: Int = 50): [Message!]!
    }
    
    type Message @key(fields: "id") {
     id: ID!
     from: String!
     to: [String!]!
     cc: [String!]
     subject: String!
     receivedAt: DateTime!
     bodyPreview: String! # CaMeL gist · NOT raw body
     attachments: [Attachment!]!
     dmarc: DMARCResult!
    }
    
    type AISummary {
     threadId: ID!
     summary: String!
     citations: [Citation!]!
     generatedAt: DateTime!
    }
    
    enum MailboxKind { PERSONAL SHARED }
    enum ThreadStatus { ACTIVE SNOOZED DONE }
    enum DMARCResult { PASS FAIL NONE }
    
    type Mutation {
     assignThread(threadId: ID!, assigneeId: ID!): Boolean!
     @requiresScopes(scopes: [["email.shared.assign"]])
     snoozeThread(threadId: ID!, until: DateTime!): Boolean!
     resolveThread(threadId: ID!): Boolean!
     addInternalComment(threadId: ID!, body: String!): InternalComment!
     draftReply(threadId: ID!, intent: ReplyIntent): DraftReplyResult!
     @requiresScopes(scopes: [["email.compose"]])
     sendMessage(input: SendMessageInput!): Message!
     @requiresScopes(scopes: [["email.send"]])
    }

### REST + admin surface

Method| Path| Purpose  
---|---|---  
GET| `/.well-known/mta-sts.txt`| MTA-STS policy.  
GET| `/jmap/`| JMAP entry point (RFC 8620).  
POST| `/jmap/api/`| JMAP method calls.  
POST| `/jmap/upload/`| JMAP blob upload.  
GET / POST| `/imap/`| IMAP IDLE / fetch (port 993).  
SMTP| `:25 /:465 /:587`| Submission and inter-MTA relay.  
POST| `/admin/dkim/rotate`| Rotate per-tenant DKIM keypair; old selector retained 30 d.  
GET| `/admin/dmarc/reports`| DMARC aggregate report list.  
POST| `/admin/shared-inbox/create`| Create a shared inbox.  
POST| `/admin/shared-inbox/{id}/participants`| Add / remove participants.  
POST| `/admin/sieve/{mailbox_id}`| Install a ManageSieve script.  
POST| `/admin/import/start`| Start Gmail / M365 / IMAP migration.  
GET| `/admin/import/{job_id}/status`| Migration progress.  
POST| `/admin/dsar/export`| Generate DSAR bundle for a subject.  
  
### MCP tool catalogue

Tool name| Inputs| Outputs| Annotations  
---|---|---|---  
`cyberos.email.list_threads`| mailbox_id?, inbox_id?, status?, limit| Thread| readonly · scope=email.read  
`cyberos.email.read_thread`| thread_id| Thread + Messages (gist only)| readonly  
`cyberos.email.draft_reply`| thread_id, intent| {draft_text, tone, language}| readonly  
`cyberos.email.send`| SendMessageInput| {message_id}| destructive · human-confirm · scope=email.send  
`cyberos.email.assign_thread`| thread_id, assignee_id| {ok}| scope=email.shared.assign  
`cyberos.email.snooze`| thread_id, until| {ok}| scope=email.compose  
`cyberos.email.promote_to_proj`| thread_id, project_id| {issue_id}| scope=proj.write  
`cyberos.email.summarise_thread`| thread_id| {summary, citations}| readonly  
  
6

## Key flows

### Flow 1 — Inbound message with CaMeL scan

sequenceDiagram autonumber participant S as External sender participant ST as Stalwart SMTP:25 participant V as DKIM/SPF/DMARC verifier participant Q as 🛡 CaMeL quarantined LLM participant T as Thread merger participant PG as PostgreSQL participant S3 as S3 + KMS participant B as memory audit participant U as Recipient SPA S->>ST: DATA (RFC 5322 message) ST->>V: verify SPF / DKIM / DMARC V-->>ST: {spf:pass, dkim:pass, dmarc:pass} ST->>S3: write ciphertext body S3-->>ST: s3_key ST->>Q: extract({from, subject, body_text, body_html_stripped}) Note over Q: quarantined LLM has  
NO tools, NO memory,  
NO memory access Q-->>ST: {gist, action_requests, entities, injection_flag} ST->>T: merge by jmap_thread_id T->>PG: insert message + camel_extraction row T->>B: append email.inbound {actor:sender, decision:"received", camel_flag} T-->>U: WebSocket push: new message preview (gist only) Note over U: SPA renders gist + redacted body;  
user can click "show raw" → fetches HTML  
with sandbox iframe; raw never enters CUO context 

If `injection_flag` is set, the message is flagged in the UI with a banner and the AI-draft-reply tool refuses to run on that thread.

### Flow 2 — Outbound message with DKIM signing

sequenceDiagram autonumber participant U as Sender SPA participant API as EMAIL GraphQL participant CO as CaMeL outbound scan participant DK as DKIM signer participant K as AWS KMS participant ST as Stalwart SMTP relay participant MX as Recipient MX participant B as memory U->>API: sendMessage({to, subject, body}) API->>CO: scan for PII / secrets / persona-leakage alt clean CO-->>API: ok else PII / secret detected CO-->>API: 422 {reason:"pii.email.detected"} API-->>U: blocked — review required API->>B: email.outbound_blocked end API->>DK: sign(headers, body) DK->>K: KMS sign with per-tenant key K-->>DK: signature DK-->>API: signed headers API->>ST: relay ST->>MX: SMTP DATA (MTA-STS / DANE next-hop) MX-->>ST: 250 OK ST-->>API: queued API->>B: email.outbound {to, dkim:signed} API-->>U: 200 {message_id} 

### Flow 3 — AI-drafted reply (CyberSkill voice, Vietnamese-aware)

sequenceDiagram autonumber participant U as Member participant API as EMAIL GraphQL participant CE as CaMeL extraction store participant AG as ⚡ AI Gateway participant CU as 🎯 CUO participant B as memory audit U->>API: draftReply(thread_id, intent:"acknowledge + ask follow-up") API->>CE: load sanitised gist + extracted entities CE-->>API: {gist, entities, language: "vi"} Note over API,CU: raw body NEVER enters this path  
only the CaMeL gist + structured fields API->>CU: compose({gist, intent, voice:"cyberskill", language:"vi"}) CU->>AG: chat.completions (persona-stamped JWT, vi salutation rules) AG-->>CU: draft text CU-->>API: draft + tone notes API->>B: email.draft_generated {thread, model, persona_version} API-->>U: {draft, tone:"warm", salutation:"Chào Anh / Chị"} Note over U: Member edits, then sends via Flow 2 

Vietnamese salutation logic: the extracted entities include the recipient's likely gender / title; if confidence < 0.7, the draft defaults to neutral "Chào Anh/Chị".

### Flow 4 — Shared-inbox assignment

sequenceDiagram autonumber participant E as External sender (support@) participant ST as Stalwart SMTP participant SI as SharedInbox router participant CUO as CUO categoriser participant PG as PostgreSQL participant N as Notify (CHAT) participant A as Default assignee E->>ST: support@cyberskill.world ST->>SI: shared-inbox match SI->>CUO: classify({gist, entities}) CUO-->>SI: {category:"billing", confidence:0.92} alt high confidence SI->>PG: assign to billing-team default assignee SI->>N: ping #shared-inbox-billing else low confidence SI->>PG: leave unassigned SI->>N: ping #shared-inbox-triage end A->>SI: claim thread / reassign SI->>PG: assignment row updated SI->>N: post "Claimed by A" 

### Flow 5 — Gmail → CyberOS migration

sequenceDiagram autonumber participant U as Migrating Member participant CLI as cyberos-email participant G as Gmail IMAP participant Q as CaMeL quarantined LLM participant T as Thread merger participant S3 as S3 participant B as memory audit U->>CLI: cyberos-email import gmail --user stephen@… CLI->>G: AUTHENTICATE XOAUTH2 G-->>CLI: SELECT [Gmail]/All Mail (12,481 messages) loop per message CLI->>G: FETCH BODYSTRUCTURE + HEADER + BODY G-->>CLI: raw message CLI->>Q: extract — produce gist, flag injections Q-->>CLI: extraction CLI->>S3: store ciphertext body CLI->>T: merge into thread T->>B: append email.imported {gmail_uid, jmap_thread_id} end CLI-->>U: import complete · 12,481 messages · 47 injection-flagged 

Injection-flagged historical messages are tagged but not deleted; the AI-draft tool refuses to operate on them, and a UI banner explains the flag.

7

## Message lifecycle

Inbound and outbound messages traverse separate state machines, but every state transition emits a memory audit row. 

stateDiagram-v2 [*] --> SMTP_Accepted: 250 OK at edge SMTP_Accepted --> Verifying: SPF/DKIM/DMARC Verifying --> Spam_Quarantined: DMARC reject + heuristics Verifying --> CaMeL_Pending: passed verification CaMeL_Pending --> Extracted: gist + entities produced CaMeL_Pending --> Injection_Flagged: prompt-injection detected Extracted --> Thread_Merged: jmap_thread_id resolved Injection_Flagged --> Thread_Merged: still merged · UI banner shown Thread_Merged --> Active: in inbox / shared inbox Active --> Snoozed: user snooze Snoozed --> Active: snooze expires Active --> Resolved: user marks done Active --> Archived: rule / age policy Active --> Deleted_Tombstoned: user delete (recoverable 30 d) Deleted_Tombstoned --> Purged: 30 d elapsed OR DSAR purge Resolved --> Archived: auto-archive 14 d Spam_Quarantined --> Deleted_Tombstoned: 30 d auto Purged --> [*] Archived --> [*] 

### Outbound state machine

State| Transition| Audit row  
---|---|---  
`composing`| draft saved every 10 s| —  
`outbound_scan`| CaMeL out-bound PII / secret detection| `email.outbound_scan`  
`queued`| signed, in Stalwart queue| `email.outbound_queued`  
`relayed`| MX SMTP 250 received| `email.outbound_relayed`  
`delivered`| DSN MDN received (if requested)| `email.outbound_delivered`  
`bounced`| permanent 5xx from MX| `email.outbound_bounced`  
`deferred`| 4xx — retry per backoff| `email.outbound_deferred`  
  
8

## Functional Requirements

The CyberOS FR catalogue is being rebuilt one feature at a time via the open [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) Agent Skill.

Previous FR enumerations were archived 2026-05-14 and are no longer reflected on this page. Specific FRs land here as they are re-authored.

9

## Non-Functional Requirements

NFRs that EMAIL must satisfy. Cross-referenced at [nfr-catalog.html#email](<../../reference/nfr-catalog.html#email>).

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`N(FR pending)`| Inbound message → thread visible to recipient| p95 ≤ 5 s| k6 SMTP-to-UI test · nightly  
`N(FR pending)`| Outbound message accepted to relay| p95 ≤ 2 s| k6 send-loop · nightly  
`N(FR pending)`| CaMeL extraction latency| p95 ≤ 1.2 s| AI Gateway histogram  
`N(FR pending)`| Thread list page load (50 threads)| p95 ≤ 350 ms| SPA RUM  
`N(FR pending)`| Outbound DMARC pass rate| ≥ 99% of attempted| weekly DMARC aggregate  
`N(FR pending)`| Inbound deliverability (Tier-1 MX accept rate)| ≥ 99.5%| SMTP 250-OK ratio  
`N(FR pending)`| SMTP-receive availability (28-day)| ≥ 99.9%| OBS SLO monitor  
`N(FR pending)`| CaMeL injection-block rate on red-team corpus| ≥ 99.5%| monthly red-team replay  
`N(FR pending)`| Body bytes in memory without opt-in| = 0| CI gate on memory ingestion paths  
`N(FR pending)`| Message durability after acceptance| 0 lost messages under crash| chaos test · memory ledger walk  
`N(FR pending)`| AI-draft acceptance rate| ≥ 40% by Member| SPA telemetry  
`N(FR pending)`| Shared-inbox first-response time| p50 ≤ 30 min business hours| memory assignment events  
`N(FR pending)`| VN-residency tenant message storage region| 100% hanoi-1| S3 key-prefix audit  
  
10

## Dependencies

EMAIL leans on AUTH (every JMAP / GraphQL call), memory (audit + opt-in summary), AI (CaMeL extraction + draft generation), MCP (CUO tools), and OBS (DMARC reports + traces). It is leaned on by CRM (activity auto-log), PROJ ("promote to task"), CUO (assignment categoriser), and KB (publishing a doc as an email digest). 

graph LR subgraph upstream ["EMAIL depends on"] AUTH["🔐 AUTH  
OAuth · RBAC"] memory["🧠 memory  
audit + opt-in summary"] AI["⚡ AI Gateway  
CaMeL + draft"] MCP["🔌 MCP  
CUO tools"] OBS["👁 OBS  
traces + DMARC"] STALWART["📦 Stalwart  
(vendored Rust)"] KMS["🔑 KMS  
DKIM keys"] S3["🗄 S3  
bodies + attachments"] end EMAIL["✉️ EMAIL"] subgraph downstream ["EMAIL is depended on by"] CRM["🤝 CRM  
email→activity"] PROJ["📋 PROJ  
thread→issue"] CUO["🎯 CUO  
shared-inbox classifier"] KB["📚 KB  
email digests"] PORTAL["Portal · P2"] end AUTH --> EMAIL memory --> EMAIL AI --> EMAIL MCP --> EMAIL OBS --> EMAIL STALWART --> EMAIL KMS --> EMAIL S3 --> EMAIL EMAIL --> CRM EMAIL --> PROJ EMAIL --> CUO EMAIL --> KB EMAIL --> PORTAL classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#fef6e0,stroke:#9c750a class memory,STALWART,KMS,S3 shipped class EMAIL,AUTH,AI,MCP,OBS,CRM,PROJ,CUO,KB,PORTAL planned 

11

## Compliance scope

Email is a magnet for regulators. EMAIL must satisfy PDPL, GDPR, RFC compliance, and Decree-13 personal data processing log requirements.

Regulation / standard| Article / clause| EMAIL feature that satisfies it  
---|---|---  
Vietnam PDPL (Law 91/2025)| Art. 14 — DSAR| `cyberos-email dsar-export` bundles every message a subject sent / received.  
Vietnam Decree 13/2023| Art. 17 — Personal data processing log| Every inbound / outbound writes a memory audit row.  
Vietnam Decree 53/2022| Art. 26 — Data localisation| Per-tenant `vn_residency` flag routes bodies + attachments to hanoi-1 S3.  
GDPR (EU 2016/679)| Art. 15 — Right of access| DSAR export.  
GDPR| Art. 17 — Right to erasure| Soft-delete + 30-day purge; DSAR-driven hard purge.  
GDPR| Art. 32 — Security of processing| TLS 1.3 SMTP, KMS-wrapped bodies, RLS, DKIM signing.  
RFC 5322| Internet Message Format| Stalwart parser is RFC-compliant; canonical headers preserved.  
RFC 6376| DKIM Signatures| `dkim_sign.rs` \+ per-tenant KMS key.  
RFC 7208| SPF| Inbound verifier; outbound DNS check.  
RFC 7489| DMARC| Inbound enforcement; aggregate report ingestion.  
RFC 8460| SMTP TLS reporting| Stalwart TLSRPT support; ingested into OBS.  
RFC 8461| MTA-STS| Outbound policy fetch; inbound policy publication.  
RFC 8617| ARC| ARC stamping on forward.  
RFC 8620| JMAP| Native JMAP server via Stalwart.  
BIMI v1| Brand Indicators| VMC + SVG-Tiny logo serving on opt-in tenants.  
OWASP Gen AI Top-10 (2025)| LLM01: Prompt Injection| CaMeL dual-LLM is the canonical mitigation.  
  
12

## Risk entries

EMAIL carries the second-highest single-module risk weight after AUTH; injection attacks here propagate to every downstream module. Tracked in the [risk register](<../../reference/risk-register.html#email>).

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-EMAIL-001`| CaMeL bypass via novel injection technique| Medium| Catastrophic| CSO| Red-team corpus refreshed monthly; quarantined LLM has zero tools; CUO never sees raw body even on bypass.  
`R-EMAIL-002`| Stalwart CVE landing zero-day| Medium| High| CTO| Subscribe to security advisories; auto-PR + canary test on patch release; rollback plan documented.  
`R-EMAIL-003`| DKIM key compromise → spoofing tenant| Low| High| CSO| KMS-wrapped; rotation every 90 d; old selector retained 30 d.  
`R-EMAIL-004`| Outbound spam reputation collapse → IP block| Medium| High| CTO| Per-tenant send rate limits; warm-up plan; SubMTA / dedicated IP at P2+; Postmaster Tools monitoring.  
`R-EMAIL-005`| Body-content leak into memory via misconfigured rule| Low| High| CDO| Static-analysis CI gate on every memory-ingestion code path; canonical ingestion is summary-only.  
`R-EMAIL-006`| VN-residency tenant body lands in non-VN S3| Low| High| CCO| S3 key-prefix audit nightly; failure pages CSO.  
`R-EMAIL-007`| Attachment with malware reaches user (AV bypass)| Medium| Medium| CSO| ClamAV + heuristic scan; "show raw" view in sandbox iframe; user banner for unscanned attachments.  
`R-EMAIL-008`| JMAP / IMAP credential leak via SPA bug| Low| High| CTO| OAuth 2.1 access tokens only; no IMAP password reuse; refresh-token rotation.  
`R-EMAIL-009`| Migration import overflows quarantine review queue| Medium| Low| CTO| Rate-limited import (1k msg / min); injection-flagged messages tagged but not blocking the import.  
`R-EMAIL-010`| AGPL-3.0 obligations on Stalwart redistribution| Medium| Medium| CLO| Service-only distribution; source-code-offer URL on Trust Center; legal review of every Stalwart fork.  
`R-EMAIL-011`| **Thread-to-issue creates wrong Engagement on conversion**|  Medium| Medium| COO| Conversion shows AM the auto-suggested Engagement (matched by domain); AM must confirm or override; per-tenant default for unmatched.  
`R-EMAIL-012`| Genie draft includes confidential data from memory that customer shouldn't see| Medium| High| DPO| Genie draft skill operates in "outbound" mode — sync_class filter applied at retrieval; can only ground in shareable/client-visible memories; AM final review.  
`R-EMAIL-013`| Bulk-send approval bypassed via API call| Low| High| CSO| Approval token required at SMTP submission gate; CI test verifies bulk-send rejection without token; OBS alarm on direct-API bulk attempts.  
`R-EMAIL-014`| Tracked-domain misconfig — personal email auto-logged to CRM| Low| High| CSO| Tracked-domain config audited monthly; CI asserts common personal domains (gmail/yahoo/icloud) excluded from default tracking; per-Member opt-out.  
`R-EMAIL-015`| CaMeL quarantine LLM cost spike from spam wave| Medium| Low| CTO| Pre-filter blocks obvious spam before quarantine; AI Gateway cost cap enforced; alarm on EMAIL cost > 30% above 7-day baseline.  
  
13

## KPIs

EMAIL health rolls up into 10 KPIs covering deliverability, injection-defence efficacy, AI-feature quality, and shared-inbox UX.

KPI| Formula| Source| Target  
---|---|---|---  
**Outbound DMARC pass rate**| `dmarc.pass / dmarc.total`| weekly aggregate| ≥ 99%  
**Inbound deliverability**| `250_ok / smtp_attempts`| Stalwart logs| ≥ 99.5%  
**CaMeL injection-block rate**| `blocked / red_team_corpus`| monthly red-team| ≥ 99.5%  
**CaMeL extraction p95**|  histogram| OBS| ≤ 1.2 s  
**AI-draft acceptance rate**| `drafts_sent / drafts_offered`| SPA telemetry| ≥ 40%  
**Shared-inbox first-response p50**|  median minutes| memory assignment events| ≤ 30 min (BH)  
**Snooze usage**|  snoozes / Member / week| SPA telemetry| tracked  
**Spam false-positive rate**| `fp / total_classified_spam`| user "not spam" actions| ≤ 0.5%  
**Outbound CaMeL block rate**|  blocked / sent attempts| memory| tracked; alert on > 1% / day  
**Attachment AV-quarantine rate**|  quarantined / total| memory| tracked; alert spike  
**Thread-to-issue conversion accuracy**|  conversions retained / total conversions (90d)| PROJ events| ≥ 0.90 (low rollback = good auto-match)  
**Genie draft confidential-leak rate**|  drafts containing private/personal memories / total drafts| CaMeL post-write audit| = 0 (hard floor)  
**Bulk-send token compliance**|  bulk sends with approval token / total bulk sends| SMTP submission audit| = 1.0 (hard floor; CI gate)  
**Tracked-domain audit pass rate**|  tenants passing monthly tracked-domain audit / total| monthly CSO review| = 1.0  
**CaMeL cost per inbound msg**|  AI Gateway cost for CaMeL / total inbound| cost ledger| tracked; alarm on baseline-drift > 30%  
  
14

## RACI matrix

EMAIL is owned by the CCO seat. Today (CCO vacant), the CEO is interim accountable; the CTO owns engineering; the CSO is consulted on every CaMeL-touching change.

Activity| CEO| CTO| CSO| CCO| CDO| CLO  
---|---|---|---|---|---|---  
Service design + spec| A| R| C| C| I| C  
Implementation| I| A/R| C| I| I| I  
CaMeL boundary review| I| C| A/R| I| C| I  
Stalwart upgrade| I| A/R| R| I| I| C  
DKIM key rotation| I| R| A| I| I| I  
DMARC policy authoring| C| R| A| I| I| C  
Shared-inbox product| C| C| I| A/R| I| I  
DSAR fulfilment| I| C| C| I| R| A  
Trust Center publication| C| C| C| R| I| A  
AGPL compliance review| I| C| I| I| I| A/R  
  
**R** Responsible · **A** Accountable · **C** Consulted · **I** Informed.

15

## Planned CLI surface

A single admin CLI `cyberos-email` for tenant operators. Destructive commands always write a chained audit row before exit.

### 1\. Create a shared inbox
    
    
    $ cyberos-email shared-inbox create \
     --tenant cyberskill \
     --address support@cyberskill.world \
     --display "CyberSkill Support" \
     --default-assignee linh@cyberskill.world
    
    [shared-inbox created]
     id: 01HZK1Y2J3K4M5N6P7Q8R9S0T1
     address: support@cyberskill.world
     audit: memory seq=14902 chain=1a2b…3c4d

### 2\. Rotate per-tenant DKIM key
    
    
    $ cyberos-email dkim rotate --tenant cyberskill --reason "quarterly-scheduled"
    
    [rotate] new selector: cyberos-2026-q2
    [kms] old selector cyberos-2026-q1 retained for verification (30 d)
    [dns] _domainkey.cyberskill.world TXT record updated
    [audit] memory seq=14905 chain=5d6e…7f80

### 3\. Import from Gmail
    
    
    $ cyberos-email import gmail \
     --user stephen@cyberskill.world \
     --since 2024-01-01 \
     --rate-limit 1000/min
    
    [import] authenticating via XOAUTH2 …
    [import] selecting [Gmail]/All Mail … 12,481 messages
    [import] walking threads (CaMeL extracting on the fly) …
    [import] complete · 12,481 messages · 47 injection-flagged · 0 errors
    [audit] memory seq=14952 chain=9a8b…7c6d

### 4\. View DMARC aggregate report
    
    
    $ cyberos-email dmarc report --tenant cyberskill --since 7d
    
    domain policy sent pass fail pct
    cyberskill.world reject 8,213 8,201 12 99.85%
    ↳ google.com - 4,102 4,098 4 99.90%
    ↳ outlook.com - 2,108 2,103 5 99.76%
    ↳ proton.me - 1,003 1,000 3 99.70%
    ↳ other - 1,000 1,000 0 100.00%

### 5\. Run CaMeL red-team replay
    
    
    $ cyberos-email redteam replay --corpus echoleak-v3 --output report.json
    
    [replay] corpus: echoleak-v3 (412 messages)
    [replay] running quarantined extraction …
    [replay] ✓ 410 / 412 injection attempts blocked (99.51%)
    [replay] ✗ 2 escapes — see report.json#escapes
    [audit] memory seq=14958 chain=ef01…2345

### 6\. Export DSAR bundle
    
    
    $ cyberos-email dsar-export --subject stephen@cyberskill.world --output stephen.zip
    
    [dsar] subject: stephen@cyberskill.world
    [dsar] threads: 1,247
    [dsar] messages: 8,213 (3,142 inbound · 5,071 outbound)
    [dsar] attachments: 412 (412 MB)
    [dsar] summaries: 1,247 (CaMeL gist)
    [dsar] written: stephen.zip (478 MB)
    [audit] memory seq=14961 chain=6a7b…8c9d

### 7\. Install ManageSieve rule
    
    
    $ cyberos-email sieve install --mailbox stephen@cyberskill.world --file my.sieve
    
    require ["fileinto", "imap4flags"];
    if address:is "from" "noreply@github.com" {
     fileinto "GitHub";
     setflag "\\Seen";
     stop;
    }
    
    [sieve] parsed · 1 rule
    [sieve] installed at priority 100
    [audit] memory seq=14963 chain=2b3c…4d5e

16

## Phase status & estimates

Status

Planned

P1 design phase

Est. LoC

~9,000

Rust adapter + TS SPA + sqlx

Planned tests

90+

incl. CaMeL red-team replay

External libs

~15

Stalwart · mail-auth · PGroonga

CLI subcommands

~20 planned

`cyberos-email`

P1 budget

~$110/mo

Fargate + RDS + S3 + Redis

Capability| Status  
---|---  
Stalwart core (SMTP / JMAP / IMAP)| planned · P1  
CaMeL quarantined extraction| planned · P1  
Per-tenant DKIM signing| planned · P1  
Shared inbox + assignment| planned · P1  
Internal-comments on threads| planned · P1  
AI-drafted reply (VN salutations)| planned · P1  
PGroonga Vietnamese search| planned · P1  
ManageSieve rule engine| planned · P1  
Gmail / M365 / IMAP import| planned · P1  
iCal calendar invite parsing| planned · P1  
DMARC aggregate reporting| planned · P1  
DSAR export bundle| planned · P1  
BIMI logo serving (VMC)| planned · P2+  
Dedicated SubMTA / reputation IP| planned · P2+  
Outbound encryption (S/MIME / PGP)| planned · P3+  
Multi-region active-active| planned · P3+  
  
17

## References

  * **Module strategy** — EMAIL strategy: Stalwart + Missive-style UX + CaMeL.
  * **NFR inheritance** — Security NFRs that EMAIL must satisfy.
  * **FR mapping** — Formal (FR pending) through (FR pending) with verification methods.
  * **Stalwart Mail Server** — `github.com/stalwartlabs/mail-server` (AGPL-3.0).
  * **CaMeL paper** — Google DeepMind, May 2025; "Defending LLM agents against prompt injection via privilege separation".
  * **EchoLeak (CVE-2025-32711)** — May 2025 advisory on M365 Copilot prompt-injection exfiltration.
  * **RFC 5322** — Internet Message Format.
  * **RFC 6376** — DomainKeys Identified Mail (DKIM) Signatures.
  * **RFC 7208** — Sender Policy Framework (SPF).
  * **RFC 7489** — Domain-based Message Authentication, Reporting, and Conformance (DMARC).
  * **RFC 8460** — SMTP TLS Reporting.
  * **RFC 8461** — SMTP MTA Strict Transport Security (MTA-STS).
  * **RFC 8617** — Authenticated Received Chain (ARC).
  * **RFC 8620** — JSON Meta Application Protocol (JMAP).
  * **Decree 53/2022/NĐ-CP (Vietnam)** — Cybersecurity Law; data residency.
  * **Bigger picture (§0 above):** 3 strategic roles + spine Mermaid + 10-row auto-vs-human matrix.
  * **Cross-module page links:** [crm.html](<../crm/index.html>) · [proj.html](<../proj/index.html>) · [cuo.html](<../cuo/index.html>) · [memory.html](<../memory/index.html>) · [kb.html](<../kb/index.html>) · [ai.html](<../ai/index.html>) · [obs.html](<../obs/index.html>)
  * **memory auto-sync vision:** [MEMORY_AUTOSYNC_DESIGN.md §5](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) — sanitised CaMeL extractions feed memory; raw bodies never.
  * **Build-readiness audit:** `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`) — EMAIL at P1 · mid (P1).
  * **FR authoring discipline:** [modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>).
  * **CaMeL paper (Google DeepMind, May 2025)** — dual-LLM pattern for defeating indirect prompt injection.
  * **EchoLeak (CVE-2025-32711, May 2025)** — the canonical exfiltration case that motivates CaMeL.
  * **Decree 13/2023/NĐ-CP (Vietnam)** — Personal data processing protection.
  * **Law 91/2025/QH15 (Vietnam PDPL)** — Personal Data Protection Law.
  * **BIMI v1** — Brand Indicators for Message Identification.
  * **Architecture context:** [infrastructure.html#email](<../../architecture/infrastructure.html#email>).



[← All modules](<../index.html#catalog>) [Next module: PROJ →](<../proj/index.html>)
