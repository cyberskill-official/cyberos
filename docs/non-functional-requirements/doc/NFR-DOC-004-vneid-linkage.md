---
id: NFR-DOC-004
title: "DOC VNeID linkage — identity-verified signers MUST be bound to VNeID identifier"
module: DOC
category: compliance
priority: SHOULD
verification: T
phase: P1
slo: "100% of VNeID-verified signers carry the VNeID identifier in the signature attestation"
owner: CLO-Legal
created: 2026-05-18
related_frs: [FR-DOC-006]
---

## §1 — Statement (BCP-14 normative)

1. When a signer's identity is verified via VNeID (Vietnamese national digital ID), the resulting signature **MUST** carry the VNeID identifier in the attestation block.
2. The VNeID identifier **MUST** be encrypted at rest (per FR-HR-003 KMS practice); plain-text storage forbidden.
3. The signature row **MUST** also include `idv_method=vneid`, `idv_session_id`, `idv_completed_at` for audit.
4. VNeID API outages **MUST NOT** silently downgrade to non-IDV signatures — the sign is blocked until VNeID is reachable or the signer chooses an alternate IDV method.
5. VNeID identifiers **MUST NOT** be displayed in UI to any user other than CLO-Legal + the signer themselves.

## §2 — Why this constraint

VNeID linkage is the strongest civilian identity proof in Vietnam. Embedding it in the signature provides legal-grade attestation. The encryption-at-rest + UI-restriction rules apply standard PII handling. The "no silent downgrade" rule prevents reduced-assurance signatures being attributed as VNeID-verified.

## §3 — Measurement

- Counter `doc_vneid_signature_total`.
- Counter `doc_vneid_outage_block_total` — surfaces VNeID availability.
- Audit row check: VNeID id is encrypted in storage.

## §4 — Verification

- Integration test (T) — VNeID sign; assert id encrypted + attestation present.
- Outage test (T) — VNeID unreachable; assert block + alternate offered.
- Privacy test (T) — non-privileged user query; assert VNeID id not returned.

## §5 — Failure handling

- Silent downgrade observed → sev-1; legal liability.
- VNeID outage block sustained → sev-3; alert ops; alternate IDV path.
- Encryption-at-rest check fails → sev-1; PII exposure.

---

*End of NFR-DOC-004.*
