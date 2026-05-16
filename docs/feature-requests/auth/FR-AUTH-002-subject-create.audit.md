---
fr_id: FR-AUTH-002
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

FR-AUTH-002 expanded from 80 lines to ~700. Added 7 §1 clauses (#4 password complexity rules, #6 idempotency, #7 audit-row PII discipline with email_hash16, #11 HTTPS-required, #12 transaction atomicity, #13 OTel span without PII, #14 metrics), 8 §2 rationale paragraphs, full Rust types + migration + role allow-list + password validation + handler in §3, expanded §4 from 8 to 17 ACs, full Rust test bodies in §5 (8 tests covering happy/cross-tenant/weak-password/breach-list/bcrypt-format/audit-no-PII/p95/RLS), 21 failure modes in §10, 8 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Password complexity rules unspecified
First-pass had no minimum length, no breach-list check, no character-class requirements. Resolved: §1 #4 specifies 12-128 chars + 3-of-4 character classes + no email-localpart + top-10K-breach-list check; `password.rs` shows the implementation; AC #7-#10 + §5 tests cover each path.

### ISS-002 — bcrypt cost 12 hardcoded; no config
First-pass said "bcrypt (cost 12)" with no rationale or config knob. Resolved: §1 #3 cites DEC-115 + NIST SP 800-63B; §2 explains the 10-vs-12-vs-14 trade-off; cost change requires FR amendment.

### ISS-003 — `password` plaintext in request — no transport-encryption requirement
First-pass had no HTTPS requirement. Plaintext password over HTTP is credentials-on-the-wire. Resolved: §1 #11 HTTPS-required check via `X-Forwarded-Proto`; AC #11 + §10 row.

### ISS-004 — Audit row didn't explicitly forbid password fields OR plaintext email
First-pass §1 #6 said "emit BRAIN audit row `auth.subject_created`" without specifying payload. Resolved: §1 #7 explicitly forbids password + plaintext email; mandates `email_hash16` (SHA-256[..16]); AC #14 + §5 test asserts no PII in audit JSON.

### ISS-005 — Roles validation: slice 1 only allows 2 roles but no enum/registry
First-pass §1 #5 mentioned "tenant-admin, tenant-member" but no allow-list constant. Resolved: `roles.rs` with `SLICE_1_ALLOWED_ROLES` constant + `validate_role_slice1` helper; AC #6 + §5 test.

### ISS-006 — Idempotency missing (mirrors FR-AUTH-001 ISS-004)
Network retries during subject create produce duplicates. Resolved: §1 #6 idempotency-key handling with same semantics as FR-AUTH-001; AC #15 + §5 idempotent-replay test.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-AUTH-002 audit.*
