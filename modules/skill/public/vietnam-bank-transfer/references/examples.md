# Worked examples

## Example 1 — static VCB transfer, no amount, no memo

Use-case: a personal account holder posts a QR on their business card or coffee-shop counter; customers scan and enter their own amount.

### Input

```json
{
  "bank": "VCB",
  "account": "0123456789"
}
```

### Generated payload

```
00020101021138540010A00000072701240006970436011001234567890208QRIBFTTA53037045802VN6304XXXX
```

Walk:

- `0002 01` — payload format "01"
- `0102 11` — static
- `38 54 ...` — merchant account info, length 54
- `0010 A000000727` — Napas AID
- `0124 ...` — beneficiary block, length 24
- `0006 970436` — VCB BIN
- `0110 0123456789` — account
- `0208 QRIBFTTA` — service code
- `5303 704` — VND
- `5802 VN` — country
- `6304 XXXX` — CRC16

## Example 2 — dynamic BIDV invoice, 250,000 VND, with memo

Use-case: an SME issues an invoice that's auto-paid by scanning; the customer's bank app pre-fills both the amount and the reference.

### Input

```json
{
  "bank": "BIDV",
  "account": "31410001234567",
  "amount": 250000,
  "memo": "Thanh toan HD ABC",
  "dynamic": true
}
```

### Notes

- Point-of-init switches to `12` (single-use, amount pre-filled).
- Tag `54` carries the amount: `5406 250000`.
- Tag `62` wraps the memo under sub-tag `08`:
- `62 21 08 17 Thanh toan HD ABC`

The customer's bank app shows: "Chuyển 250,000₫ đến STK 31410001234567 (BIDV) — ND: Thanh toan HD ABC".

## Example 3 — ACB merchant POS dynamic QR with recipient name + city + memo

Use-case: a café in Hà Nội prints a per-transaction QR for an iced coffee.

### Input

```json
{
  "bank": "ACB",
  "account": "1234567890",
  "recipient": "CAFE ABC",
  "amount": 45000,
  "memo": "Ban so 5 - Ca phe sua da",
  "dynamic": true
}
```

### Result

- Tag `59 08 CAFE ABC` — merchant name (8 chars).
- Tag `54 05 45000` — amount.
- Tag `62 ...` — memo bundle.

Bank apps that support QR-prefilled flows will show "CAFE ABC — 45,000₫ — Ban so 5 - Ca phe sua da" before the user confirms with their bank-issued PIN/biometric.

## Round-tripping

Every payload produced by `generate_qr.py` round-trips losslessly through `parse_qr.py`:

```bash
echo '{"bank":"VCB","account":"0123456789","amount":250000,"memo":"test","dynamic":true}' \
    | python scripts/generate_qr.py \
    | python scripts/parse_qr.py
# → {"crc_ok": true, "payload_format": "01", "dynamic": true,
#    "napas_aid": "A000000727", "bank_bin": "970436", "account": "0123456789",
#    "service_code": "QRIBFTTA", "currency": "704", "amount": 250000,
#    "country": "VN", "memo": "test", "bank": "VCB"}
```
