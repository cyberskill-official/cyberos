---
# ───── Machine-readable frontmatter (parsed by task-audit + future fr-catalog renderer) ─────
id: TASK-AI-017
title: "Per-tenant Redis response cache keyed by (tenant × redacted-prompt × model × persona); ≥30% hit-rate P0 target"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: AI
priority: p1
status: done
verify: T
phase: P0
milestone: P0 · slice 4
slice: 4
owner: Stephen Cheng
created: 2026-05-15
shipped: 2026-05-21
memory_chain_hash: null
related_tasks: [TASK-AI-001, TASK-AI-002, TASK-AI-008, TASK-AI-010, TASK-AI-011, TASK-AI-014, TASK-AI-018]
depends_on: [TASK-AI-008]
blocks: [TASK-AI-018]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#cache
  - website/docs/modules/ai.html#bigger-picture
source_decisions:
  - Cost-of-everything-gate efficiency target: 30% hit-rate brings $4/user/month → $2.80/user/month at break-even
  - DEC-082 (cache key MUST include persona version; persona changes invalidate cache automatically)
  - DEC-085 (per-tenant Redis namespace; cross-tenant reads MUST be impossible by construction)
  - archive/2026-05-14/RESEARCH_REVIEW.md §5.1 (Redis vs. in-process cache trade-off; Redis wins on multi-instance + restart-survival)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/cache/mod.rs
  - services/ai-gateway/src/cache/key.rs
  - services/ai-gateway/src/cache/ttl.rs
  - services/ai-gateway/src/cache/redis_backend.rs
  - services/ai-gateway/tests/cache_test.rs
  - services/ai-gateway/tests/cache_isolation_property_test.rs
  - services/ai-gateway/tests/cache_ttl_test.rs
  - services/ai-gateway/tests/cache_ttl_test.rs
  - services/ai-gateway/tests/cache_isolation_property_test.rs
  - services/ai-gateway/docker/redis/redis.conf                      # production-tuned config (maxmemory + LRU policy)
