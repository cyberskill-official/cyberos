---
id: TASK-CUO-303
title: Mechanical HITL lock - verdict-gated flips + memory.status_overridden
template: task@1
type: improvement
module: cuo
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-CUO-205, TASK-IMP-120, TASK-IMP-140]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.1.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 10
service: tools/install
new_files:
  - tools/install/tests/test_hitl_lock.sh
modified_files:
  - tools/install/docs-tools/backlog-mutate.mjs
  - tools/install/docs-tools/memory-append.mjs
  - tools/install/install.sh
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - tools/install/tests/test_e2e_skeleton.sh
  - tools/install/tests/test_workflow_helpers.sh
  - CHANGELOG.md
source_pages:
  - "tools/install/docs-tools/backlog-mutate.mjs:277-340 (cmdFlip: enum check, pre-image refusals exit 6, truth-precedes-index guard - NO special handling for the two human-acceptance transitions; any actor that first writes the frontmatter can flip reviewing->ready_to_test or testing->done)"
  - "tools/install/docs-tools/memory-append.mjs:102 (const KINDS = [workflow_phase_complete, workflow_complete, task_routed_back, artefact_write] - closed set; the memory.status_overridden row that STATUS-REFERENCE §1.4 promises for every human verdict cannot be written by the doc-driven appender)"
  - "tools/install/install.sh:319 (writes HITL_REQUIRED=\"true\" into gates.env); measured 2026-07-23: no script in the repo reads HITL_REQUIRED back (its only occurrences are the three install.sh copies: source, dist, installed)"
  - ".cyberos/cuo/STATUS-REFERENCE.md §1.4 (two human-acceptance gates: reviewing->ready_to_test and testing->done; 'Every human verdict or override emits one memory.status_overridden aux audit row capturing {actor, task_id, prior_status, new_status, reason}')"
  - "modules/cuo/EXECUTION-DISCIPLINE.md §2a (HITL required platform-wide; the agent never sets done itself)"
  - "docs/tasks/cuo/TASK-CUO-205-single-backlog-write-path/spec.md (backlog-mutate.mjs is the single documented backlog write path - which is what makes a lock in this tool load-bearing rather than decorative)"
source_decisions:
  - "2026-07-23 operator: CyberOS Hardening Plan approved; Phase 2 T2 'Mechanical HITL lock' authored as an improvement task (plan file cyberos_hardening_plan_49404998; audit finding C2)."
  - "2026-07-23 authoring: HITL_REQUIRED is REMOVED from the generated gates.env rather than consumed, because the lock this task installs is doctrine (unconditional), not configuration - a flag that suggests the lock can be turned off by editing a file would be a second lie replacing the first. The prose comment about the two human gates stays. Recorded here for the reviewer since the plan bullet offered both options ('either consumed or removed')."
---

# TASK-CUO-303: Mechanical HITL lock - verdict-gated flips + memory.status_overridden

## Summary

The two human-acceptance gates (`reviewing -> ready_to_test`, `testing -> done`) are doctrine in STATUS-REFERENCE §1.4 and EXECUTION-DISCIPLINE §2a, but nothing mechanical refuses them: `backlog-mutate.mjs` flips any enum-legal transition, the promised `memory.status_overridden` audit row is unwritable by the doc-driven appender (closed 4-kind list), and `HITL_REQUIRED="true"` in `gates.env` is read by nothing. An agent that ignores the prompt can self-approve its own work end to end. This task adds a verdict gate to the single backlog write path, adds the missing audit-row kind, and removes the dead flag.

## Problem

Audit finding C2, verified first-hand on 2026-07-23:

