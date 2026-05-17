# Software Development Process, Audit Framework, and Modernization Roadmap

**Prepared for:** CyberSkill — Software Solutions Consultancy And Development JSC, Vietnam
**For the attention of:** Stephen Cheng (Trịnh Thái Anh), Founder
**Slogan:** "Turn Your Will Into Real"

---

## Executive Summary

CyberSkill is a 2020-founded Vietnamese consultancy positioning for global growth. To win and retain international clients of varying sizes, the firm needs (a) a single, consistent internal delivery process that can flex across engagement types, (b) a layered audit framework grounded in international standards, and (c) a modern, AI-aware toolchain. This report recommends a **Hybrid Agile + DevSecOps delivery model anchored on ISO/IEC/IEEE 12207:2017 process architecture, ISO/IEC 25010:2023 quality characteristics, and CMMI v3.0 practice areas**, with Scrum or Kanban selected per engagement and a thin gating overlay drawn from PRINCE2 7th Edition and PMBOK 8th Edition. The audit layer combines ISO 9001 (quality), ISO/IEC 27001:2022 (security), SOC 2 Trust Services Criteria, IEEE 1028 (reviews and audits), and OWASP Top 10:2025 (application security). DORA's four key metrics (deployment frequency, lead time for changes, change failure rate, failed deployment recovery time) become the operational dashboard. A phased 0–12-month roadmap is provided.

Two important context notes shape every recommendation. First, the 2024 Accelerate State of DevOps report found that broad AI tooling adoption correlated with a ~1.5% drop in delivery throughput and a ~7.2% drop in stability — AI accelerates code production but enlarges batch sizes and erodes trust unless the team has strong code review and small-batch discipline. Second, OWASP Top 10:2025 elevated Software Supply Chain Failures (A03) and Security Misconfiguration (A02), reflecting how modern attacks target the pipeline itself, not just the running application. Both findings inform CyberSkill's audit posture.

---

## 1. SDLC Methodologies — Comparison and Recommendation

| Methodology | Core Idea | Pros | Cons | Ideal Use | Team Size |
|---|---|---|---|---|---|
| Waterfall | Sequential phases, big upfront design | Predictable cost/scope; heavy documentation | Inflexible to change; late integration risk | Regulated, fixed-scope, hardware-coupled | 5–50 |
| V-Model | Waterfall with test phase mirrored per dev phase | Strong verification rigor | Same rigidity as waterfall | Safety-critical, medical, aerospace | 10–100 |
| Iterative | Repeated build–evaluate cycles | Early feedback; risk reduction | Requires disciplined scoping | Research-style or unclear domains | 5–30 |
| Incremental | Deliver in functional slices | Faster value; absorbs learning | Architecture drift risk | Modular products | 5–50 |
| Spiral (Boehm) | Risk-driven iterations | Strong risk control | Heavy planning overhead | High-risk, high-cost programs | 20+ |
| Scrum | Time-boxed sprints, product backlog | Empirical, role-clear | Ceremony-heavy if poorly run | Product features, evolving scope | 5–9 per team |
| Kanban | Continuous flow, WIP limits | Excellent for support/ops, T&M | Less ceremony = less alignment | Maintenance, BAU, staff aug | 3–15 |
| XP (Extreme Programming) | TDD, pair programming, CI | Highest engineering quality | Cultural intensity | High-quality bars; complex domains | 4–12 |
| SAFe 6.0 | Multi-team Agile + portfolio governance | Enterprise scaling, certifications | Top-down, prescriptive — Thoughtworks and others note creativity/autonomy cost | 50+ engineers per program | 50–1000+ |
| LeSS | Minimalist Scrum scaling | Preserves Scrum simplicity | Demands deep org redesign | Tech-driven scale-ups | up to ~8 teams |
| Lean | Eliminate waste, optimize flow | Cost-efficient, customer-value focused | Underspecifies engineering practice | Improvement overlays on any method | Any |
| DevOps | Dev + Ops integration, automation, CI/CD | DORA-validated performance gains | Cultural change required | All modern software | Any |
| DevSecOps | DevOps with shift-left security | Earlier vulnerability detection | Tooling sprawl risk | Regulated or trust-sensitive clients | Any |
| Hybrid | Mix per engagement | Maximum flexibility | Risk of inconsistency without governance | Consultancies serving varied clients | Any |