modified_files:
  - services/ai-gateway/src/handlers/chat.rs                         # check cache before router::call_provider
  - services/ai-gateway/src/cost_ledger.rs                           # precheck reads cache_state to decide audit-row payload
  - services/ai-gateway/src/memory_writer.rs                          # canonical builders include cache_state field
  - services/ai-gateway/Cargo.toml                                   # redis@0.24, hex@0.4
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,docker}/**
  - bash: cd services/ai-gateway && cargo test cache
  - bash: docker run -d --name test-redis -p 6379:6379 redis:7
disallowed_tools:
  - cache across tenants (cache key MUST start with tenant_id; cross-tenant reads are correctness-load-bearing per §1 #2 and exhaustively tested in TASK-AI-018)
  - cache the RAW (un-redacted) prompt anywhere — the redacted prompt is the only legitimate cache key
  - cache streaming responses (TASK-AI-010 streaming defeats caching anyway; §1 #5)
  - cache failed responses (any non-200 from provider; §1 #6)
  - hardcode TTL values inside `redis_backend.rs` (TTLs live in `ttl.rs` — single source of truth per §1 #4)
  - bypass `cost_ledger::precheck` even on cache hits (audit-before-action invariant from TASK-AI-001 §1 #6)

# ───── Estimated work ─────
effort_hours: 8
subtasks:
  - "0.5h: CacheKey type + cryptographic key-derivation (SHA-256 of canonical input tuple)"
  - "0.5h: TTL table per alias-class in ttl.rs (chat.fast / chat.smart / chat.long no-cache / embed.* / rerank.*)"
  - "0.5h: Per-key TTL with 10% jitter (avoid thundering-herd expiry)"
  - "1.0h: cache::lookup(key) → Option<CachedResponse> with Redis backend"
  - "1.0h: cache::insert(key, response, ttl) — schema-versioned serialisation"
  - "0.5h: Cache schema version (`v1` prefix in serialised payload; mismatch → treat as miss)"
  - "0.5h: Maximum payload size (1MB per entry; over-size responses NOT cached, log WARN)"
  - "0.5h: Per-tenant size cap (100MB) + LRU eviction via Redis maxmemory + allkeys-lru"
  - "0.5h: Hit-rate metric per tenant + sev-3 alarm at <30% over 7-day rolling window"
  - "0.5h: ai.invocation memory row carries cache_state field (TASK-AI-002 §3 declares it)"
  - "0.5h: Cache warming hook at boot (no-op for slice 4; documented for future)"
  - "0.5h: Redis-unreachable graceful degradation (lookup returns None; insert silently drops)"
  - "0.5h: Tests: 16 ACs covering hit/miss/cross-tenant/TTL/streaming-skip/failure-skip/property-test/workload-sim"
risk_if_skipped: "Every call hits the LLM provider. At 100k calls/month × $0.001/call avg = $100/tenant/month — well above the $4/user target. Cost-of-everything gate becomes economically unsustainable; CyberOS unit economics break before 100 customers. Worse: the per-tenant cache is a dependency of TASK-AI-018 (cross-tenant leak test); without this task, TASK-AI-018's tests can't even be authored."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **SHOULD** cache successful LLM responses in Redis keyed by `(tenant_id, redacted_prompt, model, persona_handle, cache_schema_version)` to reduce duplicate-call cost. The cache and surrounding contract obey the following:

1. **MUST** derive the cache key as `SHA-256(canonical_concat(tenant_id, redacted_prompt, model, persona_handle))` where `redacted_prompt` is the post-redaction text from TASK-AI-011 (NEVER the raw prompt — caching raw would create a parallel PII repository defeating redaction). `canonical_concat` joins fields with a unit-separator (`\x1f`) to prevent ambiguity (e.g., `tenant_a + "model" = tenant_amodel` collision attack). The hash output is 32 bytes; the Redis key uses its hex encoding.
2. **MUST** isolate per tenant by construction: every Redis key has the prefix `ai_cache:v1:{tenant_id}:`. A `KEYS ai_cache:v1:tenant_a:*` scan MUST never return a `tenant_b` entry. The tenant prefix is a structural invariant — not a runtime check; the key derivation makes cross-tenant reads impossible without direct Redis surgery. TASK-AI-018 will exhaustively test this property.
3. **MUST** include the `persona_handle` (full `<id>@<version>` from TASK-AI-014) in the key derivation. A persona change MUST invalidate cache automatically — `cuo-cpo@0.4.1` and `cuo-cpo@0.4.2` are distinct keys. This prevents serving an old-persona response to a request that selected a new persona; the keys diverge naturally.
4. **MUST** apply TTLs per alias-class as defined in `cache/ttl.rs`:
   - `chat.fast` → 1 hour.
   - `chat.smart` → 30 minutes.
   - `chat.long` → no cache (skip insert; lookup always returns None).
   - `embed.standard` → 24 hours.
   - `embed.code` → 24 hours.
   - `rerank.fast` → 15 minutes.
   New alias classes added to TASK-AI-006 MUST extend `ttl.rs`; an alias with no TTL entry is treated as `chat.long` semantics (no cache) with a WARN log so the operator notices.
5. **MUST** apply per-entry TTL jitter of ±10% to prevent thundering-herd expiry. Mechanically: actual_ttl = nominal_ttl × (1 + uniform(-0.1, 0.1)). The jitter is per-insert (deterministic for testability via a seedable RNG; production seeds from system entropy).
6. **MUST NOT** cache responses on streaming calls (TASK-AI-010 streaming returns chunks, not a full response). The cache is bypassed entirely on the streaming code path; the handler decides at request time based on `req.stream == true`.
7. **MUST NOT** cache failed responses (any non-200 from provider). A retry-after-failure path that hit cache would mask transient provider errors as deterministic answers. The handler only calls `cache::insert` on the success path.
8. **MUST NOT** cache responses larger than 1MB serialised (1,048,576 bytes). Over-size payloads bypass `cache::insert` with a WARN log + metric `ai_cache_oversize_skipped_total{tenant_id}`. The 1MB cap protects against pathological responses (huge JSON arrays from rerank) chewing through the per-tenant 100MB budget on a few entries.
9. **MUST** call `cost_ledger::precheck` (TASK-AI-001) BEFORE consulting the cache. Cache hits are still cost-checked but cost zero; the audit row carries `estimated_usd: 0`, `cache_state: hit`, `cache_lookup_latency_ms: <n>`. The audit-before-action invariant from TASK-AI-001 §1 #6 applies — every call (cache hit OR miss) emits exactly one `ai.precheck` row before the gateway returns to the client.
10. **MUST** emit `cache_state` field in both `ai.precheck` (TASK-AI-001 modified) and `ai.invocation` (TASK-AI-002 §3 already declares it) memory rows. Values: `hit | miss | skipped | error`. `skipped` covers streaming + chat.long + oversize. `error` covers Redis unreachable / serialisation mismatch.
11. **MUST** target ≥30% hit rate measured per tenant over a 7-day rolling window. Hit rates below 30% trigger OBS sev-3 alarm `ai_cache_hit_rate_low{tenant_id}`. The 30% floor is the break-even point for the $4 → $2.80/user/month cost target; falling below means the cache is contributing less value than its operational cost.
12. **MUST** size-cap per tenant at 100MB via Redis `maxmemory-policy allkeys-lru` with key-pattern selection. The LRU eviction is Redis-managed (no app-level sweeper); operator dashboards show eviction rate per tenant via `ai_cache_evictions_total{tenant_id, reason=lru}`.
13. **MUST** include a cache schema version (`v1` in slice 4) in the Redis key prefix AND in the serialised payload's first 4 bytes. A schema-version mismatch on lookup MUST be treated as a miss (NOT a deserialisation error). This allows future schema migrations (e.g., adding a `model_version` field to `CachedResponse`) without flushing the cache — old entries simply expire.
14. **MUST** gracefully degrade on Redis unavailability: connection errors, timeouts (>200ms), and serialisation errors all return `cache_state: error` to the caller; the handler proceeds as if it were a miss (call provider). The metric `ai_cache_errors_total{reason}` (reason ∈ unreachable | timeout | deserialisation | oversize) tracks each path; sustained errors > 1% trigger sev-2 alarm.
15. **SHOULD** emit OTel metrics:
    - `ai_cache_lookups_total{tenant_id, alias_class, outcome}` (counter; outcome ∈ hit | miss | skipped | error).
    - `ai_cache_hit_rate{tenant_id}` (gauge; 7-day rolling).
    - `ai_cache_size_bytes{tenant_id}` (gauge; from Redis MEMORY USAGE).
    - `ai_cache_evictions_total{tenant_id, reason}` (counter; reason ∈ ttl | lru).
    - `ai_cache_errors_total{tenant_id, reason}` (counter).
    - `ai_cache_oversize_skipped_total{tenant_id}` (counter).
    - `ai_cache_lookup_latency_ms` (histogram; p99 SLO 5ms).
16. **SHOULD** support cache-warming hooks at boot for known-stable prompts (e.g., the persona system prompts themselves trigger common opening-summary calls). Slice 4 ships a no-op stub; TASK-AI-022 implements warming once we have hit-rate data showing where it pays off.

---

## §2 — Why this design (rationale for humans)

**Why Redis, not in-process Rust HashMap?** Three reasons. (1) The gateway runs as multiple instances behind a load balancer; an in-process cache wouldn't share state across instances, killing hit rate. (2) An in-process cache evaporates on restart; the warmup cost would dominate. (3) Redis is the standard answer for this exact problem with well-understood eviction (LRU), TTL, and operational tooling. The in-process latency advantage (~5µs vs. ~500µs for Redis) is irrelevant — the LLM provider call is hundreds of milliseconds.

**Why hash of REDACTED prompt (§1 #1)?** Caching the raw prompt would create a parallel PII repository inside Redis — defeating TASK-AI-011's redaction work. The redacted prompt (`<VN_CCCD_1>`-style placeholders replacing identifiers) is the legitimate cache key: it's what the LLM actually sees, and it's what determines whether two requests are functionally equivalent. Two callers passing different real CCCDs that get redacted to the same placeholder pattern will hit the same cache entry — and that's correct, because the LLM's output for both is identical.

**Why per-tenant isolation (§1 #2)?** Multiple reasons converge here. (1) Tenant policies differ: tenant A might require ZDR; tenant B might allow non-ZDR — caching across them would cross the compliance boundary. (2) Persona handles differ: tenant A's `cuo-cpo@0.4.1` may produce different outputs from tenant B's `cuo-cpo@0.4.1` if downstream tools differ. (3) Audit-trail integrity: every cache hit is attributed to a tenant in the memory row; cross-tenant reads would corrupt the attribution. The per-tenant prefix in the Redis key is a structural enforcement (not a runtime check) — even a code bug that miscalculated the key would still produce keys starting with the wrong tenant prefix, never reading the wrong tenant's data.

**Why include persona_handle in the key (§1 #3)?** A persona is a system-prompt modifier — different personas produce different outputs from the same user prompt. Caching across personas would serve `cuo-cfo@0.4.1`'s answer to a `cuo-cto@0.4.1` request. Including the handle makes this impossible by construction. The DEC-082 decision was explicit: persona-version changes auto-invalidate cache; no operational sweeper needed.

**Why TTL jitter (§1 #5)?** Without jitter, all entries inserted in the same minute expire in the same minute — producing a thundering-herd refresh-cost spike at expiry time. The 10% jitter spreads the expiry distribution and smooths provider load. The deterministic-via-seedable-RNG property allows tests to assert TTL ranges (`actual_ttl in [nominal × 0.9, nominal × 1.1]`).

**Why no cache on streaming (§1 #6)?** Streaming responses are inherently single-use — the client consumes them token-by-token. Caching the assembled string and replaying it on subsequent requests would reintroduce the latency overhead the streaming was designed to avoid. There's no good interaction between streaming UX and caching; we explicitly bypass the cache rather than try to be clever.

**Why no cache on failed calls (§1 #7)?** A retry hitting cache would replay the failure, masking transient errors as deterministic. Worse: a flaky provider's intermittent 500 responses could pollute cache with garbage that takes the entire TTL to clear. Caching only successful responses is the correctness floor.

**Why a 1MB per-entry cap (§1 #8)?** Pathological responses (huge JSON arrays from rerank, full-document embeddings) can be megabytes. A few such responses chew through the per-tenant 100MB budget. The 1MB cap is conservative — typical chat responses are 1-10KB; the cap only fires on outliers. The metric `ai_cache_oversize_skipped_total` lets operators investigate unusually-large responses (often a sign of a bug — e.g., an unbounded list response).

**Why precheck BEFORE cache lookup (§1 #9)?** The cost-of-everything invariant says EVERY call goes through the gate. A cache hit is technically free in dollars but the audit row + persona check + ZDR check still apply — these are the compliance primitives. Skipping precheck on cache hits would create an "untracked call" path; the memory audit chain would show fewer rows than actual calls. The cost is one extra precheck per call (~50ms) but the row count is honest.

**Why a cache schema version (§1 #13)?** Cache entries are serialised `CachedResponse` structs. If we add a field to that struct (e.g., `model_version: String` for TASK-AI-022), old entries deserialise as missing-field errors. Without a schema-version prefix, the deserialisation error path is brittle (might panic, might log, might silently miss). With a `v1` prefix, the lookup function checks the version first and treats mismatches as misses — clean miss-then-refresh behaviour. The `v1` prefix is in BOTH the Redis key (for KEYS-pattern compatibility) AND the payload's first 4 bytes (for serialisation defence).

**Why graceful degradation on Redis unavailability (§1 #14)?** Redis is operational infrastructure; outages happen. A hard-fail on Redis-down would take down the entire gateway. Returning `cache_state: error` and proceeding to the provider (just slower; no cache hit) preserves availability. The error metric tracks the outage duration; sustained errors > 1% page on-call. The design separates "cache helpful" (when Redis is up) from "cache critical" (when Redis is down) — the cache should be the former, never the latter.

**Why a 30% hit-rate floor (§1 #11)?** This is the break-even point for the $4 → $2.80/user/month target. Below 30%, the cache contributes less value than its operational cost (Redis hosting, network round-trips, eviction processing). The sev-3 alarm is informational (operator investigates: is the tenant's prompt diversity unusually high? Is the cache cold from a recent restart? Did a persona change wipe the cache?). Below 10% sustained → operator should consider disabling the cache for that tenant entirely.

**Why does cache-key derivation use unit-separator (`\x1f`) and not just concatenation (§1 #1)?** Without a separator, `concat(tenant_a, model_xyz)` and `concat(tenant_amodel, _xyz)` produce the same hash input. A tenant id like `org:cyberos.world` mixed with a model `chat.smart` could collide in unforeseen ways. The unit-separator (a control character that can't appear in a tenant id, model name, or persona handle by validation rules elsewhere) makes collisions structurally impossible — the SHA-256 still does its job, but we're defending against weird-input cases at the input layer.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Type definitions

```rust
// services/ai-gateway/src/cache/mod.rs

use std::time::Duration;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const CACHE_SCHEMA_VERSION: &str = "v1";
pub const MAX_PAYLOAD_BYTES: usize = 1_048_576;          // 1 MB
pub const PER_TENANT_BUDGET_BYTES: usize = 100 * 1_048_576;  // 100 MB
pub const REDIS_TIMEOUT_MS: u64 = 200;
pub const HIT_RATE_FLOOR: f64 = 0.30;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub tenant_id: String,
    pub prompt_hash: [u8; 32],            // SHA-256 of (tenant_id, redacted_prompt, model, persona_handle)
}

impl CacheKey {
    /// §1 #1: cryptographic key derivation with unit-separator-joined inputs.
    pub fn derive(
        tenant_id: &str, redacted_prompt: &str, model: &str, persona_handle: &str,
    ) -> Self {
        use sha2::{Sha256, Digest};
        let mut h = Sha256::new();
        h.update(tenant_id.as_bytes());
        h.update(b"\x1f");
        h.update(redacted_prompt.as_bytes());
        h.update(b"\x1f");
        h.update(model.as_bytes());
        h.update(b"\x1f");
        h.update(persona_handle.as_bytes());
        Self {
            tenant_id: tenant_id.into(),
            prompt_hash: h.finalize().into(),
        }
    }

    /// §1 #2 + §1 #13: per-tenant prefix + schema version.
    pub fn redis_key(&self) -> String {
        format!("ai_cache:{}:{}:{}", CACHE_SCHEMA_VERSION, self.tenant_id, hex::encode(self.prompt_hash))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    pub schema_version: String,            // §1 #13: must match CACHE_SCHEMA_VERSION on load
    pub usage: ProviderUsage,
    pub choices: Vec<Choice>,
    pub finish_reason: FinishReason,
    pub cached_at: DateTime<Utc>,
    pub provider_ms: u64,                  // original latency for observability
}

#[derive(Debug, Clone, PartialEq)]
pub enum CacheState { Hit, Miss, Skipped, Error }

pub async fn lookup(key: &CacheKey) -> CacheLookupOutcome;
pub async fn insert(key: &CacheKey, response: &ProviderResponse, alias: &str) -> CacheInsertOutcome;

pub enum CacheLookupOutcome {
    Hit(Box<CachedResponse>, /* lookup_latency */ Duration),
    Miss,
    SchemaMismatch,                        // treat as Miss; metric increments
    Error(CacheError),
}

