---
id: NFR-KB-002
title: "KB share-link revocation — revoked link MUST stop working within 60s globally"
module: KB
category: security
priority: MUST
verification: T
phase: P0
slo: "p95 < 60s from revoke action to refused-access at any edge"
owner: CTO
created: 2026-05-18
related_frs: [FR-KB-003]
---

## §1 — Statement (BCP-14 normative)

1. Revoking a KB share link **MUST** propagate to all read paths (origin + CDN + cached link tables) within 60s p95.
2. Revoked-link access attempts **MUST** return HTTP 410 (Gone) with an explanatory body — not 404, since the link existed and is being explicitly denied.
3. Revocation events **MUST** be audited with `{link_id, revoker_id, revoked_at, prior_grants}`.
4. Revocation **MUST NOT** rely on CDN cache TTL alone — explicit purge or signed-token-revocation lists are required.
5. Bulk revocation (e.g., on tenant offboarding) **MUST** complete within 5 minutes regardless of link count.

## §2 — Why this constraint

Share links are public-internet exposure. A revoked link that keeps working for hours (CDN cache lag) is a data-leak vulnerability. The 60s budget is the operational floor — fast enough that the leak window is small but not so fast that the propagation path is fragile. The 410 status is the right "intent" signal; 404 would imply the resource never existed, which is misleading.

## §3 — Measurement

- Histogram `kb_share_link_revocation_propagation_seconds`.
- Counter `kb_share_link_post_revoke_access_attempt_total{age_seconds_since_revoke}`.
- Counter `kb_share_link_bulk_revoke_duration_seconds`.

## §4 — Verification

- Integration test (T) — revoke link, immediately access from multiple edges; assert refused within 60s.
- Chaos test (T) — CDN cache simulated slow; assert revocation still propagates.
- Bulk test (T) — 10k links + bulk revoke; assert ≤ 5min.

## §5 — Failure handling

- Propagation > 60s → sev-2; refresh purge mechanism.
- Post-revoke access succeeds > 60s after revoke → sev-1 (data leak); audit + remediate.
- Bulk revocation > 5min → sev-2; halt new revokes until investigated.

---

*End of NFR-KB-002.*
