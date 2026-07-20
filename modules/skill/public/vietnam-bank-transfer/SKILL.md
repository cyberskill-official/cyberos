---
name: vietnam-bank-transfer
description: >-
  Generate VietQR / Napas247 QR-code payload strings for Vietnamese bank transfers, parse incoming VietQR strings, and look up Napas bank BIN codes. Use when the user needs a payable QR code for a Vietnamese bank account, wants to decode a VietQR they received, or asks about bank routing codes (BIN). Do NOT use for international SWIFT transfers — VietQR is Napas247-domestic only. Use when user asks to "reference vietnam bank transfer" or "look up vietnam bank transfer".
license: Apache-2.0
compatibility: >-
  Fully offline. No network access required. Python 3.11+ for the
  bundled scripts. QR image rendering is out of scope — the script
  emits the payload string; pipe it to `qrencode`, `segno`, or any
  QR renderer of your choice.
metadata:
  author: cyberskill
  version: "0.1.0"
  region: VN
  collection: cyberskill-vn
allowed-tools: read_file write_file
---

# Vietnamese Bank Transfer (VietQR / Napas247)

## When to use

- User asks for a QR code that, when scanned by any Vietnamese bank app, prefills a transfer.
- User says "tạo mã QR chuyển khoản", "VietQR", "Napas247", "QR thanh toán".
- User has a VietQR payload string and wants to decode it.
- User asks for a bank's BIN code.

## Procedure

1. **Resolve the bank** by short code (e.g. "VCB", "BIDV", "ACB") to BIN via `assets/bank-bins.json`.
2. **Validate the account number** is digits only, length 6-19.
3. **Build the EMVCo TLV string** per `references/vietqr-format.md`:
- Payload format "01", point-of-init "11" (static) or "12" (dynamic).
- Merchant account info with Napas AID + bank BIN + account.
- Currency "704" (VND); optional amount; country "VN".
- Optional memo/terminal label under tag 62.
- Compute CRC16-CCITT-FALSE of everything preceding `6304`.
4. **Emit the payload string** — pipe to any QR renderer for the image.

## Quick start

```bash
cat > /tmp/transfer.json <<'EOF'
{
  "bank": "VCB",
  "account": "0123456789",
  "recipient": "NGUYEN VAN A",
  "amount": 250000,
  "memo": "Thanh toan hoa don T5/2026",
  "dynamic": true
}
EOF

python scripts/generate_qr.py < /tmp/transfer.json
# → 00020101021238570010A0000007270127...6304ABCD
```

## VietQR format (EMVCo Merchant-Presented Mode)

See `references/vietqr-format.md` for the full TLV walk. The payload is a sequence of `TAG(2) + LENGTH(2) + VALUE(N)` triplets, terminated by a 4-character CRC16-CCITT-FALSE checksum tagged `6304`. The checksum covers EVERY byte from the start through `6304` inclusive.

## Bank BIN codes

20+ Vietnamese banks bundled in `assets/bank-bins.json`. See `references/bank-bins.md` for the full table.

## Status

Production-ready for VietQR static + dynamic payloads. Validated against the EMVCo Merchant-Presented Mode specification (v1.1). Test fixtures include round-trip generate→parse over 5 banks + 3 amount cases.
