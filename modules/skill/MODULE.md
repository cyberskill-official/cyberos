# CyberOS SKILL module — canonical catalog

Version: 2.0.0  Status: Normative for the SKILL module. Companion files: `README.md` (operational quickstart), `docs/SPEC.md` (protocol contract), `docs/CHANGELOG.md` (shipping record).

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

This document is the source of truth for **which skills exist in this module, what artifact each emits, and how skills chain**. Every skill bundle on disk MUST correspond to a row in §3; every row in §3 MUST correspond to a bundle on disk OR be marked `planned` in §3.

---

## §0  Design rules

§0.1  **Flat layout.** Every skill bundle lives at `skill/<skill-name>/`. There is no `skills/` subfolder, no persona subfolder, no owner-role subfolder. Persona/role concerns live in the `cuo/` module, not here.

§0.2  **Author + Audit pair per artifact.** For every artifact CyberSkill produces in the software delivery lifecycle, this module SHALL ship two sibling skills: `<artifact>-author` (generates the artifact) and `<artifact>-audit` (validates the artifact against a versioned rubric). Both bundles are independently invocable. The author chains to the audit by default via its output envelope's `next_skill_recommendation`.

§0.3  **Audit-fix loop until outputs fit inputs.** Every skill SHALL implement the 8-step audit loop (`docs/AUDIT_LOOP.md`): locate → hash → load-or-init report → run rubric → attempt fixes → re-audit → termination check → write report. The loop terminates only on `PASS`, `HITL_PAUSE`, `EXHAUSTED`, or `NO_PROGRESS`. The author + audit pair MUST be able to round-trip until the artifact passes its own rubric (10/10) or a human intervenes.

§0.4  **Anthropic Agent Skills compliance.** Every bundle MUST be a valid Anthropic Agent Skill (SKILL.md + frontmatter + progressive disclosure). CyberOS extensions live under the spec-permitted `metadata` map or in sibling files (`INVARIANTS.md`, `RUBRIC.md`, `AUDIT_LOOP.md`, etc.). See `docs/SPEC.md` §3.1 for the extension list.

§0.5  **No fabrication.** Every skill SHALL operate under `references/ANTI_FABRICATION.md` discipline: source-grounded claims only, authority markers per claim, HITL on ambiguity, untrusted-content wrapping. The discipline cross-references AGENTS.md §5.1 and §9.1 in the memory module.

§0.6  **Manifest re-entrancy.** Every skill that emits multiple artifacts in a batch SHALL persist its state in a `manifest.json` file in the output directory and SHALL compute its phase (PLAN / WORKER / RESUME) from manifest state, not from caller assertion.

§0.7  **CONTRACT_ECHO preamble.** Every skill invocation SHALL emit one fenced `CONTRACT_ECHO` block before any file operation, declaring `skill_id`, `skill_version`, `prompt_revision`, `template_version`, inputs, and computed `phase`.

§0.8  **Self-contained bundles.** A skill bundle SHALL NOT reference siblings outside its own folder for prompt content, rubric content, or fixtures. Cross-skill artifact schemas live in `skill/contracts/` (preserved infrastructure). The bundle MAY import from `skill/contracts/<id>/template.md` via `depends_on_contracts:` declared in frontmatter — this is the only sanctioned cross-bundle reference.

---

## §1  Skill lifecycle

| Phase | Trigger | Output |
|---|---|---|
| Draft | Author writes initial SKILL.md against `_template/` | `skill/<name>/` directory with frontmatter + body |
| Audit | Author runs the matching `<name>-audit` skill against its own SKILL.md | Sibling `.audit.md` report |
| Iterate | Author applies fixes; re-runs audit | Audit report at successively higher scores |
| Ship | Audit returns 10/10 (PASS, no warnings) | Skill is callable by CUO |
| Refine | User feedback triggers `human_fine_tune` per skill frontmatter | New `skill_version` + changelog entry + acceptance test |
| Retire | Skill is superseded; metadata.deprecated set | Bundle moved to `skill/_retired/` after 30-day soak |

The lifecycle is the same for every skill in the catalog. The `_template/` skeleton ships every required file (SKILL.md, INVARIANTS.md, PIPELINE.md, references/, acceptance/, CHANGELOG.md) so a new author can copy + customize without architectural decisions.

---

## §2  Stage map — Software Development Process to skill bundle

The Software Development Process.md document at the project root defines 13 SDLC stages (a through m). This module ships an author + audit pair for every artifact those stages produce. Stages (k) Documentation is cross-cutting and is covered by per-artifact skills below. Templates §4.1–§4.10 from the document each map to a skill or a section of one.

