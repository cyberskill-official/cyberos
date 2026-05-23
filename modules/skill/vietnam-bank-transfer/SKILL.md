---
name: vietnam-bank-transfer
description: >-
  Generate Vietnamese bank-transfer payloads for VietQR and Napas247 payment
  instructions. Use when user asks to "make a VietQR code", "prepare a bank
  transfer", or "generate payment details". Outputs a deterministic payment
  payload with bank BIN, account, amount, and memo.
metadata:
  version: 1.0.0
  module: skill
allowed_memory_scopes:
  read:
    - company:*
allowed_mcp_tools:
  - audit.append
---

# vietnam-bank-transfer

Validate bank prefix and create a payment payload. Rendering the actual QR image
is delegated to the caller.
