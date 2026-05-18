---
id: NFR-DOC-003
title: "DOC VN-CA signature compliance — Vietnamese signatures MUST chain to NEAC root"
module: DOC
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of VN-jurisdiction signatures chain to a NEAC-recognised CA"
owner: CLO-Legal
created: 2026-05-18
related_frs: [FR-DOC-004]
---

## §1 — Statement (BCP-14 normative)

1. Documents tagged `jurisdiction: VN` **MUST** be signed with certificates that chain to a National Electronic Authentication Centre (NEAC) recognised root CA.
2. The platform's VN-CA roster **MUST** be re-validated against NEAC's trusted list quarterly.
3. Each signature **MUST** carry the full certificate chain in the signed bundle for self-contained verification.
4. The TSA timestamp **MUST** come from a VN-recognised TSA for VN-jurisdiction documents.
5. Revoked VN-CA certificates **MUST** be detected via OCSP/CRL polling within 24h; existing signatures using revoked certs remain valid for their pre-revocation period.

## §2 — Why this constraint

Vietnamese law recognises specific CAs for legally-binding electronic signatures. Using a non-recognised CA produces a document that's technically signed but legally invalid in VN courts. The NEAC list is the source of truth. The chain-included-in-bundle requirement ensures the doc verifies offline. OCSP/CRL polling closes the certificate-revocation feedback loop.

## §3 — Measurement

- Counter `doc_signature_total{jurisdiction=VN, ca_recognised}` — assert non-recognised = 0.
- Gauge `doc_vn_ca_revocation_age_hours` — must stay < 24.
- Quarterly NEAC roster sync report.

## §4 — Verification

- Integration test (T) — VN sign with non-NEAC CA → reject.
- Quarterly NEAC list refresh (A).
- OCSP/CRL polling cron + alert.

## §5 — Failure handling

- Non-NEAC sign attempt → block + audit.
- CA revoked → roster update + alert CLO-Legal.
- OCSP/CRL stale > 24h → sev-3; check polling job.

---

*End of NFR-DOC-003.*
