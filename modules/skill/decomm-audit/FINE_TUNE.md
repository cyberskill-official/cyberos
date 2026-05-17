# `decomm-audit` — fine-tune discipline override

Default discipline at `../docs/FINE_TUNE.md`. This file documents the **decomm-audit-specific overrides**.

## Why decomm-audit is different

This rubric encodes **regulatory disposal requirements** that vary by jurisdiction + by data class. Each regulation has its own update cadence + its own enforcement teeth. Decommissioning a system with a botched disposal is a high-stakes compliance event (GDPR Art. 17 violations alone carry fines up to 4 % of global revenue).

The regulations tracked:

- **GDPR Article 17** (Right to Erasure) — EU. Stable since 2018 but enforcement guidance evolves (EDPB opinions, national-DPA decisions).
- **Vietnam Decree 13/2023 PDPD** — CyberSkill home jurisdiction. Effective 2023-07-01. Enforcement guidance still maturing.
- **Vietnam Decree 53/2022** (cybersecurity) — adjacent to PDPD.
- **PCI-DSS Requirement 9.8** — media destruction. Released as part of PCI-DSS 4.0 (2024); updates roughly every 3 years.
- **HIPAA 45 CFR § 164.310(d)(2)** — health-data disposal. Stable text but HHS enforcement guidance updates yearly.

## Regulator-triggered fine-tune cadence

| Trigger | Action | Bump | Reviewer |
|---|---|---|---|
| New GDPR EDPB opinion impacting Art. 17 | Update COND-001 + COMP-GDPR-* rules | minor | CLO + CPO-Privacy |
| Vietnam DPA enforcement decision affecting Decree 13/2023 disposal | Update COND-001 (Vietnam-specific addendum) + add precedent reference to rubric body | minor | CLO + CPO-Privacy |
| PCI-DSS minor release | Update COND-002 + COMP-PCI-* (when introduced) | minor | CCO-Compliance |
| PCI-DSS major release (e.g. 5.0) | Full rubric review; potentially new section family | major | CCO-Compliance + CLO |
| HHS enforcement guidance update on HIPAA disposal | Update COND-003 | minor | CLO |
| New jurisdiction regulation (e.g. Brazil LGPD disposal rules) | Add new COND-NNN trigger + COMP-NNN-* rules | minor | CLO + CPO-Privacy |

## Quarterly review cadence

Each quarter, the CLO + CPO-Privacy SHALL:

1. Scan EDPB / national-DPA decision database for Art. 17 precedents.
2. Scan PCI-SSC bulletins.
3. Scan HHS enforcement bulletins.
4. Scan Vietnam MIC / MPS bulletins for Decree 13/2023 enforcement.
5. Land changes as minor bumps with explicit regulator-citation in the changelog.

## Compliance-boundary rules — always require sign-off

Every rule in the COMP-* family + every COND-* trigger touches regulator territory. Changes require:

- **CLO** (always).
- **CPO-Privacy** (for personal-data rules).
- **CSecO** (for cybersecurity-touching rules — e.g. Vietnam Decree 53/2022 references).
- **CCO-Compliance** (when shipped — for PCI-DSS).
- **External counsel** (when the change is jurisdictionally novel).

## Forbidden without major version bump + outside-counsel review

- Removing any COMP-* rule (these encode legal positions).
- Removing any COND-001..003 trigger (GDPR / VN Decree 13 / PCI / HIPAA — each is a regulator-imposed minimum).
- Loosening any `destruction_method:` requirement in §4 / QA-DATA-001.
- Changing the witness requirement in QA-DATA-002 (this is the auditability anchor).

## Specific watchlist — known upcoming regulatory changes

| Regulation | Expected change | When | Anticipated rubric impact |
|---|---|---|---|
| EU AI Act Art. 50 (transparency) | Implementing acts 2026-2027 | rolling | Add AI-system-disposal language to COND-001 |
| Vietnam Decree 13/2023 follow-on circular | Expected 2026 H2 | TBD | Possible new mandatory cross-border-transfer-on-disposal block |
| PCI-DSS 5.0 | Expected 2027 | TBD | Likely new media-destruction methods recognised |

## Acceptance regression requirement

Every change to COND-* / COMP-* / QA-DATA-* rules SHALL ship with:

1. A synthetic decomm fixture that triggers the changed rule.
2. The expected audit report showing the rule firing correctly.
3. An "edge case" fixture that does NOT trigger the rule (to guard against false positives).

## Blackout windows

- **Active client decommissioning project** — when a CyberSkill client is mid-decomm, the rubric is frozen for the duration to ensure audit-report consistency.
- **Q4 fiscal year-end** — common decomm season; freeze unless emergency.

## Cross-references

- `RUBRIC.md` — the rubric body.
- `../docs/FINE_TUNE.md` — master default discipline.
- GDPR Article 17, Vietnam Decree 13/2023 PDPD, Vietnam Decree 53/2022, PCI-DSS Requirement 9.8, HIPAA 45 CFR § 164.310(d)(2) — primary regulatory sources.
- `../decomm-author/` — the sibling skill.
