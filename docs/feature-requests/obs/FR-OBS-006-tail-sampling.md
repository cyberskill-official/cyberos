---
id: FR-OBS-006
title: "Tail-based sampling at OTel collector — 100% errors/5xx/slow/flagged + 10% normal + decision_wait + flagged-tenants config"
module: OBS
priority: SHOULD
status: done
verify: T
phase: P0
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: 2026-06-15
memory_chain_hash: null
related_frs: [FR-OBS-001, FR-OBS-005, FR-AI-022, FR-AI-021]
depends_on: [FR-OBS-001]
blocks: []

source_pages:
  - website/docs/modules/obs.html#sampling
source_decisions:
  - DEC-165 (tail-based sampling at collector; head-based wastes data on uninteresting cases)
  - DEC-166 (100% errors + 100% slow + 10% normal; calibrated for storage/cost balance)
  - DEC-167 (flagged-tenants reload via config-watch; ops can flag tenants without collector restart)

language: yaml + Rust (collector config + admin script)
service: cyberos/deploy/obs/
new_files:
  - deploy/obs/tail_sampling_config.yaml
  - deploy/obs/scripts/flag_tenant.sh
  - deploy/obs/tests/sampling_test.sh
modified_files:
  - deploy/obs/otel-collector-config.yaml
  - services/ai-gateway/src/cli/flag_tenant.rs                  # FR-AI-021 hook
