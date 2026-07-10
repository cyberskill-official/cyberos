---
title: Milestones — CyberOS
source: website/docs/architecture/milestones.html
migrated: FR-DOCS-002
---

0

## The horizontal timeline

Five phases · five compliance gates · five module-count milestones. The slope changes after P3: P0–P3 is internal-first execution; P4 is the external-GA arc (P4 phase). 

P0 · Foundations

P0 phase

P1 · Productivity

P1 phase

P2 · Operations

P2 phase

P3 · SaaS-ready

P3 phase

P4 · Client-facing

P4 phase

P0 · start

kickoff

P0 · exit

P0 exit · 7/22

P1 · exit

P1 exit · 15/22

P2 · exit

P2 exit · 17/22

P3 · exit

P3 exit · 19/22

P4 · GA

P4 · 22/22

P0 exit · P0 · exit

**7 modules:** AUTH · AI · MCP · OBS · CHAT · memory · GENIE/CUO

**Headcount:** 10

Gate: T1 Floor

A05 DPIA · DPO · Trust Center · Stripe SAQ-A · VPAT 2.5

P1 exit · P1 · exit

**+8 modules:** PROJ · TIME · CRM · KB · HR · EMAIL · REW · LEARN

**Headcount:** 12

Gate: T2 base

SOC 2 Type I · CSA STAR L1 · AI-CAIQ · DSAR APIs

P2 exit · P2 · exit

**+2 modules:** INV · ESOP

**Headcount:** 14

Gate: T2 EU enterprise

SOC 2 Type II · ISO 27001:2022 · CSA STAR L2 · EU AI Act Annex III §4

P3 exit · P3 · exit

**+2 modules:** RES · OKR

**Headcount:** 16

Gate: T3 large enterprise

ISO 42001 AIMS · ISO 27701 · Singapore HoldCo flip (if ARR ≥ $1.5M)

P4 entry · P4 · mid

**+3 modules:** DOC · PORTAL · TEN

**Headcount:** 20

Gate: T3+ regulated

TX-RAMP · StateRAMP Cat 2 · FedRAMP 20x · eIDAS QTSP

Phase-exit gate sequence — what "P-N exit" actually means

sequenceDiagram autonumber participant ENG as Eng Lead participant FOUNDER as Founder/CEO participant DPO as DPO participant CUO as CUO (CTO skill) participant CP as CP module participant AUDIT as Auditor (SOC 2 / ISO) participant BOARD as Board (P2+) Note over ENG,BOARD: T-30 days before phase exit ENG->>CUO: ready-check { phase: P-N } CUO->>CP: enumerate compliance backlog CP-->>CUO: { open: 12, closed: 88, blocked: 0 } CUO-->>ENG: digest · what's missing ENG->>ENG: ship remaining items Note over ENG,BOARD: T-7 days DPO->>CP: pre-audit DPIA review AUDIT->>CP: dry-run access (read-only) AUDIT-->>FOUNDER: findings · severity-tiered Note over ENG,BOARD: T0 phase exit FOUNDER->>CP: declare phase exit CP->>AUDIT: trigger formal audit window CP->>BOARD: board approval (P2+ phases) BOARD-->>CP: approve CP-->>FOUNDER: phase N+1 unlocked CP->>CUO: publish achievement memo 

Module rollout — Gantt view

gantt title CyberOS phased module rollout dateFormat YYYY-MM axisFormat M+%m section P0 Foundation AUTH:done, p0a, 2026-06, 60d AI Gateway:done, p0b, 2026-06, 60d MCP Gateway:done, p0c, 2026-06, 60d OBS:done, p0d, 2026-07, 60d CHAT:done, p0e, 2026-07, 60d memory:done, p0f, 2026-06, 90d GENIE / CUO:done, p0g, 2026-07, 60d section P1 Productivity PROJ:p1a, 2026-09, 90d TIME:p1b, 2026-09, 60d CRM:p1c, 2026-09, 90d KB:p1d, 2026-10, 60d HR (full):p1e, 2026-10, 60d EMAIL:p1f, 2026-10, 90d REW (core):p1g, 2026-09, 90d LEARN:p1h, 2026-11, 60d section P2 Operations INV:p2a, 2026-12, 90d REW (full pool calc):p2b, 2026-12, 60d ESOP:p2c, 2027-01, 90d section P3 SaaS-readiness RES:p3a, 2027-03, 90d OKR:p3b, 2027-04, 60d Singapore HoldCo flip:milestone, p3c, 2027-05, 1d section P4 Client-facing DOC (eIDAS QTSP):p4a, 2027-06, 180d PORTAL:p4b, 2027-08, 180d TEN (tenancy + billing):p4c, 2027-08, 240d First external tenant:milestone, p4d, 2027-12, 1d 