pub enum CacheInsertOutcome {
    Inserted { ttl: Duration, jittered_ttl: Duration },
    Skipped(SkipReason),
    Error(CacheError),
}

pub enum SkipReason { ChatLongOrUnknownAlias, Streaming, FailedResponse, Oversize { actual_bytes: usize } }

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("redis unreachable: {0}")]
    Unreachable(String),
    #[error("redis timeout (>200ms)")]
    Timeout,
    #[error("deserialisation failed: {0}")]
    Deserialisation(String),
}
```

### TTL table

```rust
// services/ai-gateway/src/cache/ttl.rs

use std::time::Duration;

/// §1 #4: TTL per alias-class. Single source of truth.
/// New aliases added to TASK-AI-006 MUST extend this map; missing-alias → no-cache + WARN.
pub fn ttl_for_alias(alias: &str) -> Option<Duration> {
    match alias_class(alias) {
        "chat.fast"        => Some(Duration::from_secs(3600)),
        "chat.smart"       => Some(Duration::from_secs(1800)),
        "chat.long"        => None,                                  // no cache
        "embed.standard"   => Some(Duration::from_secs(86400)),
        "embed.code"       => Some(Duration::from_secs(86400)),
        "rerank.fast"      => Some(Duration::from_secs(900)),
        _                  => {
            tracing::warn!(alias = %alias, "ttl_for_alias: unknown alias class; treating as no-cache");
            None
        }
    }
}

