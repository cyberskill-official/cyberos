---
workflow_id: chief-technology-officer/certify-nfrs
workflow_version: 1.0.0
purpose: Run systemic verification against the Non-Functional Requirements catalog to certify a deployment environment. Automatically handles regressions by spawning bug-fix FRs or escalating.
persona: cuo/chief-technology-officer
cadence: per-release
status: shipped

inputs:
  - { name: target_environment,    source: workflow caller,                         format: "staging | production" }
  - { name: nfr_catalog,           source: workflow caller,                         format: "path to docs/non-functional-requirements" }
  - { name: telemetry_window,      source: workflow caller,                         format: "ISO 8601 duration e.g., 24h" }

outputs:
  - { name: nfr_certification_report, format: nfr-certification-report@1, recipient: cuo/cto + deploy owner }
  - { name: regression_frs,           format: task@1,          recipient: backlog }

skill_chain:
  - { step: 1, skill: nfr-test-runner,          inputs_from: { target_environment: target_environment, nfr_catalog: nfr_catalog, telemetry_window: telemetry_window }, outputs_to: test_results }
  - { step: 2, skill: nfr-evaluator,            inputs_from: { test_results: test_results, nfr_catalog: nfr_catalog }, outputs_to: evaluation_verdicts }
  - { step: 3, skill: nfr-certification-author, inputs_from: evaluation_verdicts, outputs_to: nfr_certification_report }
  - { step: 4, skill: nfr-regression-handler,   inputs_from: evaluation_verdicts, outputs_to: regression_frs }

escalates_to:
  - { persona: cuo/chief-information-security-officer, when: "evaluation_verdicts contains a failed security or privacy NFR" }
  - { persona: cuo/chief-technology-officer,           when: "evaluation_verdicts contains a degraded performance SLO by more than 10%" }

consults:
  - { persona: cuo/chief-ai-officer,                   when: "evaluation_verdicts contains AI model latency or correctness SLO breaches" }

audit_hooks:
  - each skill emits artefact_write rows per its frontmatter audit hook
  - workflow emits a workflow_complete row with the final go/no-go verdict and the list of spawned regression FRs
  - HITL pauses at step 4 on whether to auto-inject bug-fix FRs or halt
---

# NFR Certification Review — `chief-technology-officer/certify-nfrs`

The CTO's gate for Non-Functional Requirements. While Functional Requirements are tested individually within the `ship-tasks` pipeline, systemic NFRs (load testing, prolonged security scanning, uptime monitoring) require systemic evaluation. This workflow runs against an entire environment to certify that the `docs/non-functional-requirements` catalog is upheld.

## When to invoke

CUO routes here when the user says things like:
- "Run the load tests against staging"
- "Certify the NFRs for the v3.4.1 release"
- "Check if our SLOs are holding up in production over the last 24h"

## How to invoke

```bash
cyberos-cuo run cuo/chief-technology-officer/certify-nfrs \
  --input target_environment=staging \
  --input nfr_catalog=./docs/non-functional-requirements/ \
  --input telemetry_window=24h \
  --output-dir ./releases/v3.4.1/nfr-certification/
```

## Expected duration

- **Happy path:** 15–30 minutes (mostly waiting for test runners like Gatling or K6 to finish execution).
- **With regressions:** +1-2 hours for manual triage of bug-fix FRs and escalations to CISO/CTO.

## Skill chain — step by step

### Step 1: `nfr-test-runner`
- **What it does:** Reads the `verification` stanza of each NFR in the catalog. Orchestrates the necessary external testing harnesses (e.g., Gatling, K6, Lighthouse CI, SAST tools, Datadog queries) to run against the `target_environment`.
- **Inputs:** `target_environment`, `nfr_catalog`, `telemetry_window`.
- **Outputs:** Raw `test_results`.
- **Pause point:** None. Fully automated.

### Step 2: `nfr-evaluator`
- **What it does:** Compares the raw `test_results` against the `slo` fields defined in each NFR spec. Determines a pass/fail/degraded verdict for each requirement.
- **Inputs:** `test_results`, `nfr_catalog`.
- **Outputs:** `evaluation_verdicts`.
- **Pause point:** None.

### Step 3: `nfr-certification-author`
- **What it does:** Authors the `nfr-certification-report` detailing which NFRs passed, which failed, and the deltas from the baseline.
- **Inputs:** `evaluation_verdicts`.
- **Outputs:** `nfr_certification_report`.
- **Pause point:** None.

### Step 4: `nfr-regression-handler`
- **What it does:** For any NFR that receives a `fail` or `degraded` verdict, automatically drafts a bug-fix FR (incorporating the failure logs and reproduction steps) and prepares to insert it into the backlog with a `ready_to_implement` status.
- **Inputs:** `evaluation_verdicts`.
- **Outputs:** `regression_frs`.
- **Pause point:** HITL on inserting the regression FRs into the backlog versus simply alerting a human to triage manually.

## Failure modes — per step

| Step | Code | What happens | Recovery |
|---|---|---|---|
| 1 | BOOT-001 | nfr_catalog input missing or invalid | Point to the correct NFR docs directory |
| 1 | TEST_TIMEOUT | External harness fails to return | Retry the harness or manually supply the test result payload |
| 4 | HITL (PLAN) | Regression found | Operator decides to approve auto-FR injection or manually triage the regression |

## Operator-side decisions

1. **Auto-Remediation vs Manual Triage at step 4:** When regressions occur, the workflow proposes bug-fix FRs. The CTO decides if these should immediately hit the backlog or if the release should simply be halted for manual debugging.
2. **Escalations:** If security or privacy NFRs fail, the workflow alerts the CISO immediately, pausing further rollout until the vulnerability is addressed.

## Cross-references
- `../README.md` — CTO 9-block spec.
- `../../../docs/non-functional-requirements/README.md` — The NFR catalog architecture.
