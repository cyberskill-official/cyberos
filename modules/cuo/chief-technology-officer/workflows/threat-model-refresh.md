---
workflow_id: chief-technology-officer/threat-model-refresh
workflow_version: 1.0.0
purpose: Refresh the threat model for a given system — quarterly cadence + on every major architecture change. Captures new ADRs since last refresh, re-walks STRIDE categories, updates OWASP Top 10:2025 + ASVS mapping.
persona: cuo/chief-technology-officer
cadence: quarterly + per-event
status: shipped

inputs:
  - { name: system_under_threat,    source: workflow caller,                       format: string }
  - { name: prior_threat_model,     source: ./threat-models/<system>.md,           format: threat-model@1 }
  - { name: linked_srs,             source: current SRS for the system,            format: software-requirements-specification@1 }
  - { name: linked_adrs,            source: list of ADRs (esp. those accepted since prior_threat_model.modelled_at), format: "list[adr@1]" }
  - { name: changelog_since_prior,  source: git log since prior_threat_model.modelled_at, format: markdown }

outputs:
  - { name: threat_model, format: threat-model@1, recipient: cuo/cto + cuo/ciso + linked-ADR authors }

skill_chain:
  - { step: 1, skill: threat-model-author, inputs_from: { srs: linked_srs, adrs: linked_adrs, prior: prior_threat_model, changelog: changelog_since_prior }, outputs_to: threat_model_draft }
  - { step: 2, skill: threat-model-audit,  inputs_from: threat_model_draft, outputs_to: threat_model }

escalates_to:
  - { persona: cuo/chief-information-security-officer,         when: "STRIDE-S/E/T threats touching auth/crypto find a mitigation gap" }
  - { persona: cuo/chief-privacy-officer,  when: "COND-001 fires AND LINDDUN analysis surfaces a new privacy threat" }
  - { persona: cuo/chief-ai-officer,         when: "COND-002 fires — AI/ML threats including model evasion, model inversion, training-data poisoning, pretrained-model supply-chain" }
  - { persona: cuo/chief-legal-officer,    when: "the threat model's residual risk register contains an accepted risk that has regulatory exposure" }

consults:
  - { persona: cuo/chief-technology-officer,         when: "any STALE-003 fires — a linked ADR was accepted since the prior threat model but not enumerated; either add to the model or supersede the ADR" }

audit_hooks:
  - threat-model-author emits one artefact_write row
  - threat-model-audit emits one artefact_write row per iteration
  - workflow emits a workflow_complete row with the verdict + a STRIDE-category coverage summary (one count per S/T/R/I/D/E)
---

# Threat-model refresh — `chief-technology-officer/threat-model-refresh`

The CTO's recurring security-architecture discipline. Two-skill chain (`threat-model-author` → `threat-model-audit`) that updates an existing threat model with: (1) ADRs accepted since the last refresh, (2) any new attack-surface from feature-shipped changelog, (3) the latest OWASP Top 10:2025 + ASVS guidance. STRIDE category coverage is enforced — every trust boundary identified in §2 of the threat model MUST be covered by at least one threat in §4.

## When to invoke

CUO routes here when the user says things like:

- "Quarterly threat-model refresh for the customer portal"
- "Refresh the threat model after the auth-service split"
- "Re-walk STRIDE for the payment system"
- "Update the threat model with the new ADRs"

Also auto-triggered when:

- `cuo/chief-technology-officer/architect-new-system.md` reaches step 5 (threat-model-author) — there a NEW model is created; refresh is when ONE EXISTS already.
- `cuo/chief-technology-officer/adr-quick-capture.md` reaches step 2 and `COND-001` fires (decision touches security boundary) AND no threat-model entry references the new ADR within 14 days.

## How to invoke

```bash
cyberos-cuo run cuo/chief-technology-officer/threat-model-refresh \
  --input system_under_threat="customer-portal" \
  --input prior_threat_model=./threat-models/customer-portal.md \
  --input linked_srs=./srs/customer-portal-srs.md \
  --input linked_adrs="[./adrs/ADR-0001.md, ./adrs/ADR-0042.md, ./adrs/ADR-0051.md]" \
  --input changelog_since_prior=./threat-models/customer-portal-changelog-since-2026-Q1.md \
  --output-dir ./threat-models/
```

