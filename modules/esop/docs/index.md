---
title: ESOP — Phantom Stock vesting · Good/Bad Leaver branch · Singapore HoldCo flip · CyberOS
source: website/docs/modules/esop/index.html
migrated: FR-DOCS-002
---

ESOP is the **synthetic equity plane** — a tracked-on-books economic interest in the company that vests over time, can be valued, and can be cashed out under defined windows. No share certificates are ever issued; no Vietnamese securities-regulation filing applies (Decree 38/2020 corporate governance treatment instead, as deferred compensation). The core entities are **Grants** (each tied to a Member, type, schedule), **VestingEvents** (the monthly vest after the phased cliff), **Valuations** (annual, signed by CFO + Board), **PutOptionExercises** (Year-3+ cash-out windows), and **PoolBalance** (the pool replenishment from % of annual profit, with GL/BL branches for top-performers vs broad distribution). 

Invariant — append-only grants & valuations

Every grant row, vesting event, and valuation is append-only. Corrections happen via supersession with effective_to + reason. **UPDATE on a published grant or valuation = sev-0.** The 10-year statutory retention is enforced at S3 object-lock.

Invariant — dual sign-off (every grant + every valuation)

A new grant requires **CEO + CFO + Board-designate** co-sign (three signatures, WebAuthn-bound). An annual valuation requires **CFO + Board chair + external auditor** attestation. Single-signer events are rejected at the gateway.

Status

Planned

P2 · design phase

Est. LoC

~5,400

Rust (axum) + cap-table simulator

Planned tests

110+

incl. vesting determinism + valuation replay

Default vesting

4 years

12-mo cliff + monthly

Put windows

Year 3+

capped per Member per year

Pool replenishment

% of profit

GL / BL branch split

memory ingestion

= 0

DEC-036 · ESOP value-private

HoldCo flip

@ ARR $1.5M

VN phantom → SG real share

0

## The bigger picture — three strategic roles

ESOP is the retention moat. The "Phantom Stock" design routes around Vietnamese securities regulation while preserving the economic outcome of real ESOP. ESOP is heavily entangled with REW (BP fund + comp), HR (Good vs Bad Leaver branch), and AUTH (cap-table-as-subject). Like REW, ESOP value is structurally excluded from memory per DEC-036. 

Role 1 · Grant lifecycle

Issue · vest · cliff · cancel · put

SP (Stock Phantom) grant types: founder · early · standard · ad-hoc. Default vesting: 4 years with phased cliff + monthly thereafter. Put options open from Year 3, capped per Member per year. M&A acceleration clauses standard. Cancellation only by Board sign-off; never automatic. 

Role 2 · Good Leaver vs Bad Leaver

Branch chosen at HR offboarding

HR offboarding initiates the branch: Good Leaver (resign with notice, mutual exit, retirement) → vested SP retained at full valuation, put rights preserved. Bad Leaver (fraud, gross misconduct, breach of post-employment covenant) → vested SP retained at appendix discount (default 50%), put rights frozen. CFO + CEO co-sign required; never auto-classified. 

Role 3 · Liquidity event simulator

Annual valuation · put exec · HoldCo flip

Annual SP unit value: CFO computes base valuation + Board signs Industry Multiplier → versioned valuation row. Put option exec: Member sells N% of vested SP back to company at most recent valuation. HoldCo flip trigger: when ARR ≥ $1.5M, CEO can designate Singapore HoldCo flip — phantom shares convert to real SG shares; pre-tax conversion documented at appendix. 

### ESOP in the cap-table spine

