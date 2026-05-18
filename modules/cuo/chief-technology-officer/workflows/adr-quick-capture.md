---
workflow_id: chief-technology-officer/adr-quick-capture
workflow_version: 1.0.0
purpose: Capture a single architectural decision in Nygard ADR format + audit it through to 10/10. Use when a decision needs documenting but the full architect-new-system chain is overkill.
persona: cuo/chief-technology-officer
cadence: on-demand
status: shipped

inputs:
  - { name: decision_context, source: workflow-caller, format: "verbal brief or 1-page markdown describing the forces in tension + decision being made" }
  - { name: linked_srs_reqs,  source: optional,        format: "list of REQ-IDs this decision affects" }

outputs:
  - { name: adr, format: adr@1, recipient: cuo/cto + future-engineers + any threat-model referring to this decision }

skill_chain:
  - { step: 1, skill: architecture-decision-record-author, inputs_from: decision_context, outputs_to: adr_draft }
  - { step: 2, skill: architecture-decision-record-audit,  inputs_from: adr_draft,        outputs_to: adr }

escalates_to:
  - { persona: cuo/chief-information-security-officer, when: "ADR audit fires COND-001 (decision touches security boundary) AND no threat-model entry references this ADR within 14 days" }

consults:
  - { persona: cuo/chief-product-officer, when: "decision_context references PRD use cases — verify the ADR aligns with product intent" }

audit_hooks:
  - architecture-decision-record-author emits one artefact_write row
  - architecture-decision-record-audit emits one artefact_write row per audit iteration
  - workflow emits a workflow_complete row with the final ADR-NNNN id + sha256
---

# Capture a single architectural decision — `chief-technology-officer/adr-quick-capture`

The lightest CTO workflow. Use it when an architectural choice has been made (in a meeting, a Slack thread, a tech-spike write-up) and needs to land as a formal ADR — without dragging through the full SRS-to-impl-plan chain.

## When to invoke

CUO routes here when the user says things like:

- "Document our decision to use Postgres over Mongo"
- "We just chose <option X> — write the ADR"
- "Add an ADR for the auth-service split"
- "Capture this architecture decision"

## How to invoke

```bash
cyberos-cuo run cuo/chief-technology-officer/adr-quick-capture \
  --input decision_context=./meeting-notes/2026-05-17-auth-service-split.md \
  --input linked_srs_reqs="[REQ-AUTH-001, REQ-AUTH-002]" \
  --output-dir ./engagements/<project>/adrs/
```

## Expected duration

- **Happy path:** 5–10 minutes of skill-chain runtime.
- **With one HITL pause** (typically QA-OPT-001 — single-option ADR): +30 min for operator to surface the alternative that was considered.
- **Worst case (EXHAUSTED):** chain halts; operator authors the ADR by hand and audit-only.

## Skill chain — step by step

### Step 1: `architecture-decision-record-author`
- **What it does:** Renders the Nygard format (Context / Options Considered / Decision / Consequences / Compliance & Quality Impact / Notes & References) from the decision-context input.
- **Inputs:** `decision_context` + optional `linked_srs_reqs`.
- **Outputs:** `adr_draft` — an `architecture-decision-record@1` markdown.
- **Pause point:** typically none; the chain is short enough to run unattended.

### Step 2: `architecture-decision-record-audit`
- **What it does:** Validates the ADR against `adr_rubric@1.0`. Common failure modes: QA-OPT-001 (single option), QA-CONSEQ-001 (only positive consequences), QA-ISO-001 (missing ISO 25010 mapping).
- **Inputs:** `adr_draft`.
- **Outputs:** `adr` — the ADR at 10/10.
- **Pause point:** HITL on COND-001 (security boundary) → escalate to `cuo/ciso`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 1 | BOOT-001 | decision_context input missing → operator supplies |
| 2 | needs_human (QA-OPT-001) | Operator surfaces the alternative that was rejected |
| 2 | needs_human (COND-001 + missing threat-model link) | Escalate to CISO; either link existing threat-model entry or schedule a threat-model refresh |
| 2 | EXHAUSTED | Operator authors the ADR by hand; re-run audit-only against the hand-authored draft |

## Operator-side decisions

- **Alternative surfacing** — if the audit fires QA-OPT-001, the operator must produce at least one rejected alternative (with pros/cons). Single-option ADRs fail the rubric by design.
- **Security-boundary escalation** — if COND-001 fires and no threat-model exists for this system, decide whether to schedule a threat-model refresh now or defer (with a tracked due-date).

## Cross-references

- `../README.md` — CTO 9-block spec.
- `./architect-new-system.md` — the longer workflow that includes ADR authoring as steps 3-4.
- `../../../skill/architecture-decision-record-author/SKILL.md`, `../../../skill/architecture-decision-record-audit/SKILL.md`, `../../../skill/architecture-decision-record-audit/RUBRIC.md`.
