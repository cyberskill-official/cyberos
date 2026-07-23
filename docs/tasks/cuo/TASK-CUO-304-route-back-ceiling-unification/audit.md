---
task_id: TASK-CUO-304
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean - `node tools/install/docs-tools/task-lint.mjs docs/tasks/cuo/TASK-CUO-304-route-back-ceiling-unification/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Five §1 clauses, five ACs, six edge cases. A deliberately small task (single-constant unification + conformance pin) audited against the full rubric; the size is justified under the pure-infrastructure exception and the spec is complete, not truncated. The consequential findings were the third-copy trap in the pin test's design and the silent-skip failure mode the parser could have reproduced.

## §2 — Findings (all resolved)

### ISS-001 — the first-draft pin test hardcoded 3, creating a third copy of the constant
A literal `assert default == 3` makes the test itself a fork surface: a deliberate doctrine change to 4 fails the test with a message that reads "test expects 3", inviting a test-side fix while api.py keeps drifting. Resolved: clause 1.3 requires deriving N by parsing ship-tasks.md §11b; AC 3 proves derivation by patching the doc text in-memory and asserting the failure names both sides.

### ISS-002 — parser-miss behavior was undefined (silent-skip risk)
If §11b is reworded and the regex misses, a lazy implementation returns None and skips — the exact silently-skipping-conformance-test defect this same batch fixes in `test_schema_drift.py` (TASK-MEMORY-303). Resolved: clause 1.4 mandates loud failure on pattern miss; AC 4 asserts raise-not-skip.

### ISS-003 — semantics of the comparison operator were not pinned alongside the constant
Changing the default 2→3 while someone "helpfully" changes `>=` to `>` would reintroduce the off-by-one under a green constant check. Resolved: clause 1.1 pins BOTH the default and the `rbc >= halt_on_repeat_rework` comparison, and AC 1 asserts behaviorally (rbc 2 re-enters, rbc 3 halts) rather than only reading the signature.

### ISS-004 — `0`-disables semantics could be broken by an over-eager pin
A naive "default must equal doctrine" test could also assert the flag's *value range*, forbidding the documented `0` disable. Resolved: edge case pins `0` as untouched (the `if halt_on_repeat_rework and ...` guard at api.py:289 treats it as disabled) and instructs the pin test not to forbid explicit values.

### ISS-005 — the circuit-breaker half of the plan bullet was silently dropped in the first draft
The plan says "test pinning doctrine constants" (plural); the 5-fail breaker has no Python constant to pin (verified by grep), and the first draft simply didn't mention it — an unexplained scope narrowing. Resolved: Alternatives Considered records the deferral with the measurement, and the test module is named as the designated future home so the deferral is discoverable.

### ISS-006 — in-flight tasks at rbc 2 needed an explicit migration statement
A reviewer would reasonably ask what happens to tasks that would have halted under the old default. Resolved: edge case states no stored state migrates — `routed_back_count` is frontmatter compared only at drain time, so such tasks simply re-enter (doctrine-conformant) and halt at 3.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST default 3 + keep `>=` semantics | signature default AND behavioral halt boundary | AC 1: asserts default via inspect AND rbc-2-re-enters / rbc-3-halts behavior | sufficient after revision (ISS-003) |
| 1.2 MUST default 3 + help text states it + keeps 0-disable doc | option default + two help substrings | AC 2: asserts all three | sufficient |
| 1.3 MUST parse doctrine and compare all three surfaces, naming both sides on mismatch | derivation (not literal) + mismatch message content | AC 3: in-memory doc patch flips all assertions, message names 4 vs 3 | sufficient |
| 1.4 MUST fail loud on parser miss | raise, never skip | AC 4: asserts raise on missing pattern | sufficient |
| 1.5 MUST gain CHANGELOG entry | substrings present in top entry | AC 5: asserts flag name + 2-to-3 mention | sufficient |

## §4 — Resolution

Six findings, all resolved in the audited revision. Size exception: pure-constant + conformance-test task, complete at small scale per the authoring discipline's infrastructure exception (all sections present and substantive; no truncation). **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two human-acceptance gates in `/ship-tasks` are unchanged — this audit clears the spec-correctness gate only.

---

*End of TASK-CUO-304 audit.*
