# Changelog — OBS

## 2026-06-25 - obs triage tool renamed to conform to SEP-986 (TASK-MCP-003 slice 2)

TASK-MCP-003 slice 2 turned on SEP-986 naming enforcement at mcp-gateway registration, so the obs triage tool was renamed from `cyberos.obs.triage` to `cyberos.obs.execute_triage` (`execute` is an approved verb; `triage` alone had no `{verb}_{noun}` form). Only the tool ID changed - the triage behaviour, inputs, and verdict are identical. The constant `TRIAGE_TOOL_NAME` in `modules/cuo/cuo/triage_mcp_module.py` carries the new value and every call site follows it; the demo call in `run-demo.sh` and the obs-router README were updated to match. Anything calling the old name through `tools/call` must switch to `cyberos.obs.execute_triage`.

## 2026-06-24 - obs triage reachable as an mcp-gateway tool (TASK-OBS-007 x TASK-MCP-002)

The `obs.triage-alert` path obs-router already calls over HTTP is now also exposed on the mcp-gateway as the tool `cyberos.obs.triage`, so an alert can be triaged through `tools/call` and shows up in the desktop Tools tab alongside other federated module tools.

- **`modules/cuo/cuo/triage_mcp_module.py`** - the obs federation surface. It adopts the reference-module contract verbatim (serves the JSON-RPC the gateway forwards over `/mcp`, self-registers its catalogue at `/v1/mcp/register`, heartbeats, deregisters on shutdown). The registering module identity is `obs` and the tool is `cyberos.obs.triage`; the code lives in the cuo package because triage is a CUO skill. The tool body runs triage in-process through `triage_server.handle_triage_request` - the same pure handler the HTTP endpoint uses - so there is no second hop and no duplicated logic. With no LLM invoker on the host it serves the skill's safe-degrade verdict (confidence 0.0), so the demo needs no API key. A missing `alert` or `alert.name` is an in-band tool error (`isError: true`), not a transport failure; a successful triage returns the verdict as both a text block and `structuredContent`.
- **`modules/cuo/tests/test_triage_mcp_module.py`** - 8 tests with a fake invoker (no LLM, no network, no live gateway): the `tools/list` descriptor, the verdict projection through `tools/call`, the bad-alert in-band error, the no-invoker safe-degrade verdict, the unknown-tool error, and the registration body. The full cuo suite stays green (235 passed, 2 skipped).
- **`services/mcp-gateway/examples/run-demo.sh`** - the one-command demo now starts this module alongside the reference module, so `bash scripts/mcp_demo.sh` lights up `cyberos.obs.triage` end to end. Verified the serve path live in the sandbox (healthz, `tools/list`, `tools/call` happy + bad-alert paths). The Rust gateway itself compiles and runs on the build host.
- **`services/obs-router/README.md`** - documents the MCP-tool path next to the HTTP triage endpoint.

## 2026-06-20 - TASK-OBS-005 correlation completed in-repo (exemplars, metrics, log enrichment)

Built on the ai-gateway TraceContext boundary below. obs-sdk now carries the rest of TASK-OBS-005's correlation surface:

