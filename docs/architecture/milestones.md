---
title: Milestones
source: website/docs/architecture/milestones.html
migrated: TASK-DOCS-002
---

## The horizontal timeline

Five phases, five compliance gates, five module-count milestones. The slope changes after P3: P0-P3 is internal-first execution; P4 is the external-GA arc.

| Phase | Theme | Exit milestone | Modules added | Headcount | Gate | Gate deliverables |
|---|---|---|---|---|---|---|
| P0 | Foundations | P0 exit: 7/22 | AUTH, AI, MCP, OBS, CHAT, memory, GENIE/CUO | 10 | T1 Floor | A05 DPIA; DPO; Trust Center; Stripe SAQ-A; VPAT 2.5 |
| P1 | Productivity | P1 exit: 15/22 | +8: PROJ, TIME, CRM, KB, HR, EMAIL, REW, LEARN | 12 | T2 base | SOC 2 Type I; CSA STAR L1; AI-CAIQ; DSAR APIs |
| P2 | Operations | P2 exit: 17/22 | +2: INV, ESOP | 14 | T2 EU enterprise | SOC 2 Type II; ISO 27001:2022; CSA STAR L2; EU AI Act Annex III section 4 |
| P3 | SaaS-ready | P3 exit: 19/22 | +2: RES, OKR | 16 | T3 large enterprise | ISO 42001 AIMS; ISO 27701; Singapore HoldCo flip (if ARR >= $1.5M) |
| P4 | Client-facing | P4 GA: 22/22 (gate at P4 mid) | +3: DOC, PORTAL, TEN | 20 | T3+ regulated | TX-RAMP; StateRAMP Cat 2; FedRAMP 20x; eIDAS QTSP |

### Phase-exit gate sequence - what "P-N exit" actually means

- T-30 days: the Eng Lead runs a ready-check for the phase through CUO (CTO skill); the CP module enumerates the compliance backlog (for example open: 12, closed: 88, blocked: 0), CUO digests what is missing, and engineering ships the remaining items.
- T-7 days: the DPO runs a pre-audit DPIA review; the auditor (SOC 2 / ISO) gets dry-run read-only access and reports severity-tiered findings to the Founder.
- T0: the Founder declares phase exit in CP; CP triggers the formal audit window and, for P2+ phases, routes Board approval. On approval, phase N+1 is unlocked and CP publishes an achievement memo via CUO.

### Module rollout schedule

| Module | Phase | Build start | Duration |
|---|---|---|---|
| AUTH | P0 | 2026-06 | 60 d |
| AI Gateway | P0 | 2026-06 | 60 d |
| MCP Gateway | P0 | 2026-06 | 60 d |
| OBS | P0 | 2026-07 | 60 d |
| CHAT | P0 | 2026-07 | 60 d |
| memory | P0 | 2026-06 | 90 d |
| GENIE / CUO | P0 | 2026-07 | 60 d |
| PROJ | P1 | 2026-09 | 90 d |
| TIME | P1 | 2026-09 | 60 d |
| CRM | P1 | 2026-09 | 90 d |
| KB | P1 | 2026-10 | 60 d |
| HR (full) | P1 | 2026-10 | 60 d |
| EMAIL | P1 | 2026-10 | 90 d |
| REW (core) | P1 | 2026-09 | 90 d |
| LEARN | P1 | 2026-11 | 60 d |
| INV | P2 | 2026-12 | 90 d |
| REW (full pool calc) | P2 | 2026-12 | 60 d |
| ESOP | P2 | 2027-01 | 90 d |
| RES | P3 | 2027-03 | 90 d |
| OKR | P3 | 2027-04 | 60 d |
| Singapore HoldCo flip (milestone) | P3 | 2027-05 | - |
| DOC (eIDAS QTSP) | P4 | 2027-06 | 180 d |
| PORTAL | P4 | 2027-08 | 180 d |
| TEN (tenancy + billing) | P4 | 2027-08 | 240 d |
| First external tenant (milestone) | P4 | 2027-12 | - |

## Foundations (P0)

The infrastructure plane plus the substrate (memory), the catalog (SKILL - already shipped) and the orchestrator (CUO). At P0 exit, Slack and Zalo are decommissioned; CyberSkill's 10 Members work inside CyberOS only.

#### Modules added (7 of 22)

- memory - universal memory; 3-layer; already shipped
- GENIE/CUO - orchestrator; 5 of 47 C-suite persona workflows live (221 workflows total)
- AUTH - OAuth 2.1 + JWT + per-tenant authz server
- AI Gateway - LiteLLM + Bedrock + redaction + cost ledger
- MCP Gateway - 2025-11-25 spec + tool registry
- OBS - LGTM stack + LangSmith for CUO sessions
- CHAT - message + thread + @genie call-out

