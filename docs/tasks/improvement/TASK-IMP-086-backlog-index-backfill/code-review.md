# TASK-IMP-086 — code review packet

Files under review: modified `docs/tasks/BACKLOG.md` (+14/−0, one hunk — the
task's whole cone) and the six new artefact docs in
`docs/tasks/improvement/TASK-IMP-086-backlog-index-backfill/` (gate-log-draft.md,
context-map.md, edge-case-matrix.md, impl-plan.md, obs-injection.md, this
packet). Any other dirt in the same working tree belongs to batch siblings
(TASK-IMP-085 under tools/install/** and modules/**, TASK-IMP-087 under
docs/release/**) and is covered by their own packets. Evidence state at review:
gate-log-draft.md E0–E6 recorded; all E2–E6 commands are pure reads and
re-runnable.

## Path decision (disclosure)

The spec's Alternatives PREFER full-section regeneration via
`scripts/migrate_improvement_to_task.py regen_backlog` and require trying it
first. Tried first, against a /tmp copy so the repo stayed untouched (gate-log
E1). Rejected on the recorded output: it deletes the three pre-existing `[done]`
rows 082–084, rewrites the repo-wide `Totals:` line, and — decisively — emits
ZERO rows for 068–081, because `regen_backlog` lists only ACTIVE statuses
(migrate_improvement_to_task.py:19-20, :201) and all fourteen tasks are `done`.
The regenerator therefore both violates §1 #1.5 and cannot satisfy §1 #1.1;
the spec's own fallback (surgical backfill) was taken. Both paths were held to
the same ACs, as the spec demands.

## §1 clause → proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | every `docs/tasks/improvement/TASK-IMP-*` folder has exactly one row after this task (fourteen backfilled: 068–081) | E2 — 87 folders, 87 rows in the section; E0 — zero 068–081 rows pre-existed (grep exit 1), so one insertion each is exactly one row each; E4 — zero duplicate stems post-image (folder stems 0, row-stem field 0) |
| 1.2 | each backfilled row's status cell = frontmatter `status` at write time; title = frontmatter `title` verbatim | E5 — all fourteen rows re-compared byte-for-byte against a FRESH yaml parse of each spec.md (`- [<status>] <stem> - <title>`, exactly one matching line per task, 0 mismatches). Statuses all read `done` at write time; titles flowed yaml → row with no manual transcription (E-SPLICE), so em dashes, backticks, apostrophes and embedded ` - ` survive byte-exact |
| 1.3 | section's existing grammar, untagged, stem-ascending within the contiguous block | E5 (grammar + untagged, matching the corpus rows — no `(improvement)` suffix anywhere in the section); E4b — `sort -c` over the whole 87-row block passes, insertions landed between the 067 and 082 rows |
| 1.4 | header counts recomputed from ALL rows in the section, matching a per-status tally exactly | E3 — tally 67 draft, 3 implementing, 17 done (67+3+17=87, including the 082–084 `[done]` and 085–087 `[implementing]` rows) equals header line 171 exactly, in the file's own STATUS_ORDER convention (migrate_improvement_to_task.py:21-22, :198-199), zero-count statuses omitted. The recompute reproduced the pre-existing header byte-for-byte (E-SPLICE: `header byte-change: False`) — the old header already forward-counted 068–081 from frontmatter; the clause demands recompute-and-match, which is what happened. E1's regenerator independently computes the same header |
| 1.5 | no line outside the improvement section modified; no pre-existing row edited other than the header count line | E6 — `git diff --stat`: 1 file, 14 insertions(+), 0 deletions; single `-U0` hunk `@@ -239,0 +240,14 @@` = pure insertion after old line 239; new lines 240–253 sit inside section bounds (header :171, next section :261). Zero pre-existing lines changed — the header clause's allowance went unused because the recomputed header was byte-identical. The `Totals:` text on the `@@` line is git's funcname context label, not a change (removed: 0). The 085/086/087 rows: byte-untouched |
| 1.6 | parity check recorded in the gate log: folder count = row count, zero duplicate stems, counts match tally — the commands and their output | gate-log-draft.md — E2, E4 (three scans + the located-and-explained TASK-IMP-080 token hit inside 081's title tail), E3; plus E0 pre-image, E1 trial record, E5 verbatim rechecks, E6 footprint. Spec directs the record at audit.md §gate-log — that section is a placeholder populated at the testing phase, so this task-owned draft carries the evidence for folding in at acceptance |

## Acceptance criteria

AC 1 (§1 #1.1–1.3) evidenced by E2+E0+E5+E4b · AC 2 (§1 #1.4) by E3+E-SPLICE ·
AC 3 (§1 #1.5) by E6 (+E1 for the rejected churn path) · AC 4 (§1 #1.6) by
E0+E4. All four are ops-verified per the spec's stated rationale (one-shot
content chore; the permanent parity test is explicitly out of scope because it
would go red on other sections' pre-existing drift).

## Diff size

`docs/tasks/BACKLOG.md`: +14/−0, one hunk, insertions only — the review-sized
diff the spec promises. Six new markdown artefacts in the task folder, zero code,
zero dependencies, no other file touched (context-map:
files_outside_immediate_domain 0). Known out-of-scope residue, disclosed: the
repo-wide `Totals:` line still says 155 done while the corpus tallies 158 (E1) —
other sections' pre-existing drift, named out of scope by the spec, and fixing it
here would itself have violated §1 #1.5.

## Verdict

| Check | State |
|---|---|
| §1 clauses 1.1–1.6 | each proven above by a recorded evidence item |
| Primary metric (index parity: every folder exactly one row; header = tally) | pass (E2 87=87, E3 exact match) |
| Guardrail metric (diff touches only the improvement section, proven by recorded `git diff --stat`) | pass (E6: one insertion-only hunk inside lines 171–260) |
| Regenerator-first mandate | honored and recorded (E1), fallback taken on its own §1 #1.5 evidence |
| Edge cases (off-ramp statuses, ` - ` titles, unparseable-spec halt, duplicate stems, hostile title bytes) | matrix rows 1–6, each covered by a recorded evidence item |
| Invariants (§5: frontmatter stays truth, repair one-way; no row deletion, no reordering; HITL — agent never sets done) | intact (E6: 0 deletions; E4b: order preserved; no status field written anywhere) |

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
