---
batch: ship/batch-8-integrate
members: []
started: 2026-07-23T23:00:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# Batch 8 integrate — git story (8a + 8b + 8c)

## Topology

- `ship/batch-8a-core-locks` @ `369f1364` — CUO-302/303/304 → done
- `ship/batch-8b-install-ci-skills` @ `8dacf084` — IMP-136/137 + SKILL-202 → done
- `ship/batch-8c-memory` @ `68a7a73a` — MEMORY-303 → done

`369f1364` (8a tip) is an ancestor of both 8b and 8c. 8b and 8c diverged after shared
`e7d3eb06` (MEMORY-303 store repair + IMP-138 Branch A decision record). Each branch
advanced its own BACKLOG/status-hub rows, so a naive checkout of either tip lost the
other tip's `done` cells.

## Resolution

Created `ship/batch-8-integrate` from `ship/batch-8c-memory`, merged
`ship/batch-8b-install-ci-skills`. Conflicts in `docs/tasks/BACKLOG.md` and
`docs/status/index.html` resolved by regenerating from frontmatter:

```
python3 scripts/migrate_improvement_to_task.py --backlog
node tools/docs-site/render-status-hub.mjs . docs/status
```

Frontmatter on IMP-136/137, SKILL-202, and MEMORY-303 all read `status: done` after
the merge (no content conflict in the specs). Subsequent Batch D/E/F branches base
from this integrate tip so done-rows are not lost again.

## Do not push

Operator instruction: never push/merge/deploy without an explicit ask.
