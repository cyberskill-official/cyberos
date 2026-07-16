# TASK-IMP-090 edge-case matrix

| # | Category | Trigger | Expected | Covered by |
|---|---|---|---|---|
| 1 | NULL/EMPTY | `.workflow/.gitignore` absent on a fresh repo | seed created carrying both patterns | t07_workflow_gitignore_patterns (fresh path) |
| 2 | IDEMPOTENCE | install re-run twice on the same repo | manifest pattern appears exactly once | t07_workflow_gitignore_patterns (append-once path) |
| 3 | LEGACY STATE | existing seed carries only `*.ship.json` (every repo installed before today) | manifest pattern appended, ship pattern untouched | t07_workflow_gitignore_patterns |
| 4 | OPERATOR EDIT | seed carries extra operator lines | append-once adds one line, touches nothing else | t07_workflow_gitignore_patterns (asserts full file content) |
| 5 | MALFORMED | seed lacks a trailing newline | newline healed first, pattern lands on its own line | install.sh:52 `tail -c 1` guard; t07 exercises append |
| 6 | SECURITY | .gitignore controls what leaves the index, not what is readable | no secret surface; manifests hold ids/titles/statuses only | reviewed - no untrusted content, no execution |
| 7 | INDEX | the three manifests already tracked | `git rm --cached` removes from index, files stay on disk untracked | AC 3 (`git ls-files docs/tasks/.workflow` = only tracked artefacts) |
| 8 | RECORD LOSS | untracking removes the only PLAN-approval trail | `_audits/IMPROVEMENT-BATCHES-2026-07-16.md` carries members, verdicts, commits | AC 4 (recorded greps) |
| 9 | CONCURRENCY | 088 edits install.sh in the same round | same agent, serial order (§11a view + cone rule) | batch plan; both suites green after both edits |
| 10 | DEGRADATION | future author run writes to the old default | callers may still override manifest_path explicitly; only the default moved | SKILL.md line 184 keeps `<from caller; default: ...>` |
