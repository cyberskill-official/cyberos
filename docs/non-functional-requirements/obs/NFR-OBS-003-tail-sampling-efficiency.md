---
id: NFR-OBS-003
title: "Tail-sampling efficiency — 100% errors + 10% normal; sampler CPU overhead < 2%"
module: OBS
category: performance
priority: MUST
verification: T
phase: P0
slo: "Tail sampler CPU overhead < 2% of collector CPU; sampling rates exact per TASK-OBS-006 policy"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-OBS-006, TASK-OBS-001]
---

## §1 — Statement (BCP-14 normative)

1. The OTel collector tail-sampler **MUST** apply the TASK-OBS-006 policies (100% errors, 100% 5xx, 100% slow, 100% flagged-tenants, 10% normal) and the measured sampling rates **MUST** match the policy targets within ±1% over a 1-hour window.
2. The tail-sampler **MUST NOT** consume more than 2% of the collector pod's CPU budget at steady-state load (~25k spans/sec, slice-2 traffic). Memory consumption capped at 1GB per TASK-OBS-006 §1 #9 (`num_traces: 100000`).
3. Sampling decisions **MUST** be deterministic for a given trace — the same trace seen twice (e.g., via a replay) **MUST** produce the same sample/drop decision.
4. The metric `obs_sampled_traces_total{reason}` **MUST** be emitted per sampling decision with reason ∈ {`error`, `http_5xx`, `flagged_tenant`, `slow`, `normal_sample`, `dropped`}.
5. The 10% normal-sample rate **MUST** be uniform across `trace_id` hashspace — no trace_id-prefix bias.

## §2 — Why this constraint

Tail sampling is the only way to keep trace storage cost bounded while preserving the high-value error/slow cases. The 2% CPU ceiling is the budget below which the collector can co-tenant with other observability workloads (logs, metrics) on the same node; above 2%, the sampler becomes the bottleneck and traces queue up (eventually `obs_sampling_buffer_depth` alarms — see TASK-OBS-006 §1 #12). Deterministic decisions matter for debugging — operator runs the same query twice, gets the same sampled set; a stochastic resampling between runs would be confusing.

## §3 — Measurement

- Collector self-metric `otelcol_processor_tail_sampling_count_traces_sampled{policy}` divided by `otelcol_processor_tail_sampling_count_traces_seen` per minute. Should match policy targets ±1%.
- Container CPU usage on the collector pod, scoped to the tail-sampling processor (via Go pprof flame graph monthly review). Steady-state < 2%.
- The `obs_sampling_buffer_depth` gauge (TASK-OBS-006 §1 #12) — alarm at > 90% of 100k cap.

## §4 — Verification

- Load test `deploy/loadtest/obs-collector-sampling.k6.js` (T) — drives 50k spans/sec for 10 minutes; asserts collector CPU < 2% and sampling rates match policy.
- Daily metric check (T) — CI cron compares `obs_sampled_traces_total{reason=normal_sample}` against total seen; asserts ratio 9-11%.

## §5 — Failure handling

- Sampler CPU > 2% sustained → sev-3; review whether collector pod needs vertical scale or whether policy ordering can short-circuit faster (e.g., put `flagged_tenant` first if a single tenant generates 50%+ traffic).
- Sampling rate deviates > 5% from policy → sev-2; configuration drift, redeploy from `deploy/obs/tail_sampling_config.yaml`.
- Buffer depth > 90% → sev-2; spans being dropped; immediate scale-up.

---

*End of NFR-OBS-003.*
