---
id: TASK-MCP-002
title: "MCP per-module server registration + heartbeat lifecycle — 3-miss → unhealthy with automatic skill_unavailable propagation"
module: MCP
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: 2026-06-24
memory_chain_hash: pending
related_tasks: [TASK-MCP-001, TASK-MCP-003, TASK-OBS-007, TASK-MEMORY-111]
depends_on: [TASK-MCP-001]
blocks: []

source_pages:
  - website/docs/modules/mcp.html#heartbeat-lifecycle

source_decisions:
  - DEC-2350 2026-05-17 — Each module's MCP server registers at startup + sends heartbeat every 10s; 3 consecutive misses (30s) → unhealthy; skills marked skill_unavailable
  - DEC-2351 2026-05-17 — Closed enum `server_health_status` = {healthy, degraded, unhealthy, deregistered}; cardinality 4
  - DEC-2352 2026-05-17 — Heartbeat carries version + supported_protocols + capability_advertisement
  - DEC-2353 2026-05-17 — Recovery: unhealthy → healthy on next successful heartbeat (no manual intervention)
  - DEC-2354 2026-05-17 — memory audit kinds: mcp.server_registered, mcp.server_heartbeat_missed, mcp.server_unhealthy, mcp.server_recovered, mcp.server_deregistered

