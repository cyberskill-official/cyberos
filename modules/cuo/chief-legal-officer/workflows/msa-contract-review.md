---
workflow_id: chief-legal-officer/msa-contract-review
workflow_version: 1.0.0
purpose: Review an incoming MSA (or other counterparty contract) against the org's negotiation playbook with deviation log + redlines + GREEN/YELLOW/RED classification.
persona: cuo/chief-legal-officer
cadence: on-demand
status: shipped

inputs:
  - { name: contract_doc,       source: workflow-caller (incoming MSA / DPA / SOW / subscription), format: pdf or markdown }
  - { name: playbook,           source: cuo/clo-legal's negotiation playbook,                       format: markdown }
  - { name: business_context,   source: workflow-caller (deal size, customer tier, urgency),        format: markdown brief }

outputs:
  - { name: contract_review,    format: contract-review@1, recipient: cuo/clo-legal + cuo/cco-commercial (or originating function) }

skill_chain:
  - { step: 1, skill: contract-review-author, inputs_from: { contract_doc: contract_doc, playbook: playbook, business_context: business_context }, outputs_to: review_draft }
  - { step: 2, skill: contract-review-audit,  inputs_from: review_draft, outputs_to: contract_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "contract-review-audit fires QA-CLASS-001 — RED classification, deal > $250K or strategic customer; CEO sign-off required" }
  - { persona: cuo/chief-financial-officer,            when: "contract has unusual payment terms (>net-60, large upfront, or revenue-share)" }

consults:
  - { persona: cuo/chief-privacy-officer,    when: "contract is a DPA or has personal-data terms" }
  - { persona: cuo/chief-information-security-officer,           when: "contract has security-controls schedule" }

audit_hooks:
  - each step emits artefact_write to memory audit chain
  - workflow_complete row on PASS with contract_review hash + classification (GREEN/YELLOW/RED) + cycle-time
  - HITL pause at step 2 if QA-PLAYBOOK-001 fires (deviation not mapped to a playbook position)
---

# MSA contract review — `chief-legal-officer/msa-contract-review`

CLO-Legal's standard incoming-contract review workflow. Maps the counterparty's positions against the org's negotiation playbook, generates a structured deviation log with redlines, and classifies risk per ACC's GREEN/YELLOW/RED standard. Used for MSAs, DPAs, master subscription agreements, and SOWs.

## When to invoke

- "Review this incoming MSA from [counterparty]"
- "Run the contract review playbook on [contract]"
- "Triage this DPA"

## How to invoke

```bash
cyberos-cuo run cuo/chief-legal-officer/msa-contract-review \
  --input contract_doc=./contracts/in/2026-05-acme-msa.pdf \
  --input playbook=./legal/playbooks/msa.md \
  --input business_context=./contracts/in/2026-05-acme-msa.context.md \
  --output-dir ./contracts/in/2026-05-acme-msa/
```

## Expected duration

- **Happy path:** 30-90 min runtime + same-day operator review
- **Worst case:** RED classification + CEO escalation may add 3-5 days

## Skill chain

- **Step 1 `contract-review-author`** — drafts per ABA Model Contract Clauses + ACC Playbook structure: parties / counterparty-vs-playbook / deviation log / redlines / classification / negotiation guidance / escalation triggers.
- **Step 2 `contract-review-audit`** — validates per `contract_review_rubric@1.0` (FM + SEC + QA-PLAYBOOK-001 + QA-SEVERITY-001 + QA-IMPACT-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 1 | BOOT-001 | Counterparty doc missing | Operator supplies |
| 2 | QA-PLAYBOOK-001 | Deviation not mapped to playbook position | Operator extends playbook or adjusts deviation |
| 2 | QA-CLASS-001 | RED classification | Escalate to CEO |

## Cross-references
- `../README.md` §5 (Operational) — "contract review + signing"
- `../../../docs/The C-Suite Reference.md` §5.2
- `../../../skill/contract-review-{author,audit}/SKILL.md`