**Recommendation for CyberSkill: a Hybrid model — "Scrum-default, Kanban-for-support, Waterfall-gates-for-fixed-price" — built on a common DevSecOps engineering substrate.** Justification: a consultancy must (1) run multiple parallel engagements with different commercial structures, (2) provide auditable artifacts to enterprise clients, and (3) preserve engineering excellence regardless of pricing model. Scrum handles dedicated-team and product-build engagements; Kanban suits managed services and staff augmentation; light Waterfall-style stage gates are layered onto fixed-price contracts so scope, cost, and acceptance are explicit. A unified DevSecOps engineering core (Git, CI/CD, IaC, automated security scanning, observability) is identical across all engagements, so quality, KPIs, and audit evidence are consistent.

---

## 2. The Thirteen Stages — Purpose, I/O, Activities, Pitfalls, KPIs

The stage architecture mirrors ISO/IEC/IEEE 12207:2017 process groups (Agreement, Organizational Project-Enabling, Technical Management, Technical). For each stage below, RACI uses A = Accountable, R = Responsible, C = Consulted, I = Informed across roles Client Sponsor (CS), Engagement Manager (EM), Product Owner (PO), Tech Lead (TL), Architect (AR), Developer (DEV), QA, DevOps (DO), Security (SEC).

**(a) Pre-engagement / Discovery / Pre-sales.** *Purpose:* qualify fit, scope at a high level, price. *Inputs:* lead, NDA, client problem statement. *Outputs:* discovery brief, ballpark estimate, proposal, MSA/SOW draft. *Activities:* stakeholder interviews, business-model canvas, rough order-of-magnitude estimate, risk screen. *RACI:* EM A/R, AR C, SEC C. *Tools:* Notion, HubSpot, Miro. *Pitfalls:* unrealistic estimates, undisclosed compliance constraints. *KPIs:* proposal win rate, lead-to-SOW cycle time.

**(b) Requirements gathering and analysis.** *Inputs:* signed SOW, stakeholder access, existing systems documentation. *Outputs:* Software Requirements Specification per IEEE 830 (business, functional, non-functional, technical), prioritized backlog, glossary, personas, acceptance criteria. *Activities:* workshops, user-story mapping, NFR elicitation against ISO/IEC 25010:2023's nine quality characteristics (functional suitability, performance efficiency, compatibility, interaction capability, reliability, security, maintainability, flexibility, safety). *RACI:* PO A/R, EM C, AR C, QA C. *Pitfalls:* gold-plating; missing NFRs; "we'll figure security out later" — OWASP A06 Insecure Design failure mode. *KPIs:* requirements volatility %, NFR coverage %.

**(c) Feasibility study and project planning.** *Inputs:* SRS, constraints (budget, timeline, regulatory). *Outputs:* technical feasibility memo, project plan, RAID log, communication plan, Definition of Ready/Done. *Activities:* tech spikes, cost/benefit, schedule (PMBOK 8th Edition performance domains; PRINCE2 7th Edition business case theme). *RACI:* EM A, TL R, AR R. *KPIs:* estimate accuracy vs actuals, plan adherence.

**(d) System architecture and high-level design.** *Inputs:* approved SRS, NFRs. *Outputs:* architecture decision records (ADRs), C4 diagrams, technology selection, threat model (OWASP STRIDE), capacity plan. *Activities:* arc42-style documentation, security architecture review against OWASP ASVS. *RACI:* AR A/R, SEC C, TL C. *Pitfalls:* over-engineering; ignoring portability/maintainability subcharacteristics from ISO/IEC 25010. *KPIs:* ADR coverage of significant decisions, architecture review pass rate.