#### Compliance gate at exit: T1 Floor

- A05 DPIA filed with MoPS
- DPO designated (Founder)
- Trust Center live at `trust.cyberos.world`
- Stripe SAQ-A AOC published
- VPAT 2.5 INT (accessibility) filed
- PDPL Art. 38 SME grace-period flag set

#### Success criteria

- All 10 internal Members on CyberOS for chat + memory by P0 exit
- Slack + Zalo billing terminated by P0 exit
- CUO answers >= 50 questions/week with >= 98% citation rate (N(FR pending))
- memory search p95 <= 250 ms on a 50k-chunk corpus
- Zero compensation/equity facts ingested into memory (denylist DEC-036)
- p95 GraphQL <= 400 ms across all subgraphs (N(FR pending))

#### Risks (likelihood x impact)

- R-001 (M x H) - Anthropic Skills spec churns; the CUO loadout breaks. Mitigation: schema-pin + conformance tests.
- R-002 (M x H) - MCP 2025-11-25 spec evolves before P0 exit. Mitigation: dual-spec compat for 1 phase.
- R-003 (L x M) - Bedrock Singapore region capacity issue. Mitigation: Anthropic ZDR fallback already wired.
- R-004 (L x M) - Vietnamese embedding quality regression. Mitigation: parity tests vs the BGE-M3 baseline.

#### Key milestones within P0

- P0 start: module template + Federation router + design tokens repo live
- P0 slice 1: AUTH + AI Gateway in beta; memory + SKILL already-shipped baseline
- P0 slice 2: MCP Gateway live; CUO router-only mode
- P0 exit: CHAT live; OBS LGTM stack live; 7/23 modules ready; Trust Center launched; PLUGIN scaffold + 8 FRs at 10/10 (cross-runtime distribution to Claude Code / Cursor / Cowork / Codex CLI)

## P0 -> P1 descope gate

Mandatory; runs at P1 start.

Every plan that adds modules monotonically becomes a death march. CyberOS must have explicit language for "this module is descoped to P2 because of P1 reality." The descope gate runs at P1 start (after P0 exit is declared) and asks four questions; if any of them score Red, two P1 modules MUST be deferred to P2 before the phase commits.

### Gate questions (all four scored Green / Amber / Red at P1 start)

1. Did P0 exit ship clean? All 5 P0 modules (AI Gateway, OBS, AUTH stub, MCP Gateway, CHAT) at `status: shipped`; Trust Center live; SOC 2 readiness signal positive; 0 cross-tenant leak incidents in P0. Amber = 1 module slipped to P1; Red = 2+ modules slipped, or any incident.
2. Is CHAT decommission >= 0.95? A 14-day rolling decommission signal: how much of CyberSkill's internal chatter is in CHAT vs Slack/Zalo. Amber = 0.85-0.94; Red = below 0.85.
3. Is the AI Gateway cost-of-everything gate fully operational? TASK-AI-001..005 shipped and audited; 0 budget breaches; cache hit rate >= 30%. Amber = 1 of the 5 FRs deferred; Red = 2+ deferred, or any budget breach.
4. Is the headcount ramp on track? 10 -> 12 hires by P1 start. Amber = 1 hire late by <= 30 days; Red = 1+ hire late by more than 30 days, or any hire pulled.

### Descope rules (if any Red)

- First descope candidate: LEARN. Defer to P2 entirely. The promotion review through Hội đồng Chuyên môn needs at least one quarter of TIME data anyway; running it in P1 was always optimistic.
- Second descope candidate: HR split. Split into HR-roster (keep at P1, slice 1 of 3) and HR-full (defer to P2). HR-roster covers the member directory + 5 contract types + leave; HR-full covers the onboarding orchestrator + performance signals.
- Third descope candidate (only if 2 Reds): EMAIL slice 3. Keep Stalwart core + shared inbox (slices 1+2); defer Genie draft + bulk send (slice 3) to P2.
- Never descope these: PROJ slice 1, TIME, CRM slice 1, KB slice 1, REW core. These are load-bearing for revenue + payroll + dogfood.

### Decision protocol

1. Founder + CTO + CHRO run the 4-question scorecard within 7 days of P0 exit.
2. If any Red: descope 1 module per Red, in the order above.
3. The descope decision MUST be recorded as a memory audit row at `memories/decisions/p0-p1-descope-.md`.
4. Descoped modules carry status `deferred` with the new target phase; they reappear in BACKLOG.md at the new position.
5. Re-engaging a descoped module requires explicit phase-entry re-evaluation; it does not drift back into P1 silently.

