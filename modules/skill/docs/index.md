---
title: Skill — 104 author+audit pairs · SDP + 7 persona-driven tiers · memory-integrated · CyberOS
source: website/docs/modules/skill/index.html
migrated: FR-DOCS-002
---

**Skill is the catalog**. It does not invent a competing format; it embraces the open Anthropic _Agent Skills_ standard verbatim so a CyberOS skill works unmodified in Claude Code, Codex CLI, Cursor, VS Code with Copilot, Goose, Amp, Gemini CLI. The Rust host indexes SKILL.md frontmatter at boot (~100 tokens/skill, parsed in parallel), activates bodies lazily, runs WASM components in Wasmtime with WASI capability grants, and falls back to a sandboxed script path for trusted internal skills. The differentiation is the curated **cyberskill-vn** collection: five high-quality Vietnamese-market skills (MST · VAT · VietQR · CCCD · legal) published on the open registry where the entire Agent Skills ecosystem can install them. 

Status (open-std)

Phases 0–7 shipped

audit complete · 30-day soak active

Status (memory-int)

Phase 8 designed

capability broker → allowed_memory_scopes

LoC (Rust host)

~3,000 + ~1,500 (Phase 8)

4 crates · cargo workspace · +memory-broker

LoC (Bun toolchain)

~2,500

TS authoring → wasm32-wasi

Tests

4 + 12/12 parity + 2 grants

cargo test · pytest parity

SKILL.md bundles

208 indexed

104 author+audit pairs · flat layout

Vertical packs

1 shipped · 6 planned

vn · sg · id · th · eu · us · hr-legal

Ecosystem reach

12+ clients

via open Agent Skills standard

Cold start (header)

~100 tok/skill

parallel via tokio::JoinSet

Depends on

memory + AUTH

tenant-scoped capability broker

★

## SDP-driven catalog — 104 author+audit pairs

The runtime catalog is built around the 13-stage SDLC from `../../../modules/cuo/README.md`. Every artefact CyberSkill produces in the SDLC ships as a **`<artifact>-author` \+ `<artifact>-audit` pair** at `modules/skill/<name>/` (flat layout — no persona subfolders). All skills use full-format names (renamed 2026-05-18). The 8-step audit-fix loop discipline lives at `modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md` and terminates only on PASS / HITL_PAUSE / EXHAUSTED / NO_PROGRESS. Catalog source-of-truth: `modules/skill/README.md` §3. 

SDP §2 stage| Author skill| Audit skill| Rubric  
---|---|---|---  
(a) Pre-engagement / Discovery| `statement-of-work-author`| `statement-of-work-audit`| `sow_rubric@1.0`  
(b) Requirements — SRS (IEEE 830)| `software-requirements-specification-author`| `software-requirements-specification-audit`| `srs_rubric@1.0`  
(b) Requirements — backlog| `feature-request-author`| `feature-request-audit`| `audit_rubric@2.0`  
(b) Requirements — DoR/DoD| `definition-of-ready-and-done-author`| `definition-of-ready-and-done-audit`| `dor_dod_rubric@1.0`  
(c) Feasibility + project planning| `project-plan-author`| `project-plan-audit`| `project_plan_rubric@1.0`  
(c) Stage-gate governance| `stage-gate-author`| `stage-gate-audit`| `stage_gate_rubric@1.0`  
(d) Architecture decisions| `architecture-decision-record-author`| `architecture-decision-record-audit`| `adr_rubric@1.0`  
(d) STRIDE threat model| `threat-model-author`| `threat-model-audit`| `threat_model_rubric@1.0`  
(e) Detailed design (IEEE 1016)| `software-design-document-author`| `software-design-document-audit`| `sdd_rubric@1.0`  
(f) Implementation plan| `implementation-plan-author`| `implementation-plan-audit`| `impl_plan_rubric@1.0`  
(g) Code review (IEEE 1028 + AI checks)| `code-review-author`| `code-review-audit`| `code_review_rubric@1.0`  
(h) Test strategy| `test-strategy-author`| `test-strategy-audit`| `test_strategy_rubric@1.0`  
(i) Deployment readiness| `deployment-checklist-author`| `deployment-checklist-audit`| `deploy_checklist_rubric@1.0`  
(i) Release notes| `release-notes-author`| `release-notes-audit`| `release_notes_rubric@1.0`  
(j) Operational runbook| `runbook-author`| `runbook-audit`| `runbook_rubric@1.0`  
(j) Blameless post-mortem| `postmortem-author`| `postmortem-audit`| `postmortem_rubric@1.0`  
cross-cutting — PRD| `product-requirements-document-author`| `product-requirements-document-audit`| `prd_rubric@1.0`  
cross-cutting — Traceability matrix| `requirements-traceability-matrix-author`| `requirements-traceability-matrix-audit`| `rtm_rubric@1.0`  
(l) Retrospective| `retrospective-author`| `retrospective-audit`| `retro_rubric@1.0`  
(l) Project closure| `closure-author`| `closure-audit`| `closure_rubric@1.0`  
(m) Decommissioning| `decommissioning-author`| `decommissioning-audit`| `decomm_rubric@1.0`  
hygiene — self-auditing| `project-cleanup`| —  
  
**Total: 104 author+audit pairs = 208 skill-bundle directories + 108 contracts at`skill/<name>/` \+ project-cleanup + _template scaffold + 5 public/ VN bundles.** Every pair ships its own `SKILL.md` \+ `INVARIANTS.md` \+ `RUBRIC.md` (audit half) + `envelopes/` \+ `references/` \+ `acceptance/`. Authoritative authoring scaffold: `skill/_template/{author,audit}/`. Rust host supports flat layout via `crates/host/src/loader.rs` `EXCLUDED_DIR_NAMES`. **Catalog reached stable steady-state after Session H (2026-05-18)** — Sessions I-N shipped 124 niche workflows in the CUO module with zero new skills surfaced, validating that ~100 well-designed pairs serve the vast majority of C-suite persona orchestration. 

Each rubric maps to authoritative standards: **IEEE 830 / 1016 / 1028** · **ISO/IEC 25010:2023** (nine quality characteristics enforced via `QA-NFR-001`) · **OWASP Top 10:2025** \+ **OWASP ASVS** · **STRIDE** \+ **LINDDUN** · **PMBOK 8 / PRINCE2 7** · **DORA** four key metrics + small-batch discipline · **Keep-a-Changelog** 1.1.0 · **SemVer** 2.0.0 · **WCAG 2.2** · **GDPR Art. 17 + 33** · **Vietnam Decree 13/2023 PDPD** \+ **Decree 53/2022** · **PCI-DSS 9.8** · **HIPAA 45 CFR § 164.310(d)(2)**. 

### Catalog browser — group the 104 pairs by entry-tier

The 104 author+audit pairs were shipped in 7 catalog-expansion waves (Sessions A–H). Tier-1/2/3 deliver the universal SDP backbone (IEEE-grounded artefacts most personas hit). Tier-4/5/6/7 add domain-specific bundles (legal, security, sales, AI, customer). Each pair number below links to the matching `modules/skill/<name>` folder.

Tier-1 · SDP backbone (21 pairs, Session A — universal)

Every persona's workflow chain touches at least one Tier-1 pair. These are the IEEE-grounded canonical artefacts.

[feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) \+ audit

[statement-of-work-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/statement-of-work-author>) \+ audit

[software-requirements-specification-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/software-requirements-specification-author>) \+ audit

[definition-of-ready-and-done-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/definition-of-ready-and-done-author>) \+ audit

[project-plan-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/project-plan-author>) \+ audit

[stage-gate-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/stage-gate-author>) \+ audit

[architecture-decision-record-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/architecture-decision-record-author>) \+ audit

[threat-model-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/threat-model-author>) \+ audit

[software-design-document-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/software-design-document-author>) \+ audit

[implementation-plan-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/implementation-plan-author>) \+ audit

[code-review-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/code-review-author>) \+ audit

[test-strategy-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/test-strategy-author>) \+ audit

[deployment-checklist-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/deployment-checklist-author>) \+ audit

[release-notes-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/release-notes-author>) \+ audit

[runbook-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/runbook-author>) \+ audit

[postmortem-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/postmortem-author>) \+ audit

[product-requirements-document-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/product-requirements-document-author>) \+ audit

[requirements-traceability-matrix-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/requirements-traceability-matrix-author>) \+ audit

[retrospective-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/retrospective-author>) \+ audit

[closure-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/closure-author>) \+ audit

[decommissioning-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/decommissioning-author>) \+ audit

Tier-2 · Strategic + cadence (29 pairs, Session B)

Boardroom outputs + recurring operating-rhythm artefacts. Heavy use by CEO / CFO / COO / CHRO / CSO-Sales / CISO personas.

board-deck-author + audit · investor-update-author + audit · hire-decision-author + audit · forecast-author + audit · budget-author + audit · monthly-close-author + audit

workforce-plan-author + audit · onboarding-pack-author + audit · pipeline-report-author + audit · account-plan-author + audit · decision-log-author + audit · rhythm-of-business-author + audit · vendor-scorecard-author + audit

operating-model-author + audit · capacity-plan-author + audit · compliance-program-author + audit · breach-notification-author + audit · model-card-author + audit · ai-use-case-portfolio-author + audit · bias-audit-author + audit

enterprise-risk-framework-author + audit · data-strategy-author + audit · data-product-author + audit · data-governance-author + audit · brand-strategy-author + audit · campaign-plan-author + audit · product-roadmap-author + audit · objectives-and-key-results-set-author + audit · capital-allocation-memo-author + audit

Tier-3 · Niche outputs (8 pairs, Session C)

Specialist artefacts that close edge cases — analyst briefings, partner programs, ERG charters, etc.

analyst-briefing-author + audit · press-release-author + audit · partner-program-author + audit · channel-strategy-author + audit · journey-charter-author + audit · erg-charter-author + audit · sourcing-event-author + audit · supplier-scorecard-author + audit

Tier-4 · Legal (5 pairs, Session E)

CLO-Legal coverage. Contract review, NDA triage, regulatory filing, IP strategy, litigation updates.

contract-review-author + audit · nda-triage-author + audit · regulatory-filing-author + audit · ip-strategy-author + audit · litigation-mgmt-update-author + audit

Tier-5 · Security · sales · delivery (6 pairs, Session F)

Series-A wave. SOC2 evidence, pen-test reports, vuln management, security strategy, GTM plan, delivery review.

gtm-plan-author + audit · vulnerability-mgmt-report-author + audit · pen-test-report-author + audit · soc2-evidence-author + audit · security-strategy-author + audit · delivery-review-author + audit

Tier-6 · Revenue analysis (1 pair, Session G)

Scale-up wave. The only Tier-6 addition needed for the entire CRO-Revenue + CPO-Privacy coverage.

churn-analysis-author + audit

Tier-7 · Enterprise (5 pairs, Session H)

Growth/Enterprise wave. After this, Sessions I-N shipped 124 niche workflows with ZERO new skills surfaced.

