---
id: NFR-AI-006
title: "AI Gateway per-tenant cache cross-leak — zero cross-tenant reads under 10k random JWTs"
module: AI
category: security
priority: MUST
verification: T
phase: P0
slo: "Property test: 0 cross-tenant cache hits over 10,000 random JWT pairings"
owner: CTO
created: 2026-05-18
related_frs: [FR-AI-017, FR-AI-018]
---

## §1 — Statement (BCP-14 normative)

1. The AI Gateway prompt/completion cache **MUST** key every entry by `(tenant_id, persona_version, prompt_hash, model_alias)` — never just `prompt_hash`. The `tenant_id` is taken from the verified JWT, not from any caller-supplied parameter.
2. A request from tenant A **MUST** never receive a cached entry written by tenant B, even if the prompt text is byte-identical. This is enforced by the cache lookup signature, not by post-hoc filtering.
3. The cache **MUST** be sharded per-tenant in storage; a Redis key follows the pattern `ai-cache:<tenant_id>:<persona_version>:<prompt_hash>:<model_alias>` and the tenant_id segment is never wildcard-scanned.
4. A property-based test **MUST** generate ≥ 10,000 random (tenant_a, tenant_b, prompt) triples and verify that a write under tenant_a never produces a read hit under tenant_b. Zero false hits permitted.
5. Every cache hit **MUST** emit a structured trace event `ai_gateway.cache.hit` carrying `{tenant_id, persona_version, model_alias, prompt_hash_prefix_8}` — auditable cross-leak detection from logs alone.

## §2 — Why this constraint

Per-tenant isolation is the platform's load-bearing claim. A single cross-tenant cache leak is sufficient to break SOC 2 and PDPL multi-tenancy assertions — even if the prompt contains no PII, the **fact** that tenant A asked a specific question is itself private (e.g., M&A diligence). Indexing the cache by `(tenant_id, ...)` rather than `prompt_hash` makes the cross-leak case **architecturally impossible**, not merely unlikely. The 10k random-pair property test gives statistical confidence; the BRAIN trace event gives detective controls if a future code change breaks the invariant.

## §3 — Measurement

- Property test `services/ai-gateway/tests/cache_cross_tenant_leak_test.rs` (T) runs the 10k-pair generator on every PR; assertion: zero cross-tenant hits.
- Counter `ai_gateway_cache_cross_tenant_attempt_total` — should always be zero. Sev-0 alert on any non-zero increment.
- Trace event `ai_gateway.cache.hit` queryable from OBS; periodic audit verifies `tenant_id` in trace matches `tenant_id` in the calling JWT.

## §4 — Verification

- Property test (T) — CI gate; PR blocked on any cross-tenant hit.
- Penetration test (A) — quarterly external pen-test includes a tenant-A-impersonates-tenant-B scenario; cache must not leak.

## §5 — Failure handling

- Counter `cache_cross_tenant_attempt_total > 0` → sev-0; immediately disable cache reads (gateway falls through to live provider call); page CTO + CSO; halt new tenant onboarding until root cause identified.
- Property test fails in CI → block PR; investigation must complete before any other change merges to that branch.
- BRAIN trace shows `cache.hit` with mismatched JWT tenant_id → sev-0; same response as counter alarm.

---

*End of NFR-AI-006.*
