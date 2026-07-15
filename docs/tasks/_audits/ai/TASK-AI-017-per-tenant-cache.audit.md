---
task_id: TASK-AI-017
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.5/10        # the first-pass compressed version (230 lines)
score_post_expansion: 9.0/10      # after expanding to TASK-AI-012 / TASK-AI-014 depth (~960 lines)
score_post_revision: 10/10        # after 6 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
---

## §1 — Verdict summary

TASK-AI-017 was expanded from 230 lines to ~960 lines matching TASK-AI-012 / TASK-AI-014 depth.

The expansion added 6 §1 normative clauses (#1 cryptographic key derivation with unit-separator; #5 ±10% TTL jitter; #8 1MB per-entry cap; #13 cache schema version `v1` in BOTH key and payload; #14 graceful Redis-down degradation; #16 cache-warming hook), 7 substantive §2 rationale paragraphs (Redis-vs-in-process trade-off, redacted-prompt-as-key correctness frame, structural per-tenant isolation argument, persona-key auto-invalidation, TTL-jitter thundering-herd defence, schema-version migration discipline, graceful-degradation availability principle, hit-rate floor break-even economics), full Rust type system in §3 (CacheKey with deterministic derivation, CachedResponse with schema_version, CacheState/CacheLookupOutcome/CacheInsertOutcome enums, SkipReason variants, CacheError variants, constants for sizing/timeouts), full TTL table with alias-class fallback, full Redis backend skeleton with timeout + schema-mismatch + oversize handling, production redis.conf, expanded §4 from 10 to 18 acceptance criteria, full Rust test bodies in §5 (hit/miss/cross-tenant-key-derivation/persona-invalidation/Redis-KEYS-scan/chat.long-skip/unknown-alias-skip/oversize-skip/schema-mismatch/Redis-unreachable-error + TTL jitter range + entry-expires-after-jittered + property test 1000-trial cross-tenant + 7-day workload simulation), expanded §6 with handler integration showing precheck-then-cache-then-provider sequence + canonical builder, expanded §7 with code/concept/operational dep split, 6 example payloads in §8 (precheck row, invocation row on hit, Redis key shape, skipped-on-chat.long row, oversize WARN log, hit-rate alarm), 22 failure modes in §10 (vs. 6 in first pass), 9 implementation notes in §11.

Six residual issues prevented 10/10 at the post-expansion checkpoint; all six are mechanical and all six are resolved in this revision.

## §2 — Findings

### ISS-001 — Cache key derivation drops `persona_handle` despite §1 #1 listing it

- **severity:** error
- **rule_id:** consistency / spec-vs-type-signature
- **location:** §1 #1 (mentions persona_version), §3 (CacheKey lacks persona field)
- **status:** resolved

#### Description

The first-pass §1 #1 said: *"key by SHA-256 of (`tenant_id`, REDACTED prompt, model, agent_persona_version)"* — a four-input hash. But the §3 type signature was:

```rust
pub struct CacheKey {
    pub tenant_id: String,
    pub prompt_hash: [u8; 32],
    pub model: String,
}
```

The persona-version field was missing from the struct. A code-gen agent implementing the type would compute the prompt_hash without persona-version input, producing keys that DON'T invalidate on persona change — directly contradicting §1 #1 and DEC-082.

This is the same type-vs-spec mismatch class as TASK-AI-014 ISS-001 (PersonaVersion vs. PersonaHandle).

#### Suggested fix

Make the persona handle a hash-input. Update `CacheKey` to a single 32-byte `prompt_hash` derived from a four-input cryptographic concat. Document the unit-separator (`\x1f`) defence against input-collision attacks. Add §1 #3 emphasising persona-handle inclusion auto-invalidates cache. Add AC #5 asserting persona-version change → different key.

### ISS-002 — Alias-class detection uses brittle string match (`m if m.contains("long")`)

- **severity:** error
- **rule_id:** correctness / fragile string parsing
- **location:** §6 skeleton
- **status:** resolved

#### Description

The first-pass §6 had:

```rust
if matches!(key.model.as_str(), m if m.contains("long")) { return; }
```

Three problems: (a) "long" matches any model name with the substring (e.g., a future `chat.long-context-bedrock` would skip but so would `chat.belonger`). (b) The TTL decision is based on the FULL model string but the alias class is a prefix (`chat.long`). (c) The TTL table for OTHER aliases is undefined in code; the §1 #3 enumeration is informational.

The fragile string match silently miscategorises new aliases. A new alias `chat.long-resolved-bedrock` would skip (correct), but `chat.embed.standard` (typo) would also skip (silent miss).

#### Suggested fix

Introduce `cache/ttl.rs` with explicit per-alias-class lookup. Use `alias.split('-').next()` to extract the alias class (cleaner than substring matching). Return `Option<Duration>` — `None` means no-cache. Unknown aliases get `None` + WARN log so the operator notices the gap. Add §1 #4 normative TTL table. Add ACs #7 (chat.long skipped) and #8 (unknown alias skipped with WARN).

### ISS-003 — No cache schema versioning; struct field changes silently break lookups

- **severity:** error
- **rule_id:** robustness / migration discipline
- **location:** §1 (no clause), §3 (no version field)
- **status:** resolved

#### Description

The first-pass `CachedResponse` was a flat struct without a schema-version field. Future changes (e.g., adding `model_version: String` for TASK-AI-022) would cause `serde_json::from_slice::<CachedResponse>(&old_bytes)` to fail with a deserialisation error.

Three behaviours are possible on deserialisation error: (a) error propagates (gateway crashes on cache hit of old entries), (b) error is treated as miss but logged at ERROR level (operator panic on every cache-rollover), (c) error is treated as miss silently (acceptable but loses observability).

The first-pass spec doesn't pick — and worse, doesn't differentiate "old schema" from "corrupted entry," both of which look like deserialisation errors.

#### Suggested fix

Add `schema_version: String` field to `CachedResponse`; constant `CACHE_SCHEMA_VERSION = "v1"`. Add §1 #13 normative requirement: include `v1` in BOTH the Redis key prefix (for KEYS-pattern compatibility) AND the payload (for serialisation defence). On lookup, schema-version mismatch returns `SchemaMismatch` (treated as miss); deserialisation errors that AREN'T schema-version mismatches return `Error(Deserialisation)` (treated as miss + ERROR log). The two paths are now distinguishable.

Add AC #12 asserting schema-version mismatch is treated as miss. Add §5 test that manually injects a v0-schema payload and asserts the SchemaMismatch outcome.

### ISS-004 — TTL has no jitter; thundering-herd expiry under load

- **severity:** warning
- **rule_id:** robustness / system behavior under load
- **location:** §1 #3 (TTL declared without jitter), §6 (no jitter logic)
- **status:** resolved

#### Description

The first-pass set TTL exactly to the per-alias value (e.g., `chat.fast` = 3600s). All entries inserted in the same minute expire in the same minute — producing a thundering-herd refresh-cost spike at expiry time. Under high load (1000 req/s with 50% repeated prompts), the spike could double the provider cost transiently every hour.

This is a well-known cache pattern (HTTP cache headers use `Cache-Control: max-age` with jitter recommended; CDN providers default to it). The first-pass missed it.

#### Suggested fix

Add §1 #5: TTL jitter ±10% via `actual_ttl = nominal × (1 + uniform(-0.1, 0.1))`. The function `jittered_ttl(nominal, &mut rng)` is in `cache/ttl.rs`; production seeds from `rand::thread_rng()`, tests seed from `StdRng::seed_from_u64(42)` for determinism.

Add AC #6 enforcing TTL within `[3240, 3960]` for `chat.fast`. Add `cache_ttl_test.rs` with 1000-trial proptest ensuring `actual / nominal ∈ [0.9, 1.1]`. Add §2 rationale paragraph on thundering-herd defence.

### ISS-005 — No max-payload-size; one huge response can blow the per-tenant budget

- **severity:** warning
- **rule_id:** robustness / operational safety
- **location:** §1 (no size cap), §3 (no constant), §6 (no size check)
- **status:** resolved

#### Description

The first-pass had a per-tenant 100MB budget (LRU-evicted) but no per-entry cap. A pathological response (e.g., a 50MB JSON array from rerank, or a full-document embedding response) would consume half a tenant's budget on one entry — evicting hundreds of small recent entries and dropping hit rate.

Real systems hit this: an upstream bug producing oversized responses can wreck cache effectiveness for the rest of the day even if the bug is fixed quickly.

#### Suggested fix

Add §1 #8: `MAX_PAYLOAD_BYTES = 1_048_576` (1 MB). `cache::insert` checks payload size BEFORE writing to Redis; oversize → `Skipped(Oversize { actual_bytes })`; metric `ai_cache_oversize_skipped_total{tenant_id}` increments; WARN log. Add AC #11 asserting 2MB response is skipped. Add §10 row + §11 note explaining the cap is a debug signal for upstream issues.

### ISS-006 — Redis unreachability behaviour undefined; could crash gateway

- **severity:** warning
- **rule_id:** robustness / availability under dependency failure
- **location:** §1 (no graceful-degrade clause), §6 (skeleton uses `.ok()?` which silently propagates)
- **status:** resolved

#### Description

The first-pass §6 used `.ok()?` everywhere:

```rust
let mut conn = client.get_async_connection().await.ok()?;
let raw: Option<Vec<u8>> = conn.get(&redis_key).await.ok()?;
```

This propagates Redis errors as `None` (treated as cache miss). Acceptable for read but the SAME pattern on insert (`.ok()` discards the error silently) means the operator never knows Redis is down. There's no metric, no alarm, no log.

A Redis outage during business hours would silently drop hit rate to 0%, increase provider cost 3x, and the operator wouldn't know until the cost-attribution dashboard updated 24 hours later.

#### Suggested fix

1. Add §1 #14 normative requirement: graceful degrade with explicit `Error` outcome variant + metric per error reason.
2. Add `CacheError` enum: `Unreachable | Timeout | Deserialisation`.
3. Add `tokio::time::timeout(REDIS_TIMEOUT_MS, ...)` wrappers; expose timeout as a constant.
4. Add `ai_cache_errors_total{tenant_id, reason}` metric; sustained errors > 1% trigger sev-2 alarm.
5. Update §6 skeleton to show the timeout + error-mapping pattern.
6. Add AC #13 asserting `lookup` returns `Error(Unreachable)` when Redis stopped.
7. Add §5 test `redis_unreachable_returns_error` that stops Redis, calls lookup, asserts the error variant.

## §3 — Strengths preserved through expansion

- §3 introduces `CacheKey::derive(tenant_id, redacted_prompt, model, persona_handle)` as a single deterministic constructor — the four required inputs are mandatory by signature; no caller can accidentally drop one.
- §1 #2 commits to per-tenant Redis key prefix as a STRUCTURAL invariant (not a runtime check). This is the property under test in TASK-AI-018 (the dedicated cross-tenant leak test task).
- §1 #4 puts TTLs in their own module (`cache/ttl.rs`) with a fallback "unknown alias → no-cache + WARN" pattern. New aliases extend the table; missing ones get a loud signal rather than silent caching.
- §1 #6 + §1 #7 explicitly forbid caching streaming + failed responses; the handler-side decision is documented in the §6 skeleton. No clever logic; clean code paths.
- §1 #11 establishes the 30% hit-rate floor as the break-even economic primitive. The number is justified by the cost target ($4 → $2.80/user/month); operators have a clear threshold for "is the cache earning its cost?"
- §10 inventory grew from 6 rows to 22 — including the schema-mismatch path, the cross-region cache replication non-goal, the Redis maxmemory eviction path, the noisy-neighbour eviction path, and the workload-simulation regression path. Each row has an unambiguous detection mechanism.
- §11 documents the deliberate slice-4 simplification (single Redis instance, no per-tenant namespace, no semantic-cache) so future engineers don't second-guess the scope.

## §4 — Resolution

All 6 mechanical revisions applied (2026-05-16) within the task itself:

- **ISS-001 RESOLVED**: `CacheKey::derive(tenant_id, redacted_prompt, model, persona_handle)` is the four-input cryptographic constructor with unit-separator (`\x1f`) defence; AC #5 asserts persona-version change produces a different key; §5 test `persona_version_change_invalidates`; §1 #3 makes the persona-handle inclusion explicit.

- **ISS-002 RESOLVED**: `cache/ttl.rs` introduced with explicit `ttl_for_alias(alias)` lookup using `alias.split('-').next()` for class extraction; unknown alias returns `None` with WARN log; ACs #7 + #8 added; §5 has `chat_long_skipped` and `unknown_alias_skipped_with_warn` tests.

- **ISS-003 RESOLVED**: `CACHE_SCHEMA_VERSION = "v1"` constant; `CachedResponse.schema_version` field; §1 #13 normative requirement to embed in BOTH key prefix and payload; `CacheLookupOutcome::SchemaMismatch` variant distinguishes from `Deserialisation` errors; AC #12 + §5 test `schema_mismatch_treated_as_miss`.

- **ISS-004 RESOLVED**: `cache/ttl.rs::jittered_ttl(nominal, &mut rng)` with ±10% via `1 + uniform(-0.1, 0.1)`; §1 #5 normative; AC #6 asserts jittered TTL in `[3240, 3960]` for chat.fast; `cache_ttl_test.rs` with 1000-trial proptest; §2 rationale paragraph on thundering-herd defence.

- **ISS-005 RESOLVED**: `MAX_PAYLOAD_BYTES = 1_048_576` constant; §1 #8 normative; insert checks size BEFORE Redis write; `Skipped(Oversize { actual_bytes })` outcome variant; metric `ai_cache_oversize_skipped_total`; AC #11 + §5 test `oversize_response_skipped`; §10 row + §11 note.

- **ISS-006 RESOLVED**: §1 #14 graceful-degrade requirement; `CacheError` enum with `Unreachable | Timeout | Deserialisation`; `REDIS_TIMEOUT_MS = 200` constant; `tokio::time::timeout` wrappers; `ai_cache_errors_total{reason}` metric with sev-2 alarm at 1%; AC #13 + §5 test `redis_unreachable_returns_error`; §6 skeleton shows timeout + error-mapping pattern.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-017 audit (final). Status: PASS at 10/10.*
