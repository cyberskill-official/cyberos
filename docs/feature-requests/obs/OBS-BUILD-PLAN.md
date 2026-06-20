# OBS module build plan (FR-OBS-001..009)

Written 2026-06-20. obs is the live front of the locked P0 path (AI -> OBS -> AUTH -> MCP -> CHAT) and
all 9 FRs are `ready_to_implement`. FR-OBS-001 (the collector) has shipped its slice-1 scaffold. This
plan sequences the other eight and says, per FR, what to build, what test proves it, and how the gate
applies. Implementation is a toolchain step (cargo), run on your machine - this plan is the spec-to-code
map, not the code.

## Dependency order (within obs)

```
FR-OBS-001 collector (DONE scaffold)
  ├─ FR-OBS-002 tenant-aware Grafana proxy   [+ FR-AUTH-004]   -> new crate services/obs-proxy
  ├─ FR-OBS-003 RED metrics SDK                                 -> metrics SDK + ai-gateway wiring
  └─ FR-OBS-006 tail sampling (SHOULD)                          -> collector config + cli
FR-OBS-004 LangSmith AI traces   [+ FR-AI-022]                  -> ai-gateway/src/langsmith/
FR-OBS-005 TraceContext correlation   [001+003+004]            -> ai-gateway propagation
FR-OBS-007 obs-router: Alertmanager -> CUO   [002+003]         -> NEW crate services/obs-router
FR-OBS-008 obs-compliance-view   [002]                         -> NEW crate services/obs-compliance-view
FR-OBS-009 chain-of-custody manifest   [008]                   -> obs-compliance-view (Ed25519)
```

Two cross-module dependencies to confirm before starting the dependent FR: FR-OBS-002 needs FR-AUTH-004
(tenant context), and FR-OBS-004 needs FR-AI-022 (the AI trace hook - AI has 2 ready_to_implement FRs;
check whether AI-022 is one of them or already done).

## Per-FR

### FR-OBS-001 - OTel Collector + LGTM stack  (MUST, done scaffold)
Shipped: `services/obs-collector` with the canonical `otel-collector-config.yaml` (OTLP receivers,
resource + pii_scrub processors deleting PROMPT_TEXT/RESPONSE_TEXT/USER_EMAIL/CCCD + a secret-pattern
rule, batch 10s/1024, loki + prometheusremotewrite + otlp/tempo exporters), the `cyberos-obs`
validate-config / validate-tokens pre-flight, self-metric constants, and the bearer-token parser.
Tests: `validate_accepts_canonical_config`, `validate_rejects_missing_pii_scrub`, the `cyberos_obs`
integration test. Remaining: the live LGTM stack (Helm + docker-compose for otelcol-contrib + Loki +
Prometheus + Tempo + Grafana) lands at `deploy/obs/`. Gate: already covered by the obs goldenset.

### FR-OBS-002 - Tenant-aware Grafana proxy  (MUST; dep 001 + AUTH-004)
Build out `services/obs-proxy` (currently a stub): a Rust proxy that AST-injects `tenant_id` into LogQL
and PromQL queries so a tenant can never read another's telemetry. Files: `inject/logql.rs`,
`inject/promql.rs` (parse + inject), `auth.rs` (tenant from the AUTH JWT), `audit.rs` (emit to the
memory chain). Test plan: parse-and-inject unit tests per query language (a query without a tenant
filter gets one; a query that tries to spoof another tenant is rejected), plus an auth test. Invariant:
no cross-tenant read path exists.

### FR-OBS-003 - Per-service RED metrics  (MUST; dep 001)
A `cyberos` metrics SDK emitting rate / errors / duration per service, wired first into ai-gateway
(`handlers/chat.rs`, `main.rs`). Test plan: a handler increments the request counter, records duration,
and increments the error counter on the error branch; metric names/labels match the obs-collector
constants. Invariant: every service exposes RED with consistent names.

### FR-OBS-004 - LangSmith AI traces  (MUST; dep 001 + AI-022)
`services/ai-gateway/src/langsmith/` (`client.rs`, `payload.rs`, `mod.rs`): self-hosted + per-tenant AI
trace export. Test plan: payload shape matches the LangSmith schema; per-tenant routing; redaction
before send. Confirm FR-AI-022 first.

### FR-OBS-005 - W3C TraceContext correlation  (MUST; dep 001+003+004)
Propagate `traceparent` across logs, metrics, traces, and AI calls so one request id ties them together
(ai-gateway `main.rs` middleware). Test plan: an incoming traceparent is propagated to downstream spans
and log lines; a missing one is generated. Invariant: one correlation id end to end.

### FR-OBS-006 - Tail-based sampling  (SHOULD; dep 001)
Collector tail-sampling policy: keep 100% of errors / 5xx / slow traces, sample the rest
(`ai-gateway/src/cli/flag_tenant.rs` for the tenant flag). Test plan: policy keeps error/slow,
down-samples normal. Lowest priority of the set.

### FR-OBS-007 - obs-router (Alertmanager -> CUO)  (MUST; dep 002+003)
NEW crate `services/obs-router`: receive Alertmanager webhooks, call the CUO `obs.triage-alert@1` skill,
post to chat, handle acks (`alertmanager_webhook.rs`, `cuo_triage.rs`, `chat_post.rs`, `ack_handler.rs`).
Test plan: webhook -> triage -> chat-post happy path; ack dedupe; malformed webhook rejected.

### FR-OBS-008 - obs-compliance-view  (MUST; dep 002)
NEW crate `services/obs-compliance-view`: pre-built read-only views (EU AI Act / audit) with auth +
chain proof + JSON/PDF export (`auth.rs`, `chain_proof.rs`, `export/json.rs`, `export/pdf.rs`). Test
plan: view is read-only and tenant-scoped; export round-trips; chain proof validates.

### FR-OBS-009 - Chain-of-custody manifest  (MUST; dep 008)
Add to obs-compliance-view: an Ed25519-signed manifest over every compliance export, with a
`verify_manifest` bin (`manifest.rs`, `manifest_signing.rs`, `manifest_pdf.rs`,
`bin/verify_manifest.rs`). Test plan: sign -> verify round-trip; a tampered manifest fails; the bin
exits non-zero on a bad signature. Invariant: every export is independently verifiable.

## Keep the gate in step with the new crates

The obs goldenset (`modules/obs/.awh/goldenset.yaml`) and caf profile currently cover
`cyberos-obs-collector`. As FR-OBS-002/007/008 add the `obs-proxy`, `obs-router`, and
`obs-compliance-view` crates, extend both: add a golden-set task `cd services && cargo test -p
cyberos-obs-<crate>` per new crate, and append it to the profile's `RUN_COMMANDS`. That keeps each new
obs surface inside the same awh+caf gate.

## How to ship each FR

Run `ship-feature-requests` for the obs FRs in the dependency order above. Each FR flows through the
chain to step 28 (awh rerun) + step 29 (caf target health + audit); `testing -> done` flips only on
`awh GREEN AND caf CLEAN`. Capture the obs baseline once before the first run
(`awh eval modules/obs/.awh/goldenset.yaml --base-dir . --seeds 1 --out modules/obs/.awh/eval-baseline.json`).
There is no `awh lock` step for obs yet because the held-out acceptance is a lib unit test
(`validate_rejects_missing_pii_scrub`), not a separate `tests/` file. Once FR-OBS-001 grows a dedicated
integration test under `services/obs-collector/tests/`, point the goldenset's acceptance task at it
with `--test <name>` and seal it with `awh lock services/obs-collector/tests`.