fn alias_class(alias: &str) -> &str {
    // alias is like "chat.smart" or "chat.smart-resolved-bedrock-claude-..."
    alias.split('-').next().unwrap_or(alias)
}

/// §1 #5: ±10% jitter to prevent thundering-herd expiry.
pub fn jittered_ttl(nominal: Duration, rng: &mut impl rand::Rng) -> Duration {
    let factor = 1.0 + rng.gen_range(-0.1..0.1);
    Duration::from_secs_f64(nominal.as_secs_f64() * factor)
}
```

### Lookup + insert

```rust
// services/ai-gateway/src/cache/redis_backend.rs

use redis::AsyncCommands;
use tokio::time::timeout;

pub async fn lookup(key: &CacheKey) -> CacheLookupOutcome {
    let t0 = std::time::Instant::now();
    let mut conn = match timeout(Duration::from_millis(REDIS_TIMEOUT_MS), get_conn()).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            metrics::error("unreachable");
            return CacheLookupOutcome::Error(CacheError::Unreachable(e.to_string()));
        }
        Err(_) => {
            metrics::error("timeout");
            return CacheLookupOutcome::Error(CacheError::Timeout);
        }
    };

    let raw: Result<Option<Vec<u8>>, _> = conn.get(key.redis_key()).await;
    let bytes = match raw {
        Ok(Some(b)) => b,
        Ok(None) => {
            metrics::lookup(&key.tenant_id, "miss", t0.elapsed());
            return CacheLookupOutcome::Miss;
        }
        Err(e) => return CacheLookupOutcome::Error(CacheError::Unreachable(e.to_string())),
    };

    // §1 #13: schema-version check on load
    match serde_json::from_slice::<CachedResponse>(&bytes) {
        Ok(cr) if cr.schema_version == CACHE_SCHEMA_VERSION => {
            metrics::lookup(&key.tenant_id, "hit", t0.elapsed());
            CacheLookupOutcome::Hit(Box::new(cr), t0.elapsed())
        }
        Ok(_) => {
            metrics::lookup(&key.tenant_id, "schema_mismatch", t0.elapsed());
            CacheLookupOutcome::SchemaMismatch
        }
        Err(e) => {
            metrics::error("deserialisation");
            CacheLookupOutcome::Error(CacheError::Deserialisation(e.to_string()))
        }
    }
}

pub async fn insert(
    key: &CacheKey, response: &ProviderResponse, alias: &str,
) -> CacheInsertOutcome {
    // §1 #6 + §1 #7: streaming + failure already filtered upstream by handler.
    let Some(nominal_ttl) = ttl::ttl_for_alias(alias) else {
        return CacheInsertOutcome::Skipped(SkipReason::ChatLongOrUnknownAlias);
    };

    let cr = CachedResponse {
        schema_version: CACHE_SCHEMA_VERSION.into(),
        usage: response.usage.clone(),
        choices: response.choices.clone(),
        finish_reason: response.finish_reason.clone(),
        cached_at: chrono::Utc::now(),
        provider_ms: response.provider_ms,
    };
    let bytes = match serde_json::to_vec(&cr) {
        Ok(b) => b,
        Err(e) => return CacheInsertOutcome::Error(CacheError::Deserialisation(e.to_string())),
    };

    // §1 #8: 1MB cap.
    if bytes.len() > MAX_PAYLOAD_BYTES {
        metrics::oversize(&key.tenant_id, bytes.len());
        return CacheInsertOutcome::Skipped(SkipReason::Oversize { actual_bytes: bytes.len() });
    }

    let jittered = ttl::jittered_ttl(nominal_ttl, &mut rand::thread_rng());
    let mut conn = match timeout(Duration::from_millis(REDIS_TIMEOUT_MS), get_conn()).await {
        Ok(Ok(c)) => c,
        _ => return CacheInsertOutcome::Error(CacheError::Timeout),
    };
    let _: Result<(), _> = conn.set_ex(key.redis_key(), bytes, jittered.as_secs()).await;
    CacheInsertOutcome::Inserted { ttl: nominal_ttl, jittered_ttl: jittered }
}
```

### Redis configuration (production)

```conf
# services/ai-gateway/docker/redis/redis.conf
maxmemory 10gb                       # tenant-isolation budget × ~100 tenants slice 4
maxmemory-policy allkeys-lru         # §1 #12: LRU eviction across all keys
hash-max-listpack-entries 128
hash-max-listpack-value 1024
appendonly no                        # cache is recoverable; no AOF
save ""                              # cache is recoverable; no RDB
```

### Handler integration

```rust
// services/ai-gateway/src/handlers/chat.rs (additions)

