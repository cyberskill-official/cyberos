---
name: vietnam-mst-validate
description: >-
  Validate Vietnamese tax identifiers (MST) for accounts, invoices, and vendor
  records. Use when user asks to "validate this MST", "check Vietnamese tax ID",
  or "clean company tax code". Outputs normalized digits, shape validation, and
  a provider lookup request when online verification is available.
metadata:
  version: 1.0.0
  module: skill
allowed_memory_scopes:
  read:
    - company:*
allowed_mcp_tools:
  - audit.append
---

# vietnam-mst-validate

Normalize separators, validate 10-digit and 13-digit MST forms, and prepare
GDT lookup metadata.
