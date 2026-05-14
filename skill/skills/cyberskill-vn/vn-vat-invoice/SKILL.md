---
name: vn-vat-invoice
description: >-
  Generate Vietnamese VAT-compliant electronic invoices (hoá đơn GTGT điện tử)
  from a structured JSON line-item list. Produces XML in the General Department
  of Taxation schema v3.0 (Mẫu hoá đơn điện tử). Use when the user provides
  line items + buyer MST + seller MST and asks for a Vietnamese VAT invoice,
  e-invoice, hoá đơn GTGT, hoá đơn điện tử. Do NOT use for non-Vietnamese
  invoice formats — use locale-specific skills instead.
license: Apache-2.0
compatibility: >-
  Fully offline. No network access required. Python 3.11+ for the bundled
  generator and validator scripts. Depends on the vn-mst-validate skill being
  available in the same skill root.
metadata:
  author: cyberskill
  version: "0.1.0"
  region: VN
  collection: cyberskill-vn
  depends_on: vn-mst-validate
allowed-tools: read_file write_file
---

# Vietnamese VAT Invoice (Hoá đơn GTGT điện tử)

## When to use

- User provides line items, seller MST, buyer MST and asks for a Vietnamese VAT invoice.
- User says "tạo hoá đơn", "xuất hoá đơn GTGT", "e-invoice Vietnam", "hoá đơn điện tử".
- Migration from a paper / Excel invoice format to the GDT XML schema.

## Procedure

1. **Validate both MSTs** using the `vn-mst-validate` skill. Both seller and buyer MST must structurally pass.
2. **For each line item**, compute:
   - `thanh_tien = so_luong * don_gia` (subtotal, round half-up to integer VND)
   - `tien_thue = thanh_tien * thue_suat` (VAT, round half-up to integer VND **per line**, not at document level — this is a GDT requirement)
3. **Compute totals**:
   - `tong_tien_hang = sum(thanh_tien)` across all lines
   - `tong_tien_thue = sum(tien_thue)` across all lines
   - `tong_thanh_toan = tong_tien_hang + tong_tien_thue`
4. **Generate the XML** per the GDT schema v3.0 — see `references/gdt-xml-schema-v3.md` for the element-by-element walk.
5. **Write the XML** to the path passed via the `--out` argument, or stdout if absent.

## Quick start

```bash
cat > /tmp/invoice.json <<'EOF'
{
  "seller": {"name": "Công ty TNHH ABC", "mst": "0312345678", "address": "..."},
  "buyer":  {"name": "Công ty XYZ",       "mst": "0107654321", "address": "..."},
  "invoice_no": "INV-2026-001",
  "invoice_date": "2026-05-14",
  "payment_method": "TM",
  "currency": "VND",
  "lines": [
    {"name": "Tư vấn phần mềm", "unit": "giờ", "qty": 80,   "unit_price": 500000, "vat_rate": 0.10},
    {"name": "Triển khai hệ thống", "unit": "lần", "qty": 1, "unit_price": 50000000, "vat_rate": 0.10}
  ]
}
EOF

python scripts/generate_invoice.py < /tmp/invoice.json > /tmp/invoice.xml
python scripts/validate_invoice.py < /tmp/invoice.xml
# → {"ok": true, ...}
```

## Structure

- `scripts/generate_invoice.py` — reads JSON from stdin, writes GDT XML to stdout (or `--out <path>`).
- `scripts/validate_invoice.py` — reads XML from stdin, validates against the schema sketch, reports structural issues.
- `references/gdt-xml-schema-v3.md` — element-by-element schema reference (this is the *first* file to read when debugging).
- `references/vat-rates.md` — current Vietnamese VAT rates (0% / 5% / 8% / 10%) and when each applies.
- `references/example-2line-invoice.xml` — worked two-line example.
- `assets/template.xml` — XML template the generator fills in.

## VAT rates (as of 2026)

| Rate | Goods/services |
|---|---|
| 0% | Exports; international transport |
| 5% | Clean water, fertilizers, medical equipment, basic agricultural products |
| 8% | Temporary reduction (Decree 15/2022 + extensions; valid through 2026 per current law) |
| 10% | Standard rate — most goods and services |

See `references/vat-rates.md` for the full taxonomy.

## Production caveat

The bundled schema sketch is an **approximation of the GDT TT78/2014 + TT32/2011 schema**, suitable for development, prototyping, and integration with downstream systems that accept the rough shape. **Before submitting to a GDT-authorised invoice transmission provider (TVAN), validate against the official `.xsd` from gdt.gov.vn** — schema details (namespace URIs, optional vs required fields, specific date formats) are version-pinned to the GDT release in force at filing time. This skill is structurally correct but not production-certified.
