---
task_id: TASK-IMP-122
audited: 2026-07-18 (rewrite 2; supersedes the rewrite-1 audit)
verdict: FAIL
score: 6/10
score_history: "4/10 -> 6/10 -> 6/10 (flat, but the defects are different and narrower)"
issues_closed: 4
issues_partially_closed: 3
issues_open: 11
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean. TRACE-001/002/003 pass. Fails on judgment, three rounds running.
auditor: independent subagent; MEASURED all three candidate cones over both trees
---

## §1 - Verdict summary

FAIL at 6/10. The vendored-set DECISION is right and the auditor measured it working. The spec
cites the wrong eleven lines for it, and that single citation takes docs-tools/, lib/, memory/,
AC 2, §1.3 and §1.4 down with it.

## §2 - Prior findings
NEW-001 PARTIAL (symptom fixed, cone definition newly wrong) | NEW-002 PARTIAL (arms mis-assigned)
NEW-003 CLOSED | NEW-004 CLOSED (the model repair - the gap is now admitted, not reassigned)
NEW-005 CLOSED | NEW-006 CLOSED | NEW-007 PARTIAL ("every run" survives verbatim)

## §3 - The measurement that settles it
Auditor ran all three candidate cones over both trees:
```
A. today's cone (find cuo plugin mcp cli memory)
   payload 66bb0459  install ae756045   MISMATCH
B. §1.2's cone AS WRITTEN (:184-195 minus manifest.yaml + VERSION)
   payload ff7de70e  install ff7de70e   MATCH  <- but only by EXCLUDING docs-tools/ + lib/
C. the ADDENDUM's INTENDED cone (+ docs-tools lib memory, - memory/store)
   payload 102dc507  install 102dc507   MATCH  <- the correct answer
```
So the false-drift half IS closed and the decision IS right. B "passes" by dropping Defect 2.

## §4 - Open findings

### NEW2-001 (CRITICAL) - the cone citation excludes the blind spot; §1.2 and AC 2 contradict.
Author verified: `lib` vendors at :197, `docs-tools` at :198, `memory` at :432 - ALL OUTSIDE the
":184-195" §1.2 makes normative and exclusive ("and nothing else"). So the four artefacts in
docs-tools/ that motivated the task stay uncovered. AC 2 demands the cone include docs-tools+lib
while deriving from a range that omits them: self-defeating in one document. Correct range is
:184-198 PLUS :432.

### NEW2-002 (CRITICAL) - §1.3 and §1.4 are mutually unsatisfiable; the build fails always.
install.sh:188-189 VENDOR manifest.yaml and VERSION (author verified). §1.3 mandates both OUT of
the cone. §1.4: "a path install.sh vendors and the cone does not cover MUST fail the build."
=> the build MUST fail, unconditionally. AC 3 (manifest out of cone, build passes) and AC 4
(vendored-but-uncovered fails the build) assert contradictory outcomes. Same defect CLASS as
rewrite-1's NEW-001, at a new pair of clauses.

### NEW2-003 (MAJOR) - the cone is not mechanically derivable, and §1.4 is circular.
install's copies are conditional (:187, :197-198 `[ -d ] &&`), loop-bound (:432 inside
`for f in AGENTS.md memory.schema.json memory.invariants.yaml`), env-conditional
(memory only when CYBEROS_NO_MEMORY != 1), and :666 copies OUT of $CY (false-positives a naive
grep). Deriving the set is bash static analysis; effort_hours: 6 is not sized for an analyzer and
the spec names no mechanism. And §1.4 is circular: if install's copies come FROM the shared
definition, the check compares the list to itself - unfalsifiable. The second direction is a
tautology once §1.2 defines cone == vendored set.

### NEW2-004 (MAJOR) - §1.6's arms are per-COMPONENT; reachability is per-INVOCATION, and backwards for 2 of 3.
version.sh run as `.cyberos/version.sh` -> $here == $CY -> NO payload tree, cannot name paths.
update-check.sh sourced from .cyberos/lib/ (its PRIMARY mode, header :2) -> self_root == $CY ->
NO payload tree; the source comment even says "when it IS the install, it self-compares equal".
audit-fleet.sh's DEFAULT (:16-18) resolves a REAL tree at dist/cyberos/ - yet §1.6 puts it in the
token-only arm. AC 6 requires the first two to name three paths in invocations where they cannot.

### NEW2-005 (MODERATE) - Proposed Solution and §1.6 describe two different comparisons.
Proposed Solution says compare against "the payload's manifest TOKEN". A digest-vs-token
comparison can NEVER name paths. §1.6's first arm needs a per-file walk of TWO TREES - a
mechanism the Solution never introduces, and one that makes rules_sha redundant to `diff -rq`.
Alternatives forbids "a per-file manifest ... a second comparator" without saying whether a live
two-tree walk IS that second comparator.

### Also open
NEW2-006 AC 3 never tests §1.3's VERSION exclusion (1 of 6 exclusions untested).
NEW2-007 AC 10 tests only soft mode; §1.10 binds strict/always/off too.
NEW2-008 AC 7's cause is incomplete: pruning `cli` alone still leaves ae756045 vs 1f05a84f;
  `memory/store/` is a CO-EQUAL independent cause (payload 3 files, install 8).
NEW2-009 the Summary duplicates a proposition in two consecutive sentences - the tell of a patch
  applied over the prior finding rather than a re-read.
NEW2-010 source_pages' ":99 writes the cache on every run" survives VERBATIM from the prior audit
  (three early returns precede :99).
NEW2-011 AC 1's "BOTH stored manifest tokens" does not apply to audit-fleet.sh (one manifest, one
  env token).

## §5 - Clause-verb table: 4 of 12 weaker (was 3 of 10)
WEAKER: AC 2 (§1.2's "ONE shared definition" half untested), AC 3 (VERSION untested), AC 7
(§1.7's "out-of-cone paths MUST NOT affect the verdict" untested - ci/cli/template never
exercised), AC 10 (one mode of "MUST NOT change its exit semantics").
The rewrite's claim that each AC "states the failure it must produce against TODAY's code" holds
for AC 2 (in letter - but its own derivation cannot detect what it asserts) and AC 7 (digits
reproduced exactly), and is FALSE for AC 10's first half: today's update-check ALREADY exits 0
under soft default, so that half states no failure at all.
AC 3 does NOT test the circularity claim: circularity is a BUILD-TIME ORDERING property
(build.sh:354 computes rules_sha, :357 writes the manifest containing it). Mutating an installed
manifest tests EXCLUSION, not ordering. And the AC never says WHAT to mutate.

## §6 - Required before re-audit
1. Fix the cone citation: :184-198 PLUS :432, and it is env-conditional.
2. Carve manifest.yaml + VERSION out of §1.4's first direction, or drop them from §1.3. AC 3 and
   AC 4 must stop contradicting.
3. State the derivation mechanism, or admit the cone is a MAINTAINED LIST that a build check
   reconciles against install.sh - and re-size effort_hours. Say which direction of §1.4 can fail.
4. Re-cut §1.6 by INVOCATION, not component.
5. Reconcile the Solution's token comparison with §1.6's two-tree walk; say whether that walk is
   the second comparator Alternatives forbids.
6. AC 3 must test VERSION; AC 10 must test the modes §1.10 binds; AC 7 must name memory/store/.
