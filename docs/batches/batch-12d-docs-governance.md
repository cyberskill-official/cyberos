---
batch: batch/12d-docs-governance
members:
  - TASK-IMP-059
  - TASK-IMP-055
  - TASK-IMP-058
  - TASK-IMP-062
  - TASK-IMP-056
  - TASK-IMP-041
recorded: 2026-07-25
actor: Stephen Cheng (session operator)
---

# batch/12d — Docs/governance (+ secrets inventory)

## PR-D

| ID | Deliverable |
|---|---|
| IMP-059 | `wiki-link-check.mjs` + allowlist + `test_wiki_link_check.sh` (6/6) + build vendor |
| IMP-055 | `modules/manifest.yaml` + `docs/modules/MANIFEST.md` |
| IMP-058 | ADR-001..005 + `docs/adrs/README.md` + `docs/adr/` alias |
| IMP-062 | `docs/runbooks/quarterly-envelope-review.md` + BACKLOG pointer |
| IMP-056 | `docs/governance/api-versioning.md` |

## PR-E (rides this branch)

| ID | Deliverable |
|---|---|
| IMP-041 | `docs/runbooks/secrets-inventory-and-rotation.md` (classes only; no values) |

## Tests

```
bash tools/install/tests/test_wiki_link_check.sh
# result pass=6 fail=0
```

## Decision pauses

None — payload-scoped docs/governance; clear fork from platform won't-do.
