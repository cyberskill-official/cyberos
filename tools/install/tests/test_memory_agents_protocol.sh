#!/usr/bin/env bash
# TASK-IMP-061 AC 6 — build vendors dense Layer-1 protocol, not thin spine.
set -euo pipefail
repo="$(cd "$(dirname "$0")/../../.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# Materialise the same path build.sh uses (working tree ok; we assert content shape).
src="$repo/modules/memory/cyberos/data/AGENTS.md"
thin="$repo/AGENTS.md"
test -f "$src"
test -f "$thin"
grep -q '§19' "$src"
grep -q 'Phase 0 consent' "$src"
# Thin spine must NOT be mistaken for the protocol.
if grep -qE 'Phase 0 consent|personnel-requires-consent' "$thin"; then
  echo "FAIL: root AGENTS.md unexpectedly carries Phase 0 consent protocol" >&2
  exit 1
fi
# build.sh points at the dense source
grep -q 'modules/memory/cyberos/data/AGENTS.md' "$repo/tools/install/build.sh"
grep -q '_git_materialise "modules/memory/cyberos/data/AGENTS.md"' "$repo/tools/install/build.sh"
# Must not vendor root AGENTS into memory/
if grep -E '_git_materialise "AGENTS.md" "\$out/memory/AGENTS.md"' "$repo/tools/install/build.sh"; then
  echo "FAIL: build.sh still vendors thin root AGENTS.md into memory/" >&2
  exit 1
fi
echo "pass=5 fail=0 (memory AGENTS protocol vendor)"
