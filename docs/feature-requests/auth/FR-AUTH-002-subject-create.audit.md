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

## §10 — Implementation audit (code-vs-spec)

> Added 2026-05-18 (session 21) by `chief-technology-officer/implement-backlog-frs` workflow.

### §10.1 — Verdict

**Implementation status:** BLOCKED. **11 of 14 §1 normative clauses have spec-vs-code gaps** — the highest drift rate observed in the audit-fix loop to date. `create_subject` (services/auth/src/handlers.rs:325-410) ships the happy path (bcrypt + HIBP + INSERT) but is missing email-regex validation, password complexity rules (length / char-classes / breach-list embedding), role allow-list, Idempotency-Key support, BRAIN audit row emission, structured 4xx bodies, HTTPS gate, single-transaction discipline for HIBP audit, OTel span, and metrics. The HIBP audit row insert happens BEFORE the transaction begin, violating §1 #12's atomicity requirement (a failed subject INSERT leaves an orphan HIBP audit row).

### §10.2 — Gap list

| # | Spec ref | Gap | Severity | Effort |
|---|---|---|---|---|
| G-001 | §1 #2 | Email regex `^[^@\s]+@[^@\s]+\.[^@\s]+$` not validated; any string accepted | medium | ~15 LOC |
| G-002 | §1 #4 | Password complexity (12..=128 chars · 3-of-4 char classes · not-email-localpart · not-in-top-10K-list) not enforced | high | ~80 LOC + embedded breach list (~80KB compressed) |
| G-003 | §1 #5 | Role allow-list (`{tenant-admin, tenant-member}` slice-1) not validated; any role string accepted | high | ~25 LOC |
| G-004 | §1 #6 | Idempotency-Key header not honoured | medium | ~30 LOC (reuse idempotency module from FR-AUTH-001) |
| G-005 | §1 #7 | `auth.subject_created` BRAIN audit row NOT emitted (no `email_hash16` either) | high | ~40 LOC (extend brain_bridge with SubjectCreatedPayload) |
| G-006 | §1 #9 | 409 email-taken returns generic 500 (UNIQUE violation not caught) | medium | ~10 LOC |
| G-007 | §1 #10 | 200 ms p95 SLO not asserted by any test | medium | ~60 LOC (new integration test) |
| G-008 | §1 #11 | HTTPS-required check (`X-Forwarded-Proto: https`) absent | medium | ~20 LOC |
| G-009 | §1 #12 | HIBP audit insert lives OUTSIDE the subject-create tx → orphan rows on subject failure | critical | restructure: move HIBP audit into the tx OR after commit |
| G-010 | §1 #13 | OTel `auth.create_subject` span absent | medium | ~30 LOC (mirror FR-AUTH-001 G-004 pattern) |
| G-011 | §1 #14 | OTel metrics (`auth_subject_create_total/latency_ms/count`) absent | low | ~15 LOC |
| G-012 | §1 #1 | Cross-tenant creation guard not visible at handler layer (relies on RLS at DB) — needs explicit handler-level `caller.tenant_id == req.tenant_id` check | high | ~15 LOC |
| G-013 | §1 #3 | Password not zeroised after hashing — no `zeroize` crate dep | medium | add `zeroize` dep + wrap pw in `Zeroizing<String>` |
| G-014 | §new_files | `admin_subject_create_test.rs` declared in `frontmatter.new_files` but absent on disk | high | ~250 LOC across multiple ECM-row tests |

### §10.3 — Audit-fix log

| ts | gap | change | tests | cargo result | commit |
|---|---|---|---|---|---|
| _empty — slice-1 fix loop not yet started — see §10.7 for slicing plan_ | | | | | |

### §10.4 — BACKLOG.md mutations

| ts | line | from | to | mutation_kind |
|---|---|---|---|---|
| 2026-05-18T18:30:00Z | 213 | `planned` | `[BLOCKED: 14 spec gaps documented in FR-AUTH-002-subject-create.audit.md §10]` | status-cell-only |
| _pending audit-fix completion_ | 213 | (above) | one of `slice-1 shipped …` / `shipped + strict-audited` | status-cell-only |

### §10.5 — Working notes

**Code state at audit time:**
- `services/auth/src/handlers.rs::create_subject` (lines 325-410) takes `Extension<Claims>` (G-003 authz-middleware pattern applies)
- bcrypt::DEFAULT_COST (cost 12) is used per §1 #3 (cost OK; zeroize missing per G-013)
- HIBP breach check via `crate::hibp::check_password` (FR-AUTH-107) runs before bcrypt; returns 409 if breached
- HIBP audit row is INSERTed before the subject-create transaction → §1 #12 atomicity violation (G-009)
- Subject INSERT happens inside a tx with `SET LOCAL app.current_tenant_id` GUC for RLS
- Response shape matches §1 #8 (no password hash leaked)
- No idempotency, no BRAIN audit row, no OTel, no structured 4xx bodies

### §10.7 — Slicing plan

FR-AUTH-002 has the highest gap count of any audited FR. Three slices recommended:

**Slice 1 — security + observability foundations** (~250 LOC; estimated 1 working day):
- G-001 email regex validation
- G-003 role allow-list
- G-004 Idempotency-Key honoured (reuse FR-AUTH-001's idempotency module)
- G-005 `auth.subject_created` BRAIN audit row (extend brain_bridge.rs with `SubjectCreatedPayload`)
- G-006 structured 409 email_taken body
- G-009 move HIBP audit INTO the subject-create tx OR after commit (atomicity)
- G-010 OTel span `auth.create_subject`
- G-012 handler-level cross-tenant guard (defence-in-depth on top of RLS)

**Slice 2 — password hardening** (~150 LOC + ~80KB breach list; estimated half-day):
- G-002 password complexity rules
- G-013 zeroize wrapping
- Embed top-10K-passwords compressed list

**Slice 3 — production guardrails + tests** (~330 LOC; estimated 1 working day):
- G-008 HTTPS-required check
- G-011 OTel metrics
- G-007 200ms p95 SLO test
- G-014 full `admin_subject_create_test.rs` covering all 14 §1 clauses + ECM rows

**Cumulative slice effort:** ~2.5 working days. Matches the original `effort_hours: 8` × the discovered drift ratio.

### §10.8 — Why deferring all 3 slices

FR-AUTH-002 is end-user-facing in the production path (every human/agent that authenticates eventually has a subject row). The 14-gap drift means current behaviour silently violates spec on email format, role assignment, password strength, idempotency, atomicity, and observability simultaneously. Shipping slice-1 piecemeal (e.g. just G-001 + G-006) leaves the harder gaps (G-002 password complexity, G-009 atomicity) latent — and those are the security-load-bearing ones. A focused dedicated session for slice-1 followed by separate slice-2 + slice-3 commits is the cleanest path. Session 22 candidate.

---

*End of FR-AUTH-002 audit. Spec quality: PASS 10/10. Implementation: BLOCKED — 14 gaps documented, 3-slice plan in §10.7 (~2.5 working days total). Highest drift rate of any audited FR; recommend dedicated session.*