async fn handle_chat(req: ChatCompleteRequest) -> Result<ChatCompleteResponse, ApiError> {
    // §1 #9: precheck always runs first.
    let hold = cost_ledger::precheck(&req).await?;
    let persona = persona::load(&req.persona_handle())?;
    let key = CacheKey::derive(
        &req.tenant_id,
        &redacted_prompt(&req),
        &req.model,
        &persona.handle.display(),
    );

    // Cache lookup unless streaming (§1 #6).
    if !req.stream {
        match cache::lookup(&key).await {
            CacheLookupOutcome::Hit(cr, lookup_ms) => {
                cost_ledger::reconcile_hit(&hold, lookup_ms).await?;
                memory_writer::emit(canonical::invocation_cache_hit(&req, &cr, lookup_ms)).await?;
                return Ok(build_response_from_cached(*cr));
            }
            CacheLookupOutcome::Miss
            | CacheLookupOutcome::SchemaMismatch
            | CacheLookupOutcome::Error(_) => {/* fall through */}
        }
    }

    let response = router::call_provider(&req).await?;
    if !req.stream {
        let _ = cache::insert(&key, &response, &req.alias).await;
    }
    Ok(build_response(response))
}
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Cache hit returns** — Insert response under key K; `lookup(K)` returns `Hit(response, lookup_ms)` with `cache_state: hit`; OTel `ai_cache_lookups_total{outcome=hit}` increments.
2. **Cache miss returns Miss** — `lookup(unknown_key)` returns `Miss`; metric `outcome=miss` increments.
3. **Cross-tenant isolation: different tenant** — Insert under `(tenant_a, ...)`; `lookup((tenant_b, same prompt_hash))` returns `Miss` (different Redis key prefix, structurally isolated).
4. **Cross-tenant isolation: scan** — `redis.KEYS "ai_cache:v1:tenant_a:*"` returns ONLY tenant_a's entries even after tenant_b inserts thousands.
5. **Persona-version invalidation** — Insert under `(tenant_a, prompt, model, "cuo-cpo@0.4.1")`; `lookup((tenant_a, prompt, model, "cuo-cpo@0.4.2"))` returns `Miss` (different keys).
6. **TTL expiry (jitter-aware)** — Insert with `chat.fast` (1h nominal); the actual TTL is in `[3240, 3960]` seconds (1h ± 10%). Wait beyond the jittered TTL; lookup returns `Miss`.
7. **chat.long not cached** — `insert(key, response, "chat.long-resolved-...")` returns `Skipped(ChatLongOrUnknownAlias)`; no Redis write occurs.
8. **Unknown alias not cached** — `insert(key, response, "novel.alias")` returns `Skipped(ChatLongOrUnknownAlias)` with WARN log; no Redis write.
9. **Streaming bypasses cache** — Handler with `req.stream=true` calls neither `lookup` nor `insert`; metric `ai_cache_lookups_total{outcome=skipped}` increments via the precheck path's awareness.
10. **Failed response not cached** — Provider returns 500; handler catches the error; `cache::insert` is never called.
11. **Oversize response not cached** — A 2MB serialised `CachedResponse` returns `Skipped(Oversize { actual_bytes: 2097152 })`; OTel `ai_cache_oversize_skipped_total` increments.
12. **Schema-version mismatch treated as miss** — Insert a payload manually with `schema_version: "v0"`; lookup returns `SchemaMismatch`; handler treats as miss and re-fetches.
13. **Redis unreachable graceful degrade** — Stop Redis; `lookup` returns `Error(Unreachable)`; handler proceeds to provider; OTel `ai_cache_errors_total{reason=unreachable}` increments.
14. **30% hit rate over 7 days (workload simulation)** — `cache_workload_simulation_test.rs` runs a synthetic workload (1000 requests/day × 7 days; prompt distribution: 40% repeated, 30% varied, 30% novel). Achieves ≥30% hit rate.
15. **Property test: no cross-tenant leak** — proptest 1000 trials over `(tenant_pair, prompt_hash)`: insert under tenant_a, lookup under tenant_b → always `Miss`.
16. **LRU eviction kicks in at budget** — Insert >100MB under one tenant; older entries evicted via Redis maxmemory-policy; `ai_cache_evictions_total{reason=lru}` increments.
17. **`ai.invocation` carries cache_state** — Every invocation row has `cache_state ∈ {hit, miss, skipped, error}` populated.
18. **Hit-rate alarm fires below floor** — Synthetic workload achieving 25% hit rate over 7 days; OBS sev-3 alarm `ai_cache_hit_rate_low{tenant_id}` fires.

---

## §5 — Verification

### Hit/miss/cross-tenant tests

