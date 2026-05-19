---
id: NFR-AUTH-007
title: "AUTH signing-key rotation — active key always present; old key honored 7 days post-rotation"
module: AUTH
category: security
priority: MUST
verification: T
phase: P0
slo: "Rotation operation completes with at least one active key on JWKS at all times; old key marked retired but honored for verify for 7 days"
owner: CSO
created: 2026-05-18
related_frs: [FR-AUTH-004]
---

## §1 — Statement (BCP-14 normative)

1. The AUTH signing-key rotation procedure **MUST** ensure at least one key with `status=active` is present in JWKS at every moment during rotation — there is **never** a window where JWKS is empty or has no active key.
2. The rotation procedure **MUST** mark the previous key `status=retired` (not deleted); retired keys remain in JWKS and are honored for verify (NFR-AUTH-003) for exactly 7 days post-rotation.
3. After the 7-day window, the retired key **MUST** be hard-deleted from JWKS and from the database. The deletion **MUST** be audited as `auth.signing_key.deleted` in memory.
4. The rotation **MUST** be scriptable via `cyberos-auth rotate-signing-key` (the bootstrap CLI surface from FR-AUTH-006).
5. Two simultaneous rotations within 7 days **MUST NOT** be permitted — the CLI rejects with "previous rotation still in 7-day grace window."

## §2 — Why this constraint

A key rotation that creates an empty-JWKS window breaks every active token (mass logout). The "always one active key" invariant is the load-bearing safety property. The 7-day grace window is the JWT TTL ceiling — a token issued just before rotation must still verify for its full TTL. Hard deletion after 7d removes the rotation residue (compliance: minimise key lifetime). The no-double-rotation rule prevents an operator from chaining rotations and confusing the grace window.

## §3 — Measurement

- Gauge `auth_signing_keys_active_count` — should always be ≥ 1. Sev-0 alarm on 0.
- Gauge `auth_signing_keys_retired_count` — typically 0-1; sev-3 if > 2 (failed deletions piling up).
- memory audit query `view kind=auth.signing_key.{created,retired,deleted}` — every rotation produces a triple.

## §4 — Verification

- Integration test `services/auth/tests/signing_key_rotation_test.rs` (T) — drives a rotation, asserts JWKS contains both keys during the 7-day window, then asserts retired key is deleted after 7d.
- Property test (T) — drives 100 random rotation sequences; asserts active count never drops to 0.

## §5 — Failure handling

- `active_count = 0` → sev-0; emergency bootstrap-cli rotation to inject a new key; investigate how the invariant was violated.
- Retired key past 7d still present → sev-3; ticket to operations; manual cleanup.
- Operator runs double rotation → CLI rejects; operator waits for grace window.

---

*End of NFR-AUTH-007.*
