---
name: nfr-test-runner
description: >-
  Run non-functional requirement verification harnesses against a deployment
  environment. Use when user asks to "run NFR tests", "certify staging", or
  "check production SLOs". Outputs raw test results from load, security,
  accessibility, and telemetry checks for evaluator review.
metadata:
  version: 1.0.0
  module: skill
allowed_memory_scopes:
  read:
    - project:*
allowed_mcp_tools:
  - audit.append
  - obs.query
---

# nfr-test-runner

Read NFR verification stanzas, run or collect the declared checks, and emit a raw result bundle keyed by NFR id.