Why this gate exists: the P1 module batch (8 modules in ~3 phases of build time) is realistic only if the P0 infrastructure ships clean AND hires arrive on schedule. Both are 60-percent-confidence bets at best. Having an explicit "descope which two" gate written into the plan converts "death march" failure into "graceful descope" survival. The research review (section 1.3) names this as the single biggest sequencing protection CyberOS has not yet written down.

## Internal productivity (P1)

The productivity moat. PROJ + TIME + CRM + KB + HR + EMAIL + REW + LEARN - eight modules that turn the platform from "infrastructure" into "the thing the team uses every day." First payroll cycle, first promotion review through Hội đồng Chuyên môn. Note: this 8-module count is the maximum; the P0 -> P1 descope gate above may reduce it to 6 or 7 depending on the P0 exit scorecard.

#### Modules added (+8 = 15 of 22)

- PROJ - projects, tasks, cycles, assignments
- TIME - time entries, expense tracking
- CRM - clients, deals, activities
- KB - knowledge base, canonical docs
- HR (full) - Members, roles, leave, onboarding
- EMAIL - Stalwart-based mail + inbox + i18n RFC 6532
- REW (core) - 3P income, payslip, BP balance
- LEARN - career path, Hội đồng peer-review

#### Compliance gate: T2 base

- SOC 2 Type I issued (point-in-time)
- CSA STAR L1 self-assessment via CAIQ v4.0.3
- AI-CAIQ "Valid-AI-ted" badge
- DSAR APIs end-to-end (GDPR Art. 15 ready)
- Article 50 transparency badges in all AI surfaces

#### Headcount

10 -> 12 Members:

- +1 HR/Ops Lead (REW dogfooding)
- +1 Engineer (PROJ + KB owner)
- Existing Founder/CEO + Eng Lead + 7 Engineers continue

#### Success criteria

- First full payroll cycle issued through REW (month-end at P1 exit)
- First promotion review through Hội đồng Chuyên môn
- P1 base salary invariant verified: zero system-reductions (N(FR pending))
- BP balance tracked with anti-inflation interest at the ACB rate
- EMAIL daily summary in memory for every Member
- CRM activities auto-logged from EMAIL outgoing (CaMeL safe extraction)

#### Risks

- R-101 (M x H) - REW parameter versioning bug; a retroactive recompute could violate N(FR pending). Mitigation: anti-retroactive CI gate; replay tests on every release.
- R-102 (M x H) - EU AI Act Annex III section 4 over-application; HR features flagged as high-risk. Mitigation: DEC-054 - drafts only, no scoring.
- R-103 (M x M) - Stalwart self-hosted EMAIL reliability. Mitigation: fallback SMTP relay; OBS alarms on bounce rate.
- R-104 (L x M) - SOC 2 Type I auditor unavailability in Vietnam. Mitigation: Schellman, A-LIGN shortlist; remote audit option.

## Operations (P2)

Bill-to-cash and Phantom Stock. INV closes the revenue loop; ESOP closes the equity-honour loop. First SP grant issued; first annual SP valuation cycle complete.

#### Modules added (+2 = 17 of 22)

- INV - invoicing, MST validation, VAT e-invoice via GDT T-VAN, monthly VAT filing
- ESOP - Phantom Stock, 4-year vesting, put options from Year 3, annual SP valuation
- REW upgrade: full pool calculation with BP overflow + sabbatical accrual

#### Compliance gate: T2 EU enterprise

- SOC 2 Type II issued (6-mo observation window)
- ISO/IEC 27001:2022 Stage 1 audit complete
- CSA STAR L2 third-party attestation
- EU AI Act Annex III section 4 conformity pack for REW + LEARN
- Decree 13 full regime - graduate from SME

#### Headcount

12 -> 14 Members:

- +1 vCISO (fractional security; cert prep)
- +1 Account Manager (CRM + INV dogfooding)

#### Success criteria

- First SP grant issued (after Board-approved valuation)
- First annual SP valuation cycle complete
- First e-invoice filed to GDT successfully
- Monthly VAT return submitted via the `vietnam-tax-filing` skill (planned)
- ARR > $300k (internal-equivalent or first design partner)
- Good Leaver / Bad Leaver branch tested via tabletop exercise

#### Risks

- R-201 (L x VH) - ESOP put-option model violates Vietnamese tax interpretation. Mitigation: counsel review at every parameter version; cash-collected pool only.
- R-202 (M x H) - SP valuation methodology challenged by the Board. Mitigation: dual-signed by Founder + Board; methodology in the CP module.
- R-203 (M x M) - GDT T-VAN provider connectivity. Mitigation: multiple T-VAN provider integrations; manual fallback via the VN-tax-filing skill.

