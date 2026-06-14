# cyberos-obs-collector

**P0 module · observability spine.**
Implements [`docs/feature-requests/obs/FR-OBS-001..009`](../../docs/feature-requests/obs/) — CyberOS OTLP ingress + OTel collector + Loki + Prometheus + Tempo + Grafana stack, with tenant-isolation extension hooks, PII scrubbing in-flight, and memory-anchored compliance views.

## Status (2026-06-14 wave)

| FR | Title | Status |
|---|---|---|
| **FR-OBS-001** | OTel collector + LGTM backends + PII scrub + bearer-token auth | **done** |
| **FR-OBS-002** | Grafana tenant-aware query proxy (Rust) | **done** (`cyberos-obs grafana-proxy`; AUTH JWKS/RS256 verification plus HS256 local dev, PromQL AST injection via `promql-parser`, LogQL/TraceQL subset injection, reserved tenant-label rejection, root-admin audited bypass, datasource Compose wiring) |
| FR-OBS-003 | Per-service RED metrics via cyberos-obs-sdk | pending |
| FR-OBS-004 | LangSmith integration for AI traces | pending |
| FR-OBS-005 | W3C TraceContext correlation | pending |
| FR-OBS-006 | Tail-based sampling | pending |
| FR-OBS-007 | Alertmanager → CUO obs.triage-alert routing | pending |
| FR-OBS-008 | Compliance view scoping (EU AI Act / PDPL / SOC 2 / ISO 27001) | pending |
| FR-OBS-009 | Chain-of-custody manifest on compliance exports | pending |

## Layout

```
src/
├── lib.rs                       # public re-exports
├── config.rs                    # otelcol YAML schema validation (CI gate)
├── auth.rs                      # bearer-token file parser
├── grafana_proxy.rs             # tenant-aware Grafana query proxy
├── ingress.rs                   # OTLP HTTP/gRPC per-service token gate
├── metrics.rs                   # self-metric name + label constants
└── bin/
    └── cyberos_obs.rs           # `cyberos-obs validate-*` + `ingress` + `grafana-proxy`
config/
├── otel-collector-config.yaml   # canonical FR-OBS-001 §3 config (validated by CI)
└── auth/
    └── tokens.example           # bearer-token file template (real tokens.live mounted separately)
```

## Local development

```bash
# Build + test:
cargo build -p cyberos-obs-collector
cargo test -p cyberos-obs-collector

# Validate the slice-1 config:
cargo run -p cyberos-obs-collector --bin cyberos-obs -- \
    validate-config services/obs-collector/config/otel-collector-config.yaml

# Validate the bearer-token file template:
cargo run -p cyberos-obs-collector --bin cyberos-obs -- \
    validate-tokens services/obs-collector/config/auth/tokens.example

# Run the Grafana tenant proxy locally:
cargo run -p cyberos-obs-collector --bin cyberos-obs -- \
    grafana-proxy \
    --jwt-hs256-secret-file ../deploy/obs/auth/grafana.jwt.secret.live
```

## Deployment model

Slice 1 baseline (FR-OBS-001 §1 #13 + FR-OBS-002 proxy): 7.5 vCPU · 12.5 GB RAM · 100 GB disk for the full LGTM stack. Per-component sizing:

| Component | vCPU | RAM | Notes |
|---|---:|---:|---|
| otelcol-contrib | 1 | 1 GB | Single instance; HA defers to slice 5 |
| Loki | 2 | 4 GB | 30-day retention floor |
| Prometheus | 2 | 4 GB | 90-day retention floor |
| Tempo | 1 | 2 GB | 7-day retention floor (sampled 30d via FR-OBS-006) |
| obs-proxy | 0.5 | 0.5 GB | Tenant-aware Grafana query proxy |
| Grafana | 0.5 | 0.5 GB | Datasources provisioned at this FR; dashboards via FR-OBS-002 |

The Docker Compose deployment lives at `deploy/obs/`. Public OTLP ports are served by `cyberos-obs ingress`; otelcol remains behind that gate with a single internal bearer token. Grafana datasources point at `obs-proxy`, so Loki/Prometheus/Tempo queries require a CyberOS JWT and tenant injection before reaching the backends.

## §14 protocol emission

This module participates in the `AGENTS.md §14.1` protocol. The slice-1 ship is recorded in `docs/feature-requests/BACKLOG.md §0.5` production-status table with the building state.
