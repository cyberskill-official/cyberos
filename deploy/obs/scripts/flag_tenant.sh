#!/usr/bin/env bash
# TASK-OBS-006 §1 #3 - flag or unflag a tenant for 100% trace sampling by editing flagged_tenants.yaml.
# The collector hot-reloads on the file change (no restart). This is the same edit the
# `cyberos-ai flag-tenant <id> --confirm` subcommand performs; use it directly when the CLI is not handy.
#
#   flag_tenant.sh add    <tenant_id>
#   flag_tenant.sh remove <tenant_id>
#   flag_tenant.sh list
#
# Override the target file with OBS_FLAGGED_TENANTS.
set -euo pipefail

ACTION="${1:-list}"
TENANT="${2:-}"
DEFAULT_FILE="$(cd "$(dirname "$0")/../../../services/obs-collector/config" 2>/dev/null && pwd)/flagged_tenants.yaml"
FILE="${OBS_FLAGGED_TENANTS:-$DEFAULT_FILE}"

if [ ! -f "$FILE" ]; then
  echo "FAIL: flagged_tenants.yaml not found at $FILE" >&2
  exit 1
fi

python3 - "$ACTION" "$TENANT" "$FILE" <<'PY'
import sys, yaml
action, tenant, path = sys.argv[1], sys.argv[2], sys.argv[3]
doc = yaml.safe_load(open(path)) or {}
lst = list(doc.get("flagged_tenants") or [])
if action == "list":
    print("\n".join(lst) if lst else "(none)")
    sys.exit(0)
if action not in ("add", "remove") or not tenant:
    sys.stderr.write("usage: flag_tenant.sh add|remove|list <tenant_id>\n")
    sys.exit(2)
if action == "add" and tenant not in lst:
    lst.append(tenant)
if action == "remove":
    lst = [t for t in lst if t != tenant]
doc["flagged_tenants"] = lst
yaml.safe_dump(doc, open(path, "w"), default_flow_style=False, sort_keys=False)
print(f"{action} {tenant}: {len(lst)} tenant(s) now flagged at 100% sampling")
PY