P0

## Foundations · P0 phase

The infrastructure plane plus the substrate (memory), the catalog (SKILL — already shipped) and the orchestrator (CUO). At P0 exit, Slack and Zalo are decommissioned; CyberSkill's 10 Members work inside CyberOS-only. 

#### Modules added (7 of 22)

  * **memory** — universal memory · 3-layer · already shipped
  * **GENIE/CUO** — orchestrator · 5 of 47 C-suite persona workflows live (221 workflows total)
  * **AUTH** — OAuth 2.1 + JWT + per-tenant authz server
  * **AI Gateway** — LiteLLM + Bedrock + redaction + cost ledger
  * **MCP Gateway** — 2025-11-25 spec + tool registry
  * **OBS** — LGTM stack + LangSmith for CUO sessions
  * **CHAT** — message + thread + @genie call-out



#### Compliance gate at exit · T1 Floor

  * A05 DPIA filed with MoPS
  * DPO designated (Founder)
  * Trust Center live at `trust.cyberos.world`
  * Stripe SAQ-A AOC published
  * VPAT 2.5 INT (accessibility) filed
  * PDPL Art. 38 SME grace-period flag set



#### Success criteria

  * All 10 internal Members on CyberOS for chat + memory by P0 · exit
  * Slack + Zalo billing terminated by P0 · exit
  * CUO answers ≥ 50 questions/week with ≥ 98% citation rate (N(FR pending))
  * memory search p95 ≤ 250ms on 50k-chunk corpus
  * Zero compensation/equity facts ingested into memory (denylist DEC-036)
  * p95 GraphQL ≤ 400ms across all subgraphs (N(FR pending))



#### Risks · likelihood × impact

  * **R-001 (M×H)** — Anthropic Skills spec churns; CUO loadout breaks. _Mitigation: schema-pin + conformance tests._
  * **R-002 (M×H)** — MCP 2025-11-25 spec evolves before P0 exit. _Mitigation: dual-spec compat for 1 phase._
  * **R-003 (L×M)** — Bedrock Singapore region capacity issue. _Mitigation: Anthropic ZDR fallback already wired._
  * **R-004 (L×M)** — Vietnamese embedding quality regression. _Mitigation: parity tests vs BGE-M3 baseline._



#### Key milestones within P0

  * **P0 · start:** module template + Federation router + design tokens repo live
  * **P0 · slice 1:** AUTH + AI Gateway in beta; memory + SKILL already shipped baseline
  * **P0 · slice 2:** MCP Gateway live; CUO router-only mode
  * **P0 · exit:** CHAT live; OBS LGTM stack live; 7/23 modules ready; Trust Center launched; PLUGIN scaffold + 8 FRs at 10/10 (cross-runtime distribution to Claude Code / Cursor / Cowork / Codex CLI)



⚠️

## P0 → P1 descope gate

MANDATORY · runs at P1 · start

Every plan that adds modules monotonically becomes a death march. CyberOS must have explicit language for _"this module is descoped to P2 because of P1 reality."_ The descope gate runs at P1 · start (after P0 exit declared) and asks four questions; if any of them score Red, two P1 modules MUST be deferred to P2 before the phase commits. 

### Gate questions (all four scored Green / Amber / Red at P1 · start)

  1. **Did P0 exit ship clean?** All 5 P0 modules (AI Gateway · OBS · AUTH stub · MCP Gateway · CHAT) at `status: shipped`; Trust Center live; SOC 2 readiness signal positive; 0 cross-tenant leak incidents in P0. _Amber = 1 module slipped to P1; Red = 2+ modules slipped, or any incident._
  2. **Is CHAT decommission ≥ 0.95?** 14-day rolling decommission signal: how much of CyberSkill's internal chatter is in CHAT vs Slack/Zalo. _Amber = 0.85–0.94; Red = < 0.85._
  3. **Is the AI Gateway cost-of-everything gate fully operational?** FR-AI-001..005 shipped and audited; 0 budget breaches; cache hit rate ≥ 30%. _Amber = 1 of the 5 FRs deferred; Red = 2+ deferred or any budget breach._
  4. **Is the headcount ramp on track?** 10 → 12 hires by P1 · start. _Amber = 1 hire late by ≤ 30 days; Red = 1+ hire late by > 30 days, or any hire pulled._



