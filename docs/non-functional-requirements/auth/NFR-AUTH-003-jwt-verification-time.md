---
id: NFR-AUTH-003
title: "AUTH JWT signature verification time — < 1ms p99 single key; < 5ms p99 with 7-day rotation window"
module: AUTH
category: performance
priority: MUST
verification: T
phase: P0
slo: "Single-key verify p99 < 1ms; multi-key (during 7-day rotation) p99 < 5ms"
owner: CTO
created: 2026-05-18
related_frs: [FR-AUTH-004, FR-AUTH-007]
---

## §1 — Statement (BCP-14 normative)

1. AUTH JWT signature verification using the active EdDSA (Ed25519) signing key **MUST** complete at **p99 < 1ms** on the production hardware profile (2-vCPU, 4GB RAM container).
2. During the 7-day key-rotation window (when both the new active key AND the prior key are honored — NFR-AUTH-007), verification **MUST** complete at **p99 < 5ms** even in the worst case where the second key attempt is needed.
3. The verification path **MUST NOT** allocate (zero `Vec`/`String` allocations per verify call); the JWKS lookup is a `HashMap<KeyId, VerifyingKey>` reference.
4. JWKS fetches (cold cache, key rotation) are excluded from this SLO — they happen out-of-band on a background refresher.
5. Verification **MUST** be performed via the `ring`-based `ed25519-dalek` crate (production-grade, audited); custom crypto is forbidden.

## §2 — Why this constraint

JWT verify is on every request — to the platform. At platform scale (target 10k req/sec at slice-2), a 10ms verify becomes the bottleneck. EdDSA at < 1ms keeps verify invisible inside the request budget. The 5ms ceiling during rotation accommodates the worst case (new-key verify fails → fall through to old-key verify); even at 5x cost this stays under the admission budget (NFR-AUTH-001) of 50ms. The no-allocation rule prevents GC pressure that would manifest as p99 latency spikes.

## §3 — Measurement

- Histogram `auth_jwt_verify_seconds{key_id, result}` per verify; buckets 0.0001, 0.0005, 0.001, 0.002, 0.005, 0.01, 0.025.
- p99 alarm at > 1ms single-key; p99 alarm at > 5ms during rotation window.
- Counter `auth_jwt_verify_fallback_total` — number of verifies that needed the old key (during rotation); should drop to zero after rotation window closes.

## §4 — Verification

- Criterion benchmark `services/auth/benches/jwt_verify.rs` (T) — 1M verify calls against a fixed key; asserts p99 < 1ms.
- Rotation benchmark `services/auth/benches/jwt_verify_rotation.rs` (T) — simulates rotation window, asserts p99 < 5ms.

## §5 — Failure handling

- p99 > 1ms outside rotation → sev-3; investigate CPU contention on the AUTH pod.
- p99 > 5ms during rotation → sev-2; rotation may be slower than expected; consider shortening rotation window or scaling vertically.
- Verify failures due to clock skew → sev-3; check NTP sync on AUTH and caller pods.

---

*End of NFR-AUTH-003.*