product-metrics-review-author + audit · customer-360-author + audit · ai-strategy-author + audit · customer-health-review-author + audit · knowledge-pipeline-author + audit

Vertical-pack public bundles (5 shipped VN, +planned SG/ID/TH/EU/US)

These are not author/audit pairs — they're production-runnable Agent Skills exposed to external clients (Claude Code, Cursor, etc.). Detailed dossier in §3.6 below.

[vietnam-mst-validate](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/vietnam-mst-validate>)

[vietnam-vat-invoice](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/vietnam-vat-invoice>)

[vietnam-bank-transfer](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/vietnam-bank-transfer>)

[vietnam-vneid-integration](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/vietnam-vneid-integration>)

[vietnam-legal-compliance](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/vietnam-legal-compliance>)

vn-tax-filing (planned · Q3 2026)

### SKILL.md frontmatter shape — 5 worked examples

Every bundle's `SKILL.md` uses the same Agent-Skills frontmatter shape (YAML between two `---` markers, body in Markdown). Below are 5 representative pairs — the actual YAML is verified against `modules/skill/_template/{author,audit}/SKILL.md`. **Required keys** per Anthropic Agent Skills spec: `name`, `description`. **CyberOS-required** additions: `version` (SemVer), `allowed-tools` (capability brokerage), `license`, `kind` (author / audit / public). Optional: `outputs` (contracts produced), `allowed_memory_scopes` (memory read/write grants), `activationEvents` (VS-Code-style triggers).

feature-request-author · the author template every -author skill follows
    
    
    ---
    name: feature-request-author
    description: |
      Draft a feature-request Markdown that PASSES feature-request-audit on first attempt.
      Outputs <FR-MODULE-NNN>.md per modules/skill/contracts/feature-request/template.md.
    version: 2.0.0
    kind: author
    license: Apache-2.0
    allowed-tools:
      - read_file        # read existing FR catalog + spec sources
      - write_file       # write the new FR
      - run_audit        # invoke feature-request-audit as the gating check
    allowed_memory_scopes:
      - read: cuo.persona.*
      - read: skill.contracts.*
    outputs:
      - feature-request@2.0
    activationEvents:
      - "onCommand:cyberos.skill.fr-author"
      - "onIntent:/fr-author"
    ---
    <body — the author prompt, examples, anti-fabrication discipline, etc.>
    

feature-request-audit · the audit half (every -author has one)
    
    
    ---
    name: feature-request-audit
    description: |
      Score a feature-request Markdown against audit_rubric@2.0 and return PASS / FAIL / HITL_PAUSE.
      Drives the 8-step audit-fix loop described in AUTHORING_DISCIPLINE.md.
    version: 2.0.0
    kind: audit
    license: Apache-2.0
    allowed-tools:
      - read_file        # read the candidate FR
      - score_against_rubric
    allowed_memory_scopes:
      - read: skill.rubrics.*
    rubric: audit_rubric@2.0
    terminate_on: [PASS, HITL_PAUSE, EXHAUSTED, NO_PROGRESS]
    ---
    <body — rubric explanation, scoring guide, halt rules>
    

threat-model-author · STRIDE-driven security artefact
    
    
    ---
    name: threat-model-author
    description: Generate a STRIDE threat model for a system change, ready for threat-model-audit.
    version: 1.0.0
    kind: author
    license: Apache-2.0
    allowed-tools: [read_file, write_file, run_audit]
    outputs: [threat-model@1.0]
    references:
      - STRIDE — Microsoft SDL threat-classification taxonomy
      - LINDDUN — privacy-threat extension
    ---

vietnam-mst-validate · a public-runnable vertical-pack skill
    
    
    ---
    name: vietnam-mst-validate
    description: |
      Validate a Vietnamese tax identification number (Mã Số Thuế / MST).
      Supports both 10-digit (corporate) and 13-digit (branch / household-business) forms,
      including check-digit validation per Decision 329/QĐ-TCT (General Department of Taxation).
    version: 0.2.0
    kind: public
    license: MIT
    allowed-tools: []  # pure-function skill — no I/O, network-free
    outputs: [mst-validation-result@1.0]
    ---

vietnam-vat-invoice · stateful + signed + clock-bound
    
    
    ---
    name: vietnam-vat-invoice
    description: |
      Generate a compliant Vietnamese electronic VAT invoice (hóa đơn điện tử) per Decree 123/2020,
      Decree 70/2025 amendment, and Circular 78/2021/TT-BTC. Produces the signed XML and a human
      PDF rendering. Verifies sequence numbering, VAT period, signer cert chain (CMC-CA / VinaPhone-CA / etc.).
    version: 0.3.0
    kind: public
    license: MIT
    allowed-tools:
      - read_file        # invoice draft
      - write_file       # signed XML + PDF
      - sign_with_cert   # PKCS#7 signing through OS keychain
      - read_clock       # invoice timestamp
    outputs: [vn-vat-invoice@1.0]
    ---

### Vietnam vertical pack · per-bundle compliance dossier

The five shipped Vietnamese public skills are the proof-of-pattern for vertical-pack moats. Each ties to one specific Vietnamese regulator + one ISO-style compliance hook, so a global vendor cannot replicate without doing the same local-law homework.

vietnam-mst-validate

**MST** = _Mã Số Thuế_ · the Vietnamese tax identification number

Validates 10- and 13-digit MSTs (corporate vs branch/household). Implements the check-digit algorithm from **Decision 329/QĐ-TCT** (General Department of Taxation). Network-free pure function — safe for any client.

**Regulator:** Tổng cục Thuế (General Department of Taxation, Ministry of Finance)  
**Used by:** chief-financial-officer customer onboarding · chief-procurement-officer supplier registration · any tenant module storing a VN counterparty

vietnam-vat-invoice

**VAT** = _hóa đơn điện tử_ · the electronic VAT invoice mandated for all VN businesses since 2022

Generates compliant XML + PDF per **Decree 123/2020** , **Decree 70/2025** amendment, **Circular 78/2021/TT-BTC**. Handles sequence numbering, VAT period, signer-cert chain (CMC-CA, VinaPhone-CA, etc.), and the GDT (Tổng cục Thuế) acknowledgement loop.

**Regulator:** Tổng cục Thuế (Decree 123) + Ministry of Finance (Circular 78)  
**Used by:** chief-financial-officer billing · chief-accounting-officer monthly-close · INV module · chief-revenue-officer revenue audit

vietnam-bank-transfer

**VietQR** \+ **Napas 247** · the unified Vietnamese instant-payment rails

Generates VietQR strings (EMV-compatible) for inter-bank instant payments through the Napas 247 network. Validates bank codes from the State Bank of Vietnam (SBV) registry and routing-account checksums.

**Regulator:** Ngân hàng Nhà nước Việt Nam (State Bank of Vietnam) · Napas (National Payment Corporation of Vietnam)  
**Used by:** chief-financial-officer outbound payments · INV invoice-payment reconciliation · chief-procurement-officer supplier payouts

vietnam-vneid-integration

**CCCD** = _Căn Cước Công Dân_ · the chipped national ID · **VNeID** = the official mobile app that asserts identity

Wraps VNeID Level-2 KYC + CCCD chip-read attestation. Verifies the citizen-identifier check-digit and the certificate chain through MoPS (C06 — Cục Cảnh sát Quản lý Hành chính về Trật tự Xã hội).

**Regulator:** Bộ Công an (Ministry of Public Security) / C06 · Decree 59/2022 (CCCD chip) · Decree 69/2024 (VNeID identity assertions)  
**Used by:** chief-human-resources-officer onboarding · AUTH high-assurance login · KYC for any high-stakes tenant module

vietnam-legal-compliance

**PDPL** = the Personal Data Protection Decree 13/2023 (Vietnamese GDPR analogue) + **Law 91/2025** follow-on legislation

A meta-scaffold that produces a tenant-specific compliance dossier covering Decree 13/2023 PDPD (Bảo vệ dữ liệu cá nhân), Decree 53/2022 (Cybersecurity Law implementation), Law 91/2025 (Personal Data Protection Law upgrade), and the 72-hour breach-notification clock running parallel to GDPR Art. 33. The output is a Markdown audit pack + a structured YAML dossier consumable by chief-privacy-officer + chief-compliance-officer workflows.

**Regulators:** Bộ Công an (Decree 13 PDPD enforcement) · Bộ Thông tin và Truyền thông (cybersecurity supervision per Decree 53) · National Cybersecurity & Anti-cybercrime Department (A05)  
**Used by:** chief-privacy-officer breach-response-cycle · chief-compliance-officer per-regulatory-filing · chief-trust-officer per-trust-incident-update

### "If you wanted to write a new author+audit pair…"

The catalog has reached steady-state — Sessions I-N shipped 124 workflows with zero new skills surfaced. But the recipe is documented and reproducible. The full discipline lives at [`modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md`](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>); here is the 7-step summary.

  1. **Identify the gap.** The new artefact must be produced by ≥ 3 personas' workflows or by 1 persona's time-critical workflow. If neither, add it as a workflow-only chain through existing skills (Sessions I-N pattern).
  2. **Pick a contract slot.** Add `modules/skill/contracts/<name>/{CONTRACT.md, template.md, CHANGELOG.md}`. The contract defines the output schema; the template is the worked example the author skill produces.
  3. **Scaffold from`_template`.** Copy `modules/skill/_template/author/` → `modules/skill/<name>-author/` and `modules/skill/_template/audit/` → `modules/skill/<name>-audit/`. The scaffolds carry SKILL.md, INVARIANTS.md, PIPELINE.md, references/, envelopes/, acceptance/.
  4. **Write the author body.** Anchor on an authoritative reference (IEEE, ISO, OWASP, regulator). Cite by section. No fabrication — if a fact isn't in a citable source, mark it HITL_PAUSE.
  5. **Write the audit rubric.** Always FM-105+ format. Quantitative gates first, then qualitative rubric (1-5 per dimension), then halt rules (PASS / FAIL / HITL_PAUSE / EXHAUSTED / NO_PROGRESS).
  6. **Run the 8-step audit-fix loop on each.** Until both halves PASS or hit HITL_PAUSE / EXHAUSTED / NO_PROGRESS. The loop is not optional — every author+audit pair shipped from Session A onwards has been through it.
  7. **Register in`modules/skill/README.md` §3.** Then add the pair's row to the appropriate persona `workflows/<name>.md` skill_chain and bump `modules/cuo/README.md` persona-workflow counts.



0

## The bigger picture — three strategic roles

Skill is the only module that plays three distinct strategic roles at once. Reviewing it under any single lens (open-standard runtime, memory integration enabler, or vertical-pack moat) misses two-thirds of its value. The three roles must be held simultaneously. 

Role 1 · Open-standard citizen

🌐

Day-1 distribution reach

Every CyberOS skill loads unmodified in Claude Code, Codex, Cursor, VS Code+Copilot, Goose, Amp, Gemini CLI. The standard is open; we don't invent. Distribution via local cache · OCI registry · `agentskills.io`.

12+ clients · Anthropic Agent Skills spec · OCI + cosign verifiable

