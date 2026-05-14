# GDT XML schema v3.0 — element-by-element reference

> **Status**: working approximation of the GDT TT78/2014 + TT32/2011 e-invoice schema family. Suitable for development, prototyping, and integration with downstream systems that accept the rough shape. For production filing through a TVAN (transmission service provider), validate against the official `.xsd` from `gdt.gov.vn` in force at filing time — namespace URIs, optional/required cardinalities, and date formats are version-pinned to the current GDT release.

## Namespaces

| Prefix | URI |
|---|---|
| `inv` | `http://kekhaithue.gdt.gov.vn/TIN/2014/04/01/HoaDonDienTu` |
| `ds`  | `http://www.w3.org/2000/09/xmldsig#` (digital signature, not populated by this skill) |

The signature subtree is intentionally left out — TVAN providers sign the envelope themselves with the seller's certificate. Do not attempt to forge signatures here.

## Root element: `<Invoice>`

Required. Carries the namespace declarations as attributes. Contains exactly one each of `<Header>`, `<Seller>`, `<Buyer>`, `<Lines>`, `<Totals>`.

```xml
<Invoice xmlns:inv="..." xmlns:ds="...">
  <Header>...</Header>
  <Seller>...</Seller>
  <Buyer>...</Buyer>
  <Lines>...</Lines>
  <Totals>...</Totals>
</Invoice>
```

## `<Header>`

| Child | Type | Required | Example | Notes |
|---|---|---|---|---|
| `<InvoiceNo>` | string | yes | `INV-2026-001` | TVAN-assigned in production; freeform here. Max 20 chars per GDT recommendation. |
| `<InvoiceDate>` | ISO-8601 date | yes | `2026-05-14` | Date only, no time. Must be the date the invoice is *issued*, not the date of supply. |
| `<Currency>` | ISO 4217 code | yes | `VND` | `VND` is the default. Foreign-currency invoices must carry an exchange-rate annex (out of scope). |
| `<PaymentMethod>` | enum | yes | `TM` / `CK` / `TM/CK` | `TM` = tiền mặt (cash), `CK` = chuyển khoản (transfer). Combined `TM/CK` is accepted by GDT. |

## `<Seller>` and `<Buyer>`

Identical structure for the two parties.

| Child | Type | Required | Example | Notes |
|---|---|---|---|---|
| `<Name>` | string | yes | `Công ty TNHH ABC` | Legal name as registered with the business registry. Use UTF-8 NFC normalisation. |
| `<MST>` | string | yes | `0312345678` or `0312345678-001` | Vietnamese tax code. Validated by `vn-mst-validate`. |
| `<Address>` | string | optional but recommended | `123 Lê Lợi, P. Bến Nghé, Q.1, TP.HCM` | Full registered address. |

In production GDT XML, sellers and buyers also have `<BankAccount>`, `<Phone>`, `<Email>` subelements. They are accepted by most parsers but not required for structural validity, so this skill omits them by default. Add them downstream if your TVAN requires them.

## `<Lines>`

Wraps one or more `<Line>` children. Each `<Line>` carries an `index` attribute (1-based), used by GDT consumers to reference specific lines in correction notes.

### `<Line>`

| Child | Type | Required | Example | Notes |
|---|---|---|---|---|
| `<Name>` | string | yes | `Tư vấn phần mềm` | The goods/service description. Free-form. |
| `<Unit>` | string | optional | `giờ`, `cái`, `lần`, `kg` | Unit of measure. Use Vietnamese natural-language tokens. |
| `<Qty>` | decimal | yes | `80` | Quantity. Decimal-as-string to avoid float drift. |
| `<UnitPrice>` | integer VND | yes | `500000` | Unit price in integer VND (no fractions of dong). |
| `<Amount>` | integer VND | yes | `40000000` | `Qty × UnitPrice`, rounded half-up. |
| `<VatRate>` | percentage | yes | `10%` | One of `0%`, `5%`, `8%`, `10%`. See `vat-rates.md`. |
| `<VatAmount>` | integer VND | yes | `4000000` | `Amount × VatRate`, rounded half-up **per line**. |

#### Rounding rule (GDT requirement)

VAT is rounded **per line** to integer VND using ROUND_HALF_UP. Document-level totals are the *sum of pre-rounded line VATs*, not the rounded total VAT of the document. This produces a ±1 VND deviation versus "round at document level" in some edge cases — the per-line rule is the legally mandated one.

## `<Totals>`

| Child | Type | Required | Notes |
|---|---|---|---|
| `<TotalAmount>` | integer VND | yes | `sum(Line.Amount)` |
| `<TotalVat>` | integer VND | yes | `sum(Line.VatAmount)` (NOT `TotalAmount × rate`) |
| `<GrandTotal>` | integer VND | yes | `TotalAmount + TotalVat` |

The structural validator allows ±1 VND tolerance per total (and ±2 VND for `GrandTotal`) to accommodate the per-line rounding rule when validating externally-generated documents.

## Common pitfalls

- **Date format**: must be `YYYY-MM-DD`, NOT `DD/MM/YYYY` or `DD-MM-YYYY`. The Vietnamese-typed `14/05/2026` is rejected by the production schema.
- **MST whitespace**: trim before embedding. The validator rejects MSTs with embedded spaces.
- **VAT rate strings**: write `10%` not `0.10` or `10` — the GDT XML uses percentage-with-symbol.
- **Currency**: `VND` not `VNĐ`. ISO 4217 codes only.
- **Negative quantities**: not allowed on the primary invoice. Refunds use a separate "hoá đơn điều chỉnh" document type (out of scope for this skill).
- **Encoding**: UTF-8 with BOM is tolerated; UTF-8 without BOM is the canonical form. Latin-1 is rejected.

## Source documents

- Thông tư 78/2021/TT-BTC (e-invoice regulations from 2022)
- Thông tư 32/2011/TT-BTC (original e-invoice format)
- Nghị định 123/2020/NĐ-CP (general invoice regulations)
- Nghị định 119/2018/NĐ-CP (e-invoice mandate)

For the authoritative XSD, check `gdt.gov.vn` → "Hoá đơn điện tử" → "Tài liệu kỹ thuật" (publication cadence: typically once per major regulatory update).
