---
task_id: TASK-IMP-122
audited: 2026-07-18 (rewrite; supersedes the 4/10 audit of the first draft)
verdict: FAIL
score: 6/10
score_prior_draft: 4/10
issues_closed: 4
issues_partially_closed: 2
issues_open: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean (exit 0). TRACE-001/002/003 all PASS. Fails on judgment, as before.
auditor: independent subagent, adversarial, RAN build.sh's hashing pipeline over both trees
---

## §1 - Verdict summary

FAIL at 6/10, up from 4/10. A real rewrite, not a reword: the false thesis is gone, the true
mechanism is named, REPAIR-vs-REPLACE is decided explicitly, and 4 of 7 prior findings are closed.
Weaker ACs down from 5-of-7 to 3-of-10. It fails on a NEW, MEASURED, design-level defect: the spec
assumes the installed tree is a byte-identical copy of the payload cone. It is not.

## §2 - Prior findings

| id | status |
|---|---|
| ISS-001 "nothing compares content" false | CLOSED - title, summary, related_tasks, source_decisions all corrected |
| ISS-002 true defect never stated | CLOSED - stored-not-recomputed is now the thesis; cone gap named as an independent defect |
| ISS-003 §1.2 unimplementable | PARTIALLY - the per-path manifest is DELETED (legitimate fix; §1.3 checks the OUTPUT, needing no vendor list). But its replacement claim is false -> NEW-004 |
| ISS-004 5 of 7 ACs weaker | PARTIALLY - 4 of 5 bullets closed. One survives VERBATIM: §1.6 "byte-identical" vs AC 7 "freshly installed" |
| ISS-005 AC 7 false-by-construction | CLOSED - §1.9 carve-out + AC 8's soft-default reconciliation. Defect CLASS recurs -> NEW-001 |
| ISS-006 baseline unreproducible | CLOSED - re-anchored to a synthetic repro; e2504cf3 named as the repair |
| ISS-007 normative MUSTs in §3 | CLOSED - exclusion list -> §1.3 + AC 4; tamper ban -> §1.8 + AC 9 |

## §3 - NEW findings

### NEW-001 (CRITICAL) - a clean install CANNOT report current. MEASURED.
The auditor ran build.sh:354's exact pipeline over both trees, with cuo/plugin/mcp/docs-tools/lib
verified byte-identical (diff -rq clean):
```
recomputed over INSTALLED .cyberos : ae756045eec1c63de5e70fd3237eb3c5535f39301a21c771b8ce0f80bf19aed3
recomputed over PAYLOAD  dist      : 66bb0459ad53600e9f59c8180bae745b3dce213f27c8783c86303ca90712c806
stored token in .cyberos/manifest  : 66bb0459... (matches the payload, because it was COPIED from it)
```
Two causes, isolated by control (pruning both -> 1f05a84f on BOTH sides, MATCH):
1. `cli/` is IN the cone (`find cuo plugin mcp cli memory`) and is NEVER INSTALLED. Author
   verified: dist/cyberos/cli exists (1 file); .cyberos/cli absent. `find` drops it silently
   (2>/dev/null) -> different hash.
2. `memory/store/` is install-generated INSIDE a cone directory (payload memory/ = 3 files,
   installed = 8). §1.3 excludes this half only.
