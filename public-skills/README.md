# CyberSkill Vietnamese-market Skills

Agent Skills for Vietnamese workflows. Compatible with the open Agent Skills specification.

## Skills

| Name | Purpose | Capabilities |
|---|---|---|
| `vn-mst-validate` | Validate Vietnamese tax codes (Mã số thuế / MST) | Structural check of 10-digit and 10+3-digit MST per GDT format |
| `vn-vat-invoice` | Generate Vietnamese VAT e-invoices (hoá đơn GTGT điện tử) | XML in GDT schema v3.0 from JSON line-items; depends on `vn-mst-validate` |
| `vneid-integration` | Citizen ID (CCCD) validation + VNeID request payload builder | 12-digit CCCD structural decode (province / gender / year); offline payload construction |
| `vn-bank-transfer` | VietQR / Napas247 QR payload codec + bank BIN lookup | Generate and parse VietQR strings; lookup Napas member BIN codes |
| `vn-legal-compliance` | Vietnamese data-protection / cybersecurity reference | Decree 13/2023/NĐ-CP, Decree 53/2022/NĐ-CP, breach windows, DPO rules |

## Install

Each skill is a directory containing `SKILL.md`. Drop the skill directory into your host's skills root. Restart the host. See [INSTALL.md](INSTALL.md).

## License

Apache 2.0 — see [LICENSE](LICENSE).

## Roadmap

- VSS / BHXH social-insurance code validation
- E-signature (chữ ký số) wrapper for common Vietnamese CAs
- Customs declaration (tờ khai hải quan) payload builder
- Provincial address normaliser (63 provinces / wards)
- Vietnamese-language NLP helpers (tone marks, diacritic normalisation)

## Maintainer

CyberSkill Software Solutions Consultancy, HCM City, Vietnam.