```rust
// services/ai-gateway/tests/cache_test.rs
use cyberos_ai_gateway::cache::{self, CacheKey, CacheLookupOutcome, CacheInsertOutcome, SkipReason};

fn k(tenant: &str, prompt: &str, persona: &str) -> CacheKey {
    CacheKey::derive(tenant, prompt, "chat.smart", persona)
}

#[tokio::test]
async fn cache_hit_returns_response() {
    let key = k("tenant_a", "What's the weather?", "cuo-cpo@0.4.1");
    let resp = test_provider_response();
    let _ = cache::insert(&key, &resp, "chat.fast").await;

    match cache::lookup(&key).await {
        CacheLookupOutcome::Hit(cr, _) => {
            assert_eq!(cr.choices, resp.choices);
            assert_eq!(cr.schema_version, cache::CACHE_SCHEMA_VERSION);
        }
        e => panic!("expected Hit, got {e:?}"),
    }
}

#[tokio::test]
async fn cross_tenant_miss() {
    let k_a = k("tenant_a", "same prompt", "cuo-cpo@0.4.1");
    let k_b = k("tenant_b", "same prompt", "cuo-cpo@0.4.1");
    let _ = cache::insert(&k_a, &test_provider_response(), "chat.fast").await;

    match cache::lookup(&k_b).await {
        CacheLookupOutcome::Miss => {}
        e => panic!("expected Miss for tenant_b; got {e:?}"),
    }
}

#[tokio::test]
async fn persona_version_change_invalidates() {
    let k1 = k("tenant_a", "prompt", "cuo-cpo@0.4.1");
    let k2 = k("tenant_a", "prompt", "cuo-cpo@0.4.2");
    let _ = cache::insert(&k1, &test_provider_response(), "chat.fast").await;

    match cache::lookup(&k2).await {
        CacheLookupOutcome::Miss => {}
        e => panic!("expected Miss for new persona version; got {e:?}"),
    }
}

#[tokio::test]
async fn redis_keys_scan_is_tenant_isolated() {
    for i in 0..100 {
        let _ = cache::insert(&k("tenant_a", &format!("p{i}"), "cuo-cpo@0.4.1"),
                              &test_provider_response(), "chat.fast").await;
    }
    for i in 0..100 {
        let _ = cache::insert(&k("tenant_b", &format!("p{i}"), "cuo-cpo@0.4.1"),
                              &test_provider_response(), "chat.fast").await;
    }
    let scan: Vec<String> = redis_test_helper::keys("ai_cache:v1:tenant_a:*");
    assert!(scan.iter().all(|k| k.starts_with("ai_cache:v1:tenant_a:")));
    assert_eq!(scan.len(), 100);
}

#[tokio::test]
async fn chat_long_skipped() {
    let key = k("tenant_a", "long story prompt", "cuo-cpo@0.4.1");
    let outcome = cache::insert(&key, &test_provider_response(), "chat.long-resolved-bedrock").await;
    assert!(matches!(outcome, CacheInsertOutcome::Skipped(SkipReason::ChatLongOrUnknownAlias)));
    assert!(matches!(cache::lookup(&key).await, CacheLookupOutcome::Miss));
}

#[tokio::test]
async fn unknown_alias_skipped_with_warn() {
    let key = k("tenant_a", "prompt", "cuo-cpo@0.4.1");
    let outcome = cache::insert(&key, &test_provider_response(), "novel.alias").await;
    assert!(matches!(outcome, CacheInsertOutcome::Skipped(SkipReason::ChatLongOrUnknownAlias)));
}

#[tokio::test]
async fn oversize_response_skipped() {
    let key = k("tenant_a", "prompt", "cuo-cpo@0.4.1");
    let mut resp = test_provider_response();
    // Inflate response to 2MB.
    resp.choices[0].message.content = "x".repeat(2_000_000);

    match cache::insert(&key, &resp, "chat.fast").await {
        CacheInsertOutcome::Skipped(SkipReason::Oversize { actual_bytes }) => {
            assert!(actual_bytes > cache::MAX_PAYLOAD_BYTES);
        }
        o => panic!("expected Oversize; got {o:?}"),
    }
}

#[tokio::test]
async fn schema_mismatch_treated_as_miss() {
    let key = k("tenant_a", "prompt", "cuo-cpo@0.4.1");
    // Manually insert a v0-schema payload.
    let bad = serde_json::json!({
        "schema_version": "v0", "usage": {}, "choices": [],
        "finish_reason": "stop", "cached_at": "2026-05-15T00:00:00Z", "provider_ms": 100
    });
    redis_test_helper::set_raw(&key.redis_key(), &bad.to_string());

    match cache::lookup(&key).await {
        CacheLookupOutcome::SchemaMismatch => {}
        o => panic!("expected SchemaMismatch; got {o:?}"),
    }
}

#[tokio::test]
async fn redis_unreachable_returns_error() {
    redis_test_helper::stop();
    let key = k("tenant_a", "prompt", "cuo-cpo@0.4.1");
    match cache::lookup(&key).await {
        CacheLookupOutcome::Error(cache::CacheError::Unreachable(_)) => {}
        o => panic!("expected Error(Unreachable); got {o:?}"),
    }
    redis_test_helper::start();
}
```

### TTL test

```rust
// services/ai-gateway/tests/cache_ttl_test.rs
#[tokio::test]
async fn ttl_jitter_is_within_10_percent() {
    use cache::ttl::{ttl_for_alias, jittered_ttl};
    let nominal = ttl_for_alias("chat.fast").unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    for _ in 0..1000 {
        let actual = jittered_ttl(nominal, &mut rng);
        let ratio = actual.as_secs_f64() / nominal.as_secs_f64();
        assert!(ratio >= 0.9 && ratio <= 1.1, "TTL jitter outside ±10%: {ratio}");
    }
}

#[tokio::test]
async fn entry_expires_after_jittered_ttl() {
    let key = k("tenant_a", "prompt", "cuo-cpo@0.4.1");
    let _ = cache::insert(&key, &test_provider_response(), "chat.fast").await;
    let actual_ttl = redis_test_helper::ttl(&key.redis_key());
    assert!(actual_ttl >= 3240 && actual_ttl <= 3960);   // 1h ± 10%
}
```

### Property test (cross-tenant)

```rust
// services/ai-gateway/tests/cache_isolation_property_test.rs
use proptest::prelude::*;

fn any_tenant() -> impl Strategy<Value = String> {
    "[a-z]{4,12}".prop_map(|s| format!("tenant_{s}"))
}
fn any_prompt() -> impl Strategy<Value = String> { "[a-zA-Z ]{1,100}".prop_map(String::from) }

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn no_cross_tenant_leak(t1 in any_tenant(), t2 in any_tenant(),
                            prompt in any_prompt()) {
        prop_assume!(t1 != t2);
        let k1 = CacheKey::derive(&t1, &prompt, "chat.smart", "cuo-cpo@0.4.1");
        let k2 = CacheKey::derive(&t2, &prompt, "chat.smart", "cuo-cpo@0.4.1");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _ = cache::insert(&k1, &test_provider_response(), "chat.fast").await;
            let outcome = cache::lookup(&k2).await;
            prop_assert!(matches!(outcome, CacheLookupOutcome::Miss),
                        "cross-tenant leak: {t1} → {t2}");
            Ok(())
        }).unwrap();
    }
}
```

### Workload-simulation test

```rust
// services/ai-gateway/tests/cache_isolation_property_test.rs
#[tokio::test]
#[ignore = "long-running; run with `cargo test workload -- --ignored`"]
async fn 30_percent_hit_rate_over_7_days() {
    // §1 #11: simulate 1000 req/day × 7 days; 40% repeated, 30% varied, 30% novel.
    let prompts_repeated: Vec<String> = (0..50).map(|i| format!("repeat-{i}")).collect();
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut hits = 0u64; let mut total = 0u64;

    for _day in 0..7 {
        for _ in 0..1000 {
            let prompt = match rng.gen_range(0..10) {
                0..=3 => prompts_repeated[rng.gen_range(0..prompts_repeated.len())].clone(),
                4..=6 => format!("varied-{}", rng.gen_range(0..200)),
                _     => format!("novel-{}-{}", _day, rng.gen::<u64>()),
            };
            let key = k("tenant_a", &prompt, "cuo-cpo@0.4.1");
            match cache::lookup(&key).await {
                CacheLookupOutcome::Hit(_, _) => hits += 1,
                _ => { let _ = cache::insert(&key, &test_provider_response(), "chat.fast").await; }
            }
            total += 1;
        }
    }
    let hit_rate = hits as f64 / total as f64;
    assert!(hit_rate >= 0.30, "hit rate {hit_rate:.3} below 30% floor");
}
```

