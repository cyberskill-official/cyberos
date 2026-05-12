# `tours/` — guided walkthroughs

`.tour` files are step-by-step walkthroughs for common CyberOS workflows. Open them with the **CodeTour** VS Code extension or read as plain JSON. Each tour points at specific files + line numbers + commands to run in order.

| Tour | When to use it |
| --- | --- |
| [`onboarding.tour`](onboarding.tour) | First-time operator setup: install, init the BRAIN, write first memory. |
| [`incident-response.tour`](incident-response.tour) | Production incident playbook: capture, diagnose, recover, postmortem memory. |
| [`protocol-upgrade.tour`](protocol-upgrade.tour) | Upgrading the AGENTS protocol: §0.5 procedure + canonical-SHA pin update. |
| [`refinement-loop.tour`](refinement-loop.tour) | Acting on `cyberos refinement dashboard` candidates. |
| [`security-audit.tour`](security-audit.tour) | Pre-audit cluster security review (Aspect 13.5). |
| [`repair-audit-chain.tour`](repair-audit-chain.tour) | Recover from corrupt audit ledger. |
| [`repair-fix-frontmatter.tour`](repair-fix-frontmatter.tour) | Fix memories with invalid §5.1 frontmatter. |
| [`repair-manual-rollback.tour`](repair-manual-rollback.tour) | Manual rollback when `cyberos rollback` can't auto-recover. |
| [`repair-stuck-conflict.tour`](repair-stuck-conflict.tour) | Resolve a stuck sync conflict. |
| [`repair-tombstone-orphan.tour`](repair-tombstone-orphan.tour) | Clean up orphaned tombstones from `cyberos prune`. |

## How tours work

Each `.tour` file is JSON describing N steps:

```json
{
  "title": "onboarding",
  "steps": [
    { "file": "runtime/tools/cyberos", "line": 1, "description": "Start here…" },
    { "directory": "docs/memory/", "description": "Read AGENTS.md before proceeding." },
    ...
  ]
}
```

Install the **CodeTour** VS Code extension, open this folder, and tours appear in the activity-bar list. Or read them as plain JSON in your editor.

## Adding a new tour

1. Pick a workflow that takes >3 steps and is run rarely (so its steps blur over time).
2. Write a tour pointing at the relevant files + commands.
3. Add a row to the table above so future-you can find it.
