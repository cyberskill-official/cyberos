#!/usr/bin/env bash
# check-chain-coverage.sh - prove the payload carries every skill its own workflow docs reference,
# and that vendored author/audit skills ship as complete pairs. FR-SKILL-116.
# Read-only over the payload; run by build.sh as its final step and reusable by CI/release jobs.
#
# usage: check-chain-coverage.sh [payload-dir]     default: <repo>/dist/cyberos
#   env: CYBEROS_CHAIN_ALLOWLIST=<file>            default: <this dir>/chain-allowlist.txt
# exit 0   chain covered, pairs complete ("chain OK: N referenced, M vendored, K allowlisted")
# exit 10  MISSING <skill> (referenced by <doc>)  |  UNPAIRED <skill>
# exit 2   payload dir / workflow doc unreadable, or 0 references extracted (structure changed)
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
payload="${1:-$repo/dist/cyberos}"
allowfile="${CYBEROS_CHAIN_ALLOWLIST:-$here/chain-allowlist.txt}"

err() { echo "cyberos-init: ERROR: $*" >&2; exit 2; }
[ -d "$payload" ] || err "payload dir missing: $payload"
chain_doc="$payload/cuo/ship-feature-requests.md"
[ -f "$chain_doc" ] || err "workflow doc unreadable: $chain_doc"

# --- allowlist (comment-stripped, first field per line) ---
declare -A ALLOW=()
if [ -f "$allowfile" ]; then
  while read -r name; do [ -n "$name" ] && ALLOW["$name"]=1; done \
    < <(sed 's/#.*//' "$allowfile" | awk 'NF{print $1}')
fi

# --- extraction: chain doc `skill:` keys + command-doc backticked *-author/-audit tokens ---
declare -A REF_DOC=()
while read -r s; do [ -n "$s" ] && REF_DOC["$s"]="cuo/ship-feature-requests.md"; done \
  < <(grep -Eo 'skill: *[a-z0-9-]+' "$chain_doc" | awk '{print $2}' | sort -u)
for cmd in "$payload"/plugin/commands/*.md; do
  [ -f "$cmd" ] || continue
  while read -r s; do
    [ -n "$s" ] && [ -z "${REF_DOC[$s]:-}" ] && REF_DOC["$s"]="plugin/commands/$(basename "$cmd")"
  done < <(grep -Eo '`[a-z0-9]+(-[a-z0-9]+)*-(author|audit)`' "$cmd" | tr -d '`' | sort -u)
done
[ "${#REF_DOC[@]}" -gt 0 ] || err "0 skill references extracted from $chain_doc - doc structure changed under the checker"

fail=0
# --- MISSING rule: every referenced skill vendored in BOTH trees (unless allowlisted) ---
for s in "${!REF_DOC[@]}"; do
  [ -n "${ALLOW[$s]:-}" ] && continue
  if [ ! -f "$payload/cuo/skills/$s/SKILL.md" ] || [ ! -f "$payload/plugin/skills/$s/SKILL.md" ]; then
    echo "MISSING $s (referenced by ${REF_DOC[$s]})"; fail=1
  fi
done

# --- UNPAIRED rule: vendored -author/-audit twins ship together (unless allowlisted) ---
vendored=0
for d in "$payload"/cuo/skills/*/; do
  [ -d "$d" ] || continue
  s="$(basename "$d")"; vendored=$((vendored+1))
  [ -n "${ALLOW[$s]:-}" ] && continue
  case "$s" in
    *-author) twin="${s%-author}-audit" ;;
    *-audit)  twin="${s%-audit}-author" ;;
    *) continue ;;
  esac
  [ -d "$payload/cuo/skills/$twin" ] || { echo "UNPAIRED $s"; fail=1; }
done

# --- rot warning: allowlist entries nothing references and no dir matches ---
for a in "${!ALLOW[@]}"; do
  if [ -z "${REF_DOC[$a]:-}" ] && [ ! -d "$payload/cuo/skills/$a" ]; then
    echo "cyberos-init: WARNING: allowlist entry '$a' is unreferenced and unvendored - stale?" >&2
  fi
done

[ "$fail" -eq 0 ] || exit 10
echo "chain OK: ${#REF_DOC[@]} referenced, $vendored vendored, ${#ALLOW[@]} allowlisted"
