---
task_id: TASK-IMP-132
audited: 2026-07-22
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean after one fix — first run flagged FM-101 (title 73 chars, one over the 72 limit); title shortened to "Add a `cuo` verb to `cs` as a redirect stub"; re-run exits 0 with zero findings
---

## §1 — Verdict summary

Seven §1 clauses (including a split 1.4/1.4a), six ACs, four edge cases including one security-class row. This task's findings cluster around test-design precision and an under-explained asymmetry with its sibling task, rather than a wrong technical premise — the underlying design (a pure-text redirect stub) was sound from the first draft.

## §2 — Findings (all resolved)

### ISS-001 — AC 1 was redundant with, and weaker than, AC 4
The first draft's AC 1 asserted only that bare `cs cuo` did not print "unknown command." AC 4 (the listing behaviour) already proves something strictly stronger for the same invocation — that it prints all four valid names and exits `0` — which trivially implies the weaker claim. Citing clause 1.1 to its own separate, weaker AC added a test that could pass without adding evidence beyond what AC 4 already provides. Resolved: retraced clause 1.1 to the `cs cuo plan` AC instead, which positively proves recognition for a *different* invocation shape (a valid name, not the bare case), making the two ACs complementary rather than one subsuming the other.

### ISS-002 — clause 1.4 conflated two different exit-code semantics
The original single clause covered both "bare `cs cuo`" and "`cs cuo <unrecognised-name>`" with the same "exit 0" requirement. `cli.mjs`'s own established convention (exit `2` for "unknown command," `cli.mjs:87`) treats a genuine usage mistake differently from a bare, information-seeking invocation (exit `0`, `cli.mjs:59`). Treating a mistyped workflow name the same as asking for orientation loses a signal a calling script might want (was this a mistake, or a deliberate ask for help). Material: the spec would have shipped correct-looking behaviour with the wrong exit code. Resolved: split into 1.4 (bare, exit `0`) and 1.4a (unrecognised name, exit `2`), with matching separate ACs.

### ISS-003 — AC 5's test mechanism does not match this repo's actual test conventions
The original AC 5 called for "a process-spy wrapper around `child_process.spawnSync`" — a JS-level module-mocking approach with no precedent anywhere in `tools/install/tests/`, which is exclusively bash scripts asserting on stdout/exit codes against stub binaries on `$PATH` (see TASK-IMP-131's own tests). An implementer following this AC literally would need to invent a mocking mechanism the rest of the suite doesn't use. Resolved: revised to the tripwire-binary style already established — stand-in `python3`/`bash` scripts that write a marker file if invoked, asserted absent afterward.

### ISS-004 — AC 6's documentation check was satisfiable by an unrelated match
The original check ("contains a word from {redirect, prints, run inside}") tested for the word's presence anywhere in the whole file, which could pass on a coincidental, unrelated use of one of those common words elsewhere in `help.sh` or `tools/install/docs/index.md` (e.g. "prints this text" describing an unrelated flag) without the `cuo` verb itself being described as a redirect at all. Resolved: tightened to require the descriptive word on the same or adjacent line as the actual `cuo` mention.

### ISS-005 — `tools/install/docs/index.md:27`'s "same eight commands" count was never flagged as needing an update
This task (and its sibling TASK-IMP-131) each add a new verb to the same dispatch table `tools/install/docs/index.md:27` describes as having exactly eight commands. Neither task's first draft had any clause or AC checking that this hardcoded count gets updated — a real, concrete way for the docs to go silently stale the moment either verb ships. Resolved: added to clause 1.6 and AC 6, with an explicit note that whichever of TASK-IMP-131/132 lands second owns the numeral update.

### ISS-006 — the asymmetry with TASK-IMP-131's local-availability detection was unexplained
Clause 1.5 forbids `cuo` from probing whether `cyberos-cuo` is installed locally, while TASK-IMP-131's `memory` verb does exactly that (detects and reports local availability) for its own underlying tool. Left unexplained, a reader comparing the two sibling specs could reasonably conclude one of them made an arbitrary or inconsistent design call. It is not arbitrary — the plan itself constrains `cuo` more tightly than `memory` ("do not implement standalone execution," with no mechanism left open, versus `memory`'s "mechanism ... is an implementation decision"). Resolved: added the explanation directly to clause 1.5.

### ISS-007 — FM-101: title exceeded the 72-character limit by one (caught by the machine floor)
`task-lint.mjs`, run after the six findings above were resolved, flagged the title at 73 code points — one over the cap. Resolved: shortened to "Add a `cuo` verb to `cs` as a redirect stub" (title metadata only).

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST recognise as known command | positive routing occurs for a valid name | AC 1: `cs cuo plan` reaches its redirect output, which only happens if `cuo` was recognised | sufficient after retracing (was redundant/weak - ISS-001) |
| 1.2 MUST redirect to `/plan` | positive text + exit code | AC 1: both asserted | sufficient |
| 1.3 MUST redirect the other three names | positive text + exit code per name | AC 2: all three asserted | sufficient |
| 1.4 bare invocation MUST list + exit 0 | positive listing + specific code | AC 3: both asserted | sufficient |
| 1.4a unrecognised name MUST list + exit 2 | positive listing + a DIFFERENT specific code than 1.4 | AC 4: both asserted, distinct from AC 3 | sufficient after the split (was conflated - ISS-002) |
| 1.5 MUST NOT spawn/probe | absence of a side effect across all four valid invocations | AC 5 (revised): tripwire markers absent after all four run | sufficient after revision (mechanism was unrealistic - ISS-003) |
| 1.6 MUST document as redirect + correct count | positive proximate text + a computed-not-hardcoded number | AC 6 (revised): both asserted with proximity + dynamic count | sufficient after revision (was loose - ISS-004, ISS-005) |

## §4 — Resolution

Six findings, all material, all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two human-acceptance gates in `/ship-tasks` remain unchanged and are recorded human verdicts.

---

*End of TASK-IMP-132 audit.*
