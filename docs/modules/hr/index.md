---
title: HR — Member lifecycle · Onboarding orchestrator · Performance signal aggregator · CyberOS
source: website/docs/modules/hr/index.html
migrated: FR-DOCS-002
---

HR is the **Member lifecycle plane** — the place where a person becomes an actor in the system and the place where they exit cleanly. The data shape is conventional (profile, contract, leave, document) but the discipline is Vietnamese-labour-law-first: every contract honours Decree 145/2020 working-hour caps, every leave type is tagged with its statutory basis (annual, sick, maternity, paternity, sabbatical, unpaid, bereavement), and every CCCD photo lives in a separate KMS keyspace with sev-1 access logging. The onboarding checklist is the multi-module orchestrator: HR fires events to AUTH (account create), CRM (Member record), TIME (timesheet enrolment), CHAT (default workspaces), REW (initial pay band lookup — read-only, REW owns the number), and ESOP (founding-grant if applicable). 

Status

Planned

P1 · design phase

Est. LoC

~5,200

Rust (axum) + sqlx

Planned tests

80+

incl. leave-accrual property tests

Leave types

8

annual · sick · maternity · paternity · sabbatical · unpaid · bereavement · public-holiday

Contract types

5

indefinite · fixed-term · probation · part-time · contractor

PII keyspace

separate KMS

CCCD photo column distinct from comp keys

Depends on

AUTH · memory · OBS

\+ DOC for e-sign

Used by

REW · LEARN · ESOP · TIME · CHAT

Member-id is the cross-module spine

0

## The bigger picture — three strategic roles

HR is not a payroll bolt-on. It is the Member-id authority for the whole platform. Strip it out and every downstream module gets a slightly different roster, drift accumulates, compliance breaks. The Vietnamese labour-law discipline matters too — Decree 145 / 152 / 13 / PDPL are not "compliance things"; they're the operating constraints under which the entire team works. 

Role 1 · Member lifecycle

One state machine: hire → transfer → exit

Pre-hire → active → on-leave → suspended → terminating → terminated. Each transition fires events that AUTH (subject provisioning), TIME (timesheet enrolment), REW (pay band lookup), ESOP (grant vesting state), KB (access scope), and 4 other modules consume. Contract type and Vietnamese statutory entitlements are first-class fields. CCCD photo lives in a separate KMS keyspace. 

Role 2 · Onboarding orchestrator

New Member day 1 → ramped at day 30

HR fires a multi-module onboarding playbook: AUTH creates the subject + provisions roles; LEARN enrols the new Member in role-specific training; KB grants access to the docs they need; PROJ creates the ramp-plan Issues (read these memories, shadow these engagements, complete this onboarding rubric); CHAT adds them to default workspaces. Day-by-day checklist visible to manager + new hire + CHRO. 

Role 3 · Performance signal aggregator

Read-only consumer of cross-module signals

HR reads (never writes) performance signals from PROJ (calibration drift, blocker authoring rate), TIME (hour discipline), LEARN (course completion). Aggregates into Manager 1:1 prep + quarterly review prep + REW comp-recommendation inputs. HR never holds the comp number — REW does. HR holds the signals; REW holds the dollars; the two stay separated. 

### HR as Member-id spine across modules

