# `threat-model-audit` — fine-tune discipline override

Default discipline at `../docs/FINE_TUNE.md`. This file documents the **threat-model-audit-specific overrides**.

## Why threat-model-audit is different

This rubric tracks two fast-moving security frameworks:

- **OWASP Top 10** — major releases every 3-4 years (most recent: 2025). Each release reshapes which risks are elevated / introduced / merged. The OWASP-A01..A10 rule family directly mirrors the active release.
- **OWASP ASVS** — quarterly updates. The §6 ASVS Controls Mapping table content evolves continuously.
- **CVE catalog** — daily growth. The QA-CVE-001 rule (fabricated CVE format check) is stable but the underlying threat-model bodies cite CVEs that need refreshing.
- **LINDDUN** — privacy threat framework. Less frequent updates but each one ripples into COND-001.
- **STRIDE** — stable since Microsoft Loren Kohnfelder + Praerit Garg 1999; no expected change.

## Release-triggered fine-tune cadence

| Trigger | Action | Bump | Reviewer |
|---|---|---|---|
| New OWASP Top 10 release | Rewrite the OWASP-A01..A10 rule family. Repoint §5 mappings. | major (`@1.x → @2.0`) | CSecO + CTO + CLO (compliance impact) |
| ASVS minor release | Update the ASVS controls list. Add new control IDs. | minor | CSecO |
| LINDDUN methodology update | Update COND-001 mapping. | minor | CSecO + CPO-Privacy |
| New high-impact CVE pattern (e.g. log4shell-scale) | Add specific watch-list entry to threat-model template (not RUBRIC; the rubric just validates format) | n/a | CSecO informational |
| New AI-specific threat (e.g. prompt-injection attack class) | Add to COND-002 (AI/ML threats) | minor | CSecO + CAIO |

## Quarterly review cadence

Each quarter, the CSecO (or designee) SHALL:

1. Review the OWASP ASVS changelog for net-new controls.
2. Review the LINDDUN-Pro release notes if any.
3. Check whether any threat the team is tracking (in their CyberSkill engagements) needs to be promoted to a rubric rule.
4. Land changes as minor bumps with explicit changelog rationale.

## Compliance-boundary rules — extra scrutiny

The following rules touch regulator territory and require CLO co-review on any change:

- **COMP-GDPR-001** (if EU residency) — must resolve to actual document path or memory memory_id.
- **COMP-VN-001** (if Vietnam residency) — Decree 13/2023 PDPD + Decree 53/2022 cybersecurity references.
- **COMP-AI-001** — SDP §5 AI-use disclosure requirements.
- **COND-009** (HIPAA-aligned controls) — when PHI is in scope.

Any change to these requires CSecO + CLO sign-off.

## Forbidden without major version bump

- Removing any of the OWASP-A01..A10 rules (these enforce industry-baseline coverage).
- Removing the STRIDE category enforcement (STRIDE-S/T/R/I/D/E-001).
- Removing the QA-CVE-001 anti-fabrication check (CVE fabrication is high-stakes).
- Lowering ASVS L-level requirements below the declared `asvs_level` floor.

## Blackout windows

- **OWASP release week** — when a new Top 10 ships, freeze unrelated changes for 2 weeks to focus on the major rewrite.
- **Annual SOC 2 audit observation period** (Q3 typically) — no rubric changes that would invalidate evidence already captured.

## Cross-references

- `RUBRIC.md` — the rubric body.
- `../docs/FINE_TUNE.md` — master default discipline.
- OWASP Top 10:2025, OWASP ASVS, LINDDUN — primary frameworks tracked.
- `../threat-model-author/` — the sibling skill whose output this rubric validates.
- NVD / MITRE CVE — the format authority for QA-CVE-001.
