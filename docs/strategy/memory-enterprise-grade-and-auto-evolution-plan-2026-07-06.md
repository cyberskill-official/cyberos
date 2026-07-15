# MEMORY module: enterprise-grade + auto-evolution plan

Date: 2026-07-06. Scope: `services/memory` (Rust Layer-2 + BRAIN), `modules/memory` (Python Layer-1 personal store), the desktop app under `services/memory/desktop`, and their task catalog (`docs/tasks/memory/`). Method: full read of the brain subsystem source, migrations 0001-0008, the interaction-event path, the desktop sync supervisor, and the Python core module map, cross-checked against 2025-2026 state of the art (Mem0, Letta, Zep/Graphiti, LongMemEval, ElectricSQL/PowerSync, OpenAI Dreaming, Anthropic memory tool and contextual retrieval). Companion docs: `cyberos-brain-evaluation-plan.md` (governance + evaluation phases) and `cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md` (platform-wide R1-R52). This plan goes deep on one module only and is numbered independently: S = strength, F = finding, R = recommendation.

## 1. Executive summary

The memory module has the right skeleton and an unusually strong integrity story: an append-only hash-chained Layer 1 as the system of record, a derived and rebuildable BRAIN lens (embeddings, versioned summaries, hot/warm/cold tiers), consent-gated capture, provenance with read-time chain verification on every recall hit, and spend-capped embedding through the ai-gateway. Almost no commercial memory product carries tamper-evident provenance on recall hits; that design choice is worth protecting because the whole evaluation program (Lumi scoring people against signed documents) rests on it.

Three structural problems keep it from being the company core today. First, the platform runs two disconnected memory systems: the Python personal store holds the actual memory science (recency decay, importance scoring, MMR, dream-loop reflection, semantic dedup) while the Rust serving path uses none of it. Second, the BRAIN embeds event envelopes and kind-frequency digests rather than meaning: `content_ref` deliberately strips content, and summaries are strings like "chat.message_created x12", so semantic recall cannot answer a question like "what did we agree with the theater client about pricing". Third, the online-offline sync path is scaffolding: the desktop supervisor spawns a Python daemon that does not exist in the tree, while the chat module already shipped the exact seq/pos + idempotent-outbox pattern memory needs.

Security has one critical hole (recall and search trust `x-tenant-id` / `x-subject-id` headers, so any caller can read any tenant) and one fail-open pattern (RLS policies pass when the tenant GUC is unset). Compliance has one unsolved conflict (an append-only chain with inline bodies versus PDPL/GDPR erasure) that crypto-shredding resolves cleanly if adopted before content-bearing capture scales.

The recommendations below (R1-R108) are sequenced into four phases. P0 closes the security holes and the recall-quality bugs. P1 unifies the two systems and makes recall content-aware. P2 ships offline sync on the chat-core pattern and the compliance pack. P3 turns on the self-improvement loop: golden-set evals gating every consolidation and prompt change, usage-feedback ranking, GEPA prompt optimization, and the dream loop as a gated nightly consolidator. The end state is a memory that measures its own recall quality, tunes its own policies under evidence gates, and heals its own derived data, which is what "auto-evolve" should mean in production.

## 2. Scorecard

| Capability | State today | Grade |
|---|---|---|
| Integrity + provenance | Hash-chain anchors verified at read; derived layer rebuildable | A- |
| Capture governance | Consent gate default-deny; content_ref privacy spine; real gate unwired | B |
| Semantic recall quality | Envelope-only embeddings, digest summaries, dead confidence floor, no rerank/MMR/decay | D |
| Knowledge structure | Entities regex-only; edges untyped-temporal; graph unused at recall | D |
| Consolidation | Versioned summaries exist; extractive only; O(N^2) rebuild; no facts, dedup, or contradiction handling in serving path | C- |
| Memory science (Python L1) | Decay, importance, MMR, dream detectors, invariants walker, property/fuzz/chaos tests | B+ (unported) |
| Online-offline sync | Desktop supervisor spawns a nonexistent daemon; no first-party protocol | F |
| Multi-tenant security | RLS on brain tables but fail-open GUC; header-trust auth; no RLS on L1/L2 tables | D (one critical) |
| Privacy + compliance | PII regex-only and not in the cloud path; no retention engine; erasure unsolved vs chain | D |
| Embedding lifecycle | Model version recorded per row; fixed vector(1024); no migration flow; serial embeds | C |
| Observability | OTel RED + ingest/tier/recall metrics, Prometheus cursor gauges | B |
| Self-evolution | Dream loop spec'd (TASK-115) and implemented file-side; zero recall evals; no feedback signals | D+ |

## 3. Current state map

Two systems exist under one module name.

The Python Layer 1 (`modules/memory/`, ~28k LOC) is a file-based personal memory: an append-only markdown store with a hash-chained binlog, sync classes (private/shareable), store ACL specs, pre-ingest PII scan, importance scoring, recency decay, MMR re-ranking, episodic memory, a four-detector dream loop (duplicates, stale, patterns, verify), consolidation with zstd archival and auditable prune/restore, an invariants walker driven by `memory.invariants.yaml`, an MCP server, and a deep test suite (property, fuzz, chaos, mutation). This is where the memory science lives.

The Rust service (`services/memory`, ~9k LOC) is the enterprise plane: `l1_audit_log` as the cloud mirror of the chain (migration 0003), Layer-2 projections `l2_memory`/`l2_entity`/`l2_edge` (0001), and the BRAIN (TASK-MEMORY-121/123): interaction events as aux chain rows with generated-column indexes and a per-tenant unique `event_id` replay guard (0005), `brain_event_embedding` with hot/warm/cold tiers and a partial hot HNSW (0006), versioned `brain_summary` with supersede-not-overwrite and top-contributor provenance (0007), per-tenant ingest cursor and tier watermark advanced transactionally (0008), an ingest worker that embeds only through the ai-gateway with spend-cap degradation to `pending_embed_retry`, summaries-first recall with RRF fusion, per-hit chain verification, and an access predicate mirroring `eval::access::may_read` (founder / manager_of / self, deny-by-default).

The desktop app (`services/memory/desktop`) is a Tauri 2 scaffold whose `sync_supervisor.rs` spawns `python3 -m cyberos.core.memory_sync_daemon` with a circuit breaker; the module it spawns does not exist yet, and the file's own header says so.

The task catalog already specifies most of what modern memory systems do: TASK-MEMORY-112 episodic memory, 113 recency-decay ranking (Park et al. 0.4 relevance / 0.3 importance / 0.3 recency), 114 write-time importance, 115 dreaming, 116 semantic dedup, 117 per-store ACL, 118 put-if preconditions, 119 session transcript ledger, 120 history. Wave 1-3 sequencing and Stephen's one-approval-at-a-time protocol rule are recorded in the task README. The gap is that these are implemented (or specified) for the file store and absent from the serving path that the rest of CyberOS calls.

## 4. Strengths to preserve (S1-S12)

