#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <tenant_id> [flagged_tenants.yaml]" >&2
}

if [[ "${1:-}" == "" ]]; then
  usage
  exit 64
fi

tenant_id="$1"
file="${2:-deploy/obs/flagged_tenants.yaml}"
mkdir -p "$(dirname "$file")"
touch "$file"

if awk -v tenant="$tenant_id" '
  {
    sub(/#.*/, "", $0)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", $0)
    sub(/^-/, "", $0)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", $0)
    if ($0 == tenant) found = 1
  }
  END { exit(found ? 0 : 1) }
' "$file"; then
  echo "already_flagged tenant_id=${tenant_id} file=${file}"
  exit 0
fi

printf -- "- %s # added %s\n" "$tenant_id" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$file"
echo "flagged tenant_id=${tenant_id} file=${file}"
