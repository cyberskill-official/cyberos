---
fr_id: FR-AUTH-005
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-AUTH-005 expanded from 73 lines to ~830. Added 8 §1 clauses (#4 unrevoke, #5 cursor + HMAC sign, #8 idempotency, #9 cursor validation, #10 sessions table, #11 Redis pub/sub propagation, #12 deny-list-not-cleared-on-unrevoke, #13 sessions in RLS registry, #14 include_suspended filter). 7 §2 rationale paragraphs. Full Rust types + handlers + cursor module + sessions migration + deny_list module in §3. 19 ACs. 7 full Rust test bodies. 17 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Deny-list mechanism conflicts with FR-AUTH-004 §1 #8 jti dedup
First-pass §6 used Redis as deny-list while FR-AUTH-004 §1 #8 specified per-service bloom filter for jti dedup. These are different concerns (replay vs revocation) but the spec didn't distinguish. Resolved: §1 #11 explicitly establishes deny-list as the revocation primitive; bloom is replay-only; both consulted during JWT verify but for different reasons.

### ISS-002 — No idempotency on revoke/unrevoke
Operator double-click produces duplicate audit rows. Resolved: §1 #8 Idempotency-Key support mirrors FR-AUTH-001 §1 #5; ACs #16 + #17.

### ISS-003 — Page cursor format unspecified
First-pass §1 #4 mentioned "opaque cursor" but no encoding, no signing. Cursors could be tampered to fish for other tenants' data. Resolved: §1 #5 + #9 + cursor.rs module with HMAC-signed base64 cursors; AC #10 + §5 tampering test; AC #11 stable-under-concurrent-insert test.

### ISS-004 — Cross-tenant blocked at API but not RLS-confirmed
First-pass had API check only. Defense in depth requires RLS too — added `sessions` to TENANT_SCOPED_TABLES; AC #3 confirms RLS catches API bypass.

### ISS-005 — Revoke audit row missing reason field
Operators want to record WHY revoke happened (compromised, terminated, etc.). Resolved: §1 #6 row payload includes optional `reason` field; §9 lists reason taxonomy as deferred to slice 3.

### ISS-006 — No unrevoke path
First-pass had revoke only. Real ops needs reversibility (mistaken revoke). Resolved: §1 #4 unrevoke endpoint + §3 handler + AC #7 + #8 + audit row + §1 #12 deny-list-not-cleared-on-unrevoke security default.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-AUTH-005 audit.*