- S1. System-of-record discipline: DEC-2721 makes every brain table derived and rebuildable from Layer 1; conflict resolution is "Layer 1 wins", and `stale` rows are re-ingested rather than trusted (`migrations/0006`, `src/brain/mod.rs`).
- S2. Tamper-evident recall: every hit carries `audit_row_id` provenance and a read-time chain-anchor recompute; tampered candidates are dropped before access checks so nothing leaks through a side channel (`recall.rs:126-150`).
- S3. Consent-gated capture with a default-deny stub and a TTL cache; system actors exempt; no code path captures a person before acknowledgment (`consent_gate.rs`, DEC-2702).
- S4. The privacy spine: `content_ref` stores pointers or hashes, never raw bodies, so the years-long chain does not accumulate message text by accident (`content_ref.rs`, DEC-2701).
- S5. Gateway-only embeddings with residency, ZDR, and tenant spend caps; a 402 degrades to `pending_embed_retry` and there is no provider-direct fallback path to drift (`embed_client.rs`, DEC-2723).
- S6. Idempotent ingest: UPSERT on `(tenant_id, source_seq)` plus cursor advance in the same transaction; crash-mid-batch replays are no-ops (`ingest_worker.rs`, `event_cursor.rs`).
- S7. Cost-bounded ANN: the partial HNSW index covers only `tier='hot'`, so query cost tracks the hot window rather than lifetime volume (`migrations/0006`, `tiering.rs`).
- S8. Versioned summaries: supersede-not-overwrite with `covered_seq_range` and top contributors, so every summary is citable and auditable (`migrations/0007`, `summarize.rs`).
- S9. RLS with FORCE on all brain tables plus a transaction-local GUC helper used on every query path (`brain/mod.rs::tenant_tx`).
- S10. Degraded-mode honesty: recall reports `degraded_backends`, falls back to full-text over summaries when the gateway is down, and 503s only when nothing can answer (`recall.rs`).
- S11. Test discipline: twelve integration test files cover ingest, tiering, summaries, provenance, RLS, and access scoping; the Python side adds property/fuzz/chaos/mutation suites and a declarative invariants walker.
- S12. Observability baseline: OTel RED middleware with real tenant labels, ingest lag, tier gauges, access-denied counters, spend counters, plus Prometheus cursor gauges (`metrics.rs`, `main.rs`).

## 5. Findings (F1-F35)

Architecture and recall semantics:

- F1. Two disconnected memory systems. Decay (`decay.py`), importance (`importance.py`), MMR (`mmr.py`), combined ranking (`ranking.py`), dream detectors (`dream/detectors.py`), and semantic dedup (`consolidate.py`, `semantic.py`) exist only in the Python file store. The Rust recall path uses none of them, so the serving brain is scientifically behind its own codebase.
- F2. The BRAIN embeds envelopes, not meaning. `event_cursor::read_after` embeds the Layer-1 `body`, which for interaction events is the audit envelope (module, verb, target_ref, content_ref, capped attributes). Actual content stays behind pointers by design, so semantic recall over what people said or wrote is structurally impossible today.
- F3. Summaries carry near-zero semantics. `summarize::build_digest` emits kind-frequency strings ("[subject:...] 47 interactions: chat.message_created x12, ..."). Embedding these gives cosine rankings over verb statistics; the file header itself calls the abstractive upgrade an additive later step.
- F4. No semantic fact layer. There is no extraction of durable facts, no Mem0-style ADD/UPDATE/DELETE/NOOP decision on write, no dedup or contradiction handling in the serving path; TASK-MEMORY-116 covers the file store only.
- F5. The graph is a stub. `entity_extract.rs` pulls @handles, #tags, and [[wikilinks]] by regex; `l2_edge` has `created_at` only (no `valid_at`/`invalid_at`/`expired_at`), so facts cannot be invalidated over time, and nothing writes typed edges in the brain path.
- F6. Recall never touches the graph. `recall.rs` fuses two vector retrievers (summaries, hot events); `search.rs` fuses lexical + vector over `l2_memory`. No graph traversal, no entity-anchored retrieval anywhere.

Recall correctness and performance:

- F7. The confidence floor is dead code. `recall.rs:102` sets `best_summary = 1.0` whenever any summary row returns, regardless of distance, so the "drill into events when summary confidence is low" behavior (BrainConfig 0.30 floor) can never trigger on quality grounds.
- F8. Event hits can return empty snippets. `hot_event_search` sets `snippet: String::new()` with a comment saying verify fills it; `verify_candidate` never does, and `enrich_snippet` is never called by `recall` or the handler. Callers can receive scored hits with no text.
- F9. The hot path is N+1. Per candidate (up to 50): one L1 SELECT plus SHA-256 recompute for chain verify, then up to two `access_grant` queries, all sequential. Worst case is roughly 150 round trips per recall before the response is assembled.
- F10. No reranker, no MMR, no temporal ranking. TASK-MEMORY-108's own title promises BGE reranking; `search.rs` and `recall.rs` ship RRF only. Score is pure rank fusion with no recency, importance, or diversity terms.
- F11. Doc/code drift on warm tier: `tiering.rs` says warm embeddings are "still vector-searchable on drill", but `hot_event_search` filters `tier = 'hot'` only. Warm and cold vectors are retained yet unreachable.
- F12. No lexical retriever over events in brain recall; full-text exists only as the summaries fallback when the gateway is down, so hybrid recall (the 2025-2026 default) is absent from the primary path.
- F13. No pagination or cursoring on recall; `limit` max 100 with no stable continuation.
- F14. Per-request client construction and no query-embedding cache: `handler.rs:38` builds `EmbedClient::from_env()` per HTTP call; identical queries re-embed every time.

Security and privacy:

