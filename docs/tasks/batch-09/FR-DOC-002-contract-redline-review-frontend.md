---
title: "DOC — contract redline AI review (read-only AI; CLO sign-off required), DocuSign-equivalent frontend at /doc"
author: "@stephen-cheng"
department: legal
status: ready_for_review
priority: p3
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P3 / 2027-Q4"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Layer the Module-Federation frontend at `/doc` consuming FR-DOC-001 + the **contract redline AI review** surface introduced via a new **CUO/CLO** (Chief Legal Officer) emergent C-skill. The CLO surface reads contract documents (incoming third-party drafts received via EMAIL/upload + outgoing platform-templated drafts), surfaces **clause-by-clause analysis** comparing against the tenant's playbook (parameter-versioned policy positions on key clauses), produces a **structured redline report** ("clause 7.3 - liability cap is 12 months fees vs. our standard 6 months — accept-with-justification or push back?"), drafts **negotiation positions** (suggested counter-language with rationale), and explicitly defers to **CLO sign-off** (the human Chief Legal Officer or external counsel) — never auto-signs, never commits negotiation positions to the counterparty. The frontend at `/doc` provides envelope management, document editor, signing flow, redline review pane, and template library management. The pattern matches the FR-LEGAL skills from competing platforms but with the architectural rule that AI **only describes legal risk**; humans (or external counsel) sign-off on every position.

## Problem

PRD §14.4.1 P3 scope: "DOC module — contract redline review (read-only AI; CLO sign-off required), DocuSign-equivalent signing workflow." Three failure modes the platform must structurally avoid:

- **Slow contract review.** Each external service agreement coming in from a customer/vendor takes hours of legal-counsel review for a 10-employee company. AI redline review against the tenant's playbook compresses this to minutes.
- **AI-as-counsel risk.** The temptation to let AI auto-edit + sign contracts is real; doing so would create unbounded legal liability. Architectural prohibition is the floor.
- **Schema without UI.** FR-DOC-001 ships the substrate; without the frontend, signing flows are unusable.

## Proposed Solution

The shape of the answer is the new CUO/CLO emergent skill + the redline review pipeline + the Module-Federation frontend at `/doc`.

**CUO/CLO emergent C-skill.**

Joins the C-skill set alongside CEO/COO/CTO/CHRO/CRO/CFO/CSO + CAIO/CXO/CSO-Sus. Persona authored at `~/.cyberos/skills/cuo/clo/SKILL.md`; dual-signed by founder + Engineering Lead + (per FR-GENIE-004 pattern) feature-flagged off initially, enabled per-tenant after the CLO playbook is authored.

**Persona purpose.** Surface contract-clause risk against the tenant's playbook; never make legal commitments.

