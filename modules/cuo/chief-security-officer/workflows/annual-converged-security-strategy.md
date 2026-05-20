---
workflow_id: chief-security-officer/annual-converged-security-strategy
workflow_version: 1.0.0
purpose: Author the converged security strategy — physical, info-sec, supply-chain, insider-threat, executive-protection.
persona: cuo/chief-security-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's security-strategy@1 (converged), format: security-strategy@1 }
  - { name: ciso_strategy,         source: cuo/chief-information-security-officer/annual-security-strategy, format: security-strategy@1 }
  - { name: physical_threat_intel, source: ASIS / IFPO + corporate-security feeds, format: markdown }
  - { name: ceo_priorities,        source: cuo/ceo, format: markdown }

outputs:
  - { name: converged_security_strategy, format: security-strategy@1, recipient: cuo/cso-security + cuo/ciso + cuo/ceo + Board (annual security chapter) }

skill_chain:
  - { step: 1, skill: security-strategy-author, inputs_from: { prior_strategy: prior_strategy, ciso_strategy: ciso_strategy, physical_threat_intel: physical_threat_intel, ceo_priorities: ceo_priorities }, outputs_to: strategy_draft }
  - { step: 2, skill: security-strategy-audit,  inputs_from: strategy_draft, outputs_to: converged_security_strategy }

audit_hooks:
  - workflow_complete row on PASS with converged_security_strategy hash
  - HITL pause at step 2 on QA-POSTURE-001
---

# Annual converged security strategy — `chief-security-officer/annual-converged-security-strategy`

CSO-Security's converged-security strategy per ASIS ESRM (Enterprise Security Risk Management) + ANSI/ASIS PAP.1-2012 + NIST SP 800-160. Superset of CISO scope (info-sec only).

## Cross-references
- `../../../../modules/cuo/README.md` §5.7 — CSO-Security role profile (physical + info-sec)
- `../../chief-information-security-officer/workflows/annual-security-strategy.md` — info-sec subset peer
- `../../../skill/security-strategy-{author,audit}/SKILL.md`
