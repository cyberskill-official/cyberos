#!/usr/bin/env bash
# check-version-sync.sh - read-only comparator: root VERSION vs every stamped payload artifact.
# FR-IMP-068. Shared by payload-gate.yml (CI), .githooks/pre-commit (local), version.yml
# (bump proof), and the FR-IMP-069 release job. Zero side effects on the payload.
#
# usage: check-version-sync.sh [payload-dir]     default: <repo>/dist/cyberos
# exit 0   in sync   (prints "sync OK <version> across 6 artifacts")
# exit 10  drift     (one "DRIFT <path>: <found> != <expected>" line per drifted artifact)
# exit 2   unreadable: root VERSION missing/invalid, payload/artifact missing, tool missing
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
payload="${1:-$repo/dist/cyberos}"

err() { echo "cyberos-init: ERROR: $*" >&2; exit 2; }

command -v node  >/dev/null 2>&1 || err "node missing (needed to read JSON stamps)"
command -v unzip >/dev/null 2>&1 || err "unzip missing (needed to read the sealed cyberos.plugin)"

[ -f "$repo/VERSION" ] || err "$repo/VERSION missing"
expected="$(tr -d ' \n\r' < "$repo/VERSION")"
printf '%s' "$expected" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' \
  || err "root VERSION is not X.Y.Z semver (got '$expected')"
[ -d "$payload" ] || err "payload dir missing: $payload"

drift=0
check() { # $1 = artifact label, $2 = found value ("" = unreadable)
  [ -n "$2" ] || err "cannot read $1"
  if [ "$2" != "$expected" ]; then echo "DRIFT $1: $2 != $expected"; drift=1; fi
}

json_field() { # $1 = file, $2 = js accessor on parsed object
  node -e 'const fs=require("fs");try{const j=JSON.parse(fs.readFileSync(process.argv[1],"utf8"));const v=eval("j"+process.argv[2]);if(typeof v==="string")process.stdout.write(v);}catch(e){}' "$1" "$2" 2>/dev/null
}

# 1. plain VERSION file
v=""; [ -f "$payload/VERSION" ] && v="$(tr -d ' \n\r' < "$payload/VERSION")"
check "$payload/VERSION" "$v"

# 2. plugin manifest
check "$payload/plugin/.claude-plugin/plugin.json" "$(json_field "$payload/plugin/.claude-plugin/plugin.json" ".version")"

# 3. marketplace manifest (plugin entry)
check "$payload/.claude-plugin/marketplace.json" "$(json_field "$payload/.claude-plugin/marketplace.json" ".metadata.version")"

# 4. mcp package
check "$payload/mcp/package.json" "$(json_field "$payload/mcp/package.json" ".version")"

# 5. manifest.yaml cyberos_version
m=""; [ -f "$payload/manifest.yaml" ] && m="$(grep -E '^cyberos_version:' "$payload/manifest.yaml" | awk '{print $2}')"
check "$payload/manifest.yaml" "$m"

# 5b. manifest.yaml rules_sha - FR-IMP-074: the rule-drift fingerprint must exist and be a
# 64-hex sha256. Not compared against VERSION (it changes independently); presence + shape only.
rs=""; [ -f "$payload/manifest.yaml" ] && rs="$(grep -E '^rules_sha:' "$payload/manifest.yaml" | awk '{print $2}')"
if ! printf '%s' "$rs" | grep -Eq '^[0-9a-f]{64}$'; then
  echo "DRIFT $payload/manifest.yaml!rules_sha: '${rs:-<missing>}' is not a 64-hex sha256 (FR-IMP-074)"; drift=1
fi

# 6. plugin.json sealed inside cyberos.plugin (read in-stream, no extraction to disk)
s=""
if [ -f "$payload/cyberos.plugin" ]; then
  s="$(unzip -p "$payload/cyberos.plugin" .claude-plugin/plugin.json 2>/dev/null \
      | node -e 'const fs=require("fs");try{process.stdout.write(JSON.parse(fs.readFileSync(0,"utf8")).version||"")}catch(e){}' 2>/dev/null)"
fi
check "$payload/cyberos.plugin!.claude-plugin/plugin.json" "$s"

[ "$drift" -eq 0 ] || exit 10
echo "sync OK $expected across 6 artifacts"
