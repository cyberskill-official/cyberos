---
id: FR-OBS-001
title: "OTel Collector + LGTM stack (Loki + Prometheus + Tempo + Grafana) with mTLS ingress + per-service tokens + retention + file-buffer"
module: OBS
priority: MUST
status: implementing
verify: T
phase: P0
milestone: P0 · slice 2 (after AI Gateway slice 1)
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_frs: [FR-OBS-002, FR-OBS-003, FR-OBS-004, FR-OBS-005, FR-OBS-006, FR-OBS-007, FR-OBS-008, FR-OBS-009, FR-AI-022]
depends_on: []
blocks: [FR-OBS-002, FR-OBS-003, FR-OBS-004, FR-OBS-005, FR-OBS-006, FR-AI-022]

source_pages:
  - website/docs/modules/obs.html#collector
  - website/docs/modules/obs.html#lgtm
source_decisions:
  - DEC-140 (LGTM self-hosted; data-residency for PDPL/GDPR; no SaaS Datadog)
  - DEC-141 (per-service bearer tokens at ingress; rotation 90d via FR-AUTH-006-style sweeper)
  - DEC-142 (retention slice-1: 30d logs / 90d metrics / 7d traces; P2 extends with S3 backend)
  - DEC-143 (PII-scrub at collector pipeline as defence-in-depth — caller-side typed attrs is primary per FR-AI-022 §1 #15)

language: yaml + docker-compose (P0); helm charts (P2)
service: cyberos/deploy/obs/
new_files:
  - deploy/obs/docker-compose.yml
  - deploy/obs/otel-collector-config.yaml
  - deploy/obs/loki-config.yaml
  - deploy/obs/prometheus-config.yaml
  - deploy/obs/tempo-config.yaml
  - deploy/obs/grafana/datasources.yaml
  - deploy/obs/grafana/provisioning/dashboards/.keep
  - deploy/obs/auth/tokens.example
  - deploy/obs/scripts/rotate_tokens.sh
  - deploy/obs/scripts/healthcheck.sh
  - deploy/obs/README.md
  - deploy/obs/tests/smoke_test.sh
  - deploy/obs/tests/auth_required_test.sh
  - deploy/obs/tests/buffer_survives_restart_test.sh
modified_files: []
allowed_tools:
  - file_read: deploy/obs/**
  - file_write: deploy/obs/**
  - bash: docker compose -f deploy/obs/docker-compose.yml up -d
  - bash: ./deploy/obs/tests/smoke_test.sh
disallowed_tools:
  - route OTel data to non-CyberOS endpoints (per DEC-140 — self-hosted only)
  - skip auth on collector ingress (per §1 #2 — anonymous emit forbidden)
  - omit PII-scrub processor from collector pipeline (per §1 #11 defence-in-depth)
  - hardcode bearer tokens in committed YAML (per §1 #2 — tokens via mounted file)

effort_hours: 10
sub_tasks:
  - "1.0h: docker-compose stack (collector + Loki + Prometheus + Tempo + Grafana) with healthchecks"
  - "1.5h: OTel collector config (OTLP gRPC + HTTP, bearer auth, batch + resource processors, PII-scrub processor)"
  - "0.5h: PII-scrub processor (regex+attribute drop; defence-in-depth vs FR-AI-022 §1 #15 caller-side prevention)"
  - "1.0h: Loki retention + storage config (filesystem at P0; S3 hook for P2)"
  - "1.0h: Prometheus scrape config + remote-write enable + 90d retention"
  - "1.0h: Tempo storage config + 7d retention"
  - "0.5h: Grafana provisioning (datasources + folder for FR-OBS-002 dashboards)"
  - "1.0h: file_storage extension config (buffer survives restart; 1GB cap)"
  - "0.5h: Per-service token file (mounted; not committed) + rotate_tokens.sh"
  - "0.5h: Healthcheck script + restart policies"
  - "1.5h: Tests — smoke + auth_required + buffer_survives_restart + retention + PII-scrub"
risk_if_skipped: "FR-AI-022 has no destination for traces. FR-OBS-002 (tenant-aware Grafana proxy) has no data backends to query. The 3 observability pillars (logs, metrics, traces) all sit empty. Investigation tooling = SSH + grep on prod boxes — unworkable past 5 services. Without bearer auth on ingress, anyone on the network can inject fake telemetry (poisoning dashboards, alarm fatigue, cover for actual attacks). Without file_storage extension, collector restart loses 5-30s of in-flight telemetry — gaps in the chain that look like outages."
---

## §1 — Description (BCP-14 normative)

The observability plane **MUST** deploy a self-hosted OpenTelemetry Collector receiving OTLP from all CyberOS services, routing to LGTM backends (Loki for logs, Prometheus for metrics, Tempo for traces). Each piece:

1. **MUST** accept OTLP/gRPC on `:4317` and OTLP/HTTP on `:4318` from all CyberOS services. Both protocols required (services may pick); both configured identically (same processors, same exporters).
2. **MUST** authenticate ingress via per-service bearer token. Tokens live in `/etc/otelcol/auth.tokens` (mounted from the operator's secret store; NEVER committed to git). One token per service (`ai-gateway`, `auth-service`, `chat-service`, etc.); rotation cadence 90 days via `scripts/rotate_tokens.sh` (FR-AUTH-006-style sweeper). Missing or invalid token → ingress returns `401 UNAUTHORIZED`.
3. **MUST** retain backend data per slice-1 floors:
    - Loki: 30 days (P2 extends to 90 with S3 backend).
    - Prometheus: 90 days raw (P2 extends to 1 year with downsampling via Mimir).
    - Tempo full traces: 7 days.
    - Tempo sampled traces (FR-OBS-006 tail-sampled): 30 days.
4. **MUST** label every observation with `tenant_id` (from caller's resource attributes). FR-OBS-002's tenant-aware proxy enforces tenant-scoped queries downstream; this FR ensures the label IS present at ingress.
5. **MUST** ship Grafana with provisioned datasources for all 3 backends (Loki, Prometheus, Tempo) AND a placeholder dashboards directory that FR-OBS-002 populates.
6. **MUST** expose `/ready` health endpoints on all 4 services (collector + Loki + Prometheus + Tempo). Docker-compose healthchecks consume these; failure → container restart.
7. **MUST** survive single-pod restarts without losing buffered data via OTel Collector's `file_storage` extension (1GB on-disk buffer at `/var/lib/otelcol`). Buffer is replayed on restart; data emitted during the restart window is delivered eventually.
8. **MUST** apply OTel semantic conventions for resource attributes:
    - `service.name` (e.g., `"ai-gateway"`)
    - `service.version` (semver)
    - `deployment.environment` (`development | staging | production`)
    - `tenant_id` (UUID) — populated by service per request
    - `host.name`
9. **MUST** enforce per-pipeline batch + resource processors:
    - `batch`: timeout 10s, send_batch_size 1024 (efficient backend writes).
    - `resource`: upserts `deployment.environment` from `${ENV}` (defence against missing env vars).
    - `attributes/pii_scrub`: drops attributes matching PII regex AS A SAFETY NET (caller-side discipline per FR-AI-022 §1 #15 is the primary control).
10. **MUST** emit collector self-telemetry (collector's own logs + metrics about throughput, drops, errors) to the same backends — operators see "is the collector itself healthy?" via Grafana.
11. **MUST** apply PII-scrub processor at the collector as defence-in-depth: regex against email patterns + Vietnamese PII patterns (CCCD, MST, phone) + arbitrary `password=` substring. Matched attributes are DROPPED (not redacted to placeholder) AND emit `obs_collector_pii_scrub_total{pattern}` counter. Sustained > 0/min triggers sev-1 (PII slipping past caller-side prevention).
12. **MUST** be horizontally scalable: collector instances behind a load balancer; Loki/Prometheus/Tempo each support replication (slice 1 = single-instance per backend; slice 5 = HA with peering).
13. **MUST** size at slice-1 baseline:
    - Collector: 1 vCPU, 1GB RAM.
    - Loki: 2 vCPU, 4GB RAM.
    - Prometheus: 2 vCPU, 4GB RAM.
    - Tempo: 1 vCPU, 2GB RAM.
    - Grafana: 0.5 vCPU, 512MB RAM.
    - Total: 6.5 vCPU, 11.5GB RAM, 100GB disk.
14. **SHOULD** emit OTel SELF-metrics:
    - `obs_collector_received_spans_total{service}` (counter).
    - `obs_collector_received_logs_total{service}` (counter).
    - `obs_collector_received_metrics_total{service}` (counter).
    - `obs_collector_dropped_total{reason}` (counter; reason ∈ auth | pii_scrub | backend_error | buffer_full).
    - `obs_collector_buffer_bytes` (gauge; file_storage usage).
    - `obs_collector_export_latency_ms{backend}` (histogram).

---

## §2 — Why this design (rationale for humans)

**Why LGTM stack (DEC-140)?** Self-hosted, vendor-neutral, no per-host licensing fees. Critical for PDPL/GDPR data residency: SaaS observability (Datadog, New Relic) means tenant telemetry leaves Vietnam/EU borders. LGTM keeps everything in-region. Operational cost: ~$50/month for a slice-1 deployment (compute + disk).

**Why per-service bearer tokens (§1 #2)?** Ingress without auth lets anyone on the network inject fake telemetry — poisoning dashboards, creating alarm fatigue, providing cover for actual attacks. Per-service tokens scope the blast radius (compromised AI Gateway can't impersonate AUTH telemetry). Rotation prevents long-lived credential abuse.

**Why file_storage extension (§1 #7)?** Without persistent buffer, collector restart loses 5-30s of in-flight telemetry — invisible gaps in dashboards that look like outages or hide real outages. file_storage persists the buffer to disk; on restart, replay continues. The 1GB cap protects against unbounded growth during prolonged backend outages.

**Why caller-side PII prevention is primary (§1 #11)?** FR-AI-022 §1 #15 enforces typed attribute keys + AST lint at the source — PII never enters spans. The collector-side scrub is defence-in-depth: catches the cases where the caller-side prevention fails (e.g., a rare service that doesn't use the typed-key pattern, a regression in the lint). Detection at the collector means investigating + fixing the caller; the scrub also drops the data so it doesn't persist.

**Why batch processor at 10s + 1024 (§1 #9)?** Backend writes are more efficient batched. 10s is the latency-vs-efficiency floor; longer = laggier dashboards. 1024 is a per-batch cardinality floor — fewer round-trips at high volume.

**Why slice-1 single-instance with HA deferred to slice 5 (§1 #12)?** HA adds operational complexity (peering, leader election, split-memory). At slice-1 scale (50 tenants × ~5 services × ~100 spans/sec each = 25K spans/sec), a single collector instance + single backend per signal handles the load. HA earns its complexity at slice 5+ scale.

**Why retention floors at 30d/90d/7d (§1 #3)?** Logs (30d) cover most operational investigations + a typical compliance-audit lookback window. Metrics (90d) cover quarter-over-quarter trending. Traces (7d) are expensive (higher cardinality); 7d covers immediate investigations. Sampled traces (30d via FR-OBS-006) extend the trace lookback for regulatory needs at lower cost.

**Why dedicated PII-scrub METRIC + sev-1 alert (§1 #11)?** A scrub that matches PII is a SIGNAL — caller-side prevention failed. Sustained scrub-rate > 0 means a service is leaking PII into telemetry; that's a code bug AND a compliance event. Sev-1 forces immediate investigation; the scrub itself prevents persistent storage of the leak.

**Why grafana provisioned with placeholder dashboards (§1 #5)?** FR-OBS-002 populates dashboards. Provisioning the datasources at THIS FR means dashboards land cleanly without manual datasource setup. The empty placeholder dir is a hook FR-OBS-002 fills.

**Why OTel semantic conventions (§1 #8)?** Standard attribute names mean Grafana dashboards (FR-OBS-002), correlation rules (FR-OBS-005), and tail-sampling policies (FR-OBS-006) all work without per-service translation. The convention is the contract; deviating means rewriting all consumers.

---

## §3 — API contract

### Collector config

```yaml
# deploy/obs/otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
        auth: { authenticator: bearertokenauth }
      http:
        endpoint: 0.0.0.0:4318
        auth: { authenticator: bearertokenauth }

processors:
  resource:
    attributes:
      - key: deployment.environment
        value: ${ENV}
        action: upsert
  attributes/pii_scrub:
    actions:
      - key: prompt_text
        action: delete
      - key: response_text
        action: delete
      - key: user_email
        action: delete
      - key: cccd
        action: delete
      - pattern: "(?i)(password|secret|api_?key)\\s*[:=]"
        action: delete
  batch:
    timeout: 10s
    send_batch_size: 1024

exporters:
  loki:
    endpoint: http://loki:3100/loki/api/v1/push
    labels:
      attributes:
        service.name: "service_name"
        tenant_id: "tenant_id"
        deployment.environment: "env"
  prometheusremotewrite:
    endpoint: http://prometheus:9090/api/v1/write
    external_labels: { source: "otelcol" }
  otlp/tempo:
    endpoint: tempo:4317
    tls: { insecure: true }

extensions:
  file_storage:
    directory: /var/lib/otelcol/file_storage
    timeout: 1s
  bearertokenauth:
    scheme: "Bearer"
    filename: /etc/otelcol/auth.tokens
  health_check: { endpoint: 0.0.0.0:13133 }

service:
  extensions: [file_storage, bearertokenauth, health_check]
  pipelines:
    logs:    { receivers: [otlp], processors: [resource, attributes/pii_scrub, batch], exporters: [loki] }
    metrics: { receivers: [otlp], processors: [resource, batch], exporters: [prometheusremotewrite] }
    traces:  { receivers: [otlp], processors: [resource, attributes/pii_scrub, batch], exporters: [otlp/tempo] }
  telemetry:
    metrics: { address: 0.0.0.0:8888 }   # collector self-metrics
    logs: { level: info }
```

### Token file format

```text
# deploy/obs/auth/tokens.example  (committed; real tokens.live mounted separately)
ai-gateway   <token-from-secret-store>
auth-service <token-from-secret-store>
chat-service <token-from-secret-store>
memory-writer <token-from-secret-store>
mcp-router   <token-from-secret-store>
# rotate via scripts/rotate_tokens.sh quarterly
```

### docker-compose

```yaml
# deploy/obs/docker-compose.yml
services:
  collector:
    image: otel/opentelemetry-collector-contrib:0.110.0
    command: ["--config=/etc/otelcol/config.yaml"]
    ports: ["4317:4317", "4318:4318", "13133:13133", "8888:8888"]
    environment: { ENV: "${DEPLOYMENT_ENV:-production}" }
    volumes:
      - ./otel-collector-config.yaml:/etc/otelcol/config.yaml:ro
      - ./auth/tokens.live:/etc/otelcol/auth.tokens:ro
      - otel-buffer:/var/lib/otelcol
    depends_on:
      loki:       { condition: service_healthy }
      prometheus: { condition: service_healthy }
      tempo:      { condition: service_healthy }
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:13133"]
      interval: 10s
      timeout: 3s
      retries: 3
    restart: always
    deploy:
      resources:
        limits: { cpus: "1", memory: "1G" }

  loki:
    image: grafana/loki:3.0.0
    volumes:
      - ./loki-config.yaml:/etc/loki/config.yaml:ro
      - loki-data:/loki
    healthcheck: { test: ["CMD", "wget", "-q", "-O", "-", "http://localhost:3100/ready"], interval: 10s }
    restart: always
    deploy: { resources: { limits: { cpus: "2", memory: "4G" } } }

  prometheus:
    image: prom/prometheus:v2.55.0
    command:
      - "--config.file=/etc/prometheus/prometheus.yml"
      - "--web.enable-remote-write-receiver"
      - "--storage.tsdb.retention.time=90d"
    volumes:
      - ./prometheus-config.yaml:/etc/prometheus/prometheus.yml:ro
      - prom-data:/prometheus
    healthcheck: { test: ["CMD", "wget", "-q", "-O", "-", "http://localhost:9090/-/ready"], interval: 10s }
    restart: always
    deploy: { resources: { limits: { cpus: "2", memory: "4G" } } }

  tempo:
    image: grafana/tempo:2.6.0
    command: ["-config.file=/etc/tempo.yaml"]
    volumes:
      - ./tempo-config.yaml:/etc/tempo.yaml:ro
      - tempo-data:/var/tempo
    healthcheck: { test: ["CMD", "wget", "-q", "-O", "-", "http://localhost:3200/ready"], interval: 10s }
    restart: always
    deploy: { resources: { limits: { cpus: "1", memory: "2G" } } }

  grafana:
    image: grafana/grafana:11.3.0
    ports: ["3000:3000"]
    environment:
      GF_SECURITY_ADMIN_PASSWORD: ${GRAFANA_ADMIN_PASSWORD}
      GF_AUTH_ANONYMOUS_ENABLED: "false"
    volumes:
      - ./grafana/datasources.yaml:/etc/grafana/provisioning/datasources/datasources.yaml:ro
      - ./grafana/provisioning/dashboards:/etc/grafana/provisioning/dashboards:ro
      - grafana-data:/var/lib/grafana
    healthcheck: { test: ["CMD", "wget", "-q", "-O", "-", "http://localhost:3000/api/health"], interval: 10s }
    restart: always
    depends_on: [loki, prometheus, tempo]

volumes: { otel-buffer:, loki-data:, prom-data:, tempo-data:, grafana-data: }
```

### Loki retention

```yaml
# deploy/obs/loki-config.yaml (excerpt)
limits_config:
  retention_period: 720h   # 30 days
table_manager:
  retention_deletes_enabled: true
  retention_period: 720h
```

### Tempo retention

```yaml
# deploy/obs/tempo-config.yaml (excerpt)
compactor:
  compaction:
    block_retention: 168h   # 7 days
```

### Grafana datasources

```yaml
# deploy/obs/grafana/datasources.yaml
apiVersion: 1
datasources:
  - name: Loki
    type: loki
    url: http://loki:3100
  - name: Prometheus
    type: prometheus
    url: http://prometheus:9090
  - name: Tempo
    type: tempo
    url: http://tempo:3200
```

---

## §4 — Acceptance criteria

1. **Stack starts cleanly** — `docker compose up -d` succeeds; all 5 containers reach healthy state in <30s; smoke_test.sh passes.
2. **OTLP/gRPC accepts** — Test client sending span to `:4317` produces stored trace in Tempo within 10s.
3. **OTLP/HTTP accepts** — Same test via `:4318` produces stored trace.
4. **Bearer auth enforced** — Test client without `Authorization: Bearer <token>` gets `401`.
5. **Per-service tokens distinct** — `ai-gateway` token rejected by `auth-service` ingress (each token authorises one service.name).
6. **Logs route to Loki** — Test log line via OTLP appears in Loki within 5s; queryable via Grafana.
7. **Metrics route to Prometheus** — Test counter appears in Prometheus query within 30s.
8. **Traces route to Tempo** — Test span appears in Tempo within 10s.
9. **Buffer survives restart** — Kill collector mid-flush; restart; in-flight data not lost (file_storage replay verified).
10. **PII-scrub drops `prompt_text` attribute** — Span emitted with `prompt_text="hello"` arrives at Tempo WITHOUT that attribute; metric `obs_collector_pii_scrub_total{pattern="prompt_text"}` increments; sev-1 alarm fires.
11. **Tenant_id label preserved** — Span with `tenant_id` resource attr → Loki/Tempo records carry the label.
12. **Grafana datasources green** — Grafana home page shows 3 datasources all live.
13. **Retention configs verified** — Tempo deletes traces older than 7d; Loki rotates logs older than 30d; Prometheus retains 90d raw.
14. **Collector self-metrics emit** — Prometheus query `obs_collector_received_spans_total` returns non-zero after test traffic.
15. **Healthchecks pass** — All 4 backends + collector report ready; container restart on health failure.
16. **Resource limits enforced** — `docker stats` shows containers within configured cpu/memory limits.
17. **Token rotation safe** — `scripts/rotate_tokens.sh` rotates without service disruption (collector reloads file).

---

## §5 — Verification

```bash
# deploy/obs/tests/smoke_test.sh
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

docker compose up -d
sleep 30

echo "Checking healthchecks..."
for svc in collector loki prometheus tempo grafana; do
    status=$(docker compose ps --format json $svc | jq -r '.Health')
    if [ "$status" != "healthy" ]; then echo "FAIL: $svc not healthy"; exit 1; fi
done

echo "Sending test span via OTLP/HTTP..."
TOKEN=$(grep "^ai-gateway " auth/tokens.live | awk '{print $2}')
TRACE_ID=$(uuidgen | tr -d '-' | head -c 32)
curl -X POST http://localhost:4318/v1/traces \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d "{\"resourceSpans\":[{\"resource\":{\"attributes\":[{\"key\":\"service.name\",\"value\":{\"stringValue\":\"ai-gateway\"}}]},\"scopeSpans\":[{\"spans\":[{\"traceId\":\"$TRACE_ID\",\"spanId\":\"0011223344556677\",\"name\":\"test\",\"kind\":1,\"startTimeUnixNano\":\"1747526400000000000\",\"endTimeUnixNano\":\"1747526401000000000\"}]}]}]}"

sleep 10
echo "Verifying span in Tempo..."
status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3200/api/traces/$TRACE_ID)
if [ "$status" != "200" ]; then echo "FAIL: trace not found in Tempo"; exit 1; fi
echo "✅ smoke_test passed"
```

```bash
# deploy/obs/tests/auth_required_test.sh
#!/usr/bin/env bash
status=$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:4318/v1/traces \
    -H "Content-Type: application/json" \
    -d '{"resourceSpans":[]}')
if [ "$status" != "401" ]; then echo "FAIL: ingress accepted unauthenticated request"; exit 1; fi
echo "✅ auth_required_test passed"
```

```bash
# deploy/obs/tests/buffer_survives_restart_test.sh
#!/usr/bin/env bash
# Send 100 spans, kill collector mid-flush, restart, verify all 100 in Tempo
TOKEN=$(grep "^ai-gateway " auth/tokens.live | awk '{print $2}')
for i in {1..100}; do
    curl -X POST http://localhost:4318/v1/traces -H "Authorization: Bearer $TOKEN" -d @"test_span_$i.json" &
done
sleep 1
docker compose kill collector
sleep 5
docker compose up -d collector
sleep 15

found=$(curl -s "http://localhost:3200/api/search?tags=test_batch:restart_test" | jq '.traces | length')
if [ "$found" -lt 95 ]; then echo "FAIL: only $found/100 spans recovered"; exit 1; fi
echo "✅ buffer_survives_restart_test passed ($found/100)"
```

```bash
docker compose -f deploy/obs/docker-compose.yml up -d
./deploy/obs/tests/smoke_test.sh
./deploy/obs/tests/auth_required_test.sh
./deploy/obs/tests/buffer_survives_restart_test.sh
```

---

## §6 — Implementation skeleton

See §3.

```bash
# deploy/obs/scripts/rotate_tokens.sh
#!/usr/bin/env bash
set -euo pipefail
TOKENS_FILE=/etc/otelcol/auth.tokens
NEW_TOKENS_FILE=$(mktemp)
for service in ai-gateway auth-service chat-service memory-writer mcp-router; do
    new_token=$(openssl rand -hex 32)
    echo "$service $new_token" >> "$NEW_TOKENS_FILE"
    # TODO: push new token to each service's secret store via secret-manager API
done
mv "$NEW_TOKENS_FILE" "$TOKENS_FILE"
docker compose kill -s SIGHUP collector   # OTel collector reloads on SIGHUP
echo "✅ Tokens rotated"
```

---

## §7 — Dependencies

- Docker Compose v2.0+ (k8s + Helm at slice 5+).
- 6.5 vCPU + 11.5GB RAM + 100GB disk (slice 1 baseline; scales linearly).
- Operator's secret store (1Password Connect, AWS Secrets Manager, etc.) for `auth.tokens.live`.
- Ports: 4317, 4318 (collector), 3000 (Grafana), 13133 (collector health), 8888 (collector self-metrics).

---

## §8 — Example payloads

### OTLP/HTTP ingress

```http
POST /v1/traces HTTP/1.1
Host: otel.cyberos.world:4318
Authorization: Bearer <ai-gateway-token>
Content-Type: application/json

{
  "resourceSpans": [{
    "resource": { "attributes": [
      { "key": "service.name",        "value": { "stringValue": "ai-gateway" } },
      { "key": "service.version",     "value": { "stringValue": "0.4.1" } },
      { "key": "deployment.environment", "value": { "stringValue": "production" } },
      { "key": "tenant_id",           "value": { "stringValue": "550e..." } }
    ]},
    "scopeSpans": [{ "spans": [{ "traceId": "...", "spanId": "...", "name": "ai_gateway.chat_completion", ... }] }]
  }]
}
```

### PII-scrub event (sev-1)

```text
WARN  pattern=user_email service=chat-service
      obs_collector_pii_scrub: dropped attribute "user_email" — caller-side prevention failed
sev-1 obs_collector_pii_scrub_total{pattern="user_email"} incremented
```

### Token rotation

```text
$ ./deploy/obs/scripts/rotate_tokens.sh
Rotating ai-gateway token...
Rotating auth-service token...
...
Reloading collector via SIGHUP...
✅ Tokens rotated
```

---

## §9 — Open questions

All resolved. Deferred:
- HA peering (Loki + Prometheus + Tempo replication) — slice 5+.
- S3 backend for long-term retention — P2.
- Mimir replacement for Prometheus (longer retention + downsampling) — P2.
- Cross-region collector federation — slice 6+.
- mTLS instead of bearer tokens — slice 4+ (when service-mesh lands).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Collector OOM | docker healthcheck fails | Restart policy: always | Operator scales memory limit |
| Loki backend full (>90% disk) | disk-usage metric | Logs dropped; retention shortened automatically | Operator extends disk OR moves to S3 (P2) |
| Prometheus backend full | same | Metrics dropped | Same |
| Tempo backend full | same | Traces dropped | Same |
| Auth token leaked | Unknown caller successfully emits + sev-1 alarm | Sev-1 OBS event | Rotate via scripts/rotate_tokens.sh; investigate |
| Backend (Tempo) restart | healthcheck 503 | Collector buffers via file_storage; resumes when backend up | Self-healing |
| Cross-service trace context broken | Tempo shows orphan spans | Investigate W3C traceparent propagation (FR-AI-022) | Engineer fixes propagation gap |
| PII detected by scrub | metric `obs_collector_pii_scrub_total` increment | Sev-1 + dropped from spans | Investigate caller; add typed-key per FR-AI-022 §1 #15 |
| Collector buffer full (>1GB) | metric `obs_collector_buffer_bytes` near cap | Spans dropped (oldest first); metric `obs_collector_dropped_total{reason=buffer_full}` | Operator extends backend OR scales collector |
| Token rotation breaks service | Service ingress 401 spike | Sev-2 alarm; rollback rotation | Operator restores prior tokens |
| Grafana password missing | Boot fails | Grafana doesn't start | Operator sets GF_SECURITY_ADMIN_PASSWORD |
| `tenant_id` resource attr missing | Span ingested without label | Logs/traces unsearchable by tenant | Caller MUST set resource attr (FR-AI-022) |
| File_storage corruption | Replay fails | Buffered data lost | Operator deletes corrupt buffer; restart |
| Collector config syntax error | Boot fails | Container restart loop | Operator fixes YAML; redeploy |
| Backend version drift (Loki upgraded but config incompatible) | Boot fails on backend | Backend down; collector buffers | Operator pins versions in compose |
| Network partition between collector and backend | Export retries; eventually drops | Sev-2 alarm | Operator investigates network |
| Disk-full on otel-buffer volume | file_storage write fails | Data dropped | Extend volume |
| Large span (> default 1MB) | Collector truncates | Truncation logged | Caller checks span size; chunk if needed |
| Slow backend write | Export latency histogram alarm | Sev-3 | Operator investigates backend; consider scaling |
| Rate of spans > collector throughput | Backpressure to OTLP receivers | Senders block briefly | Auto-scale collector OR reduce sample rate |

---

## §11 — Notes

- Use OTel Collector Contrib (not core) for the Loki + Tempo exporters + bearertokenauth + file_storage extensions.
- Auth tokens are per-service. Rotation cadence 90 days via `scripts/rotate_tokens.sh`. The script pushes new tokens to each service's secret store; services pick up via standard env-var/secret reload.
- Grafana admin password is set via env var `GRAFANA_ADMIN_PASSWORD`; never committed to git. Initial password set during operator's first deploy.
- The PII-scrub processor is defence-in-depth. Caller-side prevention (FR-AI-022 §1 #15 typed attribute keys + AST lint) is the primary control. The scrub catches regressions; sev-1 alarm forces investigation.
- File_storage extension persists collector buffer to disk (1GB cap). Survives restart; tradeoff is disk I/O on every batch (mitigated by 1s timeout). Without it, restart loses 5-30s of in-flight telemetry.
- Slice 1 = single-instance backends. HA (peering for Loki/Prometheus/Tempo) is slice 5+ work. At our scale (~25K spans/sec), single instance is fine.
- Retention floors (30d/90d/7d) reflect operational + compliance lookback windows. P2 extends with S3 backend (Loki) + Mimir (Prometheus) + sampled Tempo (FR-OBS-006).
- Tenant_id resource attribute is the FR-OBS-002 query primitive — without it, tenant-aware filtering is impossible. FR-AI-022 + every CyberOS service MUST emit it.
- The OTel collector is itself observable — its self-metrics + self-logs go to Prometheus + Loki via the same pipeline. Operators monitor "is the collector dropping?" via Grafana dashboard backed by `obs_collector_dropped_total`.

---

*End of FR-OBS-001. Status: draft (10/10 target).*