## SaaS readiness (P3)

The platform earns the right to sell. RES + OKR ship; capacity planning becomes visible; the first quarterly OKR cycle closes. If ARR >= $1.5M, the Singapore HoldCo flip happens.

#### Modules added (+2 = 19 of 22)

- RES - resource plan, capacity vs forecast, CUO COO-skill rebalancing suggestions
- OKR - objectives, key results, quarterly cycle, CUO CEO/CSO-skill cycle-close prompts
- Mobile app evaluation - RN-based, P3 stretch

#### Compliance gate: T3 large enterprise

- ISO/IEC 42001 (AIMS) certified
- ISO/IEC 27701 (PIMS) - if EU/UK customers push
- SOC 2 Type II annual refresh
- Singapore HoldCo flip (CyberSkill Pte Ltd as parent; if ARR >= $1.5M)

#### Headcount

14 -> 16 Members:

- +1 Designer (mobile evaluation + PORTAL prep)
- +1 Engineer (TEN module groundwork)

#### Success criteria

- First full quarterly OKR cycle closed (Q1 2027)
- Capacity-vs-forecast rebalancing run weekly via CUO
- 3 design-partner tenants signed (paid pilot)
- ARR >= $1.5M triggers the Singapore HoldCo flip
- eu-shard activated for at least one EU pilot tenant

#### Risks

- R-301 (M x H) - Singapore HoldCo flip legal complexity exceeds the phase window. Mitigation: counsel engaged at P2 exit; the flip is an optional gate.
- R-302 (M x H) - RES rebalancing flagged as employment-decision high-risk. Mitigation: suggestion-only mode; human accept/reject required.
- R-303 (M x M) - ISO 42001 AIMS audit window misalignment. Mitigation: Stage 1 readiness check at P3 start.

## Client-facing (P4)

External GA. DOC + PORTAL + TEN close the gap. First external paying tenant onboarded. Multi-tenant external GA opens. Regulated-commercial path open via TX-RAMP, StateRAMP Cat 2, FedRAMP 20x Moderate (no-sponsor route if a US sub exists).

#### Modules added (+3 = 22 of 22)

- DOC - document signing, eIDAS QTSP integration, PDF/A-2 archival
- PORTAL - external client portal, approval workflow, client-facing AI answers via the CUO CXO skill
- TEN - tenancy management, per-tenant config, billing (Stripe + VNPay)

#### Compliance gate: T3+ regulated

- TX-RAMP (Texas state)
- StateRAMP Cat 2
- FedRAMP 20x Moderate (no-sponsor route if a US sub exists)
- eIDAS QTSP for the DOC module - EU-compliant qualified e-signature
- SOC 2 Type II + ISO 27001 + ISO 42001 all annual-refresh certified

#### Headcount

16 -> 20 Members:

- +2 Customer Engineers (onboard external tenants)
- +1 Sales (Account Manager promoted to head)
- +1 Legal Counsel (CLO; replaces CUO-as-CLO)

#### Success criteria

- First external paying tenant onboarded by P4 mid
- 5 external paying tenants by P4 late
- 10 external paying tenants by P4 GA -> ARR >= $3M
- NPS >= 40 from external tenants
- Zero tenant data leakage incidents (N(FR pending) maintained)
- First eIDAS QTSP-signed document issued

#### Risks

- R-401 (M x VH) - eIDAS QTSP integration partner unreliable. Mitigation: multiple QTSP integrations; degraded mode = advanced e-signature.
- R-402 (M x H) - first external tenant onboarding takes 4x expected. Mitigation: the TEN module ships >= 3 months before the first paid customer.
- R-403 (L x VH) - FedRAMP 20x no-sponsor route deprecated. Mitigation: TX-RAMP + StateRAMP first; FedRAMP deferred.

## Module dependency graph

Every module depends on the P0 infrastructure plane (AUTH, AI, MCP, OBS) and on memory for memory + audit. P1 modules dogfeed each other (TIME -> REW; CRM -> EMAIL); P2 modules close the revenue + equity loops; P3 modules read from everything; P4 modules sit at the edge.

Key edges:

