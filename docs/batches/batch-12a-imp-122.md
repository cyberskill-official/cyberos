---
batch: batch/12a-imp-122
members:
  - TASK-IMP-122
  - TASK-IMP-013
  - TASK-IMP-046
  - TASK-IMP-047
  - TASK-IMP-002
recorded: 2026-07-25
actor: Stephen Cheng (session operator)
---

# batch/12a — IMP-122 rules_sha recompute + dispositions

## IMP-122

Spec repaired (NEW5-001..006; AC-10 fresh-install derivation; PASS 10/10 audit).
Implemented `lib/rules-cone.sh`, rewired build/version/update-check/audit-fleet,
reconciler both directions, suite `test_rules_sha_recompute.sh` **pass=15 fail=0**.

## Dispositions (ride this PR)

| ID | Action | Why |
|---|---|---|
| IMP-013 | closed | Cross-service contract tests are platform (`services/*`); not CyberOS 1.x payload |
| IMP-046 | closed | Backup/restore drill is platform/ops; won't-do for 1.x payload (batch-10e precedent) |
| IMP-047 | closed | Rebuild-in-60 runbook is VPS/platform SPOF insurance; won't-do for 1.x payload |
| IMP-002 | closed | CORS refuse-in-prod surface lives in `services/*` / `*_DEV_CORS` boot paths, not the payload |

IMP-001 reframed for PR-B (npm-audit + license gate).
