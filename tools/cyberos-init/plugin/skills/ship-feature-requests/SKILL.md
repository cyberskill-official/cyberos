---
name: ship-feature-requests
description: "Run the CyberOS ship-feature-requests workflow in any repo: drive a feature-request (product or improvement class) through implement -> review -> test -> done, with HITL required at the two human-acceptance gates and gates from the repo's own build/lint/test. Use when asked to ship, implement, or harden an FR, or to drive a docs/feature-requests backlog."
---
# ship-feature-requests (portable)

The full, normative workflow is in `cuo/ship-feature-requests.md` (bundled beside this file), with `cuo/EXECUTION-DISCIPLINE.md` (the halt and HITL doctrine) and `cuo/STATUS-REFERENCE.md` (the status lifecycle). Read those; they are the source of truth.

In one paragraph: pick the first eligible FR in `docs/feature-requests/BACKLOG.md` (`ready_to_implement`, dependencies done). Deep-map the repo, write the edge-case matrix, implement with observability and at least 90% coverage on touched files, review the diff against every section-1 clause, and run the gates (`.cyberos/cuo/gates/run-gates.sh` = the repo's own build/lint/test + coverage; caf and awh only if present). HITL is required: halt at review acceptance and at final acceptance for a recorded human verdict, and never set `done` yourself. On any gate failure, route the FR back to `ready_to_implement`. Improvement and hardening work is the same workflow with `class: improvement`.
