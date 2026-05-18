# VietQR / Napas247 — EMVCo TLV reference

VietQR adopts the **EMVCo QR Code Specification for Merchant-Presented Mode** (v1.1, 2020) and layers Napas-specific extensions on top. Every payload is a flat sequence of TLV triplets terminated by a CRC16 trailer.

## Encoding

- ASCII only at the protocol layer (digits + uppercase letters); UTF-8 in user-visible name/memo fields is tolerated by major Vietnamese bank scanners but the lengths are byte-counts and many scanners cap at 25 bytes per field.
- Tag: 2 ASCII digits.
- Length: 2 ASCII digits, zero-padded, decimal — describes the byte-length of the value.
- Value: N bytes per the length field.

## Top-level fields

| Tag | Length | Value | Meaning |
|-----|--------|-------|---------|
| 00  | 02     | `01`                       | Payload Format Indicator (always "01") |
| 01  | 02     | `11` (static) / `12` (dyn) | Point of Initiation Method |
| 38  | varies | nested TLV                 | Merchant Account Information (Napas) |
| 53  | 03     | `704`                      | Transaction Currency (ISO 4217 — VND) |
| 54  | varies | digits                     | Transaction Amount (optional) |
| 58  | 02     | `VN`                       | Country Code |
| 59  | varies | ASCII / UTF-8              | Merchant Name (optional, recommend ≤25) |
| 60  | varies | ASCII                      | Merchant City (optional) |
| 62  | varies | nested TLV                 | Additional Data (optional — memo lives here) |
| 63  | 04     | hex string                 | CRC16-CCITT-FALSE checksum |

## Nested under tag 38 (Napas merchant info)

| Sub-tag | Length | Value | Meaning |
|---------|--------|-------|---------|
| 00      | 10     | `A000000727`         | AID for Napas (always this value) |
| 01      | varies | nested TLV           | Beneficiary account information |
| 02      | 08     | `QRIBFTTA` / `QRIBFTTC` | Service code — `QRIBFTTA` = inter-bank account transfer; `QRIBFTTC` = card-based |

### Nested under sub-tag 38/01

| Sub-sub-tag | Length | Value | Meaning |
|-------------|--------|-------|---------|
| 00          | 06     | 6-digit bank BIN | Napas-assigned routing code (see `bank-bins.md`) |
| 01          | varies | account number   | 6-19 digits |

## Nested under tag 62 (additional data)

| Sub-tag | Length | Value | Meaning |
|---------|--------|-------|---------|
| 01      | varies | bill number       | Optional |
| 02      | varies | mobile number     | Optional |
| 05      | varies | reference label   | Optional |
| 07      | varies | terminal id       | Optional |
| 08      | varies | terminal label    | This is what most apps display as "memo" / "nội dung" |

## CRC16-CCITT-FALSE

- Polynomial: `0x1021`
- Initial register: `0xFFFF`
- No final XOR.
- Input: every byte from offset 0 through the literal `"6304"` tag+length pair (i.e. include them in the checksum input).
- Output: 4 uppercase ASCII hex digits.

## Edge cases & gotchas

- **Order matters** for top-level tags by EMVCo spec (ascending); most Vietnamese scanners are lenient but the canonical writer here emits them in spec-order.
- **Amount must be ASCII digits only** — no thousand separator, no decimal (VND has no sub-unit at the QR layer).
- **Account length** is enforced as 6-19 digits. Most Vietnamese personal accounts are 10-14 digits; corporate accounts up to 16.
- **Static vs dynamic** — point-of-init "11" means the QR can be scanned many times (the user enters the amount); "12" means single-use, amount pre-filled.
- **UTF-8 in name/memo** — protocol-allowed but length is byte-count, so a 25-char ASCII string fits but a 25-codepoint Vietnamese string (with diacritics) may exceed the 25-byte cap. Truncate by byte, not by codepoint, if you go this route. This skill currently truncates by Python `str` indexing (codepoint) for simplicity; pure-ASCII memos are recommended.

## References

- EMVCo Merchant-Presented Mode Specification v1.1 — `https://www.emvco.com/specifications/`
- Napas247 VietQR partner spec — distributed under NDA; format above is the public consensus reconstruction.
