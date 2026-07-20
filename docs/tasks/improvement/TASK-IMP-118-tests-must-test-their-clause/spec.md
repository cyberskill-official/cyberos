---
id: TASK-IMP-118
title: A test that cites a clause must test that clause
template: task@1
type: improvement
module: improvement
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T15:05:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
service: modules/skill
new_files:
  - (none)
modified_files:
  - modules/skill/task-audit/RUBRIC.md
  - modules/skill/task-audit/SKILL.md
routed_back_count: 0
awh: N/A
---

# TASK-IMP-118 - a test that cites a clause must test that clause

## Summary

Add a judgment-family rubric rule: a test cited by a clause must exercise the clause's own verb. TRACE-004 checks that a cited test PASSES. Nothing checks that it tests the clause. TASK-IMP-108 §1.7 shipped `done` through both human gates with its clause unsatisfied and its cited test green.

## Problem

TASK-IMP-108 §1.7: "The status page MUST RENDER a staleness report." The implementation computed the report and injected it into the JSON payload. Nothing rendered it - `status-app.js` had zero references to the key. The cited test asserted `grep -q '"draft_staleness"'`: that the string appears in the HTML. It does, inside a JSON blob no code reads.

So the clause said RENDER and the test said PRESENT-IN-PAYLOAD. Every gate held:

- the machine floor passed - FM/SEC/COND are structural and have no view into what a test means
- TRACE-004 passed - the clause cites a test, the test passes
- the model audit passed - it read a clause with a test beside it and stopped there
- both HITL gates passed - the human was shown a green suite and a truthful-looking summary

Nothing in the chain compares the ASSERTION to the PROMISE. The gap is not that a check failed; it is that no check was pointed at this. An author who writes both the clause and its test can satisfy every existing rule while testing something strictly weaker than what was promised, and the weaker test is the one most likely to be written, because it is the one that passes first.

External review found it by reading the clause and asking whether anything did it. That question is not in the rubric.

## Proposed Solution

Add rule TRACE-006 to `audit_rubric@2.0`, judgment family: for every clause with a cited test, the audit MUST state what the clause's verb demands and what the test actually asserts, and MUST fail when the assertion is weaker than the verb. Verb-to-evidence expectations for the recurring cases (render / reject / refuse / halt / emit / preserve) go in the rubric so this is a check, not a mood.

No lint can do this - it requires reading a test and a sentence and comparing meaning. That makes it a model-audit rule, and it belongs with the other judgment families rather than in task-lint.

## Alternatives Considered

- Extend TRACE-004 rather than add a rule. Rejected: TRACE-004 is mechanical (does the cited test pass) and runs at the coverage gate; this is judgment and runs at the spec-correctness gate. One rule doing both jobs is the mistake TASK-IMP-108 itself split apart.
- Require an independent author for each test. Rejected: no mechanism, and the swarm ships one-member batches routinely. A rule nobody can execute is the disease, not the cure.
- Require every clause to name its observable. Partially adopted: TRACE-006 asks the audit to state the verb's demand, which forces the observable into the open without a schema change.
- Do nothing; rely on external review. Rejected: external review found this one, and external review is not a gate we control or can require.

## Success Metrics

- TRACE-006 exists in the rubric with worked examples, including 108 §1.7 as the anti-example.
- Re-auditing 108 against the amended rubric FAILS on §1.7's original test and PASSES on its replacement. This is the acceptance evidence: a rule that cannot fail the case that motivated it is decoration.
- `task-audit`'s SKILL.md instructs the auditor to perform the comparison explicitly per clause.

## AI Authorship Disclosure

- Tools used: Claude (Fable 5) during the 2026-07-17 hardening run, via create-tasks.
- Scope: spec drafted by the agent. The defect it generalizes is the agent's own - I wrote both 108 §1.7's implementation and the test that certified it, and shaped the test to what I had built rather than to what the clause promised. Found by external review (Devin), not by me and not by any CyberOS gate.
- Human review: @stephencheng recorded the decision to task this rule and to append the analysis to IMPROVEMENT_HANDOFF.md, at the gate where the defect was disclosed.

## Dependencies

None. audit_rubric@2.0 and the task-audit skill both exist. This adds one judgment-family rule.

## Scope

