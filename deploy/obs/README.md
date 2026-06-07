# CyberOS OBS Stack

`deploy/obs` is the FR-OBS-001 slice-1 LGTM deployment: a CyberOS OTLP ingress
gate, OpenTelemetry Collector, Loki, Prometheus, Tempo, and Grafana.

## Local Run

```bash
./deploy/obs/scripts/rotate_tokens.sh
GRAFANA_ADMIN_PASSWORD=cyberos-local-dev docker compose -f deploy/obs/docker-compose.yml up -d
./deploy/obs/scripts/healthcheck.sh
./deploy/obs/tests/auth_required_test.sh
./deploy/obs/tests/per_service_token_binding_test.sh
./deploy/obs/tests/smoke_test.sh
```

Real deployments mount `auth/tokens.live` and `auth/collector.token.live` from
the operator secret store. `auth/tokens.live` is service-first for CyberOS
validation tooling and the ingress gate enforces that each token can emit only
for its owning `service.name`. `auth/collector.token.live` is a single internal
token used only between the ingress gate and otelcol; the collector ports are not
published directly.

## Retention

- Loki logs: 30 days.
- Prometheus raw metrics: 90 days.
- Tempo traces: 7 days.

## Sizing

Slice 1 reserves 6.5 vCPU, 11.5 GB RAM, and 100 GB disk across the stack.
