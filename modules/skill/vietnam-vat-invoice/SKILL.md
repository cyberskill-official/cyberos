---
name: vietnam-vat-invoice
description: >-
  Emit Vietnamese VAT e-invoice XML for Decree 123 workflows. Use when user asks
  to "create a VAT invoice", "emit hoa don XML", or "prepare GDT invoice data".
  Outputs escaped XML fields and submission metadata for the invoicing service.
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

# vietnam-vat-invoice

Build a minimal e-invoice XML payload with escaped tax identifiers, invoice number, and totals.
