#!/usr/bin/env bash
# check-chain-coverage.sh - prove the payload carries every skill its own workflow docs reference,
# and that vendored author/audit skills ship as complete pairs. TASK-SKILL-116.
# Read-only over the payload; run by build.sh as its final step and reusable by CI/release jobs.
#
# usage: check-chain-coverage.sh [payload-dir]     default: <repo>/dist/cyberos
#   env: CYBEROS_CHAIN_ALLOWLIST=<file>            default: <this dir>/chain-allowlist.txt
# exit 0   chain covered, pairs complete ("chain OK: N referenced, M vendored, K allowlisted")
# exit 10  MISSING <skill> (referenced by <doc>)  |  UNPAIRED <skill>
# exit 2   payload dir / workflow doc unreadable, or 0 references extracted (structure changed)
#
# ── bash 3.2 ────────────────────────────────────────────────────────────────
# This script used `declare -A` (associative arrays), which is bash 4+. macOS ships
# bash 3.2 (GPLv2, frozen since 2007). On every Mac this gate did not run: `declare -A`
# errored, the arrays silently became indexed ones, and the checker reported success
# without checking anything. It has been green-by-absence since 53db4e30f.
#
# Rewritten with newline-delimited lists + grep, which is bash 3.2 clean. If you ever
# reach for `declare -A` here again, add the version guard instead.
set -uo pipefail

case "${BASH_VERSION:-}" in
  "") echo "check-chain-coverage: not running under bash" >&2; exit 2 ;;
esac

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
payload="${1:-$repo/dist/cyberos}"
allowfile="${CYBEROS_CHAIN_ALLOWLIST:-$here/chain-allowlist.txt}"

err() { echo "cyberos-install: ERROR: $*" >&2; exit 2; }
[ -d "$payload" ] || err "payload dir missing: $payload"
chain_doc="$payload/cuo/ship-tasks.md"
[ -f "$chain_doc" ] || err "workflow doc unreadable: $chain_doc"

# Reduced profile: a payload built without skill bodies vendors ZERO skills and runs
# doc-driven (the documented floor - TASK-CUO-209 §1 #7). Coverage semantics apply to
# full-profile payloads; a PARTIALLY vendored payload still fails below (that is drift).
if [ -z "$(ls -A "$payload/cuo/skills" 2>/dev/null)" ]; then
  echo "chain SKIP: reduced profile (0 vendored skills, doc-driven floor)"
  exit 0
fi

# --- allowlist: newline-delimited names (comment-stripped, first field per line) ---
ALLOW=""
if [ -f "$allowfile" ]; then
  ALLOW="$(sed 's/#.*//' "$allowfile" | awk 'NF{print $1}' | sort -u)"
fi
allowed() {  # allowed <name> -> 0 if in allowlist
  [ -n "$ALLOW" ] || return 1
  printf '%s\n' "$ALLOW" | grep -qxF -- "$1"
}

# --- extraction: chain doc `skill:` keys + command-doc backticked *-author/-audit tokens ---
# REF_DOC is a TAB-separated "skill<TAB>doc" table; first mention of a skill wins.
REF_DOC=""
ref_doc_of() { printf '%s\n' "$REF_DOC" | awk -F'\t' -v k="$1" '$1==k{print $2; exit}'; }
ref_has()    { [ -n "$REF_DOC" ] && printf '%s\n' "$REF_DOC" | cut -f1 | grep -qxF -- "$1"; }

while read -r s; do
  [ -n "$s" ] || continue
  ref_has "$s" || REF_DOC="${REF_DOC}${s}	cuo/ship-tasks.md
"
done < <(grep -Eo 'skill: *[a-z0-9-]+' "$chain_doc" | awk '{print $2}' | sort -u)

for cmd in "$payload"/plugin/commands/*.md; do
  [ -f "$cmd" ] || continue
  base="$(basename "$cmd")"
  while read -r s; do
    [ -n "$s" ] || continue
    ref_has "$s" || REF_DOC="${REF_DOC}${s}	plugin/commands/${base}
"
  done < <(grep -Eo '`[a-z0-9]+(-[a-z0-9]+)*-(author|audit)`' "$cmd" | tr -d '`' | sort -u)
done

REF_DOC="$(printf '%s' "$REF_DOC" | sed '/^$/d')"
ref_count="$(printf '%s\n' "$REF_DOC" | sed '/^$/d' | wc -l | tr -d ' ')"
[ "$ref_count" -gt 0 ] || err "0 skill references extracted from $chain_doc - doc structure changed under the checker"

fail=0
# --- MISSING rule: every referenced skill vendored in BOTH trees (unless allowlisted) ---
while IFS=$'\t' read -r s doc; do
  [ -n "$s" ] || continue
  allowed "$s" && continue
  if [ ! -f "$payload/cuo/skills/$s/SKILL.md" ] || [ ! -f "$payload/plugin/skills/$s/SKILL.md" ]; then
    echo "MISSING $s (referenced by $doc)"; fail=1
  fi
done <<EOF
$REF_DOC
EOF

# --- UNPAIRED rule: vendored -author/-audit twins ship together (unless allowlisted) ---
vendored=0
for d in "$payload"/cuo/skills/*/; do
  [ -d "$d" ] || continue
  s="$(basename "$d")"; vendored=$((vendored+1))
  allowed "$s" && continue
  case "$s" in
    *-author) twin="${s%-author}-audit" ;;
    *-audit)  twin="${s%-audit}-author" ;;
    *) continue ;;
  esac
  [ -d "$payload/cuo/skills/$twin" ] || { echo "UNPAIRED $s"; fail=1; }
done

# --- rot warning: allowlist entries nothing references and no dir matches ---
if [ -n "$ALLOW" ]; then
  while read -r a; do
    [ -n "$a" ] || continue
    if ! ref_has "$a" && [ ! -d "$payload/cuo/skills/$a" ]; then
      echo "cyberos-install: WARNING: allowlist entry '$a' is unreferenced and unvendored - stale?" >&2
    fi
  done <<EOF
$ALLOW
EOF
fi

allow_count="$(printf '%s\n' "$ALLOW" | sed '/^$/d' | wc -l | tr -d ' ')"
[ "$fail" -eq 0 ] || exit 10
echo "chain OK: $ref_count referenced, $vendored vendored, $allow_count allowlisted"
