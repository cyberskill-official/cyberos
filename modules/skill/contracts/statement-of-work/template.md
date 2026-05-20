---
template: statement-of-work@1
title: <Engagement title — e.g. "Acme Corp Customer Portal — Phase 1">
client_name: <Trading name>
client_legal_entity: <Full legal name, jurisdiction>
engagement_model: fixed_price    # fixed_price | time_and_materials | dedicated_team | staff_augmentation | managed_services
effective_date: 2026-MM-DD
target_close_date: 2026-MM-DD
sow_version: 1.0.0
cs_signer: @<client-handle>
em_signer: @<engagement-manager>
cyberskill_signer: @stephen.cheng
governing_law: Vietnam
provenance:
  source_path: ./discovery-brief.md
  source_hash: sha256:<hash>
---

# Statement of Work — <client> — <engagement title>

## 1. Objectives and Success Criteria

<!-- authority: human-edited --> <Outcome statement: what we are doing and how we will know it worked.>

## 2. Scope

### In Scope
- <Bullet 1>
- <Bullet 2>
- <Bullet 3>

### Out of Scope
- <Bullet 1>
- <Bullet 2>
- <Bullet 3>

## 3. Deliverables

| # | Deliverable | Format | Owner | Target date |
|---|---|---|---|---|
| 1 | <Name> | <Markdown / PDF / source / live system> | @<owner> | YYYY-MM-DD |

## 4. Assumptions and Constraints

- **Assumptions:** <List>
- **Constraints:** <Budget / timeline / regulatory>

## 5. Engagement Model

<engagement_model>. <Specific terms per model.>

### Fixed-Price Terms / Rate Card / Team Composition / Performance Management / SLA Definitions
<!-- Pick the subsection that matches engagement_model — see statement-of-work-audit/RUBRIC.md §4 COND-001..005 -->

## 6. Team and Roles

RACI per modules/cuo/README.md#software-development-process §2.

| Stage | CS | EM | PO | TL | AR | DEV | QA | DO | SEC |
|---|---|---|---|---|---|---|---|---|---|

## 7. Schedule and Milestones

| Milestone | Target date | Acceptance gate |
|---|---|---|

## 8. Pricing and Invoicing

<Pricing structure, invoice cadence, payment terms, late-payment policy.>

## 9. Acceptance Criteria

Per deliverable in §3. References Definition of Done at `<dor-dod-ref>` if established.

## 10. IP and Confidentiality

- IP assignment on payment: <terms>
- Pre-existing IP carve-out: <list>
- Background-IP licensing: <terms>
- NDA scope and term: <terms>
- Sub-processor list: <list> (required when personal data is processed — see COND-006)
- Data-processing addendum: <reference> (required when personal data is processed)
- AI-tool usage disclosure: <per SDP §5 — permitted tools, data-perimeter rules, AI-assisted PR labelling commitment>

## 11. Change Control

<How scope changes are proposed, approved, priced. Change-order template reference.>

## 12. Warranty, Support, and Governance Cadence

- Warranty period: <N days post-acceptance>
- Support tier: <description>
- Governance cadence: daily standup (internal); weekly client status (written + 30 min call); fortnightly demo; monthly steering committee; quarterly business review (QBR) — per SDP §6.

<!-- ── Conditionally-required sections (uncomment + fill as needed; see statement-of-work-audit/RUBRIC.md §4) ── -->
<!--
### Data Processing Addendum
### GDPR Addendum
### Vietnam Compliance  (Decree 13/2023 PDPD + Decree 53/2022 cybersecurity)
### HIPAA-aligned Controls
### On-call Coverage
-->
