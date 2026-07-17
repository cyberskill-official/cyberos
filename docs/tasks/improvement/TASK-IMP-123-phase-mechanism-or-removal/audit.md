---
task_id: TASK-IMP-123
audited: 2026-07-18
verdict: FAIL
score: 3/10
issues_open: 15
issues_resolved: 0
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean (exit 0). TRACE-001 passes STRUCTURALLY on all seven clauses.
auditor: independent subagent, adversarial, re-ran every grep against the real corpus
---

## §1 - Verdict summary

FAIL at 3/10. The underlying defect is REAL and verified: `phase` is read by no scheduler, absent
from TASK-TEMPLATE.md, absent from task-lint. Everything else is wrong. FOUR of the seven factual
claims are false. 6 of 6 ACs are weaker than their clause. §1.5's renderer obligation has NO AC.
The declared cone omits the 531 files the task rewrites - repeating TASK-IMP-117's exact mistake,
which is recorded in the source comment of the file this task modifies.

## §2 - Findings (ALL OPEN; 15 total, top 8 recorded here, full list with the auditor)

### ISS-001 (CRITICAL) - "the only readers are 2 files" is FALSE. There are FIVE.
data-extract.mjs:141, nfr-extract.mjs:161, render-task-catalog.mjs:43,54,
render-status-hub.mjs:156,319,548,550,559,627, render-nfr-catalog.mjs:53,59,89,217-282.
render-status-hub does NOT render "a badge": it builds a filter facet (`sel('ph','phase',phases)`),
a GROUP-BY dimension, a static table column, and a search hint. That is the same status page
§1.6 promises to keep renderable. §1.5 ("removed from every renderer that reads it") is scoped
against a list missing 3 of 5. And §3 already REQUIRED this: "(b) MUST confirm nothing else reads
them before deleting" - the spec violated its own MUST and shipped the unconfirmed count as fact.

