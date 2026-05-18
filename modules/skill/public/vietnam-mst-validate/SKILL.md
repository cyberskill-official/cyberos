---
name: vietnam-mst-validate
description: >-
  Validate Vietnamese tax codes (Mã số thuế / MST). Returns whether a given
  string is a structurally valid MST — 10 digits for a legal entity or
  10 digits + '-' + 3 digits for a branch / dependent unit, per General
  Department of Taxation regulations. Use when the user provides a
  Vietnamese tax code (MST), an invoice with `mst_nguoi_ban` /
  `mst_nguoi_mua` fields, or asks "is this a valid Vietnamese MST?". Do
  NOT use for non-Vietnamese tax IDs — see other locale skills.
license: Apache-2.0
compatibility: >-
  Fully offline. No network access. Python 3.11+ for scripts/validate_mst.py.
metadata:
  author: cyberskill
  version: "0.1.0"
  region: VN
  collection: cyberskill-vn
---

# Vietnamese MST Validator

## When to use

- User provides a Vietnamese tax code and asks for validation.
- Pre-validation step before generating a VAT invoice (`vietnam-vat-invoice` skill calls this).
- KYC / vendor-onboarding workflows that need a structural MST check.

## What it does

Given a string, returns:

```json
{ "ok": true,  "kind": "entity" }     // 10-digit MST
{ "ok": true,  "kind": "branch" }     // 10-digit + "-NNN" branch MST
{ "ok": false, "reason": "MST must be 10 digits, optionally followed by '-NNN'" }
```

This is **structural validation only.** It does NOT confirm the MST exists in the GDT public registry — that requires a live lookup, which is out of scope for an offline skill. For registry lookup, see `references/format.md` § "Live verification".

## Quick start

```bash
echo '0123456789' | python scripts/validate_mst.py
# → {"ok": true, "kind": "entity"}

echo '0123456789-001' | python scripts/validate_mst.py
# → {"ok": true, "kind": "branch"}

echo '12345' | python scripts/validate_mst.py
# → {"ok": false, "reason": "MST must be 10 digits, optionally followed by '-NNN'"}
```

## Structure

- `scripts/validate_mst.py` — the validator entry point. Reads stdin, writes JSON to stdout. Exit code 0 if valid, 1 if invalid.
- `references/format.md` — detailed format reference (entity vs branch, common gotchas, live-registry pointer).
- `tests/fixtures.json` — test corpus covering valid + invalid inputs.

## Examples

See `tests/fixtures.json` for the full corpus. Sample:

| Input | Valid? | Reason |
|---|---|---|
| `0312345678` | yes | entity |
| `0312345678-001` | yes | branch |
| `0312-345-678` | no | hyphens in wrong position |
| `1234567890123` | no | 13 consecutive digits — must have `-NNN` suffix |
| `031234567X` | no | non-digit character |
| (empty) | no | empty string |
