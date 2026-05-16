---
fr_id: FR-AUTH-004
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-AUTH-004 expanded from 85 lines to ~870. Added 8 §1 clauses (#5 dual rate-limit, #8 jti bloom dedup, #9 constant-time email lookup, #11 kid in JWT header, #12 agent_persona claim default, #13 scope-map mechanism, #14 suspended subject check, #16 OTel metrics). 8 §2 rationale paragraphs. Full Rust types + signing-key migration + handler skeleton + JWKS + scope_map + verify + rotation in §3. 19 ACs. 8 full Rust test bodies. 21 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — `tenant_id` ambiguity in TokenRequest (email may exist across tenants)
First-pass §6 had `WHERE tenant_id = ? AND email = ?` but the request `{email, password}` doesn't carry tenant_id. Resolved: §1 #4 requires `tenant_slug` field; constant-time slug-not-found returns same shape as bad credentials (preventing tenant enumeration).

### ISS-002 — Key rotation procedure unspecified beyond "quarterly"
First-pass §1 #1 said "rotate signing key quarterly" with no operational mechanism. Resolved: §3 0006_signing_keys.sql migration + `rotation::generate_new_signing_key` + sweep_retired functions; status state machine `active → retiring → retired` with 24h overlap; FR-AUTH-006 schedules cron.

### ISS-003 — `jti` dedup mechanism unspecified
First-pass §10 said "jti recorded; downstream services dedup by jti" without mechanism. Central store? Per-service? Resolved: §1 #8 per-service bloom filter (1MB, ~10⁻⁹ false-positive, 1h rolling); AC #14 + §5 test asserts replay rejection.

### ISS-004 — Rate limit per-IP only; distributed credential stuffing slips through
First-pass §1 #5 had per-IP-only rate limit. Distributed botnet rotating IPs iterates accounts undetected. Resolved: §1 #5 dual rate-limit (per-IP 10/min + per-account 5/min); ACs #6 + #7 + §5 tests for both paths; §2 rationale paragraph.

### ISS-005 — Refresh tokens deferred but no spec hook
First-pass §1 #8 said "FR-AUTH-007 ships the full flow" but didn't define the access-token shape that refresh would extend. Resolved: §1 #15 explicitly notes refresh ships in FR-AUTH-007; access-token shape (1h TTL + jti) is the foundation refresh extends.

### ISS-006 — Suspended subject check missing from §6 skeleton
First-pass §10 row mentioned "Subject suspended → 403" but no implementation in §6. Resolved: §1 #14 normative; §3 handler checks `subject.suspended` before issue; AC #5 + §5 test `suspended_subject_403`; audit row reason `suspended`.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-AUTH-004 audit.*