```bash
docker run -d --name test-redis -p 6379:6379 redis:7
cd services/ai-gateway
cargo test cache
cargo test workload -- --ignored
```

---

## §6 — Implementation skeleton

See §3 for type defs, key derivation, TTL table, lookup/insert with backend, Redis config, handler integration. Boot-time wiring:

```rust
// services/ai-gateway/src/lib.rs (additions)
pub async fn run() -> Result<(), Error> {
    // ... existing initialisations ...
    cache::redis_backend::init("redis://redis:6379").await?;
    persona::init_persona_registry().await?;
    cost_ledger::init().await?;
    // ... bind HTTP ...
}
```

`canonical::invocation_cache_hit` builder additions to TASK-AI-002:

```rust
pub mod canonical {
    pub fn invocation_cache_hit(req: &ChatCompleteRequest, cr: &CachedResponse, lookup_ms: Duration) -> AuditRow {
        AuditRow {
            kind: "ai.invocation".into(),
            payload: serde_json::json!({
                "tenant_id": req.tenant_id,
                "agent_persona": req.agent_persona,
                "model": req.model,
                "actual_usd": 0.0,
                "cache_state": "hit",
                "latency_ms": lookup_ms.as_millis(),
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "cached_at": cr.cached_at.to_rfc3339(),
                "original_provider_ms": cr.provider_ms,
                "request_id": req.request_id,
            }),
            ..Default::default()
        }
    }
}
```

---

## §7 — Dependencies

### Code dependencies (other tasks/modules)

- **TASK-AI-008** — `router::call_provider` returns the `ProviderResponse` shape consumed by `cache::insert`.
- **TASK-AI-001** — `cost_ledger::precheck` runs before cache lookup; `cost_ledger::reconcile_hit` zero-cost path.
- **TASK-AI-002** — `ai.invocation` memory row schema declares `cache_state` field; this task populates it.
- **TASK-AI-011** — Redacted prompt is the cache-key input. The redaction MUST run BEFORE key derivation.
- **TASK-AI-014** — Persona handle (`<id>@<version>`) is part of the cache key. Persona changes auto-invalidate by key divergence.
- **TASK-AI-018 (downstream)** — Cross-tenant cache leak test; consumes this task's per-tenant isolation invariant as its property under test.
- **TASK-AI-022 (downstream)** — Cache-warming hooks (slice 4 ships a no-op stub).

### Concept dependencies (shared types)

- `CacheKey::derive(tenant_id, redacted_prompt, model, persona_handle)` — the four-input cryptographic key derivation. Changes to the input set require explicit task amendment.
- `CACHE_SCHEMA_VERSION = "v1"` — the schema-version primitive; bumps invalidate old entries cleanly.
- TTL table in `cache/ttl.rs` is the single source of truth for per-alias TTL; new aliases extend it.
- Per-tenant prefix `ai_cache:v1:{tenant_id}:` is the structural isolation invariant tested in TASK-AI-018.

### Operational / external

- Redis 7.x running at the gateway-reachable address (Docker compose for dev: `services/ai-gateway/docker/redis/redis.conf` provides production-tuned config).
- Rust crates: `redis@0.24` with `tokio-comp` feature, `hex@0.4`, `sha2@0.10`, `serde_json@1`, `chrono@0.4`, `rand@0.8`, `proptest@1` (test-only).
- Redis maxmemory: 10 GB (configured for ~100 tenants × 100 MB at slice 4 scale).
- Redis policy: `allkeys-lru` for cross-tenant fairness under memory pressure.

---

## §8 — Example payloads

### Cache-hit `ai.precheck` row

```json
{
  "kind": "ai.precheck",
  "ts_ns": 1747526400000000000,
  "payload": {
    "tenant_id": "org:cyberskill",
    "agent_persona": "cuo-cpo@0.4.1",
    "estimated_usd": 0.0,
    "current_spent_usd": 47.23,
    "cache_state": "hit",
    "cache_lookup_latency_ms": 3,
    "request_id": "req_01HZK..."
  }
}
```

### Cache-hit `ai.invocation` row

```json
{
  "kind": "ai.invocation",
  "ts_ns": 1747526400123000000,
  "payload": {
    "tenant_id": "org:cyberskill",
    "agent_persona": "cuo-cpo@0.4.1",
    "model": "chat.fast",
    "actual_usd": 0.0,
    "cache_state": "hit",
    "latency_ms": 12,
    "prompt_tokens": 0,
    "completion_tokens": 0,
    "cached_at": "2026-05-15T14:00:00Z",
    "original_provider_ms": 850,
    "request_id": "req_01HZK..."
  }
}
```

### Redis key + payload (cache hit case)

```text
KEY: ai_cache:v1:org:cyberskill:8a7b6c...4d3e2f1a
TTL: 3245 (1h ± 10%)
VAL: {"schema_version":"v1","usage":{...},"choices":[...],"finish_reason":"stop",
      "cached_at":"2026-05-15T14:00:00Z","provider_ms":850}
```

### Skipped on chat.long

```json
{
  "kind": "ai.invocation",
  "payload": {
    "cache_state": "skipped",
    "skip_reason": "chat_long_or_unknown_alias",
    "model": "chat.long",
    "actual_usd": 0.0145
  }
}
```

### Oversize-skipped log

```text
WARN  tenant=org:cyberskill bytes=2456789 cap=1048576
      cache_oversize_skipped — response not cached
```

### Hit-rate-low alarm

