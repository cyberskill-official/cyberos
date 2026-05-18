---
id: NFR-HR-009
title: "HR performance history retention — performance signals MUST be retained ≥ 7 years"
module: HR
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of perf-signal rows retained ≥ 7 years; 0 unauthorized deletions"
owner: CHRO
created: 2026-05-18
related_frs: [FR-HR-008]
---

## §1 — Statement (BCP-14 normative)

1. Performance signals (reviews, ratings, 360s, PIPs) **MUST** be retained ≥ 7 years post-member-termination.
2. Performance data **MUST** be readable only by `hr:perf:read` permission holders + the subject member themselves.
3. Deletion **MUST** require dual approval (CHRO + CLO-Legal); single-party deletes are forbidden.
4. Cross-member queries (e.g., "show all 1-star ratings") **MUST** be possible for CHRO + restricted to declared analytics scopes.
5. Subject-member access to own perf history **MUST** be enabled — transparency obligation.

## §2 — Why this constraint

Perf history is the basis of promotion, comp, termination decisions; without retention, disputes can't be defended. 7 years aligns with VN employment-record retention. The access-control + dual-approval-delete combination prevents both unauthorized peeking + retaliatory deletion. Self-view is a transparency baseline (GDPR Art. 15 baseline).

## §3 — Measurement

- Gauge `hr_perf_record_oldest_age_years` — must be ≥ 7 for terminated members.
- Counter `hr_perf_unauthorized_read_attempt_total` — must be 0.
- Counter `hr_perf_delete_attempt_total{has_dual_approval}`.

## §4 — Verification

- Retention scan (T) — daily; assert ≥ 7 years.
- Pen test (T) — unauthorized read attempts.
- Self-view test (T) — member sees own history.

## §5 — Failure handling

- Retention violation → sev-1; halt; restore from backup if possible.
- Unauthorized read → sev-2; audit + investigation.
- Single-approval delete attempt → block + sev-3.

---

*End of NFR-HR-009.*
