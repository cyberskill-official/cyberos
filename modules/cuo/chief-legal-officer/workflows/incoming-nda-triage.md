---
workflow_id: chief-legal-officer/incoming-nda-triage
workflow_version: 1.0.0
purpose: Rapidly triage an incoming NDA — classify GREEN/YELLOW/RED, screen for embedded non-compete, non-solicit, missing carveouts.
persona: cuo/chief-legal-officer
cadence: on-demand
status: shipped

inputs:
  - { name: nda_doc,            source: workflow-caller (incoming NDA), format: pdf or markdown }
  - { name: business_context,   source: workflow-caller (counterparty, intent of disclosure), format: markdown brief }

outputs:
  - { name: nda_triage,         format: nda-triage@1, recipient: cuo/clo-legal + originating function (sales, BD, M&A) }

skill_chain:
  - { step: 1, skill: non-disclosure-agreement-triage-author, inputs_from: { nda_doc: nda_doc, business_context: business_context }, outputs_to: triage_draft }
  - { step: 2, skill: non-disclosure-agreement-triage-audit,  inputs_from: triage_draft, outputs_to: nda_triage }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "triage classification is RED — full legal review required (not auto-signable under standard delegation)" }

consults:
  - { persona: cuo/chief-strategy-officer,   when: "NDA is for M&A target (different threshold; usually mutual + tighter)" }
  - { persona: cuo/chief-human-resources-officer,           when: "NDA contains non-solicit / non-compete clauses affecting personnel" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with nda_triage hash + classification + cycle-time
  - HITL pause at step 2 if QA-CLASS-001 fires (RED classification) or QA-CARVEOUT-001 (missing standard carveouts)
---

# Incoming NDA triage — `chief-legal-officer/incoming-nda-triage`

CLO-Legal's NDA triage workflow. Target turnaround: 30 minutes for GREEN, 24 hours for YELLOW, full legal review for RED. Designed to eliminate the bottleneck of every NDA waiting on a GC by safely auto-routing standard NDAs to delegated sign-off.

## When to invoke

- "Triage this incoming NDA from [counterparty]"
- "Quick NDA review"
- "Can we sign this NDA"

## How to invoke

```bash
cyberos-cuo run cuo/chief-legal-officer/incoming-nda-triage \
  --input nda_doc=./nda/in/2026-05-acme-nda.pdf \
  --input business_context=./nda/in/2026-05-acme-nda.context.md \
  --output-dir ./nda/in/2026-05-acme-nda/
```

## Expected duration

- **Happy path:** 5-15 min runtime + same-day operator confirmation
- **Worst case:** RED escalation triggers full legal review (1-3 business days)

## Skill chain

- **Step 1 `non-disclosure-agreement-triage-author`** — drafts per ACC NDA triage standard: parties / triage / screening checklist / action required / cycle time.
- **Step 2 `non-disclosure-agreement-triage-audit`** — validates per `nda_triage_rubric@1.0` (FM + SEC + QA-CARVEOUT-001 + QA-CLASS-001 + QA-MUTUALITY-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-CARVEOUT-001 | Missing standard carveout (publicly available, independently developed, etc.) | Operator adds to deviations |
| 2 | QA-CLASS-001 | RED classification | Escalate to CLO-Legal for full review |

## Cross-references
- `../README.md` §5 (Operational) — "contract review + signing"
- `../../../../modules/cuo/README.md` §5.2
- `../../../skill/nda-triage-{author,audit}/SKILL.md`
