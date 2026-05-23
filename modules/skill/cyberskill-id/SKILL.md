---
name: cyberskill-id
description: >-
  Provide Indonesia operations helpers for NPWP normalization and e-Faktur XML
  payload generation. Use when user asks to "validate Indonesia NPWP", "prepare
  e-Faktur", or "clean Indonesian tax data". Outputs normalized identifiers and
  escaped invoice XML payloads.
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

# cyberskill-id

Normalize NPWP digits and emit a minimal escaped e-Faktur XML payload.
