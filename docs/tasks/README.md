# CyberOS Feature Request Backlog — Master Index

> Turn Your Will Into Real.

**Source of truth.** This index is the authoritative manifest of every Feature Request (FR) generated from `docs/CyberOS-PRD.docx` and `docs/CyberOS-SRS.docx`. Each FR is a free-standing markdown file at `docs/tasks/<batch-id>/<fr-id>.md` and is fully self-contained — no external template dependency. Every FR carries the same frontmatter fields (`title`, `author`, `department`, `status`, `priority`, `created_at`, `ai_authorship`, `feature_type`, `eu_ai_act_risk_class`, `target_release`, `client_visible`) and the same section structure in the body (Summary → Problem → Proposed Solution → Out of Scope → Dependencies → Constraints → Compliance / Privacy → Risk Assessment → Vietnamese-locale considerations → Scope (with Gherkin acceptance) → Success Metrics → Open Questions → References).

**Owner.** `@stephen-cheng` (Trịnh Thái Anh, Founder/CEO).
**Editor.** Lead Architect (TBD — currently shared with Founder during P0).
**Cadence.** Batches are generated and reviewed in groups of ten. New batches are appended; superseded FRs set `status: closed` with a `superseded_by` frontmatter field rather than being deleted.

---

## 1. Grouping strategy — proposed

The PRD describes **22 functional modules** delivered across **five gated phases** (P0 → P4) over **24 months**. The natural shape of the backlog — and the one this index commits to — is a **two-axis grouping**:

```
Primary axis   →  Module        (MOD code: AUTH, AI, MCP, BRAIN, GENIE, CHAT, EMAIL, PROJ, …)
Secondary axis →  Phase         (P0..P4; SubSprint S0-1..S0-6 inside P0)
```

**Why this grouping rather than alternatives.** Three alternatives were considered and rejected:

1. *Group by phase only.* Rejected because phase boundaries are time-based (12-week chunks) and tell engineers nothing about ownership; PRD §7.2 ("Module Ready" definition) explicitly assigns every module a single owner role and an independent deployable boundary, so module ownership is the strongest organising principle in the architecture.
2. *Group by C-level skill (CUO/CEO, CUO/CFO, …).* Rejected because skills are persona overlays inside a single module (GENIE/CUO), not delivery boundaries; the C-skill axis is useful for prompt design and persona evals but does not partition engineering work.
3. *Group by user role (Founder, Engineering Lead, HR/Ops Lead, …).* Rejected because most modules serve multiple roles simultaneously (e.g. CHAT is used by everyone) and the role axis would force every FR to enumerate cross-cutting concerns redundantly. The role lens is preserved through the existing PRD §3.4 RACI and §20.2 Module ↔ Role ↔ Phase matrix.

The selected axes preserve the PRD's `FR-{MOD}-{NNN}` ID convention (PRD §0.5), let every module repo own its own FR slice, align cleanly with Apollo Federation v2 subgraph boundaries (one module = one subgraph = one FR cluster), and let the phase axis drive the gate-readiness reports and the `target_release` frontmatter field without re-cutting the work.

**Module roster (22).** Codes preserved verbatim from PRD §0.5 / §7.1.