### Descope rules (if any Red)

  * **First descope candidate: LEARN.** Defer to P2 entirely. The promotion review through Hội đồng Chuyên môn needs at least one quarter of TIME data anyway; running it in P1 was always optimistic.
  * **Second descope candidate: HR split.** Split into HR-roster (keep at P1, slice 1 of 3) and HR-full (defer to P2). HR-roster covers member directory + 5 contract types + leave; HR-full covers onboarding orchestrator + performance signals.
  * **Third descope candidate (only if 2 Reds): EMAIL slice 3.** Keep Stalwart core + shared inbox (slices 1+2); defer Genie draft + bulk send (slice 3) to P2.
  * **Never descope these:** PROJ slice 1, TIME, CRM slice 1, KB slice 1, REW core. These are load-bearing for revenue + payroll + dogfood.



### Decision protocol

  1. Founder + CTO + CHRO run the 4-question scorecard within 7 days of P0 · exit.
  2. If any Red: descope 1 module per Red, in the order above.
  3. Descope decision MUST be recorded as a memory audit row at `memories/decisions/p0-p1-descope-.md`.
  4. Descoped modules carry status `deferred` with the new target phase; reappear in BACKLOG.md at the new position.
  5. Re-engaging a descoped module requires explicit phase-entry re-evaluation; it doesn't drift back into P1 silently.



**Why this gate exists:** The P1 module batch (8 modules in ~3 phases of build time) is realistic only if the P0 infrastructure ships clean AND hires arrive on schedule. Both are 60-percent-confidence bets at best. Having an explicit "descope which two" gate written into the plan converts "death march" failure into "graceful descope" survival. The research review §1.3 names this as the single biggest sequencing protection CyberOS has not yet written down.

P1

## Internal Productivity · P1 phase

The productivity moat. PROJ + TIME + CRM + KB + HR + EMAIL + REW + LEARN — eight modules that turn the platform from "infrastructure" into "the thing the team uses every day." First payroll cycle, first promotion review through Hội đồng Chuyên môn. **Note:** this 8-module count is the maximum; the P0 → P1 descope gate above may reduce it to 6 or 7 depending on P0 exit scorecard. 

#### Modules added (+8 = 15 of 22)

  * **PROJ** — projects · tasks · cycles · assignments
  * **TIME** — time entries · expense tracking
  * **CRM** — clients · deals · activities
  * **KB** — knowledge base · canonical docs
  * **HR (full)** — Members · roles · leave · onboarding
  * **EMAIL** — Stalwart-based mail + inbox + i18n RFC 6532
  * **REW (core)** — 3P income · payslip · BP balance
  * **LEARN** — career path · Hội đồng peer-review



#### Compliance gate · T2 base

  * SOC 2 Type I issued (point-in-time)
  * CSA STAR L1 self-assessment via CAIQ v4.0.3
  * AI-CAIQ "Valid-AI-ted" badge
  * DSAR APIs end-to-end (GDPR Art. 15 ready)
  * Article 50 transparency badges in all AI surfaces



#### Headcount

**10 → 12 Members**

  * +1 HR/Ops Lead (REW dogfooding)
  * +1 Engineer (PROJ + KB owner)
  * Existing Founder/CEO + Eng Lead + 7 Engineers continue



#### Success criteria

  * First full payroll cycle issued through REW (month-end P1 · exit)
  * First promotion review through Hội đồng Chuyên môn
  * P1 base salary invariant verified: zero system-reductions (N(FR pending))
  * BP balance tracked with anti-inflation interest at ACB rate
  * EMAIL daily summary in memory for every Member
  * CRM activities auto-logged from EMAIL outgoing (CaMeL safe extraction)



#### Risks

  * **R-101 (M×H)** — REW parameter versioning bug; retroactive recompute could violate N(FR pending). _Mitigation: anti-retroactive CI gate; replay tests on every release._
  * **R-102 (M×H)** — EU AI Act Annex III §4 over-application; HR features flagged as high-risk. _Mitigation: DEC-054 — drafts only, no scoring._
  * **R-103 (M×M)** — Stalwart self-hosted EMAIL reliability. _Mitigation: fallback SMTP relay; OBS alarms on bounce rate._
  * **R-104 (L×M)** — SOC 2 Type I auditor unavailability in Vietnam. _Mitigation: Schellman, A-LIGN shortlist; remote audit option._



