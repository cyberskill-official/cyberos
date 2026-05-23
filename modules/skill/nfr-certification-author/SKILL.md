---
name: nfr-certification-author
description: >-
  Author a non-functional certification report from evaluated NFR verdicts.
  Use when user asks to "write the NFR certificate", "summarize release
  readiness", or "prepare a go no-go report". Outputs a release certification
  with passed, degraded, failed, and waived requirements.
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

# nfr-certification-author

Render the evaluated NFR verdicts into a concise release certification report.
