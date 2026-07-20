---
artefact: implementation-plan@1
task_id: TASK-IMP-086
created: 2026-07-16
estimate_pts: 1
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 1)
---
# Implementation plan - TASK-IMP-086

Slices (each maps to §1 clauses and edge-case-matrix rows):
1. Byte-authority trial FIRST (spec Alternatives mandate) - relocate scripts/migrate_improvement_to_task.py plus a copy of docs/tasks under /tmp/dry86 (its ROOT resolves from its own path, so the repo cannot be written), run `--backlog`, diff the regenerated file against the pre-image, and decide on evidence: churn of pre-existing rows -> surgical fallback (§ Alternatives; matrix row 3). Outcome recorded in gate-log E1: 3 done-row deletions + Totals rewrite + zero 068-081 rows (ACTIVE filter) -> fallback.
2. Row construction from frontmatter truth - yaml.safe_load each 068-081 spec.md; HALT (never invent) on missing/unparseable/mismatched-id/ multi-line-title; emit `- [<status>] <STEM> - <title>` with status and title verbatim, untagged, matching the section corpus (§1 #1.2, #1.3; matrix rows 1, 2, 4, 6). Titles flow yaml -> f-string -> file with zero manual transcription, so em dashes, backticks, apostrophes and embedded ` - ` separators survive byte-exact.
3. Surgical splice - assert the 067 and 082 rows are adjacent, insert the fourteen rows between them (stem-ascending lands by construction), touch no other line (§1 #1.1, #1.3, #1.5; matrix row 5).
4. Header recompute from ALL rows in the section (082-084 done and 085-087 implementing included), formatted in the file's own STATUS_ORDER convention with zero-count statuses omitted; byte-compare before write (§1 #1.4; matrix row 1). Outcome: tally 67 draft, 3 implementing, 17 done - identical bytes to the pre-existing header, so the diff stays insertions-only.
5. Evidence battery recorded to gate-log-draft.md - pre-image scan (E0), folder-vs-row parity (E2), tally-vs-header (E3), duplicate-stem scans + block order (E4/E4b), per-row verbatim recheck (E5), `git diff --stat` + hunk headers + -U0 insertion coordinates + section-bounds proof (E6) (§1 #1.6; ACs 1-4).

Pattern conformance (context-map): row grammar and header grammar pinned to migrate_improvement_to_task.py:205 and :198-199/:21-22; repair direction per STATUS-REFERENCE §1 (frontmatter -> index, one way); batch-1 precedent of retaining done rows honored; no permanent parity test added (spec Out of scope), no other section touched, no row grammar change.

Estimate: 1 pt (~1 h) - matches spec effort_hours: 1. Actual landed surface: docs/tasks/BACKLOG.md +14/-0 (one hunk), six artefact docs in the task folder, zero code shipped.