Author verified the payload dirs never installed: **ci/, cli/, template/**.
**§1.3 makes it WORSE**: "the cone MUST cover every directory the payload ships" forces ci/ and
template/ in, neither installed -> EVERY install reports drift against itself, which is exactly
what §1.3's own AC 4 says "MUST FAIL if a fresh install reports drift against itself".
Root cause: the spec conflates PAYLOAD cone and INSTALLED cone. They are structurally different
sets and no clause reconciles them. §1.2, §1.3, §1.6, AC 2, AC 3 and AC 7 cannot all hold.

### NEW-002 (CRITICAL) - §1.5 is unsatisfiable by the design the spec chose.
§1.5 "a drift report MUST name every differing path". rules_sha is a SINGLE tree digest:
build.sh:355 pipes per-file lines into a final `_rsha | cut -d' ' -f1`, discarding every per-file
digest (author verified). A tree hash names nothing. Naming paths needs per-file digests -
which Alternatives rejects by name ("a second comparator ... TASK-IMP-104 §1.2 forbids"). And
audit-fleet.sh compares against a BARE TOKEN from CYBEROS_EXPECT_RULES_SHA with no tree reachable,
so per-file recomputation is impossible there even in principle. The central REPAIR decision and
§1.5 are in unacknowledged conflict. One must move.

### NEW-003 (MAJOR) - check-version-sync.sh is not a drift comparator; §1.1/AC 1 are unimplementable for it.
Author verified: `grep -c '\.cyberos' check-version-sync.sh` = **0**. Its header: "read-only
comparator: root VERSION vs every stamped payload artifact". It compares payload stamps to the
root VERSION, never to an install, and its rules_sha block is presence+shape only (:54-58,
"Not compared against VERSION ... presence + shape only"). So the Summary's "four components
compare it" is false, §1.1's enumeration is wrong for 1 of 4, and AC 1's "leaving BOTH stored
manifest tokens untouched" is meaningless for a tool with one manifest and no install.

### NEW-004 (MAJOR) - §3's honest admission is repaired with a false substitute.
§3 concedes correctly that this check cannot catch a vendor-step omission. Then: "and §1.3's
build-time cone check is what catches it instead" - FALSE. §1.3 checks DIRECTORIES; build.sh:198
drops a FILE into docs-tools/, a directory inside the widened cone. Nothing in §1 or §2 catches a
vendor-step omission. A false claim closed with a second false claim.

### NEW-005 (MODERATE) - AC 10's exclusion is WIDER than §1.9's carve-out.
§1.9 grants the .update-check-cache write to lib/update-check.sh ALONE. AC 10 excludes the path
for EVERY component, so version.sh/check-version-sync/audit-fleet could write it and still pass.

### NEW-006 (MODERATE) - §1.7 contradicts itself in one sentence.
"Exit contracts are unchanged: update-check.sh ... MUST NOT change its exit semantics;
audit-fleet.sh MUST NOT keep its present fail-open behaviour" - the second item REVERSES a
contract under a header asserting none change, and never says what audit-fleet does instead.
(The new audit-fleet requirement IS traced - AC 8's tail. The defect is internal contradiction.)

### NEW-007 (MINOR) - §1.7's "either side" and "cone unreadable" untested (AC 8 tests the payload
manifest only - the prior audit's "either side" complaint survives). source_pages' "writes the
cache on every run" is imprecise: three early returns precede :99 (mode off, no VERSION, 12h throttle).

## §4 - Clause-verb table (3 of 10 weaker; was 5 of 7)
AC 8 is the model fix: it asserts the VERDICT (1.7's verb), names the verb in the AC text, and adds
the exit code as a SEPARATE assertion rather than a substitute. AC 3 tests both halves. AC 1/4/5/6
faithful. WEAKER: AC 2 (drops §1.2's cross-platform half entirely), AC 9 (samples drift runs only;
§1.8 binds all output), AC 10 (NEW-005). AC 7 is not weaker but is FALSE (NEW-001).

## §5 - Required before re-audit
1. NEW-001: decide and state which directories are COMPARABLE vs merely SHIPPED. ci/, cli/,
   template/ ship and never install; memory/store/ installs and never ships. Re-derive AC 2 and
   AC 7 against whatever cone survives, and RE-MEASURE.
2. NEW-002: reconcile §1.5 with the tree-hash design or with Alternatives. One must move.
3. NEW-003: scope check-version-sync.sh out of §1.1/AC 1, or define "the installed side" for it.
4. NEW-004: delete or correct the "§1.3 catches it instead" sentence.
5. NEW-006: split §1.7's exit sentence; say what audit-fleet does instead of fail-open.
6. NEW-005 + ISS-004 residue: narrow AC 10 to lib/update-check.sh; reconcile §1.6 "byte-identical"
   with AC 7 "freshly installed" - the one prior finding that survived the rewrite verbatim.