1. **No transition lock.** `cmdFlip` (`backlog-mutate.mjs:277`) enforces the status enum, a byte pre-image, and truth-precedes-index (TASK-IMP-120) - all integrity checks, none of them authority checks. The two transitions that doctrine reserves for a recorded human verdict flip exactly like any other.
2. **The promised audit row cannot exist.** STATUS-REFERENCE §1.4 says every human verdict or override emits one `memory.status_overridden` aux row. `memory-append.mjs` - the only writer available to doc-driven (non-Python) workflows - refuses every kind outside its closed four (`memory-append.mjs:102`). The doctrine promises an audit trail the tooling cannot produce.
3. **A dead flag implies enforcement that does not exist.** `install.sh:319` writes `HITL_REQUIRED="true"` into every `gates.env`; nothing reads it. A reader auditing the machine sees a flag named like a lock and reasonably concludes one exists.

## Proposed Solution

Gate the two human-acceptance transitions in `cmdFlip` behind two new required flags: `--verdict-by <actor>` (non-empty identity string) and `--verdict-evidence <path>` (an existing, non-empty file - the review note, test-acceptance note, or transcript the human produced). A flip of `reviewing -> ready_to_test` or `testing -> done` without both flags refuses with a new distinct exit code 8 and a message quoting STATUS-REFERENCE §1.4; all other transitions are untouched. When the flags are present, the flip proceeds and additionally appends one `memory.status_overridden` row (payload `{actor, task_id, prior_status, new_status, reason: evidence-path}`) via `memory-append.mjs` when a BRAIN store is resolvable; when no store exists, the evidence file itself is the record and the flip still succeeds - append-failure on a *present* store, however, fails the flip (audit-before-action). Extend `memory-append.mjs`'s closed kind list with `status_overridden` and its payload validation. Remove the dead `HITL_REQUIRED` variable from the `gates.env` generator, keeping the prose comment about the two human gates. Update `ship-tasks.md`'s HITL step descriptions to pass the new flags, and add a CHANGELOG entry marking the new refusal as breaking for any tooling that flips the two gate transitions bare.

## Alternatives Considered

- **Consume `HITL_REQUIRED` (make the lock conditional on it).** Rejected: the lock is doctrine, not configuration - EXECUTION-DISCIPLINE §2a governs platform-wide and offers no opt-out; a flag that looks like it can disable the lock by editing a gitignored file would replace a dead lie with a live one.
- **Verify the verdict actor is human (signature, identity attestation).** Rejected for this task: signed/attributed verdict artifacts are the 1.4.x roadmap item the audit already names. This task makes the verdict *recorded and refusable*, not cryptographically attributable - the honest increment that closes the self-approval path for compliant tooling.
- **Lock the transitions in the spec frontmatter instead (a pre-commit hook rejecting status edits).** Rejected: frontmatter is written by humans and agents alike in editors; a commit-time reject fires after the work is staged, refuses legitimate operator overrides, and cannot capture WHO decided. The flip executor is where authority is asserted (TASK-CUO-205 made it the single write path); guarding there is both sufficient for tooling and non-intrusive for operators.
- **Auto-generate the evidence file when absent.** Rejected: an auto-generated verdict is a self-approval with paperwork; the entire point is that the file preexists the flip because a human produced it.

## Success Metrics

- Primary: by the next CyberOS release, a bare `backlog-mutate flip <id> reviewing ready_to_test` (or `testing done`) exits 8 with no file written, and the same flip with `--verdict-by` + `--verdict-evidence` succeeds and (when a store is present) lands exactly one `memory.status_overridden` row on the chain. Baseline today: the bare flip succeeds silently.
- Guardrail: zero behavior change for every other transition - the existing backlog-mutate coverage in `tools/install/tests/test_workflow_helpers.sh` and the mechanical spine in `tools/install/tests/test_e2e_skeleton.sh` pass unmodified (except where the e2e drives the two gate transitions, which gains the flags), and rework/off-ramp flips (`-> ready_to_implement`, `-> on_hold`, `-> closed`) require no verdict flags.

## Scope

In scope: `cmdFlip` verdict gate + exit code 8, `memory-append.mjs` kind extension + payload validation, `install.sh` gates.env generation (drop the dead variable, keep the prose), `ship-tasks.md` HITL step invocation updates, CHANGELOG entry, and the new test suite.