- F15. Critical: header-trust authentication. `/v1/memory/recall` and `/v1/memory/search` resolve tenant and viewer identity purely by `x-tenant-id` / `x-subject-id` headers (`handler.rs:28-32`, `search.rs:267`). Any network caller can claim founder-grade visibility in any tenant. The JWT middleware is acknowledged as a later migration in the file docs; it is a P0.
- F16. RLS is fail-open on a missing GUC. Every brain policy passes when `current_setting('app.tenant_id', true) IS NULL`, and the nil-UUID string is a bypass value (`migrations/0006-0008`). One code path that forgets `tenant_tx` reads across tenants silently instead of failing.
- F17. No RLS at all on `l1_audit_log`, `l2_memory`, `l2_entity`, `l2_edge` (migrations 0001/0003); isolation there rests on WHERE clauses alone. Deep-audit R15 (extend the RLS property gate to memory tables) is directly on point.
- F18. No rate limiting on any route (`main.rs` router), so recall doubles as an extraction oracle once F15 is fixed and before per-principal limits exist.
- F19. The real consent gate is unwired: DEC-2702's default-deny stub is what ships; TASK-MEMORY-122's TASK-EVAL-001-backed gate does not exist yet, and a stray use of the test-only `AllowAll` would silently capture everyone.
- F20. PII protection is regex-only, Python-only. `pre_ingest_pii.py` implements "Presidio-style" patterns with a swap-in note for the real analyzer; TASK-MEMORY-111's target (Presidio EN + VN recognizers, >= 99.5% recall) is unmet, and the Rust cloud ingest path applies no PII pass at all.
- F21. Erasure is currently impossible. `l1_audit_log.body` holds full markdown bodies inline for personal-store puts, the chain is append-only by invariant, there is no lineage table cascading a source row to its embeddings/summaries, and HNSW tombstones leave reconstructible "ghost vectors". PDPL Law 91/2025 (effective 2026-01-01) and GDPR both require deletion including derived artifacts.
- F22. No retention engine in the cloud: `prune.py` archives device binlogs only; nothing ages or archives `l1_audit_log` or brain rows by policy.
- F23. Recalls are not audited. The chain records `view` ops for the file store, but `/v1/memory/recall` writes no audit row, so "who searched whose memory" is unanswerable, which the evaluation program will need.
- F24. Poisoning surface: recall snippets are raw Layer-1 bodies re-injected into agent prompts with no trust scoring, no quarantine for new memories, and no instruction-stripping. The obs-triage incident (model fabricating a runbook URL from a SKILL.md example) is the in-house preview of MINJA-class attacks (>95% success in the literature).

Sync and offline:

- F25. Offline sync is scaffolding. `sync_supervisor.rs` spawns `cyberos.core.memory_sync_daemon`, which does not exist anywhere in the tree (the file header admits it); the desktop stays "online" with no syncing. TASK-MEMORY-103's daemon (laptop A <-> cloud <-> laptop B, sync_class gating, 10k-file offline queue) is unbuilt.
- F26. The conflict model assumes third-party file sync: `conflicts.py` detects Dropbox/Syncthing/Nextcloud conflict siblings. There is no first-party protocol for memory rows, while chat already shipped the MSC4186-style seq/pos + idempotent outbox and a client SQLite core that memory could reuse almost verbatim.
- F27. No at-rest encryption requirement for the desktop store; `crypto_mode.py` governs store crypto modes but the Tauri app defines no SQLCipher/keychain posture.

Lifecycle and operations:

- F28. Embedding lifecycle is half-built: `embed_model_version` is recorded per row (good), but the column is fixed `VECTOR(1024)`, there is no halfvec, no dual-column shadow migration flow, and the hot HNSW index would mix model versions after a model change. The `l2_memory` HNSW is still commented out in migration 0001.
- F29. Ingest hot path does redundant work: one embed call per event (no batch input), three COUNT queries per event via `touch_windows`, and `build_and_supersede` re-reads every event in scope on each re-summarize (O(N^2) as scopes grow); the time-window scope summarizes all events (acknowledged stub).
- F30. Tenant discovery runs a DISTINCT scan over `l1_audit_log` every tick for both daemons (`main.rs:302`).
- F31. Metrics label bug: a malformed gateway embedding increments `ingest_failure{postgres_error}` (`ingest_worker.rs:68`).
- F32. Zero recall-quality evals: no golden query set, no LLM-judge rig, no CI gate on retrieval changes; the deep audit's central complaint (no measurement) applies at full strength here.
- F33. Summary versioning has a race: concurrent `build_and_supersede` for one scope both read the same prior version and collide on the `(tenant, scope_kind, scope_id, version)` unique index with no retry.
- F34. `AI_GATEWAY_URL`'s `/v1/embeddings` route is documented-but-unwired per `embed_client.rs`'s own note, so the entire brain currently runs only against stubs unless the gateway has since lit the route.
- F35. Pooling caveat unverified: `set_config(..., true)` is transaction-local and safe under transaction pooling, but Supabase/PgBouncer behavior for the two GUC names used (`app.tenant_id`, `app.current_tenant_id`) is asserted nowhere in tests or docs.

## 6. Recommendations

Numbered R1-R108, grouped A-J. Each item names the mechanism and, where useful, the file it lands in. Items marked (P0) belong to the first phase regardless of group.

### A. One memory plane: unify the two systems (R1-R8)

- R1. Declare the BRAIN the single enterprise memory API and demote the Python personal store to an edge client of it. Every module (chat, proj, obs, Lumi) reads memory through `/v1/memory/recall`; the file store syncs into Layer 1 as it already does and stops growing its own retrieval features. One recall contract, one access model, one place to measure quality.
- R2. Port the Park-et-al combined score into brain recall: final = 0.4 x normalized relevance + 0.3 x importance + 0.3 x recency, with recency as exponential decay on `last_accessed_at` (TASK-MEMORY-113 already fixes the weights; `ranking.py` and `decay.py` are the reference implementations). Normalize each component to [0,1] before weighting and keep RRF as the candidate-stage fuser.
- R3. Add a `memory_kind` taxonomy to the schema: episodic (interaction events, already there), semantic (durable facts), procedural (playbooks/skills), profile (one per subject), resource (document pointers). This mirrors the MIRIX/LangMem typed-store consensus and lets retrieval, retention, and consolidation policy differ per kind instead of one-size-fits-all.
- R4. Create the missing semantic layer: a `brain_fact` table (id, tenant_id, subject_id, fact text, embedding, importance real, access_count, last_accessed_at, valid_at, invalid_at, revision, previous_id, derived_from jsonb of audit_row_ids, source_kind, trust real, pii_flags text[]). Facts are what Mem0, Zep, and ChatGPT-class memory actually retrieve; events and summaries alone cannot express "Daria prefers Vietnamese in DMs".
- R5. Adopt the Mem0 two-phase write path for facts: extract candidate facts from new content with an LLM (through the ai-gateway), retrieve top-k similar existing facts, then have the model choose ADD / UPDATE / DELETE / NOOP per candidate via a tool-call schema. Log every operation as its own chain row so consolidation is auditable and reversible; never mutate Layer 1.
- R6. Make ingestion content-aware without breaking DEC-2701: when an event's `content_ref` is a pointer, the ingest worker dereferences it under the owning store's RLS, runs the PII pass, embeds the redacted content, and stores only the vector plus the pointer (no raw body copy). Consent-gated per subject like everything else. This single change turns recall from verb statistics into actual memory.
- R7. Make `l2_edge` bi-temporal (Graphiti model): add `valid_at`, `invalid_at`, `expired_at`; on ingesting a contradicting edge, invalidate the old one rather than deleting it, keeping point-in-time queries (`WHERE valid_at <= t AND (invalid_at IS NULL OR invalid_at > t)`). Zep's paper reports 94.8% DMR and sub-200ms recall with this shape, with no LLM at query time.
- R8. Upgrade entity extraction: LLM extraction of typed entities and relations per episode (gateway call, batched), embedding + FTS candidate match against `l2_entity`, model-adjudicated dedup with `name_aliases[]` on merge. Keep the regex extractor as the zero-cost fallback tier.

### B. Recall quality and latency (R9-R24)

