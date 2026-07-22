---
task_id: TASK-IMP-130
audited: 2026-07-22
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean — `node tools/install/docs-tools/task-lint.mjs docs/tasks/improvement/TASK-IMP-130-cli-bin-rename-cs/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Seven §1 clauses, seven ACs, five edge cases including one security-class row. All seven clauses trace 1:1 to an AC via `traces_to`; TRACE-006 (verb-vs-assertion) review found one real gap (AC 4) which is now fixed. The most consequential finding was a test-file convention mismatch that would have pointed the implementer at a file structured the wrong way to receive these tests.

## §2 — Findings (all resolved)

### ISS-001 — AC 4 tested only the absence of the old string, not the presence of the new one (TRACE-006)
Clause 1.4 demands the doc files "read `cs` in place of `cyberos`" — a positive replacement. The original AC 4 asserted only that a grep for `cyberos <command>` returned zero matches. A test asserting the negative alone would pass on an implementation that deleted the CLI examples from the docs entirely rather than updating them, which satisfies "zero matches" while failing the clause's actual demand that the reader now sees `cs`. Material: would have passed a test that didn't prove the clause. Resolved: AC 4 now requires both the positive replacement text be present in each of the three files AND the old pattern be absent.

### ISS-002 — the cited test file uses the wrong convention for a `file::test_name` citation
All seven ACs originally pointed at `tools/install/tests/test_channels.sh` using `::t_name` citations. Reading that file showed it is a flat sequential script (`ok()`/`bad()` calls with string labels), not a file of named test functions — the citation convention this contract requires (per `test_install_hygiene.sh`'s actual `t01_gitignore_managed_block()`-style functions, which is what TASK-IMP-129 cited correctly). Citing `test_channels.sh::t_bin_renamed_to_cs` would have pointed an implementer at a file that cannot host a function by that name without restructuring it first, and conflated this task's rename-specific assertions with `test_channels.sh`'s actual purpose (proving every manifest-declared delivery channel works, per its own header comment). Resolved: introduced a new dedicated file, `tools/install/tests/test_cli_rename.sh`, using the named-function convention, and all seven ACs now cite it.

### ISS-003 — `new_files` omitted the test file entirely
Following directly from ISS-002: the original frontmatter's `new_files: [(none)]` was wrong on its own terms even before the file-choice fix — every AC required a new test, so at least one new file was always going to be needed, and the frontmatter didn't say so. Resolved: `new_files` now lists `tools/install/tests/test_cli_rename.sh`.

### ISS-004 — `related_tasks` named TASK-IMP-076 but the body never said why
`related_tasks: [TASK-IMP-076]` was present in the first draft, but nothing in Problem, Proposed Solution, or Dependencies explained the relationship — the same class of gap TASK-IMP-129's own audit (ISS-004 there) flagged as material when a related task is listed but unexplained. TASK-IMP-076 shipped the exact dispatch table and usage text this task renames, and established the "three channels cannot drift" design this task's doc sweep must preserve. Resolved: added an explanatory paragraph to Dependencies.

### ISS-005 — Success Metrics lacked a timeframe (QA-004-adjacent)
Both metrics had a baseline and a target but no deadline, which QA-004 treats as a vanity-metric risk even when the target itself is concrete. Resolved: both metrics now anchor to "the next CyberOS release after 1.0.9."

### ISS-006 — an edge case asserted a specific npm bin-symlink behaviour that was not actually verified
The original edge-case bullet stated a user's old `cyberos` invocation "will silently stop resolving" after the package upgrades to the new bin name — presented as settled fact, but npm's handling of a changed `bin` field on an existing global install (clean removal vs. a dangling stale symlink) was never independently checked in this authoring session; it is version- and install-method-dependent. Asserting a specific mechanism without having verified it is exactly the kind of unsourced technical claim the anti-fabrication discipline exists to catch. Resolved: reframed as an open question that TASK-IMP-134's clean-machine regression test must observe directly, rather than a claim this task asserts as already known.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST declare bin+name | generated config literally contains both fields | AC 1: scratch-build `package.json` has `bin.cs` and unchanged `name` | sufficient |
| 1.2 MUST refer to invocation as `cs` | positive text present AND old text absent in `--help` output | AC 2: asserts both halves against actual stdout | sufficient |
| 1.3 MUST describe npm channel as `npx cs` | same shape as 1.2, against `help.sh` output | AC 3: asserts both halves against actual stdout | sufficient |
| 1.4 MUST read `cs` in docs | positive replacement present, not just old string absent | AC 4 (revised): asserts both halves per file | sufficient after revision (was insufficient pre-revision — ISS-001) |
| 1.5 MUST correct domain string | positive new domain present AND old domain absent | AC 5: asserts both halves | sufficient |
| 1.6 MUST gain CHANGELOG entry | positive content present in top entry | AC 6: asserts three required substrings | sufficient |
| 1.7 MUST NOT modify `modules/memory` | diff scope excludes the path — a "preserve" style check | AC 7: asserts `git diff` touches no file under `modules/memory/` | sufficient |

## §4 — Resolution

Six findings, all material or TRACE-006-material, all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two human-acceptance gates in `/ship-tasks` (review acceptance, final acceptance) are unchanged and remain recorded human verdicts — this audit clears the spec-correctness gate only.

---

*End of TASK-IMP-130 audit.*