## Expected duration

- **Happy path (quarterly refresh, no major arch change):** 30–60 minutes.
- **Post-major-arch-change refresh** (e.g. monolith → microservices split): 2–4 hours; many STRIDE rows shift; new ASVS L3 controls may apply.
- **First-time threat model for an existing system** (no `prior_threat_model`): falls back to the full `threat-model-author` PLAN phase; 2-3 hours including STRIDE walk + OWASP coverage check.

## Skill chain — step by step

### Step 1: `threat-model-author`
- **What it does:** Refreshes the threat model. Takes prior_threat_model + new ADRs + changelog. Carries forward unchanged STRIDE rows; adds new rows for new attack surface; refreshes the OWASP Top 10:2025 mapping; bumps `tm_version`.
- **Inputs:** the 5 listed input artefacts.
- **Outputs:** `threat_model_draft`.
- **Pause point:** PLAN approval on `asvs_level` (L1 / L2 / L3) — only changes if business risk classification has changed; usually carries forward.

### Step 2: `threat-model-audit`
- **What it does:** Validates against `threat_model_rubric@1.0`. Most common refresh-time hits: STALE-003 (new accepted ADR not enumerated), STRIDE-S/T/R/I/D/E-001 (new trust boundary not covered), OWASP-A02/A03 (security-misconfig + supply-chain — elevated in 2025), QA-CVE-001 (fabricated CVE references).
- **Inputs:** `threat_model_draft`.
- **Outputs:** `threat_model` at 10/10.
- **Pause point:** HITL escalations per the `escalates_to:` declaration above.

## Failure modes — per step

| Step | Code | Recovery |
|---|---|---|
| 1 | BOOT-001 | prior_threat_model input missing → fall back to fresh-threat-model authoring (longer) |
| 2 | needs_human (STALE-003) | ADR accepted but not in model → operator decides: add row to the model OR mark the ADR as superseded |
| 2 | needs_human (STRIDE-*-001) | New trust boundary lacks coverage → operator brainstorms threats per category |
| 2 | needs_human (OWASP-*) | Top-10 risk treatment missing → operator declares the approach (e.g. SBOM strategy for A03) |
| 2 | needs_human (QA-CVE-001) | Fabricated CVE reference → operator validates against NVD/MITRE or removes |

## Operator-side decisions

The CTO is pulled in at:

1. **ASVS level confirmation at step 1** — confirm whether business risk classification still warrants the prior ASVS L-level.
2. **STRIDE coverage walk at step 2** — for each new trust boundary, brainstorm threats per S/T/R/I/D/E category.
3. **OWASP Top 10:2025 treatment declarations** — A02 (Security Misconfiguration) and A03 (Supply Chain Failures) are elevated in 2025; ensure the model declares concrete approaches (hardening posture for A02, SBOM + provenance for A03).
4. **Residual-risk acceptance** — for each threat the team accepts as residual, the operator (often CTO + CISO + CLO) decides the acceptance rationale + review date.
5. **LINDDUN privacy walk (if COND-001 fires)** — escalate to CPO-Privacy for the 7-category privacy threat walk (Linkability, Identifiability, Non-repudiation, Detectability, Disclosure of information, Unawareness, Non-compliance).

## Cross-references

- `../README.md` — CTO 9-block spec.
- `./architect-new-system.md` — the larger workflow that creates a fresh threat model (step 5-6 of that chain).
- `./adr-quick-capture.md` — auto-triggers this refresh when COND-001 fires.
- `../../../skill/threat-model-author/SKILL.md`, `../../../skill/threat-model-audit/RUBRIC.md`.
- `../../../docs/Software Development Process.md` §2(d) — Architecture security review.
- OWASP Top 10:2025 — A02/A03 elevations drive the refresh cadence.
- STRIDE (Microsoft), LINDDUN (KU Leuven).