Role 2 · memory-protocol enabler

🧠

Capability broker → audit chain

Every skill declares `allowed_memory_scopes`. The capability broker enforces — first-use approval, scope-limited reads against Personal memory + Lumi's memory, every invocation emits a memory row. The capture daemon is a skill bundle. The synthesis sub-skill is a skill bundle.

Per AGENTS.md §3.6 + §11 · MEMORY_AUTOSYNC_DESIGN.md Stages 1–5

Role 3 · Vertical-pack moat

🇻🇳

Locally-defensible differentiation

cyberskill-vn (5 shipped: `vietnam-mst-validate` · `vietnam-vat-invoice` · `vietnam-bank-transfer` · `vietnam-vneid-integration` · `vietnam-legal-compliance`; `vn-tax-filing` planned) is the proof-of-pattern. The same recipe produces cyberskill-sg / -id / -th / -eu / -us packs. Each pack is a defensible local moat a global vendor cannot easily replicate.

Strategy doc §4 Level-4 · vertical packs as the actual compounding asset

### Where Skill sits in the CyberOS dependency graph

graph TB memory["🧠 memory  
Personal + Lumi's  
shipped + Stages 1–5"] AUTH["🔐 AUTH  
tenant + subject identity  
planned"] AI["⚡ AI Gateway  
LLM cost ledger  
planned"] MCP["🔌 MCP Gateway  
tool federation  
planned"] SKILL["🛠 SKILL  
**this module** "] CUO["🎯 CUO  
router · 47 persona workflows · 104 author+audit pairs"] ANY["any tenant module"] EXT["external Agent Skills clients  
Claude / Codex / Cursor / etc."] memory --> SKILL AUTH --> SKILL AI --> SKILL MCP --> SKILL SKILL --> CUO SKILL --> ANY SKILL --> EXT SKILL -. allowed_memory_scopes .-> memory CUO -. uses .-> SKILL classDef shipped fill:#f5ede6,stroke:#45210e,stroke-width:2px classDef self fill:#f9c64f,stroke:#45210e,stroke-width:2.5px classDef planned fill:#f0eee9,stroke:#9c9286,stroke-dasharray:4 3 class memory,CUO,EXT shipped class SKILL self class AUTH,AI,MCP,ANY planned 

Skill is the _only_ module that touches the open external ecosystem (the right-hand "external Agent Skills clients" node) and produces consumable artefacts for it. This is what makes the OSS distribution surface real — a CyberOS skill is an Agent Skills skill, not a CyberOS-proprietary thing. 

1

## Why Skill exists

Pre-2026, every "skills module" in every product is a proprietary container that re-invents discovery, lifecycle, capability grants, and authoring. The result is universal: eager activation O(N) cold-start tax, ambient-authority sandbox holes, bespoke manifests no one else can read. In December 2025 Anthropic released **Agent Skills** as an open standard at `agentskills.io`; within six months Microsoft, GitHub, OpenAI, Cursor, Goose, Amp, Gemini CLI, Mistral, Databricks, Letta, and 15+ others adopted it. Inventing a competing format in 2026 is value-destroying. The 13 May 2026 architectural audit (AUDIT.md) made the call: _adopt the standard verbatim, rebuild the host as a Rust core with Wasmtime, ship the Bun toolchain for TypeScript skill authors, and use the saved engineering to publish a defensible Vietnamese-market skill collection_. 

🌐

Open standard, day-1 reach

Every CyberOS skill loads in Claude Code / Codex / Cursor / VS Code unchanged. Distribution is OCI registry or `agentskills.io`.

🛡

Capability-based sandbox

No ambient authority. Skills declare `allowed-tools`; the host translates to WASI grants. First-use approval; grants persisted by content hash.

🇻🇳

VN-market collection as moat

Five shipped Vietnamese-market public skills (`vietnam-mst-validate`, `vietnam-vat-invoice`, `vietnam-bank-transfer`, `vietnam-vneid-integration`, `vietnam-legal-compliance`; `vn-tax-filing` planned) — defensible differentiation a global vendor can't easily replicate.

2

## What it does — 5W1H2C5M

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is the Skill module?| A Rust host that discovers, validates, activates, invokes, and audits Agent Skills (SKILL.md bundles). Layered with a Bun + esbuild authoring toolchain for TypeScript skill authors and a capability broker enforcing WASI grants.  
**5W · Who**|  Who reads/writes?| **Skill authors** (CyberSkill team + open contributors). **Invokers:** CUO router, agent controllers, the CLI. **Owner:** CTO seat (CEO carries today).  
**5W · When**|  When does activation happen?| Two levels. Level 1 (boot): frontmatter-only indexing, ~100 tokens/skill. Level 2 (activation): body load + WASM compile, triggered by either agent controller or VS-Code-style `activationEvents`.  
**5W · Where**|  Where does it run?| The host is co-located with the user (Tauri or CLI). WASM components run in per-invocation Wasmtime `Store`s pulled from a pool. AOT-compiled components are cached on disk by content hash.  
**5W · Why**|  Why this design?| To get open-standard client compatibility (Agent Skills + MCP) on Day 1 of compliance, sub-millisecond cold-starts via progressive disclosure, capability sandboxing via WASI, and the option to publish to the open registry.  
**1H · How**|  How does invocation work?| Header lookup (O(1) DashMap shard) → capability check → lazy body load + AOT-cached Wasmtime instantiation → execute → audit-log every WASI syscall → return JSON.  
**2C · Cost**|  Cost?| Cold-start over 1,000 skills: <100 ms (header-only). Per-WASM-skill invocation cold: sub-millisecond (AOT cached). Per-native-script invocation: O(process spawn). Memory per activated skill: ~10–50 KB (header) + body bytes (lazy).  
**2C · Constraints**|  Constraints?| (a) The standard is "deliciously tiny" — under-specified in places; we treat `metadata.version` as required for registry-resolved skills. (b) WASI Preview 2 component model is required for executable skills. (c) First-use approval gates `allowed-tools`.  
**5M · Materials**|  What does it use?| Rust 1.80+ · tokio · DashMap · serde_yaml · wasmtime 27+ · zstd · cosign for signature verification · OCI clients · libyaml-backed YAML parser.  
**5M · Methods**|  Method choices?| Progressive disclosure (3 levels) · activation events (VS Code pattern) · component model + WIT (cross-language) · capability-based isolation (no ambient authority) · AOT cache + pool of `Store`s for hot-path concurrency.  
**5M · Machines**|  Where does it run?| macOS / Linux / Windows. WASM runs anywhere Wasmtime runs. AOT artifacts are content-addressed, shareable across hosts.  
**5M · Manpower**|  Who maintains?| 1 IC owner today. Rust runtime is small (~3,000 LoC) and the spec is closed. Skill authoring is open to the team via the Bun toolchain.  
**5M · Measurement**|  How measured?| Cold-start histogram (per-skill), invocation latency p50/p95, capability-grant rate, parity-test pass rate (12/12), Criterion benchmarks on registry hot path.  
  
3

## Architecture

Four Rust crates form the canonical host. A Bun toolchain handles authoring. Skills are distributed as `.skill` bundles (zip of the directory + content hash) resolvable from local cache, OCI registries, or HTTPS URLs. 

graph TB subgraph AUTHORING ["Authoring (Bun toolchain)"] AUTH_BUN["Bun 1.3 + esbuild  
cyberos skill new --lang ts"] AUTH_BUILD["build.ts  
TS → wasm32-wasi component"] AUTH_TEST["bun test"] end subgraph HOST ["Rust host (4 crates)"] MANI_C["crates/manifest  
serde model for SKILL.md frontmatter"] HOST_C["crates/host  
loader · registry · activator · invoker"] RES_C["crates/resolver  
OCI + HTTPS + local cache"] CLI_C["crates/cli  
cyberos-skill-cli"] end subgraph RUNTIME ["Runtime"] REG["DashMap registry  
sharded · read-mostly"] WT["Wasmtime engine  
WASI Preview 2 + Component Model"] POOL["Pool<Store>  
per-skill"] BROKER["Capability broker  
allowed-tools → WASI grants"] SCRIPT["Native-script executor  
(scripts/*.py / *.sh)"] end subgraph DIST ["Distribution"] LOCAL["~/.cyberos/skills/"] OCI["OCI registry  
ghcr.io/cyberskill/*"] HTTPS["HTTPS URL"] REG_OPEN["agentskills.io"] end subgraph CONSUMERS ["Consumers"] CUO_S["🎯 CUO router"] AGENT_S["Claude / Codex / Cursor"] CHAT_S["💬 CHAT / IDE host"] end AUTH_BUN --> AUTH_BUILD AUTH_BUILD --> LOCAL AUTH_TEST --> AUTH_BUILD CLI_C --> HOST_C HOST_C --> MANI_C HOST_C --> RES_C RES_C --> LOCAL RES_C --> OCI RES_C --> HTTPS LOCAL --> REG_OPEN HOST_C --> REG HOST_C --> WT WT --> POOL HOST_C --> BROKER HOST_C --> SCRIPT CUO_S --> CLI_C AGENT_S --> CLI_C CHAT_S --> CLI_C classDef shipped fill:#f5ede6,stroke:#45210e classDef pending fill:#f0eee9,stroke:#9c9286,stroke-dasharray:4 3 class AUTH_BUN,AUTH_BUILD,AUTH_TEST,MANI_C,HOST_C,RES_C,CLI_C,REG,WT,POOL,BROKER,SCRIPT,LOCAL,CUO_S,AGENT_S shipped class OCI,HTTPS,REG_OPEN,CHAT_S pending 

### Crate responsibilities

Crate| Path| Responsibility  
---|---|---  
`cyberos-skill-manifest`| crates/manifest/| Serde model for SKILL.md frontmatter (`name`, `description`, `license`, `compatibility`, `metadata`, `allowed-tools`). `parse_frontmatter` returns manifest + body offset.  
`cyberos-skill-host`| crates/host/| Two-phase loader (boot index → lazy activate). DashMap registry. Invoker with capability checks. Activator pulling Wasmtime `Store`s from a pool.  
`cyberos-skill-resolver`| crates/resolver/| `Resolver` trait — local cache + OCI + HTTPS. Cosign signature verification (refuses unsigned bundles unless `--allow-unsigned`).  
`cyberos-skill-cli`| crates/cli/| `cyberos-skill-cli` binary: list, install, validate, run, build, audit. Single entrypoint for users and CUO.  
  
### Three-level progressive disclosure

Level| When| What's read| Cost  
---|---|---|---  
**L1 · Startup**|  Host boot| SKILL.md frontmatter only · DashMap shard populated| ~100 tokens/skill · parallel · < 100 ms over 1,000 skills  
**L2 · Activation**|  Agent or activation event fires| SKILL.md body (up to ~5,000 tokens) · referenced `references/` files| One-time per skill instance · cached for session  
**L3 · Execution**|  Skill is invoked| `scripts/*.py` or `dist/skill.wasm``assets/*`| WASM cold-start sub-millisecond with AOT cache · native script: process-spawn cost  
  
3.5

## memory integration — the capability broker enforces against Personal & Lumi's memory

Per [MEMORY_AUTOSYNC_DESIGN.md](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>), every skill bundle declares `allowed_memory_scopes` in its SKILL.md frontmatter. The Skill module's **capability broker** (Phase 6 — shipped for legacy scopes; Phase 8 — designed for the universal-protocol memory scopes) enforces these declarations at invocation time: first-use approval, scope-narrowing, audit-chain emission on every read/write, refusal when out-of-scope. The broker is the single point of integration between the open Agent Skills standard and the closed memory protocol; nothing else needs to know about memory to consume the platform's audit-grade memory. 

### SKILL.md frontmatter — memory-aware fields
    
    
    ---
    name: feature-request-author
    version: 0.2.2
    description: Generate audited Feature Request backlog from product brief / spec docs.
    persona: cuo-cpo
    
    # === memory scopes — enforced by capability broker ===
    allowed_memory_scopes:
      read:
        - personal:project:*              # this user's project memories (Personal memory)
        - personal:module:*               # this user's module memories
        - lumi:org:locked-decisions       # org-wide locked decisions (Lumi's memory)
        - lumi:org:shared-rfcs            # team RFCs
        - lumi:synthesis:weekly-*         # synthesised wisdom artefacts
      write:
        - personal:project:*              # write back project memories
        - lumi:fr-decisions               # the FR row itself lands on Lumi's memory if sync_class shareable+
    
    # === MCP tools — orthogonal to memory scopes ===
    allowed_mcp_tools:
      - kb.read
      - kb.search
      - memory.search          # search Personal memory body content
      - memory.write_memory    # canonical put through Writer
      - memory.lumi_search     # search Lumi's memory within tenant scope
      - audit.append          # explicit audit row emission
    
    # === Escalation policy ===
    escalation:
      to_persona_on_legal: cuo-clo
      to_persona_on_security: cuo-cseco
      to_persona_on_compliance: cuo-clo
      to_human_on_irreversible: true
    ---

### Capability broker enforcement flow

sequenceDiagram autonumber participant C as Caller (CUO / agent / CLI) participant S as Skill host (Rust) participant B as Capability broker participant BR as memory protocol participant BC as Personal memory  
(~/.cyberos/memory/store/) participant LB as Lumi's memory  
(cloud tenant) participant W as Wasmtime / script runtime participant A as audit-chain writer C->>S: invoke(skill_id, input) S->>S: lookup SKILL.md header (DashMap O(1)) S->>B: check_grants(skill, allowed_memory_scopes, allowed_mcp_tools) alt first invocation B->>C: surface approval UI (Cowork inline / desktop notif) C-->>B: approved · grants persisted by content hash end B-->>S: grants OK S->>W: instantiate Wasmtime Store with grants applied W->>W: execute skill body W->>BR: read(personal:project:my-rfc) BR->>BC: filesystem read · seqlock BC-->>BR: body + frontmatter BR-->>W: memory record W->>BR: write(lumi:fr-decisions:FR-AUTH-001) BR->>BC: local put (becomes pending-push) BR->>LB: sync orchestrator pushes (next window) LB-->>BR: pushed · confirmed · lumi_chain_hash BR->>A: emit audit row {skill, persona-version, memory ops, mcp tools used} A-->>S: chain advanced W-->>S: skill_output S-->>C: {output, persona_version_stamp, audit_seq} 

### Universal-protocol skills shipped by CyberOS

Two skills are part of the memory protocol surface itself, not just consumers of it. Both ship as SKILL.md bundles to honour the "everything is a skill" architectural rule.

Skill bundle| Role| Stage| Description  
---|---|---|---  
`memory-capture@1`  
modules/skill/memory-capture/| The capture daemon as a skill| 2| Watches filesystem + Cowork session + Claude Code tool calls. Emits memory rows through the canonical Writer. Long-running activation via WASI host call rather than per-invocation execution. `allowed_memory_scopes` = write to `personal:*` only.  
`memory-sync@1`  
modules/skill/memory-sync/| The sync orchestrator| 4| Tails the local audit chain, filters by `sync_class`, pushes shareable+ to Lumi's memory over JWT-authenticated HTTP. Pulls inbound via the same envelope. `allowed_memory_scopes` = read all local + write `lumi:<tenant>:*`.  
`cuo/personas/synthesis-author@1`  
modules/skill/synthesis-author/| Multi-memory auto-evolve| 5| Runs nightly on Lumi's memory tenant compute window. Walks prior 24 h of memories, clusters by topic via BGE-M3 embeddings, deduplicates cross-person decisions, emits `synthesis@1` artefacts (daily / weekly / decisions-pending). `allowed_memory_scopes` = read `lumi:<tenant>:shareable` \+ write `lumi:<tenant>:synthesis`.  
`feature-request-author@0.2.2`  
modules/skill/feature-request-author/| Canonical FR creation| shipped + integrating| Standalone-mode interview or chained-from-RFC; emits `feature_request@1` markdowns to `docs/feature-requests/<module>/` with audit-chain rows. Already ships; integrates with Lumi's memory at Stage 4. Authoring discipline at `modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md`.  
`feature-request-audit@0.1.0`  
modules/skill/feature-request-audit/| FR quality gate| shipped| Chains from feature-request-author. Audits each FR against the AUDIT_RUBRIC for atomicity, BCP-14 compliance, verification method, acceptance criteria. Emits `audit_response@1` alongside the FR. Decision states: PASS / PASS_WITH_REVISIONS / FAIL. Houses the canonical `AUTHORING_DISCIPLINE.md` for FR authors.  
  
3.6

## Vertical-pack pattern — beyond cyberskill-vn

The cyberskill-vn collection (5 shipped · MST + VAT + VietQR + CCCD + legal; tax-filing planned) is not the strategy. It is the _proof-of-pattern_. The strategy is to systematise vertical packs as a repeatable production unit and replicate it across SEA, EU, US, and across cross-cutting verticals (HR, legal, accounting). Per [strategy doc §4.4](<../../strategy/CYBEROS_STRATEGY.md>), vertical packs are **Level 4** productization and the actual compounding margin (70%+ since the base CyberOS is open-source). Each pack is a defensible local moat a global vendor cannot easily replicate. 

### The pack recipe

  1. **Pick the jurisdiction.** Each pack maps to a single regulatory authority (Vietnam MoPS, Singapore PDPC, Indonesia OJK, Thailand RD, EU GDPR, US state-by-state, etc.). One regulator = one pack.
  2. **Identify 5–8 "high-pain, high-frequency" workflows.** The bar: each workflow takes a Vietnamese SME accountant or HR officer ≥ 1 hour per week today; the skill reduces that to ≤ 5 minutes. cyberskill-vn picked MST validation (every customer record), VAT invoicing (every sale), VietQR / Napas (every payment), CCCD / VNeID (every employee onboarding), legal-compliance scaffold (every Decree 13 audit). Tax filing remains the next planned bundle.
  3. **Author each as an Agent Skills bundle.** SKILL.md frontmatter declares `allowed_memory_scopes` \+ `allowed_mcp_tools` \+ jurisdiction-specific compliance metadata. The bundle ships with sample data, golden fixtures, and a parity harness against the regulator's published reference behaviour where available.
  4. **Localise the language layer.** Vietnamese: NFC normalisation, PGroonga tokenisation, Be Vietnam Pro font. Same pattern for Bahasa Indonesia, Thai, Tagalog, Arabic, etc.
  5. **Compliance-verify each skill.** Every cyberskill-vn skill cites the relevant decree article and ships a verification test that exercises the regulator's reference data (where available) or a golden-master fixture extracted from real-world filings.
  6. **Publish to agentskills.io.** Each pack ships under `agentskills.io/cyberskill/<pack>/`. Other Anthropic Agent Skills consumers (Claude, Cursor, etc.) can install from there without any CyberOS dependency.
  7. **Sell via Lumi's memory tenants.** The pack is a paid add-on to the Lumi's memory tenant SKU. Bundle revenue + base SaaS revenue compounds.



### The vertical-pack roadmap

Pack| Geo| Anchor skills| Status| Target ship| Annual unit pricing (planned)  
---|---|---|---|---|---  
`cyberskill-vn`| Vietnam (cornerstone)| vietnam-mst-validate · vietnam-vat-invoice · vietnam-bank-transfer (VietQR) · vietnam-vneid-integration (CCCD) · vietnam-legal-compliance| Shipped 5/5| P0 · start (done)| included in Pro tier · $0 add-on for Vietnam-domiciled tenants  
`cyberskill-sg`| Singapore| sg-uen-validate · sg-paynow-transfer · sg-cpf-contrib · sg-iras-tax-filing · sg-acra-filings · sg-pdpa-compliance| Planned| P3 entry (Singapore HoldCo flip · P3 · start)| USD 250 / tenant / month  
`cyberskill-id`| Indonesia| id-npwp-validate · id-bri-transfer · id-bpjs-contrib · id-djp-tax-filing · id-uupdp-compliance · id-djpb-filings| Planned| P4 (SEA-3 wave · P4 · mid)| USD 200 / tenant / month  
`cyberskill-th`| Thailand| th-tax-id · th-promptpay-transfer · th-sso-contrib · th-rd-tax-filing · th-pdpa-compliance| Planned| P4 (SEA-3 wave · P4 · mid)| USD 200 / tenant / month  
`cyberskill-eu`| EU (post-HoldCo)| eu-vat-validate · eu-sepa-transfer · eu-gdpr-dsar · eu-aia-conformity · eu-ec-mandatory-disclosures · eu-eidas-qtsp| Planned| P3 exit (eu-shard activation · P3 · exit)| EUR 400 / tenant / month  
`cyberskill-us`| United States| us-ein-validate · us-ach-transfer · us-w-2-issue · us-1099-issue · us-irs-940 · us-state-tax (50-state pluggable)| Planned| P4 (post-US-sub) · P4 · mid+| USD 500 / tenant / month  
`cyberskill-hr`| Cross-cutting (HR vertical)| vn-bhxh-bhyt · us-w-2 · eu-employment-contract-templates · sg-cpf · cross-border-payroll-prep| Planned| P2+ (depends on HR module)| USD 150 / tenant / month  
`cyberskill-legal`| Cross-cutting (legal vertical)| contract-review-redline · litigation-tracker · billable-hours · ndr-templates-by-jurisdiction · clm-import| Planned| P3+ (depends on DOC module)| USD 300 / tenant / month  
`cyberskill-accounting`| Cross-cutting (accounting vertical)| gaap-ifrs-reports · year-end-close-orchestration · audit-trail-summarisation · multi-currency-revaluation| Planned| P3+ (depends on INV + REW)| USD 300 / tenant / month  
  
**Margin math:** at 50 paying tenants × avg 2 packs per tenant × USD 300 / pack / month = USD 360k ARR from packs alone, on top of base SaaS. At 500 paying tenants × 3 packs avg = USD 5.4M ARR. The compounding rate is what makes Level-4 the actual moat — base SaaS revenue grows linearly with seat count; pack revenue grows multiplicatively with both seat count _and_ packs-per-tenant. 

3.7

## Distribution roadmap — local → agentskills.io → marketplace

Skill distribution is staged so that each rung has a clear consumer cohort and a clear next-tier graduation criterion. Per the strategy doc §4 (productization levels) and the research review §7.3 verdict ("marketplace is premature; the moat is the memory + the packs"), the marketplace rung is intentionally deferred to ≥ 50 paying tenants. 

Rung| Distribution mechanism| Consumer cohort| Status| Graduation criterion  
---|---|---|---|---  
**R0 · Local cache**| `~/.cyberos/skills/` filesystem discovery (per Anthropic Agent Skills spec)| Developer · author iteration| Shipped| —  
**R1 · .skill bundles**|  zip of the skill folder + content hash; loadable via `cyberos-skill-cli install <file>`| Single-machine distribution; teams without registry| Shipped| OCI registry available  
**R2 · OCI registry**|  Push to any OCI v1.0-conformant registry (GHCR, ECR, ACR, Harbor); pull by digest; cosign signature verification| CyberSkill internal team · OSS contributors| Partial · cosign-verify pending| cosign verify-by-default ships  
**R3 · agentskills.io**|  Submit to the open Anthropic Agent Skills directory at `agentskills.io/cyberskill/<pack>/`| Anthropic Agent Skills ecosystem (Claude products, Cursor, Cline, Codex, VS Code, Goose, Amp, Gemini CLI — 12+ clients via open Agent Skills + MCP standards)| Planned · waits for registry API| registry API stable; cyberskill-vn submitted  
**R4 · marketplace.cyberskill.world**|  CyberSkill's own marketplace UI — vetted-by-CyberSkill badge, revenue share (70/30), per-skill or per-pack pricing, in-tenant install| Lumi's memory paying tenants| Deferred| ≥ 50 paying Lumi's memory tenants  
**R5 · enterprise private marketplace**|  White-label per-enterprise. ISVs publish into the enterprise's private marketplace (not the public one); the enterprise vets + curates| Level-5 Ecosystem-as-a-Service customers (P4+24)| Aspirational| first white-label enterprise deal signed  
  
### Why each rung is gated

**R3 (agentskills.io) waits on the registry API:** Anthropic's directory accepts submissions today but lacks programmatic publishing. We don't want to gate cyberskill-vn distribution on manual upload-and-wait. Once the API is stable (target Q3 2026 per Anthropic roadmap), we ship. 

**R4 (own marketplace) waits on tenant density:** Salesforce AppExchange (~5,000 apps) took 7 years and a buyer-side ecosystem of millions to reach critical mass. Atlassian Marketplace took ~5 years. CyberOS at P4 · GA with 10 paying tenants has no buyer-side density — 3rd-party developers will not contribute. Until paying tenants exceed 50 (per the research review §7.3), the OSS Skill catalog _is_ the marketplace story. Build it as PR/recruiting; don't invest in tooling. 

**R5 (enterprise private marketplace) is the Level-5 endgame:** sells the CyberOS framework itself as a private-cloud branded platform. "Acme Corp Operating System, powered by CyberOS." 80%+ margins on multi-year contracts. Not before P4+24. 

4

## Data model

erDiagram SKILL_BUNDLE ||--|| SKILL_MANIFEST: "ships" SKILL_BUNDLE ||--o{ SCRIPT_FILE: "may contain" SKILL_BUNDLE ||--o{ REFERENCE_FILE: "may contain" SKILL_BUNDLE ||--o{ ASSET_FILE: "may contain" SKILL_BUNDLE ||--o| WASM_COMPONENT: "may ship" SKILL_MANIFEST ||--o{ ALLOWED_TOOL: "declares" ACTIVATED_SKILL ||--|| SKILL_HEADER: "instance of" ACTIVATED_SKILL ||--o| WASM_COMPONENT: "wraps" ACTIVATED_SKILL ||--o{ INVOCATION: "records" INVOCATION ||--o{ CAPABILITY_GRANT: "requires" CAPABILITY_GRANT ||--|| GRANT_RECORD: "persists as" GRANT_RECORD ||--|| CONTENT_HASH: "bound to" SKILL_BUNDLE { string bundle_id PK "content hash" string source "local | oci | https" string oci_ref string cosign_sig int64 size_bytes } SKILL_MANIFEST { string name PK "1-64 chars, lowercase + hyphens" string description "1-1024 chars" string license string compatibility obj metadata "version, author, region" string allowed_tools_raw } SKILL_HEADER { string name PK string skill_dir int body_offset int64 file_size } ACTIVATED_SKILL { string name FK string body bool has_wasm int64 invocations "AtomicU64" int64 last_used_unix_ms bool revoked } ALLOWED_TOOL { string capability "read_file | write_file | fetch_url(pattern) | …" string pattern_or_path bool experimental } WASM_COMPONENT { string component_id PK "content hash" bytes wasm_bytes string aot_cache_key string wit_interface } SCRIPT_FILE { string path PK "scripts/*.py | *.sh" string interpreter } REFERENCE_FILE { string path PK "references/*.md" } ASSET_FILE { string path PK "assets/*" } INVOCATION { string invocation_id PK string skill_name FK int64 started_at_ns int64 ended_at_ns int exit_code string executor "wasm | script | inline" } CAPABILITY_GRANT { string skill_name FK string capability string granted_by_actor int64 granted_at_ns } GRANT_RECORD { string filepath PK "~/.cyberos/grants.json entry" string content_hash FK "skill must re-approve if changed" } CONTENT_HASH { string hash PK "sha256" } 

5

## API surface

### GraphQL subgraph (planned · P0+)
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key"])
    
    type Skill @key(fields: "name") {
     name: String!
     version: String
     description: String!
     license: String
     region: String # e.g. "VN" | null
     allowedTools: [String!]!
     hasWasm: Boolean!
     installSource: InstallSource!
     contentHash: String!
     invocationsLast24h: Int!
    }
    
    type Invocation @key(fields: "invocationId") {
     invocationId: ID!
     skillName: String!
     actor: String!
     exitCode: Int!
     startedAt: DateTime!
     endedAt: DateTime!
     executor: Executor!
     capabilitiesGranted: [String!]!
    }
    
    type CapabilityGrant @key(fields: "skillName") {
     skillName: String!
     capability: String!
     grantedBy: String!
     grantedAt: DateTime!
     contentHashAtGrant: String!
     stillValid: Boolean! # false if skill rebuilt
    }
    
    enum InstallSource { LOCAL OCI HTTPS BUILTIN }
    enum Executor { WASM SCRIPT INLINE }
    
    type Query {
     skill(name: String!): Skill
     skills(region: String, hasWasm: Boolean): [Skill!]!
     invocation(invocationId: ID!): Invocation
     capabilityGrants(skillName: String): [CapabilityGrant!]!
    }
    
    type Mutation {
     installSkill(source: String!): Skill! # OCI ref or HTTPS
     invokeSkill(name: String!, input: JSON!): Invocation!
     grantCapability(skillName: String!, capability: String!): CapabilityGrant!
     revokeSkill(name: String!, note: String): Skill!
    }

### MCP tool catalogue

Tool name| Inputs| Outputs| Annotations  
---|---|---|---  
`skill.list`| region?| `Skill`| readonly · cached · scope=read  
`skill.describe`| name| `{manifest, body}`| readonly · L2 activation · scope=read  
`skill.invoke`| name, input, caps_requested| `Invocation`| destructive · scope=invoke · cap-gated  
`skill.validate`| path| `{valid, errors}`| readonly · pure · scope=read  
`skill.audit`| skillName?| `CapabilityGrant`| readonly · scope=audit  
  
### CLI — `cyberos-skill-cli`

Subcommand| Purpose| Example  
---|---|---  
`list`| Enumerate indexed skills| `cyberos-skill-cli list --region VN`  
`install`| Resolve + cosign-verify + cache| `cyberos-skill-cli install ghcr.io/cyberskill/vietnam-mst-validate:0.2.0`  
`validate`| Spec-validate a SKILL.md| `cyberos-skill-cli validate./skills/my-skill/`  
`run`| Invoke a skill| `cyberos-skill-cli run vietnam-mst-validate --executor wasm`  
`build`| Bun toolchain build| `cyberos-skill-cli build --lang ts./src/`  
`audit`| Capability grants report| `cyberos-skill-cli audit --since 7d`  
`revoke`| Mark a skill revoked| `cyberos-skill-cli revoke vn-bad-skill --note "CVE-2026-x"`  
`fixtures`| Run skill fixtures| `cyberos-skill-cli fixtures vietnam-mst-validate`  
  
6

## Key flows

### Flow 1 — Install & index

sequenceDiagram autonumber participant U as User / CUO participant CLI as cyberos-skill-cli install participant R as resolver participant CS as cosign verify participant L as loader.index_one participant REG as DashMap registry U->>CLI: install ghcr.io/cyberskill/vietnam-mst-validate:0.2.0 CLI->>R: resolve(oci_ref) R->>R: pull bundle · save to ~/.cyberos/skills/ R-->>CLI: bundle_path CLI->>CS: verify cosign signature alt unsigned CS-->>CLI: refuse (unless --allow-unsigned) end CS-->>CLI: ok CLI->>L: parse SKILL.md frontmatter L->>L: validate name matches directory L->>REG: insert_header(SkillHeader) REG-->>CLI: indexed CLI-->>U: installed (name, version, hash) 

### Flow 2 — Invocation with capability check

sequenceDiagram autonumber participant C as Caller (CUO / agent) participant I as Invoker participant REG as DashMap registry participant B as Capability broker participant ACT as Activator participant W as Wasmtime Store (from pool) participant A as Audit (memory) C->>I: invoke(name, input, requested_caps) I->>REG: get_header(name) REG-->>I: SkillHeader (O(1)) I->>B: check requested_caps subset of declared alt cap denied B-->>I: CapabilityDenied I-->>C: error end B->>B: ensure first-use approval recorded I->>ACT: activate(header) alt cold activation ACT->>ACT: read body · compile WASM · cache AOT end ACT-->>I: ActivatedSkill (Arc) I->>W: pull Store from pool W->>W: instantiate component · build WasiCtx with grants W->>W: call run export W-->>I: result JSON I->>W: return Store to pool I->>A: log invocation row to memory I-->>C: {result, exit_code, elapsed_us} 

### Flow 3 — Capability approval (first-use)

sequenceDiagram autonumber participant U as Operator participant I as Invoker participant B as Capability broker participant G as ~/.cyberos/grants.json participant H as content_hash check I->>B: ensure_granted(skill, capability) B->>G: lookup (skill, capability) alt not found B->>U: PROMPT — "vietnam-mst-validate requests read_file + write_file" U-->>B: approve / deny alt deny B-->>I: GrantDenied end B->>H: compute current content_hash B->>G: persist {skill, capability, content_hash, granted_by, granted_at} end G-->>B: existing grant B->>H: re-verify content_hash alt hash mismatch (skill rebuilt) B-->>I: GrantStale — re-prompt required end B-->>I: granted 

### Flow 4 — Python ↔ Rust parity check

sequenceDiagram autonumber participant T as Parity harness (pytest) participant FX as 12 golden fixtures participant PY as Python runner (legacy) participant RS as Rust host T->>FX: load fixture (input.json + expected.json) par parallel run T->>PY: python -m runners.fr_with_tasks <fixture> T->>RS: cyberos-skill-cli run <skill> < input.json end PY-->>T: py_output RS-->>T: rs_output T->>T: assert py_output == rs_output (byte-identical) alt mismatch T-->>T: FAIL — gate blocks Phase 7 retirement end T-->>T: 12/12 PASS — Rust host is canonical 

The parity harness is the single objective gate that lets us delete the legacy Python runner at Phase 7. Today: 12/12 green.

7

## Skill lifecycle

stateDiagram-v2 [*] --> Authored: skill author writes SKILL.md (Bun toolchain) Authored --> Built: cyberos-skill-cli build (TS → wasm32-wasi) Built --> Signed: cosign sign bundle Signed --> Published: push to OCI registry / agentskills.io Published --> Installed: cyberos-skill-cli install {oci_ref} Installed --> Indexed: loader reads frontmatter into DashMap Indexed --> Activated: agent / activation event triggers body load + AOT compile Activated --> Invoked: invoker pulls Store · runs · audits Invoked --> Activated: re-invoke (cached) Invoked --> Recorded: memory audit row written Activated --> Revoked: cyberos-skill-cli revoke Revoked --> Retired: legacy retirement runbook (30-day soak then delete) Retired --> [*] note right of Indexed Header in registry ~100 tokens · O(1) lookup end note note right of Activated Body loaded · WASM cached AOT artifact on disk end note 

8

## The 5 `cyberskill-vn` skills

The Vietnamese-market collection. Each is a valid Agent Skill that loads unchanged in Claude Code / Codex / Cursor / VS Code. All are MIT- or Apache-2.0-licensed, network-free where possible, and ship with a parity-tested Python reference implementation for audit. 

### 🇻🇳 `vietnam-mst-validate` v0.2.0 · region:VN · MIT

Validate a Vietnamese tax code (Mã số thuế). Per General Department of Taxation regulations, an MST is either 10 digits (legal entity) or 13 digits (branch — 10 digits + '-' + 3 digits). The skill returns a structured `{ok, format, reason?}`. 

Quickstart
    
    
    $ cyberos-skill-cli run vietnam-mst-validate --input '{"mst":"0123456789-001"}'
    {"ok": true, "format": "13-digit-branch"}

When to use

User says "kiểm tra MST", "validate tax code", "mã số thuế 0123...". Routed automatically by CUO via the `mst`, `tax code`, `ma so thue` keywords.

Capabilities

`read_file` · no network

### 🇻🇳 `vietnam-vat-invoice` v0.3.0 · region:VN · Apache-2.0

Generate Vietnamese VAT-compliant electronic invoices (Hoá đơn GTGT điện tử) from a structured JSON line-item list. Produces XML conforming to General Department of Taxation schema v3.0. Validates buyer MST via the `vietnam-mst-validate` dependency (Phase 3 chain). 

Quickstart
    
    
    $ cyberos-skill-cli run vietnam-vat-invoice --input '{
     "buyer_mst": "0123456789",
     "seller_mst": "9876543210",
     "lines": [{"item":"Consulting", "qty":1, "unit_price":10000000, "tax_rate":0.10}]
     }'
    <Invoice xmlns="urn:vn:gdt:v3">
     <BuyerTaxCode>0123456789<BuyerTaxCode>
     <LineItem>...<TaxAmount>1000000<TaxAmount><LineItem>
    <Invoice>

When to use

User says "tạo hoá đơn", "xuất hoá đơn GTGT", "e-invoice Vietnam". CUO keywords: `invoice`, `hoa don`, `vat`, `gtgt`, `e-invoice`, `xuat hoa don`.

Capabilities

`read_file``write_file` · no network. Round-half-up VAT per line.

### 🇻🇳 `vietnam-bank-transfer` v0.1.0 · region:VN · MIT

Napas 24/7 + VietQR generator. Given a bank short-code, an account number, and an amount, produces (a) a VietQR EMVCo string for client-side QR rendering and (b) a Napas 24/7 transfer instruction payload. Does NOT execute transfers — that always defers to human approval. 

Quickstart
    
    
    $ cyberos-skill-cli run vietnam-bank-transfer --input '{
     "bank": "VCB", "account":"0011001234567", "amount_vnd": 5000000,
     "memo": "Refund INV-2026-014"
     }'
    {
     "vietqr_emv": "00020101021238540010A0000007270124000697040401120011001234567...",
     "napas_payload": {...},
     "render_url": "https://img.vietqr.io/image/VCB-0011001234567-..."
    }

When to use

User says "tạo QR chuyển khoản", "VietQR", "Napas 247". CUO keywords: `transfer`, `qr`, `chuyen khoan`, `vietqr`, `napas`, `ma qr`.

Capabilities

No filesystem · no network. Deterministic generation only.

### 🇻🇳 `vietnam-vneid-integration` v0.1.0 · region:VN · MIT

CCCD validator + VNeID API scaffolding. Validates 12-digit Citizen Identification Card (Căn cước công dân) checksums and shapes an API request to the VNeID gateway for downstream identity verification (gateway access requires PDPL Decree 13/2023 consent — out of scope for this skill, which only prepares the request). 

Quickstart
    
    
    $ cyberos-skill-cli run vietnam-vneid-integration --input '{"cccd":"079203012345"}'
    {
     "valid": true,
     "structure": {"province_code":"079", "year_of_birth_century":"2", "gender_code":"0"},
     "vneid_request_payload": {...},
     "next_action": "POST to https://vneid.gov.vn/api/v2/verify (requires consent flow)"
    }

When to use

User says "kiểm tra CCCD", "validate citizen ID", "VNeID lookup". CUO keywords: `cccd`, `citizen id`, `can cuoc`, `vneid`, `id card`, `danh tinh`.

Capabilities

No filesystem · no network (scaffolds the request; caller is responsible for the call + consent record).

### 🇻🇳 `vietnam-legal-compliance` v0.1.0 · region:VN · CC-BY-SA-4.0

Reference-only procedural knowledge for Vietnamese legal/compliance: PDPL Law 91/2025 (incl. Art. 38 SME grace) · Decree 13/2023 (Personal Data) · Decree 53/2022 (Cyber Security) · Decree 356/2025 (PDPL implementing). Markdown-only skill (no executable code) that loads procedural guidance into the agent's context on activation. 

Quickstart
    
    
    $ cyberos-skill-cli run vietnam-legal-compliance --input '{"topic":"DSAR fulfilment under PDPL Art. 14"}'
    {
     "level": "instructions",
     "guidance": "# DSAR fulfilment under PDPL Art. 14\n\n1. Verify identity via VNeID or...\n2. Within 30 days:...\n3. Encryption envelope review (Decree 13/2023 Art. 17)..."
    }

When to use

User says "compliance check", "decree review", "PDPL question". CUO keywords: `compliance`, `law`, `decree`, `nghi dinh`, `thong tu`, `pdpl`, `cybersecurity`.

Capabilities

Markdown-only — zero runtime cost.

### 🇻🇳 `vietnam-tax-filing` planned · region:VN

_Planned 6th cyberskill-vn skill._ Monthly + quarterly VAT return helper to aggregate `vietnam-vat-invoice`-emitted XML invoices across a reporting period, sum output VAT and input VAT credits, and shape the tax return XML for the GDT online filing portal. Submission stays a deferred-to-human action. **Not yet shipped — the 5 cyberskill-vn skills currently in`modules/skill/public/` do not include this one.**

9

## Audit phases 0–7 — outcomes

The 13 May 2026 architectural audit (`skill/docs/AUDIT.md`) prescribed a seven-phase migration from the legacy in-house format to the open Anthropic Agent Skills standard. All seven phases are now done; Phase 7 is a 30-day soak window before legacy code deletion. 

0Phase 0 — Inventory & freeze shipped

Catalogued every legacy skill. Froze the legacy format — no new bespoke skills accepted past Day 0. Stood up `cyberos-skill-cli validate` that parses both legacy and SKILL.md formats and emits a diff.

1Phase 1 — Rust + Bun scaffold · spec validator shipped

Stood up the 4-crate Rust workspace (manifest · host · resolver · cli) and the Bun + esbuild authoring toolchain. Added the SKILL.md loader alongside the legacy loader behind a `--skills-format=both|legacy|standard` flag. Audited deltas vs the open spec.

2Phase 2 — Parity harness (Python ↔ Rust) shipped · 12/12 green

Built a property-test harness that asserts byte-identical agent outputs across both runners for the entire catalogue. 12/12 fixtures pass. This is the single objective gate for Phase 7.

3Phase 3 — Executor selection flag · default to Rust shipped

Flipped the default from legacy Python to Rust host. Legacy runner remains compiled in for the soak window. Announced deprecation; the `cyberskill-vn` collection is now resolvable from the open registry.

4Phase 4 — DashMap registry + Criterion benchmarks shipped · ≥ 2× at contention

Replaced the legacy global `Mutex<HashMap>` with `Arc<DashMap>` (64 shards). Criterion microbenchmarks show ≥ 2× throughput at 4+ concurrent invocations on a commodity laptop. Registry hot path is no longer a bottleneck.

5Phase 5 — Wasmtime engine + AOT cache + componentize scaffolded · runtime gated

Wasmtime executor lands behind `--exec=script|wasm|auto`. Auto selects WASM when `dist/skill.wasm` is present. AOT artifacts cached on disk by content hash. Bun toolchain compiles TS skills to `wasm32-wasi` components. Runtime is feature-gated on user install per `docs/PHASE_5_ACTIVATION.md`.

6Phase 6 — Capability broker GA shipped

Capability enforcement flipped from `warn` to `deny`. Operators approve each skill's `allowed-tools` on first use; the grant is recorded in `~/.cyberos/grants.json` bound to the skill's content hash. `cyberos-skill-cli audit` reports grants.

7Phase 7 — Legacy retirement runbook ready · 30-day soak

Runbook ready (`docs/PHASE_7_RETIREMENT.md`). Executes after a 30-day soak with zero P0 incidents on the new defaults. Deletes the legacy loader, registry primitive, and executor; tags a new major version.

8

## Functional Requirements

The CyberOS FR catalogue is being rebuilt one feature at a time via the open [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) Agent Skill. Authoring discipline lives at `modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md`.

Previous FR enumerations were archived 2026-05-14 and are no longer reflected on this page. Specific FRs land here as they are re-authored.

11

## Non-Functional Requirements

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`N(FR pending)`| Cold-start (1,000 skills indexed)| ≤ 100 ms · parallel| Criterion bench · macOS M2  
`N(FR pending)`| Header lookup p99| ≤ 5 µs (DashMap shard read)| Criterion bench  
`N(FR pending)`| WASM invocation cold-start (AOT cached)| ≤ 1 ms| wasm_invoke bench  
`N(FR pending)`| Native-script invocation| ≤ 50 ms (process spawn dominated)| script_invoke bench  
`N(FR pending)`| Trivial native-script throughput| ≥ 10,000 inv/sec| Criterion mixed workload  
`N(FR pending)`| WASM with capability checks throughput| ≥ 1,000 inv/sec| Criterion bench  
`N(FR pending)`| Python ↔ Rust parity| 12/12 fixtures byte-identical| pytest parity harness · CI  
`N(FR pending)`| Memory per indexed skill| ≤ 1 KB (header)| process RSS · 1,000-skill load  
`N(FR pending)`| Unsigned bundle refusal| 100% (unless `--allow-unsigned`)| install_test suite  
`N(FR pending)`| Open spec compliance| 100% — skill loads in Claude Code unmodified| cross-host loader test  
  
12

## Dependencies

graph LR subgraph upstream ["Skill depends on"] AUTH["🔐 AUTH  
actor identity"] end SKILL_M["🛠 Skill host"] subgraph downstream ["Used by"] CUO_M["🎯 CUO router"] AGENT_M["External agents"] EVERY["Every module that invokes a skill"] end AUTH --> SKILL_M SKILL_M --> CUO_M SKILL_M --> AGENT_M SKILL_M --> EVERY classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#fef6e0,stroke:#9c750a class SKILL_M,CUO_M,AGENT_M shipped class AUTH,EVERY planned 

13

## Compliance scope

Regulation / standard| Article / clause| SKILL feature that satisfies it  
---|---|---  
EU AI Act| Art. 12 — Logging| Every invocation is an audit row · WASI syscalls intercepted at `info`; sensitive grants logged at `warn`.  
EU AI Act| Art. 26 — Human oversight| First-use approval gate · operator must approve each `allowed-tools` set.  
NIST SSDF (SP 800-218)| § PW.4 — Code provenance| Cosign signature verification on install · refuses unsigned bundles by default.  
SLSA Level 3| Build integrity| SBOM provenance per bundle (planned P1) · AOT artifacts content-hashed.  
ISO/IEC 27001:2022| A.8.25 — Secure development| Capability-based sandboxing (defence in depth) — no ambient authority.  
ISO/IEC 42001 (AIMS)| § 8.3 — Operational planning| Phase 7 retirement runbook · soak windows · reversible flag-driven migration.  
OWASP Top 10 LLM| LLM03 — Training data poisoning| Skill bodies are loaded into LLM context — covered by content-hash binding so a modified skill must be re-approved.  
  
14

## Risk entries

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-SKILL-001`| Capability escape (WASI grant misconfiguration)| Low| High| CSO| Default empty grant set · first-use approval · WASI enforces at host syscall layer regardless of declaration · audit interception logs every syscall.  
`R-SKILL-002`| Supply-chain attack on third-party skill| Medium| High| CSO| Cosign verification mandatory · OCI registry checks · content-hash binding means modified skill triggers re-approval · audit table queryable.  
`R-SKILL-003`| WASM zero-day in Wasmtime runtime| Low| High| CTO| Track wasmtime security advisories · pinned version in `Cargo.toml` · staged rollout on bumps · skills fall back to script executor if runtime down.  
`R-SKILL-004`| Agent Skills spec drift (governance under AAIF / Linux Foundation)| Medium| Medium| CTO| Track `agentskills/agentskills` \+ AAIF mailing list · pin spec revision per release · CyberOS-specific extensions live under `metadata.cyberos-*`.  
`R-SKILL-005`| Parity regression (legacy Python ↔ Rust diverges)| High| Medium| CTO| 12/12 parity harness blocks Phase 7 retirement · CI gate on every PR · soak window mandatory.  
`R-SKILL-006`| VN-skill regulatory churn (GDT schema changes, PDPL evolution)| High| Medium| CLO| Skill versions follow GDT/PDPL revisions · CHANGELOG newest-first · semver compatibility tested vs prior outputs.  
`R-SKILL-007`| Python-on-WASI heavy cold-start regression| Medium| Medium| CTO| Default executable skills to Rust or TS-via-Bun guests · Python flagged "preview" · benchmark cold-start per skill in CI · alert > 100 ms p95.  
memory-integration risks (Phase 8 designed)  
`R-SKILL-008`| **Capability broker bypass via WASI host call** — skill emits memory write outside declared `allowed_memory_scopes`| Low| Catastrophic| CSO| Broker mediates EVERY memory write via the canonical Writer trait. WASI host call cannot reach the filesystem layer directly; only the broker can. Conformance test in CI: synthetic skill attempts out-of-scope write, expects 403. `cyberos doctor` invariant `skill-broker-mediated-only` warns if any audit row's actor matches a skill but bypasses the broker signature.  
`R-SKILL-009`| **Multi-tenant skill bleed** — skill invocation in tenant A reads/writes tenant B's Lumi's memory row| Low| Catastrophic| CSO| Lumi's memory row reads scoped by Postgres RLS on `tenant_id` · JWT issued by AUTH carries `tenant_id` claim · capability broker rewrites `lumi:<tenant>:*` scopes per invoker · weekly chaos-test attempts cross-tenant read and asserts 403. Same control plane as R-memory-009.  
`R-SKILL-010`| **Sync-state corruption from skill** — skill emits Writer ops out of order, breaking the sync orchestrator's queue invariants| Low| High| CTO| Writer is single-threaded per memory, lease-protected; the broker uses the Writer's append API so ordering is enforced. Sync state machine is authoritative — never derived from skill output. `sync-state-monotonic` invariant on the manifest.  
`R-SKILL-011`| **Synthesis sub-skill PII leak** — Stage 5 synthesis pass extracts PII into a wisdom artefact, then propagates back to all team members on pull| Medium| High| DPO| Synthesis sub-skill's `allowed_memory_scopes` = read only `shareable`\+ (private memories are never visible) · output artefact runs through Presidio at write time · weekly review of any artefact flagged for PII leak · kill-switch if synthesis-useful-rate drops < 50%.  
`R-SKILL-012`| **Vertical-pack legal drift** — a vertical pack's regulatory citations fall behind the actual regulator's published rules (e.g. PDPL evolution, new GDT VAT decree)| High| Medium| CLO| Per-pack CHANGELOG newest-first. Per-pack legal owner (CLO consult). Quarterly regulatory-drift review meeting. `vietnam-legal-compliance` skill explicitly tracks decree-revision dates. Each pack ships a "last verified against regulator" date in frontmatter.  
`R-SKILL-013`| **OCI registry signing-key compromise** — attacker steals cosign key, signs a malicious skill version, distributes via OCI| Low| Catastrophic| CSO| Cosign keys stored in KMS · per-pack signing keys (compromise scope-limits to one pack) · Rekor transparency log for all signatures · CyberSkill rotates signing keys quarterly · all clients pin acceptable cosign issuers in their `cyberos memory config`.  
`R-SKILL-014`| **agentskills.io marketplace policy hostile** — Anthropic or AAIF changes directory policy to disallow vendor-specific extensions (`cyberos-*` frontmatter), forcing pack rewrites| Low| Medium| CTO| Track AAIF + Anthropic upstream governance · CyberOS-specific fields live under `metadata.cyberos-*` namespace (always strippable) · packs work unmodified in non-CyberOS clients (the `cyberos-*` fields are ignored, not required).  
  
15

## KPIs

KPI| Formula| Source| Target| Current  
---|---|---|---|---  
**Install success rate**|  successful_installs / attempts| install_log| ≥ 99%| 100% (local · OCI pending)  
**Capability grant rate**|  granted / requested| grants.json audit| ≥ 90% (sensible asks)| ~ 95% (small sample)  
**Invocation success rate**|  exit_code 0 / total| memory audit replay| ≥ 98%| 100% (12 fixtures)  
**Parity coverage**|  fixtures matching byte-identical| parity harness| 100%| 12/12 (100%)  
**Cold-start (1k skills)**|  boot_to_indexed_ms| Criterion bench| ≤ 100 ms| ~ 65 ms (M2)  
**WASM invocation p95**|  activate_to_return| Criterion bench| ≤ 5 ms| ~ 1.4 ms (AOT-cached)  
**Ecosystem reach**|  compatible clients| open spec adoption| ≥ 20| 26+  
**VN-skill catalog growth**|  skills shipped (monthly)| git log| ≥ 1/month at P1| 6 shipped (lifetime)  
memory-integration KPIs (Phase 8 — measured once shipped)  
**Broker-mediated rate**| `broker_mediated_writes / total_memory_writes_from_skills`| audit-chain replay| 100% (every skill memory write MUST go through broker)| — (Phase 8)  
**First-use approval latency**| `time(approval) - time(prompt)` p50| capability broker logs| ≤ 30 s · alert if > 5 min (user abandoned)| — (Phase 8)  
**Capability scope reject rate**| `denied_invocations / total`| broker logs| ≤ 0.5% under steady state · spike → mis-declared `allowed_memory_scopes`| — (Phase 8)  
**Synthesis sub-skill emit rate**| `synthesis_artefacts_per_tenant_per_day`| Lumi's memory audit chain| 1 daily + 1 weekly + ≤ 3 decisions-pending per active tenant| — (Phase 8 / Stage 5)  
**Vertical-pack tenant attach rate**| `tenants_with_≥_1_pack / total_paying_tenants`| TEN module billing| ≥ 60% within 6 months of pack launch| — (P3+)  
**Vertical-pack revenue share**| `pack_revenue / total_recurring_revenue`| TEN module billing| ≥ 30% at P4 · mid (the compounding moat)| — (P3+)  
**Marketplace publish-to-install**| `installs_within_30d / publishes`| agentskills.io directory · own marketplace| ≥ 5 installs per published pack within 30 days · floor of 1 for niche packs| — (R3+)  
**Pack legal-drift detection**| `days_since_last_regulator_verification` per pack| pack CHANGELOG + jurisdictional review log| ≤ 90 days for tax/legal packs · ≤ 180 days for HR/accounting packs| — (P3+)  
  
16

## RACI matrix

Activity| CEO| CTO| CPO| CSO| CLO  
---|---|---|---|---|---  
Host runtime (Rust crates)| A| R| I| C| I  
Skill authoring SDK (Bun toolchain)| I| A/R| C| I| I  
Spec compliance (Agent Skills)| I| A/R| C| I| I  
Capability broker policy| I| R| I| A| C  
Cosign signing infrastructure| I| R| I| A| I  
cyberskill-vn collection authoring| I| C| A/R| I| C  
VN regulatory tracking (GDT, PDPL)| I| I| C| I| A/R  
Phase 7 retirement signoff| A| R| C| C| I  
memory-integration + vertical-pack + distribution additions  
Phase 8 capability broker → memory integration| I| A/R| I| C| I  
Synthesis sub-skill spec + implementation (Stage 5)| C| R| A| C| I  
memory-capture@1 + memory-sync@1 skill bundles| I| A/R| C| C| I  
cyberskill-sg pack authoring (P3 entry)| I| C| A/R| I| C  
cyberskill-eu pack (post-HoldCo)| I| C| A| I| R  
cyberskill-us pack (post-US sub)| C| C| A/R| I| R  
agentskills.io directory submission (R3)| I| C| A/R| I| I  
marketplace.cyberskill.world (R4)| A| R| C| C| C  
Quarterly regulatory-drift review (per pack)| I| I| C| I| A/R  
  
17

## CLI usage — real examples

### 1\. List installed skills
    
    
    $ cyberos-skill-cli list
    
    ╭──────────────────────────────────────┬─────────┬────────┬───────────╮
    │ name │ version │ region │ executor │
    ├──────────────────────────────────────┼─────────┼────────┼───────────┤
    │ product-requirements-document-author │ 0.4.1 │ — │ inline │
    │ feature-request-author │ 0.3.0 │ — │ inline │
    │ software-requirements-specification-author │ 0.2.0 │ — │ inline │
    │... (104 author+audit pairs total) │ │ │ │
    │ cyberskill-vn/vietnam-mst-validate │ 0.2.0 │ VN │ script │
    │ cyberskill-vn/vietnam-vat-invoice │ 0.3.0 │ VN │ script │
    │ cyberskill-vn/vietnam-bank-transfer │ 0.1.0 │ VN │ wasm │
    │ cyberskill-vn/vietnam-vneid-integration │ 0.1.0 │ VN │ script │
    │ cyberskill-vn/vietnam-legal-compliance │ 0.1.0 │ VN │ inline │
    ╰──────────────────────────────────────┴─────────┴────────┴───────────╯
    
    208 bundles indexed (104 pairs) + 5 public Vietnamese skills · cold-start 65 ms · all parity-verified

### 2\. Install from OCI registry (with cosign verification)
    
    
    $ cyberos-skill-cli install ghcr.io/cyberskill/vietnam-mst-validate:0.2.0
    
    [resolver] pulling ghcr.io/cyberskill/vietnam-mst-validate:0.2.0... OK (842 KB)
    [cosign] verifying signature... OK (key=cyberskill-release)
    [manifest] parsing SKILL.md frontmatter... OK
    [manifest] name=vietnam-mst-validate version=0.2.0 region=VN
    [registry] inserting header into DashMap (shard 14)
    [install] cached at ~/.cyberos/skills/vietnam-mst-validate-0.2.0/
    [install] content_hash sha256:a3b8d4...

### 3\. Validate a skill manifest
    
    
    $ cyberos-skill-cli validate./modules/skill/public/vietnam-mst-validate/
    
    [validate] reading SKILL.md... OK (1.3 KB)
    [validate] frontmatter parse... OK
    [validate] field: name=vietnam-mst-validate OK (dir-match)
    [validate] field: description (412 chars) OK
    [validate] field: license=MIT OK (SPDX)
    [validate] field: metadata.version=0.2.0 OK (SemVer)
    [validate] field: metadata.region=VN OK
    [validate] field: allowed-tools=read_file OK (declared)
    [validate] referenced files: references/format.md OK
    [validate] ✅ valid Agent Skill · would load in Claude Code, Codex, Cursor, VS Code

### 4\. Run a skill
    
    
    $ echo '{"mst":"0123456789-001"}' | cyberos-skill-cli run vietnam-mst-validate --executor script
    
    [invoke] skill=vietnam-mst-validate executor=script
    [broker] declared allowed-tools: [read_file]
    [broker] requested: -- no caps needed for this invocation
    [broker] ok (within declared set)
    [invoke] script: scripts/validate_mst.py
    [invoke] elapsed_ms=24 exit_code=0
    
    {"ok": true, "format": "13-digit-branch"}

### 5\. Audit capability grants
    
    
    $ cyberos-skill-cli audit --since 7d
    
    ╭───────────────────────────────────┬────────────────────────┬──────────────┬──────────╮
    │ skill │ capability │ granted_by │ valid? │
    ├───────────────────────────────────┼────────────────────────┼──────────────┼──────────┤
    │ vietnam-mst-validate │ read_file │ stephen │ ✓ │
    │ vietnam-vat-invoice │ read_file │ stephen │ ✓ │
    │ vietnam-vat-invoice │ write_file │ stephen │ ✓ │
    │ vietnam-bank-transfer │ (none — pure compute) │ — │ ✓ │
    │ vietnam-vneid-integration │ (none — pure compute) │ — │ ✓ │
    │ vietnam-legal-compliance │ (inline · no exec) │ — │ ✓ │
    ╰───────────────────────────────────┴────────────────────────┴──────────────┴──────────╯
    
    6 grants on file · 0 stale · 0 revoked

### 6\. Build a TypeScript skill (Bun toolchain)
    
    
    $ cd skills/my-new-skill/
    $ cyberos-skill-cli build --lang ts./src/
    
    [bun] installing deps... OK (240 ms)
    [esbuild] bundling src/index.ts... OK (12 ms)
    [wasm] wasm32-wasi component target... OK (1.4 MB)
    [component] wit-bindgen output... OK
    [aot] wasmtime compile · cache key sha256:b9e2c8...
    [build] dist/skill.wasm ✓
    [build] ready for: cyberos-skill-cli install./skills/my-new-skill/

18

## Phase status & code stats

LoC (Rust host)

~3,000

4 crates

LoC (Bun toolchain)

~2,500

TypeScript

SKILL.md bundles

20

14 CUO + 6 VN

Tests

4 unit + 12 parity + 2 grants

cargo + pytest harness

Audit phases shipped

7 of 7

Phase 7 in 30-day soak

Ecosystem reach

12+ clients

via open Agent Skills

Phase / capability| Status  
---|---  
Phase 0 — Inventory + freeze| shipped  
Phase 1 — Rust + Bun scaffold · spec validator| shipped  
Phase 2 — Parity harness| shipped · 12/12 pass  
Phase 3 — Executor selection| shipped (default Rust)  
Phase 4 — DashMap concurrency| shipped · ≥ 2× at contention  
Phase 5 — WASM execution| scaffolded · runtime feature-gated  
Phase 6 — Capability broker| shipped  
Phase 7 — Legacy retirement| runbook ready · 30-day soak  
VN catalog (5 shipped · 1 planned)| 5 shipped tax-filing planned  
OCI registry distribution| planned  
Cosign signature verification at install| shipped  
`agentskills.io` submission (R3)| planned (waits for registry API)  
GraphQL subgraph| planned · P0+  
Phase 8 + memory integration + vertical-pack roadmap  
**Phase 8** — Capability broker extends to memory protocol (Personal + Lumi's memory scopes)  
+1,500 LoC · broker mediates every memory read/write from a skill · audit-chain emission per invocation| designed · MEMORY_AUTOSYNC_DESIGN.md Stage 2/4  
`memory-capture@1` skill bundle (the capture daemon as a skill)| designed · Stage 2 of memory protocol  
`memory-sync@1` skill bundle (the 2-way sync orchestrator)| designed · Stage 4 of memory protocol  
`cuo/personas/synthesis-author@1` skill bundle (multi-memory auto-evolve)| designed · Stage 5 (P3)  
cyberskill-sg vertical pack (6 SG skills · Singapore HoldCo flip)| planned · P3 entry (P3 · start)  
cyberskill-eu vertical pack (6 EU skills · post-HoldCo)| planned · P3 exit (P3 · exit)  
cyberskill-id / -th vertical packs (SEA-3 wave)| planned · P4 (P4 · mid)  
cyberskill-us vertical pack (post-US sub)| planned · P4+18  
cyberskill-hr / -legal / -accounting cross-cutting packs| planned · P3+ (depend on module shipping)  
marketplace.cyberskill.world (R4 own marketplace)| deferred · gated on ≥ 50 paying tenants  
Enterprise private marketplace (R5 white-label)| aspirational · P4+24  
  
19

## References

  * **MEMORY_AUTOSYNC_DESIGN.md** (archived 2026-05-18 — original 2026-05-14 design lock) — [universal Personal memory + Lumi's memory architecture](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>). §6 Lumi's memory spec, §5 capture daemon spec, §8 multi-memory auto-evolve. Live implementation guidance now lives under `modules/memory/` \+ `services/memory/` \+ `docs/feature-requests/memory/`.
  * **AUTHORING_DISCIPLINE.md** — [canonical FR creation playbook](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>). feature-request-author + feature-request-audit are reference implementations of the universal-protocol skill pattern.
  * **AGENTS.md** (RFC v2.0.0, normative) — `cyberos/modules/memory/AGENTS.md`. §3.6 `allowed_memory_scopes` · §11 trust model · §14 cross-agent interop · §15 sync_class taxonomy.
  * **AUDIT.md** (13 May 2026) — full architectural audit + migration plan — `cyberos/modules/skill/README.md (Appendix H)`. Phases 0–7 closure rationale and Phase 8 (memory integration) handoff.
  * **SPEC.md** — protocol summary — `cyberos/modules/skill/README.md (Appendix A)`.
  * **PUBLISH.md** — OCI / cosign / agentskills.io publication workflow. The R0 → R5 distribution rungs land here as concrete operator procedures.
  * **PHASE_5_ACTIVATION.md** — Wasmtime runtime activation guidance.
  * **PHASE_7_RETIREMENT.md** — legacy retirement runbook.
  * **Source:** `cyberos/modules/skill/crates/` · `cyberos/modules/skill/toolchain/` · `cyberos/modules/skill// (flat — was skill/skills/)` · `cyberos/modules/skill/tests/`. Stage-2+ skills land at `cyberos/modules/skill// (flat layout — no skills/ subdir)`, `cyberos/modules/skill//`, `cyberos/modules/skill//`.
  * **External:** [agentskills.io/specification](<https://agentskills.io/specification>) · Anthropic engineering blog "Equipping agents for the real world with Agent Skills" (Oct 16 2025, updated Dec 18 2025) · [docs.claude.com Agent Skills overview](<https://docs.claude.com/en/docs/agents-and-tools/agent-skills/overview>).
  * **Cross-module:** [memory module page](<../memory/index.html>) — where the Stage 1–5 protocol roadmap lives. [CUO module page](<../cuo/index.html>) — where the 47 personas + 221 workflows are detailed.
  * **Strategy:** [CYBEROS_STRATEGY.md](<../../strategy/CYBEROS_STRATEGY.md>) §4.4 vertical packs as Level-4 productization · §3 docs site Tier 1+2 features that the marketplace deferred-list comes from.
  * **AUDIT + PLAN:** `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`) · `archive/2026-05-14/RESEARCH_REVIEW.md` (archived; see `cyberos/CHANGELOG.md`) §3 (Skill is "Gold standard"); §7.3 (defer the marketplace, sell the packs).
  * **CHANGELOG:** `cyberos/CHANGELOG.md (entries tagged [SKILL])` (newest-first).



[← Previous: CUO](<../cuo/index.html>) [All modules →](<../index.html#catalog>)