P2

## Operations · P2 phase

Bill-to-cash and Phantom Stock. INV closes the revenue loop; ESOP closes the equity-honour loop. First SP grant issued; first annual SP valuation cycle complete. 

#### Modules added (+2 = 17 of 22)

  * **INV** — invoicing · MST validation · VAT e-invoice via GDT T-VAN · monthly VAT filing
  * **ESOP** — Phantom Stock · 4-year vesting · put options from Year 3 · annual SP valuation
  * _REW upgrade:_ full pool calculation with BP overflow + sabbatical accrual



#### Compliance gate · T2 EU enterprise

  * SOC 2 Type II issued (6-mo observation window)
  * ISO/IEC 27001:2022 Stage 1 audit complete
  * CSA STAR L2 third-party attestation
  * EU AI Act Annex III §4 conformity pack for REW + LEARN
  * Decree 13 full regime — graduate from SME



#### Headcount

**12 → 14 Members**

  * +1 vCISO (fractional security · cert prep)
  * +1 Account Manager (CRM + INV dogfooding)



#### Success criteria

  * First SP grant issued (after Board-approved valuation)
  * First annual SP valuation cycle complete
  * First e-invoice filed to GDT successfully
  * Monthly VAT return submitted via `vietnam-tax-filing` skill (planned)
  * ARR > $300k (internal-equivalent or first design partner)
  * Good Leaver / Bad Leaver branch tested via tabletop exercise



#### Risks

  * **R-201 (L×VH)** — ESOP put-option model violates Vietnamese tax interpretation. _Mitigation: counsel review at every parameter version; cash-collected pool only._
  * **R-202 (M×H)** — SP valuation methodology challenged by Board. _Mitigation: dual-signed by Founder + Board; methodology in CP module._
  * **R-203 (M×M)** — GDT T-VAN provider connectivity. _Mitigation: multiple T-VAN provider integrations; manual fallback via VN-tax-filing skill._



P3

## SaaS Readiness · P3 phase

The platform earns the right to sell. RES + OKR ship; capacity planning becomes visible; the first quarterly OKR cycle closes. If ARR ≥ $1.5M, the Singapore HoldCo flip happens. 

#### Modules added (+2 = 19 of 22)

  * **RES** — resource plan · capacity vs forecast · CUO COO-skill rebalancing suggestions
  * **OKR** — objectives · key results · quarterly cycle · CUO CEO/CSO-skill cycle-close prompts
  * _Mobile app evaluation_ — RN-based, P3 stretch



#### Compliance gate · T3 Large enterprise

  * ISO/IEC 42001 (AIMS) certified
  * ISO/IEC 27701 (PIMS) — if EU/UK customers push
  * SOC 2 Type II annual refresh
  * Singapore HoldCo flip (CyberSkill Pte Ltd as parent · if ARR ≥ $1.5M)



#### Headcount

**14 → 16 Members**

  * +1 Designer (mobile evaluation + PORTAL prep)
  * +1 Engineer (TEN module groundwork)



#### Success criteria

  * First full quarterly OKR cycle closed (Q1 2027)
  * Capacity-vs-forecast rebalancing run weekly via CUO
  * 3 design-partner tenants signed (paid pilot)
  * ARR ≥ $1.5M triggers Singapore HoldCo flip
  * eu-shard activated for at least one EU pilot tenant



#### Risks

  * **R-301 (M×H)** — Singapore HoldCo flip legal complexity exceeds phase window. _Mitigation: counsel engaged at P2 · exit; flip is optional gate._
  * **R-302 (M×H)** — RES rebalancing flagged as employment-decision high-risk. _Mitigation: suggestion-only mode; human accept/reject required._
  * **R-303 (M×M)** — ISO 42001 AIMS audit window misalignment. _Mitigation: Stage 1 readiness check at P3 · start._



P4

## Client-Facing · P4 phase

External GA. DOC + PORTAL + TEN close the gap. First external paying tenant onboarded. Multi-tenant external GA opens. Regulated-commercial path open via TX-RAMP, StateRAMP Cat 2, FedRAMP 20x Moderate (no-sponsor route if US sub exists). 

#### Modules added (+3 = 22 of 22)

  * **DOC** — document signing · eIDAS QTSP integration · PDF/A-2 archival
  * **PORTAL** — external client portal · approval workflow · client-facing AI answers via CUO CXO skill
  * **TEN** — tenancy management · per-tenant config · billing (Stripe + VNPay)



