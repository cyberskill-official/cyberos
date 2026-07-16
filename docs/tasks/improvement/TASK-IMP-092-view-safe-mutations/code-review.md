# TASK-IMP-092 code review

Reviewer: parent ship-tasks agent (batch 3). Diff: `tools/install/docs-tools/backlog-mutate.mjs`
(retally), `tools/install/tests/test_workflow_helpers.sh` (t10-t12), `modules/cuo/chief-technology-officer/workflows/ship-tasks.md`
(two doctrine passages, 2.6.2 -> 2.6.3).

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | flip and insert retally the header from the section's rows | backlog-mutate.mjs retally path (7 retally/LIFECYCLE references); live batch-3 inserts produced `## improvement  (67 draft, 5 ready_to_implement, 20 done)` then flips produced `(67 draft, 5 implementing, 20 done)` - counts tracked the rows, not an inherited baseline |
| 1.2 | a lying header is corrected by ANY mutation | t10_retally_corrects_lying_header ok (flip and insert arms) |
| 1.3 | diff footprint stays 1 row + <=1 header | t11_footprint_holds_with_retally ok |
| 1.1 | t01-t09 stay green | test_workflow_helpers.sh suite ok (half-2 run: 12/12 suites) |
| 1.4, 1.5 | doctrine passages present in source and payload at 2.6.3 | t12_doctrine_view_rules_vendored ok (asserts both passages + version in source AND scratch payload); dist/cyberos/cuo/ship-tasks.md line 257 (one-writer-one-view) and line 211 (committed-object), `workflow_version: 2.6.3` |

## Judgment

- **Correctness vs incident**: the two enablers of the 086 lost-update incident are closed at
  their mechanism. Incremental adjust preserved a 14-off header through six mutations; a full
  retally cannot inherit a lie because it never reads the old counts. The doctrine passages name
  the environment fact (two views, self-consistent reads) and the only evidence that survives it
  (`git show <commit>:<path>`).
- **Blast radius**: the retally reads the section it was already allowed to rewrite; bare headers
  stay untouched (t06 regression green), so the footprint guarantee is unchanged (1.3).
- **Failure mode if wrong**: a header naming a status the section no longer carries, or counts
  drifting again - t10 asserts the correction, t11 asserts it stays inside the footprint.
- **Dogfooding**: this task's own BACKLOG rows were inserted and flipped by the tool under
  review; the counts above are its live output, verified on the committed object per the rule
  the same task introduces.
- **Security**: none. Tool and prose changes; no new execution surface, no secrets.
- **AI-specific**: version bump is deliberate and asserted (t12), not silent; passages are two
  short bullets in the file's existing voice, not a rewrite.

Verdict: no open findings.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
