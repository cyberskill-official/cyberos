---
workflow_id: chief-technology-officer/architect-new-system
workflow_version: 1.0.0
purpose: Drive a new system or major capability from approved PRD through ready-for-implementation, generating SRS + ADRs + threat model + SDD + impl plan in audit-passing form.
persona: cuo/chief-technology-officer
cadence: per-event
status: shipped

inputs:
  - { name: prd,                source: cuo/cpo-product or external,                                    format: product-requirements-document@1 }
  - { name: capacity_signal,    source: project-plan or fractional-PM input,                            format: markdown brief }
  - { name: target_quarter,     source: workflow-caller,                                                format: string (e.g. "2026-Q3") }

outputs:
  - { name: srs,                format: srs@1,                       recipient: cuo/cto + cuo/cpo-product }
  - { name: adrs,               format: architecture-decision-record@1 (multiple),            recipient: cuo/cto + future-engineers }
  - { name: threat_model,       format: threat-model@1,              recipient: cuo/cto + cuo/ciso }
  - { name: sdd,                format: sdd@1,                       recipient: cuo/cto + impl team }
  - { name: impl_plan,          format: impl-plan@1,                 recipient: impl team (Linear / Jira / GitHub Projects) }

skill_chain:
  - { step: 1, skill: software-requirements-specification-author,          inputs_from: prd,                            outputs_to: srs_draft }
  - { step: 2, skill: software-requirements-specification-audit,           inputs_from: srs_draft,                      outputs_to: srs }
  - { step: 3, skill: architecture-decision-record-author,          inputs_from: srs,                            outputs_to: adrs_draft }
  - { step: 4, skill: architecture-decision-record-audit,           inputs_from: adrs_draft,                     outputs_to: adrs }
  - { step: 5, skill: threat-model-author, inputs_from: { srs: srs, adrs: adrs },       outputs_to: threat_model_draft }
  - { step: 6, skill: threat-model-audit,  inputs_from: threat_model_draft,             outputs_to: threat_model }
  - { step: 7, skill: software-design-document-author,          inputs_from: { srs: srs, adrs: adrs },       outputs_to: sdd_draft }
  - { step: 8, skill: software-design-document-audit,           inputs_from: sdd_draft,                      outputs_to: sdd }
  - { step: 9, skill: implementation-plan-author,    inputs_from: { sdd: sdd, target_quarter: target_quarter }, outputs_to: impl_plan_draft }
  - { step: 10, skill: implementation-plan-audit,    inputs_from: impl_plan_draft,                outputs_to: impl_plan }

escalates_to:
  - { persona: cuo/chief-information-security-officer,           when: "threat-model-audit STRIDE-S/E/T rules fire above warning + a corresponding ADR doesn't yet exist" }
  - { persona: cuo/chief-product-officer,    when: "software-requirements-specification-audit QA-NFR-001 fires — NFR coverage gap; product owner needs to refine acceptance criteria" }
  - { persona: cuo/chief-financial-officer,            when: "implementation-plan-audit total_estimate_pts implies >25% capacity for the target quarter" }

consults:
  - { persona: cuo/chief-privacy-officer,    when: "the system processes personal data — verify GDPR / Vietnam Decree 13/2023 coverage in SRS COND-002" }
  - { persona: cuo/chief-ai-officer,           when: "the system is AI-driven — verify EU AI Act risk-class + AI-specific test cases in test-strategy" }

audit_hooks:
  - each skill emits one artefact_write row to the memory audit chain per its frontmatter audit.row_kind
  - workflow emits a single workflow_complete row at the end with the 5-artefact summary + per-artefact hash
  - HITL pauses (typically at step 1 PLAN approval, step 5 STRIDE-S boundary call, step 9 capacity-band approval) halt the chain
---

# Architect a new system — `chief-technology-officer/architect-new-system`

The canonical end-to-end CTO workflow for taking an approved product requirement (PRD) and driving it through every SDP §2(b)→§2(f) artefact until engineering can pick up tickets and start coding. Five chained skill pairs (SRS / ADR / threat-model / SDD / impl-plan) — ten author+audit invocations — with the audit-loop discipline guaranteeing each artefact passes its `<artefact>_rubric@1.0` at 10/10 before the next step starts.

