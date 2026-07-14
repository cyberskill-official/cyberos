---
name: nfr-regression-handler
description: >-
  Draft bug-fix tasks for degraded or failed non-functional
  requirements. Use when user asks to "turn NFR failures into backlog work",
  "handle SLO regressions", or "prepare remediation FRs". Outputs ready-to-review
  regression FR drafts with reproduction evidence and owner routing.
metadata:
  version: 1.0.0
  module: skill
allowed_memory_scopes:
  read:
    - project:*
  write:
    - project:*
allowed_mcp_tools:
  - audit.append
---

# nfr-regression-handler

Create remediation FR drafts for every failed or degraded NFR verdict.
