---
title: CRM — Sales-pipeline spine · Deal-to-engagement bridge · Vietnamese-market-first · CyberOS
source: website/docs/modules/crm/index.html
migrated: FR-DOCS-002
---

CRM is CyberOS's **customer pipeline, account intelligence, and Vietnamese-market commercial layer**. The data model is Salesforce-flavoured (Account → Contact → Deal) but the integrations bake in Vietnamese specifics: `vietnam-mst-validate` verifies a tax code against the General Department of Taxation registry; `vietnam-bank-transfer` generates a Napas247 / VietQR code so a deal can request collection with one tap; `vietnam-vat-invoice` emits the hóa đơn directly on deal close. Pipelines are workspace-configurable (sales / partner / inbound / outbound shapes) with custom transition rules. Activities — calls, emails, meetings, notes — appear automatically: EMAIL auto-logs inbound/outbound for tracked domains, CHAT logs meeting notes, Calendar logs meeting attendance. CUO/CRO-skill produces lead scoring, next-best-action, and confidence-banded forecasts.. 

Strategic role

Sales-pipeline spine

upstream of PROJ Engagement

Status

Planned

P1 · design phase · P1 · start

Primitives

3

Account · Contact · Deal

Pipeline shapes

4 default

sales · partner · inbound · outbound

Deal → Engagement bridge

One-click

PROJ §2.5 join contract

VN skills

3 native

MST · VietQR · hóa đơn

Activity feeds

CHAT · EMAIL · Cal

auto-logged via OBS trace_id

AI features

4

score · NBA · forecast · win-loss

Vertical-pack ready

vn → sg → id

per-jurisdiction skill bundles

Depends on

AUTH · memory · CUO

\+ AI · MCP · EMAIL · Skills

Est. LoC

~8,500

Rust + TS pipeline UI

0

## The bigger picture — three strategic roles

CRM is where revenue starts. It is also where the orchestration spine begins: a Deal moves through stages, lands on "won", and the join contract fires — a PROJ Engagement is created with the rate card pre-wired. The naive read is "another Salesforce clone". The real read: CyberOS-native CRM is the upstream half of the same spine PROJ is the downstream half of, and the Vietnamese-specific skills (MST, VietQR, hóa đơn) are first-class — not plugin afterthoughts. 

Role 1 · Sales pipeline (VN-first)

Account · Contact · Deal with VN integrations native

Three primitives + customisable pipelines. Vietnamese specifics are protocol features, not plugins: account type maps to VN legal entities (sole proprietor / LLC / JSC / FDI), MST validated through `cyberos.vietnam-mst-validate` skill, VietQR for collection, hóa đơn on deal close. Salutation logic (Anh/Chị/Em) baked into contact fields. 

Role 2 · Deal → Engagement bridge

The orchestration spine starts here

When AM clicks "Convert to Engagement" on a won deal, the PROJ §2.5 join contract fires: rate card pre-populated from deal pricing, billable rules wired, the Engagement gets the deal's contract URL + client account_id. TIME entries flow back. INV invoices from Engagement. The full PROJ-TIME-INV-REW chain begins at this click. 

Role 3 · Next-action engine

CUO ranks moves on every open deal

The CUO `crm.next-action@1` skill consults memory context (prior calls with this account, similar deal trajectories, win-loss memories) and ranks next moves — call, send proposal, schedule demo, request reference. AI lead scoring at signup; win/loss analysis after close becomes a memory memory for future deals to cite. The deal isn't just a record; it's a node in the decision graph. 

### CRM in the orchestration spine

