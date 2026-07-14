# OBS module — task index

_Generated 2026-05-17 — 9 FRs, 82 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-OBS-001](TASK-OBS-001-otel-collector/spec.md) | MUST | 1 | 10 | OTel Collector + LGTM stack (Loki + Prometheus + Tempo + Grafana) with mTLS ingress + per-service to |
| [TASK-OBS-002](TASK-OBS-002-tenant-aware-grafana/spec.md) | MUST | 1 | 12 | Tenant-aware Grafana proxy (Rust) — AST-injects tenant_id into PromQL/LogQL/TraceQL with anti-bypass |
| [TASK-OBS-003](TASK-OBS-003-red-metrics/spec.md) | MUST | 1 | 8 | Per-service RED metrics (rate/errors/duration) via cyberos-obs-sdk shared crate with macro + CI lint |
| [TASK-OBS-004](TASK-OBS-004-langsmith-ai-traces/spec.md) | MUST | 2 | 6 | LangSmith integration for AI traces — self-hosted + per-tenant opt-in + redacted-prompts-only + W3C  |
| [TASK-OBS-005](TASK-OBS-005-tracecontext-correlation/spec.md) | MUST | 2 | 8 | W3C TraceContext correlation across logs/metrics/traces/AI-traces — propagate, embed, exemplar, end- |
| [TASK-OBS-006](TASK-OBS-006-tail-sampling/spec.md) | SHOULD | 2 | 6 | Tail-based sampling at OTel collector — 100% errors/5xx/slow/flagged + 10% normal + decision_wait +  |
| [TASK-OBS-007](TASK-OBS-007-alertmanager-cuo-runbook-routing/spec.md) | MUST | 3 | 10 | obs-router: Alertmanager → CUO obs.triage-alert@1 skill → CHAT (≥0.70 conf) OR PagerDuty + sev-1 alw |
| [TASK-OBS-008](TASK-OBS-008-compliance-view-scoping/spec.md) | MUST | 3 | 14 | obs-compliance-view: pre-built read-only views (EU AI Act / PDPL / SOC 2 / ISO 27001) over memory aud |
| [TASK-OBS-009](TASK-OBS-009-chain-of-custody-manifest/spec.md) | MUST | 3 | 8 | Chain-of-custody manifest with Ed25519 signature on every compliance export — PDF cover + JSON sidec |

## Cross-module dependencies

**This module depends on:**

- **AI**: TASK-OBS-004→TASK-AI-022
- **AUTH**: TASK-OBS-002→TASK-AUTH-004

**This module is depended on by:**

- **AI**: TASK-AI-022→TASK-OBS-001
- **KB**: TASK-KB-008→TASK-OBS-007

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._