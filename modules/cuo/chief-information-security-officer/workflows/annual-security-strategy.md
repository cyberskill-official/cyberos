---
workflow_id: chief-information-security-officer/annual-security-strategy
workflow_version: 1.0.0
purpose: Author the annual information-security strategy — threat landscape, NIST CSF 2.0 posture, prioritized initiatives, budget, security OKRs.
persona: cuo/chief-information-security-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's security-strategy@1,                format: security-strategy@1 }
  - { name: posture_assessment,    source: CIS Controls v8 IG1/IG2/IG3 self-assessment,    format: csv export }
  - { name: threat_intel,          source: Verizon DBIR + industry ISAC + threat-intel feeds, format: markdown / csv }
  - { name: budget_envelope,       source: cuo/cfo (annual budget security line),          format: budget@1 chapter }

outputs:
  - { name: security_strategy,     format: security-strategy@1, recipient: cuo/ciso + cuo/cto + cuo/ceo + Board (annual security review) }

skill_chain:
  - { step: 1, skill: security-strategy-author, inputs_from: { prior_strategy: prior_strategy, posture_assessment: posture_assessment, threat_intel: threat_intel, budget_envelope: budget_envelope }, outputs_to: strategy_draft }
  - { step: 2, skill: security-strategy-audit,  inputs_from: strategy_draft, outputs_to: security_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "strategy proposes budget > 5% of revenue OR posture gap rated critical (e.g. NIST CSF IDENTIFY function < tier 2)" }
  - { persona: cuo/chief-technology-officer,         when: "strategy reshapes architecture commitments (e.g. zero-trust migration, KMS overhaul)" }

consults:
  - { persona: cuo/chief-privacy-officer, when: "strategy intersects GDPR/PDPD data-protection obligations" }
  - { persona: cuo/chief-compliance-officer, when: "strategy must satisfy compliance frameworks (SOC 2, ISO 27001, FedRAMP)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with security_strategy hash + posture-gap-count + budget-envelope
  - HITL pause at step 2 on QA-POSTURE-001 (posture-gap without initiative) or QA-OKR-001 (no measurable OKRs)
---

# Annual security strategy — `chief-information-security-officer/annual-security-strategy`

CISO's annual information-security strategy. Combines prior strategy + posture self-assessment + threat-intel + budget envelope into refreshed posture-vs-NIST-CSF-2.0 + prioritized initiatives + budget + security OKRs + board narrative. Board-reviewed annually; deltas reviewed quarterly.

## When to invoke

- "Build the 2026 security strategy"
- "Annual security review"
- "Refresh security posture + initiatives"

## How to invoke

```bash
cyberos-cuo run cuo/chief-information-security-officer/annual-security-strategy \
  --input prior_strategy=./security/2025/strategy.md \
  --input posture_assessment=./security/2026/cis-assessment.csv \
  --input threat_intel=./security/2026/threat-intel.md \
  --input budget_envelope=./budget/2026/security-chapter.md \
  --output-dir ./security/2026/strategy/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 4-8 weeks for posture assessment + board review
- **Worst case:** critical posture-gap triggers same-quarter remediation program

## Skill chain

- **Step 1 `security-strategy-author`** — drafts per NIST CSF 2.0 + ISO/IEC 27001:2022 + CIS Controls v8.
- **Step 2 `security-strategy-audit`** — validates per `security_strategy_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-POSTURE-001 | Posture gap no initiative | Operator drafts |
| 2 | QA-OKR-001 | OKRs not measurable | Operator quantifies |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3 — CISO role profile
- `../../chief-technology-officer/workflows/threat-model-refresh.md` — quarterly peer workflow
- `../../../skill/security-strategy-{author,audit}/SKILL.md`