```text
WARN  tenant=tenant_beta hit_rate=0.18 floor=0.30 window=7d
      ai_cache_hit_rate_low — investigate prompt diversity or cache cold-start
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later tasks:

- Cache warming at boot (preload likely-asked prompts) — slice 5; no-op stub in this task.
- Per-alias-class hit-rate target (chat.fast might warrant 50% floor; embed.* might warrant 70%) — TASK-AI-022.
- Semantic cache (embedding-distance lookup; "what's the weather?" hits "How's the weather?") — out of scope; current model is exact-prompt-match-only.
- Cross-region cache replication (replicate hot entries between SG and EU Redis instances for residency-aware tenants) — TASK-AI-016 area; current cache is single-region per gateway instance.
- Cost-attribution-on-hit (give the credit to the prior call that populated the cache) — out of scope; current model attributes zero to the hit.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Redis unreachable | Connection error in `lookup`/`insert` | `Error(Unreachable)`; handler proceeds to provider | Self-resolves when Redis returns; sev-2 alert if sustained |
| Redis timeout (>200ms) | `tokio::time::timeout` fires | `Error(Timeout)`; same handler path as Unreachable | Investigate Redis load; possibly add replicas |
| Cache schema version mismatch | `lookup` reads payload, `schema_version != "v1"` | `SchemaMismatch` returned; handler treats as miss | Self-resolves as old entries expire; metric tracks volume |
| Serialisation/deserialisation failure | `serde_json` error in `lookup` | `Error(Deserialisation)`; metric increments | Operator investigates payload corruption; clear cache if widespread |
| Cross-tenant leak (key derivation bug) | proptest in `cache_property_test.rs` (THIS task) + TASK-AI-018 (dedicated cross-tenant test task) | PR blocked on test failure | Fix `CacheKey::derive` |
| Cache size > 100MB per tenant | Redis MEMORY USAGE; Redis-managed eviction | LRU evicts oldest entries; `ai_cache_evictions_total{reason=lru}` | Self-resolves; operator monitors eviction rate |
| Hit rate < 30% over 7 days | `ai_cache_hit_rate{tenant_id}` gauge | Sev-3 OBS alarm | Operator investigates: prompt diversity? Cold cache? Persona churn? |
| Persona version churn (frequent edits) | Cache invalidates on every persona-bump; hit rate drops | Hit-rate alarm | By design (correctness > hit rate); operator considers persona stability |
| Streaming + cache state mismatch | Handler bypasses cache for stream=true; metric `outcome=skipped` | Streaming responses never cached | By design (§1 #6) |
| Failed-call retry pattern hits cache | `cache::insert` only called on success | Retries always go to provider | By design (§1 #7) |
| Oversize response | `insert` checks 1MB cap | `Skipped(Oversize)`; WARN log + metric | Operator investigates response size; likely a bug in upstream prompt |
| Unknown alias added to TASK-AI-006 without TTL | `ttl_for_alias` returns None + WARN log | `Skipped(ChatLongOrUnknownAlias)`; no cache | Operator extends `ttl.rs` map |
| TTL jitter outside expected range | `cache_ttl_test` proptest | PR blocked on test failure | Fix jitter formula |
| Redis maxmemory hit (cluster-wide) | Redis evicts oldest across tenants | Some tenants lose cold entries; `evictions_total{reason=lru}` increments | Operator scales Redis OR reduces per-tenant budget |
| Cache key collision (different inputs, same hash) | Cryptographic 2^128 probability; effectively impossible | N/A | If observed, treat as critical bug; investigate hash function |
| `redis_backend` connection-pool exhaustion | All conn calls error | `Error(Unreachable)` cascades | Increase pool size; investigate connection lifetime |
| Redis evicts a hot tenant's entries due to noisy-neighbour | Per-tenant eviction metric high; hit rate low | Sev-3 alarm | Operator considers per-tenant Redis namespace OR larger budget |
| Schema migration bug (v1 → v2 changes payload shape) | Manual test of v1→v2 deserialisation path; CI matrix | Catch before deploy | Schema-version bump invalidates old entries cleanly per §1 #13 |
| `ai.invocation` row missing `cache_state` field | Integration test asserts field presence | Test fails → PR blocked | Add `cache_state` to canonical builder |
| Insert race (two concurrent inserts for same key) | Redis SETEX is atomic; last-writer-wins is acceptable | Either response cached; minor inefficiency | By design |
| Workload simulation hit rate < 30% on synthetic data | `cache_workload_simulation_test` fails (run with `--ignored`) | Investigation: synthetic distribution doesn't match production | Operator updates synthetic distribution OR tunes TTLs |

---

## §11 — Notes

- Hit-rate target (30%) is conservative — real production prompt patterns (repeated Genie queries, periodic OBS digests, persona-canned greetings) typically hit 40-60% with proper tuning. Slice 4 ships the floor; slice 5+ tunes TTLs based on observed hit-rate distribution per alias class.
- Cache key derivation uses a unit-separator (`\x1f`) to prevent input-collision attacks. Without the separator, `concat(tenant_a, "model")` and `concat(tenant_amodel, "")` produce the same hash input. The unit-separator is a control character that can't appear in a tenant_id, model, or persona handle by the validation rules elsewhere — making collisions structurally impossible.
- The cache schema version (`v1`) is in BOTH the Redis key prefix AND the payload's first 4 bytes. This double-defence handles two failure modes: (a) old code reads new keys (key prefix protects), (b) new code reads old payloads (payload prefix protects). Schema bumps are operationally clean — just bump `CACHE_SCHEMA_VERSION` and old entries naturally expire.
- The 1MB per-entry cap is conservative — typical chat responses are 1-10KB. The cap mostly fires on bug-shaped responses (unbounded list output, full-document embedding chains). The metric `ai_cache_oversize_skipped_total` is a useful debug signal for upstream issues.
- The 100MB per-tenant budget × ~100 tenants slice 4 = 10GB Redis allocation. At slice 5+ scale (1000 tenants), the budget needs revisiting OR per-tenant Redis namespaces. The current monolithic Redis is a deliberate slice-4 simplification.
- `allkeys-lru` policy was chosen over `volatile-lru` because every cache entry has a TTL — there are no "permanent" entries to spare. Eviction is purely usage-recency based, which matches the cost-of-everything intuition (recent calls are likely to recur).
- The graceful-degradation-on-Redis-down path (§1 #14) is the operability primitive. The cache should be helpful (when up), never critical (when down). The alternative — refusing requests when Redis is down — would convert a Redis outage into a gateway outage.
- The cross-tenant property test in this task (§5) is the BASELINE; TASK-AI-018 is the DEDICATED cross-tenant leak task with adversarial scenarios (1000s of tenants, race conditions, malicious tenant_id strings). TASK-AI-018 is downstream and EXTENDS this task's correctness floor.
- The cache-warming hook (§1 #16) ships as a no-op stub. Slice 4 doesn't have hit-rate data showing where warming would pay off; TASK-AI-022 implements warming based on the trend analysis. The stub exists so the wiring is in place for the future implementation.
- Persona-handle inclusion in the cache key (§1 #3) means a single-byte change to a persona prompt (via TASK-AI-014's hot-reload) invalidates the entire cache for that persona. The trade-off is correctness > hit rate: a stale-persona response served from cache would violate TASK-AI-014's hash-tamper-detect contract anyway. Operators should batch persona edits to avoid hit-rate dips.

---

*End of TASK-AI-017. Status: draft (10/10 target).*
