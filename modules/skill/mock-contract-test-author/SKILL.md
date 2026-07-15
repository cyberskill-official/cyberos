---
# ── Identity ─────────────────────────────────────────────────────────
name: mock-contract-test-author
description: >-
  When a task declares an external dependency that does not yet exist (missing API key, future service, paywall, 2FA challenge, CAPTCHA, third-party that needs procurement), author a `mock-contract-test@1` artefact: (a) the exact expected Request/Response shape of the missing service, (b) a Mock Service implementation that satisfies the contract, (c) the contract-test suite (one test per shape) that the Mock passes today and the Real service will pass tomorrow with a one-line import swap, (d) a `shipped + mocked-dependency` BACKLOG status tag with sunset criteria. Used by chief-technology-officer/ship-tasks as step 7, conditional on `fr.has_external_dependency == true`. Use when user asks to "draft a mock contract test" or "create the mock contract test". Do NOT use for "audit existing mock contract test" (use mock-contract-test-audit instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: mock-contract-test@1
  cyberos-rubric-target: mock_contract_test_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
allowed_memory_scopes:
  read:
    - project:*
    - module:*
  write:
    - project:fr/{task_id}/mock-contract-test
audit:
  row_kind: mock_contract_test_authored
  required_fields: [task_id, dependency_name, request_response_pairs, contract_tests, sunset_criterion]

# ── Inputs / outputs ─────────────────────────────────────────────────
inputs:
  - { name: fr,                format: task@1,    required: true }
  - { name: edge_case_matrix,  format: edge-case-matrix@1,   required: true }
outputs:
  - { name: mock_contract, format: mock-contract-test@1 }

# ── Triggers / blockers ──────────────────────────────────────────────
triggers:
  - workflow `chief-technology-officer/ship-tasks` step 7 when fr.has_external_dependency is true
blockers:
  - "task's external dependency is undeclared — author must list the dependency before this skill runs"
  - "downstream service is being actively built in parallel — mock is wasted effort; pause this task"
---

# mock-contract-test-author

## 1. Purpose

Make missing external services **non-blocking** for the task queue. Every
expected Request/Response pair is captured as a structural contract; a
Mock Service passes that contract; the test suite stays in CI forever.
When the real dependency lands, the swap is a single import change and
the contract guarantees behavioural parity.

## 2. Output schema

```yaml
# mock-contract-test@1
task_id: task-<MODULE>-<NNN>
generated_at: <ISO-8601>
dependency_name: "<service or API name>"
dependency_kind: third-party-api | internal-future-service | env-var-not-set | paywall | CAPTCHA | 2FA-challenge

request_response_pairs:
  - id: CONTRACT-001
    description: "<one-sentence description>"
    request: { method: GET | POST | ..., path: "...", body_shape: {...}, headers: {...} }
    response: { status: <int>, body_shape: {...}, headers: {...} }
    error_modes: [ { status: <int>, body_shape: {...}, when: "..." } ]

mock_implementation:
  language: rust | python | typescript
  source_file: "<absolute path to the Mock module>"
  swap_target: "<the symbol the production code imports — swapping Mock→Real is editing this one import>"

contract_tests:
  - { test_id: CT-001, path: "<absolute>", covers_contract_ids: ["CONTRACT-001"], mock_passes: true, real_runs: false }

sunset_criterion:
  trigger: "<observable signal that retires the Mock, e.g. 'TASK-AUTH-006 ships' or 'STRIPE_API_KEY env-var present in prod'>"
  sunset_action: "delete mock_implementation.source_file, flip swap_target to Real, re-run contract_tests against Real"

backlog_status_tag: "shipped + mocked-dependency"
```

## 3. Quality gates

- ≥ 1 request_response_pair per distinct shape used by the implementation.
- Every error_mode listed in the task's edge-case-matrix SECURITY or
  DEGRADATION categories appears in `error_modes` for at least one pair.
- `mock_implementation.swap_target` is a real exported symbol (file:line resolvable).
- `sunset_criterion` has an observable trigger — not "someday".

## 4. Chains to

`mock-contract-test-audit` then `implementation-plan-author`.

---

*End of mock-contract-test-author SKILL.md.*

## Contract files (TASK-SKILL-118)

This pair is at full contract parity: `PIPELINE.md` (chain binding + HALT points), `INVARIANTS.md`, `envelopes/` (I/O schemas), `references/FAILURE_MODES.md`, `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
