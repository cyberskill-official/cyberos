# Vietnamese MST — format reference

## Legal entity (10 digits)

Pattern: `^\d{10}$`

Issued by the General Department of Taxation (Tổng cục Thuế / GDT) when a business registers. Single MST per legal entity nationally. Examples:

- `0312345678` (typical 10-digit MST)
- `0107654321`

The first two digits historically encoded the issuing province but this is no longer reliable post-centralisation; treat all 10-digit MSTs as opaque identifiers.

## Branch / dependent unit (10 + '-' + 3 digits)

Pattern: `^\d{10}-\d{3}$`

The first 10 digits are the parent entity's MST; the suffix `-NNN` indexes the branch (chi nhánh) or dependent unit (đơn vị phụ thuộc). Examples:

- `0312345678-001` (first branch of the parent above)
- `0312345678-042` (42nd branch)

## Common mistakes

- **Spaces instead of hyphen**: `0312345678 001` — INVALID. Must use `-`.
- **Hyphens in wrong position**: `0312-345-678` — INVALID. Hyphen only separates the optional 3-digit suffix.
- **Letters or special chars**: `031234567X` — INVALID.
- **Padded with zeros to 13**: `1234567890123` — INVALID (would require `-NNN` suffix).
- **Personal ID number (CMND/CCCD)** confused with MST: NOT the same. CCCD is 12 digits and unrelated.

## Live verification (out of scope for this skill)

To confirm an MST is currently registered with GDT, query:

- Public GDT search: `https://tracuunnt.gdt.gov.vn/tcnnt/mstdn.jsp` (HTML scrape, manual)
- VPBank / VietinBank / other commercial-bank APIs that wrap the GDT lookup

These are network-dependent. The `vn-mst-validate` skill performs **structural validation only**.

## Source

- Nghị định 126/2020/NĐ-CP (mã số thuế cấp cho người nộp thuế)
- Thông tư 105/2020/TT-BTC (hướng dẫn về đăng ký thuế)
