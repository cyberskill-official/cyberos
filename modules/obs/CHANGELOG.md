# Changelog — OBS

## 2026-06-20 - compliance-view HTTP service + CUO triage endpoint (FR-OBS-008, FR-OBS-007)

### What landed

**`services/obs-compliance-view/` - FR-OBS-008 read-only compliance views, I/O shell built.** The four
views (eu-ai-act, pdpl, soc2, iso27001) are now served over HTTP on top of the pure core shipped earlier.
`query.rs` reads `l1_audit_log` by tenant, audit kind, and time window (RLS GUC set per transaction);
`summary.rs` is the per-kind count block (unit-tested); `main.rs` is the axum server wiring auth
(external_auditor role) to tenant-scope enforcement, window validation, the kind-filtered query, the
defence-in-depth PII scan, the Ed25519 chain-proof, and an `obs.compliance_view_accessed` access line.
Endpoints: `GET /:view?since=&until=` plus `/healthz`.

**`services/memory/migrations/0004_l1_event_type.sql` - the unblock for the query.** A generated
`event_type` column on `l1_audit_log`, backed by an immutable extractor that returns NULL for the
non-JSON markdown bodies memory-file `put` rows carry, plus a `(tenant_id, event_type, ts_ns)` index. The
compliance query filters by audit kind without an unsafe runtime `body::jsonb` cast. Live apply is owner-run.

**`modules/cuo/cuo/triage_server.py` - FR-OBS-007 §1 #2, the CUO triage endpoint obs-router calls.** CUO
runs skills in-process, so this is the thin HTTP front door that maps `{skill, alert}` to an
`obs.triage-alert` invocation and returns the verdict contract `cuo_triage.rs` parses. Safe degradation
is deliberate (SKILL.md §5): when triage cannot reach its inputs it returns HTTP 200 with confidence 0.0,
so obs-router pages rather than the endpoint failing. `services/obs-router/README.md` documents running it
and pointing `OBS_CUO_TRIAGE_URL` at it.

### Gates

obs: caf CLEAN, awh 11/11 100% (0.0 regression). cuo: caf CLEAN (192 passed), awh 2/2 100%. 45 new tests.

### Remaining for `shipped`

Live validation needs the real targets: a CUO host with an LLM invoker, the dev Postgres applying 0004,
and the CHAT webhook plus PagerDuty routing key for obs-router. Writing `obs.alert_triaged` /
`obs.compliance_view_accessed` to the memory audit chain (rather than the current log line) is the
follow-up across both services.

---

## 2026-05-19 — P0 implementation wave — OBS collector slice-1 scaffold shipped (FR-OBS-001)

See [AI changelog](../ai/changelog.html) for AI Gateway and [MCP changelog](../mcp/changelog.html) for MCP Gateway portions of this wave.

### What landed

**`services/obs-collector/`** — Rust workspace member, slice-1 of P0.2 OBS:

- **FR-OBS-001 — OTel collector + LGTM stack — scaffold shipped (10/10), status flipped `planned → building`.** Slice-1 ship covers the canonical `config/otel-collector-config.yaml` matching FR-OBS-001 §3 byte-for-byte (OTLP grpc:4317 + http:4318 receivers with `bearertokenauth` authenticator + resource processor + `attributes/pii_scrub` processor + batch 10s/1024 + Loki / Prometheus remote-write / Tempo exporters + `file_storage` extension + health_check on :13133). `cyberos-obs validate-config` + `validate-tokens` pre-flight CI gates. Self-metric name + label constants. Bearer-token file parser + `config/auth/tokens.example` template.
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
- References expanded to universal-protocol scope: 4 in-page sections + 8 cross-module links + AUDIT_AND_PLAN §3.3 (P0 · slice 1 placement) + RESEARCH_REVIEW §6 (9/10) + MEMORY_AUTOSYNC_DESIGN.md §8 + feature-request-audit skill + EU AI Act + ISO 27001 + ISO 42001 + SOC 2 + PDPL + Decree 13 + GDPR Art. 30

