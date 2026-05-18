---
id: NFR-DOC-005
title: "DOC IDV method coverage — all 4 declared methods MUST be reachable per tenant"
module: DOC
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of IDV methods (VNeID, eIDAS, AATL, email-link) reachable from each tenant region"
owner: CLO-Legal
created: 2026-05-18
related_frs: [FR-DOC-006]
---

## §1 — Statement (BCP-14 normative)

1. The four declared IDV methods (`vneid`, `eidas`, `aatl`, `email_link`) **MUST** all be reachable + functional for every tenant in supported regions; outages are tracked per method per region.
2. The platform **MUST NOT** silently substitute one method for another — if the requested method is unavailable, the signer is offered explicit alternates.
3. Per-method synthetic probes run hourly per region; per-method availability published.
4. The audit row for any IDV completion records the method actually used; substitutions are visible.
5. Per-method monthly availability < 99% triggers a quarterly product review of that integration.

## §2 — Why this constraint

IDV methods are the platform's identity-assurance toolkit; each has different legal weight + UX. Substituting silently would mislead about assurance level. The 99% availability floor per method is the contractual minimum. Synthetic probes catch silent failures that would otherwise only show up when a user tries to use an unavailable method.

## §3 — Measurement

- Per-method availability gauges (hourly probe + monthly aggregate).
- Counter `doc_idv_completion_total{method, region}`.
- Counter `doc_idv_substitute_offered_total{requested_method, actual_method}`.

## §4 — Verification

- Hourly synthetic per method per region (A).
- Integration test (T) — each method end-to-end sign.
- Monthly availability report; quarterly review for any method < 99%.

## §5 — Failure handling

- Method down per region → alert ops + show alternates to users.
- Silent substitute → sev-2; UI logic bug.
- Sustained low availability → integration retuning + possible deprecation.

---

*End of NFR-DOC-005.*