- R9. (P0) Fix the confidence floor: return the real cosine similarity from `summary_search` and compare that against `recall_confidence_floor` instead of the constant 1.0 (`recall.rs:102`). Add a regression test that a weak summary match triggers drill.
- R10. (P0) Fill event snippets: batch-fetch the Layer-1 bodies for surviving candidates in one `WHERE seq = ANY($1)` query and populate snippets before returning (`recall.rs`, fixes F8 and most of F9 in one motion).
- R11. Add a lexical retriever to brain recall as a third RRF leg: a `tsvector` expression index over fact text and abstractive digests (plus `pg_trgm` for handles). Hybrid BM25+vector is the single highest-yield retrieval upgrade in the 2025-2026 literature (Anthropic reports 49% fewer retrieval failures for hybrid contextual retrieval, 67% with reranking on top).
- R12. Add a cross-encoder reranker stage over the fused top 50: bge-reranker-v2-m3 served by the existing embed-sidecar (TASK-MEMORY-108 already names BGE rerank). Budget one batched sidecar call per recall; skip when degraded.
- R13. Port MMR from `mmr.py` as the final diversity filter (lambda ~0.7) so five near-duplicates of one fact do not fill the whole limit.
- R14. Apply contextual embedding: prepend a 50-100 token situating prefix (who, channel, date, kind) to each text before embedding and indexing. This is Anthropic's contextual-retrieval result applied to memory rows; it also makes lexical search match names and dates.
- R15. Query rewriting at the edge of recall: parse relative time expressions ("last month", "tuan truoc") into `ts_since`/`ts_until`, expand subject handles to UUIDs, and optionally generate one paraphrase for a second vector probe. LongMemEval attributes most of its headroom to time-aware query expansion plus fact-augmented keys.
- R16. Add a graph leg: seed from entities matched in the query, expand 1-2 hops over `l2_edge`, and include the connected facts/events as a fourth candidate list; rerank by graph distance from the caller's subject node for cheap personalization (Graphiti's node-distance reranker is a SQL join here).
- R17. Route by query shape: single-hop factual queries go vector+lexical only; relational/multi-hop cues ("why", "who decided", multiple entities) enable the graph leg. A heuristic router is enough to start; log the routing decision in `explain`.
- R18. (P0) Batch the access predicate: replace per-candidate `caller_may_see` with one query computing the caller's visible-subject set (founder grant -> all; else self + granted targets) and filter candidates in memory. Two round trips total instead of 2 x candidates; keep the per-hit check for the rare summary whose subject resolves late.
- R19. Rework chain verification for latency: verify in one batched query (fetch all candidate L1 rows at once, recompute anchors in Rust), sample-verify (e.g. 1-in-N plus always-verify on `explain=true`) once the nightly chain walker (deep-audit R21) exists, and record per-hit `chain_verified` honestly either way.
- R20. Make warm tier reachable: honor the documented behavior by searching warm rows when `drill=true` (sequential scan is fine at current scale, or add a second partial index with halfvec + binary quantization to keep it cheap). Cold stays summary-only plus on-demand fetch by `audit_row_id`.
- R21. Add keyset pagination to recall (`after_score`/`after_seq` cursor) and to search, so agent callers can page beyond 100 without re-ranking drift.
- R22. Cache query embeddings (small LRU keyed by hash(model, text), 5-minute TTL) and construct one `EmbedClient` in `AppState` instead of per request (`handler.rs:38`).
- R23. Return a scores breakdown in `explain` (per-leg rank, fused score, rerank score, decay/importance terms) so eval tooling and humans can debug rankings; today `explain` reports only path and latency.
- R24. Add a feedback write-back: `POST /v1/memory/recall/:id/feedback {used, helpful}` incrementing `access_count`, setting `last_accessed_at`, and logging a feedback event to the chain. This is the raw signal the evolution loop (section E) consumes.

### C. Consolidation, forgetting, and the dream loop (R25-R36)

- R25. Replace kind-frequency digests with abstractive summaries generated through the ai-gateway chat route under the same residency and spend policy (DEC-2724 anticipated exactly this). Prompt them to name topics, decisions, and open questions with citations to contributor rows, and to exclude secrets; keep the extractive digest as the degraded mode.
- R26. Make summarization incremental: summarize only events past `covered_seq_hi`, then merge with the prior digest (rolling update), instead of re-reading the whole scope (`summarize.rs::build_and_supersede`); bound the time-window scope to its actual ISO week. This removes the O(N^2) growth and most of the per-event COUNT load.
- R27. Build a RAPTOR-style hierarchy instead of one flat summary per scope: leaf = per-day or per-episode digests, mid = weekly per-subject/channel, root = subject profile. Recall searches all levels; the profile level is what gets injected into agent context by default (the ChatGPT "Dreaming" shape: precomputed layered context, no per-turn deep search for the common case).
- R28. Episodic-to-semantic promotion: a background job extracts durable facts (R4/R5) from episodes that age past the hot window or cross an access-count threshold, writing `derived_from` provenance so promotion is reversible and erasure can cascade.
- R29. Semantic dedup in the serving path: nightly job finds fact pairs with cosine > 0.95, merges via the R5 op pipeline (UPDATE/DELETE ops with provenance), mirroring `consolidate.py`/TASK-MEMORY-116 semantics.
- R30. Contradiction handling: on fact ADD, retrieve semantic neighbors and let the model mark contradicted facts invalid (set `invalid_at`, keep the row). Never hard-delete on contradiction; history is a feature for an evaluation platform.
- R31. Write-time importance scoring (TASK-MEMORY-114): rate each fact/episode 1-10 with a cheap model against an anchored rubric (examples pinned in the prompt so ratings do not drift high), stored in `importance` and used by R2 ranking and the reaper.
- R32. Decay and forgetting: a reaper job demotes rows whose combined score falls below threshold (archive flag, drop from indexes), honoring per-kind retention policy (R81). Layer 1 is never touched; forgetting is an index/serving concept.
- R33. Port the dream loop to the BRAIN: run the four TASK-MEMORY-115 detectors (duplicates, stale, patterns, verify) as nightly jobs over brain tables, emitting proposals to a review queue, with the applier gated exactly like the existing cap-4 dream loop (disabled by default, evidence required, every apply chained). The Python `dream/` package is the reference implementation.
- R34. Cap rewrite depth and version everything: summaries and facts carry `revision` + `previous_id`; consolidation may rewrite a derived row at most N times (suggest 3) before requiring re-derivation from sources. This guards against the documented 2026 failure mode where continuously LLM-rewritten memory degrades on the tasks it was built from.
- R35. Gate every consolidation batch with evals: run the golden recall set (R45) before and after; regressions beyond threshold auto-revert the batch and file a report. This is the CSAF/CDS gate discipline applied to memory content.
- R36. Fix the summary version race: take a per-(tenant, scope) advisory lock in `build_and_supersede`, or retry on unique-violation of `(tenant, scope_kind, scope_id, version)`.