| Code | Module | Phase | Owner role (P0-P1) |
|---|---|---|---|
| AUTH | Authentication & Authorization | P0 | Engineering Lead |
| AI | AI Gateway, provider routing, budget | P0 | Engineering Lead |
| MCP | Model Context Protocol Gateway | P0 | Engineering Lead |
| OBS | Observability, dashboards, SLOs | P0 → P1 | Engineering Lead |
| BRAIN | Universal Memory (3 layers) | P0 | Engineering Lead |
| GENIE | Genie / CUO persona surface | P0 | Founder/CEO + Eng Lead (dual-sign) |
| CHAT | Internal real-time chat (Mattermost fork) | P0 | Engineering Lead |
| EMAIL | Internal email + shared inbox (Stalwart) | P1 | Engineering Lead |
| PROJ | Project management (Linear-style) | P1 | Engineering Lead |
| KB | Knowledge Base | P1 | HR/Ops Lead |
| TIME | Time & Expense | P1 | HR/Ops Lead |
| CRM | Client management | P1 | Account Manager |
| HR | Human Resources | P2 | HR/Ops Lead |
| REW | Total Rewards (Compensation) | P2 | HR/Ops Lead |
| LEARN | Learning & Promotion | P2 | HR/Ops Lead |
| INV | Invoicing | P2 | HR/Ops Lead |
| ESOP | Phantom Stock | P2 | Founder/CEO |
| RES | Resource planning | P2 | Account Manager |
| OKR | OKR / Strategy | P2 | Founder/CEO |
| DOC | Document signing | P3 / P4 | HR/Ops Lead |
| PORTAL | Client portal | P4 | Account Manager |
| TEN | Tenancy & Billing | P4 | Founder/CEO |

Two cross-cutting non-module slices that also cluster like modules:

| Code | Slice | Phase | Owner role |
|---|---|---|---|
| INFRA | Federation gateway, host shell, K8s, residency, design-tokens | P0 | Engineering Lead |
| CP | Compliance Plane (PDPL, GDPR, EU AI Act, ISO/SOC) | P0 → P3 | DPO + Founder |

**Batch protocol.** Each batch is exactly **10 FRs**, sequenced **module-first within phase**, **phase-first across batches**. The batch numbering is monotonic; the file path encodes both batch id and FR id so the validator can trace provenance:

