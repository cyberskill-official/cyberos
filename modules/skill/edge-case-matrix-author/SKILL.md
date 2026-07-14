---
# ── Identity ─────────────────────────────────────────────────────────
name: edge-case-matrix-author
description: >-
  Generate a structured edge-case-matrix@1 for an FR before implementation. Enumerates: null/empty inputs, extreme bounds (off-by-one, integer overflow, time-zone DST, leap second, Unicode normalisation), malformed payloads (truncated, oversized, non-UTF-8, type-confused), concurrent race conditions (double-submit, double-acknowledge, cross-tenant cross-talk, RLS escape), security-class entries (auth bypass, injection, token replay), and degradation modes (downstream slow, downstream unreachable, partial write). One matrix row per category-and-trigger with a pointer to the test that will cover it. Used by chief-technology-officer/ship-tasks as step 5. Use when user asks to "draft a edge case matrix" or "create the edge case matrix". Do NOT use for "audit existing edge case matrix" (use edge-case-matrix-audit instead).
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: edge-case-matrix@1
  cyberos-rubric-target: edge_case_matrix_rubric@1.0

# ── Scope contract (memory/AGENTS.md §15) ────────────────────────────
allowed_memory_scopes:
  read:
    - project:*
    - module:*
  write:
    - project:fr/{task_id}/edge-case-matrix
audit:
  row_kind: edge_case_matrix_authored
  required_fields: [task_id, total_rows, categories_covered, security_class_count, planned_test_paths]

# ── Inputs / outputs ─────────────────────────────────────────────────
inputs:
  - { name: fr,           format: task@1,            required: true }
  - { name: context_map,  format: repo-context-map@1,           required: false }
outputs:
  - { name: matrix,       format: edge-case-matrix@1 }

# ── Triggers / blockers ──────────────────────────────────────────────
triggers:
  - any FR moving from `accepted` → `building`
  - workflow `chief-technology-officer/ship-tasks` step 5
blockers:
  - "FR acceptance criteria are ambiguous — escalate to chief-product-officer"
  - "no test framework declared in repo — must be resolved first"
---

# edge-case-matrix-author

## 1. Purpose

Produce a single artefact — the `edge-case-matrix@1` — that captures every
boundary condition the implementation must address, **before** any code is
written. The matrix is the contract between the FR's acceptance criteria
and the test suite; the coverage-gate skill (which runs after
implementation) reads the matrix to verify every row has a corresponding
test.

## 2. Output schema

```yaml
# edge-case-matrix@1
task_id: FR-<MODULE>-<NNN>
generated_at: <ISO-8601>
total_rows: <int>
rows:
  - id: ECM-001
    category: NULL_INPUT | BOUNDARY | MALFORMED | CONCURRENT | SECURITY | DEGRADATION
    trigger: "<one-sentence description of what produces the edge case>"
    expected: "<one-sentence description of correct behaviour>"
    severity: critical | high | medium | low
    planned_test: "<absolute path or test name where this will be covered>"
```

## 3. Quality gates

- Every category has ≥1 row (the audit-companion will fail if any is empty).
- SECURITY rows have a `planned_test` pointing at a real test file (not
  TBD).
- DEGRADATION rows describe both detection and recovery.
- `total_rows ≥ 8` for any FR rated `MUST` priority.

## 4. Chains to

`edge-case-matrix-audit` then `implementation-plan-author`.

---

*End of edge-case-matrix-author SKILL.md.*

## Contract files (TASK-SKILL-118)

This pair is at full contract parity: `PIPELINE.md` (chain binding + HALT points), `INVARIANTS.md`, `envelopes/` (I/O schemas), `references/FAILURE_MODES.md`, `acceptance/README.md`. SKILL.md remains the normative prose; the files encode it.
