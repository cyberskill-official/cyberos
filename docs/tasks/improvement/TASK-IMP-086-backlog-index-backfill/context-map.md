---
artefact: repo-context-map@1
task_id: TASK-IMP-086
created: 2026-07-16
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated (0), ADR trigger evaluated)
---
# Repo context map - TASK-IMP-086

## Baseline patterns the change must follow
- Row grammar: `- [<status>] <STEM> - <title>`, stem = the task folder's basename, title verbatim, UNTAGGED in the improvement section (the section's whole corpus is untagged; `(improvement)` tags appear nowhere in it) - pinned_in: docs/tasks/BACKLOG.md:173 (first improvement row) through :260, and the emitter at scripts/migrate_improvement_to_task.py:205
- Header grammar: `## <module>  (<N> <status>, ...)` - two spaces before the paren, counts joined ", ", statuses in STATUS_ORDER (draft, ready_to_implement, implementing, ready_to_review, reviewing, ready_to_test, testing, done, on_hold, closed), zero-count statuses omitted - pinned_in: scripts/migrate_improvement_to_task.py:21-22 (order) and :198-199 (join); visible in every section header of the file (e.g. `## inv  (10 draft, 1 ready_to_implement)`)
- Repair direction: frontmatter is the record of truth; on any index mismatch, repair the BACKLOG toward frontmatter - pinned_in: .cyberos/cuo/STATUS-REFERENCE.md §1 (quoted in the spec's Problem section)
- Regenerator semantics (why the gap existed): `regen_backlog()` lists ONLY active-status rows and merely COUNTS done/closed/on_hold in headers - pinned_in: scripts/migrate_improvement_to_task.py:19-20 (ACTIVE set) and :201 (the filter); done tasks 068-081 therefore never had rows, while the header's `17 done` already forward-counted them from frontmatter
- Batch-1 precedent this section now follows: done rows ARE retained in the improvement section (082-084 sit as `[done]` rows at docs/tasks/BACKLOG.md:254-256 post-change) - the spec makes that explicit for the backfill ("including shipped ones like 074") and forbids deleting them (§5 "No row deletion")
- Sort key: the contiguous row block is bytewise stem-ascending (001..087); insertions land between the 067 and 082 rows - pinned_in: scripts/migrate_improvement_to_task.py:203 (`sorted(remaining)`) and gate-log E4b (sort -c over the whole block)

## Schemas / interfaces in scope
- Status vocabulary: the 12-value frontmatter enum from STATUS-REFERENCE §1 including off-ramps (`on_hold`, `closed`); the emitter carries the frontmatter value VERBATIM (no mapping table), so any off-ramp would flow through into both the row cell and the header tally. Today all fourteen read `done` (gate-log E-SPLICE, E3).
- Consumers of the rows: the ship-tasks state engine (eligibility scans key on the stem token, field `$3` of a row) and human readers (header counts). Titles are display tail after the FIRST ` - ` separator - five backfilled titles contain further ` - ` sequences and parse fine because nothing keys on the tail.

## Files outside the immediate domain (docs/tasks/BACKLOG.md + this task folder)
None. The one modified file IS the cone (docs/tasks/BACKLOG.md, +14/-0); the six new files (this map, edge-case-matrix.md, impl-plan.md, obs-injection.md, code-review.md, gate-log-draft.md) all live in docs/tasks/improvement/TASK-IMP-086-backlog-index-backfill/. The regenerator trial wrote only under /tmp/dry86 (script relocated so its ROOT resolved there - gate-log E1).

files_outside_immediate_domain: 0 (<= 3 -> no ADR trigger).

## Blast radius
file_count: 1 modified (+14/-0) + 6 new artefact docs | module_count: 1 (docs/tasks index) | cross_module_edges: none in code - the change is inert markdown; the behavioral edge is that ship-tasks' queue scan and every human glance now SEE fourteen previously invisible tasks (all done, so no new eligible work appears - the queue's visible state becomes truthful rather than larger)
module_placement_warning: null (spec declares `service: docs/tasks`; the row block and header are exactly where §1 fixes them)
Behavioral radius: zero execution surface (spec §3 security-class: none). Pre-existing rows byte-untouched, including the 085/086/087 `[implementing]` rows and the repo-wide `Totals:` line (its own 155-vs-158 drift predates this task and is spec-out-of-scope). Other sections: untouched by construction, proven by the single -U0 hunk at lines 240-253 inside section bounds 171-260 (gate-log E6).
