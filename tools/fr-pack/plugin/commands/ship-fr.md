---
description: Drive the next eligible FR in docs/feature-requests/BACKLOG.md through the ship-feature-requests lifecycle (HITL required).
argument-hint: "[repo path, default: current repo]"
---
Follow the workflow in the bundled `skills/ship-feature-requests/machine/ship-feature-requests.md` (with `EXECUTION-DISCIPLINE.md` and `STATUS-REFERENCE.md` beside it). Those are the source of truth.

Drive the next eligible FR in `docs/feature-requests/BACKLOG.md` for repo_root = ${1:-the current repo}:

- Pick the first FR whose status is `ready_to_implement` with all `depends_on` done.
- Author and audit each phase per the workflow (context map, edge-case matrix, implementation, observability, review, tests).
- Run the machine gates: `bash .cyberos/fr-pack/gates/run-gates.sh` (or the repo's own build/lint/test if the pack is not installed).
- HITL is REQUIRED: halt at review acceptance (`reviewing -> ready_to_test`) and at final acceptance (`testing -> done`). Never set `done` yourself; a human records each verdict.
- On any gate failure, route the FR back to `ready_to_implement` with a reason.

If `.cyberos/fr-pack/` is missing, run `/fr-init` first.
