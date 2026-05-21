# Changelog — OBS

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

