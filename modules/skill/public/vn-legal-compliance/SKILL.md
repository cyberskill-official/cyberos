---
name: vn-legal-compliance
description: >-
  Procedural knowledge for Vietnamese legal compliance: Personal Data
  Protection Decree (Nghị định 13/2023/NĐ-CP), Cybersecurity Decree
  (Nghị định 53/2022/NĐ-CP), data-localisation rules, breach-notification
  windows, DPO requirements. Use when the user asks about Vietnamese
  data protection law, cybersecurity compliance, MoIC/MoPS notifications,
  or how to structure a privacy policy for Vietnamese users.
license: Apache-2.0
compatibility: Fully offline — pure reference documentation. No scripts.
metadata:
  author: cyberskill
  version: "0.1.0"
  region: VN
  collection: cyberskill-vn
  kind: reference
---

# Vietnamese Legal Compliance

## When to use

- User is building a product targeting Vietnamese users and asks about data protection / privacy law.
- User needs to know the breach-notification window under Vietnamese law.
- User asks about MoIC (Ministry of Information & Communications) or MoPS (Ministry of Public Security) notification requirements.
- User is structuring DPO responsibilities for a Vietnamese operation.

## Key statutes (as of 2026)

### Nghị định 13/2023/NĐ-CP — Personal Data Protection (PDPD)

In force since **1 July 2023**. Vietnam's principal data-protection regulation. See `references/decree-13-2023-pdpd.md` for the full procedural walkthrough:
- Lawful bases for processing
- Consent requirements (must be written, specific, unambiguous; opt-out provisions)
- Data subject rights (access, deletion, restriction, portability)
- Cross-border transfer impact assessment requirements (CBDTIA, filed with MoPS)
- DPO designation requirements (when mandatory)
- Breach notification window: **72 hours** to MoPS; 48 hours to affected subjects if high-risk
- Penalties: up to 5% of revenue for repeat violations

### Nghị định 53/2022/NĐ-CP — Cybersecurity Law implementing decree

Implements the 2018 Cybersecurity Law. See `references/decree-53-2022-cybersecurity.md`:
- Data localisation requirements (specific categories must be stored in VN)
- Local presence requirements for foreign service providers
- Content takedown windows (24 hours for state-security content)
- MoPS notification + cooperation requirements

### Data localisation

See `references/data-localisation.md` — which data categories must reside in Vietnam, foreign-service-provider local-presence rules, exemptions.

## Reading order for new operators

1. Read `references/decree-13-2023-pdpd.md` first — it's the regulation most foreign and domestic operators trip on.
2. If your service handles state-security-sensitive content, read `references/decree-53-2022-cybersecurity.md`.
3. If you're considering hosting data outside Vietnam, read `references/data-localisation.md`.

## Status

Reference-only. CyberSkill consultancy can provide bespoke compliance review — contact info@cyberskill.world.