- AUTH -> every functional module (CHAT, PROJ, TIME, CRM, KB, HR, EMAIL, REW, LEARN, INV, ESOP, RES, OKR, DOC, PORTAL, TEN)
- AI -> CUO + memory; MCP -> CUO; OBS observes CHAT, PROJ, CRM, REW, ESOP, RES, OKR
- CUO -> CHAT + memory; memory -> CUO
- PROJ -> TIME -> REW; HR -> REW; REW -> LEARN and ESOP
- CRM -> EMAIL and INV; EMAIL -> memory; KB -> memory
- PROJ + TIME -> RES; HR + RES -> OKR
- CRM -> PORTAL; HR + AUTH + INV -> TEN; KB -> DOC

Cycle-free by construction. The only "circular" arrow (memory <-> CUO) is the legitimate read/write split: CUO writes inquiry context, memory returns search hits.

## Headcount, modules, revenue trajectory

Three curves on the same time axis. Headcount grows only when CyberOS itself absorbs the operational load: hires are gated on CyberOS's ability to absorb that Member's onboarding via REW + LEARN + KB (10 -> 20, +10 over P0 -> P4). Module count is the leading indicator (7 -> 22, +15 over P0 -> P4; the 7 P0 modules are the heaviest lift, and subsequent phases ship 8 / 2 / 2 / 3). Revenue is the trailing indicator ($0 at P0, internal only; design-partner pilots at P2; the $1.5M HoldCo-flip trigger at P3; $3M+ at P4 with 10 paying tenants).

| Milestone | Members | Modules shipped | ARR ($k) |
|---|---|---|---|
| P0 start | 10 | 3 | 0 |
| P0 exit | 10 | 7 | 0 |
| P1 exit | 12 | 15 | 0 |
| P2 exit | 14 | 17 | 300 |
| P3 exit | 16 | 19 | 800 |
| P4 early | 17 | 19 | 1,500 |
| P4 mid | 18 | 20 | 2,000 |
| P4 late | 19 | 22 | 2,600 |
| P4 GA | 20 | 22 | 3,200 |

## KPI dashboard targets per phase

Each phase has a "north-star plus three" KPI set. The north-star is the dogfooding-signal proxy; the three supporting metrics are the leading-edge measurements that tell the founder whether the phase is ready to exit.

| Phase | North-star | KPI 2 | KPI 3 | KPI 4 (guardrail) |
|---|---|---|---|---|
| P0 | Slack + Zalo decommissioned; 100% of Members on CHAT | CUO citation rate >= 98% | memory search p95 <= 250 ms | Zero compensation in memory (denylist) |
| P1 | First full payroll cycle issued through REW | SOC 2 Type I issued | Time-tracked hours = 100% of billable | P1 base salary system-reductions = 0 |
| P2 | First SP grant + first SP valuation cycle complete | SOC 2 Type II + ISO 27001 Stage 1 | VAT e-invoice file rate 100% | Parameter version retroactive mutations = 0 |
| P3 | Quarterly OKR cycle closed; capacity rebalanced weekly | ISO 42001 certified | ARR >= $1.5M (HoldCo trigger) | EU AI Act Annex III section 4 conformance; 100% drafts-only |
| P4 | First external paying tenant onboarded | 10 paying tenants by P4 GA | NPS >= 40 from external tenants | Tenant data leakage incidents = 0 |

#### Continuous (cross-phase) NFRs

- N(FR pending) - GraphQL p95 <= 400 ms
- N(FR pending) - memory search p95 <= 250 ms on 1M chunks
- N(FR pending) - platform availability >= 99.5%
- N(FR pending) - CHAT availability >= 99.9%
- N(FR pending) - tenant data leakage = 0
- N(FR pending) - CUO citation rate >= 98%

#### Anti-metrics (watched to not grow)

- CUO auto-acts on an irreversible op without confirm
- P1 base salary system-reductions
- Parameter version retroactive mutations
- Compensation/equity facts in memory
- CUO answers without citation when a source exists
- Prompt-injection exfiltration via email/document

## References

#### Strategy source sections

- The phased milestone arc
- OKRs by phase exit
- Guardrail (anti-)metrics
- All 23 modules catalogued
- Foundational locked decisions
- Non-Functional Requirements
- Compliance tier model per phase
- AI-driven productivity matrix
- Phased shipping plan

#### Cross-references in CyberOS docs

- [Infrastructure plane (six P0 pillars)](infrastructure.html)
- [Compliance plan (three rings)](compliance.html)
- [Tech stack (8 tiers)](tech-stack.html)
- [memory module](../modules/memory/index.html)
- [CUO module](../modules/cuo/index.html)
- [Skill module](../modules/skill/index.html)
- [NFR Catalog (full table)](../reference/nfr-catalog.html)
- [Risk Register (full)](../reference/risk-register.html)

## Changelog

History lives in the [changelog](../reference/changelog.html); this page describes only the current state.