## When to invoke

CUO routes here when the user says things like:

- "Architect a new <system> for <customer>"
- "Take this PRD and produce the full engineering pack"
- "We have approved feature X — drive it to implementation"
- "Turn this product brief into ready-for-eng work"

## How to invoke

```bash
# When the runtime orchestrator lands (currently markdown-driven; v3.0.0 will execute)
cyberos-cuo run cuo/chief-technology-officer/architect-new-system \
  --input prd=./prds/PRD-acme-portal.md \
  --input target_quarter=2026-Q3 \
  --output-dir ./engagements/acme-portal/
```

## Expected duration

- **Happy path (no HITL pauses):** 30–60 minutes of skill-chain runtime (depending on artefact size).
- **With typical HITL pauses** (PLAN approval at step 1; one COND/STRIDE escalation at step 5; capacity sign-off at step 9): 1–3 business days end-to-end including operator round-trip.
- **Worst case (audit-loop exhausts max_iterations on any step):** chain halts at that step with EXHAUSTED; operator escalates.

## Skill chain — step by step

### Step 1: `software-requirements-specification-author`
- **What it does:** Authors a Software Requirements Specification per IEEE 830 + ISO/IEC 25010:2023 NFR coverage from the input PRD.
- **Inputs from this workflow:** `prd` (path to an audited PRD artefact).
- **Outputs:** `srs_draft` — a `software-requirements-specification@1` markdown.
- **Pause point:** PLAN approval (the skill emits an SRS outline and halts for operator approval before WORKER phase).

### Step 2: `software-requirements-specification-audit`
- **What it does:** Validates `srs_draft` against `srs_rubric@1.0` (IEEE 830 conformance + ISO 25010 nine-quality coverage + cross-skill chain rules). Iterates until 10/10.
- **Inputs:** `srs_draft`.
- **Outputs:** `srs` — the SRS at 10/10.
- **Pause point:** HITL on QA-NFR-001 if NFR coverage gap → escalate to `cuo/cpo-product`.

### Step 3: `architecture-decision-record-author`
- **What it does:** Surfaces architectural decisions implied by the SRS and authors them in Michael Nygard format. Typically produces 3–7 ADRs per non-trivial SRS.
- **Inputs:** `srs`.
- **Outputs:** `adrs_draft` — list of `architecture-decision-record@1` markdowns.
- **Pause point:** PLAN approval on which decisions warrant an ADR.

### Step 4: `architecture-decision-record-audit`
- **What it does:** Validates each ADR against `adr_rubric@1.0` (Nygard format + ISO/IEC 25010 impact mapping + the ≥2-option rule).
- **Inputs:** `adrs_draft`.
- **Outputs:** `adrs` — all ADRs at 10/10.
- **Pause point:** HITL on QA-OPT-001 (single-option ADR) if the operator over-trimmed alternatives.

### Step 5: `threat-model-author`
- **What it does:** Authors a STRIDE threat model with OWASP Top 10:2025 + ASVS mapping, given the SRS + accepted ADRs as context.
- **Inputs:** `{srs, adrs}`.
- **Outputs:** `threat_model_draft` — a `threat-model@1` markdown.
- **Pause point:** PLAN approval on ASVS verification level + COND-001/002 triggers (personal data / AI/ML / public API).

### Step 6: `threat-model-audit`
- **What it does:** Validates the threat model against `threat_model_rubric@1.0` (STRIDE category coverage + OWASP Top 10:2025 OWASP-A01..A10 rules + ASVS L-level controls).
- **Inputs:** `threat_model_draft`.
- **Outputs:** `threat_model` at 10/10.
- **Pause point:** HITL on STRIDE-S/T threats touching auth/crypto boundaries → escalate to `cuo/ciso`.

