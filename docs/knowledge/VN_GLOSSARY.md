# Vietnamese ↔ English Glossary

**Created:** 2026-05-16
**Scope:** FR-* specs under `docs/tasks/`
**Policy:** Spec bodies use English-only terminology. Vietnamese names appear here when the underlying concept is VN-regulatory (no clean English equivalent). FRs reference glossary entries via `VN-GLO:<key>`.

---

## A. Legal forms & registrations

| Key | Vietnamese | English equivalent | Notes |
|---|---|---|---|
| `vn-gdt` | Tổng cục Thuế (GDT) | General Department of Taxation | The VN tax authority. Issues e-invoice schemas. |
| `vn-mst` | Mã số thuế (MST) | Tax identification number | 10-digit base + 3-digit branch suffix. Required on every B2B invoice. |
| `vn-cccd` | Căn cước công dân (CCCD) | Citizen ID card | 12-digit national ID. Replaced CMND in 2021. PII-elevated. |
| `vn-cmnd` | Chứng minh nhân dân (CMND) | National ID card (legacy) | Pre-2021 12-digit or 9-digit form. Migrating to CCCD. |
| `vn-vneid` | VNeID | National digital identity | Government-issued mobile-app identity. Used for KYC. |
| `vn-mops` | Bộ Công an (MoPS) | Ministry of Public Security | Authority for PDPL enforcement + breach reporting. |
| `vn-pdpc` | Cục An toàn Thông tin | Department of Information Security | Regulatory authority for cyber-security incidents. |

## B. Tax & invoicing

| Key | Vietnamese | English equivalent | Notes |
|---|---|---|---|
| `vn-hoa-don` | Hóa đơn điện tử | E-invoice (Decree 123) | VAT invoice format mandated by Decree 123/2020. XML schema published by GDT. |
| `vn-vat` | Thuế giá trị gia tăng (VAT) | Value-added tax | Standard rate 10%; 5% reduced; 0% export. |
| `vn-decree-123` | Nghị định 123/2020/NĐ-CP | Decree 123 (e-invoice) | Mandatory e-invoice for all businesses since 1 July 2022. |
| `vn-pit` | Thuế thu nhập cá nhân (PIT) | Personal income tax | Progressive rates 5%–35%. |
| `vn-cit` | Thuế thu nhập doanh nghiệp (CIT) | Corporate income tax | Standard 20%; SME band 15–17%. |
| `vn-fct` | Thuế nhà thầu (FCT) | Foreign contractor withholding tax | Applied to cross-border service payments. |

## C. Labour & social insurance

| Key | Vietnamese | English equivalent | Notes |
|---|---|---|---|
| `vn-bhxh` | Bảo hiểm xã hội (BHXH) | Social insurance | Employee 8% + employer 17.5% of monthly base. |
| `vn-bhyt` | Bảo hiểm y tế (BHYT) | Health insurance | Employee 1.5% + employer 3%. |
| `vn-bhtn` | Bảo hiểm thất nghiệp (BHTN) | Unemployment insurance | Employee 1% + employer 1%. |
| `vn-decree-145` | Nghị định 145/2020/NĐ-CP | Decree 145 (Labour Code) | Working-hours, OT, annual-leave calculation rules. |
| `vn-decree-152` | Nghị định 152/2020/NĐ-CP | Decree 152 (SI rates) | Social-insurance rate table; version-pinned per year. |
| `vn-art-107` | Điều 107 BLLĐ | Labour Code Article 107 | OT cap: max 200h/year (300h with special approval). |
| `vn-cap-ot` | Trần làm thêm giờ | Overtime ceiling | The Art-107 hard-block at allocation. |

## D. Payments & banking

| Key | Vietnamese | English equivalent | Notes |
|---|---|---|---|
| `vn-vietqr` | VietQR | Domestic QR-payment standard | Napas247 interbank rail. |
| `vn-napas247` | Napas 247 | Real-time interbank rail | Sub-second VND transfers; webhook callbacks. |
| `vn-vnpay` | VnPay | Domestic payment gateway | Card + wallet acquirer. |
| `vn-momo` | Momo (M_Service) | E-wallet | Consumer wallet; merchant API for B2C. |
| `vn-zalopay` | ZaloPay | E-wallet | Consumer wallet from VNG. |
| `vn-acb-rate` | Lãi suất ACB | ACB benchmark rate | Reference rate used for BP-ledger interest accrual. |
| `vn-sbv-fx` | Tỷ giá NHNN | State Bank FX snapshot | Daily official FX rate; used for invoice multi-currency. |

## E. People & organisation roles

| Key | Vietnamese | English equivalent | Notes |
|---|---|---|---|
| `vn-hd-chuyen-mon` | Hội đồng Chuyên môn | Specialist Council | 3-5-judge panel evaluating Member skill promotions. |
| `vn-bang-cap` | Bằng cấp | Degree certificate | Tertiary academic credential. Evidence type for LEARN. |
| `vn-chung-chi` | Chứng chỉ | Professional certificate | Industry certification. Evidence type for LEARN. |
| `vn-vp` | Voting Power (VP) | Voting power | Internal seniority score; PROJ + TIME + KB rollup. |
| `vn-bp` | Bonus Points (BP) | Bonus points | Quarterly bonus pool unit; ACB-rate interest accrual. |
| `vn-p1` | P1 (Lương cơ bản) | Base pay | Floor pay; DB CHECK constraint forbids reduction. |
| `vn-p2` | P2 (Phụ cấp) | Allowance | Fixed allowances. |
| `vn-p3` | P3 (Lương hiệu suất) | Performance pay | Variable; quarterly distribution from BP fund. |

## F. Tenant residency suffixes

| Code | Region | Compliance frame |
|---|---|---|
| `sg-1` | Singapore | PDPA (SG) + Cloud-Outage-Reportable (CSP) |
| `eu-1` | EU (Ireland by default) | GDPR + DORA + eIDAS QTSP requirements |
| `us-1` | US (us-east-1 by default) | CCPA + state-level breach notification |
| `vn-1` | Vietnam | PDPL Law 91/2025 + Decree 13/2023 (data localisation) + Decree 53/2022 |

## G. Compliance frameworks (regulator-side acronyms)

| Acronym | Full name | Scope |
|---|---|---|
| PDPL | Personal Data Protection Law 91/2025 (VN) | Replaces Decree 13/2023; VN data localisation + DPO requirement. |
| Decree 13/2023 | NĐ 13/2023/NĐ-CP | Pre-PDPL data-protection regulation. Still applies in transition. |
| Decree 53/2022 | NĐ 53/2022/NĐ-CP | Cybersecurity-Law implementing regulation. Data-localisation triggers. |
| DPO | Data Protection Officer | Required role under PDPL Art. 28 for high-risk processing. |

---

## Usage in FRs

When an FR needs to reference a VN-specific concept, it MUST:

1. Use the **English equivalent** in spec body prose.
2. On first use, include a parenthetical `(VN-GLO:<key>)` citing this glossary.
3. NOT use the Vietnamese term inline (the glossary is the single source of translation).

**Example (correct):**
> The handler MUST validate the tax identification number (VN-GLO:vn-mst) format using the official check-digit algorithm before persisting.

**Example (incorrect — inline VN term):**
> The handler MUST validate the MST (mã số thuế) format ...

The only exception is the FR title + frontmatter `title:` field, which MAY use the VN term if it's commonly used by stakeholders (e.g. `"Hóa đơn auto-emit"`). The first body paragraph MUST then re-introduce the term with the glossary reference.

---

*End of VN_GLOSSARY.md — version 1.0 — 2026-05-16*
