# `repo-context-map-author` - failure modes

1. Pattern cited from memory instead of disk - pinned_in resolution kills it at audit.
2. Blast radius understated by skipping cross-module greps - audit re-derives edge count from the map's own file list.
3. Placement warning raised then ignored downstream - RCM-GATE-002 requires escalation evidence.
4. Stale map reused across FRs - task_id binding in frontmatter; audit checks it matches the invoking FR.
5. Repo too large to scan fully - declared scan scope with explicit exclusions beats silent partial scan.