allowed_tools:
  - file_read: deploy/obs/**
  - file_write: deploy/obs/**, services/ai-gateway/src/cli/**
  - bash: docker compose restart collector
  - bash: ./deploy/obs/tests/sampling_test.sh
disallowed_tools:
  - sample error traces below 100% (per §1 #1)
  - sample 5xx traces below 100% (per §1 #2)
  - reduce decision_wait below 20s (per §1 #8 — slow traces won't complete in time)

effort_hours: 6
sub_tasks:
  - "0.5h: tail_sampling_config.yaml policies (errors, 5xx, slow, flagged, normal)"
  - "0.5h: Modify otel-collector-config.yaml to enable tail_sampling processor in traces pipeline"
  - "1.0h: flag_tenant.sh + flag_tenant.rs hook (FR-AI-021 integration)"
  - "0.5h: Config-watch hot reload (collector picks up flagged_tenants list without restart)"
  - "0.5h: OTel metric obs_sampled_traces_total + reason labels"
  - "0.5h: decision_wait + num_traces sizing for slice-2 load"
  - "1.0h: per-route p99 latency-budget computation (read from FR-OBS-003 metrics)"
  - "1.5h: Tests — sampling rates per policy + flagged tenant + slow trace + error 100%"
risk_if_skipped: "Trace storage cost scales 10x. At 1M traces/day × 10KB/trace = 10GB/day vs 1GB/day with sampling. P0 budget assumes ~$100/month for Tempo storage — without sampling that becomes ~$1K/month. Without 100%-on-errors, the highest-value debugging cases get probabilistically sampled out — investigations show 'the trace exists in logs but not in Tempo.' Without flagged-tenants, ops debugging a specific tenant's complaint can't capture all their traces."
---

## §1 — Description (BCP-14 normative)

The OTel collector **MUST** apply tail-based sampling per the following policies (highest precedence first):

1. **MUST** sample 100% of traces with any span where `status.code = ERROR`.
2. **MUST** sample 100% of traces with any span carrying HTTP status ≥ 500 (`http.status_code` or `http.response.status_code` attribute, depending on OTel semantic-convention version).
3. **MUST** sample 100% of traces with `tenant_id` in the flagged-tenants list. The list is configured via `deploy/obs/flagged_tenants.yaml` (one tenant_id per line); operators add via `cyberos-ai flag-tenant <id> --confirm` (FR-AI-021 subcommand). Hot-reload on file change (no collector restart).
4. **MUST** sample 100% of traces with end-to-end latency above per-route p99 budget. Default budgets per route (loaded from `deploy/obs/route_latency_budgets.yaml`):
    - `ai-gateway:/v1/chat/completions` → 5000ms
    - `ai-gateway:/v1/embeddings` → 500ms
    - `auth-service:/v1/auth/token` → 250ms
    - default for unspecified routes → 2000ms
5. **MUST** sample 10% of normal traces (no error, no 5xx, not flagged tenant, not slow). Probabilistic sampling on trace_id hash.
6. **MUST** evaluate policies in order; first-match wins. A trace matching error AND slow AND normal is sampled exactly once (counted as "error" in metrics).
7. **MUST** emit metric `obs_sampled_traces_total{reason}` per sampling decision (reason ∈ `error | http_5xx | flagged_tenant | slow | normal_sample | dropped`).
8. **MUST** wait `decision_wait: 30s` for trace completion before applying policies. Traces in flight at decision time are buffered; full-trace evaluation (all spans seen) is the policy input.
9. **MUST** size `num_traces: 100000` (in-flight buffer cap). At slice-2 load (~25K spans/sec), this caps memory at ~1GB.
10. **MUST** route the sampling processor only on the TRACES pipeline (not logs, not metrics — those have their own retention controls).
11. **MUST** support hot-reload of `flagged_tenants.yaml` via collector's file-watch extension (no restart needed). Reload latency ≤ 30s.
12. **MUST** emit `obs_sampling_buffer_depth` gauge (current count of in-flight traces); sev-2 alarm when > 90% of `num_traces`.
13. **SHOULD** support per-tenant sampling rate override (e.g., one tenant requests 50% sampling for their own monitoring). Tenant-policy field `obs_sampling_rate` overrides the 10% default.

---

## §2 — Why this design (rationale for humans)

**Why tail-based not head-based (DEC-165)?** Head-based sampling decides at trace start without knowing if the trace will error. Result: 90% of error traces (the high-value cases) get sampled out probabilistically. Tail-based waits until trace completes (all spans seen), then decides — error traces ALWAYS captured.

**Why 100% on errors + 5xx (§1 #1, #2)?** Errors are the high-value debugging cases. Probabilistic sampling on them defeats the purpose of having traces. The cost is small: error rate is typically <1% of traffic, so 100%×1% + 10%×99% ≈ 11% effective rate vs uniform 10%.

**Why 100% on slow traces (§1 #4)?** A slow call is a debugging case ("why was this 5x normal latency?"). Tempo with full span detail is the answer. Probabilistic sampling would lose the slow trace 90% of the time. Per-route p99 budgets adapt to each route's normal latency profile.

**Why flagged-tenants override (§1 #3)?** When a tenant complains "calls are slow today," ops needs ALL their traces for the investigation window — not 10%. Flagging captures the next ~hour at 100%; ops investigates; un-flag when done. Hot-reload makes this fast.

**Why 30s decision_wait (§1 #8)?** Long enough that slow traces (up to 30s) complete + are sampled by latency policy. Shorter (10s) would miss long traces — they'd be evaluated mid-flight and the slow status wouldn't trigger.

**Why first-match policy ordering (§1 #6)?** Avoids double-counting AND avoids ambiguous "is this an error or a slow?" — pick one based on order.

**Why hot-reload via file-watch (§1 #11)?** Collector restart loses 30s of in-flight traces (decision_wait window). Hot-reload preserves the buffer; sampling decisions just start using the new flagged_tenants list. Critical for ops responsiveness during incidents.

**Why num_traces 100K (§1 #9)?** Sizing math: 25K spans/sec × 30s decision_wait × ~3 spans/trace = 250K spans buffered. With 100K trace limit (each trace averaging 3 spans), buffer ≈ 1GB. Higher would over-provision memory; lower would drop in-flight traces during traffic bursts.

**Why per-route latency budgets (§1 #4)?** A 5s ai-gateway call is normal (LLM round-trip); a 5s auth call is catastrophic. One global threshold can't capture both. Per-route lets ops express "what's slow for THIS route."

**Why metric per sampling decision (§1 #7)?** Operators need to know "what fraction of error traces are we keeping?" The metric answers directly. Without it, sampling decisions are invisible.

---

## §3 — API contract

### Tail sampling config

```yaml
# deploy/obs/tail_sampling_config.yaml
processors:
  tail_sampling:
    decision_wait: 30s
    num_traces: 100000
    expected_new_traces_per_sec: 1000
    policies:
      # Order matters — first match wins.
      - name: errors_100pct
        type: status_code
        status_code: { status_codes: [ERROR] }

      - name: http_5xx_100pct
        type: numeric_attribute
        numeric_attribute:
          key: http.response.status_code
          min_value: 500
          max_value: 599

      - name: flagged_tenants_100pct
        type: string_attribute
        string_attribute:
          key: tenant_id
          values: ${flagged_tenants}   # populated from flagged_tenants.yaml

      - name: slow_traces_100pct
        type: latency
        latency: { threshold_ms: 2000 }   # default; per-route via composite policy below

      - name: per_route_slow
        type: composite
        composite:
          composite_sub_policy:
            - name: chat_completion_slow
              type: latency
              latency: { threshold_ms: 5000 }
            - name: chat_completion_route
              type: string_attribute
              string_attribute: { key: route, values: [/v1/chat/completions] }
          rate_allocation:
            - { policy: chat_completion_slow, percent: 100 }

      - name: normal_sample_10pct
        type: probabilistic
        probabilistic: { sampling_percentage: 10 }
```

### Modified collector config

```yaml
# deploy/obs/otel-collector-config.yaml (modified)
service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [resource, attributes/pii_scrub, tail_sampling, batch]
      exporters: [otlp/tempo]
```

### Flag-tenant CLI

```rust
// services/ai-gateway/src/cli/flag_tenant.rs (FR-AI-021 subcommand)
pub async fn flag_tenant(tenant_id: Uuid, confirm: bool) -> Result<(), CliError> {
    require_confirm(confirm)?;
    let path = "deploy/obs/flagged_tenants.yaml";
    let mut current: Vec<String> = serde_yaml::from_str(&std::fs::read_to_string(path)?)?;
    if current.contains(&tenant_id.to_string()) {
        return Err(CliError::AlreadyFlagged);
    }
    current.push(tenant_id.to_string());
    std::fs::write(path, serde_yaml::to_string(&current)?)?;
    // Collector picks up via file-watch within 30s
    memory::emit(canonical::tenant_flagged_for_sampling(tenant_id, claims.subject_id, request_id)).await?;
    Ok(())
}
```

---

## §4 — Acceptance criteria

1. **100 normal traces → ~10 sampled** (10% probabilistic).
2. **1 error trace among 99 normal → error sampled + ~10 normal sampled** (error 100%).
3. **1 5xx trace among 99 normal → 5xx sampled + ~10 normal**.
4. **1 slow trace (3s, default route) among 99 normal → slow + ~10 normal**.
5. **chat-completions trace at 6s sampled** (per-route 5000ms budget exceeded).
6. **Flagged tenant: all traces sampled** — flag tenant_id; emit 100 traces from that tenant; all 100 in Tempo.
7. **Sampling metric per decision** — `obs_sampled_traces_total{reason="error"}` increments by 1 per error trace sampled.
8. **decision_wait blocks early decisions** — trace completing in 5s evaluated only after decision_wait expires.
9. **Hot-reload of flagged_tenants** — modify file; within 30s, new tenant's traces sampled at 100% without collector restart.
10. **First-match precedence** — error+slow trace counted as `reason="error"` not `reason="slow"`.
11. **num_traces buffer cap** — burst of 200K traces → oldest dropped; metric `obs_sampled_traces_total{reason="dropped"}` increments.
12. **Buffer-depth metric** — `obs_sampling_buffer_depth` reflects in-flight count.
13. **Per-route latency budget loaded from yaml** — change route_latency_budgets.yaml; policy uses new threshold.

---

## §5 — Verification

```bash
# deploy/obs/tests/sampling_test.sh
#!/usr/bin/env bash
set -euo pipefail

echo "Sending 100 normal traces + 1 error + 1 slow + 1 5xx..."
for i in $(seq 1 100); do
    emit_trace --trace-id "normal_$i" --duration-ms 500 --status ok
done
emit_trace --trace-id "error_1" --duration-ms 500 --status error
emit_trace --trace-id "slow_1" --duration-ms 3000 --status ok
emit_trace --trace-id "5xx_1" --duration-ms 500 --status ok --http-status 503

echo "Waiting for decision_wait + processing..."
sleep 40

# Query Tempo for sampled traces
sampled_normal=$(tempo_query --filter "name=normal_*" | jq 'length')
sampled_error=$(tempo_query --filter "name=error_1" | jq 'length')
sampled_slow=$(tempo_query --filter "name=slow_1" | jq 'length')
sampled_5xx=$(tempo_query --filter "name=5xx_1" | jq 'length')

# Assert
[ "$sampled_normal" -ge 7 ] && [ "$sampled_normal" -le 13 ] || { echo "FAIL: normal count $sampled_normal not in [7,13]"; exit 1; }
[ "$sampled_error" -eq 1 ] || { echo "FAIL: error not sampled"; exit 1; }
[ "$sampled_slow" -eq 1 ] || { echo "FAIL: slow not sampled"; exit 1; }
[ "$sampled_5xx" -eq 1 ] || { echo "FAIL: 5xx not sampled"; exit 1; }
echo "✅ sampling_test passed"
```

```bash
# Hot-reload test
flagged_yaml="deploy/obs/flagged_tenants.yaml"
echo "- 550e8400-e29b-41d4-a716-446655440000" > "$flagged_yaml"
sleep 35   # wait for file-watch reload
for i in $(seq 1 50); do
    emit_trace --trace-id "flagged_$i" --tenant-id "550e8400-..." --duration-ms 500 --status ok
done
sleep 40
flagged_count=$(tempo_query --filter "tenant_id=550e8400" | jq 'length')
[ "$flagged_count" -eq 50 ] || { echo "FAIL: only $flagged_count/50 flagged traces sampled"; exit 1; }
```

---

## §6 — Implementation skeleton

See §3.

---

## §7 — Dependencies

- **FR-OBS-001** — OTel collector running.
- **FR-OBS-005** — trace_id propagation (sampling decision is per-trace, not per-span).
- **FR-AI-021** — operator CLI for flag-tenant.

---

## §8 — Example payloads

### Sampling metric

```text
obs_sampled_traces_total{reason="error"} 423
obs_sampled_traces_total{reason="http_5xx"} 18
obs_sampled_traces_total{reason="flagged_tenant"} 1247
obs_sampled_traces_total{reason="slow"} 89
obs_sampled_traces_total{reason="normal_sample"} 9842
obs_sampled_traces_total{reason="dropped"} 0
```

### Flagged tenants config

```yaml
# deploy/obs/flagged_tenants.yaml
- 550e8400-e29b-41d4-a716-446655440000   # added 2026-05-15 — investigating customer complaint
```

### `obs.tenant_flagged_for_sampling` audit row

```json
{
  "kind": "obs.tenant_flagged_for_sampling",
  "payload": {
    "tenant_id": "550e8400-...",
    "flagged_by_subject_id": "...",
    "request_id": "cli_..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Adaptive sampling (auto-adjust rate based on volume) — slice 5+.
- Per-tenant sampling override beyond flag — slice 4+ with FR-AI-005 schema extension.
- Sampling for specific user_ids (not just tenants) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| decision_wait too short (slow trace evaluated mid-flight) | `obs_sampled_traces_total{reason=slow}` lower than expected | Increase wait | Config change |
| Memory pressure (100K traces in flight) | OTel OOM | Reduce `num_traces` | Config change |
| Sampling decision incorrect (high error trace dropped) | OBS investigation finds gap | Adjust policies | Config change |
| Flagged-tenants list growth | `obs_sampling_buffer_depth` near cap | Sev-2 alarm; ops un-flags | Operator removes from list |
| Hot-reload fails (file-watch not enabled) | Test fails | Enable extension; restart | One-time fix |
| Flagged tenant adds 100x normal traffic to Tempo | Tempo storage growth | Sev-2; un-flag if unintended | Operator action |
| First-match double-counting (regression) | Metric sum != trace count | Test fails → PR blocked | Fix policy order |
| Probabilistic sampler skewed | Statistical test | Investigate hash | Change sampler implementation |
| Burst > 200K traces | num_traces cap; oldest dropped | metric `dropped` increments | Auto-scale collector |
| Per-route budget yaml malformed | Collector config validation | Refuse to start | Operator fixes yaml |

---

## §11 — Notes

- The 30s decision_wait is the standard. Don't reduce below 20s (some slow traces won't complete in time).
- Tail-based sampling at the collector is the sole sampling decision point — services emit 100% (FR-AI-022 §1 #10), the collector reduces.
- Flagged-tenants is the operational primitive for "investigate this tenant deeply." Flag → wait → un-flag pattern.
- num_traces 100K caps memory at ~1GB at slice-2 load. Higher would over-provision; lower would drop traces during bursts.
- Per-route latency budgets recognise that "slow" is route-specific. ai-gateway 5s is normal; auth 5s is incident.
- The CLI `cyberos-ai flag-tenant` is the operator surface; emits memory audit row; 24h auto-unflag is FR-AI-021 enhancement (slice 4+).

---

## §12 — Shipped implementation (2026-06-15)

- `deploy/obs/otel-collector-config.yaml` and `services/obs-collector/config/otel-collector-config.yaml` now wire `tail_sampling` into the traces pipeline after `attributes/pii_scrub` and before `batch`.
- `services/obs-collector/src/tail_sampling.rs` implements deterministic first-match sampling decisions for errors, HTTP 5xx, flagged tenants, slow traces, normal 10% sampling, and drops, with YAML loaders for flagged tenants and per-route budgets.
- `services/obs-collector/src/config.rs` validates that the collector config keeps tail sampling traces-only, preserves the processor order, and includes the required policy types, decision wait, buffer size, and normal sample rate.
- `deploy/obs/tail_sampling_config.yaml`, `deploy/obs/flagged_tenants.yaml`, `deploy/obs/route_latency_budgets.yaml`, `deploy/obs/scripts/flag_tenant.sh`, and `deploy/obs/tests/sampling_test.sh` provide the operator-facing config and focused sampling gate.
- `cyberos-ai flag-tenant <tenant> --confirm` updates `deploy/obs/flagged_tenants.yaml` and emits the `obs.tenant_flagged_for_sampling` memory audit row.

Verification completed:

```bash
cd services
cargo fmt -p cyberos-obs-collector -p cyberos-ai-gateway
cargo test -p cyberos-obs-collector tail_sampling -- --nocapture
cargo test -p cyberos-obs-collector config::tests -- --nocapture
cargo test -p cyberos-ai-gateway --test cli_test -- --nocapture
./deploy/obs/tests/sampling_test.sh
cargo test -p cyberos-obs-collector --tests -- --nocapture
cargo test -p cyberos-ai-gateway --tests -- --nocapture
```

---

*End of FR-OBS-006. Status: done (10/10 target).*