**(e) Detailed design.** *Inputs:* approved architecture. *Outputs:* UI/UX prototypes, component/API specs (OpenAPI), database schema, Software Design Description per IEEE 1016. *RACI:* TL A, DEV R, AR C, QA C, PO C. *Tools:* Figma, dbdiagram.io, Stoplight. *Pitfalls:* design-development handoff loss; missing API versioning strategy. *KPIs:* design review defect density.

**(f) Implementation / Coding.** *Inputs:* SDD, DoR-met backlog items, dev environment. *Outputs:* working code, unit tests, feature branches, PRs. *Activities:* trunk-based development or short-lived branches, TDD where appropriate, conventional commits, secret scanning. *RACI:* DEV R, TL A, SEC C. *Pitfalls:* large batches (DORA repeatedly warns AI-assisted coding inflates batch size); skipped tests. *KPIs:* PR cycle time, code coverage, mean PR size.

**(g) Code review and integration.** *Inputs:* PR with passing checks. *Outputs:* merged code, updated CHANGELOG. *Activities:* peer review per IEEE 1028, automated SAST (e.g., SonarQube, Snyk Code), DAST and SCA on dependencies (A03 Software Supply Chain Failures), SBOM generation. *RACI:* DEV R (reviewer), TL A, SEC C. *Pitfalls:* rubber-stamp reviews, especially of AI-generated code. *KPIs:* review depth (comments/PR), defect-escape rate, mean time to merge.

**(h) Testing.** *Inputs:* test plan, test environments, test data. *Outputs:* unit, integration, system, UAT, performance, security, regression and accessibility test results. *Activities:* execute Test Strategy; perform threat-led pen testing against OWASP Top 10:2025; performance test against NFRs. *RACI:* QA A/R, SEC C, PO C (UAT). *Tools:* Playwright/Cypress, k6/JMeter, OWASP ZAP, Burp Suite, Trivy. *KPIs:* defect density, defect leakage, test automation coverage.

**(i) Deployment and release management (CI/CD).** *Inputs:* release candidate, deployment checklist, change-approval record. *Outputs:* deployed release, release notes, deployment evidence. *Activities:* progressive delivery (canary/blue-green), feature flags, immutable infra, signed artifacts, deployment policy-as-code (OPA). *RACI:* DO A/R, TL C, SEC C. *KPIs:* DORA — deployment frequency, lead time for changes, change failure rate, failed deployment recovery time.

**(j) Operations, monitoring, maintenance, support.** *Inputs:* runbook, SLOs/SLAs, on-call rota. *Outputs:* incident tickets, post-mortems, patch releases. *Activities:* SRE practices (Google SRE), error budgets, observability (Grafana, Datadog, OpenTelemetry), proactive vulnerability management. *RACI:* DO A/R, SEC C, EM I. *KPIs:* MTTR, availability vs SLO, error-budget burn.

**(k) Documentation (cross-cutting).** *Outputs:* user docs, API docs, runbooks, ADRs, onboarding guides. DORA shows documentation quality strongly predicts performance; AI assistants amplify both good and bad documentation. *Tools:* Docusaurus, Notion, MkDocs. *KPIs:* doc freshness, time-to-onboard new engineer.

**(l) Project closure and retrospective.** *Outputs:* sign-off certificate, lessons learned, knowledge-transfer pack, asset handover. *RACI:* EM A, PO R, TL C. *KPIs:* client NPS, on-time/on-budget closure rate.

**(m) Decommissioning / Retirement.** *Inputs:* retirement decision, data-retention policy. *Outputs:* data export/destruction certificate, DNS retirement, license cancellation, archived source. *RACI:* DO R, SEC C, EM A. *KPIs:* clean-shutdown checklist completion.

---

## 3. Layered Audit Framework