flowchart TB HR["👥 HR  
Member lifecycle"] ESOP["📊 ESOP  
SP grants · vesting · valuation"] REW["💰 REW  
BP fund context"] BOARD["🏛 Board  
annual valuation sign-off"] CFO["📈 CFO  
base valuation input"] LEAVER["🚪 Good/Bad Leaver branch  
CFO + CEO co-sign"] HOLDCO["🇸🇬 Singapore HoldCo  
flip at ARR ≥ $1.5M"] memory["🚫 memory  
structurally excluded · DEC-036"] HR -- "active → vest accrual" --> ESOP HR -- "terminating" --> LEAVER LEAVER --> ESOP CFO --> ESOP BOARD --> ESOP REW <\-- "BP fund retention context" --> ESOP ESOP --> HOLDCO ESOP -. "structural exclusion" .x memory classDef hub fill:#bfdbfe,stroke:#1e3a8a,stroke-width:3px,color:#172554 classDef mod fill:#e0e7ff,stroke:#3730a3 classDef forbidden fill:#fee2e2,stroke:#b91c1c,stroke-width:3px,color:#7f1d1d class ESOP hub class HR,REW,BOARD,CFO,LEAVER,HOLDCO mod class memory forbidden 

### Auto vs human-in-loop operations matrix

Operation| How it happens| Why this split  
---|---|---  
SP grant issuance| **Manual** CEO + CFO + Board sign-off| Cap-table change; tri-sign required.  
Monthly vesting accrual| **Auto** deterministic from vesting schedule| Cliff + monthly formula; CI replay.  
Annual valuation| **Manual** CFO base + Board multiplier sign-off| Major financial event; versioned; immutable after publish.  
Put option exercise| **Manual** Member request → CFO approve → wire| Money movement; CFO verifies eligibility (Year 3+, cap not exceeded).  
Good vs Bad Leaver classification| **Manual** CFO + CEO co-sign| Consequential; never algorithmic.  
Cancellation (forfeiture)| **Manual** Board sign-off| Cancelling unvested portion happens on Bad Leaver or termination.  
M&A acceleration trigger| **Manual** Board declaration| Acceleration clauses fire on Board-declared change of control.  
HoldCo flip designation| **Manual** CEO + Board sign-off| Singapore conversion is a one-way operation; deliberate.  
memory ingestion of ESOP value| **BLOCKED**|  DEC-036; CI gate verifies absence.  
Member ESOP dashboard view| **Auto** personal view only| Member sees own grants; cross-Member view requires CFO sign-off audit row.  
  
1

## Why ESOP exists

The economic insight: people stay if they have a meaningful, predictable financial upside in the company's success. The legal insight: issuing real shares in a Vietnamese JSC triggers securities-issuance regulation, accounting complexity, and a minority-shareholder governance overhead that's expensive at 10 members and impossible at 100. The synthesis: **Phantom Stock**. The Member's outcome is identical (synthetic units worth X VND/SGD that grow with the company); the legal posture is "deferred compensation" governed by employment contracts, not securities law. ESOP encodes this carefully — every grant is an audited row, every valuation is dual-signed, every put exercise wires cash on a published schedule, every exit branches cleanly to Good Leaver or Bad Leaver outcomes. When the company is ready for a real cap table (typically post-Series A or at ARR $1.5M), the CEO designates a **HoldCo flip** — phantom units convert to real Singapore HoldCo shares with the same economic terms. 

👻

Phantom — not real shares

No Vietnamese securities-issuance regulation. Treated as deferred compensation. Members get the economic upside without the governance overhead.

📜

Annual signed valuation

CFO + Board + external auditor sign every annual valuation. Industry Multiplier × base value = SP unit value. Versioned, immutable.

💰

Year-3 put options

From Year 3, Members can sell back N% of vested SP per year at the most recent valuation. Real cash out without M&A exit.

The retention bet: a Phantom Stock plan with credible annual valuation, real put-option windows, and a clean HoldCo-flip path is functionally a real ESOP from the Member's standpoint while being radically simpler from the company's standpoint. Done well, it removes the "I should leave to get equity elsewhere" pressure that destroys early-stage teams. 

2

## What it does — 5W1H2C5M

Every cell traces back to and.

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is ESOP?| A Phantom Stock ledger — grants (founding, milestone, retention), vesting schedules (4-year default), annual valuations (CFO + Board signed), put options (Year 3+), Good/Bad Leaver branches, pool balance + replenishment, optional HoldCo flip to real Singapore shares.  
**5W · Who**|  Who is touched?| **Grantees:** Members with at least one grant. **Approvers:** CEO + CFO + Board chair for every grant; CFO + Board + auditor for every valuation. **Read-only:** Members see own vesting + put schedule; never aggregate cap table.  
**5W · When**|  When does ESOP act?| (a) Hire — founding grant (if applicable). (b) Milestone — performance / promotion grant. (c) Monthly — vesting compute (idempotent). (d) Annual — valuation cycle (Q4). (e) July annually — put-option exercise window. (f) M&A — acceleration clause fires. (g) Termination — Good/Bad Leaver branch chosen.  
**5W · Where**|  Where does it run?| P2: single region (SG-1) backed by AWS RDS Postgres with ESOP-specific KMS key (separate from REW, HR, memory). PDFs at rest with retention lock = 10 years.  
**5W · Why**|  Why a separate module?| Because grant + valuation history is sensitive board-level data that must be append-only and dual-signed. Co-mingling with HR or REW would weaken the audit posture and risk leakage.  
**1H · How**|  How does it work?| Append-only grant + vesting_event + valuation rows in Postgres. Vesting compute is a deterministic function of (grant, today, valuation). Put-option exercise schedules a wire transfer via Wise (multi-currency) or VietQR (VND). HoldCo flip is a one-way migration — phantom row → real_share row with full audit trail.  
**2C · Cost**|  Cost budget?| P2: ~$25/month (small RDS schema + Fargate share + S3 retention). 50-tenant: ~$80/month. Per-grant cost negligible.  
**2C · Constraints**|  Constraints?| (a) Dual sign-off mandatory on grants + valuations ((FR pending)). (b) Append-only ((FR pending)). (c) Zero memory ingestion ((FR pending) + DEC-036). (d) 10-year retention. (e) M&A acceleration is Board-only. (f) HoldCo flip is CEO-only designation.  
**5M · Materials**|  Stack?| Rust 1.81 · axum · sqlx · PostgreSQL 16 · tectonic for grant + valuation PDFs · ring for SHA-256 · cyberskill-vn skills (vietnam-bank-transfer for put-option wires) · AWS S3 with retention lock · KMS.  
**5M · Methods**|  Method choices?| Append-only with supersession. Deterministic vesting compute (pure fn). Dual sign-off enforced at AUTH cosign predicate. Annual valuation is a versioned row analogous to REW parameter versioning.  
**5M · Machines**|  Deployment?| Fargate in SG-1 (P2). Multi-AZ Postgres RDS. S3 retention 10 years.  
**5M · Manpower**|  Who maintains?| CFO (R for valuation + grant) + CEO (A for grants + HoldCo flip) + 0.1 FTE eng for monthly vesting + put-option workflows.  
**5M · Measurement**|  How measured?| KPIs: grant audit trail integrity (= 100%), vesting determinism (replay pass), put-option SLO (exercise → wire ≤ 10 days), Good Leaver / Bad Leaver branch correctness.  
  
3

## Architecture

ESOP is one Rust service with three surfaces (GraphQL read-mostly for Member portals, REST admin for grant + valuation flows with multi-sign predicates, MCP narrator-only tools), a strict isolation policy from memory (zero numeric ingestion), and a HoldCo migration path that wires through DOC for Singapore shareholder paperwork. 

graph TB subgraph CLIENTS ["Clients"] CEO["CEO (grants · HoldCo flip)"] CFO["CFO (valuation co-sign)"] BOARD["Board chair (co-sign)"] AUDITOR["External auditor (attest)"] MEM["Member portal  
own grants + put schedule"] CUO["🤖 CUO/CFO-skill  
narrator only"] end subgraph EDGE ["Edge"] AR["Apollo Router  
JWT + RBAC + multi-sign predicate"] end subgraph ESOP ["ESOP service (Rust · axum)"] GQL["GraphQL subgraph  
read-mostly · self-scope"] REST["REST admin  
grant · vest · valuate · put"] MCP["MCP narrator"] GRANT["grant.rs  
3-sign issuance"] VEST["vesting.rs  
deterministic monthly compute"] VAL["valuation.rs  
annual cycle"] PUT["put_option.rs  
Y3+ exercise + wire"] LEAVE["leaver.rs  
GL/BL branch"] POOL["pool.rs  
balance + replenish"] MA["ma_acceleration.rs  
board-only trigger"] HOLDCO["holdco_flip.rs  
phantom → SG real share"] NARR["narrator.rs  
explain vesting + put"] end subgraph STORES ["Stores (isolated)"] PG[("PostgreSQL  
grant · vesting_event  
valuation · put_exercise  
ESOP-specific KMS key")] S3[("AWS S3  
grant PDFs · valuation reports  
10-year object-lock")] KMS[("AWS KMS  
esop-key  
distinct from REW + HR + memory")] end subgraph BRIDGES ["External bridges"] BANK["🛠 vietnam-bank-transfer  
put-option wire (VietQR)"] WISE["Wise (SGD wire)"] DOC["📄 DOC  
HoldCo paperwork (P4)"] HRMOD["👥 HR  
roster · termination"] end subgraph BOUNDARIES ["Compliance boundaries"] memory["🧠 memory  
opaque event refs ONLY  
(no ESOP numbers)"] OBS["👁 OBS  
timing only"] AUTH["🔐 AUTH"] end CEO --> AR CFO --> AR BOARD --> AR AUDITOR --> AR MEM --> AR CUO --> AR AR --> GQL AR --> REST AR --> MCP REST --> GRANT REST --> VEST REST --> VAL REST --> PUT REST --> LEAVE REST --> POOL REST --> MA REST --> HOLDCO PUT --> BANK PUT --> WISE HOLDCO --> DOC LEAVE --> HRMOD GRANT --> AUTH VAL --> AUTH REST --> PG GQL --> PG PG --> KMS S3 --> KMS GRANT --> S3 VAL --> S3 REST -.opaque ref.-> memory ESOP --> OBS classDef planned fill:#bfdbfe,stroke:#45210e classDef store fill:#f5f3ff,stroke:#7c3aed classDef boundary fill:#fee2e2,stroke:#dc2626 classDef extern fill:#f5ede6,stroke:#45210e class GQL,REST,MCP,GRANT,VEST,VAL,PUT,LEAVE,POOL,MA,HOLDCO,NARR planned class PG,S3,KMS store class memory,OBS boundary class BANK,WISE,DOC,HRMOD,AUTH extern 

### Internal components

Component| Path (planned)| Responsibility  
---|---|---  
`grant.rs`| services/esop/src/grant.rs| Append-only grant issuance. Requires CEO + CFO + Board-designate WebAuthn co-sign before INSERT. Types: founding, milestone, retention.  
`vesting.rs`| services/esop/src/vesting.rs| Pure-function vesting compute. Input: (grant, asof_date). Output: vested_sp count. phased cliff + monthly thereafter. Deterministic.  
`vesting_cron.rs`| services/esop/src/vesting_cron.rs| Monthly cron that materialises vesting_event rows. Idempotent (UNIQUE by (grant_id, period)).  
`valuation.rs`| services/esop/src/valuation.rs| Annual valuation cycle. CFO inputs base value; Board signs Industry Multiplier; auditor attests. New `valuation` row supersedes prior.  
`put_option.rs`| services/esop/src/put_option.rs| Put-option exercise. Year 3+ window (annual July). Capped by (FR pending) per-Member per-year limit. Wires cash via vietnam-bank-transfer or Wise.  
`leaver.rs`| services/esop/src/leaver.rs| Termination handler. Good Leaver: vested retained, future vesting halted, put rights preserved. Bad Leaver: vested retained at appendix discount, put rights frozen. Requires CFO + CEO co-sign.  
`pool.rs`| services/esop/src/pool.rs| Pool balance. Replenishment annually by % of profit (Board-signed parameter). GL/BL split for top-performers vs broad.  
`ma_acceleration.rs`| services/esop/src/ma_acceleration.rs| M&A acceleration. Board fires; bulk vesting event for all open grants. Tax-implication note attached.  
`holdco_flip.rs`| services/esop/src/holdco_flip.rs| HoldCo flip — one-way migration. Phantom row → real_share row in Singapore HoldCo schema. CEO-only designation; DOC handles paperwork.  
`cosign_guard.rs`| services/esop/src/cosign_guard.rs| Predicate at boundary. Grants: CEO + CFO + Board. Valuations: CFO + Board + auditor.  
`narrator.rs`| services/esop/src/narrator.rs| Read-only narrator — explains a Member's vesting curve + put schedule + valuation history. Never reveals aggregate cap table.  
`memory_bridge.rs`| services/esop/src/memory_bridge.rs| Writes opaque event refs to memory (e.g. `esop.grant.issued:opaque_id`). Never writes numeric values. CI gate inspects.  
`pdf_renderer.rs`| services/esop/src/pdf_renderer.rs| Deterministic grant + valuation PDFs via tectonic. SHA-256 stored.  
`migrations/`| services/esop/migrations/| sqlx migrations. Append-only constraints. Separate KMS key from HR + REW + memory.  
  
4

## Data model

All grant + vesting + valuation rows are append-only. Corrections happen via supersession. The pool balance is a materialised view — derived from the sum of pool_contribution + grant.sp_count − put_exercise.sp_count. 

erDiagram TENANT ||--|| POOL_BALANCE: "owns" POOL_BALANCE ||--o{ POOL_CONTRIBUTION: "replenished by" MEMBER ||--o{ GRANT: "receives" GRANT ||--o{ VESTING_EVENT: "produces" GRANT ||--o{ PUT_EXERCISE: "exercised against" GRANT ||--o| GRANT_TERMINATION: "ends with" TENANT ||--o{ VALUATION: "publishes annually" VALUATION ||--o{ PUT_EXERCISE: "priced at" TENANT ||--o| MA_ACCELERATION: "fired once at exit" MEMBER ||--o| HOLDCO_FLIP_RECORD: "migrated to SG" GRANT { uuid id PK uuid tenant_id FK uuid member_id FK string kind "founding | milestone | retention" decimal sp_count "synthetic units" date grant_date date cliff_date "+ P0 → P3 horizon default" date final_vest_date "+ 48 months default" string vesting_schedule "monthly | quarterly | custom" uuid pdf_s3_uri FK bytea pdf_sha256 string status "active | accelerated | terminated_gl | terminated_bl | flipped" uuid superseded_by FK timestamp issued_at uuid ceo_cosign_sig uuid cfo_cosign_sig uuid board_cosign_sig } VESTING_EVENT { uuid id PK uuid grant_id FK date period_end decimal sp_vested_this_period decimal sp_vested_cumulative string status "scheduled | materialised | accelerated | halted" timestamp computed_at } VALUATION { uuid id PK uuid tenant_id FK string period "2026" decimal base_value_vnd decimal industry_multiplier decimal sp_unit_value_vnd "= base × multiplier / total_outstanding_sp" date effective_from date effective_to "NULL = current" uuid superseded_by FK uuid cfo_sig uuid board_chair_sig uuid auditor_attestation_id uuid report_s3_uri FK timestamp published_at } POOL_BALANCE { uuid tenant_id PK decimal total_pool_sp decimal issued_sp decimal available_sp timestamp last_recomputed } POOL_CONTRIBUTION { uuid id PK uuid tenant_id FK string period "2026" decimal profit_vnd decimal pct_of_profit "Board-signed param" decimal sp_added string split "GL_branch | BL_branch | custom" timestamp recorded_at } PUT_EXERCISE { uuid id PK uuid grant_id FK uuid member_id FK decimal sp_count uuid valuation_id FK decimal cash_amount_vnd string payment_rail "vietqr | wise_sgd | manual" string status "requested | approved | wired | settled" timestamp requested_at timestamp wired_at } GRANT_TERMINATION { uuid grant_id PK string branch "good_leaver | bad_leaver" decimal sp_vested_at_termination decimal sp_forfeited decimal bl_discount_pct "if bad_leaver" string put_rights_status "preserved | frozen" uuid cfo_cosign_sig uuid ceo_cosign_sig timestamp terminated_at } MA_ACCELERATION { uuid tenant_id PK date event_date string buyer_party decimal sp_accelerated_total string tax_implication_note uuid board_sig timestamp recorded_at } HOLDCO_FLIP_RECORD { uuid member_id PK uuid tenant_id FK decimal phantom_sp_at_flip decimal real_shares_issued "in Singapore HoldCo" string ratio "phantom: real" date flip_date uuid ceo_designation_sig uuid doc_paperwork_id "DOC e-sign reference" timestamp recorded_at } 

### Grant type catalogue

Kind| When issued| Typical schedule| Notes  
---|---|---|---  
`founding`| At hire| 4-year, 12-mo cliff, monthly after| Largest single grant; reflects offer-letter ESOP allocation.  
`milestone`| Achievement-based| 2-year, 6-mo cliff, monthly| Triggered by promotion, project landing, leadership milestone.  
`retention`| Annual top-up| 4-year, no cliff, monthly| Smooth annual top-up to maintain forward vesting trajectory.  
`special`| Board-designated| custom (any)| Founder discretion, requires extra approval; ad-hoc.  
  
5

## API surface

Three surfaces — GraphQL read-mostly with strict self-scope, REST admin with multi-sign predicates, MCP narrator-only tools. Aggregate cap table is restricted to CEO + CFO + Board roles; Members see only their own grants and put schedule. 

### GraphQL subgraph (read-mostly · self-scope)
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key", "@requiresScopes"])
    
    type Grant @key(fields: "id") {
     id: ID!
     memberId: ID!
     kind: GrantKind!
     spCount: String! # string to avoid float drift on units
     grantDate: Date!
     cliffDate: Date!
     finalVestDate: Date!
     status: GrantStatus!
     vestingEvents: [VestingEvent!]!
     spVestedToDate: String!
    }
    
    type VestingEvent {
     periodEnd: Date!
     spVestedThisPeriod: String!
     spVestedCumulative: String!
     status: VestingStatus!
    }
    
    type Valuation @key(fields: "id") {
     id: ID!
     period: String!
     spUnitValueVnd: String!
     effectiveFrom: Date!
     effectiveTo: Date
     publishedAt: DateTime!
     # base_value + industry_multiplier are CEO/CFO/Board scope only
    }
    
    type PutExercise @key(fields: "id") {
     id: ID!
     grantId: ID!
     spCount: String!
     cashAmountVnd: String!
     status: PutStatus!
     requestedAt: DateTime!
     wiredAt: DateTime
    }
    
    enum GrantKind { FOUNDING MILESTONE RETENTION SPECIAL }
    enum GrantStatus { ACTIVE ACCELERATED TERMINATED_GL TERMINATED_BL FLIPPED }
    enum VestingStatus { SCHEDULED MATERIALISED ACCELERATED HALTED }
    enum PutStatus { REQUESTED APPROVED WIRED SETTLED }
    
    type Query {
     myGrants: [Grant!]!
     myValuationHistory: [Valuation!]! # public valuation rows
     myPutSchedule: PutSchedule!
     capTable: CapTable! @requiresScopes(scopes: [["esop.cap_table_read"]])
    }
    
    type Mutation {
     requestPutExercise(grantId: ID!, spCount: String!): PutExercise!
    }

### REST admin surface (multi-sign required)

Method| Path| Purpose| Co-sign?  
---|---|---|---  
POST| `/admin/grants`| Issue a new grant.| **CEO + CFO + Board**  
POST| `/admin/valuations`| Publish annual valuation.| **CFO + Board chair + auditor**  
POST| `/admin/valuations/{id}/auditor-attest`| External auditor attestation step.| auditor (separate token)  
POST| `/admin/put-exercise/{id}/approve`| Approve a put exercise; trigger wire.| CFO  
POST| `/admin/grants/{id}/terminate`| Trigger Good Leaver or Bad Leaver branch.| **CEO + CFO**  
POST| `/admin/ma-acceleration`| Fire M&A acceleration; bulk vesting event.| **Board**  
POST| `/admin/holdco-flip/designate`| CEO designates HoldCo flip eligibility.| CEO  
POST| `/admin/holdco-flip/{member_id}/execute`| Execute flip for a Member; routes to DOC for SG paperwork.| **CEO + Board**  
POST| `/admin/pool/replenish`| Annual pool replenishment from profit.| **CEO + CFO + Board**  
GET| `/admin/cap-table`| Aggregate cap table (CEO/CFO/Board scope only).| readonly · scope-gated  
POST| `/admin/dsar/{member_id}/export`| DSAR — own grants + put exercises only.| DPO  
  
### MCP tool catalogue (narrator-only · no write tools)

Tool name| Inputs| Outputs| Annotations  
---|---|---|---  
`cyberos.esop.explain_my_vesting`| member_id (own)| narrative vest curve| readonly · self-scope  
`cyberos.esop.simulate_put`| grant_id, sp_count| narrative simulation (no commit)| readonly · simulation only  
`cyberos.esop.explain_holdco_flip`| —| narrative policy text| readonly  
`cyberos.esop.upcoming_put_window`| —| "next window opens YYYY-MM-DD"| readonly  
`cyberos.esop.explain_leaver_outcome`| scenario (good/bad)| narrative outcome| readonly · policy lookup  
  
**Forbidden:** no `cyberos.esop.issue_grant`, no `cyberos.esop.publish_valuation`, no `cyberos.esop.terminate`. All destructive ops go through REST admin with multi-sign.

6

## Key flows

### Flow 1 — Grant issuance (3-way co-sign)

sequenceDiagram autonumber participant CEO as CEO participant CFO as CFO participant BC as Board chair participant E as ESOP /admin/grants participant G as grant.rs participant P as PDF render participant S3 as AWS S3 (object-lock) participant K as KMS participant B as 🧠 memory CEO->>E: POST /admin/grants  
{member_id, kind:"founding", sp_count:5000, schedule:"4y-12mo"} E->>E: validate pool.available_sp ≥ 5000 E->>CEO: request WebAuthn co-sign E->>CFO: request WebAuthn co-sign E->>BC: request WebAuthn co-sign CEO-->>E: WebAuthn ✓ CFO-->>E: WebAuthn ✓ BC-->>E: WebAuthn ✓ E->>E: cosign_guard ✓ (3-sign within 24h window) E->>G: INSERT grant row (status="active") E->>P: render grant PDF (deterministic) P->>K: wrap with esop-key K-->>P: ciphertext P->>S3: PUT grants/<member>/<id>.pdf (10y object-lock) S3-->>E: s3_uri E->>E: update pool_balance.issued_sp += 5000 E->>B: opaque ref "esop.grant.issued:<opaque_id>" E-->>CEO: grant issued Note over E,B: memory row contains NO sp_count, NO valuation.  
Only an opaque pointer to ESOP DB. 

### Flow 2 — Annual valuation (CFO + Board + Auditor)

sequenceDiagram autonumber participant CFO as CFO participant BC as Board chair participant AUD as External auditor (KPMG/EY/PwC) participant V as ESOP /admin/valuations participant POOL as pool.rs participant S3 as AWS S3 participant B as 🧠 memory CFO->>V: POST /admin/valuations  
{period:"2026", base_value_vnd:50e9, industry_multiplier:8.0} V->>POOL: read total_outstanding_sp POOL-->>V: 100,000 SP V->>V: compute sp_unit_value = base × mult / total = 4,000,000 VND/SP V->>V: persist valuation row status="draft" V->>CFO: request co-sign V->>BC: request co-sign V->>AUD: request auditor attestation CFO-->>V: WebAuthn ✓ BC-->>V: WebAuthn ✓ AUD-->>V: attestation document + signature (auditor-token) V->>V: cosign_guard ✓ (CFO + Board + Auditor) V->>V: status="published"  
prior valuation effective_to=now V->>S3: archive valuation report PDF (10y) V->>B: opaque ref "esop.valuation.published:2026" Note over V,B: SP unit value, base value, multiplier all  
stay in ESOP keyspace. Members see only the unit value. 

Once published, the valuation row is immutable. Subsequent put exercises in 2026 use this valuation. A 2027 valuation supersedes; 2026 historical put exercises remain priced at the 2026 valuation forever.

### Flow 3 — Vesting compute (monthly · deterministic)

sequenceDiagram autonumber participant CRON as Monthly cron participant VC as vesting_cron.rs participant V as vesting.rs (pure fn) participant PG as PostgreSQL participant B as 🧠 memory CRON->>VC: trigger end-of-month VC->>PG: SELECT all grants status="active" loop per grant VC->>V: vest_for(grant, today) V->>V: if today before cliff: 0 V->>V: else: sp_count × (months_since_cliff / 36) capped at sp_count V-->>VC: vesting_event row (idempotent by grant_id+period) VC->>PG: INSERT vesting_event (idempotent) end VC->>B: opaque ref "esop.vesting.materialised:2026-04 ×N" Note over V: pure fn · no I/O · no clock (today is INPUT).  
Replay test: vest at 2025-12-31 always gives same result. 

### Flow 4 — Put-option exercise (Year 3+)

sequenceDiagram autonumber participant M as Member participant E as ESOP /admin/put-exercise participant P as put_option.rs participant V as latest valuation participant CFO as CFO (approver) participant BANK as 🛠 vietnam-bank-transfer / Wise participant B as 🧠 memory Note over M: July annual window opens for Year-3+ Members. M->>E: requestPutExercise {grant_id, sp_count:500} E->>P: validate eligibility (Year 3+ · within cap) P->>P: cap check: ≤ 25% of vested SP this year P->>V: read sp_unit_value_vnd V-->>P: 4,000,000 VND/SP P->>P: cash_amount = 500 × 4,000,000 = 2,000,000,000 VND P->>E: put_exercise row status="requested" E->>CFO: request approval CFO-->>E: approve E->>P: status="approved" P->>BANK: wire 2B VND to member's bank account BANK-->>P: transfer_id + ack P->>P: status="wired" → "settled" P->>P: grant.sp_count_remaining -= 500 E->>B: opaque ref "esop.put.settled:<opaque_id>" E-->>M: notification "put exercise settled" Note over E,B: ESOP value not in memory; only opaque event ref. 

### Flow 5 — HoldCo flip (phantom → SG real share)

sequenceDiagram autonumber participant CEO as CEO participant BOARD as Board participant E as ESOP /admin/holdco-flip participant H as holdco_flip.rs participant DOC as 📄 DOC e-sign participant SGREG as Singapore HoldCo registry participant M as Member participant B as 🧠 memory Note over CEO: ARR reached $1.5M; CEO designates flip. CEO->>E: POST /admin/holdco-flip/designate CEO-->>E: WebAuthn ✓ BOARD-->>E: WebAuthn ✓ E->>H: open flip period; freeze phantom unit_value loop per Member with active grants H->>H: compute real_shares = phantom_sp × ratio H->>DOC: generate Singapore HoldCo share-issuance docs DOC->>M: present for WebAuthn e-sign M-->>DOC: signed DOC->>SGREG: file share-issuance with ACRA SGREG-->>DOC: registered + share certificate id H->>H: holdco_flip_record row (one-way; phantom grant → status=flipped) end E->>B: opaque ref "esop.holdco.flipped:2027-Q2" Note over H: One-way migration. Phantom grants permanently inactive.  
Future grants are real SG HoldCo shares. 

The flip is a strategic transition — it changes the legal substrate but not the economic outcome. Each Member's real shares carry the same vesting schedule as the source phantom grant (e.g. unvested phantom → unvested real with same dates).

7

## Grant lifecycle

A grant traverses up to seven terminal states. Each transition writes an opaque memory row + a DB audit trail. 

stateDiagram-v2 [*] --> Issued: CEO + CFO + Board co-sign Issued --> Cliffed: 12-mo cliff passed Cliffed --> Vesting: monthly vesting accrues Vesting --> FullyVested: 48 months reached Vesting --> Accelerated: M&A; acceleration fires (Board) Vesting --> TerminatedGL: Good Leaver branch Vesting --> TerminatedBL: Bad Leaver branch FullyVested --> Vesting: (still active for puts) Vesting --> Flipped: HoldCo flip executed Cliffed --> TerminatedGL: GL during cliff (rare; vested=0) Cliffed --> TerminatedBL: BL during cliff (vested=0) TerminatedGL --> [*] TerminatedBL --> [*] Accelerated --> [*] Flipped --> [*] FullyVested --> [*] 

### Good Leaver vs Bad Leaver branch comparison

Outcome| Good Leaver| Bad Leaver  
---|---|---  
Vested SP| Retained at face valuation| Retained at appendix discount (e.g. 40-60% of valuation)  
Unvested SP| Forfeited| Forfeited  
Put rights| Preserved (still exercisable in future windows)| Frozen (no future put exercises)  
HoldCo flip eligibility| Eligible if pre-flip| Excluded from flip  
Trigger| Voluntary resignation, redundancy, contract end, mutual agreement| Termination for cause (per Code of Conduct), competitive breach  
Approval| CFO confirms| CEO + CFO co-sign (mandatory)  
  
The branch decision is human-only — no system path selects GL vs BL automatically. The offboarding orchestrator (in HR) surfaces a recommendation; the actual selection is a CEO + CFO signed event in ESOP.

8

## Functional Requirements

The CyberOS FR catalogue is being rebuilt one feature at a time via the open [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) Agent Skill.

Previous FR enumerations were archived 2026-05-14 and are no longer reflected on this page. Specific FRs land here as they are re-authored.

9

## Non-Functional Requirements

(security) and §11.2.4 (reliability) bind on ESOP — particularly grant immutability and valuation auditability.

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`N(FR pending)`| Grant row UPDATE attempted| = 0 — DB role lacks UPDATE| DB role inspection in CI  
`N(FR pending)`| Valuation row UPDATE attempted| = 0 — DB role lacks UPDATE| DB role inspection in CI  
`N(FR pending)`| memory row containing ESOP numeric value| = 0 — sev-0| CI: memory_bridge emit JSON inspected against numeric blocklist  
`N(FR pending)`| Single-signer grant or valuation| = 0 — cosign_guard blocks| integration test injects single-sign attempt  
`N(FR pending)`| KMS key isolation (esop-key distinct)| 1 distinct key handle| KMS policy inspection  
`N(FR pending)`| Vesting determinism — replay equality| 100% (property test)| proptest: vest at past dates → identical output  
`N(FR pending)`| Cap-table snapshot integrity (quarterly)| SHA-256 stable| chaos test recomputes; asserts equality  
`N(FR pending)`| Put-option SLO (exercise → wire)| ≤ 10 working days p95| OBS dashboard  
`N(FR pending)`| ESOP availability| ≥ 99.5%| SLO monitor  
`N(FR pending)`| Grant + valuation durability (10-year)| 0 lost objects| S3 object-lock + quarterly inventory  
`N(FR pending)`| Vesting compute p95 (single grant)| ≤ 5 ms| bench/vesting.rs  
`N(FR pending)`| Annual vesting cron (50 members · 200 grants)| ≤ 30 s| bench/vesting_cron.rs  
  
10

## Dependencies

ESOP reads roster from HR (for terminations) and writes opaque references to memory. Put-option wires go through vietnam-bank-transfer (VND) or Wise (SGD). HoldCo flip goes through DOC for Singapore paperwork. 

graph LR subgraph upstream ["ESOP depends on"] AUTH["🔐 AUTH  
multi-sign predicate"] HRMOD["👥 HR  
roster + termination"] memory["🧠 memory  
opaque refs only"] OBS["👁 OBS"] KMS["🔑 esop-key (distinct)"] S3["🗂 S3 (10y object-lock)"] end subgraph skills ["External services"] BANK["🛠 vietnam-bank-transfer  
put-option VND wires"] WISE["Wise  
SGD wires"] DOC["📄 DOC  
HoldCo paperwork · P4"] ACRA["Singapore ACRA  
HoldCo registry"] end ESOP["📊 ESOP"] subgraph downstream ["ESOP feeds"] MEM["Member portal  
own grants + put"] CUO["🤖 CUO/CFO-skill  
narrator"] CEO["CEO + CFO + Board  
cap-table reads"] DPO["DPO  
DSAR own scope"] end AUTH --> ESOP HRMOD --> ESOP memory --> ESOP OBS --> ESOP KMS --> ESOP S3 --> ESOP ESOP --> BANK ESOP --> WISE ESOP --> DOC DOC --> ACRA ESOP --> MEM ESOP --> CUO ESOP --> CEO ESOP --> DPO classDef planned fill:#bfdbfe,stroke:#45210e classDef shipped fill:#f5ede6,stroke:#45210e classDef ext fill:#fef6e0,stroke:#9c750a class ESOP planned class AUTH,HRMOD,MEM,CUO,CEO,DPO,DOC,OBS planned class memory,KMS,S3,BANK shipped class WISE,ACRA ext 

11

## Compliance scope

ESOP defends against Vietnamese securities law (by being phantom), Decree 38/2020 corporate governance, PIT obligations on put exercises, and Singapore ACRA filings (on HoldCo flip).

Regulation / standard| Article / clause| ESOP feature that satisfies it  
---|---|---  
Vietnam Law on Enterprises 59/2020/QH14| Art. 114 — Share issuance| Phantom Stock — no shares issued; not subject to share-issuance regulation.  
Vietnam Decree 38/2020/NĐ-CP| Art. 6 — Corporate governance| Phantom Stock treated as deferred compensation; governance via employment contract.  
Vietnam Securities Law 54/2019/QH14| Art. 30 — Public offering| N/A — phantom plan is not a securities offering.  
Circular 111/2013/TT-BTC| Art. 7 — PIT on deferred comp| Put-exercise cash treated as deferred-comp PIT; line item generated for Vietnamese tax filings.  
Decree 119/2018/NĐ-CP| Art. 4 — Record retention| S3 object-lock 10-year on grant + valuation PDFs.  
Vietnam PDPL (Law 91/2025)| Art. 14 — DSAR| Member DSAR returns own grant + put schedule; cap table is excluded (not personal data).  
VN Accounting Standards (VAS 17)| Income taxes / deferred| Liability accrued for unvested phantom stock; tax-effect tracked.  
Singapore Companies Act| S 67 — Share issuance| HoldCo flip generates real share issuance; DOC files via ACRA.  
Singapore ACRA filings| Form 24 / Form 45| DOC paperwork includes ACRA Form 24 (allotment) per Member at flip.  
EU AI Act| (N/A)| ESOP decisions are not automated employment decisions — every grant is a 3-sign human action; not in scope.  
GDPR (EU 2016/679)| Art. 32 — Security| KMS-wrapped at rest · distinct key · co-sign + audit chain.  
ISO/IEC 27001:2022| A.8.10 — Information deletion| Grant + valuation rows append-only; supersession not deletion.  
SOC 2 Type II| CC8.1 — Change management| 3-sign + audit chain + 10-year retention.  
  
12

## Risk entries

ESOP's principal risks are grant-row mutation, single-signer grant issuance, and Bad-Leaver mis-classification (legal exposure).

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-ESOP-001`| Grant row mutated in place (audit trail compromised)| Low| Catastrophic| CSO| DB role lacks UPDATE on grant table; migration grep; integration test attempts UPDATE and asserts rejection.  
`R-ESOP-002`| Single-signer grant issuance (CEO unilateral)| Low| Catastrophic| CEO| cosign_guard requires 3 signatures within 24h window; integration test asserts single-sign rejected.  
`R-ESOP-003`| Valuation drift across recompute (non-determinism)| Low| High| CTO| vesting.rs is a pure fn; replay test asserts byte-identical output for historical dates.  
`R-ESOP-004`| Bad Leaver classification disputed (legal exposure)| Medium| High| CLO| BL decision is human-only (CEO + CFO co-sign), with documented reason; appeal path via Board chair.  
`R-ESOP-005`| Put-option wire fails or delays past 10-day SLO| Medium| Low| CFO| vietnam-bank-transfer / Wise integration with retry + alert; manual fallback workflow.  
`R-ESOP-006`| External auditor unavailable for annual valuation| Low| Medium| CFO| Engaged auditor on retainer + secondary auditor relationship for continuity.  
`R-ESOP-007`| ESOP value leaks via memory audit row| Low| Catastrophic| CSO| memory_bridge inspects emit JSON; CI gate rejects numeric values in ESOP rows; opaque refs only.  
`R-ESOP-008`| HoldCo flip fails — Singapore filing rejected| Medium| High| CEO + CLO| DOC integration with ACRA tested in P4 sandbox; flip executed Member-by-Member with rollback path.  
`R-ESOP-009`| Pool over-allocation (issued_sp > total_pool_sp)| Low| High| CFO| DB CHECK constraint on pool_balance; INSERT grant validates pool.available_sp ≥ requested.  
`R-ESOP-010`| Vietnamese securities-law re-classification (phantom deemed real)| Low| High| CLO| Annual legal review; CLO sign-off on Phantom Stock structure; ready-to-flip path if regulatory landscape changes.  
`R-ESOP-011`| Good/Bad Leaver branch chosen by AI without sign-off| Low| Critical| CSO| Branch classification has zero auto-route; CFO + CEO co-sign always required; CI gate verifies no algorithmic path to branch decision.  
`R-ESOP-012`| Put-option request blocked due to ARR-trigger config drift| Medium| Medium| CFO| ARR threshold versioned in tenant config; mismatch with valuation parameters flagged at quarterly review; manual override with audit row.  
`R-ESOP-013`| Vesting accrual stops on maternity/paternity leave (HR statutory leave drift)| Medium| Medium| CLO| Statutory leaves (per HR §0) accrue vesting; sabbatical / unpaid leave pauses; classification version-pinned with HR; quarterly accrual audit.  
`R-ESOP-014`| M&A acceleration trigger fires on Board declaration without Member notification| Low| High| CEO| Board declaration of change-of-control triggers acceleration + per-Member notice within 5 business days; CI test asserts notification fan-out.  
`R-ESOP-015`| HoldCo flip leaves some Members un-flipped (partial migration state)| Low| High| CEO + CLO| Flip is Member-by-Member with rollback path per R-ESOP-008; ACRA filing batch designed to fully succeed or fully roll back per cohort; partial-state alarm on monitoring.  
  
13

## KPIs

ESOP health rolls up into 10 KPIs across grant integrity, retention, and compliance.

KPI| Formula| Source| Target  
---|---|---|---  
**Grant audit chain integrity**|  grants with full 3-sign / total| ESOP DB| = 100%  
**Valuation auditor-attestation rate**|  attested / published| ESOP DB| = 100%  
**Vesting determinism replay pass**|  replay passes / runs| CI| = 100%  
**Put-option SLO (exercise → wire)**|  p95 days| OBS| ≤ 10 working days  
**Single-signer attempts blocked**|  cosign_guard rejections| OBS| tracked; alert on prod > 0  
**ESOP-value-in-memory incidents**|  CI gate failures| CI| = 0  
**Retention rate (Members with active grants)**|  active_grant_holders / member_count| HR + ESOP| tracked; target ≥ 80%  
**Bad-Leaver appeal rate**|  appeals / BL classifications| ESOP DB| tracked  
**Pool over-allocation incidents**|  CHECK constraint failures| DB| = 0  
**Annual valuation on-time**|  year_end + Q1 completion| ESOP DB| ≤ Mar 31 each year  
**Good/Bad Leaver co-sign integrity**|  branch decisions with CFO + CEO tokens / total| memory audit| = 1.0 (hard floor)  
**Vesting accrual statutory-leave correctness**|  maternity/paternity periods correctly accrue / total such periods| quarterly audit| = 1.0  
**M &A acceleration notification SLA**| p95 (Board declare → Member notice)| OBS| ≤ 5 business days  
**HoldCo flip cohort success rate**|  cohorts fully migrated / total flip cohorts| ACRA logs + ESOP DB| = 1.0 (rollback on partial)  
**Put-option exec query latency**|  request → eligibility check p95| OBS| ≤ 30 s (Member sees status quickly)  
  
14

## RACI matrix

ESOP is owned by CEO + Board jointly; CFO drives valuation cycle; CLO advises on regulatory posture.

Activity| CEO| CFO| Board| CLO| Auditor| HR/Ops  
---|---|---|---|---|---|---  
Grant issuance| A/R| R| R| C| I| C  
Annual valuation| C| A/R| R| C| R| I  
Vesting compute (cron)| I| A| I| I| I| I  
Put-option approval + wire| I| A/R| I| I| I| C  
Good Leaver decision| C| A/R| I| I| I| R  
Bad Leaver decision| A/R| R| C| C| I| C  
M&A acceleration| C| C| A/R| C| I| I  
HoldCo flip designation| A/R| R| R| R| C| I  
Pool replenishment| C| R| A| I| I| I  
Regulatory review (annual)| C| C| C| A/R| I| I  
  
**R** Responsible · **A** Accountable · **C** Consulted · **I** Informed.

15

## Planned CLI surface

Admin CLI `cyberos-esop`.

### 1\. Issue a founding grant (3-way co-sign)
    
    
    $ cyberos-esop grant issue \
     --member mai@cyberskill.com \
     --kind founding \
     --sp-count 5000 \
     --schedule 4y-12mo \
     --cosign-ceo --cosign-cfo --cosign-board
    
    [pool] available_sp: 80,000 → 75,000 after issue ✓
    [cosign] ceo: stephen@cyberskill.com (WebAuthn) ✓
    [cosign] cfo: hoa@cyberskill.com (WebAuthn) ✓
    [cosign] board: thanh@board.cyberskill.com (WebAuthn) ✓
    [guard] 3-sign within 24h ✓
    [grant] 01HZJ8…F9D issued
    [pdf] rendered · SHA-256 = 9f3e…
    [s3] grants/mai/01HZJ8.pdf (10y object-lock)
    [audit] memory seq=17001 (opaque "esop.grant.issued")

### 2\. Publish annual valuation
    
    
    $ cyberos-esop valuation publish \
     --period 2026 \
     --base-value-vnd 50000000000 \
     --industry-multiplier 8.0 \
     --auditor kpmg-vietnam \
     --cosign-cfo --cosign-board --auditor-attest
    
    [compute] sp_unit_value = 50B × 8.0 / 100,000 SP = 4,000,000 VND/SP
    [cosign] cfo ✓ board ✓
    [auditor] kpmg-vietnam attestation: SHA-256 7e8a…
    [publish] valuation 2026 status=published
    [supersede] valuation 2025 effective_to = 2025-12-31
    [s3] valuations/2026.pdf (10y object-lock)
    [audit] memory seq=17012 (opaque)

### 3\. Read your vesting curve
    
    
    $ cyberos-esop my-vesting
    
    [your grants]
     founding 2025-09-01 · 5,000 SP · 4y-12mo
     cliff: 2026-09-01 (vested at cliff: 1,250 SP)
     vested-now: 1,520 SP (as of 2026-05-14)
     next-vest: 2026-05-31 → +104 SP
     fully-vests: 2029-09-01
    
     retention 2026-04-01 · 1,000 SP · 4y-no-cliff
     vested-now: 35 SP
     next-vest: 2026-05-31 → +21 SP
     fully-vests: 2030-04-01
    
    [total vested today] 1,555 SP
    [next put window] 2028-07 (Year 3 from founding grant cliff)

### 4\. Exercise put option
    
    
    $ cyberos-esop put exercise \
     --grant 01HZJ8…F9D \
     --sp-count 400
    
    [eligibility] Year 3+ ✓
    [cap] max 25%/yr of vested = max 500 SP this year ✓
    [valuation] 2028 unit value: 5,500,000 VND/SP
    [cash] 400 × 5,500,000 = 2,200,000,000 VND
    [wire] via vietnam-bank-transfer → mai's BIDV account
    [status] requested → awaiting CFO approval
    
    # (CFO approves separately)
    $ cyberos-esop put approve --id put-01HZK0…
    
    [wire] sent · transfer_id BIDV-9F3E2A1B
    [status] wired → settled (T+0 via Napas247)
    [audit] memory seq=17034 (opaque "esop.put.settled")

### 5\. Terminate (Good Leaver)
    
    
    $ cyberos-esop terminate \
     --member nam@cyberskill.com \
     --branch good_leaver \
     --reason "voluntary resignation · 6 weeks notice" \
     --cosign-cfo
    
    [branch] good_leaver
    [vesting] halted at 2026-05-14
    [vested] 1,150 SP retained at face valuation
    [forfeit] 3,850 SP unvested forfeited (returned to pool)
    [put] rights preserved · next window 2028-07
    [pool] available_sp: 75,000 → 78,850 after forfeiture
    [audit] memory seq=17041 (opaque)

### 6\. Cap-table read (CEO/CFO/Board scope)
    
    
    $ cyberos-esop cap-table
    
    [cap table · 2026-05-14]
    member founding milestone retention total_vested vested%
    stephen@… founding - - 17,200 86%
    hoa@… founding milestone retention 8,420 72%
    mai@… founding - retention 1,555 28%
    … … … … … …
    ───────── ────── ────── ───────── ────────── ──────
    pool issued: 55,400 55%
    pool available: 44,600 45%
    pool total: 100,000 100%
    
    [valuation 2026] sp_unit_value: 4,000,000 VND/SP

### 7\. HoldCo flip designation
    
    
    $ cyberos-esop holdco-flip designate \
     --reason "ARR reached $1.5M trigger · 2027-Q2" \
     --cosign-ceo --cosign-board
    
    [trigger] ARR $1.52M ≥ $1.5M ✓
    [cosign] ceo ✓ board ✓
    [freeze] phantom unit value @ 2027-Q2 valuation
    [doc] routing 24 Members for SG share issuance paperwork via DOC
    [acra] Form 24 templates generated
    [status] flip period OPEN — Members can sign in DOC
    [audit] memory seq=17051 (opaque "esop.holdco.designated")

16

## Phase status & estimates

Status

Planned

P2 design phase

Est. LoC (Rust)

~5,400

services/esop + cap-table simulator

Planned tests

110+

incl. vesting + valuation determinism

External libs

~12

axum · sqlx · tectonic · ring · cyberskill-vn

CLI subcommands

~22 planned

`cyberos-esop` entrypoint

P2 budget

~$25/mo

RDS schema + Fargate share

Capability| Status  
---|---  
Append-only grant issuance with 3-sign| planned · P2  
Deterministic vesting compute + monthly cron| planned · P2  
Annual valuation cycle (CFO + Board + Auditor)| planned · P2  
Put-option exercise (Year 3+, capped)| planned · P2  
Good Leaver / Bad Leaver branch| planned · P2  
M&A acceleration (Board-fire)| planned · P2  
Pool replenishment (% of profit)| planned · P2  
Member portal: own grants + put schedule| planned · P2  
Cap table (CEO/CFO/Board scope)| planned · P2  
Quarterly cap-table immutable snapshot| planned · P2  
Dilution simulator (read-only)| planned · P2  
vietnam-bank-transfer integration for put VND wires| planned · P2  
Wise integration for SGD wires| planned · P3  
HoldCo flip — phantom → SG real share| planned · P3  
ACRA Form 24 generation via DOC| planned · P4  
Narrator MCP (read-only · simulation only)| planned · P2  
  
17

## References

  * **FR catalogue** — ESOP module FRs ((FR pending) through (FR pending)).
  * **Architecture spec** — ESOP architecture posture, Phantom Stock framing.
  * **NFR catalogue** — Security NFRs (zero ESOP in memory, dual sign-off).
  * **FR mapping** — Formal (FR pending) through (FR pending) with verification methods.
  * **Total Rewards & Career Path Appendix** — ESOP allocation, BL discount %, vesting defaults.
  * **Vietnam Law on Enterprises (Law 59/2020/QH14)** — Art. 114 share issuance (which Phantom Stock avoids).
  * **Vietnam Securities Law (Law 54/2019/QH14)** — Art. 30 public offering (which Phantom Stock is not).
  * **Vietnam Decree 38/2020/NĐ-CP** — Corporate governance; treats Phantom Stock as deferred compensation.
  * **Circular 111/2013/TT-BTC** — PIT on deferred comp at exercise.
  * **Decree 119/2018/NĐ-CP** — 10-year statutory retention.
  * **Vietnam PDPL (Law 91/2025)** — DSAR self-scope.
  * **Bigger picture (§0 above):** 3 strategic roles + cap-table spine Mermaid + 10-row auto-vs-human matrix.
  * **Cross-module page links:** [hr.html](<../hr/index.html>) · [rew.html](<../rew/index.html>) · [memory.html](<../memory/index.html>) · [auth.html](<../auth/index.html>) · [doc.html](<../doc/index.html>)
  * **memory auto-sync vision:** [MEMORY_AUTOSYNC_DESIGN.md §5](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) \+ DEC-036 — ESOP value structurally excluded from memory.
  * **Build-readiness audit:** `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`) — ESOP at P2 · exit (P2).
  * **FR authoring discipline:** [modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>).
  * **Singapore Companies Act (Cap. 50)** — S 67 share issuance, applicable post-HoldCo-flip.
  * **Singapore ACRA filings** — Form 24 (share allotment), Form 45 (allotment return).
  * **VAS 17 — Income Taxes** — deferred-comp liability accrual.
  * **ISO/IEC 27001:2022** — A.5.13, A.8.10 mapped to append-only grant + valuation.
  * **cyberskill-vn collection** — vietnam-bank-transfer skill used for put-option VND wires.
  * **Architecture context:** [infrastructure.html#esop](<../../architecture/infrastructure.html#esop>).



[← INV](<../inv/index.html>) [All modules →](<../index.html#catalog>)

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
