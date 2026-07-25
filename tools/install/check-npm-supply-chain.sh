#!/usr/bin/env bash
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
audit_json=""; license_pkg=""
while [ $# -gt 0 ]; do
  case "$1" in
    --root) root="$(cd "$2" && pwd)"; shift 2 ;;
    --audit-json) audit_json="$2"; shift 2 ;;
    --license-package) license_pkg="$2"; shift 2 ;;
    *) echo "check-npm-supply-chain: unknown arg: $1" >&2; exit 2 ;;
  esac
done
if [ -n "$audit_json" ]; then
  [ -f "$audit_json" ] || { echo "missing $audit_json" >&2; exit 2; }
  node -e '
    const fs=require("fs"); const j=JSON.parse(fs.readFileSync(process.argv[1],"utf8"));
    let high=0, critical=0; const meta=j.metadata&&j.metadata.vulnerabilities;
    if(meta){high=Number(meta.high||0); critical=Number(meta.critical||0);}
    for (const v of Object.values(j.vulnerabilities||{})) {
      const s=String(v.severity||"").toLowerCase();
      if(s==="high") high++; if(s==="critical") critical++;
    }
    if(high+critical>0){ console.error("planted audit high="+high+" critical="+critical); process.exit(1);}
    console.log("planted audit clean");
  ' "$audit_json"; exit 0
fi
if [ -n "$license_pkg" ]; then
  node "$here/check-npm-licenses.mjs" --allowlist "$here/npm-license-allowlist.txt" "$license_pkg"; exit $?
fi
for d in "$root/tools/install/mcp" "$root/tools/install/docs-tools"; do
  [ -f "$d/package.json" ] || { echo "missing package.json in $d" >&2; exit 2; }
  echo "==> npm audit --audit-level=high ($d)"
  (cd "$d" && npm audit --audit-level=high)
  node "$here/check-npm-licenses.mjs" --allowlist "$here/npm-license-allowlist.txt" "$d/package.json"
done
echo "check-npm-supply-chain: ok"
