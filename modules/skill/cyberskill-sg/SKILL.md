---
name: cyberskill-sg
description: >-
  Provide Singapore operations helpers for ACRA UEN checks, GST invoice
  references, and CPF contribution estimates. Use when user asks to "validate
  Singapore company ID", "prepare GST invoice metadata", or "estimate CPF".
  Outputs normalized identifiers and deterministic calculation payloads.
metadata:
  version: 1.0.0
  module: skill
allowed_memory_scopes:
  read:
    - company:*
    - client:*
allowed_mcp_tools:
  - audit.append
---

# cyberskill-sg

Validate UEN-shaped identifiers, prepare GST invoice references, and estimate CPF amounts from configured rates.