- Histogram exemplars (§1 #3): `exemplar::record_with_exemplar` records a `cyberos_duration_ms` sample so it links to its trace (the trace_id rides via the OTel context at the request boundary); `record_request` routes the duration through it.
- Correlation metrics (§1 #12): `obs_tracecontext_extracted_total{outcome}` (counted at the gateway boundary by extracted / missing_generated_new / malformed) and `obs_exemplar_emission_total`.
- Log enrichment (§1 #2): `logging::request_span` carries trace_id / span_id / tenant_id, and `init_json_subscriber` renders the span scope on every event, so every log line emitted while handling a request carries the trace_id - the `loki: {trace_id="..."}` query the whole design hinges on. The gateway instruments each request with the span and ships JSON logs. Verified with an in-memory capture.

With this, the obs module is feature-complete in-repo across TASK-OBS-001..009. The only remaining TASK-OBS-005 clause is the end-to-end correlation CI test (§1 #7), which asserts Loki + Tempo + Prometheus + LangSmith all hold the same trace_id for one synthetic call - owner-run, since it needs the live stack.

## 2026-06-20 - ai-gateway HTTP surface unblocks the obs-AI integration (TASK-OBS-003, 004, 005)

### What landed

The AI Gateway gained an HTTP serving surface (`services/ai-gateway/src/server` + `bin/cyberos_gateway`): an axum listener (`POST /v1/chat`, `/healthz`, `/metrics`) that binds the existing pipeline (policy loader, alias resolver, provider call) behind injectable `PolicySource` and `ChatBackend` seams, with an `EchoBackend` since the TASK-AI-008 provider adapters are still stubs. This is the surface the obs-AI tasks needed.

On it:

- **TASK-OBS-003** - the RED middleware (`tenant_ctx` + `red_mw` + `init`) now wraps the gateway, so RED covers all three CyberOS-authored HTTP services (auth, memory, ai-gateway). ADR-OBS-003-001 updated; only `chat` (a pinned image) remains deferred.
- **TASK-OBS-005** - a `trace_ctx` middleware ensures every request carries a W3C trace context (extract the inbound `traceparent` strictly, or generate one), stamps it as a request extension, and echoes it on the response for downstream correlation. Builds on the obs-sdk `tracecontext` primitive.
- **TASK-OBS-004** - the LangSmith export: a `langsmith` module (redaction newtypes, payload + 100KB truncation, opt-in gate, fire-and-forget dispatch, retry + `Idempotency-Key`, error/outcome taxonomy), a `langsmith_export` opt-in field on `AiPolicy` (default false), and the gateway handler dispatching the redacted, trace-correlated export when a tenant opts in.

### Gates

ai-gateway: 7 server tests + 5 langsmith tests pass; all test binaries compile. The full ai gate needs the dev Redis/Postgres stack (cache-isolation test) and is owner-run while docker is down this session. The LangSmith live POST and the opt-in redaction (Presidio) path are owner-run; the default opt-out path is tested.

---

## 2026-06-20 - compliance-view HTTP service + CUO triage endpoint (TASK-OBS-008, TASK-OBS-007)

### What landed

**`services/obs-compliance-view/` - TASK-OBS-008 read-only compliance views, I/O shell built.** The four views (eu-ai-act, pdpl, soc2, iso27001) are now served over HTTP on top of the pure core shipped earlier. `query.rs` reads `l1_audit_log` by tenant, audit kind, and time window (RLS GUC set per transaction); `summary.rs` is the per-kind count block (unit-tested); `main.rs` is the axum server wiring auth (external_auditor role) to tenant-scope enforcement, window validation, the kind-filtered query, the defence-in-depth PII scan, the Ed25519 chain-proof, and an `obs.compliance_view_accessed` access line. Endpoints: `GET /:view?since=&until=` plus `/healthz`.

**`services/memory/migrations/0004_l1_event_type.sql` - the unblock for the query.** A generated `event_type` column on `l1_audit_log`, backed by an immutable extractor that returns NULL for the non-JSON markdown bodies memory-file `put` rows carry, plus a `(tenant_id, event_type, ts_ns)` index. The compliance query filters by audit kind without an unsafe runtime `body::jsonb` cast. Live apply is owner-run.

**`modules/cuo/cuo/triage_server.py` - TASK-OBS-007 §1 #2, the CUO triage endpoint obs-router calls.** CUO runs skills in-process, so this is the thin HTTP front door that maps `{skill, alert}` to an `obs.triage-alert` invocation and returns the verdict contract `cuo_triage.rs` parses. Safe degradation is deliberate (SKILL.md §5): when triage cannot reach its inputs it returns HTTP 200 with confidence 0.0, so obs-router pages rather than the endpoint failing. `services/obs-router/README.md` documents running it and pointing `OBS_CUO_TRIAGE_URL` at it.

### Gates

obs: caf CLEAN, awh 11/11 100% (0.0 regression). cuo: caf CLEAN (192 passed), awh 2/2 100%. 45 new tests.

### Also in this wave

**Memory-chain audit writes (TASK-OBS-007 §1 #6, TASK-OBS-008 §1 #10).** New shared crate `services/shared/cyberos-audit-chain` (chain anchor byte-identical to memory's canonical and auth's `memory_bridge`; best-effort genesis-row insert). obs-router now writes `obs.alert_triaged` / `obs.alert_acked` to `l1_audit_log` off the request path when `DATABASE_URL` is set (else the log sink), and obs-compliance-view appends `obs.compliance_view_accessed` best-effort. The earlier log-line placeholders are gone.

**W3C TraceContext core (TASK-OBS-005 §1 #1, #4, #11).** TASK-AI-022 (gateway OTel trace emission) is already shipped, so TASK-OBS-004/005 are unblocked. `cyberos-obs-sdk/src/tracecontext.rs` ships the pure correlation primitive: strict version-00 `traceparent` parse/validate, format/inject/extract over axum headers, and a forensic `hash16` so a malformed value is logged as a hash, never raw. The `with_trace_context` wrapper, the log-enrichment layer, the exemplar, the end-to-end CI test, and TASK-OBS-004's LangSmith per-call export are deferred: they need the ai-gateway request-serving surface that does not exist in-repo (the same blocker as the RED ai-gateway deferral, ADR-OBS-003-001).

### Remaining for `shipped`

Live validation needs the real targets: a CUO host with an LLM invoker, the dev Postgres applying 0004, and the CHAT webhook plus PagerDuty routing key for obs-router. The migration was validated against the real PostgreSQL grammar (pglast/libpg_query) and the triage endpoint end-to-end over HTTP this session; applying 0004 to a live Postgres and the chain-reconcile check on obs-written rows remain owner-run.

---

## 2026-05-19 — P0 implementation wave — OBS collector slice-1 scaffold shipped (TASK-OBS-001)

See [AI changelog](../ai/changelog.html) for AI Gateway and [MCP changelog](../mcp/changelog.html) for MCP Gateway portions of this wave.

### What landed

**`services/obs-collector/`** — Rust workspace member, slice-1 of P0.2 OBS:

- **TASK-OBS-001 — OTel collector + LGTM stack — scaffold shipped (10/10), status flipped `planned → building`.** Slice-1 ship covers the canonical `config/otel-collector-config.yaml` matching TASK-OBS-001 §3 byte-for-byte (OTLP grpc:4317 + http:4318 receivers with `bearertokenauth` authenticator + resource processor + `attributes/pii_scrub` processor + batch 10s/1024 + Loki / Prometheus remote-write / Tempo exporters + `file_storage` extension + health_check on :13133). `cyberos-obs validate-config` + `validate-tokens` pre-flight CI gates. Self-metric name + label constants. Bearer-token file parser + `config/auth/tokens.example` template.
- **Remaining for `shipped` status:** the live LGTM deployment (Helm chart + docker-compose) lands at `deploy/obs/` next session.

---

## 2026-05-15 — OBS module page rewritten to Gold (observability spine + auto-runbook router + compliance evidence surface)

Rewrote `website/docs/modules/obs.html` to Gold by encoding three strategic roles: (1) three-pillars unified pane (logs/metrics/traces/AI-traces correlated by trace_id × tenant_id; pillar × signal table; cross-pillar correlation example; tenant query proxy isolation), (2) auto-runbook router (alerts → CUO triage skill → CHAT self-service OR PagerDuty escalation; severity × routing matrix; runbook-catalogue growth loop), (3) compliance evidence surface (per-regulator scoped read-only views over memory audit chain; YAML view definitions; chain-of-custody manifest with Ed25519 signature).

Key changes:
- Title/meta + hero reframed to 3 strategic roles
- Fact-grid extended (8→12 cards: + Correlation key, Auto-runbook coverage, Compliance surfaces, etc.)
- NEW §0 "The bigger picture" — 3-card layout + emitter/consumer Mermaid + 9-row auto-vs-human matrix
- NEW §2.5 "Three-pillars unified pane" — pillar × signal-type mapping table + concrete 5-step cross-pillar investigation walkthrough + tenant query proxy isolation guarantee
- NEW §2.6 "Auto-runbook router" — 6-step routing sequence Mermaid + severity × routing matrix (P0/P1/P2/P3/P4) + runbook-catalogue self-growth loop
- NEW §2.7 "Compliance evidence surface" — regulator × audit scope matrix (EU AI Act, PDPL, SOC 2, ISO 27001, GDPR, Vietnam Decree 13/2023) + per-view scoping YAML + chain-of-custody manifest with chain anchors
- Risks +10 (R-OBS-011..020): auto-runbook miscategorising P0 (Critical) · compliance export tampering (Critical) · triage skill down → page storm · LangSmith EU residency · trace sampling drops wrong tail · persona-drift false positive · OTel context propagation breaks · query proxy DOS · runbook catalogue drift · maintenance-window noise
- KPIs +10: auto-runbook coverage (≥ 60% by P1) · P0/P1 false-suppression (= 0 hard floor) · compliance export verification rate (= 1.0) · cross-pillar correlation completeness (≥ 0.95) · tail-sampling error coverage (= 1.0) · persona-drift detector precision · query proxy violations · self-service ticket MTTR · dogfooding alert ACK (we live by this) · compliance surfaces × regulator
- References expanded to universal-protocol scope: 4 in-page sections + 8 cross-module links + AUDIT_AND_PLAN §3.3 (P0 · slice 1 placement) + RESEARCH_REVIEW §6 (9/10) + MEMORY_AUTOSYNC_DESIGN.md §8 + task-audit skill + EU AI Act + ISO 27001 + ISO 42001 + SOC 2 + PDPL + Decree 13 + GDPR Art. 30