#### Compliance gate · T3+ regulated

  * TX-RAMP (Texas state)
  * StateRAMP Cat 2
  * FedRAMP 20x Moderate (no-sponsor route if US sub exists)
  * eIDAS QTSP for DOC module — EU-compliant qualified e-signature
  * SOC 2 Type II + ISO 27001 + ISO 42001 all annual-refresh certified



#### Headcount

**16 → 20 Members**

  * +2 Customer Engineers (onboard external tenants)
  * +1 Sales (Account Manager promoted to head)
  * +1 Legal Counsel (CLO; replaces CUO-as-CLO)



#### Success criteria

  * First external paying tenant onboarded by P4 · mid
  * 5 external paying tenants by P4 · late
  * 10 external paying tenants by P4 · GA → ARR ≥ $3M
  * NPS ≥ 40 from external tenants
  * Zero tenant data leakage incidents (N(FR pending) maintained)
  * First eIDAS QTSP-signed document issued



#### Risks

  * **R-401 (M×VH)** — eIDAS QTSP integration partner unreliable. _Mitigation: multiple QTSP integrations; degraded mode = advanced e-signature._
  * **R-402 (M×H)** — first external tenant onboarding takes 4× expected. _Mitigation: TEN module ships ≥ 3 months before first paid customer._
  * **R-403 (L×VH)** — FedRAMP 20x no-sponsor route deprecated. _Mitigation: TX-RAMP + StateRAMP first; FedRAMP deferred._



A

## Module dependency graph

Every module depends on the P0 infrastructure plane (AUTH, AI, MCP, OBS) and on memory for memory + audit. P1 modules dogfeed each other (TIME → REW; CRM → EMAIL); P2 modules close revenue + equity loops; P3 modules read from everything; P4 modules sit at the edge. 

flowchart LR subgraph P0 ["P0 Foundation"] AUTH(("AUTH")) AI(("AI")) MCP(("MCP")) OBS(("OBS")) memory(("memory")) CUO(("CUO")) CHAT(("CHAT")) end subgraph P1 ["P1 Productivity"] PROJ(("PROJ")) TIME(("TIME")) CRM(("CRM")) KB(("KB")) HR(("HR")) EMAIL(("EMAIL")) REW(("REW")) LEARN(("LEARN")) end subgraph P2 ["P2 Operations"] INV(("INV")) ESOP(("ESOP")) end subgraph P3 ["P3 SaaS-ready"] RES(("RES")) OKR(("OKR")) end subgraph P4 ["P4 External"] DOC(("DOC")) PORTAL(("PORTAL")) TEN(("TEN")) end AUTH --> CHAT & PROJ & TIME & CRM & KB & HR & EMAIL & REW & LEARN & INV & ESOP & RES & OKR & DOC & PORTAL & TEN AI --> CUO & memory MCP --> CUO OBS -. observes.-> CHAT & PROJ & CRM & REW & ESOP & RES & OKR CUO --> CHAT & memory memory --> CUO TIME --> REW PROJ --> TIME CRM --> EMAIL CRM --> INV EMAIL --> memory KB --> memory HR --> REW REW --> LEARN REW --> ESOP PROJ --> RES TIME --> RES HR --> OKR RES --> OKR CRM --> PORTAL HR --> TEN AUTH --> TEN INV --> TEN KB --> DOC classDef p0 fill:#e8d4c2,stroke:#45210e,stroke-width:2px classDef p1 fill:#f5ede6,stroke:#45210e classDef p2 fill:#fef6e0,stroke:#9c750a classDef p3 fill:#f9c64f,stroke:#9c750a classDef p4 fill:#f0eee9,stroke:#475569 class AUTH,AI,MCP,OBS,memory,CUO,CHAT p0 class PROJ,TIME,CRM,KB,HR,EMAIL,REW,LEARN p1 class INV,ESOP p2 class RES,OKR p3 class DOC,PORTAL,TEN p4 

Cycle-free by construction. The only "circular" arrow (memory ⇄ CUO) is the legitimate read/write split: CUO writes inquiry context, memory returns search hits.

B

## Headcount · modules · revenue trajectory

Three curves laid on the same time axis. Headcount grows only when CyberOS itself absorbs the operational load. Module count is the leading indicator; revenue is the trailing indicator. 

Headcount (Members)

10 → 20

+10 over P0 → P4. Hires are gated on CyberOS's ability to absorb that Member's onboarding via REW + LEARN + KB.

Modules shipped

7 → 22

