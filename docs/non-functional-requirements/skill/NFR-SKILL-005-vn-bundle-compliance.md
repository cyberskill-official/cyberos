---
id: NFR-SKILL-005
title: "SKILL VN-bundle compliance — MST/VietQR/HoaDon skills MUST pass external validator"
module: SKILL
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% pass rate against GDT MST + NAPAS VietQR + GDT hoadondientu sandboxes weekly"
owner: CFO
created: 2026-05-18
related_frs: [FR-SKILL-108, FR-SKILL-109, FR-SKILL-110]
---

## §1 — Statement (BCP-14 normative)

1. The three Vietnamese-locale skills (`vn-mst-validate`, `vn-bank-transfer`, `vn-vat-invoice`) **MUST** pass a weekly end-to-end validation run against their respective external authorities: GDT MST lookup endpoint, NAPAS VietQR specification validator, and GDT hoadondientu sandbox.
2. The validation harness **MUST** execute against a fixed regression set of 50+ cases per skill covering happy path + known edge cases (invalid MST checksum, malformed bank account, foreign-currency VAT, etc.).
3. Any week with < 100% pass **MUST** block the next skill-bundle release of that skill.
4. The skills **MUST NOT** silently degrade behaviour when external authorities are unreachable — they return `E_UPSTREAM_UNAVAILABLE` to the caller, never a fabricated success.
5. Each skill's audit row **MUST** include the external-authority transaction reference where applicable (MST cache-source, NAPAS sandbox tx-id, GDT invoice number) so post-hoc verification is possible.

## §2 — Why this constraint

Vietnamese tax + payment + invoicing is highly regulated; an incorrect MST lookup or malformed VietQR can void a transaction. The three skills are the platform's regulatory "promises" — they're trusted by the rest of the stack (CRM, REW, EMAIL) to be correct. Weekly external-validator runs catch silent drift: GDT changes a checksum rule, NAPAS adds a new QR field, GDT invoicing rejects a new edge case. Coupling release gates to validator pass ensures the platform never knowingly ships a regressed VN skill.

## §3 — Measurement

- CI metric per skill: `skill_vn_validator_pass_count` and `skill_vn_validator_fail_count` per weekly run.
- Counter `skill_vn_upstream_unavailable_total{skill}` — surfaces external-authority downtime separately from skill bugs.
- Histogram of validator response latency — surfaces silent perf regressions in external authorities.

## §4 — Verification

- Weekly CI job `vn-skill-validator` (T) — runs the 50-case regression set per skill against the live external sandboxes.
- Quarterly compliance attestation report combining 13 weeks of CI results — handed to CFO + CLO-Legal.
- Synthetic monitoring (T) — hourly single-case ping against each external authority; alarms surface authority downtime within 5 minutes.

## §5 — Failure handling

- Single weekly run fails → sev-3; release of that skill blocked until validators pass.
- Two consecutive weekly runs fail → sev-2; CFO + CLO-Legal informed; root-cause investigation.
- External authority down > 4 hours → sev-3 logged but does not block release (it's their outage, not ours); skill returns E_UPSTREAM_UNAVAILABLE to callers.

---

*End of NFR-SKILL-005.*