### ISS-002 (CRITICAL) - "two vocabularies" is FALSE. There are FIVE, 31 distinct values.
Measured over 531 specs carrying phase: P-number P0..P5 (330); release-gate `pre-1.0.0 release`,
`pre-1.0.0 hardening`, `post-1.0.0` (38 - THREE values, not two); Wave-numeric Wave 1..6 plus 6
parenthesised track variants (74); Wave-lettered Wave A..E (20); Phase-prose `Phase 0 - safety
rails`..`Phase 4 - ...` (66). The module split is ALSO false: improvement alone carries three
vocabularies / 17 values, and release-gate is the MINORITY there (38 of 123; 81 carry Wave prose).
render-module-changelog.mjs:39 hardcodes ten:{phase:'P4'} while TEN's tasks carry P2/P3/P4 - the
"matching the module map" claim fails on its own exemplar. §1.3 ("the two live vocabularies
reconciled") is normative and written against a false denominator; effort_hours: 4 is sized
against it.

### ISS-003 (CRITICAL) - "the four tasks missing phase" is FALSE. It is TWENTY.
551 specs, 531 carry phase, 20 do not. IMP-117..120 are four of them; the other sixteen span
app, memory, chat (x5), docs, ai, cuo (x6). The author greped the improvement module and reported
it as the corpus. §1.3 and §3 both carry a denominator wrong by 5x.

### ISS-004 (CRITICAL) - "183 done tasks carry phase" is FALSE. It is 176.
183 is the count of `done` tasks TOTAL. 176 carry phase; 7 do not (DOCS-002, CUO-200..204, 301).
One quantity asserted as another - and it is the number sizing the largest open scope question
(rewrite vs grandfather history). A spec whose thesis is "a field that claims something nothing
verifies" published four unverified counts.

### ISS-005 (CRITICAL) - AC 5 is decoration by its own closing sentence.
AC 5 ends "A rule that cannot fail the case that motivated it is decoration." Under branch (b) its
arm is "the test asserts phase is absent from the corpus" - which IS AC 3. The fixture evaporates
and nothing about SCHEDULING is asserted. Worse, the auditor RE-RAN batch-select on the current
tree: TASK-IMP-106 (p1, pre-1.0.0) takes batch 1 and excludes TEN tasks, with phase playing no
part. The motivating harm PROVABLY RECURS under (b). AC 5's headline claim is false. Under (a) it
is circular: it tests the implementation against a rule the implementation itself invents.

### ISS-006 (CRITICAL) - AC 4 is a branch-(b) hole; "the acceptance is symmetric" is false.
AC 4 is "branch (a) only" with no (b) arm, no ops check, no N/A. Under (b): leave it unchecked and
the task can never reach done; check it anyway and it is an unfalsifiable checkbox. TRACE-004
blocks done because t32 is never authored. The rubric HAS a vocabulary for this (TRACE-001's
`(deferred to slice N)`, TRACE-005's §10.7 enumeration) and the spec uses none of it.

### ISS-007 (CRITICAL) - §1.5's renderer obligation [ii] has NO AC.
§1.5 carries three obligations: [i] remove from corpus (AC 3), [iii] lint rejects it (AC 2),
[ii] remove from every renderer that reads it -> NOTHING. AC 6 only asserts build.sh exits 0, and
a build still calling badge('phase', task.phase) on an empty string exits 0 happily. An
implementation stripping the corpus and adding the lint rule while leaving all five phase reads
passes all six ACs and violates §1.5. TRACE-001 sees §1.5 cited twice and passes.

### ISS-008 (MAJOR) - the cone omits 536 of ~540 files, repeating TASK-IMP-117's recorded mistake.
Declared: service tools/install + 4 files. Actually touched: 531 docs/tasks/**/spec.md (§1.3/§1.5),
5 tools/docs-site/*.mjs (§1.5), tools/docs-site/build.sh (AC 6), ship-manifest.mjs (AC 1).
batch-select.mjs:75-82 - the file this task modifies - records the precedent verbatim: "TASK-IMP-117
rewrites 501 specs, TASK-TEMPLATE.md and build.sh, DECLARED NONE OF IT, and was admitted alongside
a task excluded for touching build.sh. Two sub-agents, one file, one parallel round." The author
quoted this comment earlier in the same session and then reproduced the defect.

### Also open: ISS-009 AC 1 discharges a MUST NOT with a self-attested record and compares a
timestamp ship-manifest.mjs does not record (hitl:{gate,requested_at}, no verdict/branch field,
no first-file-write timestamp anywhere - grep 0 hits). ISS-010 AC 2 cannot distinguish branch (a)
from (b) (a lint rejecting ALL phase values passes both arms) and never tests §1.2's template half.
ISS-011 four normative MUST/MUST NOTs hide in §3 outside TRACE-001's reach, and TWO contradict §1
(§3 "grandfathering is the PLAN gate's call" vs AC 3 "the whole corpus" - AC 3 is unpassable if §3
is honoured). ISS-012 AC 3 asserts membership where §1.3 demands presence: 20 bare tasks pass while
violating "every task MUST carry". ISS-013 the Non-Goal freezes release intent inside `priority`,
so branch (b) does not "end the ambiguity" - it relocates it into the field that schedules and
declares the question out of scope. ISS-014 the Problem conflates two causes: "wrong task first"
(phase) and "ten tasks excluded" (service-cone subsumption at :90) - only the first is phase's.
ISS-015 §1.5's "reject phase as an unknown key" needs a strict key whitelist task-lint does not
have; 43 distinct frontmatter keys exist across 551 specs.

## §3 - Verified accurate (credit)
- The core defect is REAL: grep -c phase = 0 in batch-select.mjs AND task-lint.mjs; phase absent
  from TASK-TEMPLATE.md. The status quo is indefensible and the task should exist.
- :88 is the sort (priority THEN id - the spec said "priority alone"; harmless imprecision).
- d19362ad verified, does exactly what the spec says.
- TASK-IMP-118 §3 row 11 quoted accurately - but citing it backfires: 118 §3 is a MATRIX WITH A
  TEST COLUMN, all 11 rows citing t01/t02/t03. 123 put four normative clauses in untested prose
  and cited 118 as licence. The precedent is the counterexample.
- Alternatives Considered is genuinely strong (QA-005 passes). The [DECISION] framing is right.
- TRACE-003 passes: test_batch_select.sh exists (17KB, t01..t17); t30..t34 are consistent additions.

## §4 - Required before re-audit
Re-measure every count (5 vocabularies/31 values, 20 missing, 176 done-with-phase, 5 readers) and
re-derive effort from them. Name all five readers and give §1.5[ii] an AC. Fix the cone to declare
docs/tasks/** + tools/docs-site/*.mjs. Close AC 4 under (b) or exempt it explicitly per TRACE-005
§10.7. Rebuild or delete AC 5. Give AC 1 a negative arm (118 §3 row 8) and confirm what
ship-manifest.mjs can record. Give AC 2 a positive case. Resolve §3 against §1. Reconcile the
Non-Goal with the Problem.