| SDP §2 stage | Primary artifact(s) | Author skill | Audit skill | Template ref |
|---|---|---|---|---|
| (a) Pre-engagement / Discovery / Pre-sales | SOW / Project Charter | `statement-of-work-author` | `statement-of-work-audit` | §4.9 |
| (b) Requirements gathering and analysis | SRS per IEEE 830 | `software-requirements-specification-author` | `software-requirements-specification-audit` | — |
| (b) Requirements — backlog | Prioritized feature backlog | `feature-request-author` | `feature-request-audit` | — |
| (b) Requirements — governance | Definition of Ready/Done | `definition-of-ready-and-done-author` | `definition-of-ready-and-done-audit` | §4.1, §4.2 |
| (c) Feasibility study and project planning | Feasibility memo + project plan + RAID log | `project-plan-author` | `project-plan-audit` | — |
| (c) Planning — governance | Stage-gate sign-off | `stage-gate-author` | `stage-gate-audit` | §4.3 |
| (d) System architecture and high-level design | ADRs + C4 diagrams + tech selection | `architecture-decision-record-author` | `architecture-decision-record-audit` | — |
| (d) Architecture — security | STRIDE threat model + OWASP ASVS map | `threat-model-author` | `threat-model-audit` | — |
| (e) Detailed design | SDD per IEEE 1016 + OpenAPI + DB schema | `software-design-document-author` | `software-design-document-audit` | — |
| (f) Implementation / Coding | Implementation plan (FR → tasks) | `implementation-plan-author` | `implementation-plan-audit` | — |
| (g) Code review and integration | Code-review write-up per IEEE 1028 | `code-review-author` | `code-review-audit` | §4.5 |
| (h) Testing | Test Strategy | `test-strategy-author` | `test-strategy-audit` | §4.6 |
| (i) Deployment and release management | Deployment Readiness Checklist | `deployment-checklist-author` | `deployment-checklist-audit` | §4.7 |
| (i) Release | Customer-facing release notes | `release-notes-author` | `release-notes-audit` | — |
| (j) Operations, monitoring, maintenance | Operational runbook + SLO/SLA declaration | `runbook-author` | `runbook-audit` | — |
| (j) Operations — incidents | Blameless post-mortem | `postmortem-author` | `postmortem-audit` | — |
| Cross-cutting | Requirements Traceability Matrix | `requirements-traceability-matrix-author` | `requirements-traceability-matrix-audit` | §4.4 |
| Cross-cutting | Product Requirements Document | `product-requirements-document-author` | `product-requirements-document-audit` | — |
| (l) Project closure and retrospective | Retrospective | `retrospective-author` | `retrospective-audit` | §4.8 |
| (l) Closure — sign-off | Closure cert + lessons learned + KT pack | `closure-author` | `closure-audit` | — |
| (m) Decommissioning / Retirement | Data export/destruction certificate + DNS retirement + license cancellation | `decommissioning-author` | `decommissioning-audit` | — |
| Project hygiene | Repo state audit + fragment absorption | `project-cleanup` | (self-auditing) | — |

Total: 22 author+audit pairs + 1 self-auditing utility = **45 skill bundles** when the catalog is fully shipped.

---

## §3  Skill catalog — status

| Skill bundle | Stage | Status | Version |
|---|---|---|---|
| `_template/` | meta | shipped | 1.0.0 |
| `project-cleanup/` | hygiene | shipped (preserved) | 1.0.1 |
| `statement-of-work-author/` | (a) | shipped | 1.0.0 |
| `statement-of-work-audit/` | (a) | shipped | 1.0.0 |
| `software-requirements-specification-author/` | (b) | shipped | 1.0.0 |
| `software-requirements-specification-audit/` | (b) | shipped | 1.0.0 |
| `feature-request-author/` | (b) | shipped | 1.0.0 |
| `feature-request-audit/` | (b) | shipped | 1.0.0 |
| `definition-of-ready-and-done-author/` | (b) | shipped | 1.0.0 |
| `definition-of-ready-and-done-audit/` | (b) | shipped | 1.0.0 |
| `project-plan-author/` | (c) | shipped | 1.0.0 |
| `project-plan-audit/` | (c) | shipped | 1.0.0 |
| `stage-gate-author/` | (c) | shipped | 1.0.0 |
| `stage-gate-audit/` | (c) | shipped | 1.0.0 |
| `architecture-decision-record-author/` | (d) | shipped | 1.0.0 |
| `architecture-decision-record-audit/` | (d) | shipped | 1.0.0 |
| `threat-model-author/` | (d) | shipped | 1.0.0 |
| `threat-model-audit/` | (d) | shipped | 1.0.0 |
| `software-design-document-author/` | (e) | shipped | 1.0.0 |
| `software-design-document-audit/` | (e) | shipped | 1.0.0 |
| `implementation-plan-author/` | (f) | shipped | 1.0.0 |
| `implementation-plan-audit/` | (f) | shipped | 1.0.0 |
| `code-review-author/` | (g) | shipped | 1.0.0 |
| `code-review-audit/` | (g) | shipped | 1.0.0 |
| `test-strategy-author/` | (h) | shipped | 1.0.0 |
| `test-strategy-audit/` | (h) | shipped | 1.0.0 |
| `deployment-checklist-author/` | (i) | shipped | 1.0.0 |
| `deployment-checklist-audit/` | (i) | shipped | 1.0.0 |
| `release-notes-author/` | (i) | shipped | 1.0.0 |
| `release-notes-audit/` | (i) | shipped | 1.0.0 |
| `runbook-author/` | (j) | shipped | 1.0.0 |
| `runbook-audit/` | (j) | shipped | 1.0.0 |
| `postmortem-author/` | (j) | shipped | 1.0.0 |
| `postmortem-audit/` | (j) | shipped | 1.0.0 |
| `requirements-traceability-matrix-author/` | cross | shipped | 1.0.0 |
| `requirements-traceability-matrix-audit/` | cross | shipped | 1.0.0 |
| `product-requirements-document-author/` | cross | shipped | 1.0.0 |
| `product-requirements-document-audit/` | cross | shipped | 1.0.0 |
| `retrospective-author/` | (l) | shipped | 1.0.0 |
| `retrospective-audit/` | (l) | shipped | 1.0.0 |
| `closure-author/` | (l) | shipped | 1.0.0 |
| `closure-audit/` | (l) | shipped | 1.0.0 |
| `decommissioning-author/` | (m) | shipped | 1.0.0 |
| `decommissioning-audit/` | (m) | shipped | 1.0.0 |