In scope: the TRACE-006 rule text in `modules/skill/task-audit/RUBRIC.md`, the verb-to-evidence table, `task-audit/SKILL.md`'s per-clause instruction, a re-audit of 108 as the acceptance evidence, and the §12 analysis in IMPROVEMENT_HANDOFF.md.

### Out of scope / Non-Goals

- Re-auditing the other 180 done tasks against TRACE-006. That is a corpus sweep and its own decision, exactly like TASK-IMP-117's. Sizing it belongs in the handoff, not in this task.
- Any change to TRACE-004 or the coverage gate. This rule fires at spec correctness, before code.
- A lint implementation. This rule is unmechanizable by construction; pretending otherwise would ship a check that passes on 108 §1.7's original test, which is the whole failure again.

## 1. Clauses

1.1 `RUBRIC.md` MUST carry TRACE-006: for each clause citing a test, the audit states the clause's verb, states what the cited test asserts, and fails when the assertion is weaker than the verb. Test: `t01_rubric_carries_trace_006`

1.2 TRACE-006 MUST carry a verb-to-evidence table covering at minimum render, reject, refuse, halt, emit, preserve - naming, for each, what evidence discharges it and what does NOT. Test: `t02_verb_table_is_complete`

1.3 TRACE-006 MUST use 108 §1.7 as its worked anti-example, quoting the clause, the original assertion, and why the assertion was weaker. Test: `t03_anti_example_is_present_and_specific`

1.4 `task-audit/SKILL.md` MUST instruct the auditor to perform the comparison per clause and to record both halves in the audit body. The single source `modules/skill/task-audit/SKILL.md` carries the instruction; `build.sh` vendors it to every payload location. Test: `t04_skill_instructs_the_comparison`

1.5 TRACE-006 MUST be judgment-family and MUST NOT be added to task-lint. A structural check that appears to enforce it would pass 108 §1.7's original test and restore the false assurance. Test: `t05_not_in_the_machine_floor`

## 3. Edge case matrix

| # | Category | Trigger | Expected | Test |
|---|---|---|---|---|
| 1 | NULL/EMPTY | clause cites no test | TRACE-001/004 territory, TRACE-006 silent | t01 |
| 2 | NULL/EMPTY | clause has no verb (a statement of fact) | TRACE-006 does not fire | t02 |
| 3 | BOUNDS | test asserts MORE than the verb demands | passes - stronger is never a finding | t02 |
| 4 | BOUNDS | test asserts exactly the verb | passes | t02 |
| 5 | MALFORMED | clause carries two verbs ("MUST render and MUST NOT change") | both compared; either weaker fails | t02 |
| 6 | MALFORMED | test name suggests the verb, assertion does not | fails - the name is not evidence | t03 |
| 7 | SECURITY | "MUST refuse" discharged by asserting a log line | fails - refusal is an exit code | t02 |
| 8 | SECURITY | "MUST NOT execute" discharged by a passing happy path | fails - absence needs a negative arm | t02 |
| 9 | CONCURRENT | one test cited by several clauses | compared against each verb separately | t02 |
| 10 | DEGRADATION | auditor cannot read the cited test | fails - unreadable is not satisfied | t01 |
| 11 | DEGRADATION | rule applied to a pre-TRACE-006 done task | reported, never auto-failed - history is not retroactively broken | t03 |

## 4. Out of scope / non-goals

See "## Scope -> ### Out of scope / Non-Goals" above.

## Acceptance criteria

- AC1 (traces_to #1.1): TRACE-006 is in the rubric. Test: `t01_rubric_carries_trace_006`.
- AC2 (traces_to #1.2): the verb table covers the six verbs and names non-discharging evidence. Test: `t02_verb_table_is_complete`.
- AC3 (traces_to #1.3): 108 §1.7 is the worked anti-example, quoted. Test: `t03_anti_example_is_present_and_specific`.
- AC4 (traces_to #1.4): the source task-audit SKILL.md instructs the comparison. Test: `t04_skill_instructs_the_comparison`.
- AC5 (traces_to #1.5): TRACE-006 is absent from task-lint. Test: `t05_not_in_the_machine_floor`.
- AC6: re-auditing 108 §1.7 against the amended rubric FAILS on the original test and PASSES on the replacement. A rule that cannot fail its own motivating case is decoration. Test: `t06_rule_fails_the_case_that_motivated_it`.