### Out of scope / Non-Goals

- Cryptographic signing or identity verification of verdict actors (1.4.x roadmap; this task records, it does not attest).
- Locking direct frontmatter edits or the `regen` path - an agent editing `spec.md` by hand bypasses any tool gate; that residual is explicitly accepted and documented in ship-tasks.md, with the transition-locked state engine (1.5.0 roadmap on the 1.x line) as the full closure. The G2 benchmark checker (TASK-IMP-140) asserts the tool-path refusal this task ships.
- The route-back ceiling constant (TASK-CUO-304) and gate-floor behavior (TASK-CUO-302) - separate tasks in this batch.
- Retroactively generating `status_overridden` rows for historical transitions.

## Dependencies

None blocking. Builds on TASK-CUO-205 (done - made `backlog-mutate.mjs` the single documented backlog write path, which is what gives a lock in this tool its force) and TASK-IMP-120 (done - truth-precedes-index; this task's gate runs AFTER those refusals so the refusal precedence is: missing row / drift / truth-mismatch first, verdict gate last). TASK-IMP-140's benchmark gate G2 verifies this task's refusal in CI - soft forward reference via `related_tasks`, no cycle.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill in Cursor, as the task-authoring wave of the 2026-07-23 hardening plan.
- **Scope:** every `source_pages` line was read at HEAD in this checkout during authoring; the absence of any `HITL_REQUIRED` consumer and the closed kind list were verified by repo-wide grep, not carried from the audit report.
- **Human review:** the hardening plan (including this task's scope bullet) was operator-approved on 2026-07-23; the remove-not-consume decision for `HITL_REQUIRED` is recorded in `source_decisions` for the reviewer to revisit at the review acceptance gate.

## 1. Description (normative)

- 1.1 `backlog-mutate.mjs flip` MUST refuse the transitions `reviewing -> ready_to_test` and `testing -> done` unless BOTH `--verdict-by <actor>` (non-empty string) and `--verdict-evidence <path>` (a path that exists and is a non-empty regular file at flip time) are supplied. The refusal MUST use the new distinct exit code 8 (verdict required), MUST name STATUS-REFERENCE §1.4 in its message, and MUST NOT write any file. Every other transition MUST behave exactly as today, verdict flags ignored if supplied.
- 1.2 The verdict gate MUST evaluate AFTER the existing refusals (missing/duplicate row exit 6, pre-image drift exit 6, truth-precedes-index exit 6), so existing failure modes keep their codes and the new code 8 means exactly one thing: the transition was otherwise legal but no verdict was recorded.
- 1.3 `memory-append.mjs` MUST accept the kind `status_overridden` (emitted on-chain as op `status_overridden`, consistent with the existing four kinds), validating a payload object with required non-empty string fields `actor`, `task_id`, `prior_status`, `new_status`, `reason`. Unknown kinds MUST keep today's refusal behavior.
- 1.4 On a verdict-gated flip where a BRAIN store is resolvable (the same store-resolution the appender already implements), the flip MUST append exactly one `status_overridden` row whose payload carries `{actor: <--verdict-by>, task_id, prior_status, new_status, reason: <--verdict-evidence path>}`, and a failed append on a present store MUST fail the flip (audit-before-action: no index move without its audit row). When no store is resolvable, the flip MUST succeed without a row - the evidence file is the record - and MUST say so on stderr.
- 1.5 `install.sh` MUST stop emitting the `HITL_REQUIRED` variable into generated `gates.env` files, keeping the prose comment that states the two human-acceptance gates are never automated. No script consumes the variable today (verified 2026-07-23), so removal changes no behavior.
- 1.6 `ship-tasks.md`'s two HITL steps MUST document the flag-carrying flip invocation as the way the recorded human verdict advances the cell, and `CHANGELOG.md` MUST gain an entry marking the bare-flip refusal as a breaking change for tooling that automates the two gate transitions.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - bare `flip <id> reviewing ready_to_test` and `flip <id> testing done` exit 8, mention "STATUS-REFERENCE" and "verdict", and leave BACKLOG.md byte-identical; the same flips with both flags succeed; `flip <id> testing ready_to_implement` (route-back) needs no flags - test: `tools/install/tests/test_hitl_lock.sh::t01_bare_gate_flip_refused`
- [ ] AC 2 (traces_to: #1.1) - `--verdict-evidence` pointing at a missing path or an empty file refuses with exit 8 and no write; `--verdict-by ""` refuses with exit 8 - test: `tools/install/tests/test_hitl_lock.sh::t02_evidence_must_exist_nonempty`
- [ ] AC 3 (traces_to: #1.2) - on a row whose cell drifted from the recorded pre-image AND with verdict flags absent, the exit is 6 (pre-image) not 8, proving refusal precedence - test: `tools/install/tests/test_hitl_lock.sh::t03_refusal_precedence_six_before_eight`
- [ ] AC 4 (traces_to: #1.3) - `memory-append.mjs append <store> status_overridden` with a complete payload appends a chained row; each missing/empty required field refuses with exit 2 and writes nothing; an unknown kind still refuses - test: `tools/install/tests/test_hitl_lock.sh::t04_status_overridden_kind_validated`
- [ ] AC 5 (traces_to: #1.4) - a verdict-gated flip against a scratch repo WITH a seeded store lands exactly one `status_overridden` row (payload fields match the flags); with the store made unwritable the flip fails and BACKLOG.md is unchanged; with NO store the flip succeeds, no row, stderr notes the evidence file is the record - test: `tools/install/tests/test_hitl_lock.sh::t05_audit_before_action`
- [ ] AC 6 (traces_to: #1.5) - a scratch install's generated `gates.env` contains no `HITL_REQUIRED` substring and retains the human-gates prose comment - test: `tools/install/tests/test_hitl_lock.sh::t06_dead_flag_removed`
- [ ] AC 7 (traces_to: #1.6) - `ship-tasks.md` documents `--verdict-by` and `--verdict-evidence` at both HITL steps, and CHANGELOG's top entry mentions the refusal, the word "breaking", and exit code 8 - test: `tools/install/tests/test_hitl_lock.sh::t07_docs_and_changelog`

## 3. Edge cases

- **Operator superset overrides (STATUS-REFERENCE §1.4)** - e.g. `done -> ready_to_review` re-audit, `ready_to_review -> ready_to_test` skip-review - are NOT the two forward gate transitions and stay flag-free in this task. Widening verdict recording to all overrides is deliberate future scope; this task locks exactly the two transitions doctrine names as mandatory-human.
- **Direct frontmatter edit + regen bypass:** an agent can write `status: done` into spec.md and regenerate the backlog without ever calling `flip`. Accepted residual, stated in ship-tasks.md: the lock closes the documented tool path (the only path compliant workflows use); the 1.5.0 state engine closes the rest. The G2 checker tests the tool path.
- **Evidence file is a directory or unreadable:** treated as "does not exist" - refusal 8. The gate checks regular-file-ness and non-zero size, nothing else; content quality is the reviewer's judgment, not the tool's.
- **`--json` output mode:** the refusal and the success MUST both carry the verdict fields in the JSON envelope so ship-manifest consumers can record them; the exit code is authoritative either way.
- **Two flips racing on the same row:** unchanged from today - the pre-image/optimistic-concurrency refusal (exit 6) fires before the verdict gate (AC 3's precedence), so the race loser cannot consume a verdict.
- **Security-class:** the verdict flags introduce no new execution surface (no eval, no shell-out); the evidence path is read for existence/size only, never executed or parsed. The appended row goes through the existing appender's §4.2 lock + two-phase write discipline.
