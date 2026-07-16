# TASK-IMP-101 edge-case matrix

| # | Category | Trigger | Expected | Covered by |
|---|---|---|---|---|
| 1 | DOUBLE-HANDLING | valid manifest + odd-looking status | resume semantics own it; reconcile does not fire | § text ("A VALID manifest means resume semantics own the task"); t14 greps the trigger clause |
| 2 | ENTRY DRIFT | status past ready_to_implement, no manifest | step 0 fires, report to the gate | t14 (step 0 + trigger present in source and payload) |
| 3 | HUMAN RULE | any branch | never executed without the recorded verdict | t14 greps "NEVER executes a branch" |
| 4 | HISTORICAL CORPUS | done upstream with artefacts under docs/tasks/.workflow/<id>/ | evidence accepted - no false block | § text (both homes); t14 greps the deps rule |
| 5 | OFF-RAMP DEP | depends_on names a closed/duplicate task | unmet dependency, surfaced | § text (existing eligibility semantics restated) |
| 6 | OVERRIDE | operator overrides the block or the recommendation | permitted; emits memory.status_overridden | § text; the two §§ both name the row |
| 7 | VERSION | normative change without a bump | impossible - t09/t12 exact pins fail | t09, t12 at 2.7.0 |
| 8 | VENDORING | passages in source but not payload | t14 asserts BOTH (the "correct in modules/, absent from dist/" class) | t14 |
| 9 | REGRESSION | the additions disturb helper behavior | t01-t13 green | full suite 14/14 |
| 10 | SECURITY | none - doctrine prose + pins; the named tool is read-only by 100's contract | n/a | reviewed |