flowchart LR HR["👥 HR  
Member directory · contracts · leave · lifecycle"] AUTH["🔐 AUTH  
Subject provisioning"] TIME["⏱ TIME  
timesheet enrolment"] REW["💰 REW  
pay band + comp number"] ESOP["📈 ESOP  
vesting state"] LEARN["🎓 LEARN  
role-specific training"] KB["📚 KB  
access scope"] PROJ["📋 PROJ  
calibration signals"] CHAT["💬 CHAT  
workspace add"] memory["🧠 memory  
lifecycle audit"] HR --> AUTH HR --> TIME HR -- "pay band → ref only" --> REW HR --> ESOP HR --> LEARN HR --> KB HR --> CHAT PROJ -- "calibration · blocker rate" --> HR TIME -- "hour discipline" --> HR LEARN -- "completion" --> HR HR --> memory classDef hub fill:#fce7f3,stroke:#9d174d,stroke-width:3px,color:#500724 classDef mod fill:#e0e7ff,stroke:#3730a3 classDef memory fill:#fef6e0,stroke:#9c750a class HR hub class AUTH,TIME,REW,ESOP,LEARN,KB,PROJ,CHAT mod class memory memory 

### Auto vs human-in-loop operations matrix

Operation| How it happens| Why this split  
---|---|---  
Hire — pre-hire to active| **Manual** HR action; **auto-orchestrate** downstream| Hiring is a relationship decision; orchestration is mechanical.  
Annual-leave accrual| **Auto** nightly| Decree 145 specifies accrual rules; deterministic.  
Sick-leave deduction| **Manual** Member entry; **auto-validate** caps| Member self-reports; cap validation runs at write time.  
Maternity / paternity leave| **Manual** with statutory basis tag| Decree 13/2023 + Labour Code 2019 mandate; HR confirms eligibility.  
Contract renewal reminder| **Auto** 90 days pre-expiry| Statutory; missing renewal triggers labour-law issues.  
Onboarding playbook fire| **Auto** on active transition| Multi-module fan-out; saga pattern.  
Performance signal aggregation| **Auto** nightly| Read-only; raw signals never used as sole basis for decision.  
Comp recommendation| **Auto** draft; **manual** REW review| HR proposes; REW disposes; comp number lives in REW.  
Termination — terminate| **Manual** CHRO + CEO sign-off; **auto-orchestrate** downstream| Termination is gravely consequential; auto-fan-out only after dual sign-off.  
CCCD photo access| **Manual** request + audit row per view| VN sensitive PII; every access is sev-1 audited; never bulk-export.  
  
1

## Why HR exists

HR is the single source of truth for who is a Member of the company and what their employment relationship is. Without that single source of truth, every downstream module reinvents a partial picture: REW imagines a payroll roster, TIME imagines a list of timesheet-enabled people, ESOP imagines a list of grantees. They drift; data inconsistency follows; compliance violations follow shortly after. HR centralises the answer to a deceptively hard question — "is this person currently employed by this tenant, in what capacity, since when, and with what statutory entitlements?" — and makes that answer a row, not a query across five systems. 

🇻🇳

Vietnamese labour law first-class

Decree 145/2020 hour caps, Decree 152/2020 SI rates, BHXH/BHYT/BHTN numbers, sabbatical accrual — all schema fields, not free-text comments.

🔐

PII keyspace separation

CCCD photos, contracts, government IDs each in a separate KMS-wrapped column-level key. Compensation is structurally absent — that's REW's job.

🔁

Lifecycle orchestrator

Onboarding fires events to 8 modules; offboarding revokes from all of them. One workflow, eight side-effects, one audit row per transition.

The bet is the same bet AUTH makes about identity: pay the cost once at the canonical layer, and every module inherits the property for free. Without HR as the canonical Member directory, "who can take leave?" becomes a search across mailing lists, Slack channels, and Excel files. With HR as the canonical Member directory, the answer is a JOIN against `member` and `leave_balance`, scoped by `tenant_id`, audited in memory, and provable to a Decree 13/2023 auditor. 

2

## What it does — 5W1H2C5M

A structured decomposition of HR's scope. Every cell traces back to and.

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is HR?| A Member-directory + lifecycle service. Stores profile, contract, leave balance, sabbatical accrual, training records (pointer to LEARN), and government ID with KMS-wrapped storage. Runs the onboarding orchestrator and the offboarding orchestrator.  
**5W · Who**|  Who is in HR?| **Members:** employees, contractors, interns — anyone who needs a payroll row, a leave balance, or a Member-id. **Owners:** HR/Ops Lead (R/A); CEO (A for policy); CHRO (R for cross-tenant policy at P3+); DPO (R for PDPL DSAR).  
**5W · When**|  When does HR act?| (a) Member join — onboarding wizard fires; (b) every leave request; (c) every contract renewal / amendment; (d) monthly close — REW reads Member roster; (e) Member exit — offboarding orchestrator runs; (f) annual sabbatical-eligibility tick.  
**5W · Where**|  Where does it run?| P1: single region (Singapore SG-1) backed by AWS RDS Postgres with column-level KMS for CCCD + contract PDFs. P3: VN data-residency option for Vietnamese tenants (vn-hanoi-1).  
**5W · Why**|  Why a separate module?| Because Member-directory drift across modules is the single biggest source of operational pain in growing companies. Centralise the primitive, never the policy.  
**1H · How**|  How does it work?| Postgres with column-level KMS on PII columns. GraphQL subgraph publishes Member + Profile to Apollo Router. NATS events for lifecycle transitions: `hr.member.joined`, `hr.leave.requested`, `hr.leave.approved`, `hr.contract.renewed`, `hr.member.terminated`. AUTH RBAC scopes every read; CCCD reads are sev-1 audit rows.  
**2C · Cost**|  Cost budget?| P1: ~$25/month (RDS row-scoped to HR schema + one Fargate task). 50-tenant: ~$80/month. Per-member-month cost: ~$0.50 amortised.  
**2C · Constraints**|  Constraints?| (a) Decree 145/2020 — max 200 hours/year overtime; system rejects timesheet entries that would push a Member past the cap. (b) Decree 152/2020 — BHXH 8% / BHYT 1.5% / BHTN 1% employee contribution rates (employer 17.5% / 3% / 1%); stored as parameter version. (c) PDPL Art. 14 — DSAR export available within 30 days. (d) Comp data structurally excluded.  
**5M · Materials**|  Stack?| Rust 1.81 · axum 0.7 · sqlx · PostgreSQL 16 · async-graphql for the subgraph · KMS for column-level encryption · S3 for contract PDFs (with object-lock for retention) · NATS JetStream for events.  
**5M · Methods**|  Method choices?| Append-only contract table with `effective_to` \+ `superseded_by` for amendments (anti-retroactive). Leave-balance state machine: `requested → approved → consumed` or `requested → rejected`. Accrual computed lazily at quarter boundaries (not real-time) to keep audit deterministic.  
**5M · Machines**|  Deployment?| Fargate task in SG-1 (P1). Multi-AZ Postgres RDS. S3 for contract PDFs with retention lock = 10 years (matches VN SI/PIT statutory minimum).  
**5M · Manpower**|  Who maintains?| 0.25 FTE today (covered by HR/Ops Lead). By P3: full HR/Ops Lead + 1 engineer.  
**5M · Measurement**|  How measured?| (FR pending)..008. KPIs: onboarding-checklist completion time, leave-request approval p95, CCCD-access audit-row coverage = 100%, contract-renewal lead time.  
  
3

## Architecture

HR is one Rust service with four surfaces (GraphQL subgraph, REST admin, NATS event publisher, MCP tool catalogue), three stores (Postgres for relational state, S3 for contract PDFs + CCCD images, KMS for per-column encryption keys), and an audit sink to memory. The diagram below shows the canonical request flow for the onboarding orchestrator. 

graph TB subgraph CLIENTS ["Clients"] SPA["CyberOS SPA  
(HR admin UX)"] CUO["🤖 CUO router  
(narrative only)"] MEMBER["Member self-service  
(leave + profile)"] end subgraph EDGE ["Edge"] AR["Apollo Router  
JWT + RBAC"] end subgraph HR ["HR service (Rust · axum)"] GQL["GraphQL subgraph  
Member · Profile · Leave"] REST["REST admin  
onboarding · termination"] MCP["MCP tool catalogue  
(read-mostly)"] OB["onboarding.rs  
multi-module orchestrator"] OFF["offboarding.rs  
revoke + settle"] LV["leave.rs  
request · approve · accrual"] CON["contract.rs  
versioned · KMS-wrapped"] SAB["sabbatical.rs  
5-year eligibility tick"] DOC["doc_bridge.rs  
e-sign integration"] end subgraph STORES ["Stores"] PG[("PostgreSQL  
member · contract · leave  
RLS by tenant_id")] S3[("AWS S3  
contract PDFs · CCCD images  
10-year object-lock")] KMS[("AWS KMS  
per-column key  
CCCD distinct from contracts")] end subgraph DOWNSTREAM ["Lifecycle downstreams"] AUTH["🔐 AUTH  
account create / revoke"] REW["💎 REW  
roster sync · final pay"] ESOP["📊 ESOP  
Good/Bad Leaver branch"] LEARN["📈 LEARN  
training records"] TIME["⏱ TIME  
timesheet enrolment"] CHAT["💬 CHAT  
workspace provision"] DOCM["📄 DOC  
e-sign"] end subgraph SINKS ["Audit & telemetry"] memory["🧠 memory  
hr.lifecycle rows"] OBS["👁 OBS  
traces + metrics"] NATS["📡 NATS JetStream  
hr.* events"] end SPA --> AR CUO --> AR MEMBER --> AR AR --> GQL AR --> REST AR --> MCP REST --> OB REST --> OFF GQL --> LV REST --> CON CON --> SAB CON --> DOC DOC --> DOCM OB --> AUTH OB --> TIME OB --> CHAT OB --> LEARN OFF --> AUTH OFF --> REW OFF --> ESOP GQL --> PG REST --> PG CON --> S3 S3 --> KMS PG --> KMS GQL --> memory REST --> memory OB --> NATS OFF --> NATS LV --> NATS HR --> OBS classDef planned fill:#fde7b3,stroke:#9c750a classDef store fill:#f5f3ff,stroke:#7c3aed classDef sink fill:#f5ede6,stroke:#45210e classDef down fill:#fef6e0,stroke:#9c750a class GQL,REST,MCP,OB,OFF,LV,CON,SAB,DOC planned class PG,S3,KMS store class memory,OBS,NATS sink class AUTH,REW,ESOP,LEARN,TIME,CHAT,DOCM down 

### Internal components

Component| Path (planned)| Responsibility  
---|---|---  
`member.rs`| services/hr/src/member.rs| Member CRUD. Manages name, role, level, start date, manager_id self-reference. RLS by `tenant_id`.  
`profile.rs`| services/hr/src/profile.rs| Extended profile fields — DoB, address, BHXH/BHYT/BHTN numbers, emergency contact, bank account for payroll (encrypted column).  
`contract.rs`| services/hr/src/contract.rs| Versioned contracts — indefinite, fixed-term, probation, part-time, contractor. Append-only with `effective_to` \+ `superseded_by`. PDF in S3 (KMS-wrapped).  
`leave.rs`| services/hr/src/leave.rs| Leave request state machine: `draft → submitted → approved → consumed` or `rejected`. Calendar visibility via TIME integration.  
`accrual.rs`| services/hr/src/accrual.rs| Quarterly accrual job for annual leave + sabbatical. Deterministic from contract effective dates.  
`sabbatical.rs`| services/hr/src/sabbatical.rs| Sabbatical eligibility tick — every 5 continuous years per Total Rewards Appendix. Emits `hr.sabbatical.eligible` event.  
`onboarding.rs`| services/hr/src/onboarding.rs| Multi-module orchestrator. Checklist with state per step. Idempotent retry on partial failure.  
`offboarding.rs`| services/hr/src/offboarding.rs| Multi-module revoker. Settlement compute (via REW), ESOP Good/Bad Leaver branch decision (CFO + Founder co-sign), asset return checklist, AUTH revoke.  
`document.rs`| services/hr/src/document.rs| Document storage layer — contract PDFs, CCCD photos, NDA, signed offer letters. Each with classification tag (`restricted` for CCCD).  
`doc_bridge.rs`| services/hr/src/doc_bridge.rs| Bridge to DOC module for WebAuthn-bound e-signature flows.  
`cccd.rs`| services/hr/src/cccd.rs| CCCD (Vietnamese citizen ID) photo handler — separate KMS keyspace, sev-1 access audit, never embedded in API responses.  
`org_chart.rs`| services/hr/src/org_chart.rs| Auto-rendered org chart from `manager_id` self-references. Returns adjacency list + Mermaid string for SPA.  
`review_hook.rs`| services/hr/src/review_hook.rs| Performance-review hook to LEARN. HR initiates a review cycle; LEARN owns the peer-review workflow; outcome summary lands back as a non-comp HR field.  
`audit_bridge.rs`| services/hr/src/audit_bridge.rs| Writes every lifecycle transition to memory. CCCD reads are sev-1 ((FR pending)).  
`migrations/`| services/hr/migrations/| sqlx migrations. RLS by `tenant_id`. Separate column-level KMS for CCCD vs contracts vs comp-shadow fields.  
  
4

## Data model

The schema is normalised around the `Member` as the canonical entity. Contracts are append-only with supersession. Leave is a state-machine row with a balance side-table. CCCD lives in its own table with its own KMS key handle, so a developer cannot accidentally over-fetch by joining the Member table. 

erDiagram TENANT ||--o{ MEMBER: "employs" MEMBER ||--|| PROFILE: "extended fields" MEMBER ||--o{ CONTRACT: "has versioned" MEMBER ||--o{ LEAVE_REQUEST: "submits" MEMBER ||--|| LEAVE_BALANCE: "accrues" MEMBER ||--o{ DOCUMENT: "stores" MEMBER ||--o| CCCD_RECORD: "has (KMS-isolated)" MEMBER ||--o{ ONBOARDING_TASK: "completes" MEMBER ||--o{ REVIEW_CYCLE: "subject of" MEMBER ||--o{ SABBATICAL_TICK: "accrues toward" CONTRACT ||--o{ CONTRACT_AMENDMENT: "amended by" REVIEW_CYCLE ||--o{ REVIEW_OUTCOME: "produces summary" MEMBER { uuid id PK uuid tenant_id FK string email string display_name string role_code "engineer | designer | …" string level "L1 | L2 | L3 | …" date start_date date end_date "NULL if active" uuid manager_id FK "self-ref" string status "active | on_leave | terminated" string sync_class "private (always)" timestamp created_at } PROFILE { uuid member_id PK date dob string address_encrypted "KMS-wrapped" string bhxh_number "Vietnamese social insurance" string bhyt_number "health insurance" string bhtn_number "unemployment insurance" string emergency_contact_encrypted string bank_account_encrypted "for payroll" string nationality } CONTRACT { uuid id PK uuid member_id FK string kind "indefinite | fixed_term | probation | part_time | contractor" date effective_from date effective_to "NULL = open-ended" uuid superseded_by FK "NULL if current" string pdf_s3_uri "KMS-wrapped" bytea pdf_sha256 string status "draft | signed | active | superseded | terminated" timestamp signed_at } CONTRACT_AMENDMENT { uuid id PK uuid contract_id FK string change_summary date effective_from string pdf_s3_uri timestamp created_at } LEAVE_REQUEST { uuid id PK uuid member_id FK string kind "annual | sick | maternity | paternity | sabbatical | unpaid | bereavement | public_holiday" date start_date date end_date decimal days string status "draft | submitted | approved | rejected | consumed | cancelled" uuid approved_by FK string reason timestamp submitted_at } LEAVE_BALANCE { uuid member_id PK decimal annual_remaining decimal sick_remaining decimal sabbatical_accrued_days "from sabbatical_tick" integer service_years date last_accrual_at } SABBATICAL_TICK { uuid id PK uuid member_id FK date period_start date period_end boolean continuous "no gap longer than X" integer ticks_so_far "out of 5" boolean eligible_at "5th tick reached" } DOCUMENT { uuid id PK uuid member_id FK string kind "nda | offer | promotion_letter | id_proof | other" string s3_uri bytea sha256 string classification "internal | confidential | restricted" timestamp created_at } CCCD_RECORD { uuid member_id PK string cccd_number_encrypted "KMS key = cccd, separate" string front_photo_s3_uri "KMS-wrapped" string back_photo_s3_uri "KMS-wrapped" timestamp last_accessed_at integer access_count "sev-1 audit on read" } ONBOARDING_TASK { uuid id PK uuid member_id FK string task_code "calendar_import | mailbox_forward | tauri_install | …" string status "pending | in_progress | done | skipped" timestamp completed_at } REVIEW_CYCLE { uuid id PK uuid member_id FK string period "2026-Q2" string status "open | in_review | closed" timestamp opened_at timestamp closed_at } REVIEW_OUTCOME { uuid review_cycle_id PK string summary "outcome only; no per-judge scores" string recommendation "advance | hold | refine" uuid recorded_by FK timestamp recorded_at } 

### Leave-type catalogue (Vietnamese labour law context)

Code| Statutory basis| Default entitlement| Notes  
---|---|---|---  
`annual`| Labour Code Art. 113| 12 working days/yr (≥ 5 yrs service: +1 day per 5 yrs)| Accrued quarterly; cannot exceed 30 days carryover.  
`sick`| Decree 152/2020 Art. 26| 30/40/60 days/yr (tier per BHXH service period)| Requires medical certificate ≥ 3 days.  
`maternity`| Labour Code Art. 139| 6 months| Twins: +30 days/child. BHXH-paid.  
`paternity`| Decree 152/2020 Art. 34| 5–14 working days| 5 days normal, 7 for C-section, more for twins.  
`sabbatical`| Total Rewards Appendix| 4 weeks every 5 continuous years| CyberSkill-specific; eligibility tracked by `SABBATICAL_TICK`.  
`unpaid`| Labour Code Art. 115| by agreement| Beyond annual + sick allocations.  
`bereavement`| Labour Code Art. 115| 3 days (immediate family)| Direct ascendant/descendant/spouse.  
`public_holiday`| Labour Code Art. 112| 11 days/yr (VN)| Auto-charged by calendar, not requested.  
  
5

## API surface

Three surfaces: a federated GraphQL subgraph for cross-module Member queries, a REST admin API for the orchestrators (onboarding, offboarding, contract issuance), and an MCP tool catalogue for the CUO/CHRO-skill agent. Compensation routes are never exposed here — those live on REW behind a CFO + CHRO co-sign predicate. 

### GraphQL subgraph (federated)

HR publishes Member + Profile + LeaveBalance to Apollo Router. CCCD and contract PDF URIs are never resolvable through the subgraph — admin REST only.
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key", "@external", "@shareable", "@requiresScopes"])
    
    type Member @key(fields: "id") {
     id: ID!
     tenantId: ID!
     email: String!
     displayName: String!
     roleCode: String!
     level: String!
     startDate: Date!
     endDate: Date
     managerId: ID
     status: MemberStatus!
     profile: Profile @requiresScopes(scopes: [["hr.profile_read"]])
     leaveBalance: LeaveBalance @requiresScopes(scopes: [["hr.leave_read"]])
     reports: [Member!]! @requiresScopes(scopes: [["hr.read"]])
    }
    
    type Profile @requiresScopes(scopes: [["hr.profile_read"]]) {
     dob: Date
     bhxhNumber: String
     bhytNumber: String
     bhtnNumber: String
     nationality: String!
    }
    
    type LeaveBalance {
     annualRemaining: Float!
     sickRemaining: Float!
     sabbaticalAccruedDays: Float!
     serviceYears: Int!
     lastAccrualAt: DateTime!
    }
    
    type LeaveRequest @key(fields: "id") {
     id: ID!
     memberId: ID!
     kind: LeaveKind!
     startDate: Date!
     endDate: Date!
     days: Float!
     status: LeaveStatus!
     approvedBy: ID
     reason: String
     submittedAt: DateTime!
    }
    
    enum MemberStatus { ACTIVE ON_LEAVE TERMINATED }
    enum LeaveKind { ANNUAL SICK MATERNITY PATERNITY SABBATICAL UNPAID BEREAVEMENT PUBLIC_HOLIDAY }
    enum LeaveStatus { DRAFT SUBMITTED APPROVED REJECTED CONSUMED CANCELLED }
    
    type Query {
     me: Member!
     member(id: ID!): Member
     membersByManager(managerId: ID!): [Member!]!
     leaveRequests(memberId: ID, status: LeaveStatus, since: DateTime): [LeaveRequest!]!
     @requiresScopes(scopes: [["hr.leave_read"]])
     orgChart(rootId: ID): OrgChartNode!
    }
    
    type Mutation {
     requestLeave(input: LeaveRequestInput!): LeaveRequest!
     approveLeave(id: ID!, reason: String): LeaveRequest!
     @requiresScopes(scopes: [["hr.leave_approve"]])
     rejectLeave(id: ID!, reason: String!): LeaveRequest!
     @requiresScopes(scopes: [["hr.leave_approve"]])
     cancelLeave(id: ID!): LeaveRequest!
    }

### REST admin surface (planned)

Method| Path| Purpose  
---|---|---  
POST| `/admin/members`| Create Member; kicks onboarding orchestrator.  
GET| `/admin/members/{id}`| Read Member (HR-scope).  
POST| `/admin/members/{id}/terminate`| Kick offboarding orchestrator. Requires CEO co-sign for ESOP Bad Leaver branch.  
POST| `/admin/contracts`| Issue or renew a contract. Generates PDF; routes via DOC for e-sign.  
POST| `/admin/contracts/{id}/amend`| Append amendment row. Original contract never mutated.  
GET| `/admin/contracts/{id}/pdf`| Pre-signed S3 URL (60-second TTL). Audit row written.  
POST| `/admin/cccd`| Upload CCCD photos (multipart). KMS-wrapped at rest; sev-1 audit on read.  
GET| `/admin/cccd/{member_id}`| Pre-signed S3 URL (30-second TTL). Sev-1 audit row.  
POST| `/admin/onboarding/{member_id}/advance`| Mark an onboarding step done.  
POST| `/admin/sabbatical/eligibility-tick`| Run the quarterly sabbatical-eligibility job.  
POST| `/admin/review/cycles`| Open a performance-review cycle; LEARN owns the workflow.  
POST| `/admin/dsar/{member_id}/export`| PDPL Art. 14 DSAR — bundles profile + leave + contracts + non-comp documents.  
  
### MCP tool catalogue (CUO/CHRO-skill)

Tool name| Inputs| Outputs| Annotations  
---|---|---|---  
`cyberos.hr.list_members`| filter?| Member| readonly · scope=hr.read  
`cyberos.hr.read_profile`| member_id| Profile| readonly · scope=hr.profile_read  
`cyberos.hr.read_leave_balance`| member_id| LeaveBalance| readonly · scope=hr.leave_read  
`cyberos.hr.draft_leave_request`| member_id, kind, dates| LeaveRequest (draft)| readwrite (own only)  
`cyberos.hr.org_chart`| root_id?| OrgChartNode| readonly · scope=hr.read  
`cyberos.hr.onboarding_status`| member_id| OnboardingTask| readonly  
`cyberos.hr.draft_offer_letter`| candidate_id, role, level, start_date| DraftOffer (markdown)| readonly (narrative only) · destructive=false  
`cyberos.hr.summarise_review_outcome`| review_cycle_id| summary text| readonly · individual scores never exposed  
`cyberos.hr.dsar_export`| member_id| signed-URL| destructive=false · scope=hr.dsar · human-confirm  
  
6

## Key flows

### Flow 1 — Onboarding orchestrator (new Member join)

sequenceDiagram autonumber participant HR as HR/Ops Lead (SPA) participant H as HR onboarding.rs participant A as 🔐 AUTH participant T as ⏱ TIME participant CH as 💬 CHAT participant L as 📈 LEARN participant N as 📡 NATS participant B as 🧠 memory HR->>H: POST /admin/members  
{name, role, level, start_date, manager_id} H->>H: create member row H->>H: initialise onboarding_task rows (8 default tasks) H->>B: audit "hr.member.joined" H->>N: publish hr.member.joined par fan-out to downstreams H->>A: provision account (email + temp passkey enrol link) A-->>H: ack and H->>T: enrol in timesheet T-->>H: ack and H->>CH: provision default workspaces (#general · team) CH-->>H: ack and H->>L: seed learning profile (role-based reading list) L-->>H: ack end H-->>HR: 201 Created · onboarding_id Note over H,B: each downstream ack writes its own audit row;  
partial failure is retried with idempotency key. 

Onboarding is the multi-module fan-out. Each downstream call carries an idempotency key so retries are safe; if AUTH succeeds but CHAT times out, the orchestrator retries CHAT without re-provisioning AUTH.

### Flow 2 — Leave request + approval

sequenceDiagram autonumber participant M as Member (SPA) participant H as HR leave.rs participant BAL as leave_balance row participant MGR as Manager (notif) participant T as ⏱ TIME (calendar) participant B as 🧠 memory M->>H: requestLeave(kind=annual, start, end) H->>BAL: check sufficient balance alt enough balance H->>H: create leave_request status="submitted" H->>B: audit "hr.leave.requested" H->>MGR: notify (CHAT + email) MGR->>H: approveLeave(id) H->>H: leave_request status="approved" H->>BAL: decrement annual_remaining H->>T: emit calendar event member-on-leave H->>B: audit "hr.leave.approved" H-->>M: notify approved else insufficient balance H-->>M: 422 "balance insufficient" H->>B: audit "hr.leave.rejected" reason="balance" end 

### Flow 3 — Contract renewal + e-sign via DOC

sequenceDiagram autonumber participant HR as HR/Ops Lead participant H as HR contract.rs participant TPL as PDF template engine participant S3 as AWS S3 (object-lock 10y) participant K as AWS KMS participant D as 📄 DOC e-sign participant M as Member participant B as 🧠 memory HR->>H: POST /admin/contracts  
{member_id, kind, effective_from, terms} H->>TPL: render PDF (Vietnamese + English) TPL-->>H: PDF bytes H->>K: wrap with contract KMS key K-->>H: ciphertext H->>S3: PUT contracts/<member_id>/<id>.pdf (object-lock 10y) S3-->>H: s3_uri + etag H->>D: send for e-sign (WebAuthn binding) D->>M: present contract + WebAuthn challenge M-->>D: signed assertion D-->>H: signed PDF + audit event H->>S3: archive signed version H->>H: contract.status = "active" H->>B: audit "hr.contract.signed" with PDF SHA-256 Note over H,B: previous contract row is superseded;  
old row retained for 10-year statutory minimum. 

### Flow 4 — Performance review cycle (HR initiates · LEARN executes)

sequenceDiagram autonumber participant HR as HR/Ops Lead participant H as HR review_hook.rs participant L as 📈 LEARN participant J as Peer judges (3-5) participant CHR as CHRO (or CEO) participant B as 🧠 memory HR->>H: open review cycle for Q2-2026 H->>L: POST /learn/review-cycles {period, member_ids} L->>L: open per-Member peer review (Hội đồng Chuyên môn) L->>J: invite (5 judges per case) J-->>L: submit scores (individual rows; not ingested into memory) L->>L: compute aggregate (median; no per-judge exposure) L->>CHR: present aggregate + recommendation CHR-->>L: confirm outcome L-->>H: review_outcome {summary, recommendation} H->>H: store review_outcome row (no per-judge data) H->>B: audit "hr.review.closed" reason=<recommendation> Note over L,H: individual peer scores NEVER cross the HR boundary;  
(FR pending) enforced at LEARN export gate. 

### Flow 5 — Termination + offboarding orchestrator

sequenceDiagram autonumber participant CEO as CEO (co-sign) participant HR as HR/Ops Lead participant OFF as HR offboarding.rs participant A as 🔐 AUTH participant R as 💎 REW (final pay) participant E as 📊 ESOP (GL/BL) participant ASSET as Asset return checklist participant B as 🧠 memory HR->>OFF: POST /admin/members/<id>/terminate {kind=resignation|dismissal} OFF->>CEO: request co-sign (mandatory for Bad Leaver) CEO-->>OFF: approve par revocations OFF->>A: revoke all sessions + API keys ((FR pending) 5s SLO) OFF->>R: compute final pay (BHXH + PIT + accrued leave cash-out) OFF->>E: branch GL vs BL; vest pause; put-rights freeze if BL OFF->>ASSET: open asset-return tasks (laptop, badge, …) end OFF->>OFF: member.status = "terminated", end_date = today OFF->>B: audit "hr.member.terminated" reason={kind} Note over OFF,B: contract NOT deleted — superseded with effective_to=today;  
10-year statutory retention applies. 

Bad Leaver is a CFO + CEO co-sign decision ((FR pending)). The offboarding orchestrator never decides GL vs BL unilaterally; it surfaces a recommendation and waits for the human gate.

7

## Member lifecycle

A Member traverses five states from offer to terminated, with three special branches (probation pass/fail, leave-of-absence, sabbatical). Every transition writes a memory audit row and emits a NATS event. 

stateDiagram-v2 [*] --> Offer: offer letter generated Offer --> Probation: contract signed, start_date reached Offer --> Withdrawn: candidate declines Probation --> Active: probation_pass after 60 days (default) Probation --> Terminated: probation_fail Active --> OnLeave: leave_request approved 5d or longer OnLeave --> Active: leave end_date passed Active --> Sabbatical: sabbatical_tick reaches 5; user opts in Sabbatical --> Active: sabbatical end_date passed Active --> Terminated: resignation OR dismissal Terminated --> [*] Withdrawn --> [*] 

### Service-period entitlement table

Service years| Annual leave (days/yr)| BHXH sick allowance (days)| Sabbatical eligibility  
---|---|---|---  
< 5| 12| 30 (per Decree 152)| Not yet eligible  
5–9| 13 (+1 per 5 years)| 40| 1st sabbatical at 5y  
10–14| 14| 40| 2nd at 10y  
15–19| 15| 60| 3rd at 15y  
20+| 16| 60| 4th at 20y · etc.  
  
Vietnamese Labour Code Art. 113 + Decree 152/2020 Art. 26. CyberSkill-specific sabbatical from Total Rewards Appendix.

8

## Functional Requirements

The CyberOS FR catalogue is being rebuilt one feature at a time via the open [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) Agent Skill.

Previous FR enumerations were archived 2026-05-14 and are no longer reflected on this page. Specific FRs land here as they are re-authored.

9

## Non-Functional Requirements

security + §11.2.5 usability NFRs that bind on HR. Cross-referenced at [nfr-catalog.html#hr](<../../reference/nfr-catalog.html#hr>).

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`N(FR pending)`| CCCD access without sev-1 audit row| = 0 occurrences| chaos test: read CCCD; assert audit row present + classification=restricted  
`N(FR pending)`| Comp data appearing in HR table| = 0 — CI gate| schema diff bot; sqlx migration grep for blocklist columns  
`N(FR pending)`| KMS-key separation (CCCD vs contracts vs Profile)| 3 distinct keys| KMS policy inspection; cross-key access blocked  
`N(FR pending)`| Leave-request submission (mobile)| ≤ 3 taps from home| mobile UX walkthrough · usability test  
`N(FR pending)`| Onboarding checklist completion time (median Member)| ≤ 5 working days| onboarding_task timestamps  
`N(FR pending)`| Member directory query p95| ≤ 80 ms| k6 load test  
`N(FR pending)`| Org-chart render p95 (≤ 100 Members)| ≤ 150 ms| bench/org_chart.rs  
`N(FR pending)`| HR availability (28-day)| ≥ 99.5%| SLO monitor  
`N(FR pending)`| Contract durability (10-year retention)| 0 lost objects| S3 object-lock + quarterly inventory audit  
`N(FR pending)`| Onboarding orchestrator idempotency| 100% (property test)| proptest: duplicate-fire + retry → same final state  
`N(FR pending)`| DSAR fulfilment time| ≤ 30 days (PDPL)| DSAR queue dashboard  
  
10

## Dependencies

HR is the Member-directory primitive that every comp/learn/equity module reads from. It depends on AUTH for identity, memory for audit, OBS for telemetry, and (P4) DOC for WebAuthn e-sign. 

graph LR subgraph upstream ["HR depends on"] AUTH["🔐 AUTH  
identity · RBAC"] memory["🧠 memory  
audit chain"] OBS["👁 OBS  
traces + metrics"] KMS["🔑 AWS KMS  
per-column key"] S3["🗂 AWS S3  
contract PDFs · CCCD"] DOC["📄 DOC  
WebAuthn e-sign · P4"] end HR["👥 HR"] subgraph downstream ["HR is consumed by"] REW["💎 REW  
roster + final pay"] LEARN["📈 LEARN  
review cycles + skill tree"] ESOP["📊 ESOP  
grants + GL/BL"] TIME["⏱ TIME  
timesheet enrolment"] CHAT["💬 CHAT  
workspaces"] INV["🧾 INV  
contractor lookup"] OKR["🎯 OKR  
per-Member objectives"] CRM["🏢 CRM  
contact owner mapping"] end AUTH --> HR memory --> HR OBS --> HR KMS --> HR S3 --> HR DOC --> HR HR --> REW HR --> LEARN HR --> ESOP HR --> TIME HR --> CHAT HR --> INV HR --> OKR HR --> CRM classDef planned fill:#fde7b3,stroke:#9c750a classDef shipped fill:#f5ede6,stroke:#45210e class HR planned class AUTH,REW,LEARN,ESOP,TIME,CHAT,INV,OKR,CRM,DOC,OBS planned class memory,KMS,S3 shipped 

11

## Compliance scope

HR is the Vietnamese-labour-law front door. Every contract, every leave row, every CCCD record has to defend against a Decree 13/2023 inspector, a PDPL DSAR request, and a 10-year SI/PIT statutory audit.

Regulation / standard| Article / clause| HR feature that satisfies it  
---|---|---  
Vietnam Labour Code (2019)| Art. 113 — Annual leave| Leave accrual schedule keyed to service years; quarterly accrual job.  
Vietnam Labour Code| Art. 139 — Maternity leave| Leave kind `maternity` with default 6 months + twins +30/child.  
Decree 145/2020/NĐ-CP| Art. 60 — Overtime cap| (FR pending) enforces ≤ 200h/year overtime per Member; check at TIME submission.  
Decree 152/2020/NĐ-CP| Art. 26 — Sick leave allowance| BHXH sick allowance days tiered by service period.  
Decree 152/2020/NĐ-CP| Art. 5 — SI contribution rates| BHXH 8%/17.5%, BHYT 1.5%/3%, BHTN 1%/1% as versioned parameters; comp owned by REW.  
Decree 13/2023/NĐ-CP| Art. 17 — Personal data processing log| Every Member read / write writes an HR audit row to memory.  
Decree 53/2022/NĐ-CP| Art. 26 — Data localisation| P3: VN tenants pin `data_residency = "vn-hanoi-1"`; HR Postgres replica in-country.  
Law 91/2025/QH15 (PDPL)| Art. 14 — DSAR| (FR pending) — `cyberos-hr dsar-export` bundles Member-scoped data within 30 days.  
Law 91/2025/QH15 (PDPL)| Art. 7 — Sensitive personal data| CCCD photo classified `restricted`; separate KMS key; sev-1 audit on read.  
GDPR (EU 2016/679)| Art. 32 — Security of processing| Column-level KMS · S3 object-lock · row-level security · DPO-scoped access.  
ISO/IEC 27001:2022| A.5.13 — Labelling of information| Every Member field carries a classification tag.  
ISO/IEC 27001:2022| A.8.10 — Information deletion| Termination workflow respects 10-year retention; supersession not deletion.  
SOC 2 Type II| CC6.1 — Logical access| RBAC predicate at every HR API; CCCD requires `hr.cccd_read` scope.  
VN Decree 38/2020/NĐ-CP| Art. 6 — Foreign labour| Contract kind + nationality field; work-permit tracker (P2).  
  
12

## Risk entries

HR-specific risks tracked in the [risk register](<../../reference/risk-register.html#hr>). The highest-impact risk is comp leakage into HR — a structural failure that would force a schema rebuild.

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-HR-001`| Compensation field leaks into HR table| Low| High| HR/Ops Lead| CI schema-diff bot; sqlx migration grep for {salary, bonus, comp, p1, p2, p3}; DEC-036 denylist.  
`R-HR-002`| CCCD photo accidentally embedded in API response| Low| Catastrophic| CSO| CCCD never resolvable through GraphQL; admin REST returns pre-signed S3 URL only; integration test asserts response body never contains image bytes.  
`R-HR-003`| Onboarding orchestrator partial failure leaves Member half-provisioned| Medium| Medium| CTO| Idempotency keys; retry-until-success queue; alert if a task pending > 24h.  
`R-HR-004`| Leave-balance race condition (two simultaneous approvals)| Medium| Low| CTO| Optimistic locking on `leave_balance.version`; conflict triggers re-read + retry.  
`R-HR-005`| Contract deleted instead of superseded| Low| High| HR/Ops Lead| DB role lacks DELETE on contract table; admin REST only exposes amend + supersede.  
`R-HR-006`| Decree 145/2020 overtime cap bypassed| Low| High| HR/Ops Lead| (FR pending) cap enforced at TIME submission; HR exposes cap-status MCP read for CUO.  
`R-HR-007`| Bad-Leaver branch chosen without human gate| Low| Catastrophic| CEO| Offboarding orchestrator requires CFO + CEO co-sign; cannot proceed without both signatures recorded.  
`R-HR-008`| Per-judge review scores leak to HR via summary field| Low| Medium| HR/Ops Lead| (FR pending) enforced at LEARN export gate; HR review_outcome schema rejects fields beyond {summary, recommendation}.  
`R-HR-009`| Sabbatical eligibility tick miscounted across leave gaps| Medium| Low| HR/Ops Lead| Continuous-service definition: gap < 30 days counted; longer breaks reset the tick chain. Property test on the accrual.  
`R-HR-010`| 10-year retention violated by S3 lifecycle policy bug| Low| High| CTO| S3 object-lock governance mode; lifecycle rule rejected by IAM if reduces retention; quarterly inventory audit.  
`R-HR-011`| HR aggregated performance signal used as sole basis for comp decision| Medium| High| CHRO| HR signals are _inputs_ ; REW comp recommendation requires Member manager review + CHRO + CFO sign-off; never auto-applied.  
`R-HR-012`| Cross-tenant Member-id collision (rare but catastrophic)| Low| Critical| CSO| Member-id is UUIDv7 + tenant prefix; CI property-test asserts no collision across 1M-tenant simulation; collision = release blocked.  
`R-HR-013`| Onboarding playbook fires before AUTH provisioning ready| Medium| Medium| CTO| Playbook saga waits for AUTH ready event; idempotent retry; alert if AUTH provisioning > 60s.  
`R-HR-014`| Vietnamese labour-law amendment (Decree 145 sub-decree) changes leave accrual mid-year| Low| High| CLO| Legal monitor on labour-law amendments; accrual rules version-pinned; mid-year change creates pro-rated transition period.  
`R-HR-015`| Sabbatical eligibility tick miscounts due to maternity-leave gap classification| Medium| Low| CHRO| Property test asserts statutory leaves (maternity/paternity/medical) count toward tick; vacation gaps do not; quarterly accrual audit.  
  
13

## KPIs

HR health rolls up into 9 KPIs covering lifecycle throughput, compliance posture, and orchestrator correctness.

KPI| Formula| Source| Target  
---|---|---|---  
**Onboarding completion (median days)**|  median(`onboarding_task.completed_at - member.start_date`)| HR DB| ≤ 5 working days  
**Leave request approval p95**|  p95(`approved_at - submitted_at`)| HR DB| ≤ 2 business days  
**Org-chart drift**| `members_without_manager_id / total_members`| HR DB| = 0%  
**CCCD access audit coverage**| `cccd_reads / audit_rows_with_class=restricted`| memory| = 100%  
**Contract renewal lead time**|  median(days between expiry warning and renewal)| HR DB| ≥ 30 days  
**Sabbatical-tick correctness**|  property-test pass rate| CI| 100%  
**DSAR fulfilment p95**|  p95(`exported_at - requested_at`)| HR DB| ≤ 14 days (well under 30d PDPL cap)  
**Onboarding orchestrator partial-failure rate**| (orchestrations with retry) / total| OBS| ≤ 1%  
**Comp-field-in-HR incidents**|  CI gate failures| CI| = 0  
**Signal-only comp decision rate**|  comp recs where REW/CHRO/CFO sign-off recorded / total recs| memory audit| = 1.0 (hard floor)  
**Onboarding playbook saga p95**|  histogram (member.start_date → all tasks fired)| OBS| ≤ 5 min  
**Labour-law version stamp coverage**|  contracts/leave records with cap_version stamped / total| HR DB| = 1.0  
**HR-to-REW handoff p95**|  histogram (member.active → REW comp record created)| OBS| ≤ 1 min  
**Statutory-leave classification accuracy**|  maternity/paternity/medical correctly counted toward sabbatical / total| quarterly accrual audit| ≥ 0.99  
  
14

## RACI matrix

HR is owned by the HR/Ops Lead. CEO is accountable for policy; DPO is responsible for PDPL DSAR; CFO co-signs Bad-Leaver branches.

Activity| CEO| HR/Ops| CFO| CTO| CSO| DPO  
---|---|---|---|---|---|---  
Member onboarding| I| A/R| I| C| I| I  
Leave approvals| I| R| I| I| I| I  
Contract issuance| C| A/R| C| I| I| I  
Termination (Bad Leaver)| A| R| R| I| I| I  
Sabbatical tick (annual)| I| A/R| I| I| I| I  
CCCD ingestion| I| R| I| I| C| A  
DSAR fulfilment (HR scope)| I| R| I| I| C| A  
Org-chart maintenance| C| A/R| I| I| I| I  
10-year retention audit| I| R| C| A| C| C  
  
**R** Responsible · **A** Accountable · **C** Consulted · **I** Informed.

15

## Planned CLI surface

A single admin CLI `cyberos-hr` for HR/Ops Lead. Every destructive command writes an audit row before exit.

### 1\. Add a Member
    
    
    $ cyberos-hr member add \
     --email mai@cyberskill.com \
     --display "Mai Nguyen" \
     --role engineer --level L2 \
     --start-date 2026-06-01 \
     --manager stephen@cyberskill.com
    
    [member created]
     id: 01HZJ8R4M2K7QXP3F9D8YN7B2T
     email: mai@cyberskill.com
     start: 2026-06-01
    [onboarding] checklist created: 8 tasks
    [fanout] auth.account.create → ack
    [fanout] chat.workspace.provision → ack
    [fanout] time.timesheet.enrol → ack
    [fanout] learn.profile.seed → ack
    [audit] memory seq=14843 chain=a1c4…b8e2

### 2\. Submit a leave request (Member-self)
    
    
    $ cyberos-hr leave request \
     --kind annual \
     --start 2026-07-15 --end 2026-07-19 \
     --reason "family trip"
    
    [leave request submitted]
     id: 01HZJ8…JTC
     days: 5
     status: submitted
     approver: stephen@cyberskill.com (manager)
    [audit] memory seq=14844 chain=b2d5…c9f3

### 3\. Issue a contract
    
    
    $ cyberos-hr contract issue \
     --member mai@cyberskill.com \
     --kind indefinite \
     --effective-from 2026-06-01 \
     --template indefinite-vn-2026
    
    [contract drafted] indefinite-vn-2026 → /tmp/contract-mai.pdf
    [pdf] SHA-256: 9f3e…2a1b
    [s3] contracts//.pdf (KMS contract-key)
    [doc] sent for WebAuthn e-sign → notification dispatched
    [status] pending_signature
    [audit] memory seq=14851 chain=d4e7…f1a9

### 4\. Render the org chart
    
    
    $ cyberos-hr org-chart --format mermaid --root stephen@cyberskill.com
    
    graph TD
     stephen[Stephen Cheng · CEO]
     stephen --> mai[Mai Nguyen · L2 Engineer]
     stephen --> hoa[Hoa Tran · L3 Engineer]
     hoa --> linh[Linh Pham · L1 Engineer]
    
    [org-chart] 4 members; rendered in 11 ms

### 5\. Read a leave balance
    
    
    $ cyberos-hr leave balance --member mai@cyberskill.com
    
    [balance for mai@cyberskill.com]
     annual_remaining: 11.0 / 12.0 days
     sick_remaining: 30.0 / 30.0 days
     sabbatical_accrued: 0.0 days (eligible at 5y)
     service_years: 0 (started 2026-06-01)
     last_accrual: 2026-06-30T00:00:00Z

### 6\. Terminate a Member (Bad Leaver — requires co-sign)
    
    
    $ cyberos-hr member terminate \
     --member alex@cyberskill.com \
     --kind dismissal \
     --reason "Code of Conduct violation" \
     --cosign-ceo --cosign-cfo
    
    [terminate] dismissal · 2026-05-14
    [cosign] ceo: stephen@cyberskill.com (WebAuthn)
    [cosign] cfo: hoa@cyberskill.com (WebAuthn)
    [auth] sessions revoked (5s SLO) → done
    [rew] final pay computed; cash-out 11.0 days annual leave
    [esop] branch: BAD_LEAVER → vested SP retained at 60% discount
    [asset] opened: laptop, badge, github SSH (3 tasks)
    [contract] superseded; effective_to = 2026-05-14
    [retention] 10-year hold remains
    [audit] memory seq=14862 chain=e7f1…a8b4

### 7\. DSAR export
    
    
    $ cyberos-hr dsar-export --member mai@cyberskill.com --output dsar.zip
    
    [dsar] member: mai@cyberskill.com
    [dsar] profile: 1 row
    [dsar] contracts: 1 (active)
    [dsar] leave: 4 rows (last P0 → P3 horizon)
    [dsar] documents: 3 (offer, NDA, equipment list)
    [dsar] cccd: 1 (sev-1 audit added)
    [dsar] comp: — (REW DSAR separate)
    [dsar] written: dsar.zip (1.8 MB, KMS-encrypted)
    [audit] memory seq=14871 chain=f8a2…c4d6

16

## Phase status & estimates

Status

Planned

P1 design phase

Est. LoC (Rust)

~5,200

services/hr + sqlx migrations

Planned tests

80+

unit · integration · property (accrual)

External libs

~10

axum · sqlx · aws-sdk-s3 · aws-sdk-kms · async-graphql

CLI subcommands

~20 planned

`cyberos-hr` entrypoint

P1 budget

~$25/mo

RDS schema + Fargate share

Capability| Status  
---|---  
Member directory (CRUD + org chart)| planned · P1  
Leave request + accrual (8 leave types)| planned · P1  
Contract storage + versioning| planned · P1  
Onboarding orchestrator (8-step fan-out)| planned · P1  
Offboarding orchestrator + asset return| planned · P1  
CCCD storage + sev-1 audit| planned · P1  
Sabbatical eligibility tick| planned · P1  
BHXH/BHYT/BHTN profile fields| planned · P1  
DSAR export (PDPL Art. 14)| planned · P1  
Performance-review cycle hook → LEARN| planned · P2  
WebAuthn e-sign via DOC| planned · P4  
Decree 145/2020 overtime cap (TIME boundary)| planned · P2  
Singapore HoldCo branch (SG residency)| planned · P3  
Work-permit tracker (foreign labour)| planned · P2  
Multi-tenant data-residency (vn-hanoi-1)| planned · P3  
  
17

## References

  * **FR catalogue** — HR module FRs ((FR pending) through (FR pending)).
  * **Architecture spec** — HR architecture posture + lifecycle orchestrators.
  * **NFR catalogue** — Security NFRs binding on HR (CCCD classification, KMS separation).
  * **FR mapping** — Formal (FR pending) through (FR pending) with verification methods.
  * **Total Rewards & Career Path Appendix** — Sabbatical eligibility (every 5 continuous years).
  * **Vietnam Labour Code (2019)** — Art. 112 (public holidays), 113 (annual leave), 115 (other leave), 139 (maternity).
  * **Decree 145/2020/NĐ-CP** — Implementing the Labour Code; Art. 60 overtime cap.
  * **Decree 152/2020/NĐ-CP** — Social-insurance contribution rates and sick-leave allowances.
  * **Decree 13/2023/NĐ-CP** — Personal data protection regulations; Art. 17 processing log.
  * **Bigger picture (§0 above):** 3 strategic roles + Member-id spine Mermaid + 10-row auto-vs-human matrix.
  * **Cross-module page links:** [auth.html](<../auth/index.html>) · [rew.html](<../rew/index.html>) · [esop.html](<../esop/index.html>) · [learn.html](<../learn/index.html>) · [time.html](<../time/index.html>) · [proj.html](<../proj/index.html>) · [memory.html](<../memory/index.html>)
  * **memory auto-sync vision:** [MEMORY_AUTOSYNC_DESIGN.md §5](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) — HR lifecycle events become memory audit rows; CCCD photo never enters memory.
  * **Build-readiness audit:** `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`) — HR at P1 · mid (P1, alongside PROJ).
  * **FR authoring discipline:** [modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>).
  * **Decree 53/2022/NĐ-CP** — Cybersecurity Law implementation; data localisation.
  * **Law 91/2025/QH15 (PDPL)** — Personal Data Protection Law; Art. 7 sensitive data, Art. 14 DSAR.
  * **Decree 38/2020/NĐ-CP** — Foreign labour management.
  * **ISO/IEC 27001:2022** — A.5.13, A.8.10 mapped to HR record management.
  * **Architecture context:** [infrastructure.html#hr](<../../architecture/infrastructure.html#hr>).



★

## Personas & skill bundles that touch HR

HR is the system-of-record for the workforce. Of the 47 CUO personas, the people-cluster reads from and writes to HR continuously.

Persona affinities (8 of 47)

  * [chief-human-resources-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-human-resources-officer/workflows>) · 7 workflows (workforce-plan, talent-review, comp-cycle, etc.)
  * [chief-people-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-people-officer/workflows>) · EVP + people-strategy + people-review (CHRO synonym at some companies)
  * [chief-diversity-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-diversity-officer/workflows>) · DEI program + ERG charters + progress review
  * [chief-learning-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-learning-officer/workflows>) · learning-strategy + leadership-development
  * [chief-happiness-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-happiness-officer/workflows>) · eNPS deep-dive + wellbeing intervention
  * [chief-remote-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-remote-officer/workflows>) · distributed-work program + remote effectiveness
  * [chief-executive-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-executive-officer/workflows>) · c-suite-hire-decision
  * [chief-financial-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-financial-officer/workflows>) · headcount & payroll feed for forecast / budget



Skill-bundle reads & writes

  * [workforce-plan-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/workforce-plan-author>) \+ audit · canonical Q-plan output
  * [onboarding-pack-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/onboarding-pack-author>) \+ audit · per-new-hire packet
  * [hire-decision-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/hire-decision-author>) \+ audit · interview-loop adjudication
  * [erg-charter-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/erg-charter-author>) \+ audit · employee-resource-group authoring
  * [vietnam-vneid-integration](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/vietnam-vneid-integration>) · VN onboarding identity assertion



[← All modules](<../index.html#catalog>) [Next module: REW (Total Rewards) →](<../rew/index.html>)