### D. Embedding lifecycle (R37-R44)

- R37. Move to `halfvec(1024)` for all vector columns and rebuild HNSW with m=16, ef_construction=64, plus a tuned `hnsw.ef_search` per route: ~50% index size for negligible recall loss, straight from pgvector guidance.
- R38. Add `embedded_at` and `content_hash` beside `embed_model_version`; skip re-embeds when the hash is unchanged and make every backfill idempotent by hash.
- R39. Define the model-migration runbook now: add `embedding_v2` shadow column, backfill in checkpointed batches through the gateway, build the new partial index concurrently, run the golden set old-versus-new, flip a config flag per tenant, drop the old column after soak. Extend `backfill.rs` to drive it. Rollback is the flag flip.
- R40. Never mix model versions in one ANN index: make the partial indexes include `embed_model_version = '<current>'` in their predicates, so a half-migrated table cannot produce nonsense neighborhoods.
- R41. Batch embedding calls: the gateway contract already takes `input: []`; send up to 64-256 texts per call from the ingest worker and the backfill, with a concurrency cap. This is the main ingest-lag lever (F29).
- R42. When the model roadmap allows, prefer an MRL-capable embedding model and adopt Supabase-style adaptive retrieval: index a 256-dim truncation for candidate generation, rescore top-100 with full vectors.
- R43. Drift sentinels: a nightly job re-embeds a fixed sentinel corpus, compares cosine against stored baselines, and alerts on shift alongside a golden-set recall drop; this catches silent gateway model swaps.
- R44. (P0) Light up the gateway `/v1/embeddings` route (F34) and add a contract test in memory's CI against a gateway stub binary, so the documented-but-unwired dependency cannot regress silently.

### E. Auto-evolution: measure, learn, tune, heal (R45-R58)

