# `BUG-*` — per-type rule family for `type: bug`

Loaded by `task-audit` when `type: bug` (FM-108). Applies **in addition to** the
common `FM-*` / `SEC-*` / `SAFE-*` / `TRACE-*` families, and **replaces** the
feature-only edge-case-matrix floor (`total_rows >= 8`).

Scored out of 10 like every other rubric. `task-audit` refuses to pass below 10/10.

---

## §10.1  Structure

| rule_id | Rule | Severity |
|---|---|---|
| `BUG-001` | `## Reproduction` present, with **numbered, deterministic** steps and an `Environment` line. Prose like "sometimes fails under load" is not reproduction. A reader who cannot make the bug happen from these steps cannot confirm it is gone. | error |
| `BUG-002` | `## Expected vs observed` present, with the two stated **separately**. Not a merged narrative. Separating them is how you catch the bugs where the expectation was the thing that was wrong. | error |
| `BUG-003` | `## Root cause` present and is a **mechanism**, not a restatement of the symptom. Rejected if the root-cause text is a paraphrase of the observed text (token overlap > 0.6 after stopword removal), or if it contains no file/function/line reference. | error |
| `BUG-004` | `## Blast radius` present with all four: who is affected, since when, workaround, data integrity. The data-integrity line decides whether a code fix is sufficient or a backfill is also owed — the single most commonly skipped question in bug triage. | error |
| `BUG-005` | `## Prevention` present and non-empty. | warning |

## §10.2  Frontmatter

| rule_id | Field | Rule | Severity |
|---|---|---|---|
| `BUG-010` | `severity` | required, one of `sev1`..`sev4`. **Distinct from `priority`.** Severity is how bad it is if left alone; priority is when we will get to it. A sev1 you have chosen to defer is a legitimate, and legible, state. Collapsing them hides that decision. | error |
| `BUG-011` | `regression_test` | required, `<path>::<testname>`, and the path must exist. | error |
| `BUG-012` | `first_bad_commit` | optional, but if present must resolve (`git cat-file -e`). | error |
| `BUG-013` | `incident` | required when `severity: sev1`. A sev1 with no incident link is a sev1 nobody told anyone about. | error |
| `BUG-014` | `type` | must be `bug` for this family to load. | error |

## §10.3  REGRESSION — the load-bearing rule

Run by `coverage-gate-audit` during the `testing -> done` transition, alongside the
`TRACE-004` clause-to-test check that features get.

| rule_id | Rule | Severity |
|---|---|---|
| `REGRESSION-001` | The test named in `regression_test` **passes at `HEAD`**. | error |
| `REGRESSION-002` | The test named in `regression_test` **fails at `first_bad_commit`** (or `HEAD~1` if null). Checked by worktree-checkout + run — not by inspection. | error |
| `REGRESSION-003` | The gate records the raw terminal output of **both** runs (red before, green after) in the coverage-gate artefact. Assertion without evidence is not evidence. | error |

### Why REGRESSION-002 exists

A test written *after* a fix, against the *fixed* code, will pass. That proves
nothing at all — it does not test the bug, it tests the absence of the bug in a
world where the bug was never there. The only way to know a regression test tests
the regression is to watch it fail on the broken commit.

This is the bug-type analogue of the feature-type edge-case matrix, and it is
strictly stronger, because it is machine-checkable:

```bash
git worktree add /tmp/bugcheck "${first_bad_commit:-HEAD~1}"
# apply ONLY the new test file into the old worktree
( cd /tmp/bugcheck && <test-runner> "$regression_test" )   # MUST be non-zero
( cd .              && <test-runner> "$regression_test" )   # MUST be zero
```

Non-zero then zero, with both terminals captured. Anything else and the task does
not reach `done`.

### The one exception

If the bug **cannot** be reproduced in a test (an infra failure, a vendor outage, a
data-corruption event with no code path), `regression_test` may be `null` **only**
with an explicit `no_regression_test_reason:` field carrying an operator-recorded
justification. `REGRESSION-004` then requires that reason to be non-empty and
signed by a human, and the task carries the exemption in its audit row forever.

Making the exemption *possible* but *loud* is the point. A gate you cannot ever
bypass gets bypassed by disabling the gate.

## §10.4  What a bug does NOT need

Bugs skip these phases unless `repo-context-map` reports the fix crosses a module
boundary (`blast_radius.module_count > 1`):

- `architectural-spike-author`
- `architecture-decision-record-author`
- `software-design-document-author`

A one-line null check does not need an ADR. Forcing one is how you teach people to
route around the process.

`edge-case-matrix-author` still runs, but the `total_rows >= 8` floor is lifted:
the matrix is scoped to the cause's neighbourhood, not the whole feature surface.
