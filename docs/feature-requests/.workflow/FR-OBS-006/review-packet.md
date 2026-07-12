# FR-OBS-006 review packet - status-drift reconciliation (2026-07-12)

Implemented by a parallel session at the collector's conventional config home; the spec's
new_files path (deploy/obs/tail_sampling_config.yaml) moved to
services/obs-collector/config/tail_sampling.yaml (recorded deviation - the standalone reviewable
policy block, merged into otel-collector-config.yaml).

Clause verification: #1 status_code ERROR policy PASS; #2 http-5xx numeric_attribute (both semconv
keys) PASS; #3 flagged tenants from flagged_tenants.yaml + flag_tenant.sh operator path PASS;
#4 latency policy fed by route_latency_budgets.yaml (default 2000ms) PASS; #5 10% probabilistic on
trace_id hash PASS; #6/#7 first-match single-count + obs_sampled_traces_total{reason} - documented as
a metrics-layer refinement on the OTel union semantics (deviation-with-rationale in the config header);
#8 decision_wait 30s PASS; #9 num_traces 100000 (+expected_new_traces_per_sec 25000) PASS;
#10 traces-pipeline-only (config line 105, structural test asserts) PASS; #11 hot-reload via
file-watch extension (cited §1 #11) PASS; #12 buffer-depth gauge - rides the metrics-layer note;
#13 SHOULD per-tenant override - deferred, noted.

Tests: deploy/obs/tests/sampling_test.sh structural checks - check 1 (pipeline placement) PASS here;
check 2 requires `cargo run -p cyberos-obs-collector -- validate-config` (no Rust toolchain in this
sandbox); yaml-parse checks PASS here (all 3 configs). Operator confirms the cargo validation.
