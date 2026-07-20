---
task_id: TASK-IMP-128
audited: 2026-07-20
verdict: NEEDS_HUMAN
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 9/10
issues_resolved: 4
issues_open: 1
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_body_sha256_prefix: c4d7f06ecc5861fd
audited_file_sha256_prefix: 6d154032ab437a16
machine_floor: task-lint clean (0 findings) after TRACE-003 fix
hitl_category: success_metric_targets
---

## §1 — Verdict summary

Four §1 clauses, four ACs, five edge cases including one security-class row. Machine floor clean after one structural fix. Three of four clauses pass TRACE-006. **§1.1 does not**, and the gap is not a drafting slip that can be fixed by rewording — it is a genuine bootstrap limitation that needs an operator decision. This audit HALTS rather than resolving it, per the rubric's `clause_verb_untested` routing.

## §2 — Findings

### ISS-001 — TRACE-003: cited test file existed in neither `new_files` nor the repo (RESOLVED)
All four ACs cite `tools/install/tests/test_ci_runs_suite.sh`. The file does not exist and frontmatter `new_files` read `- (none)`, so every AC traced to a path nothing would ever create. Caught by the machine floor at four line offsets. Material: all four ACs were untestable as written. Resolved: `new_files` now declares the file.

### ISS-002 — the original finding this task came from was factually wrong (RESOLVED)
An earlier statement of this defect claimed a release dispatch would give `test_release_assets.sh` its first execution. That was false — the payload job runs `release-assets.sh`, the producer, not the test. Carrying the wrong claim into the spec would have made the Problem section unfalsifiable in the reader's favour. Resolved: the corrected claim (no workflow invokes it, so it has never executed anywhere) is what the spec carries, and the correction is disclosed in AI Authorship rather than quietly substituted.

### ISS-003 — "suite green" needed its actual enforcement named (RESOLVED)
The spec initially described the CI gap without stating what the 1.0.0 pass figure actually rests on. That is the consequence a reader needs. Resolved: Problem now states the figure is enforced only by a machine-local pre-commit hook, on one macOS host, bypassed by docs-only commits and by `--no-verify` — which was itself used during this release.

### ISS-004 — the job must not be merged green-by-suppression (RESOLVED)
First ubuntu run will likely surface real BSD-vs-GNU failures. Without a bound, the path of least resistance is `continue-on-error` to get the PR merged, which would ship a job that cannot fail — the exact defect class this task exists to end. Resolved: §1.2 forbids masking, and §3 states explicitly that the job MUST NOT be merged with failures suppressed.

### ISS-005 — TRACE-006: §1.1's verb is not assertable by any in-repo test (OPEN — needs_human)

**Clause verb-demand.** §1.1 says a CI job "MUST run `bash scripts/tests/run_all.sh` on `ubuntu-latest` for every push and pull request". The verb is *run*, and per RUBRIC.md §9 a run-verb demands evidence of execution.

**Cited test assertion.** AC 1 (`t_suite_job_declared`) parses the workflow YAML and asserts a job is *declared* with the right runner, invocation and triggers. That is a declaration check. It would pass identically against a workflow that GitHub never executes — a syntactically valid job on a disabled workflow, in a repo with Actions turned off, or gated behind a condition that is never true.

**Comparison.** Declaration is strictly weaker than execution. The assertion does not exercise the clause's verb.

This is not fixable by rewording without making it worse. Weakening §1.1 to "MUST declare a job" would produce a clause whose test passes while no test ever runs in CI — a rule that documents its own non-enforcement, which is the precise pattern this task and its two siblings were authored to eliminate. Strengthening the test is not available either: no test inside the suite can assert that CI ran the suite, because the suite only runs if CI runs it. The task that makes CI enforcement real cannot be enforced by CI until it has landed.

**Operator decision required.** Three options, none auto-selectable:

1. Keep §1.1's verb and accept that its evidence is the workflow's first green run on the merge commit — an operator-verified step recorded at the acceptance gate, not an AC. Honest, and puts the evidence where it actually exists.
2. Split §1.1 into a declaration clause (testable, AC 1 as written) plus a separate operator-attested criterion for the first observed run. Preserves a 1:1 clause-to-test mapping at the cost of an extra clause.
3. Accept AC 1 as sufficient on the grounds that declaration is the only lever this repo controls, and record the limitation in §3 rather than as an open finding. Fastest; leaves a known verb/evidence gap in the corpus.

I have not chosen. Option 1 is what I would recommend, but the rubric routes `clause_verb_untested` to the operator and the whole point of this batch is that gates which cannot fail must not be waved through by the party that authored them.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demands | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST run on ubuntu for push/PR | execution evidence | AC 1: workflow YAML *declares* the job | **INSUFFICIENT — ISS-005** |
| 1.2 failing test MUST fail the job, MUST NOT be masked | non-zero propagation and absence of suppression | AC 2: fixture failure exits non-zero and no step swallows it | sufficient on both halves |
| 1.3 MUST execute rather than self-skip | the skip branch is not taken on Linux | AC 3: asserts assertions run instead of the GNU-tar skip | sufficient |
| 1.4 MUST report pass/fail/skip counts | three counts on stdout | AC 4: asserts all three emitted | sufficient |

## §4 — Resolution

Four findings resolved, one open and routed to the operator. **Score = 9/10.** Status remains `draft`; this audit does NOT authorise the `draft -> ready_to_implement` transition. Re-audit after the ISS-005 decision is recorded in `source_decisions`.

---

*End of TASK-IMP-128 audit.*
