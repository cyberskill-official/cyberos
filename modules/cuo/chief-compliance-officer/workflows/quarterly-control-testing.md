---
workflow_id: chief-compliance-officer/quarterly-control-testing
workflow_version: 1.0.0
purpose: Test compliance controls quarterly — control inventory + sample + test results + remediation tracking.
persona: cuo/chief-compliance-officer
cadence: quarterly
status: shipped

inputs:
  - { name: compliance_program,    source: cuo/chief-compliance-officer/annual-compliance-program, format: compliance-program@1 }
  - { name: prior_testing,         source: last quarter's testing results, format: compliance-program@1 (testing chapter) }
  - { name: control_evidence,      source: GRC tool (Drata / Vanta / Secureframe / Tugboat / AuditBoard), format: csv }

outputs:
  - { name: control_testing,       format: compliance-program@1 (quarterly testing chapter), recipient: cuo/cco-compliance + cuo/clo-legal + cuo/ciso + Board (compliance chapter) }

skill_chain:
  - { step: 1, skill: compliance-program-author, inputs_from: { compliance_program: compliance_program, prior_testing: prior_testing, control_evidence: control_evidence }, outputs_to: testing_draft }
  - { step: 2, skill: compliance-program-audit,  inputs_from: testing_draft, outputs_to: control_testing }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "control failure rate > 10% triggers material-weakness assessment" }
  - { persona: cuo/chief-legal-officer,      when: "control failure triggers regulatory self-disclosure" }

consults:
  - { persona: cuo/chief-information-security-officer,           when: "security-control failures need engineering remediation" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with control_testing hash + sample size + pass rate + remediation count
  - HITL pause at step 2 on QA-FAILURE-001 (control failure no remediation plan)
---

# Quarterly control testing — `chief-compliance-officer/quarterly-control-testing`

CCO-Compliance's quarterly control-testing per AICPA SOC 2 testing + ISO 37301 monitoring + AuditBoard / LogicGate testing patterns. Continuous-monitoring discipline that feeds the annual compliance program review.

## When to invoke

- "Run the Q<n> compliance control testing"
- "Quarterly control sample"
- "Compliance testing cycle"

## How to invoke

```bash
cyberos-cuo run cuo/chief-compliance-officer/quarterly-control-testing \
  --input compliance_program=./compliance/2026/program.md \
  --input prior_testing=./compliance/2026-Q1/testing.md \
  --input control_evidence=./grc/2026-Q1/evidence.csv \
  --output-dir ./compliance/2026-Q1/testing/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 2-4 weeks for sampling + testing + remediation
- **Worst case:** material-weakness assessment triggers external-audit firm intervention

## Skill chain

- **Step 1 `compliance-program-author`** — drafts testing chapter per AICPA SOC 2 + ISO 37301.
- **Step 2 `compliance-program-audit`** — validates per `compliance_program_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-FAILURE-001 | Failure no remediation | Escalate to control owner |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.6 — CCO-Compliance role profile
- `./annual-compliance-program.md` — upstream parent
- `../../../skill/compliance-program-{author,audit}/SKILL.md`