```
feature-requests/
├── README.md                    ← this file
├── batch-01/                    ← P0 INFRA + AUTH + AI + MCP + BRAIN + GENIE + CHAT + OBS  (THIS BATCH)
│   ├── FR-INFRA-001-federation-host-shell-multi-tenant-postgres.md
│   ├── FR-AUTH-001-oauth21-webauthn-rbac-rls.md
│   ├── FR-AUTH-002-append-only-merkle-chained-audit-log.md
│   ├── FR-AI-001-ai-gateway-bedrock-zdr-fallback-redaction.md
│   ├── FR-MCP-001-mcp-gateway-2025-11-25-tool-registry.md
│   ├── FR-BRAIN-001-layer1-filesystem-cyberos-memory.md
│   ├── FR-BRAIN-002-layer2-vector-graph-hybrid-retrieval.md
│   ├── FR-GENIE-001-cuo-base-persona-three-skills-notify.md
│   ├── FR-CHAT-001-mattermost-fork-integrated-camel-ingestion.md
│   └── FR-OBS-001-observability-skeleton-compliance-cockpit.md
├── batch-02/                    ← P0 Stabilisation: BRAIN L3 + Conflict + NLCRUD; GENIE Q/R + Daily Flow; Design System; OBS SLOs; CP skeleton + RTBE; AUTH step-up
│   ├── FR-BRAIN-003-layer3-archival-corpus.md
│   ├── FR-BRAIN-CONFLICT-001-memory-conflict-resolution-ui.md
│   ├── FR-BRAIN-NLCRUD-001-natural-language-memory-crud.md
│   ├── FR-GENIE-002-question-review-modes-trust-calibration.md
│   ├── FR-GENIE-003-founder-daily-flow-uc-flow-001.md
│   ├── FR-DESIGN-001-design-system-v1-tokens-component-library.md
│   ├── FR-OBS-002-full-slos-alerting-per-module-dashboards.md
│   ├── FR-CP-001-compliance-plane-skeleton-dpia-a05-decisions.md
│   ├── FR-CP-002-rtbe-dsar-synthetic-tenant-drill.md
│   └── FR-AUTH-003-step-up-auth-agent-client-lifecycle.md
├── batch-03/                    ← P1 EMAIL: Stalwart core, Missive UX, CaMeL, AI, vi-VN, CRM seam, PROJ promote, deliverability, Gmail migration, attachment security
│   ├── FR-EMAIL-001-stalwart-core-integration.md
│   ├── FR-EMAIL-002-missive-style-shared-inbox-ux.md
│   ├── FR-EMAIL-003-camel-dual-llm-anti-injection.md
│   ├── FR-EMAIL-004-ai-features-summarisation-replies-categorisation.md
│   ├── FR-EMAIL-005-vietnamese-aware-composition.md
│   ├── FR-EMAIL-006-crm-bidirectional-integration-seam.md
│   ├── FR-EMAIL-007-promote-to-proj-task.md
│   ├── FR-EMAIL-008-deliverability-operations-warmup-reputation.md
│   ├── FR-EMAIL-009-gmail-migration.md
│   └── FR-EMAIL-010-attachment-security-smime-link-rewriting.md
├── batch-04/                    ← P1 PROJ: schema + Engagement; sync engine; lifecycle + WIP; cycle planning + close; frontend remote; AI features; CRM/GitHub seam; MCP mutations; Asana/Linear migration; notifications + standup
│   ├── FR-PROJ-001-three-primitives-schema-engagement-overlay.md
│   ├── FR-PROJ-002-linear-style-sync-engine-offline.md
│   ├── FR-PROJ-003-issue-lifecycle-custom-states-wip-limits.md
│   ├── FR-PROJ-004-cycle-planning-execution-close.md
│   ├── FR-PROJ-005-frontend-remote-board-list-timeline.md
│   ├── FR-PROJ-006-ai-features-triage-blocker-calibration.md
│   ├── FR-PROJ-007-engagements-crm-seam-github-integration.md
│   ├── FR-PROJ-008-mcp-mutation-surface-agent-parity.md
│   ├── FR-PROJ-009-migration-from-asana-linear-jira-trello.md
│   └── FR-PROJ-010-notifications-standup-time-seam.md
├── batch-05/                    ← P1 KB + TIME + CRM: KB schema/editor + AI Q&A + frontend; TIME entries + leave/sabbatical + expense; CRM schema + pipeline + AI + HubSpot migration
│   ├── FR-KB-001-schema-notion-style-block-editor.md
│   ├── FR-KB-002-ai-qa-graphrag-cross-page.md
│   ├── FR-KB-003-frontend-permissions-mcp.md
│   ├── FR-TIME-001-time-entries-capture-weekly-approval.md
│   ├── FR-TIME-002-leave-sabbatical-capacity-heatmap.md
│   ├── FR-TIME-003-expense-capture-ocr-vat-compliance.md
│   ├── FR-CRM-001-schema-accounts-contacts-deals-activities-signals.md
│   ├── FR-CRM-002-pipeline-ux-frontend-mutation-mcp.md
│   ├── FR-CRM-003-ai-features-cro-next-action-deal-insight.md
│   └── FR-CRM-004-hubspot-migration.md
├── batch-06/                    ← P2 HR + REW: schema+contracts+KMS; onboarding+probation; 1:1+directory; REW schema+anti-retroactive+P1-protection; BP fund+ACB interest; payroll close+anomaly; VN SI/PIT; frontend+payslip narrator; Excel migration; Good/Bad Leaver
│   ├── FR-HR-001-employee-schema-contracts-kms-comp-fields.md
│   ├── FR-HR-002-onboarding-workflow-genie-checklist.md
│   ├── FR-HR-003-1-on-1-templates-directory-org-chart.md
│   ├── FR-REW-001-schema-3p-anti-retroactive-p1-invariant.md
│   ├── FR-REW-002-bonus-points-fund-acb-anti-inflation.md
│   ├── FR-REW-003-payroll-cycle-close-payslip-anomaly.md
│   ├── FR-REW-004-vietnamese-si-pit-statutory-engine.md
│   ├── FR-REW-005-frontend-payslip-narrator.md
│   ├── FR-REW-006-excel-payroll-migration.md
│   └── FR-REW-007-good-leaver-bad-leaver-termination.md
├── batch-07/                    ← P2 LEARN + ESOP + OKR: VP schema+roll-up; Hội đồng promotion+disputes; career path+360+next-step; frontend; phantom stock schema+anti-retroactive; put options+Good/Bad Leaver+simulator; equity dashboard; OKR schema+cascade; quarterly cycle+CUO/CEO+CSO review; frontend tree+heatmap
│   ├── FR-LEARN-001-variable-performance-schema-rollup.md
│   ├── FR-LEARN-002-hoi-dong-chuyen-mon-promotion-disputes.md
│   ├── FR-LEARN-003-career-path-360-next-step-recommender.md
│   ├── FR-LEARN-004-frontend-remote-member-manager-council-views.md
│   ├── FR-ESOP-001-phantom-stock-schema-anti-retroactive.md
│   ├── FR-ESOP-002-put-options-good-bad-leaver-simulator.md
│   ├── FR-ESOP-003-frontend-equity-dashboard-grant-management.md
│   ├── FR-OKR-001-schema-objectives-key-results-cascade.md
│   ├── FR-OKR-002-quarterly-cycle-cuo-ceo-cso-review.md
│   └── FR-OKR-003-frontend-remote-tree-heatmap-checkins.md
├── batch-08/                    ← P2 INV + RES + close-out: schema+lifecycle+Stripe/VNPay/Wise+frontend; allocation Gantt+CUO/COO simulator+frontend; Decree 13 graduation; CAIO/CXO/CSO-Sus emergent skills; P2→P3 phase-gate evidence map
│   ├── FR-INV-001-schema-vendors-pos-invoices-ar-ap.md
│   ├── FR-INV-002-lifecycle-dunning-ar-aging-cfo-features.md
│   ├── FR-INV-003-stripe-vnpay-wise-payment-integrations.md
│   ├── FR-INV-004-frontend-remote-ar-ap-dashboards.md
│   ├── FR-RES-001-schema-allocation-capacity-staffing.md
│   ├── FR-RES-002-cuo-coo-rebalancer-staffing-simulator.md
│   ├── FR-RES-003-frontend-allocation-gantt-heatmap-scenario.md
│   ├── FR-CP-003-decree-13-full-regime-graduation.md
│   ├── FR-GENIE-004-emergent-c-skills-caio-cxo-cso-sus.md
│   └── FR-OBS-003-p2-p3-phase-gate-evidence-map.md
├── batch-09/                    ← P3 SaaS-readiness: TENANT ×3 + DOC ×2 + CP ×2 (GDPR + ISO/SOC) + BILL + CORP + OBS-004 P3→P4 gate
│   ├── FR-TEN-001-full-multi-tenancy-residency-partitioning.md
│   ├── FR-TEN-002-tenant-lifecycle-provision-suspend-archive-delete.md
│   ├── FR-TEN-003-per-tenant-theming-custom-domains.md
│   ├── FR-DOC-001-document-schema-esignature-aatl-eidas.md
│   ├── FR-DOC-002-contract-redline-review-frontend.md
│   ├── FR-CP-004-gdpr-posture-eu-shard-external-dsar.md
│   ├── FR-CP-005-iso-27001-stage-1-2-soc2-type-1-audits.md
│   ├── FR-BILL-001-per-tenant-subscription-billing-metered-ai.md
│   ├── FR-CORP-001-singapore-holdco-flip.md
│   └── FR-OBS-004-p3-p4-phase-gate-evidence-map.md
├── batch-10/                    ← P4 Client-facing: PORTAL ×3 + TEN ×2 (P4) + API ×2 + GTM ×2 + OBS-005 P4→GA gate (FINAL BATCH)
│   ├── FR-PORTAL-001-client-portal-foundation.md
│   ├── FR-PORTAL-002-document-billing-deliverable-surface.md
│   ├── FR-PORTAL-003-cxo-emergent-skill-client-assistant.md
│   ├── FR-TEN-004-self-service-tenant-onboarding.md
│   ├── FR-TEN-005-tenant-admin-console.md
│   ├── FR-API-001-public-rest-api.md
│   ├── FR-API-002-public-graphql-api-webhooks.md
│   ├── FR-GTM-001-marketing-site-trust-center-launch.md
│   ├── FR-GTM-002-iso-42001-soc2-type-2-close.md
│   └── FR-OBS-005-post-launch-observability-final-close-out.md
└── (backlog complete — 100 / 100 FRs across 10 batches)
```

