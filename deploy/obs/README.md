# CyberOS observability stack (tenant-aware)

This stack runs Grafana and the LGTM backends (Loki, Prometheus, Tempo) with the TASK-OBS-002 proxy in front. The point is tenant isolation: a tenant can only ever read its own telemetry.

```
Grafana  -->  obs-proxy  -->  { Prometheus | Loki | Tempo }
```

Grafana never talks to a backend directly. The provisioned datasources (`grafana/provisioning/datasources/datasources.yaml`) all point at `obs-proxy:8088`. For every query the proxy verifies the caller's JWT, reads `tenant_id` from the claims, and AST-injects a `tenant_id` label filter into the PromQL, LogQL, or TraceQL before forwarding. A query that tries to set `tenant_id` itself is refused with HTTP 400 and a sev-1 audit row. A root-admin token (tenant `00000000-0000-0000-0000-000000000000`) is forwarded unfiltered. This is the security core proven by the crate's cross-tenant property test (2000 cases): the forwarded query carries only the caller's tenant, and a user-supplied tenant is always refused.

## Run it locally

The proxy verifies tenant JWTs against the auth service JWKS in a full deployment. For a quick local stack it falls back to a dev HS256 secret, so you can mint tenant tokens without standing up auth.

```sh
cd deploy/obs

# 1. Mint a tenant token signed with the dev secret (same secret the proxy falls back to).
export OBS_DEV_HS256_SECRET="dev-insecure-secret"
export OBS_TENANT_TOKEN="$(python3 - <<'PY'
import base64, hmac, hashlib, json, time
secret = b"dev-insecure-secret"
def b64(b): return base64.urlsafe_b64encode(b).rstrip(b"=")
header = b64(json.dumps({"alg":"HS256","typ":"JWT"}).encode())
payload = b64(json.dumps({"sub":"grafana","tenant_id":"org:cyberskill","exp":int(time.time())+86400}).encode())
sig = b64(hmac.new(secret, header+b"."+payload, hashlib.sha256).digest())
print((header+b"."+payload+b"."+sig).decode())
PY
)"

# 2. Bring up the stack.
docker compose up --build
```

Open Grafana at http://localhost:3000 and run a query against the CyberOS-Prometheus, CyberOS-Loki, or CyberOS-Tempo datasource. Every query is silently scoped to `org:cyberskill`. Change the `tenant_id` in the token to prove a second tenant sees only its own data.

To point at a different tenant or a real deployment, set `OBS_TENANT_TOKEN` to a JWT signed by the auth service and set `OBS_AUTH_JWKS_URL` on the obs-proxy service to the auth JWKS, e.g. `http://auth:8080/.well-known/jwks.json`. With a JWKS configured the dev HS256 fallback is not used.

## Backend routing

The proxy routes by request path (`detect_backend`): `/api/v1/...` and `/api/labels` go to Prometheus, `/loki/...` to Loki, and `/tempo/...`, `/api/search`, `/api/traces` to Tempo. Those are exactly the paths Grafana's Prometheus, Loki, and Tempo datasources emit, so no per-datasource path rewriting is needed.

## Known limitation

Endpoints that carry no `query` parameter (Prometheus `/api/v1/labels`, Loki `/loki/api/v1/labels`, and the series endpoints) are forwarded as-is today. Scoping label and series results per tenant is a documented follow-up; the query path (the data path) is fully scoped.

## Files

- `docker-compose.yml` - the stack (obs-proxy, grafana, prometheus, loki, tempo).
- `Dockerfile.obs-proxy` - builds the obs-proxy binary from the `services/` workspace.
- `grafana/provisioning/datasources/datasources.yaml` - the three datasources, all via the proxy.
- `prometheus/prometheus.yml`, `tempo/tempo.yaml` - minimal backend configs (Loki uses its image default).
- `auth/` - live token material (rotate before any real deployment; not used by the dev fallback).
