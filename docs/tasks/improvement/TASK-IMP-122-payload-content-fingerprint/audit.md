---
task_id: TASK-IMP-122
audited: 2026-07-18 (rewrite 3; supersedes the rewrite-2 audit)
verdict: FAIL
score: 6/10
score_history: "4/10 -> 6/10 -> 6/10 -> 6/10 (FLAT for four rounds)"
issues_closed: 4
issues_partially_closed: 3
issues_open: 4 (survived VERBATIM) + 8 new
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean, four rounds running. TRACE-001/002/003 pass every time.
auditor: independent subagent; diffed 2d478393..f8899d64 and measured all four cone combinations
STOP_SIGNAL: >
  The author has failed to raise this score across three rewrites. The failure mode is now
  legible and is recorded in §2. A fourth rewrite by the same author, patching the findings
  below, will likely reproduce it. Read §2 before attempting one.
---

## §1 - Verdict summary

FAIL at 6/10, flat for a fourth round. The operator's maintained-list decision IS honoured and
rewrite 2's two CRITICALs ARE genuinely closed. It fails because the diff proves the hypothesis
it was given: rewrite 3 edited ONLY §1.2, §1.3, §1.4, §1.6 and AC 2/3/4/6/7/10 - the exact block
the prior audit named. Everything outside that block survived VERBATIM, and two of the edits are
REGRESSIONS that deleted working normative text.

## §2 - THE FAILURE MODE (read this before rewriting)

The author patches what the audit NAMES and does not re-read the document. Consequences, all
measured this round:

