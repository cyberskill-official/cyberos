---
title: Compliance — CyberOS
source: website/docs/architecture/compliance.html
migrated: FR-DOCS-002
---

0

## Three rings · one platform

CyberOS is built in Vietnam, sold globally, and processes both employee and (eventually) client data. The compliance posture therefore satisfies three concentric rings: (1) Vietnamese law for the entity that owns the platform, (2) cross-border requirements for clients in Singapore, EU, US, (3) sectoral standards (ISO/SOC/CSA/AI-CAIQ) that procurement teams at mid-market clients require before signing. Each ring lights up gate-by-gate as the platform crosses phase boundaries. 

Ring 3 · International standards ISO/IEC 27001:2022 · SOC 2 (Type I → II) · ISO 42001 AIMS · ISO 27701 PIMS · CSA STAR L1 → L2 · AI-CAIQ Ring 2 · Cross-border GDPR · EU AI Act (Annex III §4) · Singapore PDPA · HoldCo flip Ring 1 · Vietnam — Home Regime PDPL Law 91/2025 · Decree 356 Decree 13/2023 (Personal Data) Decree 53/2022 (Cybersecurity) PDPL Art. 38 (SME grace) NQ 142/2024 + TT 80/2021 (Tax) cornerstone · every deployment ⌖ P3 exit · ISO 42001 (AIMS) certified ⌖ P3 exit · ISO 27701 (PIMS) — if EU/UK push ⌖ P2 exit · ISO 27001:2022 Stage 1 ⌖ P1 exit · SOC 2 Type I + CSA STAR L1 + AI-CAIQ ⌖ P2 exit · SOC 2 Type II + CSA STAR L2 ⌖ P2 exit · EU AI Act Annex III §4 conformity pack (REW + LEARN) ⌖ P3+ · Singapore HoldCo flip (if ARR ≥ $1.5M) ⌖ P3+ · GDPR posture (EU-shard residency) ⌖ P4 · External Authorised Reps (EU/UK) ⌖ Ongoing · DSAR APIs + 30-day SLA ⌖ P0 exit (P0 · exit) A05 DPIA filed · DPO designated · Trust Center live Stripe SAQ-A AOC · VPAT 2.5 INT

Ring 1 · cornerstone

Vietnam home regime

Every CyberOS deployment respects PDPL Law 91/2025 + Decree 356, Decree 13/2023, Decree 53/2022, PDPL Art. 38 (SME grace), and the tax framework. Non-negotiable.

Ring 2 · cross-border

Where customers are

GDPR for EU-resident clients, EU AI Act Annex III §4 because REW + LEARN are high-risk-adjacent, Singapore PDPA because of the HoldCo flip at P3.

Ring 3 · standards

What buyers ask for

ISO 27001:2022, SOC 2 Type II, ISO 42001 (AIMS), ISO 27701 (PIMS), CSA STAR L1→L2, AI-CAIQ. The procurement checklist gets shorter with each cert.

1

## Ring 1 — Vietnam home regime

Vietnamese law is the cornerstone. CyberSkill JSC is a Vietnam-incorporated entity processing Vietnamese-citizen personal data. Every architectural decision in — RLS-by-default, per-tenant region pinning, Merkle-chained audit, A05 filings, mandatory DPO — exists to satisfy Vietnam first. documents the obligations; this section traces each regulation to a CyberOS module and FR. 

🇻🇳

### Decree 13/2023/NĐ-CP — Personal Data Protection

in force 2023-07-01

Vietnam's first comprehensive personal data protection regime. Introduces "sensitive personal data" (health, biometric, financial, criminal-record, ethnic, religious, sexual-orientation), requires a Data Protection Impact Assessment (DPIA) for any processing of sensitive data, mandates data subject consent for cross-border transfer, and establishes a notification regime to the Ministry of Public Security (MoPS / A05) for breaches and cross-border transfer of Vietnamese-citizen data.