+15 over P0 → P4. The 7 P0 modules are the heaviest lift; subsequent phases ship 8 / 2 / 2 / 3.

ARR target

$0 → $3M

$0 at P0 (internal). Design-partner pilots at P2. $1.5M trigger for HoldCo flip at P3. $3M+ at P4 with 10 paying tenants.

Headcount trajectory

xychart-beta title "Member count over P0 → P4" x-axis ["P0 · start", "P0 · exit", "P1 · exit", "P2 · exit", "P3 · exit", "P4 · early", "P4 · mid", "P4 · late", "P4 · GA"] y-axis "Members" 0 --> 22 line [10, 10, 12, 14, 16, 17, 18, 19, 20] 

Module count trajectory

xychart-beta title "Modules shipped (out of 22)" x-axis ["P0 · start", "P0 · exit", "P1 · exit", "P2 · exit", "P3 · exit", "P4 · early", "P4 · mid", "P4 · late", "P4 · GA"] y-axis "Modules" 0 --> 22 line [3, 7, 15, 17, 19, 19, 20, 22, 22] 

ARR trajectory ($k)

xychart-beta title "Annual Recurring Revenue projection ($k)" x-axis ["P0 · start", "P0 · exit", "P1 · exit", "P2 · exit", "P3 · exit", "P4 · early", "P4 · mid", "P4 · late", "P4 · GA"] y-axis "ARR ($k)" 0 --> 3500 line [0, 0, 0, 300, 800, 1500, 2000, 2600, 3200] 

C

## KPI dashboard targets per phase

Each phase has a "north-star plus three" KPI set. North-star is the dogfooding-signal proxy; the three supporting metrics are the leading-edge measurements that tell the founder whether the phase is ready to exit. 

Phase| North-star| KPI 2| KPI 3| KPI 4 · guardrail  
---|---|---|---|---  
**P0** | Slack + Zalo decommissioned · 100% Members on CHAT | CUO citation rate ≥ 98% | memory search p95 ≤ 250ms | Zero compensation in memory (denylist)  
**P1** | First full payroll cycle issued through REW | SOC 2 Type I issued | Time-tracked hours = 100% of billable | P1 base salary system-reductions = 0  
**P2** | First SP grant + first SP valuation cycle complete | SOC 2 Type II + ISO 27001 Stage 1 | VAT e-invoice file rate 100% | Parameter version retroactive mutations = 0  
**P3** | Quarterly OKR cycle closed · capacity rebalanced weekly | ISO 42001 certified | ARR ≥ $1.5M (HoldCo trigger) | EU AI Act Annex III §4 conformance · 100% drafts-only  
**P4** | First external paying tenant onboarded | 10 paying tenants by P4 · GA | NPS ≥ 40 from external tenants | Tenant data leakage incidents = 0  
  
#### Continuous (cross-phase) NFRs

  * N(FR pending) GraphQL p95 ≤ 400ms
  * N(FR pending) memory search p95 ≤ 250ms on 1M chunks
  * N(FR pending) Platform availability ≥ 99.5%
  * N(FR pending) CHAT availability ≥ 99.9%
  * N(FR pending) Tenant data leakage = 0
  * N(FR pending) CUO citation rate ≥ 98%



#### Anti-metrics (we watch these _not_ grow)

  * CUO auto-acts on irreversible op without confirm
  * P1 base salary system-reductions
  * Parameter version retroactive mutations
  * Compensation/equity facts in memory
  * CUO answers without citation when source exists
  * Prompt-injection exfiltration via email/document



∞

## References

#### Strategy source sections

  * The phased milestone arc
  * OKRs by phase exit
  * Guardrail (anti-)metrics
  * All 23 modules catalogued
  * Foundational locked decisions
  * Non-Functional Requirements
  * Compliance tier model per phase
  * AI-driven productivity matrix
  * phased shipping plan



#### Cross-references in CyberOS docs

  * [Infrastructure plane (six P0 pillars)](<infrastructure.html>)
  * [Compliance plan (three rings)](<compliance.html>)
  * [Tech stack (8 tiers)](<tech-stack.html>)
  * [memory module](<../modules/memory/index.html>)
  * [CUO module](<../modules/cuo/index.html>)
  * [Skill module](<../modules/skill/index.html>)
  * [NFR Catalog (full table)](<../reference/nfr-catalog.html>)
  * [Risk Register (full)](<../reference/risk-register.html>)



[ Previous · Tech stack ](<tech-stack.html>) [ Back to home  ](<../index.html>)
