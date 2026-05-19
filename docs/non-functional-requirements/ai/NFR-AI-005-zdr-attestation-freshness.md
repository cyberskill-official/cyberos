---
id: NFR-AI-005
title: "ZDR attestation freshness — re-verified weekly per provider"
module: AI
category: privacy
priority: MUST
verification: I
phase: P0
slo: "ZDR attestation last-verified-at age ≤ 7 days per active provider"
owner: CSO
created: 2026-05-18
related_frs: [FR-AI-015, FR-AI-016]
---

## §1 — Statement (BCP-14 normative)

1. For every upstream AI provider used by the gateway, an active **Zero Data Retention** (ZDR) attestation **MUST** be on file under `docs/compliance/zdr-attestations/<provider>/YYYY-MM-DD.{pdf,sha256}`.
2. The attestation **MUST** be re-verified at least **weekly** (every 7 days, calendar-rolling) by an automated check that fetches the provider's current ZDR statement endpoint OR verified manually by CSO with a sign-off entry committed to `docs/compliance/zdr-attestations/verification-log.md`.
3. The gateway's provider registry (`services/ai-gateway/src/providers/registry.toml`) **MUST** carry a `zdr_attestation_last_verified_at` ISO-8601 timestamp per provider, refreshed on each verification.
4. A provider whose `zdr_attestation_last_verified_at` is older than 8 days (24h grace beyond the weekly window) **MUST** be auto-removed from active routing — the gateway returns HTTP 503 for any route targeting that provider until re-verification.
5. Every verification (success or failure) **MUST** emit a memory audit row `compliance.zdr_verification` with `{provider, verified_at, verifier, attestation_hash, source_url}`.

## §2 — Why this constraint

ZDR is the platform's load-bearing contractual claim for enterprise tenants — "your prompts never train the upstream model." Without a freshness check, a provider could silently update their ToS to drop the ZDR clause and the platform would keep routing traffic for weeks before anyone noticed. The 7-day cadence balances operational cost (one CSO touch per week per provider) against the regulatory exposure window (max 8 days of routing under stale terms). The auto-remove behaviour ensures **default safety** — if the check is somehow missed, the provider drops out rather than continues operating under unverified terms.

## §3 — Measurement

- Cron job `deploy/compliance/zdr-attestation-check.sh` runs nightly; for each provider it fetches the current ZDR URL, computes SHA-256, compares to the on-file attestation hash, and updates `zdr_attestation_last_verified_at` if the hash matches.
- Gauge `compliance_zdr_attestation_age_seconds{provider}` exposed to OBS; alert fires at > 7d (sev-3 warning) and > 8d (sev-2 auto-remove).
- memory query `view kind=compliance.zdr_verification` returns the full verification log; sortable by provider.

## §4 — Verification

- Inspection (I) — quarterly internal audit verifies the `verification-log.md` is current and every active provider has a fresh attestation row.
- CI smoke `tests/compliance/zdr_attestation_freshness_test.sh` (T) — runs in nightly compliance CI, asserts no provider in active routing has age > 8d.

## §5 — Failure handling

- Age > 7d → sev-3, ticket auto-opened to CSO; investigation must complete within 24h.
- Age > 8d → sev-2, provider auto-dropped from routing pool; sev-2 page to on-call; CSO has 4h to either re-verify (close ticket) or escalate to CEO if provider has materially changed terms.
- ZDR attestation hash diverged from on-file (provider silently changed the page) → sev-1; immediate halt of all routes targeting that provider; CSO + Legal review the new attestation before re-enabling.

---

*End of NFR-AI-005.*
