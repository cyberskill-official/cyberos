---
id: NFR-DOC-001
title: "DOC eIDAS QTSP attestation — signatures MUST be QES-grade for EU jurisdiction docs"
module: DOC
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of EU-jurisdiction signatures carry a valid QES attestation from a recognised QTSP"
owner: CLO-Legal
created: 2026-05-18
related_tasks: [TASK-DOC-002]
---

## §1 — Statement (BCP-14 normative)

1. Signatures applied to documents tagged `jurisdiction: EU` **MUST** be Qualified Electronic Signatures (QES) per eIDAS Regulation EU 910/2014, backed by a QTSP from the EU trusted list.
2. The platform's QTSP integration **MUST** support at least two QTSPs to avoid single-vendor risk.
3. Each signature **MUST** carry a verifiable attestation: QTSP issuer, certificate serial, timestamp from a qualified TSA, signing-time policy OID.
4. The QTSP integration **MUST** undergo annual re-validation against the EU Trust List; expired/revoked QTSPs are auto-removed.
5. Signature events **MUST** be auditable with full chain-of-trust artifact persisted.

## §2 — Why this constraint

QES carries the highest legal weight in the EU — equivalent to handwritten signature. Anything less (AES, SES) doesn't qualify. Using a non-QTSP or expired QTSP voids the legal status of the signature. Two-QTSP redundancy avoids single-vendor outage breaking signing capability. Annual re-validation against the EU Trust List is the only way to know your QTSP is still trusted.

## §3 — Measurement

- Counter `doc_signature_total{jurisdiction, signature_class}` — assert EU + non-QES = 0.
- Gauge `doc_qtsp_trust_status{qtsp_id}` — 1=trusted, 0=removed/expired.
- Audit row per signature with full chain.

## §4 — Verification

- Integration test (T) — EU doc sign; assert QES attestation present.
- CI gate (T) — QTSP roster matches EU Trust List.
- Annual external audit (A) — eIDAS conformance review.

## §5 — Failure handling

- EU + non-QES → block sign; CLO-Legal investigates.
- QTSP removed from Trust List → auto-remove from roster; alert CLO-Legal.
- QTSP outage → fall back to secondary QTSP; alert.

---

*End of NFR-DOC-001.*
