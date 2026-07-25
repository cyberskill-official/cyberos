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
  printf '%s
' 'aws_access_key_id = AKIA0B1C2D3E4F5G6H7J' 'aws_secret_access_key = wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLZKEY' > "$tmp/planted-aws.env"
  echo "check-secrets: probing planted AWS key under $tmp"
  if gitleaks dir --no-banner --exit-code=1 "$tmp" >/tmp/gitleaks-probe.log 2>&1; then
    echo "check-secrets: ERROR — planted AWS key was NOT detected" >&2; cat /tmp/gitleaks-probe.log >&2 || true; exit 1
  fi
  echo "check-secrets: probe ok (gitleaks failed on planted key as expected)"; exit 0
fi
args=(dir --no-banner --exit-code=1)
[ -f "$cfg" ] && args+=(--config "$cfg")
echo "check-secrets: gitleaks ${args[*]} $root"
gitleaks "${args[@]}" "$root"