**Naming rule.** `FR-{MOD}-{NNN}-{kebab-slug}.md`, slug ≤ 64 chars. The `{NNN}` counter is **module-local** (each module's FR sequence starts at 001 and is monotonically incremented across all batches forever — never reset, never reused).

**Status discipline.** PRD §16 governance applies: `draft → ready_for_review → approved → in_implementation → shipped → closed`. A lightweight FR linter (`tools/tool-fr-validator/`) runs in CI on every PR that touches a `docs/tasks/**.md` file: it asserts each FR has the canonical frontmatter fields + section headers + at least one Gherkin block in the Scope section.

---

## 2. Batch index

| Batch | Phase focus | FR count | Status |
|---|---|---|---|
| batch-01 | P0 Foundations — federation, AUTH, AI, MCP, BRAIN, GENIE, CHAT, OBS | 10 | generated |
| batch-02 | P0 Stabilisation — BRAIN L3, BRAIN conflict, BRAIN NLCRUD, GENIE Q/R, Daily Flow, Design System, OBS SLOs, CP skeleton, RTBE drill, AUTH step-up | 10 | generated |
| batch-03 | P1 EMAIL — Stalwart core, Missive UX, CaMeL, AI features, vi-VN composition, CRM seam, PROJ promote, deliverability ops, Gmail migration, attachment security | 10 | generated |
| batch-04 | P1 PROJ — primitives + Engagement, Linear sync engine, lifecycle + WIP, cycle planning, frontend remote, AI features, CRM/GitHub seam, MCP mutation surface, Asana migration, notifications + standup | 10 | generated |
| batch-05 | P1 KB ×3 + TIME ×3 + CRM ×4 — block editor + AI Q&A + frontend; entries + leave/sabbatical + expense; CRM schema + pipeline UX + AI + HubSpot migration | 10 | generated |
| batch-06 | P2 HR ×3 + REW ×7 — Total Rewards Appendix encoded: schema+contracts+KMS, onboarding, 1:1+directory; REW schema+anti-retroactive+P1-protection, BP fund+ACB interest, payroll close+anomaly, VN SI/PIT, frontend+narrator, Excel migration, Good/Bad Leaver | 10 | generated |
| batch-07 | P2 LEARN ×4 + ESOP ×3 + OKR ×3 — VP roll-up + Hội đồng + career path + frontend; phantom stock + put options + frontend; OKR schema + cycle + heatmap | 10 | generated |
| batch-08 | P2 INV ×4 + RES ×3 + close-out ×3 — invoicing + Stripe/VNPay/Wise + AR/AP frontend; resource planning + CUO/COO simulator + Gantt; Decree 13 graduation + emergent C-skills + P2→P3 gate evidence | 10 | generated |
| batch-09 | P3 SaaS-readiness — TEN ×3 (residency partitioning + lifecycle + theme/custom domains) + DOC ×2 (schema/eIDAS-AATL signing + redline frontend) + CP ×2 (GDPR posture + EU-shard DSAR + ISO 27001 Stage 1/2 + SOC 2 Type I) + BILL ×1 (per-tenant subscription + metered AI) + CORP ×1 (Singapore HoldCo flip) + OBS-004 P3 → P4 gate | 10 | generated |
| batch-10 | P4 Client-facing — PORTAL ×3 (foundation, document/billing/deliverable surface, CXO emergent assistant) + TEN ×2 P4 (self-service onboarding, tenant admin console) + API ×2 (public REST + public GraphQL/webhooks) + GTM ×2 (marketing site + Trust Center, ISO 42001 + SOC 2 Type II close) + OBS-005 P4 → GA gate | 10 | generated |

**Total: 100 / 100 FRs across 10 batches. Backlog complete.**

Total target: **~100 FRs across 10 batches**, sized to one engineer-week of clarification-free work each. The user explicitly commands the next batch; this README never auto-advances.

---

## 3. Generation rules (binding for every batch)

These rules are pinned so that subsequent batches stay consistent without re-stating them:

1. **Section adherence.** Every FR uses the canonical section list (Summary → Problem → Proposed Solution → Out of Scope → Dependencies → Constraints → Compliance / Privacy → Risk Assessment → Vietnamese-locale considerations → Scope → Success Metrics → Open Questions → References). No new sections. Sections may be marked `_(N/A)_` when truly inapplicable, but never omitted.
2. **Frontmatter.** Each FR has the canonical frontmatter fields (`title`, `author`, `department`, `status`, `priority`, `created_at`, `ai_authorship`, `feature_type`, `eu_ai_act_risk_class`, `target_release`, `client_visible`). FRs are self-contained — no external template dependency.
3. **No invented facts.** Every numerical target, every module name, every locked-decision reference (`DEC-XXX`), every NFR ID, every FR cross-reference, and every compliance regime traces back to a citation in the PRD or SRS. Sources are listed in the FR's `Dependencies` section. If the spec is silent, the FR explicitly marks the gap as an *Open Question* (`OQ-XXX`) and routes resolution to the founder.
4. **Auditable acceptance criteria.** Every FR ends its `Scope` and `Success Metrics` with criteria that a CI job, a phase-gate review, or an external auditor can verify with no human interpretation. Where Gherkin is appropriate (PRD §19.18), the FR uses Gherkin verbatim.
5. **AI Risk Assessment.** When the feature emits AI-generated content visible to a natural person, `eu_ai_act_risk_class: limited` is the floor and the three required subsections are filled. When the feature decides on compensation, equity, hiring, or any HR-impacting axis, `eu_ai_act_risk_class: high` and Article 14 human-oversight controls are spelled out at the system-property level.
6. **Vietnamese-first.** Where a feature has a user-visible surface, the FR explicitly addresses Vietnamese-locale behaviour (PGroonga tokenisation, Be Vietnam Pro typography, Anh/Chị salutations, vi-VN as default locale).
7. **Compliance cross-references.** Every FR that touches personal data names the applicable regime: PDPL Law 91/2025 + Decree 356/2025 (Vietnam), Decree 13/2023 (PDPL implementing decree), GDPR (EU), EU AI Act Articles 5–7 + 14 + 50, SOC 2 trust criteria, ISO/IEC 27001, and where relevant ISO/IEC 42001.
8. **Locked decisions.** When an FR depends on a locked decision, it cites the `DEC-XXX` ID and the PRD §11.1 or SRS Decisions Log section. FRs never silently override locked decisions; a change request must be filed against the decisions log first.

---

## 4. Cross-references

- PRD: `docs/CyberOS-PRD.docx` (source of truth for *what* and *why*).
- SRS: `docs/CyberOS-SRS.docx` (source of truth for *how* and *to what standard*).
- Decisions log: `docs/decisions.yaml` (DEC-001 .. DEC-066+; SRS Decisions Log entries up to ~DEC-300 referenced from FRs are pending author + sign).
- FR linter: `tools/tool-fr-validator/` (CI-blocking; asserts canonical frontmatter + section structure + at least one Gherkin block per FR).
- Compliance mapping: `docs/compliance/eu-ai-act-risk-classes.md` (TBD — to be authored at FR-CP-001 pickup).

---

*Last updated: 2026-05-03 (post-batch-10 — backlog complete) · author: `@stephen-cheng` · ai_authorship: co_authored*
