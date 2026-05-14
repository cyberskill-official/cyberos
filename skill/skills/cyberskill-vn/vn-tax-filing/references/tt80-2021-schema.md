# Mẫu 01/GTGT — element-by-element schema reference

> **Status**: working approximation of the Mẫu 01/GTGT VAT-return form defined in Appendix II of Thông tư 80/2021/TT-BTC (in force from 1 January 2022). Suitable for development, prototyping, and integration with downstream systems that accept the rough shape. For production filing through a TVAN (transmission service provider), validate against the official `.xsd` from `gdt.gov.vn` in force at filing time — element names, namespace URIs, and required/optional cardinalities are version-pinned to the current GDT release.

## Namespaces

| Prefix | URI |
|---|---|
| `ret` | `http://kekhaithue.gdt.gov.vn/TIN/2021/Mau01GTGT` |
| `ds`  | `http://www.w3.org/2000/09/xmldsig#` (digital signature, not populated by this skill) |

The signature subtree is intentionally left out — TVAN providers sign the envelope themselves with the taxpayer's certificate.

## Root element: `<TKhaiThue>`

Attributes:

| Attribute | Required | Value | Notes |
|---|---|---|---|
| `mauTKhai` | yes | `01/GTGT` | Form code. Fixed for this return type. |
| `kyKKhai`  | yes | `thang` or `quy` | Period type: monthly or quarterly. |
| `xmlns:ret`, `xmlns:ds` | yes | URIs above | Namespace decls. |

Children: exactly one `<TTinChung>`, `<ThueDauRa>`, `<ThueDauVao>`, `<ThuePhaiNop>` in that order.

## `<TTinChung>` — Header / general info

| Child | Type | Required | Example | Notes |
|---|---|---|---|---|
| `<kyKKhai>` | string | yes | `2026-05` or `2026-Q2` | Period being declared. Monthly = `YYYY-MM`. Quarterly = `YYYY-QN` where `N ∈ {1,2,3,4}`. |
| `<ngayLap>` | ISO-8601 date | yes | `2026-06-15` | Date the return is prepared / submitted. |
| `<NNT>` | element | yes | — | Taxpayer (Người nộp thuế) block. |

### `<NNT>` — Taxpayer

| Child | Type | Required | Example | Notes |
|---|---|---|---|---|
| `<tenNNT>` | string | yes | `Công ty TNHH ABC` | Legal name as registered. UTF-8 NFC. |
| `<mst>`    | string | yes | `0312345678` or `0312345678-001` | Vietnamese tax code. Validated by `vn-mst-validate`. |
| `<diaChi>` | string | yes (may be empty) | `123 Lê Lợi, P. Bến Nghé, Q.1, TP.HCM` | Registered address. |

## `<ThueDauRa>` — Section A — Output VAT

Aggregates the period's taxable sales by VAT rate. The form supplies four rate rows. Rows MAY be omitted if a rate had zero activity, but the totals MUST still balance.

| Tag | Rate | Notes |
|---|---|---|
| `<A1_chiu_thue_0>`   | 0%   | Exports, international transport, etc. |
| `<A2_chiu_thue_5>`   | 5%   | Clean water, fertilizers, medical equipment, basic ag products. |
| `<A3_chiu_thue_8>`   | 8%   | Temporary reduction (Decree 15/2022 + extensions; valid through 2026 per Nghị quyết 142/2024/QH15). |
| `<A4_chiu_thue_10>`  | 10%  | Standard rate. |

Each row has the structure:

```xml
<A4_chiu_thue_10>
  <giaTri>90000000</giaTri>  <!-- taxable value, integer VND -->
  <thue>9000000</thue>       <!-- VAT, integer VND, rounded half-up -->
</A4_chiu_thue_10>
```

### Totals

| Tag | Type | Required | Notes |
|---|---|---|---|
| `<tongGiaTri>` | integer VND | yes | Sum of all `<giaTri>` rows. |
| `<tongThue>`   | integer VND | yes | Sum of all `<thue>` rows (NOT `tongGiaTri × average_rate`). |

#### Rounding rule

VAT per rate row is rounded **half-up** to integer VND. The section total `<tongThue>` is the *sum of the rounded per-rate VATs*, not the rounded sum of taxable values multiplied by an effective rate. This matches the per-line rule for individual invoices in `vn-vat-invoice`.

## `<ThueDauVao>` — Section B — Input VAT

| Tag | Type | Required | Notes |
|---|---|---|---|
| `<B1_tongGiaTriMua>`        | integer VND | yes | Total value of purchases this period. |
| `<B2_tongThueDauVao>`       | integer VND | yes | Total deductible input VAT (from supplier VAT invoices). |
| `<B3_tongThueDuocKhauTru>`  | integer VND | yes | Allowable deduction this period; usually equals B2, but the taxpayer MAY defer a portion if the input VAT is contested or pending substantiation. |

## `<ThuePhaiNop>` — Section C — Net payable / refundable

| Tag | Type | Required | Formula |
|---|---|---|---|
| `<C1_thuePhatSinh>`           | integer VND | yes | `sum(A.thue) - B3` (may be negative — surplus deductible) |
| `<C2_thueKyTruocChuyenSang>`  | integer VND | yes | Carry-over from previous period (zero if first period or no carry-over). |
| `<C3_thueDaNopTrongKy>`       | integer VND | yes | VAT already paid in this period (e.g. provisional payments). |
| `<C4_thueConPhaiNop>`         | integer VND | yes | `max(0, C1 + C2 - C3)` — tax payable. |
| `<C5_thueDeNghiHoan>`         | integer VND | yes | `max(0, -(C1 + C2 - C3))` — refund request. |

Exactly one of `C4` / `C5` is non-zero (or both zero in a wash period).

## Common pitfalls

- **Period format**: `2026-05`, not `05/2026` or `Tháng 5/2026`. Quarterly is `2026-Q2`, not `2026Q2` or `Q2/2026`.
- **Date format**: `YYYY-MM-DD` everywhere; the Vietnamese-typed `dd/mm/yyyy` is rejected.
- **MST whitespace**: trim before embedding. Validator rejects MSTs with embedded spaces.
- **Per-rate rounding**: round each row, then sum. Do NOT sum first then apply a single rate.
- **Negative C1**: legal. Represents surplus deductible input VAT carried forward (becomes the next period's `C2`).
- **Both C4 and C5 non-zero**: validator rejects. Choose one based on the sign of `C1 + C2 - C3`.
- **Mixing monthly and quarterly**: the `kyKKhai` attribute on root MUST agree with the period string. A taxpayer switches between frequencies once per fiscal year, not mid-period.

## Source documents

- **Thông tư 80/2021/TT-BTC** (Ministry of Finance) — defines Mẫu 01/GTGT. Appendix II carries the form layout. Article 8 covers monthly-vs-quarterly eligibility.
- **Luật Quản lý thuế 38/2019/QH14** (Law on Tax Administration) — Article 44 (filing deadlines); Article 86 (weekend / holiday roll-forward); Article 59 (interest on late payment).
- **Luật Thuế GTGT 13/2008/QH12** (VAT Law) and amendments — substantive VAT rates.
- **Nghị quyết 142/2024/QH15** — extends the 8% reduced rate through 31 December 2026.

For the authoritative XSD, check `gdt.gov.vn` → "Quản lý thuế" → "Mẫu biểu" → "Mẫu 01/GTGT".
