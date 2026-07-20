# TASK-IMP-101 code review

Reviewer: parent ship-tasks agent (batch 5). Diff: ship-tasks.md (+~55, two §§ + step 0 + outputs + version), test_workflow_helpers.sh (+~20, t14 + pins).

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | reconcile entry §: trigger, invocation, fork, no-silent-execution, route-back mechanics | ship-tasks.md "## Reconcile entry" (before Resume semantics); t14 greps trigger + rule in source AND payload |
| 1.2 | depends_on evidence gate §: MUST + both homes + override with its memory row | ship-tasks.md "## depends_on evidence gate"; t14 greps "MUST carry evidence" |
| 1.3 | chain step 0 conditional, reconcile_report in outputs | ship-tasks.md skill_chain step 0 (no renumbering of 1-31); outputs list entry; t14 greps step 0 |
| 1.4 | version 2.7.0, pins moved | frontmatter 2.7.0; t09 + t12 pins at 2.7.0; suite 14/14 |
| 1.5 | source AND payload gated; t01-t13 green | t14 asserts both trees; `test_workflow_helpers: pass=14 fail=0` |

## Judgment

- **This closes the trust model's third leg.** The workflow trusted its manifests and its gates, then extended that trust to any status cell it read. After 086 we know a cell is a claim. The § says what happens when a claim outruns its evidence - and, deliberately, it says a human decides. The mechanism measures; it never adopts.
- **Why blocking deps rather than warning**: the operator's call, and the right one. A warning that appears in a wall of output is a warning nobody reads; a block with a one-key override costs a second when the foundation is fine and saves a subtree when it is not. The three false-block guards (both artefact homes, off-ramps, history) keep it from crying wolf on the existing corpus.
- **No renumbering** of steps 1-31 for step 0: every ship-manifest records step indices, so renumbering would silently invalidate resume state across the corpus. Step 0 sits above the chain as a conditional entry - the cheap correctness choice.
- **Placement**: entry § immediately before Resume semantics, so a reader meets both trust mechanisms in one pass and cannot miss which owns which state.
- **Security**: doctrine text; the tool it invokes is read-only by TASK-IMP-100's contract.

## Disclosures

1. **t12 and t09_doctrine_wiring pins moved together** to 2.7.0 - the known identical pair (first found in batch 4). Both moved, behavior untouched.
2. **The chain-coverage obligation** (skill named -> payload must carry it in both trees) was satisfied by TASK-IMP-100's vendoring; that is why the two are ordered, and why 101 depends on 100 rather than shipping alone.

Verdict: no open findings.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
