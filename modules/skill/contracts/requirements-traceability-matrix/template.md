---
template: requirements-traceability-matrix@1
title: <project> — Requirements Traceability Matrix
project: <project name>
rtm_version: 1.0.0
generated_at: 2026-MM-DDTHH:MM:SS+07:00
source_set:
  - { path: ./srs.md,         hash: sha256:<hash> }
  - { path: ./prd.md,         hash: sha256:<hash> }
  - { path: ./feature-requests/, hash: sha256:<hash-of-dir-canon> }
provenance: { source_path: <canonical concat path>, source_hash: sha256:<combined-hash> }
# release: 2026-Q3.1    # optional — ties matrix to a release tag
---

# <project> — Requirements Traceability Matrix

## 1. Summary
- Total REQs: <N>
- Traced REQs: <N> (<%>)
- Untested REQs: <N>
- Orphan code (no REQ linkage): <N>

## 2. Matrix
| REQ-ID | Description | Source | Priority | Linked Design | Linked Code/PR | Linked Test | Status | Release |
|---|---|---|---|---|---|---|---|---|

## 3. Orphans
| REQ-ID | Missing linkage | Suggested next step |
|---|---|---|

## 4. Untested
| REQ-ID | Priority | Status | Notes |
|---|---|---|---|

## 5. Untraceable Code
| PR / commit | Author | Reason no REQ-ID |
|---|---|---|

## 6. Coverage Stats
| dimension | value |
|---|---|
| per-source-doc coverage | ... |
| per-priority traced %   | P0:..%, P1:..%, P2:..%, P3:..% |
| per-status counts       | drafted/designed/in_dev/in_test/shipped/deferred |

<!-- ## 7. Release-Scope Filter      — when release set -->
<!-- ## 8. Regulatory Mapping        — when project is regulated -->
