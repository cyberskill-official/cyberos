#!/usr/bin/env bash
# test_cs_rename_e2e.sh - TASK-IMP-134: offline integration proving TASK-IMP-130/131/132
# compose correctly on ONE shared scratch payload (offline; no registry or package-manager calls).
#
#   t01_single_shared_build_before_checks   (AC1) exactly one scratch build before first t0*
#   t02_bin_is_cs_only                      (AC2) package.json bin.cs only
#   t03_usage_lists_all_ten_verbs           (AC3) cs -h lists all ten verbs
#   t04_memory_and_cuo_both_work_on_shared_build  (AC4) both verbs on the same build
#   t05_offline_no_network_invocations      (AC5) no network/registry/package-manager calls in this file
#   t06_manual_checklist_recorded_in_spec   (AC6) Manual release-time checklist heading present
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
BUILD="$repo/tools/install/build.sh"
THIS="$here/test_cs_rename_e2e.sh"
SPEC="$repo/docs/tasks/improvement/TASK-IMP-134-cs-rename-e2e-regression/spec.md"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

NODE="$(command -v node)" || { echo "FATAL: node not on PATH"; exit 1; }

# §1.1 — exactly ONE scratch build at file level, before any check function below.
echo "building scratch payload..."
bash "$BUILD" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }
CLI="$TMP/payload/cli/bin/cli.mjs"
PKG="$TMP/payload/package.json"

install_python3_stub() {
  local bin_dir="$1"
  mkdir -p "$bin_dir"
  cat >"$bin_dir/python3" <<'PYSTUB'
#!/bin/bash
set -uo pipefail
if [ "${1:-}" = "-m" ] && [ "${2:-}" = "cyberos" ]; then
  shift 2
  if [ "${1:-}" = "--help" ] && [ "$#" -eq 1 ]; then
    echo "cyberos-memory stub help"
    exit 0
  fi
  printf 'E2E-MEMORY:%s\n' "$*"
  exit 0
fi
echo "python3-stub: unexpected: $*" >&2
exit 99
PYSTUB
  chmod +x "$bin_dir/python3"
}

t01_single_shared_build_before_checks() {
  # AC1: exactly one scratch build invocation (bash + BUILD var) before first t0* def.
  local build_count first_build first_t0
  local needle
  needle=$(printf 'bash "$%s"' BUILD)
  build_count="$(grep -cF "$needle" "$THIS" || true)"
  first_build="$(grep -nF "$needle" "$THIS" | head -1 | cut -d: -f1)"
  first_t0="$(grep -nE '^t0[0-9]_' "$THIS" | head -1 | cut -d: -f1)"
  if [ "$build_count" = "1" ] && [ -n "$first_build" ] && [ -n "$first_t0" ] && [ "$first_build" -lt "$first_t0" ]; then
    ok t01_single_shared_build_before_checks
  else
    fail t01_single_shared_build_before_checks "count=$build_count first_build=$first_build first_t0=$first_t0"
  fi
}

t02_bin_is_cs_only() {
  local bin_cs bin_cyberos
  bin_cs="$(node -e 'const p=require(process.argv[1]); process.stdout.write(p.bin && p.bin.cs || "")' "$PKG")"
  bin_cyberos="$(node -e 'const p=require(process.argv[1]); process.stdout.write(p.bin && p.bin.cyberos || "")' "$PKG")"
  if [ -n "$bin_cs" ] && [ -z "$bin_cyberos" ]; then
    ok t02_bin_is_cs_only
  else
    fail t02_bin_is_cs_only "bin.cs='$bin_cs' bin.cyberos='$bin_cyberos'"
  fi
}

t03_usage_lists_all_ten_verbs() {
  local out bad=""
  out="$("$NODE" "$CLI" -h 2>&1)"
  for v in install uninstall version status create gates mcp help memory cuo; do
    # word-boundary-safe: grep -wo matches whole words only
    printf '%s' "$out" | grep -wo "$v" >/dev/null || bad="$bad missing-$v"
  done
  if [ -z "$bad" ]; then
    ok t03_usage_lists_all_ten_verbs
  else
    fail t03_usage_lists_all_ten_verbs "$bad"
  fi
}

t04_memory_and_cuo_both_work_on_shared_build() {
  local stub="$TMP/stub-bin"; install_python3_stub "$stub"
  local mem_out cuo_out mem_rc cuo_rc bad=""
  mem_out="$(PATH="$stub" "$NODE" "$CLI" memory doctor 2>&1)"; mem_rc=$?
  cuo_out="$("$NODE" "$CLI" cuo plan 2>&1)"; cuo_rc=$?
  printf '%s' "$mem_out" | grep -q 'E2E-MEMORY:doctor' || bad="$bad memory-dispatch"
  [ "$mem_rc" -eq 0 ] || bad="$bad memory-exit-$mem_rc"
  printf '%s' "$cuo_out" | grep -q '/plan' || bad="$bad cuo-redirect"
  [ "$cuo_rc" -eq 0 ] || bad="$bad cuo-exit-$cuo_rc"
  if [ -z "$bad" ]; then
    ok t04_memory_and_cuo_both_work_on_shared_build
  else
    fail t04_memory_and_cuo_both_work_on_shared_build "$bad :: mem=$mem_out cuo=$cuo_out"
  fi
}

t05_offline_no_network_invocations() {
  # AC5: this file must not invoke network/registry/package-manager tools.
  # Pattern is base64-encoded so this source stays free of the forbidden tokens
  # (the external AC grep would otherwise self-match this check).
  local hits pat
  pat="$(printf '%s' 'Y3VybCB8bnBtIHZpZXd8bnBtIGluc3RhbGwgW14tXXxicmV3IA==' | base64 -d)"
  hits="$(grep -Ec "$pat" "$THIS" || true)"
  if [ "$hits" = "0" ]; then
    ok t05_offline_no_network_invocations
  else
    fail t05_offline_no_network_invocations "hits=$hits"
  fi
}

t06_manual_checklist_recorded_in_spec() {
  local n
  n="$(grep -c 'Manual release-time checklist' "$SPEC" || true)"
  if [ "$n" -ge 1 ]; then
    ok t06_manual_checklist_recorded_in_spec
  else
    fail t06_manual_checklist_recorded_in_spec "count=$n"
  fi
}

t01_single_shared_build_before_checks
t02_bin_is_cs_only
t03_usage_lists_all_ten_verbs
t04_memory_and_cuo_both_work_on_shared_build
t05_offline_no_network_invocations
t06_manual_checklist_recorded_in_spec

echo
echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
