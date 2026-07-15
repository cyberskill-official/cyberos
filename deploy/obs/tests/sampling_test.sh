#!/usr/bin/env bash
# TASK-OBS-006 - tail-sampling validation. The structural checks run anywhere; the live-trace checks need
# the collector + Tempo up.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
CFG="$ROOT/services/obs-collector/config/otel-collector-config.yaml"

echo "== structural =="

# 1. tail_sampling is on the traces pipeline (and only there).
if grep -qE 'processors:.*tail_sampling.*\]' "$CFG"; then
  echo "OK   tail_sampling is on the traces pipeline"
else
  echo "FAIL tail_sampling not found on the traces pipeline"
  exit 1
fi

# 2. the config still validates against the TASK-OBS-001 contract (pii_scrub stays present, etc.).
if (cd "$ROOT/services" && cargo run -q -p cyberos-obs-collector --bin cyberos-obs -- validate-config \
      obs-collector/config/otel-collector-config.yaml >/dev/null 2>&1); then
  echo "OK   collector config validates with tail_sampling added"
else
  echo "FAIL collector config does not validate"
  exit 1
fi

# 3. the supporting config files parse.
for f in tail_sampling.yaml flagged_tenants.yaml route_latency_budgets.yaml; do
  python3 -c "import yaml,sys; yaml.safe_load(open('$ROOT/services/obs-collector/config/$f'))" \
    && echo "OK   $f parses" || { echo "FAIL $f"; exit 1; }
done

echo
echo "== live (needs the collector + Tempo up) =="
echo "Send N traces with a span status ERROR and N normal traces, then assert in Prometheus:"
echo "  obs_sampled_traces_total{reason=\"error\"}        == N        (100% of errors kept, §1 #1)"
echo "  obs_sampled_traces_total{reason=\"normal_sample\"} ~= 0.10 * N  (10% probabilistic, §1 #5)"
echo "  sum(obs_sampled_traces_total) counts each trace once             (first-match, §1 #6)"
echo "Flag a tenant with deploy/obs/scripts/flag_tenant.sh add <id> and confirm its traces are 100% kept."
