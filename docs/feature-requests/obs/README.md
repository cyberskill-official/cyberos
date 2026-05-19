# OBS module — feature request index

_Generated 2026-05-17 — 9 FRs, 82 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-OBS-001](FR-OBS-001-otel-collector.md) | MUST | 1 | 10 | OTel Collector + LGTM stack (Loki + Prometheus + Tempo + Grafana) with mTLS ingress + per-service to |
| [FR-OBS-002](FR-OBS-002-tenant-aware-grafana.md) | MUST | 1 | 12 | Tenant-aware Grafana proxy (Rust) — AST-injects tenant_id into PromQL/LogQL/TraceQL with anti-bypass |
| [FR-OBS-003](FR-OBS-003-red-metrics.md) | MUST | 1 | 8 | Per-service RED metrics (rate/errors/duration) via cyberos-obs-sdk shared crate with macro + CI lint |
| [FR-OBS-004](FR-OBS-004-langsmith-ai-traces.md) | MUST | 2 | 6 | LangSmith integration for AI traces — self-hosted + per-tenant opt-in + redacted-prompts-only + W3C  |
| [FR-OBS-005](FR-OBS-005-tracecontext-correlation.md) | MUST | 2 | 8 | W3C TraceContext correlation across logs/metrics/traces/AI-traces — propagate, embed, exemplar, end- |
| [FR-OBS-006](FR-OBS-006-tail-sampling.md) | SHOULD | 2 | 6 | Tail-based sampling at OTel collector — 100% errors/5xx/slow/flagged + 10% normal + decision_wait +  |
| [FR-OBS-007](FR-OBS-007-alertmanager-cuo-runbook-routing.md) | MUST | 3 | 10 | obs-router: Alertmanager → CUO obs.triage-alert@1 skill → CHAT (≥0.70 conf) OR PagerDuty + sev-1 alw |
| [FR-OBS-008](FR-OBS-008-compliance-view-scoping.md) | MUST | 3 | 14 | obs-compliance-view: pre-built read-only views (EU AI Act / PDPL / SOC 2 / ISO 27001) over memory aud |
| [FR-OBS-009](FR-OBS-009-chain-of-custody-manifest.md) | MUST | 3 | 8 | Chain-of-custody manifest with Ed25519 signature on every compliance export — PDF cover + JSON sidec |

## Cross-module dependencies

**This module depends on:**

- **AI**: FR-OBS-004→FR-AI-022
- **AUTH**: FR-OBS-002→FR-AUTH-004

**This module is depended on by:**

- **AI**: FR-AI-022→FR-OBS-001
- **KB**: FR-KB-008→FR-OBS-007

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._