### Step 7: `software-design-document-author`
- **What it does:** Authors a Software Design Description per IEEE 1016 viewpoints, using SRS + ADRs as input. Produces 1-N SDDs per major component.
- **Inputs:** `{srs, adrs}`.
- **Outputs:** `sdd_draft` — list of `software-design-document@1` markdowns.
- **Pause point:** PLAN approval on component decomposition (which viewpoints to include).

### Step 8: `software-design-document-audit`
- **What it does:** Validates each SDD against `sdd_rubric@1.0` (IEEE 1016 viewpoint coverage + traceability to SRS REQ-IDs + design-contradicts-ADR check).
- **Inputs:** `sdd_draft`.
- **Outputs:** `sdd` at 10/10.
- **Pause point:** HITL on QA-ADR-001 (design contradicts an accepted ADR — usually means the ADR needs revision).

### Step 9: `implementation-plan-author`
- **What it does:** Translates the SDD into engineering tickets with estimates, owner assignments, branch/PR strategy, test approach, observability hooks. Conducts a 2-3 question sprint-planning interview.
- **Inputs:** `{sdd, target_quarter}`.
- **Outputs:** `impl_plan_draft` — an `implementation-plan@1` markdown.
- **Pause point:** capacity sign-off — if `total_estimate_pts` > 25% of the quarter's capacity, escalate to `cuo/cfo`.

### Step 10: `implementation-plan-audit`
- **What it does:** Validates the impl plan against `impl_plan_rubric@1.0` (DORA small-batch + AI-tooling discipline + per-task acceptance link + AI-specific code-review plan if `ai_assisted` is high).
- **Inputs:** `impl_plan_draft`.
- **Outputs:** `impl_plan` at 10/10 — ready for ticket creation in the target proj-management system.
- **Pause point:** HITL on QA-BATCH-001 if `total_estimate_pts > 40` without a decomposition rationale.

## Failure modes — per step

| Step | Code | What happens | Recovery |
|---|---|---|---|
| 1, 3, 5, 7, 9 | BOOT-001 | Required input file missing | Operator supplies the missing input; resume |
| 1, 5, 7, 9 | HITL (PLAN approval) | Skill pauses for operator approval of the plan-render | Operator approves / revises / aborts; resume |
| 2, 6 | needs_human | Audit found a rule violation requiring human input | Operator answers via HITL_BATCH_REQUEST reply; resume |
| 2, 4, 6, 8, 10 | EXHAUSTED | Audit loop hit max_iterations without converging | Escalate to `cuo/cto` for manual revision; do NOT auto-ship below 10/10 |
| 3, 5 | XCHAIN-002 | provenance.source_hash drift mid-chain | STALE handling — operator chooses REVERT_TO_MANIFEST or OVERWRITE |

## Operator-side decisions

The CTO (or delegate) is pulled into this workflow at these pause points:

1. **PLAN approval at step 1** — review the SRS outline before WORKER phase generates the full SRS body.
2. **NFR-gap escalation at step 2** — when SRS-audit's QA-NFR-001 fires, decide whether to refine the PRD (escalate to CPO-Product) or accept the gap (mark waived).
3. **Single-option ADR review at step 4** — if any ADR was authored with only one realistic option, decide whether the alternatives section is genuinely thin or if it's missing.
4. **ASVS L-level + STRIDE boundary at step 5** — for security-relevant decisions, confirm the targeted ASVS level (L1 / L2 / L3) and escalate STRIDE-S/E findings to CISO.
5. **PLAN approval at step 7** — for SDD, decide which IEEE 1016 viewpoints apply at this depth.
6. **Capacity sign-off at step 9** — if impl-plan estimates exceed the quarter's capacity, decide phasing.

## Cross-references

- `../README.md` — the CTO 9-block spec.
- `../../../docs/The C-Suite Reference.md` §5.3 — CTO role profile.
- `../../../docs/Software Development Process.md` §2(b)–§2(f) — the SDLC stages this workflow drives.
- `../../docs/AGENTS.md` — protocol normativity.
- `../../docs/ROUTING.md` — how the CUO reaches this workflow.
- `../../../skill/{srs,adr,threat-model,sdd,impl-plan}-{author,audit}/SKILL.md` — the per-skill specs.