1. **Four findings survived verbatim** because they sat outside the named block: NEW2-005 (the
   Proposed Solution still says "token"), NEW2-009 (Summary duplicates a proposition), NEW2-010
   (":99 on every run" - THIRD round unedited), NEW2-011 (AC 1's "BOTH stored manifest tokens").
2. **A fix DELETED the clause that catches the live bug** (NEW3-001, below).
3. **A repaired clause now contradicts two unrepaired sections** (NEW3-002, below).
4. **A false number was copied from the evidence file's parenthetical without re-measuring**
   (AC 7's memory counts), while the AI-authorship disclosure claims "every claim here was
   re-measured against source that day". It was not.

The pattern across four rounds: close the named finding, introduce the same defect class one
layer deeper. Rewrite 1 conflated payload/installed cones. Rewrite 2 cited a line range that
excluded the blind spot. Rewrite 3 deleted the direction that catches `cli`. Each is "the check
does not cover the thing it exists to check", relocated.

## §3 - Prior findings
CLOSED: NEW2-001 (cone now covers lib/docs-tools/memory; `:185-198` is BETTER than the audit's
`:184-198` - `:184` is `rm -rf`, not a vendor), NEW2-002 (all 18 vendored paths land in cone ∪
exclusions - the build no longer always-fails), NEW2-007, NEW2-008 (auditor measured all four
combinations: prune cli only -> 1f05a84f/ae756045 MISMATCH; prune store only -> 66bb0459/1f05a84f
MISMATCH; prune both -> MATCH. The independence claim is EXACTLY right).
PARTIALLY: NEW2-003, NEW2-004, NEW2-006.
NOT CLOSED (verbatim survivors): NEW2-005, NEW2-009, NEW2-010, NEW2-011.

## §4 - New findings

### NEW3-001 (CRITICAL, REGRESSION) - §1.4's second direction was DELETED; the check can no longer catch the live defect.
Author verified by diff:
  rewrite 2: "A path the cone covers that `install.sh` does not vendor MUST fail the build."
  rewrite 3: [deleted]
The prior audit called that direction a tautology - true ONLY while §1.2 defined cone == vendored
set. §1.2 no longer says that (it now says "a single explicit list"), so the tautology is gone and
the direction is LOAD-BEARING again. And today's live defect IS exactly that direction: `cli` is
IN the cone and NEVER vendored - the measured cause of 66bb0459 vs ae756045, named by §1.7, and
half of AC 7's own reasoning. Nothing in §1.3 forbids `cli` in the cone; §1.4 now fires only on
vendored-but-unclassified. The build check the rewrite was built around CANNOT FAIL on the defect
that motivated it.

### NEW3-002 (CRITICAL) - the Proposed Solution and Success Metrics still mandate the RETRACTED cone.
Author verified, unedited at :102-103 and :128:
  ":102 ... compare that against the payload's manifest token. Widen the cone"
  ":103 to every directory the payload ships."
  ":128 - Guardrail: every directory present in the payload is inside the cone, enforced at build."
The evidence file RETRACTS that formulation by name: "'the cone MUST cover every directory the
payload SHIPS' is wrong and is what forced ci/, cli/, template/ in and guaranteed self-drift".
§1.3 was repaired; the identical retracted wording survives in two other sections, where it now
contradicts §1.3, §1.7, AC 7 and the operator decision - and mandates precisely the cone AC 7
exists to fail on.

### NEW3-003 (MAJOR) - AC 3 tests class (a) by its trivial member.
`gates.env` sits OUTSIDE every coned dir - excluding it requires nothing. `memory/store/` sits
INSIDE a coned dir - excluding it requires an ACTIVE PRUNE, and is a measured co-equal cause of
the false drift. AC 3 tests the exclusion that cannot break and skips the only one that can.
6 of the 7 paths in class (a) untested.

### NEW3-004 (MAJOR) - the cone's element grammar is undefined; `memory` is the proof.
§1.2 mandates "a single explicit list" and never says what an entry IS. build.sh:354 is
dir-granular. §1.3 needs three kinds: dirs (`cuo`), files (the three under `memory/`), prunes
(`memory/store/`). "cover the vendored FILES under memory/" is file-granular - under which class
(a)'s `memory/store/` entry is DEAD TEXT. Listing `memory/store/` as an exclusion implies
dir-minus-prune - under which AC 3 never tests it. Either reading carries a defect.

### NEW3-005 (MODERATE) - §1.8 has §1.2's disease and did not get §1.2's cure.
§1.8 requires "build.sh's OWN `_rsha()`". `_rsha()` is defined INLINE at build.sh:353, and
**build.sh is NEVER VENDORED** (absent from dist/cyberos/). So `.cyberos/version.sh` and
`.cyberos/lib/update-check.sh` cannot reach it - §1.8 is unsatisfiable for two of three
comparators. It needs exactly what §1.2 gave the cone (one shared, vendored definition). AC 8
tests EQUALITY of digest, not IDENTITY of implementation - so AC 8 PERMITS the duplicated second
implementation that AC 2 forbids for the cone.

### NEW3-006 (MODERATE) - §1.6's invocation cut is incomplete; AC 6 dropped a capability.
update-check.sh:84 gives CYBEROS_PAYLOAD PRECEDENCE over self_root - so reachability there is
decided by the env var, not by where it was sourced. §1.6 hedges ("in its PRIMARY mode") and never
says what the non-primary mode owes. And AC 6 DROPPED update-check.sh entirely - rewrite 2's AC 6
tested it. Regression.

### NEW3-007 (MINOR) - §1.2's deliverable is in neither new_files nor modified_files.
The shared cone file appears nowhere; no such file exists in-tree; install.sh (which must vendor
it for the two installed comparators to read it) is absent from modified_files. effort_hours: 6
is UNCHANGED across all four revisions despite the prior audit's explicit "re-size effort_hours".

### NEW3-008 (LOW) - three citation errors the author inherited and did not re-measure.
- `build.sh:357` cited as the manifest write. Author verified: **:357 is a BLANK LINE**; the write
  is `cat > "$out/manifest.yaml" <<EOF` at **:358**. This is the load-bearing rationale for
  exclusion class (b).
- AC 7: "`memory/store/` 3 payload files vs 8 installed" is **FALSE**. Author measured: **0 and 5**.
  The 3-vs-8 is `memory/`'s tree total, misattributed. It also CONTRADICTS the spec's own §1.7
  ("installs and never ships"). Two clauses, one document, opposite counts.
- §1.2 says "three separate places" and cites TWO ranges (`:185-198` and `:432`).

## §5 - Clause-verb table: 4 of 12 weaker (identical count to rewrite 2)
The SET moved: AC 2/7/10 CLOSED; AC 4/6/8 NEWLY weaker; AC 3 weaker for a new reason. **Three of
the four weak ACs are ones the author rewrote this round.** The rewrite relocated the weakness.
AC 4 restates §1.4's prohibition with no fixture and no observable - the identical defect the
prior audit already flagged once ("the AC never says WHAT to mutate"), recurring at a new AC.

## §6 - Required before re-audit
1. RESTORE §1.4's second direction and AC 4's matching half. Rewrite 2 had it right.
2. Rewrite the Proposed Solution AND Success Metrics - never edited, still retracted wording.
3. Name the mechanism by which §1.4 reads install.sh's vendored set, given §1.2 says no static
   read yields it, and say how it resolves the `memory/$f` loop without hardcoding.
4. Give §1.2's list an element grammar (dir / file / prune); settle `memory`.
5. Give `_rsha()` §1.2's treatment; make AC 8 test identity, not equality.
6. AC 3 must test memory/store/. AC 6 must test update-check.sh + CYBEROS_PAYLOAD. AC 4 needs a fixture.
7. Fix the four verbatim survivors and the three citation errors.
8. Add the shared cone file to new_files, install.sh to modified_files, re-size effort_hours.