build_envelope:
  language: rust 1.81
  service: cyberos/services/mcp/
  new_files:
    - services/mcp/migrations/0002_server_heartbeats.sql
    - services/mcp/src/heartbeat/mod.rs
    - services/mcp/src/heartbeat/registrar.rs
    - services/mcp/src/heartbeat/health_monitor.rs
    - services/mcp/src/handlers/heartbeat_routes.rs
    - services/mcp/src/audit/heartbeat_events.rs
    - services/mcp/tests/server_health_enum_cardinality_test.rs
    - services/mcp/tests/heartbeat_3_miss_unhealthy_test.rs
    - services/mcp/tests/recovery_on_heartbeat_test.rs
    - services/mcp/tests/skill_unavailable_propagation_test.rs
    - services/mcp/tests/heartbeat_audit_emission_test.rs

  modified_files:
    - services/mcp/src/lib.rs

  allowed_tools:
    - file_read: services/mcp/**
    - file_write: services/mcp/{src,tests,migrations}/**
    - bash: cd services/mcp && cargo test heartbeat

  disallowed_tools:
    - skip heartbeat (per DEC-2350)
    - manual unhealthy (per DEC-2353 — auto-recovery only)

effort_hours: 6
subtasks:
  - "0.3h: 0002_server_heartbeats.sql"
  - "0.3h: heartbeat/mod.rs"
  - "0.5h: registrar.rs"
  - "0.6h: health_monitor.rs"
  - "0.4h: handlers/heartbeat_routes.rs"
  - "0.3h: audit/heartbeat_events.rs"
  - "2.5h: tests — 5 test files"
  - "1.1h: docs + skill_unavailable cascade"

risk_if_skipped: "Without heartbeat, dead MCP servers serve stale errors. Without DEC-2350 3-miss threshold, transient blips trigger false unhealthy. Without DEC-2353 auto-recovery, sysadmin overhead."
---

## §1 — Description (BCP-14 normative)

The MCP service **MUST** ship heartbeat lifecycle at `services/mcp/src/heartbeat/` with register + 10s heartbeat + 3-miss-unhealthy + skill cascade, 5 memory audit kinds.

1. **MUST** validate `server_health_status` against closed enum per DEC-2351.

2. **MUST** register at `registrar.rs::register(server_info)` per DEC-2350 — captures version + protocols + capabilities.

3. **MUST** monitor at `health_monitor.rs::monitor()` per DEC-2350 + DEC-2353:
   - Run every 5s; check each server's last_heartbeat_at
   - 0-1 misses (10-20s lag) → healthy
   - 2 misses (20-30s) → degraded
   - 3+ misses (≥30s) → unhealthy + cascade skill_unavailable
   - Heartbeat received → recover to healthy

4. **MUST** define table at migration `0002`:
   ```sql
   CREATE TABLE mcp_servers (
     server_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     module_name TEXT NOT NULL,
     version TEXT NOT NULL,
     supported_protocols TEXT[] NOT NULL,
     capability_advertisement JSONB,
     status TEXT NOT NULL DEFAULT 'healthy'
       CHECK (status IN ('healthy','degraded','unhealthy','deregistered')),
     last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     registered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     deregistered_at TIMESTAMPTZ,
     trace_id CHAR(32),
     UNIQUE (tenant_id, module_name)
   );
   ALTER TABLE mcp_servers ENABLE ROW LEVEL SECURITY;
   CREATE POLICY servers_rls ON mcp_servers
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT UPDATE (version, supported_protocols, capability_advertisement, status, last_heartbeat_at, deregistered_at) ON mcp_servers TO cyberos_app;
   ```

5. **MUST** propagate skill_unavailable per DEC-2350 — when server unhealthy, mark its skills as unavailable (read TASK-MCP-001 skill registry).

6. **MUST** expose endpoints:
   ```text
   POST /v1/mcp/servers/register      (server self-registers)
   POST /v1/mcp/servers/heartbeat     (server self-heartbeat)
   POST /v1/mcp/servers/deregister    (graceful shutdown)
   GET  /v1/mcp/servers               (status list)
   ```

7. **MUST** emit 5 memory audit kinds per DEC-2354. PII per TASK-MEMORY-111: server module_name + version (public) ok.

8. **MUST** thread trace_id from registration → heartbeat → audit.

9. **MUST NOT** require manual unhealthy → healthy transition per DEC-2353.

10. **MUST NOT** mark unhealthy on single miss per DEC-2350 (3-miss minimum).

---

## §2 — Why this design

**Why 10s heartbeat + 3-miss (DEC-2350)?** Balances detection latency (30s max) vs false-positives from transient blips.

**Why 4 statuses (DEC-2351)?** Captures degraded (2 missed = warn) before unhealthy.

**Why auto-recovery (DEC-2353)?** Self-healing; reduces ops overhead.

---

## §3 — API contract

Sample server status:
```json
{
  "server_id": "uuid",
  "module_name": "calendar",
  "version": "1.3.2",
  "status": "healthy",
  "last_heartbeat_at": "2026-05-17T10:00:05Z"
}
```

---

## §4 — Acceptance criteria
1. **server_health_status enum cardinality 4**. 2. **10s heartbeat interval**. 3. **3-miss → unhealthy**. 4. **Recovery on heartbeat**. 5. **skill_unavailable cascade**. 6. **5 memory audit kinds emitted**. 7. **PII: module_name + version ok**. 8. **RLS denies cross-tenant**. 9. **Trace_id preserved**. 10. **UNIQUE(tenant, module_name)**. 11. **Capability advertisement stored**. 12. **Supported protocols array**. 13. **Deregistered status (graceful shutdown)**. 14. **Append-only via REVOKE except status cols**. 15. **Concurrent heartbeats handled**. 16. **2-miss → degraded**. 17. **Monitor cron 5s interval**. 18. **Health check perf < 50ms**. 19. **TASK-OBS-007 integration on unhealthy**. 20. **Cross-tenant isolation**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn three_miss_unhealthy() {
    let ctx = TestContext::with_registered_server().await;
    ctx.advance_time(Duration::seconds(35)).await;
    ctx.run_monitor().await;
    let s = ctx.fetch_server(ctx.server_id).await;
    assert_eq!(s.status, "unhealthy");
}

#[tokio::test]
async fn recovery_on_heartbeat() {
    let ctx = TestContext::with_unhealthy_server().await;
    ctx.send_heartbeat(ctx.server_id).await;
    let s = ctx.fetch_server(ctx.server_id).await;
    assert_eq!(s.status, "healthy");
}

#[tokio::test]
async fn skill_unavailable_propagation() {
    let ctx = TestContext::with_registered_server_and_skills().await;
    ctx.advance_time(Duration::seconds(35)).await;
    ctx.run_monitor().await;
    let skills = ctx.fetch_skills_for_server(ctx.server_id).await;
    assert!(skills.iter().all(|s| s.available == false));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-MCP-001.
**Downstream:** TASK-MCP-003 (naming validator).
**Cross-module:** TASK-OBS-007 (alert on unhealthy), TASK-MEMORY-111 (audit).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Network partition | heartbeat missed | unhealthy + audit | recover on reconnect |
| Server crash | no heartbeat | unhealthy | restart |
| Clock skew | use server time | tolerance | inherent |
| Cross-tenant register | RLS | reject | inherent |
| Module rename | UNIQUE collision | inherent | careful migration |
| Burst restart | rate-limit register | inherent | inherent |
| Monitor cron lag | sev-2 | inherent | inherent |
| Concurrent heartbeat | UPDATE | last-writer-wins | inherent |
| Deregister mid-heartbeat | flag check | OK | inherent |
| Skill registry drift | reconcile cron | inherent | inherent |

## §11 — Implementation notes
- §11.1 Monitor cron via async tokio interval; 5s period.
- §11.2 Skill cascade: `UPDATE skill_registry SET available=false WHERE server_id IN (unhealthy servers)`.
- §11.3 memory audit body: server_id, module_name, status; version + protocols ok.
- §11.4 TASK-OBS-007 receives event on transition to unhealthy.
- §11.5 Deregister gracefully sets status=deregistered + cascade.

---

*End of TASK-MCP-002 spec.*
