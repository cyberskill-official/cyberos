---
id: TASK-IMP-011
title: "Structured gate-failure taxonomy"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: improvement
priority: p0
status: done
phase: Wave 1 - see and survive
refs: [Stage, 0]
depends_on: []
created: 2026-07-08
verify: T
owner: Stephen Cheng (CTO)
language: bash
service: tools/install/gates/
new_files:
  - tools/install/tests/test_gate_failure_taxonomy.sh
modified_files:
  - tools/install/gates/run-gates.sh
  - CHANGELOG.md
source_pages:
  - "tools/install/gates/run-gates.sh (machine floor: build/lint/test/coverage + optional caf/awh + doctor)"
  - "tools/install/tests/test_fail_closed_gates.sh (exit-code contract: 0/1/2/3)"
source_decisions:
  - "2026-07-25 batch/12c: scope to CyberOS 1.x payload gates (not platform cargo/caf taxonomy). Failure classes map 1:1 onto real gate steps plus empty-floor."
  - "2026-07-25 operator mission: emit machine-readable summary on failure (.cyberos/last-gate-failure.json + stdout JSON line) so failures can be mined."
---

# TASK-IMP-011: Structured gate-failure taxonomy

## Summary

When the machine-gate floor goes RED, operators and automation today only see human prose (`FAIL test`, `GATES: RED`). This task makes every gate failure emit a structured class and a durable JSON summary so regressions can be mined without scraping logs.

## Problem

`run-gates.sh` prints PASS/FAIL/SKIP lines but does not classify failures. Downstream tooling (auto-revert, dashboards, triage) cannot tell a lint miss from a coverage regression or an empty floor without brittle string matching.

## Proposed Solution

Extend `tools/install/gates/run-gates.sh` so each configured gate step maps to a fixed taxonomy class. On any RED outcome, write `.cyberos/last-gate-failure.json` (schema `gate-failure@1`) listing every failed gate with class, command, and source, and emit one `GATE_FAILURE_JSON:{...}` line on stdout for log miners. Green runs MUST remove a stale failure file so "last failure" always means the most recent RED.

## Alternatives Considered

- **Emit only stderr prose codes.** Rejected: not machine-parseable across shells/CI wrappers.
- **Full OpenTelemetry span per gate.** Rejected: out of 1.x payload scope; JSON summary is enough to mine.
- **Per-ecosystem subclass taxonomy (cargo/clippy/etc.).** Rejected: 1.x floor is build|lint|test|coverage|doctor|caf|awh; subclasses belong in later waves.

## Success Metrics

- Primary: a forced failing gate writes `gate-failure@1` JSON with the matching class; suite asserts taxonomy present.
- Guardrail: green runs stay exit 0 and clear any prior failure file; exit codes 1/2/3 unchanged.

## Scope

In scope: `run-gates.sh` taxonomy + failure artifact; install suite tests; CHANGELOG.

### Out of scope / Non-Goals

- Platform cargo llvm-cov / caf finding taxonomies.
- Changing HITL verdict mechanics.
- Shipping a dashboard consumer (miners are out of band).

## Dependencies

None. Soft: TASK-IMP-026 may read the failure artifact; not required to ship 011.

## AI Authorship Disclosure

- **Tools used:** Cursor agent authoring against live `run-gates.sh` in batch/12c.
- **Scope:** 1.x payload gate steps only; classes adjusted to real floor.
- **Human review:** session operator (Stephen Cheng) via batch/12c HITL continuum.

## §1 - Description (normative)

1. Every gate step `run-gates.sh` executes (build, lint, test, coverage, caf, awh, doctor) MUST map to exactly one failure class from the closed set: `build | lint | test | coverage | doctor | caf | awh | empty-floor | other`.
2. When any configured gate command exits non-zero, the runner MUST record that failure (class, gate name, command, provenance source) and continue remaining gates (today's behavior), then exit 1.
3. On exit 1 (one or more gate failures) OR exit 3 (empty floor), the runner MUST write `$root/.cyberos/last-gate-failure.json` with schema `gate-failure@1` containing at least: `schema`, `exit_code`, `failures` (array of `{class,gate,cmd,source}`), and for empty-floor a single failure with `class: empty-floor`.
4. On those RED exits the runner MUST also print exactly one stdout line matching `GATE_FAILURE_JSON:` followed by the same JSON object (compact, single line).
5. On exit 0 (GREEN or EMPTY-ACKNOWLEDGED) the runner MUST delete `.cyberos/last-gate-failure.json` if present.
6. Exit codes MUST remain: 0 green/ack, 1 gate failed, 2 missing/malformed config, 3 empty floor.
7. The taxonomy MUST be covered by `tools/install/tests/test_gate_failure_taxonomy.sh` with scenarios that force a failure and assert class + artifact presence.

## Acceptance criteria

- [x] AC1: Forced `gates.test: "false"` yields exit 1, `last-gate-failure.json` with `class: test`, and a `GATE_FAILURE_JSON:` stdout line. (t01)
- [x] AC2: Empty floor yields exit 3 and failure class `empty-floor`. (t02)
- [x] AC3: Green run deletes a pre-existing `last-gate-failure.json`. (t03)
- [x] AC4: Unknown/custom gate label (if any) maps to `other` without crashing. (t04)
- [x] AC5: Payload build still vendors `cuo/gates/run-gates.sh` from `tools/install/gates/run-gates.sh`. (t05)

## Test plan

`bash tools/install/tests/test_gate_failure_taxonomy.sh` via `scripts/tests/run_all.sh` glob.
