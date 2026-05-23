---
name: nfr-evaluator
description: >-
  Evaluate raw non-functional test results against the NFR catalog. Use when
  user asks to "score NFR results", "compare SLOs", or "decide if the release
  passes non-functional gates". Outputs pass, degraded, or fail verdicts with
  evidence links for every requirement.
metadata:
  version: 1.0.0
  module: skill
allowed_memory_scopes:
  read:
    - project:*
allowed_mcp_tools:
  - audit.append
---

# nfr-evaluator

Compare test output to SLO thresholds and produce one verdict per NFR.