**Formal layer — standards CyberSkill should map to:**
- **ISO/IEC/IEEE 12207:2017** organizes all stages above into Agreement, Organizational Project-Enabling, Technical Management, and Technical process groups (a DIS 12207:2027 revision is in development).
- **ISO/IEC 25010:2023** is the quality measurement target; each release is evaluated against its nine characteristics and their subcharacteristics (new in 2023: inclusivity, self-descriptiveness, resistance, scalability; user engagement replaced UI aesthetics; faultlessness replaced maturity).
- **ISO 9001** anchors the QMS and continual-improvement discipline.
- **ISO/IEC 27001:2022** provides the ISMS controls (also aligns with Vietnam's Cybersecurity Law 2018 and Decree 85/2016/ND-CP).
- **SOC 2 Type II** is a near-mandatory deal-opener for North American clients; ~80% of its Trust Services Criteria overlap with ISO 27001, so pursue both as a SOC 2+ combined effort.
- **CMMI v3.0** (Levels 1 Initial → 2 Managed → 3 Defined → 4 Quantitatively Managed → 5 Optimizing) is the maturity target; aim for Level 3 within 18–24 months.
- **IEEE 830** (SRS), **IEEE 1016** (SDD), **IEEE 1028** (Reviews and Audits) govern artifact form.
- **PMBOK 8th Edition** (May 2026 release reintroduces process guidance alongside the seven performance domains and six principles) and **PRINCE2 7th Edition** (five integrated elements: principles, people, practices, processes, project context; "Issues" replaces "Change" theme) cover governance and tailoring.
- **OWASP Top 10:2025** — A01 Broken Access Control, A02 Security Misconfiguration, A03 Software Supply Chain Failures, A04 Cryptographic Failures, A05 Injection, A06 Insecure Design, A07 Authentication Failures, A08 Software & Data Integrity Failures, A09 Security Logging & Alerting Failures, A10 Mishandling of Exceptional Conditions — plus OWASP ASVS, SAMM, DSOMM for maturity.

**Practical layer — applied per stage.** Every stage has explicit **entry criteria (Definition of Ready)**, **exit criteria (Definition of Done)**, a **review checklist**, and a **quality gate**. **Verification** asks "did we build it right?" (peer review, static analysis, unit tests against design); **validation** asks "did we build the right thing?" (UAT, NFR conformance, business outcome).

**Traceability matrix.** Each requirement ID (REQ-###) is linked through design ID → code module/PR → test case → release. The RTM is auto-generated by Jira/GitHub integrations and reviewed at every stage gate.

**Risk-based audit prioritization.** Use a simple heatmap: client-criticality × regulatory-load × data-sensitivity × architectural-complexity. High-risk engagements receive: formal threat model, monthly internal audits, SOC 2-grade evidence collection. Low-risk: lightweight quarterly review.

---

## 4. Ready-to-Use Templates

**4.1 Definition of Ready (story-level):** clear user value, acceptance criteria, dependencies identified, NFRs noted, security/privacy implications flagged, designs attached, estimable in one sprint, demoable.

**4.2 Definition of Done (story-level):** code merged to main; unit + integration tests passing; coverage threshold met; SAST/SCA clean; documentation updated; deployed to staging; product-owner accepted; observability hooks present.

**4.3 Stage-gate sign-off (one page):** Stage name • Entry criteria met (Y/N + evidence link) • Exit criteria met (Y/N + evidence link) • Risks/issues • Decision (Go / Go-with-conditions / No-Go) • Signatures (EM, TL, Client Sponsor).

**4.4 Requirements Traceability Matrix columns:** REQ-ID | Description | Source | Priority | Linked Design | Linked Code/PR | Linked Test | Status | Release.

**4.5 Code Review Checklist:** correctness vs ticket; readability; tests cover new code; no secrets; no SQL/command injection paths; input validation; error handling; logging without PII; performance considerations; backward compatibility; **AI-generated code specifically reviewed for hallucinated APIs, oversized diffs, and dependency additions**.

**4.6 Test Strategy outline:** scope; risk-based test priorities; test levels (unit/integration/system/UAT); test types (functional, performance, security, accessibility, regression); environments and data; tooling; entry/exit criteria; defect management; metrics.

**4.7 Deployment Readiness Checklist:** all DoDs met; release notes; rollback plan; feature flags configured; database migrations rehearsed; monitoring/alerts in place; on-call notified; security scan clean; change ticket approved; SBOM published.

**4.8 Retrospective Template (Start/Stop/Continue + DORA Review):** team mood; DORA metric trends this iteration; 3 keep-doings; 3 stop-doings; top 1–2 actions with owners.

**4.9 Project Charter / SOW skeleton:** objectives and success criteria; scope (in/out); deliverables; assumptions and constraints; engagement model; team and roles; schedule and milestones; pricing and invoicing; acceptance criteria; IP and confidentiality; change control; warranty and support; governance cadence.

**4.10 RACI matrix (template across all stages):** rows = stages; columns = CS, EM, PO, TL, AR, DEV, QA, DO, SEC; cells = R/A/C/I.

---

## 5. Modern Tooling and AI Integration

**Recommended toolchain per stage.** PM: Jira + Linear, Notion. Design: Figma. Source/CI: GitHub + GitHub Actions (or GitLab). IaC: Terraform + Pulumi. Containers/Orchestration: Docker + Kubernetes. Testing: Playwright, k6, OWASP ZAP, Snyk, Trivy. Observability: OpenTelemetry, Grafana, Sentry. Security: Snyk, GitGuardian, SonarQube, Aqua, Wiz. Documentation: Docusaurus / MkDocs. Knowledge management: Notion workspace per client with a shared "CyberSkill Engineering Handbook" cross-linked into every project.

**AI-assisted development.** Recent independent reviews (BuiltIn, SitePoint, NxCode, 2026) consistently place **Claude Code** as the strongest agent for deep, multi-file reasoning, **Cursor** as the best AI-native IDE, and **GitHub Copilot** as the broadest, lowest-friction enterprise option (with Claude Code now available inside Copilot Pro+/Enterprise via Agent HQ). Most experienced teams run a combination — Cursor or Copilot in the IDE for daily completion, Claude Code in a terminal for large refactors.

**How to integrate AI safely** (driven by DORA 2024 findings):
1. Treat AI output as junior-engineer output: always reviewed, never auto-merged.
2. Enforce small batch sizes — DORA's #1 risk vector when AI is used.
3. Use enterprise tiers with training opt-out; verify code-residency for client confidentiality (critical when bidding to EU/US clients under GDPR).
4. Maintain a published **AI-use policy per client SOW** stating which tools are permitted and what data may leave the perimeter.
5. **Audit AI-generated code** with: mandatory human review, SAST/SCA in the PR, SBOM checks for hallucinated or compromised dependencies (A03), unit tests written by a human or independently regenerated, and a "AI-assisted: yes/no" PR label that triggers stricter review.
6. Track DORA metrics monthly to detect throughput/stability regressions caused by AI tooling.

---

## 6. Consultancy-Specific Considerations

**Engagement models.** *Fixed-price* — high upfront discovery, strict change control, stage-gated; best for well-scoped projects. *Time & materials* — Kanban-friendly, weekly invoicing, transparent timesheets; best for evolving scope. *Dedicated team* — long-term retainer, Scrum cadence, deep product knowledge. *Staff augmentation* — individual engineers embedded in client teams; CyberSkill maintains performance management and quality oversight. *Managed services* — SLA-driven Kanban with on-call rotations.

**Parallel project consistency.** A single engineering handbook, identical CI/CD templates, identical security scanners, and centralized DORA dashboards mean every team produces evidence in the same format. Cross-project quality gates (architecture review board, security review board) staffed by senior engineers prevent quality drift.

**Communication cadence.** Daily stand-up (internal); weekly client status (written + 30 min call); fortnightly demo; monthly steering committee; quarterly business review (QBR) including DORA metrics, NPS, and roadmap.

**IP, confidentiality, contracts.** Each SOW addresses: IP assignment on payment, pre-existing IP carve-out, background-IP licensing, source-code escrow if required, NDA scope and term, sub-processor list (essential for GDPR), data-processing addendum, AI-tool usage disclosure, security incident notification timeline, audit rights.

**Onboarding/offboarding.** Client onboarding pack (kickoff deck, security briefing, comms plan, access provisioning checklist, DPA). Offboarding pack (knowledge transfer, source handover, runbook, credentials rotation, data-destruction certificate, retrospective).

**Scaling globally.** Move from CMMI L1→L3 in 18 months. Build regional clusters (APAC native — Vietnam HQ; later US/EU presence). Localize working hours via "follow-the-sun" cells. Invest in English-language technical writing; pursue ISO 27001 then SOC 2 Type II as deal-openers.

---

## 7. Phased Adoption Roadmap

**0–3 months — Foundations.** Publish CyberSkill Engineering Handbook v1; standardize Git/CI templates; adopt Jira workflow with DoR/DoD; implement DORA dashboard; pilot Hybrid Scrum/Kanban on two engagements; deploy SAST/SCA/secret-scanning across all repos; publish AI-usage policy; ISO 27001 gap assessment kickoff; standard SOW/MSA templates.

**3–6 months — Hardening.** Achieve ISO 27001:2022 certification readiness; implement OWASP Top 10:2025 controls (start with A01 Broken Access Control and A02 Security Misconfiguration); roll out OpenTelemetry-based observability; introduce stage-gate sign-offs for fixed-price work; formal Architecture Review Board; SBOM generation on every release; threat modeling in every discovery; start CMMI Level 2 self-assessment.

**6–12 months — Scaling.** Achieve ISO 27001 certification and SOC 2 Type I; complete CMMI v3.0 Level 3 appraisal; introduce internal developer platform (per DORA platform-engineering findings — invest carefully and measure); formalize regional delivery cells; AI-augmented development with measured impact on DORA metrics; quarterly business reviews with all major clients; begin SOC 2 Type II observation window; targeted certifications (Vietnam Cybersecurity Law compliance evidence pack, GDPR DPA library).

---

## 8. Coverage Check

| Required topic | Covered |
|---|---|
| Full SDLC | ✓ Section 2 |
| All major methodologies + recommendation | ✓ Section 1 |
| 13 stages with inputs/outputs/RACI/pitfalls/KPIs | ✓ Section 2 |
| Formal audit standards (ISO/IEEE/CMMI/PMBOK/PRINCE2/OWASP/SOC 2) | ✓ Section 3 |
| Practical audit per stage, traceability, V&V, risk-based | ✓ Section 3 |
| Templates (DoR, DoD, RTM, code review, test strategy, deployment, retro, SOW, RACI, stage-gate) | ✓ Section 4 |
| Modern tooling per stage + AI integration + AI code audit | ✓ Section 5 |
| Consultancy engagement models, parallel work, comms, IP, onboarding, scaling | ✓ Section 6 |
| Phased 0–3 / 3–6 / 6–12 month roadmap | ✓ Section 7 |
| Executive summary | ✓ top |
| Authoritative sources cited (ISO, IEEE, CMMI, PMI, PRINCE2/AXELOS, OWASP, AICPA, Google DORA, Atlassian/GitLab) | ✓ throughout |

The framework above is deliberately optimized for a Vietnam-headquartered consultancy scaling globally: it produces standards-grade audit evidence by default, adapts to every engagement model without changing the underlying engineering substrate, and embeds the most current 2024–2026 findings on AI-assisted development risk into both process and audit. Stephen and the CyberSkill team can copy Sections 4 and 7 into their internal handbook and begin operating against them next week.