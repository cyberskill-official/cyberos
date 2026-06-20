# Retrospective template — score every run

Copy this file to `core/improve/retros/<date>-<project>.md` after each run of AUDIT.md
(on a client project, an internal repo, or this repo itself). Score each item
**0 (bad) / 1 (partial) / 2 (good)**. Target: keep total **>= 16/20**.

A retrospective takes ~10 minutes and is the raw material for every protocol
change. No retro → no edit.

```markdown
# Retrospective — Project: <name> | Protocol: v<x.y.z> | Date: <ISO> | Mode: <gated|autonomous>

1. Did EVERY metric include a real command + raw output (or honest UNMEASURED)?   __
2. Were all targets either cited-with-URL or labeled INTERNAL (no fake SOTA)?     __
3. Did it AVOID inventing findings to hit a number (no padding)?                  __
4. Did the loop stop for a real reason (budget / 2 empty loops / all closed)?     __
5. Did the 3-strike circuit breaker behave correctly on any failures?             __
6. Was BACKLOG.md/HANDOFF.md correctly formatted and resumable?                   __
7. Were PROTECTED_AREAS and the public API left behavior-preserving?              __
8. Were commits atomic, descriptive, and temp scripts cleaned up?                 __
9. Were the actual code changes sensible and genuinely valuable?                  __
10. Did it stay on scope (no unrequested surface-area expansion)?                 __

TOTAL: __/20
Lowest-scoring item(s): ____
Candidate ONE-LINE prompt edit (if any): ____
Evidence (paths to BACKLOG/HANDOFF, eval results, transcript excerpts): ____
```

## How scores drive changes

- **Total >= 16/20 and no repeat failure** → no edit. A stable prompt is the goal; stop tuning.
- **An item scores 0, or the same failure appears in 2+ retros** (Rule of Three: codify only after a pattern recurs) → log it in `core/improve/FAILURE_LOG.md` and propose ONE minimal wording change via `core/improve/CRITIC.md`.
- **Total < 16/20** → run a critic cycle now.

Machine-checkable items (1, 2, 4, 6, and the redaction half of 8) can be scored
automatically: run `python3 core/evals/validate.py --run <path-to-docs-dir>` against
the run's `docs/` output and map violations to the matching question.
