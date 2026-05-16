---
fr_id: FR-SKILL-108
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

FR-SKILL-108 authored direct-to-10/10. ~720 lines. 14 §1 clauses (10/13-digit MST, GDT checksum + algorithm, SOAP API client, status code interpretation, 24h cache, force_refresh, exp-backoff retry, rate limit 100/min, audit-on-every-validation, log redaction, offline mode, OTel, metrics). 6 §2 rationale paragraphs. Full SKILL.md + Rust API + checksum + SOAP client + audit shape in §3. 23 ACs. 4 checksum + 4 integration tests. 19 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Local vs remote validation order
Network-first wastes quota on typos; local-only misses inactive-but-checksum-valid. Resolved: §1 #2 + #3 two-stage: local checksum first; remote GDT second; no GDT call on checksum failure.

### ISS-002 — Status code "winding down" ambiguity (04)
Reject vs accept? GDT statuses aren't binary. Resolved: §1 #5 + §2 explicit list (00 + 04 = valid for transactions; all else inactive); rationale documented.

### ISS-003 — Cache TTL: too long = stale, too short = wasteful
Resolved: §1 #6 + DEC-212 24h calibrated for same-day churn; FR-INV re-validates before hóa đơn anyway.

### ISS-004 — PII redaction discipline (PDPL 2025)
Without enforcement, raw MSTs leak into logs. Resolved: §1 #11 + §3 `redact()` helper + AC #18 log-redact verification; audit row uses redacted form too.

### ISS-005 — Offline-mode UX
Network outage during onboarding shouldn't halt all KYC. Resolved: §1 #12 `CYBEROS_OFFLINE=true` returns stale cache with `stale: true` flag; caller decides; AC #14 #15.

### ISS-006 — Rate limit enforcement (fail fast)
Sending requests over the 100/min limit triggers GDT-side IP block (15-min penalty). Resolved: §1 #9 governor pre-check returns `RateLimited` immediately; AC #10.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of FR-SKILL-108 audit.*