flowchart LR LEADS["Inbound leads  
EMAIL · web form · referral"] CRM["🤝 CRM  
Account · Contact · Deal · Activity"] CUO["🎯 CUO  
crm.next-action@1  
crm.score-lead@1"] PROJ["📋 PROJ Engagement"] INV["🧾 INV invoice"] memory["🧠 memory  
deal history · win/loss memories"] EMAIL["✉ EMAIL"] LEADS --> CRM EMAIL --> CRM CRM --> CUO CUO --> CRM CRM -- "deal.won → engagement.create" --> PROJ PROJ --> INV CRM --> memory memory --> CUO classDef hub fill:#ffe4e6,stroke:#9f1239,stroke-width:3px,color:#881337 classDef mod fill:#e0e7ff,stroke:#3730a3 classDef memory fill:#fef6e0,stroke:#9c750a class CRM hub class CUO,PROJ,INV,EMAIL,LEADS mod class memory memory 

### Auto vs human-in-loop operations matrix

Operation| How it happens| Why this split  
---|---|---  
Activity auto-logging| **Auto** from EMAIL/CHAT/Cal by domain/contact match| Stops the "log it in the CRM" tax that AMs hate; auto-log = no manual entry.  
Lead scoring| **Auto** on Contact create; refreshed nightly| Score is a hint, not an action — AM decides outreach priority.  
Stage transition| **Manual** with workflow validator| Sales is a relationship, not an automaton; AM owns when a deal progresses.  
MST validation| **Auto** on every account write that includes an MST| VN tax-code authority is the GDT registry; we just call them.  
VietQR generation| **Auto-available** , **manual-trigger** per deal| One click on the deal page; cached for 24 h.  
Hóa đơn (VAT invoice) emission| **Auto** on deal.stage=won via INV; **manual confirm** required| Tax document — AM must confirm amount + recipient before emission.  
Deal → Engagement conversion| **Manual** AM click| Not every won deal becomes a PROJ Engagement (some are pure resale); AM intent required.  
Win/loss analysis| **Auto** draft via CUO; **AM accept**|  Generates a memory memory citable by future deals; AM owns the narrative.  
Inbound web-form deal create| **Auto** with lead-scoring pre-applied| Reduces friction; AM triages from a pre-scored queue.  
  
1

## Why CRM exists

Off-the-shelf CRMs (Salesforce, HubSpot, Pipedrive) work, but they do not natively model Vietnamese commercial realities. MST validation requires a custom integration; VietQR generation needs a separate plugin; hóa đơn generation almost always lands as a third-party Misa or Viettel-eInvoice connector that breaks on every release. Building CRM as a CyberOS module means these become first-class: the `vietnam-mst-validate` skill is invoked on every Account write, the `vietnam-bank-transfer` skill is one click away on every Deal, and `vietnam-vat-invoice` fires automatically on deal close into INV. Beyond the VN specifics, CyberOS-native CRM benefits from the audit chain (every stage transition is a memory row), the agent-equal model (CUO can log activities just like a Member), and tight coupling to PROJ (a closed deal becomes an Engagement, no manual re-keying). 

🇻🇳

Vietnamese-first

MST, VietQR, hóa đơn, Anh/Chị salutation, address-parsing helpers — built in, not bolted on.

🪢

Activity auto-log

Inbound / outbound emails to tracked domains, calendar attendance, CHAT meeting notes — all appear in the activity timeline without manual entry.

🔮

AI forecasting + NBA

CUO/CRO-skill produces confidence-banded forecasts and concrete next-best-action prompts for each open Deal.

The bet is that the Vietnamese consultancy buyer values "this works with our hóa đơn flow" more than "this has 700 SaaS integrations". Coupling CRM tightly to CyberOS skills means the VN-native flows are native, not a bolted-on plugin layer. The cost is that CyberOS owns the surface forever; the benefit is that a deal close is a one-tap event from VietQR to hóa đơn to Engagement. 

2

## What it does — 5W1H2C5M

A structured decomposition of CRM's scope. Every cell traces back to and §19.6.

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is CRM?| An Account / Contact / Deal pipeline tracker with customisable per-pipeline stages, automatic activity feed from CHAT / EMAIL / Calendar, memory-ingested closed deals, and Vietnamese-market skill integrations (MST, VietQR, hóa đơn).  
**5W · Who**|  Who uses it?| **Sales:** create and progress deals; daily pipeline review. **Account Managers:** own client relationships; log activities. **CRO:** dashboard + forecast review. **Marketing:** read-only on activity + lead source. **Agents:** CUO/CRO-skill logs activities, suggests NBAs, produces forecasts.  
**5W · When**|  When does it run?| Continuous: SPA pipeline view; WebSocket on stage transitions. Hourly: activity feed reconciliation (EMAIL → CRM). Nightly: AI lead scoring refresh + win/loss analysis on closed deals. On deal close: VietQR + hóa đơn + PROJ Engagement creation.  
**5W · Where**|  Where does it run?| P1: single region (SG-1) with VN-residency RDS for VN tenants. P3+: multi-region. The CRM service is one axum binary; the pipeline UI is a React SPA component.  
**5W · Why**|  Why a separate module?| Activity volume, ACL granularity (per-deal), and Vietnamese-skill integration are too specific to belong elsewhere. Folding into PROJ corrupts both data models.  
**1H · How**|  How does it work?| Accounts are validated on write (MST → GDT registry). Contacts have a normalised phone (E.164) and email; merge candidates surface when two contacts share either. Deals carry stage, value, probability, expected close. Activities are append-only and feed from EMAIL / CHAT / Calendar with idempotency keys.  
**2C · Cost**|  Cost budget?| P1: ~$60 / month single-tenant pilot (Fargate + RDS + Redis). 50-tenant: ~$240 / month. AI lead scoring runs nightly batch; ~$0.001 per deal per night.  
**2C · Constraints**|  Constraints?| (a) Per-deal ACL — private deals not visible cross-workspace ((FR pending)). (b) Email-to-CRM auto-log only for tracked domains ((FR pending)) — never auto-attach a personal email. (c) Vietnamese-formatting helpers (Anh / Chị / Bạn) on every contact display. (d) Deals only ingested into memory after closed / lost — never open pipeline data.  
**5M · Materials**|  Stack?| Rust 1.81 · axum · sqlx · PostgreSQL 16 · Redis 7 · React + Zustand · vietnam-mst-validate skill · vietnam-bank-transfer skill · vietnam-vat-invoice skill · libphonenumber · OpenTelemetry SDK.  
**5M · Methods**|  Method choices?| Three-primitive Account / Contact / Deal (no Lead as separate primitive — Leads are Contacts with no Deal). Configurable pipeline stages (not free-form). Phone normalisation to E.164 (libphonenumber). Per-deal ACL. Idempotent activity feed (idempotency key = source event id).  
**5M · Machines**|  Deployment?| Fargate axum service. RDS Postgres Multi-AZ. Redis hot cache. NATS for pipeline-event fan-out. Skill invocations via the Skill module's WASM host.  
**5M · Manpower**|  Who maintains?| 0.5 FTE (CRO seat) at P1 launch. By P2+: CRO owns product; CMO consulted on activity feed schema; CFO consulted on hóa đơn flow.  
**5M · Measurement**|  How measured?| Pipeline drag-drop response p95 ≤ 200 ms; activity-feed reconciliation p95 ≤ 60 s after EMAIL receipt; lead-scoring precision ≥ 70% on validated wins; deal-close → INV invoice generated within 60 s (P1: 5 min).  
  
3

## Architecture

CRM is one axum service with four surfaces (GraphQL subgraph, REST admin, MCP, WebSocket for pipeline drag-drop), three stores (Postgres canonical, Redis hot cache, NATS pipeline-event fan-out), and three Vietnamese-market skill invocations (MST validate, VietQR, hóa đơn). 

graph TB subgraph CLIENT ["Clients"] SPA["SPA pipeline · contact view"] AGENT["🎯 CUO via MCP"] end subgraph EDGE ["Edge"] GQL["GraphQL subgraph"] REST["REST admin"] WS["WebSocket  
pipeline drag-drop"] MCP["MCP tools"] end subgraph CORE ["CRM service (Rust)"] ACC["Account handler"] CON["Contact handler"] DEAL["Deal handler"] PIPE["Pipeline engine  
(stages + rules)"] ACT["Activity feed  
auto-log"] AI_SCORE["AI lead scoring"] AI_NBA["Next-best-action"] AI_FORECAST["Forecast"] MERGE["Merge candidate detector"] MEMORY_ING["memory ingestor  
(closed deals)"] end subgraph SKILLS ["VN-market skills"] MST["vietnam-mst-validate"] VQR["vietnam-bank-transfer  
(VietQR)"] HD["vietnam-vat-invoice  
(hóa đơn)"] end subgraph STORES ["Stores"] PG[("PostgreSQL  
RLS by tenant_id  
per-deal ACL")] RED[("Redis 7  
WS rooms · search cache")] NATS[("NATS JetStream  
pipeline events")] end subgraph SOURCES ["Activity sources"] EMAIL["✉️ EMAIL  
tracked domains"] CHAT["💬 CHAT  
meeting notes"] CAL["📅 Calendar  
attendance"] end subgraph SINKS ["Sinks"] memory["🧠 memory  
closed deals only"] PROJ["📋 PROJ  
deal close → Engagement"] INV["🧾 INV  
hóa đơn invoice"] OBS["👁 OBS"] end SPA --> WS SPA --> GQL AGENT --> MCP GQL --> ACC GQL --> CON GQL --> DEAL REST --> ACC REST --> CON REST --> DEAL MCP --> ACC MCP --> CON MCP --> DEAL WS --> PIPE ACC --> MST DEAL --> VQR DEAL --> HD HD --> INV EMAIL --> ACT CHAT --> ACT CAL --> ACT ACT --> DEAL ACT --> CON DEAL --> PIPE PIPE --> NATS NATS --> WS CON --> MERGE DEAL --> AI_SCORE DEAL --> AI_NBA DEAL --> AI_FORECAST DEAL --> MEMORY_ING MEMORY_ING --> memory DEAL --> PROJ ACC --> PG CON --> PG DEAL --> PG ACT --> PG PIPE --> RED DEAL --> OBS classDef planned fill:#ffe4e6,stroke:#9f1239 classDef skill fill:#fef6e0,stroke:#9c750a classDef store fill:#f5f3ff,stroke:#7c3aed classDef sink fill:#f5ede6,stroke:#45210e class SPA,AGENT,GQL,REST,WS,MCP,ACC,CON,DEAL,PIPE,ACT,AI_SCORE,AI_NBA,AI_FORECAST,MERGE,MEMORY_ING planned class MST,VQR,HD skill class PG,RED,NATS store class memory,PROJ,INV,OBS,EMAIL,CHAT,CAL sink 

### Internal components

Component| Path (planned)| Responsibility  
---|---|---  
`account.rs`| services/crm/src/account.rs| Account CRUD. MST validation on write (Vietnamese tenants). Address parser. Custom fields per workspace.  
`contact.rs`| services/crm/src/contact.rs| Contact CRUD. E.164 phone normalisation. Anh / Chị / Bạn salutation derivation. Merge candidate detection.  
`deal.rs`| services/crm/src/deal.rs| Deal CRUD. Stage transitions with workflow rules. Value / probability / expected-close. Per-deal ACL.  
`pipeline.rs`| services/crm/src/pipeline.rs| Pipeline definition + stage transition rules. Workspace-configurable. WebSocket fan-out on drag-drop.  
`activity_feed.rs`| services/crm/src/activity_feed.rs| Auto-log from EMAIL / CHAT / Calendar. Idempotency by source event id.  
`email_logger.rs`| services/crm/src/email_logger.rs| Listens to EMAIL events; auto-logs inbound / outbound for tracked-domain mailboxes ((FR pending)).  
`ai_lead_scoring.rs`| services/crm/src/ai_lead_scoring.rs| Nightly batch lead scoring. Features: activity recency, engagement size, account vertical, historical win-rate.  
`ai_nba.rs`| services/crm/src/ai_nba.rs| Next-best-action generator. For each open deal, produces a concrete suggestion ("send proposal", "schedule check-in", "request VietQR collection").  
`ai_forecast.rs`| services/crm/src/ai_forecast.rs| Confidence-banded forecast (CUO/CRO-skill). Per-pipeline VND total + USD total with 50% / 80% / 95% bands.  
`merge_detector.rs`| services/crm/src/merge_detector.rs| Detects ≥ 2 contacts sharing email or normalised phone; surfaces as a Notify ((FR pending)).  
`vn_mst.rs`| services/crm/src/vn_mst.rs| Adapter to `vietnam-mst-validate` skill. Caches results 7 days.  
`vn_vietqr.rs`| services/crm/src/vn_vietqr.rs| Adapter to `vietnam-bank-transfer` skill. Generates Napas247 QR on demand.  
`vn_hoadon.rs`| services/crm/src/vn_hoadon.rs| Adapter to `vietnam-vat-invoice` skill. Fires on deal close.  
`memory_ingest.rs`| services/crm/src/memory_ingest.rs| Layer 3 ingestion of closed deals only ((FR pending)).  
`acl.rs`| services/crm/src/acl.rs| Per-deal ACL. Private deals not visible cross-workspace.  
`vn_salutation.rs`| services/crm/src/vn_salutation.rs| Vietnamese salutation helper. Anh / Chị / Bạn picker from gender + age signal.  
`migrations/`| services/crm/migrations/| sqlx migrations. RLS on every table.  
  
4

## Data model

Three primitives (Account, Contact, Deal) plus support tables for activities, pipelines, stages, merges, and AI outputs. Accounts carry Vietnamese-specific fields (account_type, MST). Contacts normalise phone to E.164 and persist a salutation hint. Deals carry per-deal ACL and stage transitions are audited. 

erDiagram TENANT ||--o{ ACCOUNT: "owns" ACCOUNT ||--o{ CONTACT: "has" ACCOUNT ||--o{ DEAL: "has" CONTACT ||--o{ DEAL: "primary contact" DEAL ||--o{ ACTIVITY: "log" CONTACT ||--o{ ACTIVITY: "log" ACCOUNT ||--o{ ACTIVITY: "log" DEAL ||--o| PIPELINE: "lives in" PIPELINE ||--o{ STAGE: "has" DEAL ||--o| STAGE: "currently at" DEAL ||--o{ STAGE_TRANSITION: "history" DEAL ||--o| LEAD_SCORE: "scored" DEAL ||--o| NBA_SUGGESTION: "advised" DEAL ||--o| VIETQR: "collection" DEAL ||--o| HOA_DON: "invoice" CONTACT ||--o{ CONTACT_MERGE_CANDIDATE: "candidate" DEAL ||--o| DEAL_ACL: "private" TENANT { uuid id PK string slug } ACCOUNT { uuid id PK uuid tenant_id FK string name string account_type "sole_proprietor | llc | jsc | fdi | individual | other" string vn_mst "10 or 13 digits" bool vn_mst_validated string industry string country string address_street string address_ward string address_district string address_province string lead_source timestamp created_at uuid owner_member_id FK string custom_fields "json" } CONTACT { uuid id PK uuid tenant_id FK uuid account_id FK string first_name string last_name string display_name string email string phone_e164 string phone_raw string salutation "Anh | Chị | Bạn | Em" string title bool is_primary string custom_fields "json" } DEAL { uuid id PK uuid tenant_id FK uuid account_id FK uuid primary_contact_id FK string name string description_short int amount_minor_vnd int amount_minor_usd "nullable" string currency "VND | USD" string pipeline_id FK uuid stage_id FK int probability_pct date expected_close_date string status "open | won | lost | cancelled" string lost_reason "nullable" date closed_date uuid owner_member_id FK string acl_mode "default | restricted" timestamp created_at } PIPELINE { uuid id PK uuid tenant_id FK string name "sales | partner | inbound | outbound" bool default } STAGE { uuid id PK uuid pipeline_id FK string code string display_name_vi string display_name_en int order_idx int default_probability_pct string category "open | won | lost" } STAGE_TRANSITION { uuid id PK uuid deal_id FK uuid from_stage_id FK uuid to_stage_id FK uuid actor_id FK timestamp ts string memory_chain } ACTIVITY { uuid id PK uuid tenant_id FK uuid deal_id FK "nullable" uuid contact_id FK "nullable" uuid account_id FK "nullable" string kind "email | call | meeting | note | task | site_visit" string subject string body_markdown timestamp occurred_at string source "manual | email | chat | calendar | mcp" string source_event_id "idempotency key" uuid created_by FK } LEAD_SCORE { uuid deal_id PK int score_0_100 string tier "hot | warm | cold" string reasons "json array" timestamp computed_at } NBA_SUGGESTION { uuid deal_id PK string action_code "send_proposal | schedule_checkin | request_payment | …" string explanation_markdown int confidence_pct timestamp computed_at } VIETQR { uuid id PK uuid deal_id FK string bank_bin "970418 | …" string account_number int amount_minor_vnd string addinfo string qr_payload "EMVCo string" timestamp generated_at } HOA_DON { uuid id PK uuid deal_id FK string invoice_number string template_code int total_minor_vnd int vat_minor_vnd string xml_url "GDT-compliant XML" timestamp issued_at } CONTACT_MERGE_CANDIDATE { uuid id PK uuid contact_a FK uuid contact_b FK string match_reason "email | phone | both" float confidence string status "pending | merged | dismissed" } DEAL_ACL { uuid deal_id PK string allow_role_codes "csv" string allow_member_ids "csv" } 

### Vietnamese account types

Code| Vietnamese| English| MST length| Hóa đơn template  
---|---|---|---|---  
`sole_proprietor`| Hộ kinh doanh| Sole proprietor| 10 digits| 06GTKT (simplified)  
`llc`| Công ty TNHH| LLC| 10 / 13 digits| 01GTKT  
`jsc`| Công ty cổ phần| JSC| 10 / 13 digits| 01GTKT  
`fdi`| FDI| Foreign-invested| 10 / 13 digits| 01GTKT  
`individual`| Cá nhân| Individual| 10 digits (CCCD-linked)| 06GTKT  
`other`| Khác| Other| —| manual  
  
5

## API surface

Four surfaces: a federated GraphQL subgraph (cross-module joins to PROJ Engagement and INV Invoice); a REST admin surface for bulk operations and migration import; a WebSocket sync for the pipeline drag-drop UI; and an MCP tool catalogue for CUO. 

### GraphQL subgraph
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key", "@requiresScopes"])
    
    type Account @key(fields: "id") {
     id: ID!
     name: String!
     accountType: AccountType!
     vnMst: String
     vnMstValidated: Boolean!
     industry: String
     contacts: [Contact!]!
     deals(status: DealStatus): [Deal!]!
     activities(limit: Int = 50): [Activity!]!
     customFields: JSON
     leadSource: String
     owner: Subject
    }
    
    type Contact @key(fields: "id") {
     id: ID!
     account: Account!
     firstName: String!
     lastName: String!
     displayName: String!
     email: String
     phoneE164: String
     salutation: Salutation!
     title: String
     isPrimary: Boolean!
    }
    
    type Deal @key(fields: "id") {
     id: ID!
     account: Account!
     primaryContact: Contact
     name: String!
     amountMinorVnd: Int!
     currency: Currency!
     pipeline: Pipeline!
     stage: Stage!
     probabilityPct: Int!
     expectedCloseDate: Date
     status: DealStatus!
     leadScore: LeadScore
     nextBestAction: NBA
     activities: [Activity!]!
     vietqr: VietQR
     hoaDon: HoaDon
     acl: DealACL
    }
    
    enum AccountType { SOLE_PROPRIETOR LLC JSC FDI INDIVIDUAL OTHER }
    enum Salutation { ANH CHI BAN EM NEUTRAL }
    enum DealStatus { OPEN WON LOST CANCELLED }
    enum Currency { VND USD }
    
    type Mutation {
     createAccount(input: CreateAccountInput!): Account!
     @requiresScopes(scopes: [["crm.write"]])
     validateAccountMst(id: ID!): Account!
     createDeal(input: CreateDealInput!): Deal!
     moveDealStage(id: ID!, toStageId: ID!): Deal!
     generateVietQR(dealId: ID!, bankBin: String!, accountNumber: String!): VietQR!
     closeDeal(id: ID!, outcome: DealStatus!, lostReason: String): CloseDealResult!
     @requiresScopes(scopes: [["crm.close"]])
     logActivity(input: ActivityInput!): Activity!
     mergeContacts(keepId: ID!, mergeId: ID!): Contact!
     @requiresScopes(scopes: [["crm.contact.merge"]])
    }

### MCP tool catalogue

Tool name| Inputs| Outputs| Annotations  
---|---|---|---  
`cyberos.crm.search_accounts`| query, account_type?, limit| Account| readonly  
`cyberos.crm.get_account`| id| Account| readonly  
`cyberos.crm.create_account`| CreateAccountInput| Account| scope=crm.write  
`cyberos.crm.validate_mst`| mst| {valid, name, address}| readonly  
`cyberos.crm.list_deals`| account_id?, pipeline_id?, stage_id?| Deal| readonly · scope=crm.read  
`cyberos.crm.create_deal`| CreateDealInput| Deal| scope=crm.write  
`cyberos.crm.move_stage`| deal_id, to_stage| Deal| scope=crm.write  
`cyberos.crm.close_deal`| deal_id, outcome, lost_reason?| Deal + side-effects| destructive · human-confirm · scope=crm.close  
`cyberos.crm.generate_vietqr`| deal_id, bank_bin, account_number| VietQR| scope=crm.payment  
`cyberos.crm.forecast`| pipeline_id?, range| Forecast with bands| readonly  
`cyberos.crm.next_best_action`| deal_id?| NBA| readonly  
  
6

## Key flows

### Flow 1 — Add an Account with MST validation

sequenceDiagram autonumber participant U as Member SPA participant API as CRM GraphQL participant MST as vietnam-mst-validate skill participant GDT as GDT registry participant PG as PostgreSQL participant B as memory audit U->>API: createAccount {name, account_type:LLC, vn_mst:"0314…"} API->>API: validate MST format (10 or 13 digits) API->>MST: invoke skill {mst:"0314…"} MST->>GDT: HTTP GET registry endpoint GDT-->>MST: {name:"CÔNG TY TNHH ACME VN", address:"…"} MST-->>API: {valid:true, canonical_name, address} alt name match within tolerance API->>PG: INSERT account {vn_mst_validated:true} else mismatch API->>PG: INSERT account {vn_mst_validated:true, name=canonical_name} API-->>U: warning: name normalised to GDT value end API->>B: crm.account_created API-->>U: 200 Account 

### Flow 2 — Move deal through pipeline (drag-drop)

sequenceDiagram autonumber participant U as Member SPA participant WS as WebSocket participant API as CRM service participant PIPE as Pipeline rules participant PG as PostgreSQL participant N as NATS pipeline events participant B as memory audit U->>U: optimistic move card to new stage U->>WS: deal.move_stage {deal_id, to_stage_id} WS->>API: dispatch API->>PIPE: validate transition (rule engine) alt allowed PIPE-->>API: ok API->>PG: INSERT stage_transition + UPDATE deal.stage_id API->>N: publish pipeline.{pipeline_id} N->>WS: fan-out to other SPAs API->>B: crm.stage_transition API-->>U: confirm (canonical state) else blocked PIPE-->>API: violation API-->>U: rollback + reason end 

### Flow 3 — Deal close → VietQR → hóa đơn → Engagement

sequenceDiagram autonumber participant U as Sales Member participant API as CRM participant VQR as vietnam-bank-transfer skill participant HD as vietnam-vat-invoice skill participant INV as 🧾 INV participant PROJ as 📋 PROJ participant BR as 🧠 memory U->>API: closeDeal(id, outcome:WON) API->>API: validate user has crm.close scope API->>API: UPDATE deal.status=won, closed_date=today API->>VQR: generate VietQR (bank, account, amount, addinfo) VQR-->>API: {qr_payload, qr_image_url} API->>HD: emit hóa đơn (template based on account_type) HD->>HD: assemble XML per Circular 78/2021 HD-->>API: {invoice_number, xml_url} API->>INV: register invoice (status=issued) INV-->>API: invoice_id API->>PROJ: create Engagement (auto-link from Deal) PROJ-->>API: engagement_id API->>BR: crm.deal_closed {ingested into Layer 3 — closed deals only} API-->>U: 200 {deal, vietqr, invoice, engagement} 

(FR pending): deals only ingested into memory after closed / lost — open pipeline data never enters memory.

### Flow 4 — Email auto-log to CRM (tracked domain)

sequenceDiagram autonumber participant EM as ✉️ EMAIL inbound participant ACT as CRM activity feed participant PG as PostgreSQL participant N as NATS EM->>ACT: event {from:"buyer@acme.com", subject:"…", message_id} ACT->>PG: SELECT account WHERE domain = "acme.com" AND tracked=true alt account found ACT->>PG: SELECT contact WHERE email = "buyer@acme.com" ACT->>PG: INSERT activity {kind:"email", source_event_id=message_id} ON CONFLICT DO NOTHING ACT->>N: publish crm.activity.created else not tracked ACT-->>EM: ignore ((FR pending) boundary respected) end 

Personal emails never auto-attach. Only tenant-configured tracked domains; the configuration is itself audited.

### Flow 5 — AI lead scoring nightly batch

sequenceDiagram autonumber participant CR as Nightly cron participant SC as Lead scoring engine participant PG as PostgreSQL participant AG as ⚡ AI Gateway (CUO/CRO-skill) participant B as memory audit CR->>SC: run for tenant SC->>PG: SELECT open deals + features (recency, value, account vertical, history) PG-->>SC: rows SC->>AG: chat.completions {features, scoring rubric} AG-->>SC: scores + reasons SC->>PG: UPSERT lead_score SC->>B: crm.lead_scoring_run {count, avg_score} 

7

## Deal lifecycle

A deal traverses the pipeline stages, ending in one of three terminal statuses (won, lost, cancelled). Stage transitions are governed by per-pipeline workflow rules. 

stateDiagram-v2 [*] --> Qualifying: created Qualifying --> Discovery: budget + authority confirmed Discovery --> Proposal: needs scoped Proposal --> Negotiation: proposal sent Negotiation --> ClosedWon: contract signed Negotiation --> ClosedLost: lost reason recorded Negotiation --> Cancelled: client withdrew Discovery --> ClosedLost: dropped during discovery Proposal --> ClosedLost: rejected Qualifying --> Cancelled: unqualified ClosedWon --> [*] ClosedLost --> [*] Cancelled --> [*] Note right of ClosedWon: VietQR generated · hóa đơn issued · Engagement created in PROJ Note right of ClosedLost: lost_reason mandatory · win/loss analysis triggered 

### Default pipeline stages (sales)

Code| vi| en| Default probability| Category  
---|---|---|---|---  
`qualifying`| Đánh giá| Qualifying| 10%| open  
`discovery`| Khám phá| Discovery| 25%| open  
`proposal`| Đề xuất| Proposal| 50%| open  
`negotiation`| Đàm phán| Negotiation| 75%| open  
`closed_won`| Thắng| Closed Won| 100%| won  
`closed_lost`| Thua| Closed Lost| 0%| lost  
`cancelled`| Huỷ| Cancelled| 0%| lost  
  
8

## Functional Requirements

The CyberOS FR catalogue is being rebuilt one feature at a time via the open [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) Agent Skill.

Previous FR enumerations were archived 2026-05-14 and are no longer reflected on this page. Specific FRs land here as they are re-authored.

9

## Non-Functional Requirements

NFRs from that CRM must satisfy.

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`N(FR pending)`| Pipeline view load (200 deals)| p95 ≤ 350 ms| SPA RUM  
`N(FR pending)`| Stage drag-drop response| p95 ≤ 200 ms| WS latency histogram  
`N(FR pending)`| Account search (10k accounts)| p95 ≤ 300 ms| k6  
`N(FR pending)`| MST validation| p95 ≤ 1.5 s (cache-cold)| histogram  
`N(FR pending)`| Pipeline drag-drop drop-rate (loss)| = 0| memory audit reconciliation  
`N(FR pending)`| Vietnamese salutation accuracy| ≥ 92% on 200-contact eval| quarterly review  
`N(FR pending)`| Service availability| ≥ 99.9% (28-day)| OBS SLO  
`N(FR pending)`| Private-deal isolation| = 0 cross-leak| CI test + quarterly red-team  
`N(FR pending)`| Personal email auto-attach| = 0 (tracked-domain only)| policy gate test  
`N(FR pending)`| Activity-feed idempotency| = 0 duplicates| idempotency key DB constraint  
`N(FR pending)`| Deal close → Engagement durability| 100% within 60 s| memory audit cross-check  
`N(FR pending)`| Hóa đơn XML schema compliance| 100% pass Circular 78/2021| CI test against GDT XSD  
  
10

## Dependencies

CRM depends on AUTH (every call), memory (activity ingest + audit), EMAIL (auto-log), CHAT (meeting notes), Calendar (attendance), AI (scoring + NBA + forecast), MCP, OBS, and three CyberOS skills (vietnam-mst-validate, vietnam-bank-transfer, vietnam-vat-invoice). It is depended on by PROJ (Engagement creation), INV (hóa đơn emission), and PORTAL (client-visible deal status at P2+). 

graph LR subgraph upstream ["CRM depends on"] AUTH["🔐 AUTH"] memory["🧠 memory"] EMAIL["✉️ EMAIL"] CHAT["💬 CHAT"] CAL["📅 Calendar"] AI["⚡ AI Gateway"] MCP["🔌 MCP"] OBS["👁 OBS"] MST["vietnam-mst-validate"] VQR["vietnam-bank-transfer"] HD["vietnam-vat-invoice"] end CRM["🤝 CRM"] subgraph downstream ["CRM is depended on by"] PROJ["📋 PROJ"] INV["🧾 INV"] PORTAL["Portal · P2"] end AUTH --> CRM memory --> CRM EMAIL --> CRM CHAT --> CRM CAL --> CRM AI --> CRM MCP --> CRM OBS --> CRM MST --> CRM VQR --> CRM HD --> CRM CRM --> PROJ CRM --> INV CRM --> PORTAL classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#fef6e0,stroke:#9c750a class memory,MST,VQR,HD shipped class CRM,AUTH,EMAIL,CHAT,CAL,AI,MCP,OBS,PROJ,INV,PORTAL planned 

11

## Compliance scope

CRM holds Customer personal data and processes hóa đơn-relevant fields; it sits inside PDPL / Circular 78/2021 / GDPR scope.

Regulation / standard| Article / clause| CRM feature that satisfies it  
---|---|---  
Vietnam PDPL (Law 91/2025)| Art. 14 — DSAR| DSAR export includes contact + activity + deal data.  
Vietnam Decree 13/2023| Art. 17 — Processing log| Every CRM mutation writes a memory audit row.  
Vietnam Decree 53/2022| Art. 26 — Residency| Per-tenant residency tag; VN tenants on hanoi-1.  
Vietnam Circular 78/2021/TT-BTC| Hóa đơn e-invoice format| vietnam-vat-invoice emits compliant XML.  
Vietnam Decree 123/2020/NĐ-CP| Invoice issuance & storage| HOA_DON table with XML link + 10-year retention.  
GDPR (EU 2016/679)| Art. 17 — Right to erasure| Contact delete + DSAR-driven hard purge.  
GDPR| Art. 25 — Privacy by design| Per-deal ACL prevents cross-workspace leak.  
ISO/IEC 27001:2022| A.8.2 — Privileged access| RBAC + per-deal ACL.  
SOC 2 Type II| CC6.1 — Logical access| RLS + ACL enforced at every mutation.  
OWASP Top-10 (web)| A01 — Broken access control| ACL enforced server-side; CI test on every PR.  
  
12

## Risk entries

CRM-specific risks tracked in the [risk register](<../../reference/risk-register.html#crm>).

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-CRM-001`| GDT MST registry endpoint outage| Medium| Medium| CTO| 7-day cache; validation marked stale; user warned but write permitted.  
`R-CRM-002`| Hóa đơn XML schema drift breaks emission| Medium| High| CFO| CI test against GDT XSD weekly; skill version pinned; rollback plan documented.  
`R-CRM-003`| Personal email auto-attaches to CRM via misconfig| Low| High| CSO| Tracked-domain config audited; CI test asserts personal domains stay out.  
`R-CRM-004`| Private-deal leak via cross-workspace GraphQL query| Low| High| CSO| ACL CI gate; quarterly red-team on subgraph.  
`R-CRM-005`| Activity-feed duplicate from EMAIL retry| Medium| Low| CTO| Idempotency key on source event id; DB unique constraint.  
`R-CRM-006`| Lead-scoring bias against specific account verticals| Medium| Medium| CDO| Quarterly fairness review of scoring inputs; reasons surfaced to user.  
`R-CRM-007`| VietQR generated with wrong account number → payment misdirected| Low| High| CFO| Per-tenant bank-account allow-list; Member confirm step on first use.  
`R-CRM-008`| Merge proposes false-positive (same email shared family)| Medium| Low| CRO| Merge requires explicit Member confirm; confidence threshold ≥ 0.85.  
`R-CRM-009`| memory ingest of closed deals leaks competitive data| Low| Medium| CDO| Closed deals respect deal ACL on memory side; private deals never ingested.  
`R-CRM-010`| Open-pipeline data ingested into memory by accident ((FR pending) violation)| Low| High| CDO| memory ingest path gated by deal.status & ('won','lost'); CI test asserts.  
`R-CRM-011`| Deal-to-Engagement bridge fails partially — Engagement created without rate card| Medium| High| CTO| Bridge is transactional; either full Engagement + rate card OR rollback; failed bridge surfaces as actionable error to AM.  
`R-CRM-012`| Won deal converted to wrong Engagement billing mode (T&M vs fixed-fee)| Medium| Medium| COO| AM must explicitly choose billing mode in conversion dialog; default is deal's pricing-page choice; locked once Engagement created.  
`R-CRM-013`| CUO next-action skill suggests inappropriate move (e.g. propose to lost-cause deal)| Medium| Low| CRO| Confidence threshold ≥ 0.60; AM sees suggestion ranked, never auto-actioned; quarterly review of suggestion acceptance correlation with deal outcomes.  
`R-CRM-014`| Vertical-pack drift (cyberskill-vn pack updates → MST validation regression)| Medium| High| CTO| Pack version pinning per tenant; integration tests against GDT staging endpoint; quarterly drill on real MST samples.  
`R-CRM-015`| Account-merge causes loss of historical activity for one of the merged accounts| Low| Medium| CRO| Merge preserves both audit trails (no deletion); merged account points to "survivor" via redirect record; unmerge is supported up to 90 days post-merge.  
  
13

## KPIs

CRM rolls up 10 KPIs covering pipeline health, AI feature efficacy, integration durability, and VN-specific flow correctness.

KPI| Formula| Source| Target  
---|---|---|---  
**Pipeline value (VND)**|  sum(open_deal.amount × probability)| PG| tracked daily  
**Stage drag-drop p95**|  histogram| OBS| ≤ 200 ms  
**Win rate**|  won / (won + lost)| memory audit| tracked per pipeline  
**Average deal cycle**|  median(closed_date − created_at)| memory audit| tracked per Engagement type  
**Lead-scoring precision**|  true_hot / scored_hot| retrospective on wins| ≥ 70%  
**NBA acceptance**|  actioned / suggested| SPA telemetry| tracked; target ≥ 35%  
**Activity auto-log durability**|  logged / source_events| EMAIL ↔ CRM reconciliation| 100%  
**Hóa đơn emission success**|  issued / close_won| memory audit| 100%  
**MST validation rate**|  validated / vn_accounts| memory audit| ≥ 95%  
**Cross-workspace ACL violations**|  count| memory audit| = 0  
**Deal-to-Engagement conversion rate**|  engagements_created / deals_won| PROJ-CRM cross-check| tracked; expect 0.70–0.95 (some won deals are pure resale)  
**Conversion bridge p95**|  histogram (click → Engagement created)| OBS| ≤ 3 s  
**Win/loss memory citation rate**|  new deals citing past win/loss memories / total open deals| MEMORY_LINK table| tracked; expect ≥ 0.30 of high-priority deals  
**Next-action acceptance per deal**|  actioned suggestions / shown suggestions| SPA telemetry| ≥ 0.35  
**Stage-stuck deal alert**|  deals in same stage > 30 days| nightly batch| flagged on AM dashboard; SLA-style nudge  
**Forecast accuracy**|  |actual_close − forecast_close| / forecast_close| retrospective| ≤ 0.20 quarterly  
  
14

## RACI matrix

CRM is owned by CRO seat (interim CEO).

Activity| CEO| CRO| CFO| CTO| CSO| CMO  
---|---|---|---|---|---|---  
Service design + spec| A| R| C| C| C| C  
Implementation| I| C| I| A/R| C| I  
Pipeline / stage config| C| A/R| I| I| I| C  
Tracked-domain config| I| A| I| R| C| I  
Hóa đơn flow review| I| C| A/R| C| I| I  
Lead-scoring fairness review| C| A| I| C| I| R  
ACL audit| C| C| I| R| A| I  
DSAR fulfilment| I| C| C| C| R| I  
Incident response| A| R| C| R| R| I  
  
**R** Responsible · **A** Accountable · **C** Consulted · **I** Informed.

15

## Planned CLI surface

`cyberos-crm` for tenant operators and bulk import / export.

### 1\. Add a Vietnamese LLC account
    
    
    $ cyberos-crm account create \
     --name "ACME Vietnam Co., Ltd." \
     --type llc \
     --mst 0314556677 \
     --owner stephen@cyberskill.world
    
    [mst] vietnam-mst-validate skill invoked
    [mst] ✓ matches GDT registry: "CÔNG TY TNHH ACME VIỆT NAM"
    [mst] ⚠ name normalised to GDT canonical
    [create] account_id: 01HZK7…
    [audit] memory seq=15301 chain=…

### 2\. List contacts at an account
    
    
    $ cyberos-crm contact list --account 01HZK7…
    
    display_name email phone salutation primary
    Nguyễn Văn A vana@acme.vn +84903123456 Anh yes
    Trần Thị B thib@acme.vn +84909222333 Chị no
    Phạm Minh C minhc@acme.vn +84908111222 Anh no

### 3\. Move a deal stage
    
    
    $ cyberos-crm deal move --id 01HZK8… --to-stage proposal
    
    [validate] transition discovery → proposal allowed
    [move] deal status: open stage: proposal probability: 50%
    [ws] broadcast to 4 connected SPAs
    [audit] memory seq=15315 chain=…

### 4\. Close a deal (won) — full flow
    
    
    $ cyberos-crm deal close --id 01HZK8… --outcome won
    
    [close] status=won · closed_date=2026-05-14
    [vietqr] vietnam-bank-transfer skill invoked
    [vietqr] generated: bank=Vietcombank · account=0011 0023 4567 · amount=750,000,000 VND
    [hoadon] vietnam-vat-invoice skill invoked (template=01GTKT)
    [hoadon] invoice_number: C26TAA/0001234 · XML: s3:/cyberos/hoadon/01HZK9…xml
    [inv] INV invoice registered (id=01HZKB…)
    [proj] Engagement created: "ACME Q3 platform build"
    [memory] deal ingested into Layer 3 (closed deal)
    [audit] memory seq=15327 chain=…

### 5\. Forecast
    
    
    $ cyberos-crm forecast --pipeline sales --range 90d
    
    pipeline: sales (next 90 days)
    band vnd_low vnd_mid vnd_high
    50% 1,200,000,000 1,450,000,000 1,750,000,000
    80% 800,000,000 1,100,000,000 1,500,000,000
    95% 500,000,000 750,000,000 1,100,000,000
    
    top 5 deals by probability×value:
     01HZK8… ACME Q3 build 750,000,000 75%
     01HZKC… BetaCo retainer 240,000,000 80%
     …

### 6\. Merge contacts
    
    
    $ cyberos-crm contact merge --keep 01HZKD… --merge 01HZKE…
    
    [merge] keep: Nguyễn Văn A (vana@acme.vn)
    [merge] merge: A Nguyen Van (vana@acme.vn) ← duplicate
    [merge] ✓ activities reassigned: 47
    [merge] ✓ deals reassigned: 3
    [merge] ✓ contact 01HZKE… tombstoned
    [audit] memory seq=15334 chain=…

### 7\. DSAR export for a contact
    
    
    $ cyberos-crm dsar-export --contact 01HZKD… --output a-nguyen.zip
    
    [dsar] contact: 01HZKD… Nguyễn Văn A
    [dsar] activities: 412
    [dsar] deals: 6 (3 won, 1 lost, 2 cancelled)
    [dsar] written: a-nguyen.zip (8 MB)

16

## Phase status & estimates

Status

Planned

P1 · design phase

Est. LoC

~8,500

Rust + TS pipeline UI

Planned tests

100+

incl. ACL fuzzing

External libs

~12

axum · sqlx · libphonenumber

CLI subcommands

~20 planned

`cyberos-crm`

P1 budget

~$60/mo

Fargate + RDS + Redis

Capability| Status  
---|---  
Account / Contact / Deal CRUD| planned · P1  
Configurable pipeline + stages| planned · P1  
WebSocket drag-drop pipeline| planned · P1  
Activity auto-log (EMAIL / CHAT / Calendar)| planned · P1  
vietnam-mst-validate integration| planned · P1  
vietnam-bank-transfer (VietQR)| planned · P1  
vietnam-vat-invoice (hóa đơn)| planned · P1  
Deal close → PROJ Engagement| planned · P1  
AI lead scoring (nightly batch)| planned · P1  
Next-best-action suggestion| planned · P1  
Confidence-banded forecast| planned · P1  
Contact merge candidate detection| planned · P1  
Per-deal ACL| planned · P1  
Vietnamese salutation helper| planned · P1  
HubSpot / Salesforce migration import| planned · P2+  
Client-visible PORTAL view| planned · P2+  
  
17

## References

  * **Bigger picture (§0 above):** 3 strategic roles + orchestration spine Mermaid + 9-row auto-vs-human matrix.
  * **Cross-module page links:** [proj.html](<../proj/index.html>) · [memory.html](<../memory/index.html>) · [cuo.html](<../cuo/index.html>) · [skill.html](<../skill/index.html>) · [email.html](<../email/index.html>) · [inv.html](<../inv/index.html>) · [ten.html](<../ten/index.html>)
  * **Deal-to-Engagement join contract:** [PROJ §2.5](<../proj/index.html#orchestration-spine>) — 9-row canonical contract table.
  * **Vertical-pack pattern:** [SKILL §3.6](<../skill/index.html>) — cyberskill-vn ships first; sg/id/th/eu/us follow.
  * **memory auto-sync vision:** [MEMORY_AUTOSYNC_DESIGN.md §5](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) — CRM closed deals become memory win/loss memories.
  * **Build-readiness audit:** `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`) — CRM at P1 · start (P1, after PROJ).
  * **FR authoring discipline:** [modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>).
  * **Vietnam Decree 123/2020/NĐ-CP** — Electronic invoice issuance and storage.
  * **Vietnam Circular 78/2021/TT-BTC** — Hóa đơn format.
  * **Vietnam PDPL (Law 91/2025/QH15)** — Art. 7 erasure, Art. 14 DSAR, Art. 20 security.
  * **Vietnam Decree 13/2023** — Personal data processing.
  * **Napas247 / VietQR specification** — bank transfer QR.
  * **vietnam-mst-validate / vietnam-bank-transfer / vietnam-vat-invoice skills** — CyberOS skill bundle (cyberskill-vn pack).
  * **libphonenumber** — E.164 normalisation.
  * **Architecture context:** [infrastructure.html#crm](<../../architecture/infrastructure.html#crm>).



★

## Personas & skill bundles that touch CRM

CRM holds prospect + customer relationships and feeds the revenue cluster. Of the 47 CUO personas, the GTM seats below interact with CRM continuously.

Persona affinities (8 of 47)

  * [chief-revenue-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-revenue-officer/workflows>) · weekly-revenue-cadence + monthly-forecast + churn-analysis
  * [chief-sales-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-sales-officer/workflows>) · weekly-pipeline-review + quarterly-account-plan
  * [chief-customer-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-customer-officer/workflows>) · health-review + CAB + churn-collab + per-account engagement
  * [chief-marketing-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-marketing-officer/workflows>) · per-campaign-plan + marketing-metrics-review
  * [chief-growth-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-growth-officer/workflows>) · weekly-growth-cadence + experimentation portfolio
  * [chief-commercial-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-commercial-officer/workflows>) · partner-scorecard + strategic-partnership-per-deal
  * [chief-experience-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-experience-officer/workflows>) · per-journey-charter + customer-360 engagement
  * [chief-brand-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-brand-officer/workflows>) · per-analyst-brand-briefing



Skill-bundle reads & writes

  * [pipeline-report-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/pipeline-report-author>) \+ audit · weekly/monthly pipeline output
  * [account-plan-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/account-plan-author>) \+ audit · enterprise key-account plan
  * [forecast-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/forecast-author>) \+ audit · monthly revenue forecast
  * [churn-analysis-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/churn-analysis-author>) \+ audit · quarterly churn diagnosis
  * [customer-health-review-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/customer-health-review-author>) \+ audit · CCO recurring
  * [customer-360-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/customer-360-author>) \+ audit · CDO + CXO partnership
  * [vietnam-mst-validate](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/vietnam-mst-validate>) · VN customer onboarding tax-ID check



[← Previous module: TIME](<../time/index.html>) [Next module: KB →](<../kb/index.html>)