**Inputs.**
- The document (Markdown + PDF rendering via FR-KB-001's TipTap engine extended for legal-doc structure).
- The tenant's `doc.playbook` (introduced below) — structured per-clause positions.
- Prior signed contracts (BRAIN Layer 3) for "have we accepted this clause before?" pattern detection.
- Current Vietnamese legal context (PDPL + Labour Code + Civil Code) + EU GDPR + selected reference jurisdictions.

**Outputs.**
- Per-clause risk classification (low / medium / high / blocker).
- Per-clause comparison to playbook (matches / divergence / no-position).
- Suggested counter-language for divergences.
- Estimated negotiation effort.
- Cited references (the playbook entry + similar prior contracts).

**Tools forbidden.** `cyberos.doc.send_envelope`, `cyberos.doc.sign_envelope`, `cyberos.doc.void_envelope` — every signing action is human + step-up. CLO advises; CLO never signs.

**Playbook schema.**

```sql
CREATE TABLE doc.playbook (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  description_md TEXT NOT NULL,
  positions JSONB NOT NULL,                                            -- structured: per-clause positions
                                                                       -- [{
                                                                       --   clause_kind: "liability_cap",
                                                                       --   our_position: "12 months fees",
                                                                       --   acceptable_range: "6-24 months fees",
                                                                       --   blocker_below: "3 months fees",
                                                                       --   acceptable_language_md: "...",
                                                                       --   counter_language_md: "...",
                                                                       --   notes_md: "..."
                                                                       -- }, ...]
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_legal_counsel_ref TEXT NOT NULL,                            -- external counsel ref
  effective_from DATE NOT NULL,
  superseded_by UUID REFERENCES doc.playbook(id),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

The first playbook is authored by the founder + external Vietnamese legal counsel + (for international tenants) per-jurisdiction local counsel. It covers ~30-50 standard clauses for the documents the platform handles: liability caps, indemnification, IP ownership, termination + survival, confidentiality, payment terms, MFN, audit rights, data-protection commitments, jurisdiction + arbitration.

**Redline review pipeline.**

Triggered when:
- A document is received via EMAIL (CaMeL-sanitised; FR-EMAIL-003) + has `metadata.classification: legal_document`.
- A document is uploaded directly to the DOC module's intake.
- An envelope is created from an external counterparty's redline.

The pipeline:
1. Document parsing — convert PDF/Word to structured text (using the FR-EMAIL-010 attachment-extractor pattern; reused).
2. Clause segmentation — identify clause boundaries via heuristics + structural markers (Roman numerals, "WHEREAS", "Section", etc.).
3. Per-clause classification — for each clause, identify clause_kind (liability_cap / indemnification / etc.).
4. Comparison to playbook — fetch the active `doc.playbook`; compare per-clause language; classify risk.
5. Counter-language draft — for divergent clauses, draft suggested counter-language citing the playbook's `counter_language_md`.
6. Surface as a Review-mode card in the Genie panel for the founder + Account Manager + (when present) external counsel.

**Frontend at `/doc`.**

Module-Federation remote with surfaces:

- **Envelopes inbox** (`/doc/envelopes`) — list of in-flight envelopes; per-status filter; quick actions per envelope.
- **Envelope detail** (`/doc/envelopes/<id>`) — document preview (multi-page PDF render), signing-status timeline per signer, send/sign/void actions, internal-comments thread (FR-EMAIL-002 pattern reused).
- **Document editor** (`/doc/templates/<id>/edit`) — Markdown editor for templates with placeholder validation; preview-as-PDF; legal-counsel review request.
- **Redline review panel** — when a document is in review, the side-by-side redline view: original doc on left, playbook positions on right, AI-suggested counter-language as inline comment-style suggestions; each suggestion has accept / edit / reject + explicit "send to legal counsel for review" action.
- **Signing flow** — when an envelope is sent to the calling Member, the inline signing surface (PDF preview + draw/type/click/QTSP signature actions per tier).
- **Templates library** (`/doc/templates`) — list per-tenant templates; create new from blank or duplicate; legal-counsel sign-off workflow.
- **Playbook editor** (`/doc/playbook`) — founder + legal counsel author the playbook positions; structured per-clause editor; sign-and-publish flow.

**Cross-module integration.**

- **EMAIL** — incoming legal documents auto-route via classification; redline review draft surfaces in the `/email` thread + `/doc` review panel.
- **HR** — employment contracts created from FR-DOC-001 templates; FR-HR-001's `hr.contract.signed_document_id` references DOC's envelope.
- **REW** — salary letters + grant agreements + amendments via templates.
- **ESOP** — phantom-stock grant agreements via QES tier.
- **CRM** — customer service agreements link from `crm.deal` to DOC envelope on close-won.
- **CP** — DPAs + processing-record templates; per-tenant DPA template is part of FR-CP-003's regulator-artefact bundle.

**Persona scope contract for CUO/CLO.**

Allowed:
- `cyberos.doc.list_my_envelopes` (read; the doc context).
- `cyberos.doc.get_envelope` (read).
- `cyberos.doc.list_templates` (read).
- `cyberos.doc.get_playbook` (read).
- `cyberos.brain.search` (read; for prior-contract pattern).
- `cyberos.genie.draft_review` (Review mode for the redline report).

Forbidden:
- All `cyberos.doc.send_*`, `cyberos.doc.sign_*`, `cyberos.doc.void_*` mutations.
- `cyberos.email.send_*` (cannot send the redline report directly to counterparty).

**Member surfaces.**

The frontend is restricted by role:
- All Members: see envelopes they're a signer on; sign their assigned envelopes.
- Account Manager: create + send envelopes for their assigned customer Engagements.
- HR/Ops Lead: create employee contracts + acknowledgement envelopes.
- Founder: create + send any envelope; sign as required signer.
- DPO + Legal Counsel: review envelopes; access the redline review panel; author/edit playbooks.

**MCP tool surface (extending FR-DOC-001).**

- `cyberos.doc.draft_redline_review(document_id)` — read; CUO/CLO produces the structured redline report.
- `cyberos.doc.get_playbook` — read.
- `cyberos.doc.suggest_counter_language(clause_id, current_text)` — read.

CUO scope contract: read + draft allowed; commit forbidden.

## Alternatives Considered

- **Auto-redline + auto-counter-propose to counterparty.** Rejected: legal commitments without human review create unbounded liability.
- **Use a hosted contract-AI tool (Spellbook, BlackBoiler).** Rejected: residency + per-tenant playbook integration + the per-tenant CLO-emergent-skill pattern; the tenant data leaves residency.
- **Skip the redline AI entirely; just a signing surface.** Rejected: PRD §14.4.1 explicitly names "contract redline review (read-only AI; CLO sign-off required)" as scope.
- **One-size-fits-all playbook across all tenants.** Rejected: each tenant's risk tolerance + jurisdiction + business model differs; per-tenant playbook is the floor.

## Success Metrics

- **Primary metric.** P3 sprint demo passes: (1) the playbook is authored + signed for the canonical CyberSkill tenant; (2) a synthetic incoming third-party service agreement is uploaded; the redline review pipeline produces a structured per-clause report with counter-language suggestions; (3) the regression suite confirms the persona refuses to send/sign documents; (4) the frontend signs a synthetic envelope end-to-end including QES path; (5) the playbook editor allows founder + counsel to publish a v1 playbook.
- **Quality metric.** Redline review accuracy ≥ 80% on a curated 30-contract test corpus (the AI's clause classifications + risk levels + counter-language match the human counsel's separately-produced review).
- **Adoption metric.** ≥ 70% of incoming external contracts go through the redline review before being routed to external counsel; per-deal contract-review latency reduced by ≥ 50%.

## Scope

**In-scope.**
- New CUO/CLO emergent skill authored + dual-signed.
- `doc.playbook` schema + multi-version sign chain.
- Redline review pipeline (parsing + segmentation + classification + comparison + counter-language).
- Module-Federation frontend at `/doc` with all 6 surfaces.
- Cross-module integration with EMAIL + HR + REW + ESOP + CRM + CP.
- The 3 read-only MCP tools.
- Persona scope contract.
- Audit integration in scope `doc.ai.{tenant}`.

**Out-of-scope (deferred).**
- AI auto-suggested playbook updates from observed counterparty patterns (P4).
- Multi-language playbook (vi-VN + en-US separate playbook positions per language) — P4.
- Negotiation-coaching surface (P4 — for Account Managers preparing for negotiation calls).
- Mobile native (P3+).
- Customer-facing redline review (P4 PORTAL).

## Dependencies

- FR-DOC-001.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-DESIGN-001.
- FR-EMAIL-003 / FR-EMAIL-010 (CaMeL + attachment extraction).
- FR-KB-001 (TipTap editor reused for template + playbook editing).
- FR-BRAIN-001 / FR-BRAIN-002 / FR-BRAIN-003 (prior-contract retrieval).
- FR-GENIE-001 / FR-GENIE-002 / FR-GENIE-004 (CUO emergent-skill substrate).
- FR-CP-003 (DPA template).
- External Vietnamese legal counsel + (for international tenants) per-jurisdiction counsel for first playbook authoring.
- Compliance: PDPL Decree 13; EU AI Act Article 22 (no automated decisions on legal commitments); Article 50 (transparency disclosure on every redline report); Vietnamese + cross-border contract law.
- Locked decisions referenced: DEC-273 (CUO/CLO emergent skill), DEC-274 (per-tenant playbook with founder + counsel sign), DEC-275 (CLO read-only on docs; never sends or signs).

## AI Risk Assessment

The redline review surface affects high-stakes legal decisions. EU AI Act risk class: `limited` (informational; never automated decision; human commits via CLO sign-off + the FR-DOC-001 signing flow's separate human-in-the-loop step).

### Data Sources

Per-tenant only: documents + playbook + BRAIN. CUO/CLO runs through the AI Gateway with persona-stamping. Per-tenant residency; cross-jurisdiction reference data is static (Vietnamese law texts + GDPR + eIDAS).

### Human Oversight

- The redline report is a Review-mode card; the human reviews + accepts/rejects.
- Counter-language goes to legal counsel (internal or external) before being communicated to counterparty.
- Sending + signing envelopes is human-only with step-up (FR-DOC-001).
- Playbook authoring requires founder + counsel sign.

### Failure Modes

- **CLO mis-classifies a clause.** Mitigation: regression suite gates persona-version; legal counsel review on every novel-pattern document.
- **Counter-language drafts unfavourable position.** Mitigation: the playbook's `acceptable_range` defines bounds; the human reviews before sending.
- **Hallucinated reference to non-existent prior contract.** Mitigation: citation-correctness regression test; the cited prior must be in BRAIN.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted CLO scope, redline pipeline, frontend layout, persona-scope contract, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the first playbook + the CLO persona's first SKILL.md will be authored by the founder + Vietnamese legal counsel before P3 production.
