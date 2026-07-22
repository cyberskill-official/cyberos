---
task_id: TASK-IMP-131
audited: 2026-07-22
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean after one fix — first run flagged FM-101 (title 85 chars, over the 72 limit); title shortened to "Add a `memory` verb to `cs`, gated on local availability"; re-run exits 0 with zero findings
---

## §1 — Verdict summary

Five §1 clauses, five ACs, five edge cases including one security-class row and one newly-added identity-confusion row. The most consequential finding in this task is not a rubric-mechanical one — it's the discovery, made by reading `build.sh`'s actual file-copy lines rather than trusting the plan's prose, that the npm payload does not vendor `modules/memory`'s Python implementation at all. That finding reshaped this task's entire scope before the six ISS findings below were even reached.

## §2 — Findings (all resolved)

### ISS-001 — AC 1 tested only the absence of a failure message, not the presence of correct routing (TRACE-006)
Clause 1.1 demands `memory` be recognised as a known command. The original AC 1 asserted only that "unknown command" text was absent. A no-op implementation that silently swallowed the `memory` verb without dispatching anywhere would also produce no "unknown command" text, and would pass. Resolved: AC 1 now also asserts that, given a working stub, the stub's own output actually appears — proving the command is routed to the resolution/dispatch logic, not merely that one specific failure string is missing.

### ISS-002 — AC 2's non-$PATH-lookup requirement had no concretely distinguishing test
Clause 1.2 forbids resolving availability via a bare `$PATH` lookup of `cyberos`. The original AC 2 described this requirement but did not specify a test setup capable of telling the two mechanisms apart — "not used to decide availability" is not itself observable. Resolved: AC 2 now sets up two differently-labelled fake binaries (a `$PATH` `cyberos` that would print `WRONG-PATH-DISPATCH`, a `python3 -m cyberos` stub that would print `CORRECT-DISPATCH`) so a test can prove which mechanism actually fired.

### ISS-003 — Success Metrics' primary lacked a baseline statement
Unlike TASK-IMP-129 and TASK-IMP-130 (both of which state "Baseline today: ..."), the first draft's primary metric stated only the target state. Resolved: added "Baseline today: `memory` is not in `cli.mjs`'s `SCRIPTS` table at all... falls through to the 'unknown command' branch."

### ISS-004 — the resolution check does not verify the resolved module's identity
`python3 -m cyberos --help` succeeding is treated as sufficient evidence that `cyberos-memory` specifically is present. An unrelated Python package also importable as `cyberos` on the same machine (low probability, but not impossible) would be indistinguishable to this resolution check and could be silently mis-dispatched to. This was not named anywhere in the first draft. Resolved: added as an explicit, accepted-limitation edge case rather than left as a silent gap — proportionate hardening (a signature check against the real CLI's own `--help` text) is named as a deferred future option, not built into this task.

### ISS-005 — no coordination note between this task and its dispatch-table sibling, TASK-IMP-132
Both this task and TASK-IMP-132 add a new entry to the same `cli.mjs` dispatch table, and both depend only on TASK-IMP-130 — nothing in either task as first drafted stopped them from being implemented in parallel with no awareness of each other, risking a merge conflict or an inconsistent dispatch-table shape. Resolved: added a Dependencies note naming the risk and requiring whichever lands second to rebase against the first.

### ISS-006 — clause 1.4's failure exit code was left as "non-zero" instead of matching the file's own established convention
`cli.mjs` already uses exit code `2` for two other "recognised but unusable" cases (gates missing at `cli.mjs:76`, unknown command at `cli.mjs:87`). The first draft's clause 1.4 and AC 4 said only "non-zero," which would technically pass an implementation using an inconsistent code (e.g. exit `1`) elsewhere in the same file. Resolved: tightened both the clause and its AC to require exit code `2` specifically.

### ISS-007 — FM-101: title exceeded the 72-character limit (caught by the machine floor, not the manual pass)
Running `task-lint.mjs` against the spec — after the six findings above were already resolved — flagged `FM-101`: the frontmatter title was 85 code points, over the rubric's 72-character cap. The manual audit pass above did not catch this (title length is exactly the kind of mechanical check a linter exists to catch reliably where manual review is inconsistent). Resolved: shortened to "Add a `memory` verb to `cs`, gated on local availability" (title metadata only — no clause, AC, or normative content changed). Recorded here rather than silently folded into the machine-floor summary line, since a real defect the manual pass missed is worth surfacing on its own.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST recognise as known command | positive routing occurs, not just one failure string absent | AC 1 (revised): both halves - absence of unknown-command text AND presence of stub output when available | sufficient after revision (was insufficient - ISS-001) |
| 1.2 MUST resolve via python3, MUST NOT via bare `$PATH` cyberos | a test that can tell the two mechanisms apart | AC 2 (revised): two distinctly-labelled fakes prove which one fired | sufficient after revision (was unfalsifiable as originally worded - ISS-002) |
| 1.3 MUST spawn + forward exit code | args and exit code both observed | AC 3: echoed args + exit code 3 both asserted | sufficient |
| 1.4 MUST print message + exit code 2 + not invoke python | specific code, specific message content, and the environment already precludes invocation | AC 4 (revised): code exactly 2, message substrings, no python3 present at all | sufficient after revision |
| 1.5 MUST document + state gating | positive mention plus a caveat marker in both files | AC 5: asserts both | sufficient |

## §4 — Resolution

Six findings, all material, all resolved in the audited revision. Machine floor clean by manual pass. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two human-acceptance gates in `/ship-tasks` remain unchanged and are recorded human verdicts, not superseded by this audit.

---

*End of TASK-IMP-131 audit.*
