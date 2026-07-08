# Improvement backlog migration map (2026-07-08)

One-time renumber of the three docs/improvement backlogs into feature-requests as
`class: improvement` FRs. No `legacy_id` was kept on the FRs (operator choice); this
table is the only record linking old ids to new. Use it to reconcile the in-flight
`auto/memory-enterprise` and `auto/chat-enterprise` branches after merge.

## memory

| old | new | status | title |
|---|---|---|---|
| MEM-001 | FR-MEMORY-201 | draft | JWT auth on /v1/memory endpoints (kill header-trust identity) |
| MEM-002 | FR-MEMORY-202 | draft | Fail-closed RLS on brain tables (drop NULL-GUC arm + nil-uuid bypass) |
| MEM-003 | FR-MEMORY-203 | draft | RLS with FORCE on l1_audit_log, l2_memory, l2_entity, l2_edge + cross- |
| MEM-004 | FR-MEMORY-204 | draft | Per-principal and per-tenant rate limits on recall/search |
| MEM-005 | FR-MEMORY-205 | draft | Fix dead recall confidence floor (use real cosine, not constant 1.0) |
| MEM-006 | FR-MEMORY-206 | draft | Batched candidate pipeline - snippets filled, one-query verify, set-ba |
| MEM-007 | FR-MEMORY-207 | draft | Light up ai-gateway /v1/embeddings route + memory-side contract test |
| MEM-008 | FR-MEMORY-208 | draft | Metrics correctness - embed_malformed label, per-leg recall latency sp |
| MEM-009 | FR-MEMORY-209 | draft | Golden recall eval runner + 25 seed cases wired into CI |
| MEM-010 | FR-MEMORY-210 | draft | Architecture ADRs - single memory plane, Rust-in-Tauri sync, no CRDTs  |
| MEM-011 | FR-MEMORY-211 | draft | Pin pooled-connection GUC behavior; standardize on app.tenant_id |
| MEM-012 | FR-MEMORY-212 | draft | brain_fact table + memory_kind taxonomy (episodic/semantic/procedural/ |
| MEM-013 | FR-MEMORY-213 | draft | Fact write pipeline - extract then ADD/UPDATE/DELETE/NOOP with chained |
| MEM-014 | FR-MEMORY-214 | draft | Content-aware ingestion - dereference content_ref pointers under RLS,  |
| MEM-015 | FR-MEMORY-215 | draft | PII sidecar (Presidio + VN recognizers) in the cloud ingest path |
| MEM-016 | FR-MEMORY-216 | draft | Abstractive summaries via gateway chat (extractive digest stays as deg |
| MEM-017 | FR-MEMORY-217 | draft | Incremental summarization - queue off hot path, window-bounded scopes, |
| MEM-018 | FR-MEMORY-218 | draft | Summary hierarchy - leaf/mid/root with subject profiles as default inj |
| MEM-019 | FR-MEMORY-219 | draft | Lexical retriever leg in brain recall (tsvector + pg_trgm) fused via R |
| MEM-020 | FR-MEMORY-220 | draft | Cross-encoder rerank stage (BGE via embed-sidecar) over fused top-50 |
| MEM-021 | FR-MEMORY-221 | draft | Park scoring + MMR - relevance/importance/recency weights, access_coun |
| MEM-022 | FR-MEMORY-222 | draft | Contextual embedding - situating prefix (who/where/when/kind) at embed |
| MEM-023 | FR-MEMORY-223 | draft | Query rewriting - relative-time parsing (EN+VN) and subject-handle exp |
| MEM-024 | FR-MEMORY-224 | draft | Write-time importance scoring with anchored rubric (FR-MEMORY-114) |
| MEM-025 | FR-MEMORY-225 | draft | Day-1 emitters (chat/auth/proj/obs) + real consent gate wired to FR-EV |
| MEM-026 | FR-MEMORY-226 | draft | Embedding lifecycle - halfvec, content_hash+embedded_at, batch embeds, |
| MEM-027 | FR-MEMORY-227 | draft | Warm-tier reachable on drill (honor documented behavior) |
| MEM-028 | FR-MEMORY-228 | draft | Recall API v1.1 - keyset pagination, scores breakdown in explain, feed |
| MEM-029 | FR-MEMORY-229 | draft | Query-embed LRU cache + EmbedClient in AppState |
| MEM-030 | FR-MEMORY-230 | draft | Bi-temporal l2_edge - valid_at/invalid_at/expired_at + invalidation no |
| MEM-031 | FR-MEMORY-231 | draft | LLM entity extraction + resolution, graph recall leg, query-shape rout |
| MEM-032 | FR-MEMORY-232 | draft | Sync server - shape-scoped seq stream down + idempotent outbox endpoin |
| MEM-033 | FR-MEMORY-233 | draft | Tauri SQLite client - synced/local tables, combining views, changes ou |
| MEM-034 | FR-MEMORY-234 | draft | Derived-data policy - server recomputes embeddings/summaries; optional |
| MEM-035 | FR-MEMORY-235 | draft | Desktop at-rest encryption - SQLCipher + OS keychain + file perms |
| MEM-036 | FR-MEMORY-236 | draft | Offline capture outbox + hydration (hot window snapshot, cold on deman |
| MEM-037 | FR-MEMORY-237 | draft | Device identity - device_id on chain rows, device-scoped JWTs, per-dev |
| MEM-038 | FR-MEMORY-238 | draft | Sync test matrix + chaos suite (offline x concurrent x crash x replay) |
| MEM-039 | FR-MEMORY-239 | draft | Retention policy engine + reaper (per-kind TTLs, archive not delete) |
| MEM-040 | FR-MEMORY-240 | draft | Erasure - crypto-shredding (per-subject DEKs), lineage cascade, ghost- |
| MEM-041 | FR-MEMORY-241 | draft | PDPL 91/2025 pack - DPIA, CTIA draft, consent mapping, evidence cadenc |
| MEM-042 | FR-MEMORY-242 | draft | Recall read-audit rows (who searched whom) on the chain |
| MEM-043 | FR-MEMORY-243 | draft | External chain anchoring + nightly chain-integrity walker |
| MEM-044 | FR-MEMORY-244 | draft | Rust denylist enforcement at emit/ingest + admin binary hardening |
| MEM-045 | FR-MEMORY-245 | draft | Calibrated LLM judge + golden-set growth + internal LongMemEval-style  |
| MEM-046 | FR-MEMORY-246 | draft | Usage-signal loop - retrieved/used/cited counters, used-ratio demotion |
| MEM-047 | FR-MEMORY-247 | draft | Dream loop on the BRAIN - four detectors, proposal queue, gated applie |
| MEM-048 | FR-MEMORY-248 | draft | Promotion + dedup + contradiction jobs, eval-gated with auto-revert |
| MEM-049 | FR-MEMORY-249 | draft | Retrieval config A/B + shadow evaluation on sampled live traffic |
| MEM-050 | FR-MEMORY-250 | draft | GEPA prompt optimization for extraction/summary/judge prompts |
| MEM-051 | FR-MEMORY-251 | draft | Self-healing job registry + drift sentinels + backpressure/DLQ |
| MEM-052 | FR-MEMORY-252 | draft | Telemetry to standard + memory-ops console tile + weekly auto-report |
| MEM-053 | FR-MEMORY-253 | draft | Poisoning defenses - source trust scores, quarantine, quoted-data prom |
| MEM-054 | FR-MEMORY-254 | draft | Calibrated abstention - confidence output + not-in-memory behavior + f |
| MEM-055 | FR-MEMORY-255 | draft | MCP memory server + Anthropic memory-tool file adapter |
| MEM-056 | FR-MEMORY-256 | draft | Perf + scale pass - tenant registry, parallel ingest, l2 HNSW decision |
| MEM-057 | FR-MEMORY-257 | draft | Module-author guide + per-module namespaces and spend budgets |
| MEM-058 | FR-MEMORY-258 | draft | Lumi integration - recall-API-only access, mandatory citations, person |

## chat

| old | new | status | title |
|---|---|---|---|
| T-001 | FR-CHAT-201 | draft | Rate limiting (subject + IP layers) |
| T-002 | FR-CHAT-202 | draft | JWT hardening: aud always, iss pin, JWKS refetch on unknown kid |
| T-003 | FR-CHAT-203 | draft | Account-wide socket kill on revoke (<60 s) |
| T-004 | FR-CHAT-204 | draft | BYTEA attachment closeout (fs-only prod) |
| T-005 | FR-CHAT-205 | draft | Edit-history revisions table |
| T-006 | FR-CHAT-206 | draft | WS heartbeat + idle reaper |
| T-007 | FR-CHAT-207 | draft | External synthetic probe (login-send-receive) |
| T-008 | FR-CHAT-208 | draft | Metrics baseline + core alerts (Prom/Grafana/AM) |
| T-009 | FR-CHAT-209 | draft | Staging compose profile + seeded tenant |
| T-010 | FR-CHAT-210 | draft | Go-live checklist doc (living) |
| T-011 | FR-CHAT-211 | draft | chat_events log + per-tenant seq (transactional) |
| T-012 | FR-CHAT-212 | draft | Seq-stamped ws frames + client gap detection |
| T-013 | FR-CHAT-213 | draft | POST /v1/chat/sync v1 (pos, subscriptions, initial/limited, reset) + l |
| T-014 | FR-CHAT-214 | draft | Idempotent send: client_msg_id + unique index + echo reconcile |
| T-015 | FR-CHAT-215 | draft | chat-core package skeleton + web SQLite/OPFS adapter (IndexedDB fallba |
| T-016 | FR-CHAT-216 | draft | Persistent outbox (FIFO, backoff, restart-safe, visible states) + conn |
| T-017 | FR-CHAT-217 | draft | Store-driven rendering: messages/channels/read state + drafts from cha |
| T-018 | FR-CHAT-218 | draft | Single account-scoped multiplexed socket (subscribe frames) |
| T-019 | FR-CHAT-219 | draft | Socket hardening: resume hint, backpressure, envelope v, origin/frame  |
| T-020 | FR-CHAT-220 | draft | Convergence + outbox property suites |
| T-021 | FR-CHAT-221 | draft | Offline end-to-end Playwright suite |
| T-022 | FR-CHAT-222 | draft | chat-core as CI citizen (typecheck/unit/property gates) |
| T-023 | FR-CHAT-223 | draft | Push relay worker (FCM v1 + APNs) + collapse keys + badges |
| T-024 | FR-CHAT-224 | draft | Web push: VAPID + declarative payload |
| T-025 | FR-CHAT-225 | draft | Quiet hours / DND schedule server-side |
| T-026 | FR-CHAT-226 | draft | Desktop: bundle SPA into Tauri + CSP (web + shell) |
| T-027 | FR-CHAT-227 | draft | Desktop: tauri-plugin-sql adapter + native notifications |
| T-028 | FR-CHAT-228 | draft | Desktop: tray/badge, deep links + single instance, window-state/shortc |
| T-029 | FR-CHAT-229 | draft | Desktop: updater keys/channels + crash reporting + CI matrix |
| T-030 | FR-CHAT-230 | draft | Version-skew policy: min_supported_client + blocking banner |
| T-031 | FR-CHAT-231 | draft | Error tracking client + server (GlitchTip/Sentry) |
| T-032 | FR-CHAT-232 | draft | Per-tenant feature flags + canary rollout habit |
| T-033 | FR-CHAT-233 | draft | Capacitor init (ios/ + android/ committed) + release lanes live |
| T-034 | FR-CHAT-234 | draft | Mobile: sqlite adapter + push plugin + deep-link tap-through |
| T-035 | FR-CHAT-235 | draft | Mobile UX pass + store readiness (privacy forms, deletion path) |
| T-036 | FR-CHAT-236 | draft | Attachments to object storage (presigned) |
| T-037 | FR-CHAT-237 | draft | Upload pipeline: resumable, EXIF/thumbs, AV scan, orphan GC |
| T-038 | FR-CHAT-238 | draft | Fan-out seam trait + LISTEN/NOTIFY backend + shared presence + push of |
| T-039 | FR-CHAT-239 | draft | Calls: coturn TURN + ring via account socket + push wake + call log +  |
| T-040 | FR-CHAT-240 | draft | PDPL ops: DSAR export, retention jobs, legal hold |
| T-041 | FR-CHAT-241 | draft | PDPL governance: data mapping/DPIA, transfer assessment, breach playbo |
| T-042 | FR-CHAT-242 | draft | Web: unread from event log (kill polls) + Chat.tsx refactor + virtuali |
| T-043 | FR-CHAT-243 | draft | Web: PWA completion, a11y AA + axe CI, perf budget, QoL batch |
| T-044 | FR-CHAT-244 | draft | Sync v1.1: cold-start snapshot, lists/windows, offline local search, l |
| T-045 | FR-CHAT-245 | draft | Ops drills: zero-downtime deploy, off-site backup + restore drill, run |
| T-046 | FR-CHAT-246 | draft | Load/fuzz/migration-gate/chaos/soak suites |
| T-047 | FR-CHAT-247 | draft | Observability completion: OTel traces, capacity dashboard, SLOs + erro |
| T-048 | FR-CHAT-248 | draft | Product wave 1: pins, saved-for-later, forward/quote, notif-override U |
| T-049 | FR-CHAT-249 | draft | Group DMs + scheduled send + reminders |
| T-050 | FR-CHAT-250 | draft | Link previews (SSRF-safe fetcher + cards) |
| T-051 | FR-CHAT-251 | draft | Product wave 2: slash commands, custom emoji, status/DND states, jump- |
| T-052 | FR-CHAT-252 | draft | AI set: catch-me-up/summaries/action items (streaming), semantic searc |
| T-053 | FR-CHAT-253 | draft | API contracts: error envelope, OpenAPI, generated TS client, ws frame  |
| T-054 | FR-CHAT-254 | draft | Ecosystem: incoming/outgoing webhooks, bot accounts, MCP tool surface |
| T-055 | FR-CHAT-255 | draft | Imports: Slack then Zalo (idempotent, checkpointed) |
| T-056 | FR-CHAT-256 | draft | Scale wave: Redis/NATS backend, load-shed order, partitioning, quotas, |
| T-057 | FR-CHAT-257 | draft | Security completion: upload hardening, device/session UI, secrets scan |
| T-058 | FR-CHAT-258 | draft | External penetration test (scope: chat + auth + uploads + ws) |
| T-059 | FR-CHAT-259 | draft | Encryption posture: honest security page + attachment encryption at re |
| T-060 | FR-CHAT-260 | draft | E2EE channel class on MLS (capture/AI/search off, labeled) |
| T-061 | FR-CHAT-261 | draft | Attachment offline cache policy + sync conformance fixture |
| T-062 | FR-CHAT-262 | draft | Calls extended: 1:1 screen share; SFU spike for group calls |
| T-063 | FR-CHAT-263 | draft | Notification extras: offline email digest, delivery-ticks decision, Za |
| T-064 | FR-CHAT-264 | draft | Share-to-chat intent (mobile share sheet) |
| T-065 | FR-CHAT-265 | draft | Admin data-governance panel (retention, holds, DSAR, consent, capture) |
| T-066 | FR-CHAT-266 | draft | Decision memos D1-D6 prepared for Stephen (one page each) |

## improvement

| old | new | status | title |
|---|---|---|---|
| IMP-001 | FR-IMP-001 | draft | Dependency audit in CI (cargo-audit + cargo-deny) |
| IMP-002 | FR-IMP-002 | draft | Refuse dev CORS in production boot |
| IMP-003 | FR-IMP-003 | draft | Secret scanning in CI and pre-push |
| IMP-004 | FR-IMP-004 | draft | Deploy observability stack to P0 |
| IMP-005 | FR-IMP-005 | draft | External uptime probes and alerting |
| IMP-006 | FR-IMP-006 | draft | Canary healthcheck and auto-rollback in deploy |
| IMP-007 | FR-IMP-007 | draft | apps/web test spine |
| IMP-008 | FR-IMP-008 | draft | Goldensets as first-class gate inputs |
| IMP-009 | FR-IMP-009 | draft | LLM call ledger in ai-gateway |
| IMP-010 | FR-IMP-010 | draft | Telemetry-to-FR bridge |
| IMP-011 | FR-IMP-011 | draft | Structured gate-failure taxonomy |
| IMP-012 | FR-IMP-012 | draft | Coverage measurement and ratchet |
| IMP-013 | FR-IMP-013 | draft | Cross-service contract tests |
| IMP-014 | FR-IMP-014 | draft | External audit-chain anchoring |
| IMP-015 | FR-IMP-015 | draft | Nightly chain-integrity monitor |
| IMP-016 | FR-IMP-016 | draft | Staging environment |
| IMP-017 | FR-IMP-017 | draft | OTLP tracing export |
| IMP-018 | FR-IMP-018 | draft | Prometheus metrics endpoints |
| IMP-019 | FR-IMP-019 | draft | SLO definitions and burn-rate alerts |
| IMP-020 | FR-IMP-020 | draft | FR outcome scoring |
| IMP-021 | FR-IMP-021 | draft | Rubric evals for LLM outputs, anchored judge |
| IMP-022 | FR-IMP-022 | draft | Ban defensive asserts |
| IMP-023 | FR-IMP-023 | draft | Groom draft FRs with value and confidence |
| IMP-024 | FR-IMP-024 | draft | Dream proposal ranking |
| IMP-025 | FR-IMP-025 | draft | Dream budget, latency and drift gates |
| IMP-026 | FR-IMP-026 | draft | Auto-revert on gate regression |
| IMP-027 | FR-IMP-027 | draft | Enable auto mode for docs/skills envelope |
| IMP-028 | FR-IMP-028 | draft | ACE-style skill curation loop |
| IMP-029 | FR-IMP-029 | draft | Paired-trajectory skill audits |
| IMP-030 | FR-IMP-030 | draft | QLoRA fine-tuning pilot (obs triage) |
| IMP-031 | FR-IMP-031 | draft | Unified error envelope crate |
| IMP-032 | FR-IMP-032 | draft | Extract cyberos-service-kit |
| IMP-033 | FR-IMP-033 | draft | Wire cloud router adapters in ai-gateway |
| IMP-034 | FR-IMP-034 | draft | Chat realtime fanout seam |
| IMP-035 | FR-IMP-035 | draft | unwrap/expect burn-down and panic removal |
| IMP-036 | FR-IMP-036 | draft | Finish and property-test audit-chain crate |
| IMP-037 | FR-IMP-037 | draft | OpenAPI generation per service |
| IMP-038 | FR-IMP-038 | draft | Extend RLS property gate, cross-tenant probe |
| IMP-039 | FR-IMP-039 | draft | Load and soak test suite |
| IMP-040 | FR-IMP-040 | draft | Mutation testing pilot on shared crates |
| IMP-041 | FR-IMP-041 | draft | Secrets inventory and rotation runbook |
| IMP-042 | FR-IMP-042 | draft | Rate limits beyond login |
| IMP-043 | FR-IMP-043 | draft | Supply-chain hardening (pin, SBOM, sign) |
| IMP-044 | FR-IMP-044 | draft | Automated dependency updates |
| IMP-045 | FR-IMP-045 | draft | Session and token security validation |
| IMP-046 | FR-IMP-046 | draft | Backup independence and restore drill |
| IMP-047 | FR-IMP-047 | draft | Rebuild-in-60-minutes runbook |
| IMP-048 | FR-IMP-048 | draft | Build caching with cargo-chef |
| IMP-049 | FR-IMP-049 | draft | Deploy events into audit chain |
| IMP-050 | FR-IMP-050 | draft | Client and service error tracking |
| IMP-051 | FR-IMP-051 | draft | Least-privilege DB roles and layout doc |
| IMP-052 | FR-IMP-052 | draft | Migration discipline CI check |
| IMP-053 | FR-IMP-053 | draft | pgvector operations plan |
| IMP-054 | FR-IMP-054 | draft | Generalized retention schedule |
| IMP-055 | FR-IMP-055 | draft | Spec-only module manifest |
| IMP-056 | FR-IMP-056 | draft | API versioning and deprecation policy |
| IMP-057 | FR-IMP-057 | draft | Frontend state and fetch consolidation |
| IMP-058 | FR-IMP-058 | draft | ADR backfill for irreversible decisions |
| IMP-059 | FR-IMP-059 | draft | Wiki link-integrity gate |
| IMP-060 | FR-IMP-060 | draft | Generated CONTINUE-HERE |
| IMP-061 | FR-IMP-061 | draft | BRAIN Phase 0 consent completion |
| IMP-062 | FR-IMP-062 | draft | Quarterly envelope review ritual |
| IMP-063 | FR-IMP-063 | draft | Track A: serve a chat model in P0 and verify AI end-to-end |
| IMP-064 | FR-IMP-064 | draft | Track B: desktop signing, auto-update, release verify |
| IMP-065 | FR-IMP-065 | draft | Track B: mobile shells and store release pipeline |
| IMP-066 | FR-IMP-066 | draft | Track C: brain activation rollout (deploy, notice, ack, capture) |
| IMP-067 | FR-IMP-067 | draft | Go-live readiness gate (safety nets before fully on) |

## memory (branch additions, reconciled 2026-07-08)

| old | new | status | note |
|---|---|---|---|
| MEM-059 | FR-MEMORY-259 | draft | added on auto/memory-enterprise (post-migration); brain_common harness fix |
| MEM-060 | FR-MEMORY-260 | draft | added on auto/memory-enterprise (post-migration); summarize scope-filter fix |
