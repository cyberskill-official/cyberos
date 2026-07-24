---
id: TASK-OBS-001
title: "OTel collector scaffold + LGTM compose (validate-config, PII scrub)"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: obs
priority: p0
status: done
entered_via: rework
routed_back_count: 1
verify: T
phase: P0
milestone: P0 · slice 2 (after AI Gateway slice 1)
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: [TASK-OBS-002, TASK-OBS-003, TASK-OBS-004, TASK-OBS-005, TASK-OBS-006, TASK-OBS-007, TASK-OBS-008, TASK-OBS-009, TASK-AI-022]
depends_on: []
blocks: [TASK-OBS-002, TASK-OBS-003, TASK-OBS-004, TASK-OBS-005, TASK-OBS-006, TASK-AI-022]

source_pages:
  - website/docs/modules/obs.html#collector
  - website/docs/modules/obs.html#lgtm

source_decisions:
  - DEC-140 (LGTM self-hosted; data-residency for PDPL/GDPR; no SaaS Datadog)
  - DEC-141 (per-service bearer tokens at ingress; rotation 90d via TASK-AUTH-006-style sweeper)
  - DEC-142 (retention slice-1: 30d logs / 90d metrics / 7d traces; P2 extends with S3 backend)
  - DEC-143 (PII-scrub at collector pipeline as defence-in-depth — caller-side typed attrs is primary per TASK-AI-022)

language: rust 1.81 + yaml
service: cyberos/services/obs-collector/
new_files:
  - services/obs-collector/Cargo.toml
  - services/obs-collector/README.md
  - services/obs-collector/docs/index.md
  - services/obs-collector/src/lib.rs
  - services/obs-collector/src/config.rs
  - services/obs-collector/src/auth.rs
  - services/obs-collector/src/metrics.rs
  - services/obs-collector/src/bin/cyberos_obs.rs
  - services/obs-collector/config/otel-collector-config.yaml
  - services/obs-collector/config/auth/tokens.example
  - deploy/obs/docker-compose.yml
  - deploy/obs/README.md
  - deploy/obs/prometheus/prometheus.yml
  - deploy/obs/tempo/tempo.yaml
  - deploy/obs/grafana/provisioning/datasources/datasources.yaml

modified_files:
  - services/Cargo.toml

