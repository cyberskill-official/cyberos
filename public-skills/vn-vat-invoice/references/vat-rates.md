# Vietnamese VAT rates — when each applies

As of 2026, Vietnamese VAT (Thuế Giá trị Gia tăng / GTGT) has four primary rates: `0%`, `5%`, `8%`, `10%`. The applicable rate depends on the nature of the goods or services supplied, and (for the temporary 8% rate) the date of supply.

## 0%

**Applies to**:

- Exported goods (`hàng hoá xuất khẩu`).
- Exported services (`dịch vụ xuất khẩu`), e.g. consulting delivered to a foreign buyer with consumption outside Vietnam.
- International transport (`vận tải quốc tế`).
- Construction, installation, processing performed for export-processing zones (EPZs).

**Substantiation required**: a 0% invoice is only valid if the seller can substantiate the export with customs declarations, contracts with the foreign buyer, and bank-credit evidence of payment in convertible currency. Without these, the supply is reclassified at 10% on audit.

## 5%

**Applies to** (non-exhaustive — see Article 8 of the VAT Law for the full list):

- Clean water (`nước sạch`) supplied for production and household use.
- Fertilizers (`phân bón`), pesticides, animal feed.
- Medical equipment and pharmaceuticals.
- Teaching aids, scientific equipment.
- Children's toys (specifically educational).
- Basic agricultural products at the wholesale stage.
- Sugar and by-products from sugar production.

## 8% (temporary)

**Status**: temporary reduction from the standard 10% rate. Originally introduced by Nghị định 15/2022/NĐ-CP to support post-COVID economic recovery; extended several times. Per the current legal framework, the 8% rate remains available through end of 2026 for most goods and services that would otherwise be at 10%, with notable exclusions:

- Telecommunications, IT services
- Financial activities, banking, securities, insurance
- Real estate
- Metals, mining products, coke, refined petroleum, chemicals
- Goods and services subject to excise tax (alcohol, tobacco, etc.)

**Implementation note**: invoices issued during the reduction window must explicitly carry the `8%` rate string; substituting `10%` and refunding the 2 percentage points later is not permitted. The applicable rate is determined by the *date of supply*, not the date of invoice issuance.

## 10%

The standard rate. Applies to any taxable supply not specifically listed at 0%, 5%, or covered by the temporary 8% reduction.

## VAT-exempt (no rate)

Some supplies are *exempt* from VAT rather than zero-rated. The distinction matters:

- **0% rate**: supplier charges 0% but can claim input VAT credits.
- **Exempt**: supplier charges nothing but cannot claim input VAT credits.

Exempt supplies include agricultural production at the farmer stage, some financial services, and specific educational/medical services. Exempt invoices are issued without a `<VatRate>` element (or with `KCT` / `Không chịu thuế` in some legacy formats). This skill currently does **not** model exempt supplies — submit a feature request if you need that.

## Source documents

- Luật Thuế GTGT 13/2008/QH12 (and amendments)
- Nghị định 209/2013/NĐ-CP
- Nghị định 15/2022/NĐ-CP (the original 8% reduction)
- Nghị quyết 110/2023/QH15 (extension to 2024)
- Nghị quyết 142/2024/QH15 (extension through 2026, current as of this writing)
