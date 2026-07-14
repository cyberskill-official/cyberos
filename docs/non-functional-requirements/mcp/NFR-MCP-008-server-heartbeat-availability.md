---
id: NFR-MCP-008
title: "MCP server heartbeat availability — registered servers MUST respond within 2s"
module: MCP
category: reliability
priority: MUST
verification: T
phase: P0
slo: "p95 < 2s for server heartbeat; 99.5% monthly availability per server"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-MCP-002]
---

## §1 — Statement (BCP-14 normative)

1. Every registered MCP server **MUST** respond to the gateway's heartbeat probe (`ping` or equivalent) within 2s p95.
2. A server failing 3 consecutive heartbeats **MUST** be marked `degraded` in the registry; clients are informed; new traffic is paused.
3. A server failing 10 consecutive heartbeats **MUST** be unregistered; manual re-registration required.
4. Heartbeats run every 30s; the gateway samples response latency for the `mcp_server_heartbeat_latency_seconds` histogram.
5. Monthly per-server availability is reported; servers below 99.5% are reviewed quarterly.

## §2 — Why this constraint

The gateway routes client requests to registered servers; an unhealthy server holding the slot blocks traffic. Active heartbeat probing detects silent failures (process alive but unresponsive). The 3-failure threshold avoids flapping; the 10-failure threshold ensures stuck servers don't linger in the registry. 99.5% availability per server is the registration covenant — below that, the server is functionally unreliable and shouldn't be in the platform.

## §3 — Measurement

- Histogram `mcp_server_heartbeat_latency_seconds{server_id}`.
- Counter `mcp_server_failed_heartbeat_total{server_id}`.
- Monthly availability per server published in OBS dashboard.

## §4 — Verification

- Integration test (T) — kill a registered server's responder; assert degraded → unregistered transition.
- Synthetic test (T) — slow heartbeat (1.5s); assert not flagged degraded (within budget).
- Chaos test (T) — flapping server; assert no flapping in registered/degraded state.

## §5 — Failure handling

- Heartbeat slow → degraded → client traffic paused.
- 10-failure unregistration → server author paged; manual re-register on fix.
- Monthly availability < 99.5% → server reviewed quarterly; possible eviction.

---

*End of NFR-MCP-008.*