**Catalog complete: 22 author+audit pairs + project-cleanup + _template = 46 bundles.** Every pair shipped at 10/10 on its own rubric (each `<artifact>-audit` validates its sibling `<artifact>-author`'s output against the corresponding `<artifact>_rubric@1.0`).

---

### §3.1  Session A (2026-05-17 evening continuation) — Tier-1 catalog expansion

Per `../cuo/docs/NEEDED_SKILLS.md` §1, 29 new author+audit pairs shipped to unblock CyberSkill scale-up-critical personas (CFO / CEO / CHRO / CRO-Revenue / Chief-of-Staff / CCO-Customer / COO / CPO-Privacy / CAIO / Chief-Ethics-Officer). All scaffolded from `_template/`, customised with persona-aware descriptions + compact RUBRIC.md following `docs/RUBRIC_FORMAT.md`, with matching contracts under `contracts/<name>/`.

| Skill bundle | Persona driver | Status | Version |
|---|---|---|---|
| `objectives-and-key-results-set-author/` + `objectives-and-key-results-set-audit/` | CEO / Chief of Staff | shipped | 1.0.0 |
| `board-deck-author/` + `board-deck-audit/` | CEO / CFO / CTO | shipped | 1.0.0 |
| `investor-update-author/` + `investor-update-audit/` | CEO / CFO | shipped | 1.0.0 |
| `capital-allocation-memo-author/` + `capital-allocation-memo-audit/` | CEO + CFO | shipped | 1.0.0 |
| `hire-decision-author/` + `hire-decision-audit/` | CEO + CHRO | shipped | 1.0.0 |
| `forecast-author/` + `forecast-audit/` | CFO | shipped | 1.0.0 |
| `budget-author/` + `budget-audit/` | CFO + CEO | shipped | 1.0.0 |
| `monthly-close-author/` + `monthly-close-audit/` | CFO + CAO-Accounting | shipped | 1.0.0 |
| `compensation-plan-author/` + `compensation-plan-audit/` | CHRO + CRO-Revenue (for sales) | shipped | 1.0.0 |
| `workforce-plan-author/` + `workforce-plan-audit/` | CHRO | shipped | 1.0.0 |
| `onboarding-pack-author/` + `onboarding-pack-audit/` | CHRO + Chief of Staff | shipped | 1.0.0 |
| `employee-net-promoter-score-program-author/` + `employee-net-promoter-score-program-audit/` | CHRO + Chief Happiness Officer | shipped | 1.0.0 |
| `pipeline-report-author/` + `pipeline-report-audit/` | CRO-Revenue + CSO-Sales | shipped | 1.0.0 |
| `account-plan-author/` + `account-plan-audit/` | CRO-Revenue + CCO-Customer | shipped | 1.0.0 |
| `decision-log-author/` + `decision-log-audit/` | Chief of Staff | shipped | 1.0.0 |
| `rhythm-of-business-author/` + `rhythm-of-business-audit/` | Chief of Staff | shipped | 1.0.0 |
| `customer-success-engagement-author/` + `customer-success-engagement-audit/` | CCO-Customer | shipped | 1.0.0 |
| `net-promoter-score-program-author/` + `net-promoter-score-program-audit/` | CCO-Customer + CPO-Product | shipped | 1.0.0 |
| `vendor-scorecard-author/` + `vendor-scorecard-audit/` | COO + CPO-Procurement | shipped | 1.0.0 |
| `operating-model-author/` + `operating-model-audit/` | COO | shipped | 1.0.0 |
| `capacity-plan-author/` + `capacity-plan-audit/` | COO | shipped | 1.0.0 |
| `diversity-equity-inclusion-program-author/` + `diversity-equity-inclusion-program-audit/` | CDO-Diversity + CHRO | shipped | 1.0.0 |
| `compliance-program-author/` + `compliance-program-audit/` | CCO-Compliance + CPO-Privacy | shipped | 1.0.0 |
| `privacy-impact-assessment-author/` + `privacy-impact-assessment-audit/` | CPO-Privacy | shipped | 1.0.0 |
| `breach-notification-author/` + `breach-notification-audit/` | CPO-Privacy + CISO + CLO-Legal | shipped | 1.0.0 |
| `data-subject-request-runbook-author/` + `data-subject-request-runbook-audit/` | CPO-Privacy | shipped | 1.0.0 |
| `model-card-author/` + `model-card-audit/` | CAIO + Chief Ethics Officer | shipped | 1.0.0 |
| `ai-use-case-portfolio-author/` + `ai-use-case-portfolio-audit/` | CAIO | shipped | 1.0.0 |
| `bias-audit-author/` + `bias-audit-audit/` | Chief Ethics Officer + CAIO | shipped | 1.0.0 |

**Session A subtotal: 29 new pairs (58 new bundles) + 29 new contracts shipped.**

**Updated catalog grand total: 22 (Session morning) + 29 (Session A evening) = 51 author+audit pairs = 102 SDP/persona-aligned bundles + project-cleanup + _template + 5 public/ VN bundles + 54 contracts.**

Per `../cuo/docs/NEEDED_SKILLS.md` §2, **Session B (next continuation)** ships the remaining 29 Tier-2 pairs (CRO-Risk / CDO-Data / CMO / Chief-Brand / Chief-Transformation / Chief-Digital / Chief-ESG / CSO-Sustainability / CCO-Communications / Chief-Knowledge / Chief-Innovation / Chief-Automation / Chief-Trust / CPO-Procurement / CSO-Strategy workflows). Session C ships the 8 Tier-3 niche pairs. Sessions D-E ship the per-persona workflows (~100-150 workflow files).

Each Session-A skill RUBRIC is at v1.0 compact form (FM-001..004 + FM-101..104 + SEC-001..004 + COND-001 + QA-CITE/AUTH/NUM/VAGUE/OWNER/DUE/TODO/QUOTE + SAFE-001..004 + XCHAIN-001/002 + STALE-001). Skill-specific per-field rules (FM-105+) and quality heuristics (QA-NNN) are added in v1.1+ via the fine-tune discipline per `docs/FINE_TUNE.md`.

---

### §3.2  Session B (2026-05-17 evening continuation) — Tier-2 catalog expansion

Per `../cuo/docs/NEEDED_SKILLS.md` §2, 29 new author+audit pairs shipped to unblock the growth/enterprise-tier personas (CRO-Risk / CDO-Data / CMO / Chief-Brand / Chief-Transformation / Chief-Digital / Chief-ESG / CSO-Sustainability / CCO-Communications / Chief-Knowledge / Chief-Innovation / Chief-Automation / Chief-Trust / CPO-Procurement / CSO-Strategy / Chief-Ethics-Officer). All scaffolded from `_template/`, customised with persona-aware descriptions + compact RUBRIC.md, matching contracts under `contracts/<name>/`.

| Skill bundle | Persona driver | Status | Version |
|---|---|---|---|
| `enterprise-risk-framework-author` + `-audit` | CRO-Risk | shipped | 1.0.0 |
| `key-risk-indicator-dashboard-author` + `-audit` | CRO-Risk | shipped | 1.0.0 |
| `data-strategy-author` + `-audit` | CDO-Data | shipped | 1.0.0 |
| `data-product-author` + `-audit` | CDO-Data | shipped | 1.0.0 |
| `data-governance-author` + `-audit` | CDO-Data + CCO-Compliance + CPO-Privacy | shipped | 1.0.0 |
| `brand-strategy-author` + `-audit` | CMO + Chief Brand | shipped | 1.0.0 |
| `campaign-plan-author` + `-audit` | CMO | shipped | 1.0.0 |
| `product-roadmap-author` + `-audit` | CPO-Product | shipped | 1.0.0 |
| `customer-advisory-board-author` + `-audit` | CCO-Customer + CPO-Product | shipped | 1.0.0 |
| `transformation-roadmap-author` + `-audit` | Chief Transformation + Chief Digital | shipped | 1.0.0 |
| `sustainability-report-author` + `-audit` | CSO-Sustainability + Chief ESG | shipped | 1.0.0 |
| `emissions-inventory-author` + `-audit` | CSO-Sustainability | shipped | 1.0.0 |
| `crisis-communications-playbook-author` + `-audit` | CCO-Communications + CSO-Security | shipped | 1.0.0 |
| `press-release-author` + `-audit` | CCO-Communications | shipped | 1.0.0 |
| `internal-newsletter-author` + `-audit` | CCO-Communications + Chief of Staff | shipped | 1.0.0 |
| `analyst-briefing-author` + `-audit` | CCO-Communications + CEO + CPO-Product | shipped | 1.0.0 |
| `knowledge-asset-author` + `-audit` | Chief Knowledge Officer | shipped | 1.0.0 |
| `knowledge-taxonomy-author` + `-audit` | Chief Knowledge Officer | shipped | 1.0.0 |
| `automation-roadmap-author` + `-audit` | Chief Automation Officer | shipped | 1.0.0 |
| `trust-portal-update-author` + `-audit` | Chief Trust Officer + CPO-Privacy | shipped | 1.0.0 |
| `transparency-report-author` + `-audit` | Chief Trust Officer | shipped | 1.0.0 |
| `ethics-review-author` + `-audit` | Chief Ethics Officer | shipped | 1.0.0 |
| `change-management-plan-author` + `-audit` | Chief Transformation Officer | shipped | 1.0.0 |
| `program-charter-author` + `-audit` | Chief Transformation + Chief Innovation | shipped | 1.0.0 |
| `innovation-portfolio-author` + `-audit` | Chief Innovation Officer | shipped | 1.0.0 |
| `partner-program-author` + `-audit` | CCO-Commercial | shipped | 1.0.0 |
| `procurement-strategy-author` + `-audit` | CPO-Procurement | shipped | 1.0.0 |
| `mergers-and-acquisitions-thesis-author` + `-audit` | CSO-Strategy + CFO | shipped | 1.0.0 |
| `strategy-document-author` + `-audit` | CSO-Strategy or CEO | shipped | 1.0.0 |

**Session B subtotal: 29 new pairs (58 new bundles) + 29 new contracts shipped.**

**Updated catalog grand total: 21 (SDP morning) + 29 (Tier-1 Session A) + 29 (Tier-2 Session B) = 79 author+audit pairs = 158 SDP/persona-aligned bundles + project-cleanup + _template + 5 public/ VN bundles + 83 contracts.**

Note: `product-roadmap` was renamed from the originally-listed `prd-strategic` in `NEEDED_SKILLS.md` §2 to avoid collision with the existing `product-requirements-document-author/audit` pair (which covers the broader PRD artefact per SDP §2(b)).

---

### §3.3  Session C (2026-05-17 evening continuation) — Tier-3 niche catalog expansion

Per `../cuo/docs/NEEDED_SKILLS.md` §3, 8 new author+audit pairs shipped to unblock the specialty/niche personas (Chief-Medical-Officer / CIO-Investment / CRO-Restructuring / Chief-Remote / Chief-Happiness). All scaffolded from `_template/`, customised with persona-aware descriptions + compact RUBRIC.md, matching contracts under `contracts/<name>/`.

| Skill bundle | Persona driver | Status | Version |
|---|---|---|---|
| `clinical-protocol-author` + `-audit` | Chief Medical Officer | shipped | 1.0.0 |
| `safety-report-author` + `-audit` | Chief Medical Officer | shipped | 1.0.0 |
| `investment-thesis-author` + `-audit` | CIO-Investment | shipped | 1.0.0 |
| `limited-partner-letter-author` + `-audit` | CIO-Investment | shipped | 1.0.0 |
| `turnaround-plan-author` + `-audit` | CRO-Restructuring | shipped | 1.0.0 |
| `thirteen-week-cash-flow-author` + `-audit` | CRO-Restructuring + CFO (distress) | shipped | 1.0.0 |
| `remote-policy-author` + `-audit` | Chief Remote Officer + CHRO | shipped | 1.0.0 |
| `happiness-program-author` + `-audit` | Chief Happiness Officer + CHRO | shipped | 1.0.0 |

**Session C subtotal: 8 new pairs (16 new bundles) + 8 new contracts shipped.**

**Updated catalog grand total: 21 (SDP morning) + 29 (Tier-1 Session A) + 29 (Tier-2 Session B) + 8 (Tier-3 Session C) = 87 author+audit pairs = 174 SDP/persona-aligned bundles + project-cleanup + _template + 5 public/ VN bundles + 91 contracts.**

Standards cited in Tier-3 contracts: ICH-GCP E6(R3) (`clinical-protocol`); ICH E2D PSUR + 21 CFR 314.80 (`safety-report`); Soros/Druckenmiller/Marks investment-pattern literature (`investment-thesis`); ILPA Reporting Template (`lp-letter`); AlixPartners / FTI / Alvarez & Marsal turnaround playbooks (`turnaround-plan`); industry-standard 13-week TWCF model (`13-week-cash-flow`); GitLab Remote Manifesto + Buffer State-of-Remote (`remote-policy`); Officevibe / TINYpulse / Culture Amp + Shawn Achor positive-psychology research (`happiness-program`).

**Per-skill fine-tune overrides:** none of the 8 Tier-3 skills shipped with a `<skill>/FINE_TUNE.md` override (default discipline applies). The 4 existing FINE_TUNE overrides (feature-request-audit, code-review-audit, threat-model-audit, decommissioning-audit) remain. For clinical-protocol + safety-report specifically, a v1.1 fine-tune is a likely candidate once a regulated-pharma engagement surfaces ICH-GCP/E2D section-ordering deviations through real use.

With Session C complete, **66 / 66 = 100 % of the original new-pair scope identified in `NEEDED_SKILLS.md` is shipped**. Session E later added 5 more pairs (Tier-4 legal, surfaced during Session D workflow authoring) bringing the closed-gap total to 71/71.

---

### §3.4  Session E (2026-05-17 evening continuation) — Tier-4 legal-specific catalog expansion

Per `../cuo/docs/NEEDED_SKILLS.md` §4, 5 new author+audit pairs shipped to unblock CLO-Legal workflows. These were surfaced during Session D (Now-tier workflow authoring) when CLO-Legal workflows could not chain through existing shipped skills — they were enumerated as Tier-4 and shipped Session E before CLO-Legal's workflows were authored. All scaffolded from `_template/`, customised with persona-aware descriptions + compact RUBRIC.md, matching contracts under `contracts/<name>/`.

| Skill bundle | Persona driver | Status | Version |
|---|---|---|---|
| `contract-review-author` + `-audit` | CLO-Legal (general contract review across the firm) | shipped | 1.0.0 |
| `non-disclosure-agreement-triage-author` + `-audit` | CLO-Legal | shipped | 1.0.0 |
| `regulatory-filing-author` + `-audit` | CLO-Legal + CPO-Privacy (breach notification) | shipped | 1.0.0 |
| `intellectual-property-strategy-author` + `-audit` | CLO-Legal | shipped | 1.0.0 |
| `litigation-management-update-author` + `-audit` | CLO-Legal | shipped | 1.0.0 |

**Session E subtotal: 5 new pairs (10 new bundles) + 5 new contracts shipped.**

**Updated catalog grand total: 21 (SDP morning) + 29 (Tier-1 Session A) + 29 (Tier-2 Session B) + 8 (Tier-3 Session C) + 5 (Tier-4 Session E) = 92 author+audit pairs = 184 SDP/persona-aligned bundles + project-cleanup + _template + 5 public/ VN bundles + 96 contracts.**

Standards cited in Tier-4 contracts: ABA Model Contract Clauses + ACC Contract Playbook + WorldCC (formerly IACCM) (`contract-review`); ACC NDA triage standard + WorldCC NDA best practices + ABA Model Mutual NDA (`nda-triage`); SEC Reg S-K (10-K/10-Q/8-K) + FDA 21 CFR + EU AI Act Regulation (EU) 2024/1689 Annex IV + GDPR Art. 33 + Vietnam Decree 13/2023 PDPD (`regulatory-filing`); WIPO IP strategy framework + USPTO MPEP + Madrid Protocol + UTSA/DTSA + PCT (`ip-strategy`); Litify case-mgmt model + ACC Value Challenge + ABA Litigation Section (`litigation-mgmt-update`).

**Per-skill fine-tune overrides:** none of the 5 Tier-4 skills shipped with a `<skill>/FINE_TUNE.md` override (default discipline applies). For `regulatory-filing` specifically, a v1.1 per-regulator fine-tune is a likely candidate (SEC vs FDA vs AI Act vs GDPR have very different section ordering — the v1.0 RUBRIC enforces "every H2 in template.md is present" but a regulated-industry engagement will surface the need for regulator-specific SEC-NNN rules).

With Session E complete, the catalog is **92 author+audit pairs / 184 bundles / 96 contracts** — and the Now-tier (≤50 people per C-Suite Reference §7) persona-workflow wave is closed (5/47 personas have shipped workflows: CTO 5 + CEO 5 + CFO 5 + chief-of-staff 4 + clo-legal 5 = 24 workflows live). The next wave is Sessions F+ for Series-A tier personas.

---

### §3.5  Session F (2026-05-17 evening continuation) — Tier-5 Series-A catalog expansion

Per `../cuo/docs/NEEDED_SKILLS.md` §4b, 6 new author+audit pairs shipped to unblock Series-A tier persona workflows (COO + CHRO + CSO-Sales/CGO + CISO/vCISO per C-Suite Reference §7). Same surface-then-ship pattern Session E used for CLO-Legal: Session F's Series-A workflow authoring identified missing skills, those were enumerated as Tier-5, and shipped before the workflows were authored. All scaffolded from `_template/`, customised with persona-aware descriptions + compact RUBRIC.md, matching contracts under `contracts/<name>/`.

| Skill bundle | Persona driver | Status | Version |
|---|---|---|---|
| `go-to-market-plan-author` + `-audit` | CSO-Sales / CGO | shipped | 1.0.0 |
| `vulnerability-management-report-author` + `-audit` | CISO | shipped | 1.0.0 |
| `penetration-test-report-author` + `-audit` | CISO | shipped | 1.0.0 |
| `soc2-evidence-author` + `-audit` | CISO + CCO-Compliance | shipped | 1.0.0 |
| `security-strategy-author` + `-audit` | CISO | shipped | 1.0.0 |
| `delivery-review-author` + `-audit` | COO (Head of Delivery) | shipped | 1.0.0 |

**Session F subtotal: 6 new pairs (12 new bundles) + 6 new contracts shipped.**

**Updated catalog grand total: 21 (SDP morning) + 29 (Tier-1 Session A) + 29 (Tier-2 Session B) + 8 (Tier-3 Session C) + 5 (Tier-4 Session E) + 6 (Tier-5 Session F) = 98 author+audit pairs = 196 SDP/persona-aligned bundles + project-cleanup + _template + 5 public/ VN bundles + 102 contracts.**

Standards cited in Tier-5 contracts: Winning by Design + MEDDIC/MEDDPICC + Predictable Revenue + OpenView PLG playbook (`gtm-plan`); CIS Controls v8 + NIST SP 800-40 Rev 4 + ISO/IEC 27001:2022 + CVSS v3.1/v4.0 (`vulnerability-mgmt-report`); OWASP WSTG v4.2 + PTES + NIST SP 800-115 + OWASP ASVS v5.0 + MITRE ATT&CK (`pen-test-report`); AICPA TSC 2017 (2022 points-of-focus) + ISAE 3000/3402 (`soc2-evidence`); NIST CSF 2.0 + ISO/IEC 27001:2022 + CIS Controls v8 + Verizon DBIR + MITRE ATT&CK (`security-strategy`); Bain agile-PMO + McKinsey delivery-excellence + DORA metrics + TSIA benchmarks (`delivery-review`).

**Per-skill fine-tune overrides:** none of the 6 Tier-5 skills shipped with a `<skill>/FINE_TUNE.md` override. Likely v1.1 candidates: `soc2-evidence` (TSC-specific per-control evidence rules per audit firm preferences); `pen-test-report` (per-engagement-type rules — web-app vs cloud-infra vs network-segmentation tests have different methodology emphasis); `vulnerability-mgmt-report` (industry-specific SLA enforcement — financial services tighter than SaaS).

With Session F complete, the catalog is **98 author+audit pairs / 196 bundles / 102 contracts** — and the Series-A (50-100 people) persona-workflow wave is closed (9/47 personas: Now-tier 5 + Series-A 4 = 40 workflows live). The next wave is Sessions G+ for Scale-up tier personas.

---

### §3.6  Session G (2026-05-17 evening continuation) — Tier-6 Scale-up catalog expansion

Per `../cuo/docs/NEEDED_SKILLS.md` §4c, 1 new author+audit pair shipped to unblock the CRO-Revenue `quarterly-churn-analysis` workflow. The other 9 Session-G Scale-up workflows (CPO-Privacy 4 + CFO depth 1 + CHRO depth 1 + 3 more CRO-Revenue) chain entirely through already-shipped Tier-1/Tier-2/Tier-4/Tier-5 skills — no further new skills required.

| Skill bundle | Persona driver | Status | Version |
|---|---|---|---|
| `churn-analysis-author` + `-audit` | CRO-Revenue + CCO-Customer | shipped | 1.0.0 |

**Session G subtotal: 1 new pair (2 new bundles) + 1 new contract shipped.**

**Updated catalog grand total: 21 (SDP morning) + 29 (Tier-1 Session A) + 29 (Tier-2 Session B) + 8 (Tier-3 Session C) + 5 (Tier-4 Session E) + 6 (Tier-5 Session F) + 1 (Tier-6 Session G) = 99 author+audit pairs = 198 SDP/persona-aligned bundles + project-cleanup + _template + 5 public/ VN bundles + 103 contracts.**

Standards cited in the Tier-6 contract: Reichheld customer-economics (Loyalty Effect / Net Promoter System) + Gainsight customer-success operating model + Catalyst customer-success benchmarks + TSIA churn-benchmarking + Bessemer Cloud Index SaaS churn benchmarks.

**Per-skill fine-tune overrides:** `churn-analysis` shipped without a `<skill>/FINE_TUNE.md` override. Likely v1.1 candidates: per-business-model churn-reason taxonomy (SaaS vs services vs marketplace have very different reason vocabularies; cohort decay-curve shape differs).

With Session G complete, the catalog is **99 author+audit pairs / 198 bundles / 103 contracts** — and CRO-Revenue + CPO-Privacy first-coverage Scale-up wave is closed (11/47 personas: Now-tier 5 + Series-A 4 + Scale-up new 2 = 50 workflows live). The next wave is Sessions H+ for Enterprise tier (CPO-Product + CDO-Data/CAIO + CCO-Customer + Chief-Knowledge-Officer).

---

### §3.7  Session H (2026-05-18) — Tier-7 Enterprise catalog expansion

Per `../cuo/docs/NEEDED_SKILLS.md` §4d, 5 new author+audit pairs shipped to unblock Enterprise tier workflows (CPO-Product + CDO-Data + CAIO + CCO-Customer + Chief-Knowledge-Officer per C-Suite Reference §7). Largest skill-surfacing wave since Tier-5 — Enterprise personas have the most distinctive artefact production patterns.

| Skill bundle | Persona driver | Status | Version |
|---|---|---|---|
| `product-metrics-review-author` + `-audit` | CPO-Product | shipped | 1.0.0 |
| `customer-360-author` + `-audit` | CDO-Data | shipped | 1.0.0 |
| `ai-strategy-author` + `-audit` | CAIO | shipped | 1.0.0 |
| `customer-health-review-author` + `-audit` | CCO-Customer | shipped | 1.0.0 |
| `knowledge-pipeline-author` + `-audit` | Chief Knowledge Officer | shipped | 1.0.0 |

**Session H subtotal: 5 new pairs (10 new bundles) + 5 new contracts shipped.**

**Updated catalog grand total: 21 (SDP morning) + 29 (Tier-1 Session A) + 29 (Tier-2 Session B) + 8 (Tier-3 Session C) + 5 (Tier-4 Session E) + 6 (Tier-5 Session F) + 1 (Tier-6 Session G) + 5 (Tier-7 Session H) = 104 author+audit pairs = 208 SDP/persona-aligned bundles + project-cleanup + _template + 5 public/ VN bundles + 108 contracts.**

Standards cited in Tier-7 contracts: Amplitude / Mixpanel / Pendo + Reforge + Sequoia PLG + Hooked (Nir Eyal) (`product-metrics-review`); CDP Institute + Segment / RudderStack / mParticle + IAB GVL/TCF + ISO/IEC 19944 + DAMA-DMBOK MDM (`customer-360`); NIST AI RMF 1.0 + EU AI Act Regulation (EU) 2024/1689 + ISO/IEC 42001:2023 + Stanford HAI + Google MLOps maturity + Anthropic Responsible Scaling Policy (`ai-strategy`); Gainsight + Catalyst + TSIA + Bessemer Cloud Index (`customer-health-review`); Nonaka SECI + Wenger CoP + Davenport + ANSI Z39.19 + McKinsey KM (`knowledge-pipeline`).

**Per-skill fine-tune overrides:** none of the 5 Tier-7 skills shipped with a `<skill>/FINE_TUNE.md` override. Likely v1.1 candidates: `ai-strategy` (EU AI Act risk-class enforcement will tighten as Annex IV ICR guidance matures in 2026-2027); `customer-360` (per-CDP-vendor section conventions — Segment vs mParticle vs RudderStack diverge); `knowledge-pipeline` (per-knowledge-domain ANSI Z39.19 taxonomy depth — legal/medical/financial knowledge each have tighter vocabulary control than software-consulting).

With Session H complete, the catalog is **104 author+audit pairs / 208 bundles / 108 contracts** — and the Enterprise (200+ people) persona-workflow wave is closed (16/47 personas: Now-tier 5 + Series-A 4 + Scale-up new 2 + Enterprise new 5 = 70 workflows live). The next wave is Sessions I+ for the remaining ~31 niche/specialty personas (CMO, CRO-Risk, Chief-Brand, Chief-Transformation, Chief-Digital, Chief-ESG, CSO-Sustainability, CCO-Communications, etc.) and depth additions across already-shipped personas.

---

## §4  Chain graph

Skills are independently invocable, but the **default chain** for a full delivery flow is:

```
statement-of-work-author → statement-of-work-audit
   ↓ (on PASS)
product-requirements-document-author → product-requirements-document-audit
   ↓ (on PASS)
software-requirements-specification-author → software-requirements-specification-audit
   ↓ (on PASS)
feature-request-author → feature-request-audit
   ↓ (on PASS, per-FR)
implementation-plan-author → implementation-plan-audit
   ↓ (on PASS, per-FR)
[engineering execution — no skill]
   ↓
code-review-author → code-review-audit
   ↓ (on PASS)
test-strategy-author → test-strategy-audit
   ↓ (on PASS)
deployment-checklist-author → deployment-checklist-audit
   ↓ (on PASS, per-release)
release-notes-author → release-notes-audit
   ↓ (on PASS)
runbook-author → runbook-audit
   ↓ (per-incident, on demand)
postmortem-author → postmortem-audit
   ↓ (per-sprint, on demand)
retrospective-author → retrospective-audit
   ↓ (on engagement end)
closure-author → closure-audit
   ↓ (on decommission decision)
decommissioning-author → decommissioning-audit
```

Side-chains:
- `architecture-decision-record-author → architecture-decision-record-audit` runs alongside (d) — typically before `software-design-document-author`. Triggered whenever an architectural decision is taken.
- `threat-model-author → threat-model-audit` runs alongside (d) — once per major architecture change.
- `stage-gate-author → stage-gate-audit` runs at each stage boundary in fixed-price engagements.
- `definition-of-ready-and-done-author → definition-of-ready-and-done-audit` runs once per engagement at kickoff (and on policy changes).
- `requirements-traceability-matrix-author → requirements-traceability-matrix-audit` runs continuously — re-generated after every merged FR or test.

`project-cleanup/` runs on demand — monthly or pre-handoff — and is self-auditing (4-phase pipeline includes inventory + verify).

Each skill's `produces.next_skill_recommendation` field encodes its default downstream link. The CUO supervisor reads this field and queues the next skill unless the user opts out.

---

## §5  Cross-references

- `docs/Software Development Process.md` (project root) — the source document this catalog implements. §2 stages (a–m), §4 templates (4.1–4.10), §3 audit framework all map directly into the catalog.
- `cuo/` — the router module that picks which skill to invoke for a natural-language request. CUO consumes this module's catalog via `cyberos-cuo catalog`.
- `memory/` — the memory. Skills declare `allowed_memory_scopes` in their SKILL.md frontmatter; the host's capability broker enforces them. AGENTS.md §0–§17 govern the memory protocol that constrains memory writes.
- `skill/contracts/` — artifact schemas (PRD, SRS, FR, task, impl-plan, project-brief, chain-manifest, nats-subjects). Skills import these via `depends_on_contracts:` rather than redefining schemas locally.
- `skill/docs/SPEC.md` — the protocol contract every skill MUST satisfy.
- `skill/docs/AUDIT_LOOP.md` — the 8-step audit algorithm every audit skill implements.
- `skill/docs/RUBRIC_FORMAT.md` — the rubric column format every audit skill's `RUBRIC.md` follows.

---

## §6  Operating principles

§6.1  **Each skill is self-contained.** A bundle's SKILL.md tells the agent everything needed to run the skill. Reference files (`references/*.md`) ship with the bundle, not in a shared location. Acceptance fixtures (`acceptance/*.json`, `acceptance/*.md`) ship with the bundle. This satisfies AGENTS.md §0 of the memory protocol (precedence + immutability) and matches the project-self-containment feedback memory: never reference sibling bundles for content.

§6.2  **The audit skill is the spec.** Every author skill's behavior is normative-fixed by its sibling audit skill's `RUBRIC.md`. To change an author's behavior, update the audit's rubric first, regenerate the artifact, observe the new audit verdict, then update the author body to satisfy the new rubric. This is the only way the catalog stays internally consistent.

§6.3  **Audit-fix loop until 10/10.** Per the FR-authoring loop discipline (memory: `feedback_fr_authoring_loop.md`), every artifact SHALL pass its rubric at 10/10 before the next artifact is started. Half-finished artifacts are forbidden. The audit loop's `EXHAUSTED` exit condition triggers HITL escalation rather than ship-with-warnings.

§6.4  **March autonomously on continue.** Per the FR-autonomous-march memory, when a user says "continue", drain the planned-skill frontier until a decision is needed or the catalog is complete — do not pause between skills.

§6.5  **AGENTS.md §14 emission.** After every session that touches non-memory files, emit the §14.1 / §14.2 block as the memory heartbeat signal (memory: `feedback_section_14_emission.md`). The block summarises file ops, scopes touched, rejections, and token budget.

§6.6  **Project self-containment.** This module references the SDP document at `docs/Software Development Process.md` in the same project root. It does NOT reference other CyberSkill projects (sale-noti, landing-page, design-system, tamagochi, design-system-audit-framework). Per memory `feedback_project_self_containment.md`, deliverables stay inside their project.

---

## §7  Migration record (2026-05-17)

| Action | Reason |
|---|---|
| Wiped `skill/skills/` (47 bundle dirs across `cuo/cpo/`, `cuo/chief-technology-officer/`, `cuo/_shared/`, `cyberskill-vn/`, `shared/`) | User direction: persona-organized layout retired. CUO module already handles persona concerns. |
| Preserved `skill/project-cleanup/` (moved from `skill/skills/shared/project-cleanup/`) | Working hygiene utility; user explicitly called out to keep it. |
| Preserved `skill/contracts/` (artifact schemas: chain-manifest, feature-request, impl-plan, nats-subjects, prd, project-brief, srs, task) | Schemas remain canonical; new author skills import via `depends_on_contracts:`. |
| Preserved `skill/crates/, toolchain/, runners/, tools/, tests/, tours/` | Rust host, Bun toolchain, Python parity runners, parity harness — runtime infrastructure unchanged. |
| Added `skill/MODULE.md` (this file) | Canonical catalog. |
| Added `skill/_template/` | Canonical skeleton for new author/audit skills. |
| Added `skill/statement-of-work-author/, statement-of-work-audit/, feature-request-author/, feature-request-audit/, product-requirements-document-author/, product-requirements-document-audit/` | Four canonical reference pairs proving the new pattern (stages a, b, cross). |
| `skill/<name>-author/` + `skill/<name>-audit/` for stages (b second skill), (c)–(m), cross | Planned. Built session-by-session under the FR-authoring loop discipline. |
| Vietnamese-market skills (vietnam-bank-transfer, vietnam-legal-compliance, vietnam-mst-validate, vn-tax-filing, vietnam-vat-invoice, vietnam-vneid-integration) wiped from this module | Preserved at `public-skills/` (cyberos project root) for open-registry publication. Not part of the SDP-driven core catalog. |

The wipe was destructive. Recovery is via git history. No memory entries were touched.

---

## §8  Vietnamese-market skills + the `public/` open-distribution channel

The five VN-market skills (`vietnam-bank-transfer`, `vietnam-legal-compliance`, `vietnam-mst-validate`, `vietnam-vat-invoice`, `vietnam-vneid-integration`) live at **`skill/public/<skill-name>/`** as of the 2026-05-17 evening rebuild. They were absorbed from the legacy `cyberos/public-skills/` tree into the SKILL module per user direction. They co-exist with the SDP-driven catalog: runtime discovery picks them up automatically via the loader's flat-layout walk (no exclusion for `public/`).

§8.1  **Why a `public/` subfolder (not flat at `skill/<vn-name>/`).** Two channels with different audiences:

| Channel | Path | Audience | Distribution |
|---|---|---|---|
| SDP-driven runtime catalog | `skill/<artifact>-author/` + `<artifact>-audit/` | Internal CyberSkill engagements | Loader auto-discovery + CUO workflows |
| Open-publication channel | `skill/public/<vn-name>/` | External Vietnamese-market users | `agentskills.io` registry + `cyberos-skill install` |

Keeping them under `public/` preserves the OSS-distribution assets shipped alongside the VN skills (`LICENSE`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `INSTALL.md`, `.github/workflows/validate.yml`, `.github/ISSUE_TEMPLATE/`, `announcements/blog-post.md` + `linkedin-*.md` + `twitter-thread.md`) — those would clutter the runtime catalog if flattened.

§8.2  **Runtime discovery.** `crates/host/src/loader.rs` does NOT exclude `public/` from the walkdir. Each VN bundle's `SKILL.md` is registered alongside the SDP-driven pairs. Operators invoke via `cyberos-skill run vietnam-mst-validate ...` exactly as they invoke `cyberos-skill run statement-of-work-author ...`.

§8.3  **Open-publication workflow.** When a new VN skill is ready for publication, drop it under `skill/public/<name>/`, run the validate workflow at `skill/public/.github/workflows/validate.yml`, and push. The `agentskills.io` submission process pulls from the public/ subtree only — the SDP runtime catalog (everything else under `skill/`) is internal.

§8.4  **Roadmap for `public/`.** Per `skill/public/README.md` §Roadmap: VSS / BHXH validation, e-signature wrapper, customs declaration builder, provincial address normaliser, Vietnamese NLP helpers. New VN skills land at `skill/public/<name>/` with their own SKILL.md + the standard bundle subfolders (`scripts/`, `references/`, `assets/`, `tests/`).

§8.5  **Phase 7 legacy compat window.** The Rust host's `primary_script()` (see `crates/cli/src/main.rs`) keeps the VN bundle name → script-path mappings as long as the public/ skills still ship script-tier executors. WASM compilation per Phase 5 will eventually retire these mappings.

§8.6  **`vn-tax-filing`.** The legacy `vn-tax-filing` skill is NOT present under `skill/public/` after the absorption — it only existed in the previous `skill/skills/cyberskill-vn/` location and was wiped during the 2026-05-17 morning rebuild. If needed, restore from git history (`git checkout HEAD~N -- skill/skills/cyberskill-vn/vn-tax-filing/`) and move into `skill/public/`.