allowed_tools:
  - file_read: services/obs-collector/**, deploy/obs/**
  - file_write: services/obs-collector/{src,config,docs}/**
  - file_write: deploy/obs/**
  - bash: cd services && cargo test -p cyberos-obs-collector

disallowed_tools:
  - route OTel data to non-CyberOS endpoints (per DEC-140 — self-hosted only)
  - skip auth on collector ingress (per DEC-141)
  - omit PII-scrub processor from collector pipeline (per DEC-143)
  - hardcode bearer tokens in committed YAML (per DEC-141)
  - claim phantom deploy/obs flat otel-collector-config.yaml or loki-config.yaml paths

effort_hours: 10
subtasks:
  - "1.0h: config.rs YAML contract validator + inline tests"
  - "0.5h: auth.rs bearer-token file parser + inline tests"
  - "0.5h: metrics.rs self-metric name constants"
  - "0.5h: cyberos_obs validate-config + validate-tokens CLI"
  - "1.0h: canonical otel-collector-config.yaml under services/obs-collector/config/"
  - "1.5h: deploy/obs LGTM compose (prometheus, loki, tempo, grafana, obs-proxy)"
  - "0.5h: batch/9b-obs re-spec + audit"

risk_if_skipped: "TASK-AI-022 has no destination for traces. TASK-OBS-002 (tenant-aware Grafana proxy) has no data backends to query. Without bearer auth + PII-scrub contract validation, misconfigured collectors accept anonymous or leaky telemetry."
---

# TASK-OBS-001: OTel collector scaffold + LGTM compose

## Summary

Slice-1 ships the `cyberos-obs-collector` crate (`services/obs-collector/`): canonical `otel-collector-config.yaml`, bearer-token file shape, YAML contract validation in `config.rs`, token parsing in `auth.rs`, self-metric name constants in `metrics.rs`, and the `cyberos-obs` CLI (`validate-config`, `validate-tokens`). `deploy/obs/` provides the LGTM backends plus the TASK-OBS-002 `obs-proxy` in `docker-compose.yml` — **not** a live `otelcol-contrib` container in compose. Upstream otelcol process supervision, Helm charts, mTLS ingress, and operator rotation/healthcheck shell suites are explicitly deferred.

## Problem

The original engineering-spec claimed flat `deploy/obs/otel-collector-config.yaml`, `loki-config.yaml`, `prometheus-config.yaml`, `rotate_tokens.sh`, `healthcheck.sh`, and three bash smoke suites that were never built. It used `## §N` body grammar (FM-004) and described a five-service compose with a running collector. The as-built surface is the obs-collector validation crate + nested deploy layout (`prometheus/`, `tempo/`, `grafana/provisioning/`) wired to obs-proxy — honest re-spec required before re-entry.

## Proposed Solution

Adopt the as-built layout:

- `services/obs-collector/src/config.rs` — `validate()` enforces OTLP receiver, Loki + prometheusremotewrite + otlp/tempo exporters, `attributes/pii_scrub` on logs and traces pipelines, `bearertokenauth` + `file_storage` extensions
- `services/obs-collector/src/auth.rs` — `TokenFile::parse` for the canonical `service_name token` file format
- `services/obs-collector/config/otel-collector-config.yaml` — slice-1 canonical collector YAML (validated by CI/tests)
- `services/obs-collector/src/bin/cyberos_obs.rs` — pre-flight `validate-config` / `validate-tokens` subcommands
- `deploy/obs/docker-compose.yml` — Prometheus, Loki, Tempo, Grafana (datasources → obs-proxy), obs-proxy; **no collector service**
- `deploy/obs/prometheus/prometheus.yml`, `deploy/obs/tempo/tempo.yaml`, `deploy/obs/grafana/provisioning/datasources/datasources.yaml`

## Alternatives Considered

- **Resume the old engineering-spec as-is.** Rejected: FM-004 blocks re-entry; paths and compose shape lie.
- **Run otelcol-contrib inside compose in this slice.** Rejected: slice-1 goal is contract validation + LGTM+proxy stack; process supervision lands next slice.
- **Flat deploy/obs/*.yaml configs at repo root.** Rejected: as-built nests configs under `prometheus/`, `tempo/`, `grafana/provisioning/`, and keeps canonical collector YAML in the crate.

## Success Metrics

- Primary: invalid collector YAML (missing PII scrub) is rejected at `validate()`; canonical config and token file parse cleanly; AWH held-out `acceptance-obs-pii-scrub` stays green.
- Guardrail: `deploy/obs/docker-compose.yml` starts LGTM + obs-proxy without claiming a collector container.

## Scope

In scope (as-built):

- `services/obs-collector/**` validation crate + canonical config + `tokens.example`
- `deploy/obs/docker-compose.yml` LGTM backends + obs-proxy + Grafana provisioning
- `deploy/obs/prometheus/prometheus.yml`, `deploy/obs/tempo/tempo.yaml`, `deploy/obs/grafana/provisioning/datasources/datasources.yaml`
- Inline unit tests in `config.rs` and `auth.rs`
- AWH tasks `obs-collector-rust` and `acceptance-obs-pii-scrub`

### Out of scope / Non-Goals

- Phantom flat `deploy/obs/otel-collector-config.yaml`, `deploy/obs/loki-config.yaml`, or root-level prometheus/tempo YAML paths from the old spec
- `deploy/obs/scripts/rotate_tokens.sh`, `deploy/obs/scripts/healthcheck.sh`, and claimed `deploy/obs/tests/smoke_test.sh` / `auth_required_test.sh` / `buffer_survives_restart_test.sh` shell suites
- Live `otelcol-contrib` process supervision inside compose, Helm charts, and mTLS ingress (next slice)
- Running the collector as a compose service — compose is LGTM + obs-proxy only

## Dependencies

`depends_on: []`. Soft: TASK-OBS-002 obs-proxy already consumes the LGTM backends in `deploy/obs/`; TASK-AI-022 typed attribute keys (caller-side PII prevention primary over DEC-143 collector scrub).

## 1. Description (normative)

- 1.1 `config::validate` MUST reject collector YAML missing OTLP receiver, Loki/prometheusremotewrite/otlp-tempo exporters, `bearertokenauth`, `file_storage`, or `pii_scrub` on logs and traces pipelines.
- 1.2 `auth::TokenFile::parse` MUST accept the canonical `service_name <token>` lines (comments and blank lines allowed) and reject malformed rows.
- 1.3 `cyberos-obs` MUST expose `validate-config` and `validate-tokens` subcommands that call `config::validate` and `auth::TokenFile::load` respectively.
- 1.4 The canonical collector YAML MUST live at `services/obs-collector/config/otel-collector-config.yaml`; committed token shape at `services/obs-collector/config/auth/tokens.example`.
- 1.5 `deploy/obs/docker-compose.yml` MUST provision Prometheus, Loki, Tempo, Grafana, and obs-proxy; it MUST NOT include a collector container in slice-1.
- 1.6 This adopt MUST NOT claim phantom deploy scripts, smoke shell suites, flat deploy config paths, or shipped otelcol supervision/Helm/mTLS.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - canonical config passes validate - test: `services/obs-collector/src/config.rs::validate_accepts_canonical_config`
- [ ] AC 2 (traces_to: #1.1) - missing pii_scrub rejected - test: `services/obs-collector/src/config.rs::validate_rejects_missing_pii_scrub`
- [ ] AC 3 (traces_to: #1.2) - token file parses canonical rows - test: `services/obs-collector/src/auth.rs::parse_canonical`
- [ ] AC 4 (traces_to: #1.2) - extra columns rejected - test: `services/obs-collector/src/auth.rs::parse_rejects_extra_columns`
- [ ] AC 5 (traces_to: #1.3,#1.4) - AWH held-out PII-scrub invariant - verify: `cd services && cargo test -p cyberos-obs-collector validate_rejects_missing_pii_scrub`
- [ ] AC 6 (traces_to: #1.5) - compose lists LGTM + obs-proxy without collector service - verify: `deploy/obs/docker-compose.yml`
- [ ] AC 7 (traces_to: #1.6) - Out of scope lists phantom flat configs and shell suites; new_files cite real paths only - verify: this spec Scope / new_files
- [ ] AC 8 (traces_to: #1.3) - cyberos-obs binary exposes validate subcommands - verify: `services/obs-collector/src/bin/cyberos_obs.rs`

## Verification

```bash
cd services && cargo test -p cyberos-obs-collector
cd services && cargo test -p cyberos-obs-collector validate_rejects_missing_pii_scrub
```

| Path | Covers |
|------|--------|
| `src/config.rs` inline tests | YAML contract (canonical + missing pii_scrub) |
| `src/auth.rs` inline tests | Bearer-token file parse + reject malformed lines |
| `src/bin/cyberos_obs.rs` | validate-config / validate-tokens CLI surface |
| `config/otel-collector-config.yaml` | Canonical slice-1 collector pipeline |
| `deploy/obs/docker-compose.yml` | LGTM + obs-proxy (no collector container) |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9b-obs`.
- **Scope:** Re-spec/adopt against as-built obs-collector crate + deploy/obs LGTM compose; deferred otelcol supervision, Helm, mTLS, and shell smoke suites ledgered Out of scope.
- **Human review:** Required at the two HITL gates (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9b-obs adopt — TASK-OBS-001 re-spec against as-built obs-collector + deploy/obs LGTM compose.*
