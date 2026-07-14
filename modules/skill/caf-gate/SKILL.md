---
# ── Identity ─────────────────────────────────────────────────────────
name: caf-gate
description: >-
  Code-audit gate for the testing to done transition, complementary to awh-gate. Reruns the
  module's own build, lint, typecheck, and test (its TARGET HEALTH) via
  `tools/caf/core/evals/verify-target.sh` and, when a sealed audit exists, runs the deterministic
  `code-audit-validate` over it, blocking the transition on a broken target or a new High/Critical
  finding. Emits a caf-gate@1 artefact: the target-health verdict, the audit findings count, and a
  CLEAN or RED result. Used by chief-technology-officer/ship-tasks at step 29, after the
  awh-gate (step 28) and before the done flip (step 30). CLEAN is required to reach done; RED routes
  the task back to ready_to_implement per STATUS-REFERENCE section 1.3. Use when the user
  asks to "run the caf gate", "code-audit this FR", or "gate testing to done with caf". Do NOT use
  for test-regression detection (that is awh-gate) or for spec correctness (that is
  task-audit); this skill catches the class awh cannot see - a build or lint break, a
  route that 404s, a changed data contract.
license: Apache-2.0
metadata:
  version: 1.0.0
  module: skill
  stage: e
  cyberos-template: caf-gate@1
  cyberos-rubric-target: caf_gate_rubric@1.0

allowed_memory_scopes:
  read:
    - project:*
    - module:*
  write:
    - project:fr/{task_id}/caf-gate
audit:
  row_kind: caf_gate_result
  required_fields: [task_id, module, outcome, target_health, findings_high, harness_version]

inputs:
  - { name: fr,            format: task@1, required: true }
  - { name: module,        format: string,            required: true }
  - { name: audit_profile, format: path,              required: true }
outputs:
  - { name: report, format: caf-gate@1 }

triggers:
  - "run the caf gate"
  - "code-audit this FR"
  - "gate testing to done with caf"
  - "target health before done"
---

# caf-gate

The code-audit gate. awh-gate reruns the tests; this skill reruns the target's own build, lint,
typecheck, and test, and audits the code. The two are complementary: awh catches test regressions,
caf catches what awh cannot see - a build or lint break, a route that 404s, a changed data contract
(the CCAF and kymondongiap class of defect). Absorbed from CyberSkill/code-audit-framework, vendored
at `tools/caf/`.

## When it runs

Step 29 of `chief-technology-officer/ship-tasks`, between the awh-gate (step 28) and the
`backlog-state-update-author` done flip (step 30). The done flip is conditional on this skill
returning CLEAN and awh-gate returning GREEN.

## What it does

1. Resolve the task's module and its `modules/<module>/audit-profile.yaml`.
2. Run `bash scripts/caf_gate.sh <module>`. The deterministic floor:
   - TARGET HEALTH: `tools/caf/core/evals/verify-target.sh modules/<module>` runs the module's own
     RUN_COMMANDS (build / lint / typecheck / test) and fails closed if any breaks.
   - AUDIT CONFORMANCE: when a sealed audit exists at `modules/<module>/.caf/`,
     `code-audit-validate --run modules/<module>/.caf --fail-on High` confirms it is conformant and
     carries no new High or Critical finding.
3. Read the verdict. CLEAN (target health passes and no new High/Critical) is required to proceed to
   the done flip. RED routes the task back to `ready_to_implement` per STATUS-REFERENCE
   section 1.3 with `routed_back_count += 1`.
4. Emit one `caf_gate_result` row into the memory audit chain carrying `{task_id, module, outcome,
   target_health, findings_high, harness_version}`. Until the protocol row kind lands, the verdict is
   written to a side log (`.caf/gate-results.jsonl`).

## What it is not

This skill does not rewrite code or tests. It gates and measures. It does not replace the awh gate;
it is the second, complementary axis. See `website/docs/architecture/verification-gate.html` and
`docs/verification/caf-absorption-design.md`, and `tools/caf/` for the vendored tool, with
`tools/caf/RETIREMENT.md` for how the standalone code-audit-framework is retired once every module is
clean under this gate.

## Provenance

Vendored from CyberSkill/code-audit-framework (validator self-test 40/40). Field-data calibration
records at `tools/caf/field-data/`.
