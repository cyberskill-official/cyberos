---
task_id: TASK-IMP-133
audited: 2026-07-22
verdict: PASS (after revision)
score_pre_revision: 5/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean after three fixes — first run flagged FM-101 (title 76 chars), TRACE-001 (clause 1.5 cited by no AC's structured traces_to field, only by prose), and TRACE-002 (AC 1's "test:" marker broken by a parenthetical inserted before the colon); all three fixed; re-run exits 0 with zero findings
---

## §1 — Verdict summary

Five §1 clauses, four ACs (one removed as redundant during revision), four edge cases including one security-class row. Scored lowest pre-revision of the batch so far because it contained a genuine fabricated test citation — the most serious class of finding possible under this rubric — not merely a precision gap.

## §2 — Findings (all resolved)

### ISS-001 — AC 2 cited a CI workflow that does not exist
The first draft's AC 2 pointed at "`homebrew-tap`'s own CI workflow" as the test authority for a passing `brew test`. Checking the actual repository (directory listing for `.github/workflows/`) found none — `homebrew-tap` has no CI at all. Citing a non-existent automated check as a test is a fabricated citation, materially worse than an imprecise one: an implementer or reviewer trusting this AC would believe a safety net exists where none does. Resolved: revised to a local `brew install --build-from-source` + `brew test` run, documented in the PR description, with the absence of CI stated explicitly rather than implied.

### ISS-002 — AC 3 deferred to unstructured manual review when a mechanical check was straightforward
The original AC 3 called the header-comment wording change "not mechanically testable." A grep for the exact stale sentence (absence) plus a positive marker for the new bin name (presence) is a direct, mechanical check — the same shape already used successfully in TASK-IMP-130's domain-string AC. Resolved: converted to a `grep -c` pair with explicit expected counts.

### ISS-003 — AC 1 didn't verify the same-commit atomicity clause 1.1 actually demands
Clause 1.1 requires `url` and `sha256` to update "together, in the same commit" — the whole point being that the two must never be individually staggered (which is exactly the deterministic-`brew-test`-failure window this task exists to prevent). The original AC 1 checked only that the final values were correct, not that they arrived together. Resolved: added a `git log -p` check confirming both fields changed in one commit.

### ISS-004 — AC 5 was redundant with the corrected AC 2
Once AC 2 was corrected to a real local `brew test` run (ISS-001), it already provides direct proof that the referenced release exists — a `brew test` can only pass if the pinned `url`/`sha256` resolve to a real tarball containing the `cs` bin. The original AC 5 (a PR-description checklist item confirming the release "was queried live") added a second, weaker, human-trust-based check for something AC 2's mechanical pass/fail already settles. Resolved: removed AC 5, retraced clause 1.5 directly to AC 2.

### ISS-005 — no task in this five-task batch actually covers cutting and publishing the npm release both this task and TASK-IMP-134 depend on
TASK-IMP-130's own ACs only prove a scratch build's `package.json` is correct — none of them require an actual release reach the npm registry. This task's entire premise (a `cs`-bin release exists to point the Formula at) and TASK-IMP-134's premise (an end-to-end regression against a real install) both assume that operational step happened, but nothing in the batch owns it. This is a real gap in the plan's task set, not something TASK-IMP-133 can close by itself. Resolved: named explicitly as an edge case rather than left as an unstated assumption, and carried forward to the batch-level report.

### ISS-006 — the header comment fix's target wording was under-specified before AC 3's mechanical rewrite
Before ISS-002's fix, clause 1.3 said only "reworded to state the current, accurate naming" with no concrete required substring — soft enough that two different implementers could satisfy it in incompatible ways. Resolved as a side effect of the AC 3 mechanical rewrite, which now pins a specific required marker (a `` `cs` `` mention adjacent to "bin"/"command").

### ISS-007 — FM-101 title length, and two TRACE structural failures caught only by the machine floor
Running `task-lint.mjs` after the six findings above were resolved surfaced three more, none of which the manual pass caught: (1) `FM-101`, title at 76 chars, over the 72 cap — shortened to "Homebrew tap: update cyberos-cli.rb for the cs rename". (2) `TRACE-001` on clause 1.5 — the clause's prose said "(traced by AC 2: ...)" but AC 2's own `traces_to:` field only listed `#1.2`, so the linter (which parses the structured field, not free prose) correctly saw clause 1.5 as uncited — fixed by adding `#1.5` to AC 2's `traces_to:` list. (3) `TRACE-002` on AC 1 — the AC's test description read "test (manual, ops flow - ...): ..." with a parenthetical inserted between the word "test" and its colon, so the literal substring `test:` never appeared and the linter flagged it as carrying neither a `test:` nor `verify:` entry — fixed by moving the parenthetical after the colon. All three are exactly the class of defect a mechanical floor exists to catch and a careful manual read can still miss.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST update url+sha256 together, same commit | correct values AND atomic commit | AC 1 (revised): both value correctness and `git log` atomicity | sufficient after revision (atomicity was untested - ISS-003) |
| 1.2 MUST update test block bin name | positive behavioural proof | AC 2 (revised): a real local `brew test` pass, not a fictitious CI | sufficient after revision (was fabricated - ISS-001) |
| 1.3 MUST correct header comment | absence of stale sentence AND presence of correct marker | AC 3 (revised): grep pair, both counts specified | sufficient after revision (was unmechanised - ISS-002/006) |
| 1.4 Formula name MUST NOT change | diff scope excludes a rename | AC 4: `git diff --stat` shows no rename | sufficient |
| 1.5 MUST NOT merge before release exists | the same evidence AC 2 already provides | AC 2 (retraced): a passing local `brew test` cannot exist without a real release | sufficient after retracing (was redundantly double-covered - ISS-004) |

## §4 — Resolution

Six findings, including one fabricated-citation defect (the most severe class this rubric checks for), all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. This task's two `/ship-tasks` human-acceptance gates will need to run against a `homebrew-tap` checkout, not this repo — noted in the spec's Dependencies section, unaffected by this audit's scope.

---

*End of TASK-IMP-133 audit.*
