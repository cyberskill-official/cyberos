---
# ── Identity ─────────────────────────────────────────────────────────
name: mock-contract-test-audit
description: >-
  Audit a mock-contract-test@1 against mock_contract_test_rubric@1.0: enforces ≥1 request_response_pair, error_modes coverage of every SECURITY/DEGRADATION matrix row, swap_target is a real symbol, sunset_criterion has an observable trigger, and contract_tests pass against the Mock today. Emits a `score / 10` verdict; refuses to pass on <10/10. Use when user asks to "audit this mock contract test" or "check the mock contract test". Do NOT use for "draft a new mock contract test" (use mock-contract-test-author instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: mock-contract-test-audit@1
  cyberos-rubric-target: mock_contract_test_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
  write:
    - project:task/{task_id}/mock-contract-test.audit

audit:
  row_kind: mock_contract_test_audited
  required_fields: [task_id, score, issues_open, issues_resolved]

inputs:
  - { name: mock_contract, format: mock-contract-test@1, required: true }
outputs:
  - { name: audit_report, format: mock-contract-test-audit@1 }
---

# mock-contract-test-audit

## 1. Rubric (mock_contract_test_rubric@1.0)

| Rule ID | Check | Weight | Severity if failed |
|---|---|---|---|
| MCT-001 | ≥ 1 request_response_pair captured | 20% | error |
| MCT-002 | Every SECURITY / DEGRADATION matrix row maps to an entry in `error_modes` of some pair | 20% | error |
| MCT-003 | `mock_implementation.swap_target` resolves to a real exported symbol | 15% | error |
| MCT-004 | Each contract_test path exists + the test passes against the Mock | 20% | error |
| MCT-005 | `sunset_criterion.trigger` is observable (not "someday" / TBD) | 15% | error |
| MCT-006 | `backlog_status_tag == "shipped + mocked-dependency"` | 10% | warning |

## 2. Pass criterion

10/10 only. Any error-class miss returns the artefact to the author with a fix list. The workflow proceeds to step 9 (implementation-plan-author) once this audit passes.

---

*End of mock-contract-test-audit SKILL.md.*

## Contract files (TASK-SKILL-118)

This pair is at full contract parity: `RUBRIC.md` (versioned rules + prose->rule map), `AUDIT_LOOP.md` (canonical-loop binding), `REPORT_FORMAT.md`, `envelopes/` (I/O schemas), `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
