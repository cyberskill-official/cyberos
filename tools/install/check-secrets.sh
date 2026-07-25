#!/usr/bin/env bash
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
mode="scan"; soft=0
while [ $# -gt 0 ]; do
  case "$1" in
    --probe-fixture) mode="probe"; shift ;;
    --soft) soft=1; shift ;;
    --root) root="$(cd "$2" && pwd)"; shift 2 ;;
    *) echo "check-secrets: unknown arg: $1" >&2; exit 2 ;;
  esac
done
if ! command -v gitleaks >/dev/null 2>&1; then
  if [ "$soft" = "1" ]; then echo "check-secrets: gitleaks not installed — soft-skip (CI hard-fails via secret-scan.yml)"; exit 0; fi
  echo "check-secrets: gitleaks not found on PATH" >&2; exit 2
fi
cfg="$root/.gitleaks.toml"
if [ "$mode" = "probe" ]; then
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/gitleaks-probe.XXXXXX")"
  trap 'rm -rf "$tmp"' EXIT
  printf '%s\n' \
    'aws_access_key_id = AKIA0B1C2D3E4F5G6H7J' \
    'aws_secret_access_key = wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLZKEY' \
    > "$tmp/planted-aws.env"
  probe_log="$tmp/gitleaks-probe.log"
  probe_json="$tmp/gitleaks-probe.json"
  echo "check-secrets: probing planted AWS key under $tmp"
  # Exit 1 means leak or error — require the planted finding in the JSON report
  # (console output may only say "leaks found: N" without the secret body).
  set +e
  gitleaks dir --no-banner --exit-code=1 \
    --report-format json --report-path "$probe_json" \
    "$tmp" >"$probe_log" 2>&1
  rc=$?
  set -e
  if [ "$rc" -eq 0 ]; then
    echo "check-secrets: ERROR — planted AWS key was NOT detected" >&2
    cat "$probe_log" >&2 || true
    exit 1
  fi
  if [ ! -s "$probe_json" ] || ! grep -Eq 'AKIA0B1C2D3E4F5G6H7J|wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLZKEY|generic-api-key|aws-access-key' "$probe_json"; then
    echo "check-secrets: ERROR — gitleaks exited $rc but probe report lacks planted finding" >&2
    cat "$probe_log" >&2 || true
    cat "$probe_json" >&2 || true
    exit 1
  fi
  echo "check-secrets: probe ok (gitleaks failed on planted key as expected)"
  exit 0
fi
args=(dir --no-banner --exit-code=1)
[ -f "$cfg" ] && args+=(--config "$cfg")
echo "check-secrets: gitleaks ${args[*]} $root"
gitleaks "${args[@]}" "$root"
