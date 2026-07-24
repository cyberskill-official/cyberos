---
batch: batch/9c-app
members:
  - TASK-APP-001
started: 2026-07-24T14:15:00Z
ended: null
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# batch/9c-app — code review (session HITL override)

Reviewed: 2026-07-24 on branch `batch/9c-app` against `main` @ `77ba6adf`.

## Scope

Resume/adopt TASK-APP-001 against as-built `services/memory/desktop` Ops. Code pre-existed; this PR closes process drift (COND-004 / Check-command wording) and adds residual guard unit tests.

## Diff review

| Area | Verdict |
|------|---------|
| Spec task@1 + authorship | Pass |
| Paths `services/memory/desktop/` (not `apps/desktop`) | Pass |
| Check = `version.sh` | Pass |
| Guard unit tests | Pass |
| Out of scope honest | Pass |

## Operator notes

1. Manual Mac GUI smoke remains the human verification for Build/Check/Install UX (not CI).
2. Merge independently of #140 (9b) is OK — APP-001 does not depend on OBS status flips.
