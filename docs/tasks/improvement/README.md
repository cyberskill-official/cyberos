# improvement — task class index

Improvement tasks (`class: improvement`) are enterprise-hardening, refactoring, and audit-remediation work. They are NOT a separate track: each runs the full `ship-tasks` lifecycle with the mandatory human-acceptance gates, exactly like a product task. This folder is the home and index for cross-cutting improvement tasks, and the migration record for the retired `docs/improvement/` backlogs.

See `modules/cuo/chief-technology-officer/workflows/ship-tasks.md` section 1a for the lifecycle and the gate profile.

## Where improvement tasks live

- Module-scoped hardening goes to `task-<MODULE>-*` under `docs/tasks/<module>/` (memory hardening is `task-MEMORY-*`, chat hardening is `task-CHAT-*`, and so on). Same module index, same BACKLOG grooming as any product task for that module.
- Cross-cutting hardening that spans modules (for example a repo-wide audit remediation) is tracked here as `task-IMP-*`, with this README as the index.

`class: improvement` in the task frontmatter is what marks a task as hardening (the default is `class: product`). The class selects the gate profile (section 1a) and lets grooming and reporting separate hardening from net-new; it does not change the lifecycle or the two human-acceptance gates.

## Migration from the old docs/improvement backlogs (done 2026-07-08)

The three old improvement programs were folded into tasks on 2026-07-08 and renumbered to fresh ids (no `legacy_id` kept, operator choice). The old `docs/improvement/` tree was deleted.

| Old program | Old ids | New tasks | Count |
|---|---|---|---|
| memory enterprise | `MEM-001..060` | `TASK-MEMORY-201..258` (module `memory`) | 58 |
| chat enterprise | `T-001..066` | `TASK-CHAT-201..266` (module `chat`) | 66 |
| deep audit | `IMP-001..067` | `TASK-IMP-001..067` (here, cross-cutting) | 67 |

Every migrated task carries `class: improvement` and landed as `status: draft` (a stub carrying the old title, refs, deps, and acceptance note; the normative clauses get authored when the task is picked up). `done` and `superseded` source tasks mapped to `done` and `closed`. Dependencies were remapped to the new ids.

The old-to-new id map is `MIGRATION-MAP-2026-07-08.md` in this folder. It is the only record linking old ids to new (there is no `legacy_id` on the tasks), so use it to reconcile the in-flight `auto/memory-enterprise` and `auto/chat-enterprise` branches after they merge; their `MEM-*` and `T-*` references no longer resolve to a live backlog. The migration was produced by `scripts/migrate_improvement_to_task.py`.