- R45. (P0 for the runner, content grows forever) Golden recall set: 100+ curated (query -> expected audit_row_ids / expected answer) cases harvested from real usage per tenant-class, stored in `services/eval`, run by a `memory-eval` binary in CI on every PR touching retrieval, ranking, summarization, or prompts. Fail the gate on >3% drop in recall@10 or judge score. This is the substrate every other evolution mechanism stands on.
- R46. LLM-as-judge with calibration: rubric-based grading of recall relevance and answer support, validated against a small human-labeled set until agreement reaches 75-90%, re-calibrated quarterly; judge prompts and versions live in the repo beside the golden set.
- R47. Usage-signal loop: from R24 feedback plus automatic citation detection (was the memory quoted in the agent's answer), maintain per-memory `retrieved_count`, `used_count`, and a used-ratio; demote chronically retrieved-but-unused memories via the importance term and feed the ratio into dedup candidacy.
- R48. Retrieval config A/B: a `retrieval_config` table (weights, k, floor, reranker on/off, MMR lambda) with hash-based assignment per caller and judge-scored outcomes; promote winners by flipping the default config row, keeping the loser for rollback. All server-side, no client changes.
- R49. Prompt/policy optimization offline: run GEPA (dspy.GEPA) over the extraction, summarization, and judge prompts against the golden set on a cadence; check optimized prompts into the repo with their eval evidence attached to the PR. GEPA's reflective evolution beats RL-style tuning with ~35x fewer rollouts, which matches the spend-cap posture.
- R50. Telemetry to standard: adopt OpenTelemetry GenAI semantic conventions for gateway spans and add `memory.op` counters (add/update/delete/noop), recall hit-rate, per-leg contribution, and judge-score gauges, so Datadog/Langfuse-class tooling reads it natively alongside the existing obs pipeline.
- R51. Memory-ops console tile: ingest lag, pending retries, tier counts, recall p50/p95, hit-rate, golden-set trend, spend per tenant, poisoning quarantine queue, erasure queue. The operator view is what makes self-healing trustworthy.
- R52. Self-healing job registry: re-embed pending rows (exists), re-summarize stale scopes, re-dedupe, reindex, rebuild graph communities, refresh profiles. Every job records before/after metrics to the ops tile and auto-reverts on eval regression; every run writes a ledger entry per the AUTO_WORK protocol.
- R53. Weekly memory-quality report to Stephen, auto-generated: metric trends, top regressions, pending dream proposals, and one recommended action; the report itself is chained (it is evidence).
- R54. Shadow evaluation on live traffic: sample N% of recalls, replay against the candidate config in the background, log score deltas; promotes configs with production evidence rather than benchmark-only evidence.
- R55. Procedural memory via ACE-style playbooks: store per-agent playbooks as itemized bullets with delta updates (Generator/Reflector/Curator), curated by the dream loop, never monolithically rewritten (context-collapse guard). This is the SKILL.md pattern already used across CyberSkill, made a first-class memory kind (R3) and connected to deep-audit's ACE recommendation.
- R56. Build an internal LongMemEval-style benchmark: five ability buckets (single-session extraction, multi-session reasoning, temporal reasoning, knowledge update, abstention) generated from CyberOS's own domains; track per-release. Public LoCoMo-style numbers are contested (Mem0 versus Zep both re-scored each other); own the benchmark instead of borrowing marketing numbers.
- R57. Abstention behavior: recall exposes a calibrated confidence; below floor, agents must answer "not in memory" rather than synthesize. Measure false-memory rate explicitly; it is the most dangerous memory failure for an evaluation platform.
- R58. Poisoning red-team in CI: MINJA-style injection cases (instructions embedded in captured content, adversarial near-duplicates targeting someone else's record) asserted to end up quarantined, de-ranked, or stripped. Pair with R79/R80 defenses.

### F. Online-offline sync (R59-R72)

- R59. (P0 decision) Retire the phantom Python daemon plan: either implement `cyberos.core.memory_sync_daemon` immediately or, better, build memory sync in Rust inside the Tauri app the way chat-core was built. Today the supervisor loops on a module that does not exist (F25), which reads as working in the tray while syncing nothing.
- R60. Reuse the chat-core pattern wholesale: server-authoritative download stream keyed by the per-tenant `l1_audit_log` seq (the cursor already exists), shape-scoped to what the device may hold (own subject rows plus granted scopes, sync_class shareable only), and an idempotent client-to-server outbox keyed by `event_id` (the unique replay guard from migration 0005 already makes uploads exactly-once). This is also exactly ElectricSQL's architectural split (shape-based reads, your-API writes), so the design has independent validation.
- R61. Client store: SQLite with the through-the-DB write pattern: `memory_synced` tables (immutable, written only by the sync stream), `memory_local` shadow tables (pending changes), a combining view, and INSTEAD OF triggers appending to a local `changes` outbox that the sync worker drains. Instant local reads and writes, offline by construction, no bespoke merge engine.
- R62. Derived data is never uploaded: clients sync raw memory text, ops, and tombstones; the server recomputes embeddings, summaries, facts, and graph edges on ingest (the rebuildable-lens invariant already guarantees this is safe). Optionally ship embeddings down read-only plus sqlite-vec for on-device semantic search, keyed by `embed_model_version`.
- R63. Conflict policy: per-column last-write-wins with server timestamp plus a `revision` integer; the outbox handler rejects stale revisions with 409 and the client rebases. Local-first practice (Linear, Figma, the Electric write guide) converged on server-arbitrated simplicity for exactly this data shape; real conflicts are rare for single-owner memory rows.
- R64. Do not adopt CRDTs for memory rows. They solve concurrent collaborative editing, which memory does not have (single writer per row, server-derived everything else), and they complicate audit. If collaboratively edited memory notes ever ship, scope Automerge 3 or Loro to those documents only.
- R65. Enforce `sync_class` server-side at the shape boundary (private rows never leave the owner's devices; shareable rows follow access grants), not just in the Python client (`sync_class.py`); add an RLS-backed test.
- R66. Desktop at-rest encryption: SQLCipher (AES-256) with the key in the OS keystore via the keyring crate, WAL mode, 0600 permissions including WAL/journal files; document the recovery story (re-hydrate from server on key loss).
- R67. Offline capture: interaction events and quick-capture notes queue in the local outbox with client-generated UUIDv7 `event_id`s; replays collide harmlessly with the unique guard. Document this contract in the task so module authors rely on it.
- R68. Initial hydration: snapshot of the hot window (30d) plus profiles and current summaries, then cursor streaming; cold history stays server-side and fetches on demand by `audit_row_id` (the recall API already supports that shape).
- R69. Keep `conflicts.py` sibling detection for the file store (Dropbox-class safety net), but mark it legacy once first-party sync ships; surface any detected sibling into the doctor report as today.
- R70. Multi-device chain identity: add `device_id` to pushed chain rows so per-device segments interleave attributably in cloud L1; the anchor verify already tolerates this because anchors are per-row.
- R71. Transport: device-scoped JWTs (auth module) with short expiry and refresh, TLS only, per-device rate limits, resumable uploads; the existing supervisor backoff and circuit breaker carry over unchanged.
- R72. Sync test matrix in CI: offline-create/edit/delete x concurrent-server-change x crash-mid-drain x replay, plus a chaos suite mirroring the Python one; assert convergence and no duplicate chain rows (the chat-core test style applies directly).

### G. Security, privacy, and compliance (R73-R90)

- R73. (P0, critical) Replace header-trust with TASK-AUTH-004 JWT verification middleware on `/v1/memory/*`: tenant and subject come from verified claims; `x-tenant-id` remains only as an internal hop header signed by the gateway if needed. Add a negative test proving a forged header cannot cross tenants (F15).
- R74. (P0) Make RLS fail-closed: drop the `IS NULL` arm from every policy; require the GUC explicitly; replace the nil-UUID magic bypass with a dedicated `cyberos_memory_admin` role used only by rebuild/backfill, so a forgotten `tenant_tx` yields zero rows instead of all rows (F16).
- R75. (P0) Add RLS with FORCE to `l1_audit_log`, `l2_memory`, `l2_entity`, `l2_edge`, and extend the RLS property gate plus a scheduled cross-tenant probe to memory tables (deep-audit R15) (F17).
- R76. Keep `tenant_id` leading every composite index (already true for new tables; verify `l1_audit_log` query plans under RLS since its PK is bare `seq`), and add an RLS performance smoke so policies do not silently 100x a query.
- R77. Per-principal rate limits on recall/search (token bucket per subject and per tenant) plus a global concurrency cap; recall is an extraction oracle without them (F18).
- R78. Ship real PII protection where content enters the cloud: a Presidio sidecar (analyzer + anonymizer) invoked by the Rust ingest path before any content-bearing body is chained, with the custom VN recognizers TASK-MEMORY-111 specifies (CCCD/CMND ids, VN phone formats, addresses), held to the task's >= 99.5% recall bar on a labeled set; store `pii_flags[]` per row to drive retention and masking (F20).
- R79. Trust-scored provenance: every memory row carries `source_kind` (user | agent | tool | external) and a trust score that decays ranking for low-trust sources; recall snippets are wrapped as quoted data in agent prompts (never interpolated as instructions), and prompts state that memory content is untrusted input (F24).
- R80. Quarantine new memories: facts younger than a soak window (or from low-trust sources) are excluded from recalls that feed privileged actions (evaluations, money, deploys) until aged or human-approved; expose `include_quarantined` only to admin tooling.
- R81. Retention policy engine: per-kind and per-event-class TTLs (presence/view events weeks, content events years, facts until invalidated), a policy table that is itself versioned and chained, and a reaper that archives to cold object storage rather than deleting Layer 1 within its legal window (F22).
- R82. Solve erasure with crypto-shredding before content capture scales: encrypt content-bearing bodies (and dereferenced content embeddings' source text, where kept) with a per-subject DEK wrapped by a tenant KEK (KMS); DSAR erasure = destroy the DEK, physically delete the subject's vectors and re-summarize affected windows, and record the erasure event on the chain. The chain stays append-only and verifiable (hashes remain; plaintext is gone). Two implementation cautions from the 2026 literature: HNSW soft-deletes leave reconstructible ghost vectors, so erase means delete + reindex the partial index; and backups need a re-deletion-on-restore ledger (F21).
- R83. Vietnam PDPL compliance pack (Law 91/2025/QH15, effective 2026-01-01, Decree 356/2025): DPIA for the monitoring program; a Cross-border Transfer Impact Assessment within 60 days of first transfer, since Supabase SG + Vultr SG hosting VN employees' data is a cross-border transfer; consent records mapped to the existing acknowledgment ledger; encryption obligations mapped to R82/R84. Fines reach 5% of prior-year revenue for cross-border violations, so this is a business risk item as much as a legal one. Pair with the brain-evaluation-plan's Phase 0 governance notice.
- R84. Field-level encryption for the most sensitive kinds (credentials-adjacent facts, HR notes): app-layer AES-GCM with per-tenant KMS-wrapped DEKs, or pgcrypto where queryability is not needed; keys never in env files.
- R85. Audit reads: every recall writes a chained `view`-class row (caller, query hash, hit count, subjects touched). The evaluation program cannot defend itself in a dispute without "who searched whom" (F23).
- R86. Pull forward external chain anchoring and the nightly chain-integrity walker (deep-audit R20/R21): publish the chain head signature outside the database on a schedule. Every provenance guarantee in recall inherits its strength from this.
- R87. Enforce the denylist in Rust: port the Python denylist tests (secrets, tokens, key material must never be chained) to the ingest path so a module emitting a secret in `attributes` is rejected at validate time.
- R88. Harden the admin binary: `cyberos-memory-admin rebuild|reconcile|backfill` requires the dedicated admin role, logs a chained row per invocation with operator identity, and refuses to run against prod without a break-glass flag.
- R89. Backups: encrypted, retention-scheduled, with the erasure ledger replayed on any restore so deleted subjects stay deleted (beyond-use posture regulators expect).
- R90. Compliance evidence cadence: quarterly access-grant reviews, RLS probe reports, PII-recall measurements, and erasure-drill results filed as chained artifacts; this is the SOC2-track paper trail an enterprise buyer will ask for.

### H. Performance and operations (R91-R100)

- R91. Replace per-tick tenant discovery scans with a `memory_tenant_registry` table maintained on first emit (or LISTEN/NOTIFY on insert), removing two DISTINCT scans per second at steady state (F30).
- R92. Parallelize per-tenant ingest with a bounded semaphore (fairness across tenants, cap total gateway concurrency); today one slow tenant delays all others in the serial loop.
- R93. Move summarization off the ingest hot path: enqueue touched scopes into a `brain_summary_queue` (tenant, scope, dirty count) and drain it on the maintenance tick; combined with R26 this removes the 3-COUNT-per-event pattern (F29).
- R94. Actually create the `l2_memory` HNSW index (commented out since migration 0001) or delete the code path that implies it; measure either way.
- R95. Document and test the pooling posture: transaction-local `set_config` works under transaction pooling, but pin it with an integration test against PgBouncer/Supavisor in transaction mode, and standardize on one GUC name (`app.tenant_id`) across eval and memory to end the set-both workaround (F35).
- R96. Set SLOs and enforce them in CI: recall p95 < 300 ms warm-cache at 1M hot rows, ingest lag p95 < 5 s, summary staleness < 10 min; a nightly perf smoke against a synthetic corpus tracks release-over-release.
- R97. Backpressure and DLQ: alert when `pending_embed_retry` exceeds a threshold or ages beyond a bound; poison events (repeatedly malformed) park in a dead-letter state with an ops-tile card instead of retrying forever.
- R98. Property-test the idempotency envelope: crash between UPSERT and cursor advance, duplicate seq delivery, out-of-order seq, gateway flapping mid-batch; the Python suite's style applied to the Rust worker.
- R99. Partition for scale: monthly partitions on `l1_audit_log` and time-or-tier partitioning on `brain_event_embedding` once row counts justify it, with pg_partman; partial indexes already bound the ANN side.
- R100. Fix the mislabeled metric (`Malformed` -> `embed_malformed`, not `postgres_error`, `ingest_worker.rs:68`) and add per-leg recall latency spans so slow legs are attributable (F31).

### I. Product surface and integration (R101-R108)

- R101. Ship an MCP memory server for the BRAIN (recall, remember, feedback tools) so Claude-family agents, Cursor, and Cowork consume CyberOS memory natively; the Python file-store MCP server (`runtime/mcp/cyberos_memory_server.py`) is the in-house precedent, and Anthropic's reference memory server defines the tool shapes.
- R102. Add a memory-tool adapter: expose a path-addressed `/memories` file view (list/read/write per Anthropic's memory tool contract) backed by brain facts and profiles, so any agent runtime that speaks the file protocol gets CyberOS memory without custom integration.
- R103. Wire the day-1 emitters (TASK-MEMORY-122): chat messages, auth events, proj activity, obs incidents, each with `content_ref` pointers into their owning stores, and wire the real consent gate to the TASK-EVAL-001 acknowledgment ledger at the same time (F19). Capture volume is the raw material for everything else in this plan.
- R104. Console memory tile: recall search UI with provenance chips (chain-verified badge, source row links), feedback thumbs (feeds R47), and the ops dashboard (R51); this is also the transparency surface employees see, which the governance plan treats as a trust requirement.
- R105. Lumi reads memory only through the recall API with her own subject identity and access grants, never raw SQL; every evaluation cites `audit_row_id`s (TASK-EVAL-003), and the citation check is enforced at the evaluation engine, not by convention.
- R106. Keep the personal-versus-company boundary explicit: `sync_class: private` rows never leave the owner's devices in plaintext (R65/R66); the enterprise BRAIN indexes only consented, shareable material; publish this split in the employee-facing notice.
- R107. Per-module namespaces and budgets: memory kinds and spend caps per emitting module via gateway policy, so one chatty module cannot starve embedding budget or pollute recall for everyone.
- R108. Write the module-author guide: how to emit events, what belongs in `attributes` versus `content_ref`, how to call recall with scopes, and the poisoning rules (memory output is data, not instructions). Add it to `docs/` beside the task index and link it from AGENTS.md.

## 7. Target architecture

```
capture (consent-gated, PII-scanned)                       serving
┌──────────────────────────────┐                ┌──────────────────────────────┐
│ module emitters (chat, proj, │   append-only  │  /v1/memory/recall (JWT)     │
│ obs, auth, desktop capture)  ├──────────────► │  hybrid: lexical + vector +  │
│ content_ref pointers         │  l1_audit_log  │  graph + summaries, RRF,     │
└──────────────────────────────┘  (hash chain,  │  rerank, MMR, decay/importance│
                                   RLS, anchors)│  access-scoped, provenance   │
        ▲  outbox (event_id idempotent)         └──────────────┬───────────────┘
        │                                                      │ feedback
┌───────┴───────────┐   seq/pos stream   ┌─────────────────────▼─────────────┐
│ devices: Tauri +  │◄───────────────────┤ BRAIN derived plane (rebuildable) │
│ SQLite (synced/   │   shapes, sync_    │ episodes → facts → summaries →    │
│ local/outbox,     │   class enforced   │ profiles; bi-temporal l2 graph;   │
│ SQLCipher)        │                    │ tiers; embeddings via ai-gateway  │
└───────────────────┘                    └─────────────────────┬─────────────┘
                                                               │ nightly, gated
                                          ┌────────────────────▼──────────────┐
                                          │ evolution loop: dream detectors,  │
                                          │ dedup/contradiction, promotion,   │
                                          │ golden-set evals + LLM judge,     │
                                          │ GEPA prompt tuning, A/B configs,  │
                                          │ self-healing jobs, ops dashboard  │
                                          └───────────────────────────────────┘
```

The load-bearing invariants stay exactly as designed: Layer 1 is the only truth, everything derived is rebuildable, every derived artifact points back to chain rows, and all model calls go through the ai-gateway. The additions are the fact layer, the hybrid ranking stack, the sync plane, and the measurement loop around all of it.

## 8. The auto-evolution loop, concretely

Auto-evolution means the module improves itself under evidence gates, in the same discipline as the CSAF/CDS loop and the AUTO_WORK protocol. The loop has four legs, and every leg already has a hook in the codebase.

Measure. The golden recall set plus the LLM judge (R45/R46) run in CI and nightly. Usage signals (R47) and shadow evals (R54) grade production behavior. The internal LongMemEval-style benchmark (R56) grades ability classes per release. Everything lands on the ops tile (R51) and in the weekly report (R53).

Learn. The dream detectors (R33) mine the corpus for duplicates, stale facts, contradictions, and patterns; the promotion job (R28) distills episodes into facts; ACE-style curation (R55) maintains procedural playbooks from session ledgers and gate logs.

Tune. GEPA optimizes the prompts (R49); the A/B table optimizes retrieval parameters (R48); importance and decay weights re-fit against feedback data on a slow cadence. Every change ships as a PR with eval evidence, through the same testing-to-done gate as code.

Heal. Self-healing jobs (R52) re-embed, re-summarize, re-dedupe, and reindex, each bracketed by before/after evals with auto-revert (R35), each writing ledger entries. Drift sentinels (R43) and the chain walker (R86) watch the substrate itself.

The gate rule that makes this safe to leave running: no consolidation, prompt, or config change reaches the serving path without a passing golden-set delta, and anything the loop applies is reversible (versioned rewrites, flag flips, DEK-preserving archives). That is the same evidence-gate posture as the rest of CyberOS, so the memory loop can eventually run inside the existing AUTO_WORK sessions rather than as a separate machine.

## 9. Phased roadmap

Phase 0, safety and truth (about 1-2 weeks of focused work). R73 JWT auth, R74/R75 fail-closed RLS everywhere, R77 rate limits, R9 confidence-floor fix, R10 snippets + batched verify/access (R18/R19), R44 gateway embeddings route + contract test, R100 metric fix, R59 sync decision, R45 golden-set runner skeleton. Acceptance: forged-header test fails closed; cross-tenant probe zero rows; recall returns non-empty snippets with real confidence; golden runner executes in CI with 25 seed cases.

Phase 1, a brain that understands content (2-4 weeks). R6 content-aware ingestion behind consent + R78 PII sidecar, R4/R5 fact layer with op pipeline, R25/R26/R27 abstractive incremental hierarchical summaries, R2 ranking + R11 lexical leg + R12 rerank + R13 MMR + R14 contextual embedding, R31 importance, R103 day-1 emitters + real consent gate, R37-R41 embedding lifecycle basics. Acceptance: golden-set recall@10 doubles versus Phase 0 baseline on content questions; p95 recall < 300 ms; every fact carries provenance and `valid_at`.

Phase 2, sync + compliance (3-5 weeks, parallelizable with Phase 1 tail). R60-R63 first-party sync on the chat-core pattern, R65/R66 sync-class enforcement + SQLCipher, R67/R68 offline capture + hydration, R72 sync test matrix, R81 retention engine, R82 crypto-shredding + erasure drill, R83 PDPL pack (DPIA + CTIA filings), R85 recall audit rows, R86 chain anchoring pulled forward. Acceptance: laptop offline for a day converges cleanly with zero duplicates; a test subject's erasure removes plaintext and vectors and survives a backup restore; CTIA filed.

Phase 3, evolution on (2-4 weeks, then permanent). R33 dream loop gated-live, R28/R29/R30 promotion + dedup + contradiction, R47/R24 feedback loop, R48 A/B, R49 GEPA, R52 self-healing registry, R53 weekly report, R56 benchmark, R58 poisoning suite, R101/R102 MCP + memory-tool surfaces, R104 console tile. Acceptance: one full month of unattended loop operation with every applied change carrying eval evidence and zero manual reverts.

Suggested task numbering for net-new work: TASK-MEMORY-125 auth+RLS hardening, 126 fact layer + op pipeline, 127 content-aware ingestion + PII sidecar, 128 hybrid ranking stack, 129 first-party sync, 130 retention + erasure + PDPL, 131 evolution loop + evals, 132 MCP/memory-tool surface. Existing TASK-MEMORY-112..120 fold into these rather than shipping file-store-only.

## 10. What not to do

Do not adopt a graph database; the relational `l2_edge` plus recursive CTEs and bi-temporal columns reproduces Graphiti's results on Postgres, and the AGE removal decision already went this way. Do not use CRDTs for memory rows (R64); the data is single-writer with server-derived artifacts, and audit clarity beats merge cleverness. Do not let any consolidation rewrite content without version lineage and a rewrite-depth cap; the 2026 failure literature on self-rewriting memory is consistent and ugly. Do not trust vendor benchmark numbers (the Mem0-versus-Zep LoCoMo dispute is a cautionary tale); own an internal benchmark. Do not capture more content before the consent gate is real and the PII pass exists in the cloud path; the governance plan's Phase 0 ordering is correct. Do not build a second store of record anywhere; DEC-2703's single-chain rule is the best decision in the module and every new feature should be forced through it.

## 11. Key sources

Mem0 pipeline and ops (docs.mem0.ai; arXiv 2504.19413 ecosystem), Letta memory blocks + sleep-time compute (letta.com/blog/sleep-time-compute), Zep/Graphiti bi-temporal graph (arXiv 2501.13956; help.getzep.com/graphiti), LongMemEval (arXiv 2410.10813), MemBench (arXiv 2506.21605), RAPTOR (arXiv 2401.18059), Generative Agents scoring/reflection (arXiv 2304.03442), A-MEM (arXiv 2502.12110), MIRIX (arXiv 2507.07957), HippoRAG 2 (arXiv 2502.14802), Anthropic contextual retrieval (anthropic.com/news/contextual-retrieval) and context engineering (anthropic.com/engineering/effective-context-engineering-for-ai-agents), Anthropic memory tool (platform.claude.com docs), OpenAI ChatGPT memory/Dreaming V3 analyses (manthanguptaa.in/posts/chatgpt_memory), ElectricSQL write patterns (electric-sql.com/docs/guides/writes), PowerSync + Supabase (powersync.com), Automerge 3 (automerge.org/blog/automerge-3), pgvector HNSW/halfvec guidance (github.com/pgvector/pgvector), Supabase Matryoshka adaptive retrieval (supabase.com/blog/matryoshka-embeddings), GEPA (arXiv 2507.19457; dspy.ai), OTel GenAI semconv (opentelemetry.io/docs/specs/semconv/gen-ai), Presidio (microsoft.github.io/presidio), MINJA memory injection (arXiv 2503.03704), HNSW ghost-vector erasure caveat (arXiv 2606.18497), self-rewriting memory degradation (arXiv 2605.12978), Vietnam PDPL 91/2025 analyses (tilleke.com; luatvietnam.vn), AWS RLS multi-tenant guidance (aws.amazon.com/blogs/database).

## 12. Verification appendix

Findings were grounded in direct reads of: `services/memory/src/brain/{mod,recall,summarize,tiering,ingest_worker,handler,access_scope,embed_client,event_cursor}.rs`, `src/interaction/{emit,consent_gate,content_ref}.rs`, `src/main.rs`, migrations 0001/0003/0005/0006/0007/0008, `desktop/src-tauri/src/sync_supervisor.rs`, the `modules/memory/cyberos/core` module map (decay, importance, mmr, ranking, consolidate, semantic, memory_sync, conflicts, pre_ingest_pii, crypto_mode, sth, episode, dream/*), `memory.invariants.yaml`, `docs/tasks/memory/README.md`, and the two companion strategy docs. Line references are as of commit state on 2026-07-06. Research claims come from the sources in section 11; vendor-reported benchmark numbers are marked contested where they are.



