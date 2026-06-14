# cyberos-obs-router

FR-OBS-007 service that routes Alertmanager webhooks through CUO triage to CHAT
and PagerDuty.

## Local checks

```bash
cd services
cargo test -p cyberos-obs-router --tests
```

## Runtime

`cyberos-obs-router` listens on `0.0.0.0:7777` by default:

- `POST /alert` accepts Alertmanager webhooks with `X-CyberOS-Webhook-Secret`.
- `POST /ack/<alert_id>` records CHAT acks and resolves PagerDuty for sev-1 dual routes.
- `POST /escalate/<alert_id>` triggers PagerDuty after an initial CHAT route.
- `GET /ready` supports container health checks.
- `GET /metrics` exposes FR-OBS-007 metric counters.

Deploy wiring lives in `deploy/obs/docker-compose.yml` and
`deploy/obs/alertmanager-config.yaml`.