#### Key obligations

  * Lawful basis required for every processing activity
  * Sensitive-data DPIA before processing (Article 24)
  * Cross-border transfer impact assessment (PDPL Art. 20 — submit one original copy to A05 within **60 days** of the first transfer; Decree 13's 15-day pre-form regime is superseded)
  * Breach notification to MoPS within 72 h (PDPL adds: data-subject notification required for biometric or financial-service incidents)
  * Data Subject Access Request (DSAR) within 30 days
  * DPO designation (PDPL Art. 38 grace period for SMEs at <10 employees / <VND 10B revenue; CyberSkill JSC qualifies through P1)
  * Outright ban on personal-data sale (PDPL Art. 7) — applies to every CRM, PORTAL, and outbound surface



#### Penalties

  * Administrative fines up to VND 100 M (~$4,000) per violation
  * Compounded by Law 91/2025 fine schedule
  * Service suspension orders for repeat violations
  * Criminal liability under Penal Code §288 for major incidents



#### CyberOS response (locked in §8)

  * **vn-shard Postgres** with RLS for all Vietnamese-origin PD; Singapore region default, Vietnam-resident shard available from P2.
  * **DPIA template** as KB artifact; DPO (Founder until P3) signs before any new data category is ingested.
  * **A05 cross-border-transfer impact assessment** in the Compliance subgraph; PDPL Art. 20 60-day post-audit submission auto-generated; one original copy per transfer activity.
  * **Sensitive-tagged facts** in memory Layer 2 are encrypted-at-rest with per-tenant KMS; never written to Layer 3 without explicit consent.
  * **memory ingestion denylist (DEC-036)** structurally excludes compensation, equity, government IDs, bank accounts, and health data.



🛡

### Decree 53/2022/NĐ-CP — Cybersecurity Law implementing decree

in force 2022-10-01

Obliges in-scope services — those that store user data of Vietnamese citizens for at least P0 → P4 horizon — to maintain data on Vietnamese soil and maintain an in-country office. CyberOS, while sold to a Vietnamese entity, is unlikely to qualify as in-scope until the platform processes data for Vietnamese client end-users (P4 PORTAL).

#### Trigger criteria (in-scope test)

  * Collects Vietnamese-citizen personal info
  * Stores said data for ≥ P0 → P4 horizon
  * Offers services to ≥ 100k Vietnamese users _or_ revenue ≥ VND 100 B
  * Receives data-localisation order from MoPS



#### CyberOS response

  * Employee data on Vietnamese soil from P2 onward (DEC-027)
  * PORTAL designed so VN-client-facing tenant defaults to vn-shard residency
  * Vietnam office maintained — CyberSkill JSC HCMC HQ
  * Annual review with VN counsel at every phase gate



🧮

### PDPL Art. 38 — SME grace period

in force 2026-01-01 (PDPL Law 91/2025)

Law 91/2025/QH15 Article 38 ("Quy định chuyển tiếp cho doanh nghiệp nhỏ và vừa") provides a transitional regime for small and medium-sized enterprises that defers certain obligations — most notably the formal DPO appointment and the dedicated DPIA team requirement — until the entity crosses the SME threshold (10 employees / VND 10 B revenue / processing > 100,000 data subjects). CyberSkill JSC currently qualifies as a micro-enterprise under this article.

**DEC-053 (revised 2026-05-15):** CyberSkill JSC operates under PDPL Art. 38 grace-period regime in P0–P1 (Founder serves as DPO; informal DPIA). At P2 entry, CyberSkill graduates pre-emptively to the full PDPL regime — formal DPO appointment, registered processing activities, formal DPIA — regardless of whether the SME threshold has been crossed. Hiring a formal DPO at P0 is a $50–80k/year cost the regulator does not require for an entity at this stage.

  * **P0–P1 (P0 · start–P1 · exit):** Art. 38 grace period; Founder serves as DPO; DPIA informal but maintained.
  * **P2 (P2 · start):** graduate preemptively; appoint formal DPO; run formal DPIA; register processing activities with MoPS A05.
  * **Compliance module (CP):** tracks Art. 38 grace-period flag per tenant; one-click graduation UX for tenants who buy CyberOS.



⚖️

### Law 91/2025/QH15 + Decree 356/2025 — PDPL elevation

in force 2026-01-01

Vietnam's Personal Data Protection Law (PDPL) — Law 91/2025 elevates Decree 13's decree-level obligations into national law with significantly higher penalties. Decree 356/2025 is the implementing decree, locking the DPO requirement, mandatory A05 filings, and the breach-notification 72-hour clock.

#### What's new vs Decree 13

  * Fine ceiling raised to 5% of preceding-year revenue
  * DPO becomes a national-law requirement (not just decree-level)
  * Cross-border-transfer impact assessment formalised
  * Civil right of action by data subjects
  * "Right to explanation" for automated decisions (parallels EU AI Act)



#### CyberOS response

  * DEC-055: every CUO output that touches a person ships with "Why this?" affordance (persona version + memory citations) — implements right-to-explanation by design
  * CP module's Compliance Cockpit shows live PDPL conformance per tenant
  * Per-tenant DPIA template auto-pre-fills with subgraph-declared data categories
  * Breach notification timer wired to OBS module 



💰

### NQ 142/2024 + TT 80/2021 — VAT & e-invoice framework

in force 2022 / 2024 amend.

Mandatory e-invoice issuance, monthly VAT declaration, MST (tax code) validation on every invoice line. CyberOS handles this via the INV module (P2) and the `vietnam-mst-validate` \+ `vietnam-vat-invoice` skills (shipped). `vietnam-tax-filing` for the monthly VAT return is planned (not yet in the 5-skill public collection).

  * **e-invoice format:** XML (Decree 123/2020), uploaded to GDT (General Department of Taxation) via T-VAN provider.
  * **MST validation:** 10-digit (entity) or 13-digit (branch); checksum via Modulus-11 algorithm; live GDT API lookup.
  * **VAT rates:** 0% (export), 5% (essentials), 8% (NQ 142/2024 stimulus, expires periodically), 10% (default).
  * **Monthly filing:** by 20th of following month via the `vietnam-tax-filing` skill (planned).
  * **Retention:** 10 years (Law on Accounting Article 41).



### Vietnam traceability — regulations → CyberOS modules + FRs

Regulation| Obligation| CyberOS module| FR / DEC| Phase  
---|---|---|---|---  
Decree 13/2023 Art. 24| DPIA before sensitive-data processing| CP (Compliance)| (FR pending) · DEC-053| P0  
Decree 13/2023 Art. 14| DSAR within 30 days| CP + AUTH| (FR pending)| P1  
Decree 13/2023 Art. 28| Cross-border-transfer A05 notification| CP| (FR pending)| P0  
Decree 13/2023 Art. 23| 72-hour breach notification| OBS + CP| (FR pending)| P0  
Decree 53/2022 Art. 26| Data localisation for in-scope services| (Infra) · per-tenant region pinning| DEC-009 · DEC-027| P2  
PDPL Art. 38| SME grace-period tracking + graduation| CP| DEC-053 (rev. 2026-05-15)| P0  
Law 91/2025 + Decree 356| National-law DPO, fines 5% revenue| CP · HR (DPO role)| (FR pending)| P2  
Law 91/2025 right-to-explanation| CUO output explainability| AI + CUO| DEC-055| P0  
Penal Code §288| Criminal liability prevention| (Infra) · audit chain| N(FR pending)| P0  
Law on Accounting Art. 41| 10-year retention| INV + memory archival| DEC-020| P2  
NQ 142/2024 + TT 80/2021| e-invoice, MST, monthly VAT filing| INV + Skill (3 VN skills)| (FR pending)..050| P2  
Labour Code 2019| SI/PIT remittance, payslip retention| REW| (FR pending)..080| P1  
  
2

## Ring 2 — Cross-border

Cross-border obligations are triggered by where the data subject is, not where CyberSkill is incorporated. CyberOS does not actively process EU data subjects in P0–P2 (all employees are Vietnamese, most clients are SEA), but P3 multi-tenant readiness opens the door. Three regimes matter: GDPR for EU residents, the EU AI Act for HR/REW/LEARN flows that touch employment-decision territory, and Singapore PDPA for the HoldCo flip strategy. 

🇪🇺

### GDPR (Regulation EU 2016/679)

in force 2018-05-25

Triggered when CyberOS processes EU-resident personal data — P3 onward via the eu-shard. The multi-tenant architecture already supports per-tenant region pinning to eu-central-1 with Bedrock EU endpoints.

#### Obligations

  * Lawful basis tracking (Art. 6 + 9)
  * DSAR within 1 month (Art. 12 + 15)
  * Right to erasure with downstream propagation (Art. 17)
  * 72-hour breach notification to DPA (Art. 33)
  * DPIA for high-risk processing (Art. 35)
  * DPA contract template for every customer
  * EU/UK Authorised Reps when no EU establishment (Art. 27)



#### CyberOS posture

  * eu-shard via AWS Frankfurt at P3
  * DSAR surface via CP module
  * Right-to-erasure propagates: memory UPDATE → DELETE through Layer 2, timestamped tombstone in Layer 3
  * 72-h breach timer wired to OBS
  * DPA template auto-attached to every tenant contract



**Penalty:** up to €20 M or 4% of global turnover, whichever is higher.

🧭

### EU AI Act (Regulation EU 2024/1689) — Annex III §4 focus

in force 2025-08-01 · obligations 2026-08-02

Tiers AI systems into four risk categories: prohibited, high-risk, limited-risk, minimal-risk. CUO's default classification across CyberOS modules is **limited-risk** (Article 50 transparency only — disclose AI interaction). Two specific module integrations are **high-risk-adjacent** and need explicit boundary work.

**DEC-054 locked decision:** No CyberOS AI feature, in any module, in any phase, produces a number or grade that ranks, scores, or classifies a person without a human-in-the-loop review on the same surface. Drafts and summaries are permitted; rankings and scores about people are forbidden.

#### Annex III §4 — employment-decision high-risk

Annex III §4 covers "AI systems intended to be used for the recruitment or selection of natural persons, in particular for placing targeted job advertisements, screening or filtering applications, evaluating candidates" — and "evaluating performance, work behaviour or personal traits."

Module| High-risk-adjacent flow| CyberOS mitigation  
---|---|---  
**HR**|  Offer-letter drafting, 1:1 prep, onboarding checklists| Drafts only · human writes the decision · CUO never assigns a score  
**REW**|  Payslip narrative explainer, anomaly surfacing| Read-only narrative · "payslip_explain" tool annotated read-only · compute path is deterministic SQL, not LLM  
**LEARN**|  Career-path next-step suggestion, Hội đồng peer-review summariser| Outcomes-only summaries · no individual scoring · Hội đồng (human council) issues the decision  
**PROJ**|  Cycle-review draft generation, blocker detection| Drafts/anomalies only · the human owner produces the final evaluation  
**RES**|  Capacity-vs-forecast rebalancing suggestion| Suggestion only · Question mode · Engineering Lead accepts/rejects  
  
Article 50 transparency: every AI-touched UI surface carries a small persistent badge (model, persona version, intervention mode) — Notify = ochre, Question = umber, Review = bronze ( Design System). Satisfies the transparency obligation by visible design, not buried disclosure.

**Penalty:** €35 M or 7% of global turnover (prohibited practices); €15 M or 3% (high-risk non-compliance).

🇸🇬

### Singapore PDPA + the HoldCo flip

PDPA in force 2014 · 2020/2021 amend.

Singapore's PDPA is similar in structure to GDPR but more permissive on cross-border transfer (no "adequacy-equivalent" regime required). The strategic value of Singapore is the **HoldCo flip** : at P3 (month 10–12), if ARR ≥ $1.5 M, CyberSkill incorporates a Singapore parent (a private limited "Pte Ltd") and the Vietnamese entity becomes a wholly-owned subsidiary.

#### Why flip

  * Easier fundraising (USD-denominated, well-known to global VCs)
  * IP holding company → cleaner exits
  * Dividend flexibility
  * Talent equity (ESOP without Vietnamese SP-tax friction)
  * SOX/PCAOB pathway if US-listed later



#### Technical posture supporting the flip

  * Singapore-region default for shared infrastructure (ap-southeast-1)
  * Every data-model entity tagged with `legal_entity_owner` field
  * Post-flip migration is a tag flip + CRDT-style audit record, not data movement
  * IP licences maintained inside the company; JSC → Pte Ltd assignment is a one-page doc



3

## Ring 3 — International standards

Procurement teams at mid-market clients require these certs before signing. The compliance ladder is locked in DEC-011: **SOC 2 → ISO 27001 → ISO 42001** , with CSA STAR and AI-CAIQ layered alongside. Each standard adds a phase-gated effort; the architectural choices in mean the controls themselves are already satisfied — the work is documentation + audit. 

🛡

### ISO/IEC 27001:2022 — Information Security Management

target · P3 Stage 1 · P4 full

The 93 Annex A controls in the 2022 revision are largely satisfied by: encryption at rest (A.10.1), key management (A.10.2), access control by least privilege (A.9.1–A.9.4), change management (A.12.1), logging and monitoring (A.12.4), incident management (A.16.1), supplier relationships (A.15.1). The gap-list to certification readiness is documented in OBS module.

  * **Timeline:** Stage 1 audit at P3 (P3 · exit), full certification at P4 (P4 · mid)
  * **Scope:** CyberOS platform + CyberSkill JSC organisation
  * **Auditor:** shortlist Schellman, A-LIGN, KPMG VN
  * **Key controls already satisfied:** RLS (A.9), mTLS in cluster + HTTPS external (A.13), Merkle-chained audit (A.12.4), tenant KMS (A.10.2)



📋

### SOC 2 Type I → Type II

target · P1 Type I · P2 Type II

Trust Service Criteria (TSC) covered: **Security** (mandatory), **Availability** , **Confidentiality**. Privacy and Processing Integrity added when client demand justifies them.

  * **Type I (point-in-time):** at P1 exit (P1 · exit) — "controls are designed effectively"
  * **Type II (operating window):** at P2 exit (P2 · exit) — six-month minimum observation window starting at month 4
  * **Reusable for sub-processor disclosure:** CyberOS publishes a SOC 3 (Type II public summary) at Trust Center



🤖

### ISO/IEC 42001:2023 — AI Management System (AIMS)

target · P3 exit (P3 · exit)

The world's first AI management system standard. Covers AI risk assessment, lifecycle governance, transparency obligations, third-party AI integration controls. Pairs neatly with EU AI Act Annex III §4 work — the conformity pack done at P2 for REW + LEARN feeds directly into ISO 42001 evidence.

  * AI Impact Assessment (AIIA) for every CUO skill
  * Model registry with persona-version stamps (already in CUO module)
  * Continuous monitoring of AI behaviour drift (OBS + LangSmith)
  * Third-party AI clauses in Bedrock + Anthropic + OpenAI contracts



🔐

### ISO/IEC 27701:2019 — Privacy Information Management (PIMS)

target · P3 (if EU/UK push)

Extension to ISO 27001 specifically for privacy. Useful for EU/UK consultancies pushing for one-stop GDPR + PDPA evidence. Optional at P3 — pursued only if customer demand justifies.

⭐

### CSA STAR L1 → L2 (Cloud Security Alliance)

target · P1 L1 · P2 L2

CSA's Security, Trust, Assurance, Risk (STAR) program. Level 1 = self-assessment (CAIQ questionnaire) — cheap, fast, opens many doors. Level 2 = third-party audit — required by enterprise buyers.

  * **L1 self-assessment (CAIQ v4.0.3):** ~290 questions across 17 control families. Done at P1 exit (P1 · exit) via Trust Center.
  * **L2 third-party attestation:** auditor-validated CAIQ. Done at P2 exit (P2 · exit) when SOC 2 Type II auditor is engaged.



🧪

### AI-CAIQ — "Valid-AI-ted" extension

target · P1 exit (P1 · exit)

CSA's AI-extended CAIQ — adds AI-specific control questions (training data provenance, model bias monitoring, prompt safety, persona versioning). Completed alongside L1. The persona-version stamp (DEC-035), model registry, and memory ingestion denylist (DEC-036) cover most of the AI-CAIQ surface by construction.

4

## Compliance gates per phase

Each phase exit ships a discrete bundle of compliance deliverables ( + §11.1). Without these, the phase is not "exited" — even if every module ships on time. 

How each compliance tier unlocks customer cohorts

flowchart LR P0E[("P0 exit · T1 Floor")] --> COHORT_A["SME Vietnam tenants  
(internal CyberSkill only)"] P1E[("P1 exit · T2 base")] --> COHORT_B["VN mid-market  
\+ first design partners"] P2E[("P2 exit · T2 EU")] --> COHORT_C["EU/UK B2B SaaS  
\+ enterprise procurement"] P3E[("P3 exit · T3 large")] --> COHORT_D["Regulated EU + US enterprise  
\+ Singapore HoldCo entities"] P4E[("P4 entry · T3+ regulated")] --> COHORT_E["State/local gov sub-paths  
TX-RAMP · StateRAMP · FedRAMP 20x"] P0E -.adds.-> A0["A05 DPIA · DPO · Trust Center · VPAT"] P1E -.adds.-> B0["SOC 2 Type I · CSA STAR L1 · AI-CAIQ"] P2E -.adds.-> C0["SOC 2 Type II · ISO 27001:2022 · CSA STAR L2 · EU AI Act §4"] P3E -.adds.-> D0["ISO 42001 · ISO 27701 · HoldCo flip option"] P4E -.adds.-> E0["eIDAS QTSP · TX-RAMP · FedRAMP 20x"] classDef gate fill:#e8d4c2,stroke:#45210e,stroke-width:2px classDef cohort fill:#f5ede6,stroke:#45210e classDef cert fill:#fef6e0,stroke:#9c750a class P0E,P1E,P2E,P3E,P4E gate class COHORT_A,COHORT_B,COHORT_C,COHORT_D,COHORT_E cohort class A0,B0,C0,D0,E0 cert 

timeline title CyberOS compliance gate timeline section P0 · P0 phase · T1 Floor P0 · exit: A05 DPIA filed : DPO designated (Founder) : Trust Center live : Stripe SAQ-A AOC : VPAT 2.5 INT section P1 · P1 phase · T2 P1 · exit: SOC 2 Type I issued : CSA STAR L1 self-assessment : AI-CAIQ "Valid-AI-ted" : DSAR APIs end-to-end : First payroll through REW section P2 · P2 phase · T2 EU enterprise P2 · exit: SOC 2 Type II issued : ISO/IEC 27001:2022 certified : CSA STAR L2 attestation : EU AI Act Annex III §4 conformity pack (REW+LEARN) : Decree 13 full regime (graduate from SME) section P3 · P3 phase · T3 Large enterprise P3 · exit: ISO/IEC 42001 (AIMS) certified : ISO/IEC 27701 (PIMS) — if EU/UK pushes : Singapore HoldCo flip — if ARR ≥ $1.5M : First quarterly OKR cycle closed section P4 · P4 phase · T3+ regulated P4 · mid: TX-RAMP (Texas state) : StateRAMP Cat 2 : FedRAMP 20x Moderate (no-sponsor route) : eIDAS QTSP for DOC module : First external paying tenant 

###  — Compliance tier per phase

Phase| Months| Vietnam regime| EU AI Act tier| GDPR posture| ISO 27001| SOC 2  
---|---|---|---|---|---|---  
**P0**|  1–3| PDPL Art. 38 grace| Limited-risk| Off| Gap list| —  
**P1**|  4–6| PDPL Art. 38 grace| Limited-risk| Off| Gap list| Type I prep  
**P2**|  7–9| PDPL full + Decree 13| Limited-risk · §4 boundary| Off| Pre-readiness| Type I issued · Type II prep  
**P3**|  10–12| PDPL full + Decree 13| Limited-risk + Art. 50 badges| On (eu-shard)| Stage 1 audit| Type II issued  
**P4**|  13–24| PDPL full + Decree 13| Limited-risk; HR boundary tested| On (eu-shard)| Certified| Type II  
  
5

## Trust Center pattern

Trust Center is CyberOS's public-facing compliance surface. One URL — `trust.cyberos.world/{tenant}` — serves the entire procurement Q&A in one place. Live at P0 exit; deepens at every phase gate. 

#### What ships at P0 (P0 · exit)

  * VPAT 2.5 INT (Voluntary Product Accessibility Template)
  * Stripe SAQ-A AOC (subprocessor disclosure)
  * Sub-processor list (public — N(FR pending))
  * Accessibility statement (WCAG 2.2 AA target)
  * Data residency disclosure (region per tenant)
  * Incident response runbook (DEC-027)
  * DPO contact + DPIA template download



#### What deepens by phase

  * P1: SOC 2 Type I report PDF, CAIQ v4.0.3 self-assessment
  * P2: SOC 2 Type II, ISO 27001:2022 cert, CSA STAR L2
  * P3: ISO 42001 AIMS, AI Impact Assessments per skill
  * P4: eIDAS QTSP cert, TX-RAMP, FedRAMP 20x SSP



Flow · Procurement asks "are you SOC 2?"

sequenceDiagram autonumber actor B as Buyer procurement participant TC as Trust Center participant AU as AUTH (DSAR + access) participant CP as CP subgraph participant FILES as R2 / signed PDF store B->>TC: visit trust.cyberos.world/cyberskill TC->>CP: GET /certifications CP-->>TC: { soc2_type_2: { issued: "2026-09-15", scope, link }, iso27001: {...},... } TC-->>B: render list with badges B->>TC: request SOC 2 Type II report TC->>AU: NDA gate (e-sign click-through) AU->>B: NDA pre-filled with buyer org B-->>AU: signed AU->>FILES: time-limited signed URL FILES-->>B: download PDF (24h TTL) TC->>CP: log access (N(FR pending)) 

6

## Breach notification matrix

Every regime has its own clock and its own recipient. CyberOS's OBS module wires a 72-hour breach timer triggered by audit-log anomaly or manual classification by the DPO. The CP module routes notifications to the correct authority per affected jurisdiction. 

Jurisdiction| Trigger| Authority| Window| Form| Data subject notice  
---|---|---|---|---|---  
**🇻🇳 Vietnam** | Personal data breach (Decree 13 Art. 23) | MoPS / A05 | 72 h | A05 incident form (mẫu sự cố) | "Without delay" when high risk  
**🇪🇺 EU** | GDPR Art. 33 personal data breach | Lead DPA (one-stop-shop) — for CyberOS Pte Ltd via Authorised Rep | 72 h | DPA web form | "Without undue delay" when high risk  
**🇪🇺 EU (AI Act)** | Serious incident from high-risk AI (Art. 73) | Market surveillance authority | 15 d (15 days; 10 d for death/widespread harm) | Member-State-specific | Affected subjects via deployer  
**🇸🇬 Singapore** | PDPA notifiable data breach | PDPC (Personal Data Protection Commission) | 72 h (significant harm or ≥ 500 individuals) | PDPC online form | "As soon as practicable"  
**🇺🇸 US (state)** | State breach laws (varies by state, e.g. CCPA) | State AG (CA: AG office) | varies · CA: 60 d typical | State-specific | Affected residents directly  
**SOC 2** | Material change in TSC scope | Auditor | 30 d | Auditor portal | Internal only  
**ISO 27001** | Security event requiring corrective action (A.16.1) | Certification body | Annual surveillance | Audit-cycle log | Internal only  
  
Flow · Vietnam breach → 72-h notification → 30-d DSAR clock

sequenceDiagram autonumber participant OB as OBS · anomaly detector participant DPO as DPO (Founder until P3) participant CP as CP subgraph participant A05 as MoPS / A05 participant SUBJ as Affected data subjects participant TC as Trust Center OB->>DPO: alert · anomaly score above 0.9 DPO->>CP: classify(severity=high, scope=acme-tenant, dataCount=312) CP->>CP: T0 timestamp · 72-h clock starts CP->>OB: span "breach.classified" par MoPS notification (72h) CP->>A05: A05 incident form · within 48h target A05-->>CP: ack and Trust Center disclosure CP->>TC: publish status (post-72h, redacted) and Affected subjects (without delay if high risk) CP->>SUBJ: email + in-app banner + payslip stub flag end Note over CP: DSAR clock begins for affected subjects SUBJ->>CP: DSAR request "what data of mine was affected?" CP-->>SUBJ: 30-day response · structured export CP->>OB: span "breach.notification.complete" 

Flow · DPIA workflow (PDPL Art. 24 + GDPR Art. 35)

flowchart TB TRIGGER[("New module / new data category  
going into memory")] --> CHECK{"Sensitive PD?  
Health · biometric · financial · employment-decision"} CHECK -- "no" --> SKIP[/"No DPIA · log assessment"/] CHECK -- "yes" --> TEMPLATE["Open DPIA template  
(KB module · auto-prefilled)"] TEMPLATE --> FILL["DPO drafts  
1\. processing description  
2\. lawful basis  
3\. minimisation  
4\. retention  
5\. sharing chain  
6\. risk × severity  
7\. mitigations  
8\. residual risk"] FILL --> COUNCIL{"DPO + Founder sign-off"} COUNCIL -- "reject" --> TEMPLATE COUNCIL -- "approve" --> A05["Cross-border? File A05 form"] A05 --> BR_GATE["Update memory ingest denylist  
per DPIA categorisation"] BR_GATE --> AUDIT["Append to compliance audit chain"] AUDIT --> LIVE[("Processing may begin")] classDef trig fill:#fef6e0,stroke:#9c750a classDef gate fill:#e8d4c2,stroke:#45210e,stroke-width:2px classDef done fill:#f5ede6,stroke:#45210e class TRIGGER trig class CHECK,COUNCIL gate class SKIP,LIVE done 

Flow · Data Subject Access Request (PDPL Art. 14 + GDPR Art. 15)

sequenceDiagram autonumber actor S as Data subject participant TC as Trust Center · DSAR form participant AU as AUTH · identity proof participant CP as CP subgraph participant BR as memory participant DB as Per-module Postgres participant FILES as R2 signed exports S->>TC: submit DSAR with identity proof TC->>AU: verify (passkey + government ID match) AU-->>TC: verified · subject_id resolved TC->>CP: open DSAR ticket · 30-day clock starts par data discovery CP->>DB: SELECT all rows with subject_id (RLS-aware) and CP->>BR: memorySearch + memoryFacts(subject_id) end DB-->>CP: rows · JSON export BR-->>CP: facts · with Layer 2 + Layer 3 hits CP->>FILES: bundle as signed Ed25519 zip CP->>S: secure link (24h TTL) alt erasure request S->>CP: also request erasure (Art. 17) CP->>BR: memoryForget(scope=subject_id) CP->>DB: redact / nullify retained-fields CP->>S: erasure confirmation with retention rationale end CP->>CP: log to Merkle audit chain (cannot be erased) 

∞

## References

#### Vietnamese regulations (Ring 1)

  * [Decree 13/2023/NĐ-CP](<https://thuvienphapluat.vn/>) — Personal Data Protection
  * [Decree 53/2022/NĐ-CP](<https://thuvienphapluat.vn/>) — Cybersecurity
  * [PDPL Art. 38 (Law 91/2025)](<https://thuvienphapluat.vn/>) — SME grace period
  * [Law 91/2025/QH15](<https://thuvienphapluat.vn/>) — PDPL national law
  * [Decree 356/2025](<https://thuvienphapluat.vn/>) — PDPL implementing decree
  * NQ 142/2024/QH15 — National Assembly tax stimulus
  * TT 80/2021/TT-BTC — Tax administration
  * Decree 123/2020/NĐ-CP — e-invoice rules
  * Law on Accounting 2015 — 10-year retention
  * Labour Code 2019 — SI/PIT framework



#### Cross-border + standards (Ring 2/3)

  * [GDPR (Regulation EU 2016/679)](<https://gdpr.eu/>)
  * [EU AI Act (Regulation EU 2024/1689)](<https://artificialintelligenceact.eu/>)
  * [Singapore PDPA](<https://www.pdpc.gov.sg/>)
  * [ISO/IEC 27001:2022](<https://www.iso.org/standard/27001>)
  * [ISO/IEC 42001:2023 (AIMS)](<https://www.iso.org/standard/81230.html>)
  * [ISO/IEC 27701:2019 (PIMS)](<https://www.iso.org/standard/71670.html>)
  * [AICPA SOC 2 TSC 2017](<https://www.aicpa.org/>)
  * [CSA STAR Program](<https://cloudsecurityalliance.org/star>)
  * [AI-CAIQ / AICM](<https://cloudsecurityalliance.org/research/working-groups/ai-controls>)
  * Internal spec — Compliance Strategy (full text)
  * Internal spec — Locked decisions DEC-053 / DEC-054 / DEC-055



[ Back to home ](<../index.html>) [ Next · Tech stack  ](<tech-stack.html>)
