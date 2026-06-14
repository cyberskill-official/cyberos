# CyberOS OBS Stack

`deploy/obs` is the FR-OBS-001 slice-1 LGTM deployment: a CyberOS OTLP ingress
gate, OpenTelemetry Collector, Loki, Prometheus, Tempo, and Grafana.

## Local Run

```bash
./deploy/obs/scripts/rotate_tokens.sh
cp deploy/obs/auth/grafana.jwt.secret.example deploy/obs/auth/grafana.jwt.secret.live
GRAFANA_ADMIN_PASSWORD=cyberos-local-dev \
GRAFANA_USER_JWT=<dev-jwt-signed-with-grafana-secret> \
docker compose -f deploy/obs/docker-compose.yml up -d
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

Grafana datasources are provisioned through `obs-proxy` on port 8088. The proxy
requires `Authorization: Bearer <JWT>` and injects the caller's `tenant_id` into
PromQL, LogQL, and TraceQL before forwarding to the backends. Local/dev stacks
use `auth/grafana.jwt.secret.live` for HS256 tokens; production SHOULD mount an
AUTH JWKS source and run `cyberos-obs grafana-proxy --jwt-jwks-url ...` (or
`--jwt-jwks-file ...` for boot-time secret-store mounts).

## Retention

- Loki logs: 30 days.
- Prometheus raw metrics: 90 days.
- Tempo traces: 7 days.

## Sizing

Slice 1 reserves 6.5 vCPU, 11.5 GB RAM, and 100 GB disk across the stack.
