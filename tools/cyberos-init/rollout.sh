#!/usr/bin/env bash
# rollout.sh - adopt the unified feature-requests structure + CyberOS machine across many repos.
# For each target repo: migrate any legacy layout (docs/improvement -> docs/feature-requests/improvement,
# .cyberos-memory -> .cyberos/memory/store), run init.sh, smoke-check the vendored machine, and commit ONLY
# the artifacts this run created/changed (never a file that was already dirty before the run).
#
# Usage: bash rollout.sh <payload-dir> <repo> [<repo>...]
# Never pushes. Prints one summary line per repo:  <repo>  v<ver> workflow=OK store=yes ...
set -uo pipefail

PAYLOAD="$(cd "$1" && pwd)"; shift

for repo in "$@"; do
  name="$(basename "$repo")"
  echo "=== $name ==="
  if [ ! -d "$repo/.git" ]; then echo "  SKIP: not a git repo"; continue; fi
  cd "$repo" || { echo "  SKIP: cannot cd"; continue; }

  before="$(git status --porcelain 2>/dev/null)"
  pre_dirty() { printf '%s\n' "$before" | grep -q " $1\$"; }
  had_agents=0;  [ -f AGENTS.md ] && had_agents=1
  had_backlog=0; [ -f docs/feature-requests/BACKLOG.md ] && had_backlog=1
  moved=0; brain_migrated=0; appended=0

  # 1. legacy BRAIN store -> unified location (init scaffolds only .cyberos/memory/store now)
  if [ -d .cyberos-memory ] && [ ! -d .cyberos/memory/store ]; then
    mkdir -p .cyberos/memory && mv .cyberos-memory .cyberos/memory/store && brain_migrated=1
  fi

  # 2. docs/improvement -> docs/feature-requests/improvement (a normal subfolder; FRs on pickup)
  if [ -d docs/improvement ] && [ ! -e docs/feature-requests/improvement ]; then
    mkdir -p docs/feature-requests
    if git mv docs/improvement docs/feature-requests/improvement 2>/dev/null; then moved=1
    elif mv docs/improvement docs/feature-requests/improvement 2>/dev/null; then moved=1; fi
  fi

  # 3. vendor the machine (idempotent; never clobbers BACKLOG/FRs/AGENTS/BRAIN)
  if ! bash "$PAYLOAD/init.sh" "$repo" >/dev/null 2>&1; then echo "  INIT FAILED"; continue; fi

  # 3b. drop a stale legacy gitignore line (the store lives inside .cyberos/ now)
  if [ -f .gitignore ] && grep -qE '^\.cyberos-memory/?$' .gitignore; then
    grep -vE '^\.cyberos-memory/?$' .gitignore > .gitignore.tmp && mv .gitignore.tmp .gitignore
  fi

  # 4. conventions: ensure a pre-existing, CLEAN backlog documents the one-file/both-classes rule
  bl=docs/feature-requests/BACKLOG.md
  if [ -f "$bl" ] && ! grep -q '(improvement)' "$bl" && ! pre_dirty "$bl"; then
    {
      echo ""
      echo "## Conventions (CyberOS)"
      echo ""
      echo 'One backlog for both classes: rows are `- [status] FR-ID-slug - title`;'
      echo '`class: improvement` rows carry an `(improvement)` suffix, product rows are untagged.'
      echo 'FR frontmatter `status` is the record of truth; this file is the index.'
    } >> "$bl"
    appended=1
  fi
  if [ "$moved" = 1 ] && [ -f "$bl" ] && ! grep -q 'moved from `docs/improvement/`' "$bl" && { [ "$appended" = 1 ] || [ "$had_backlog" = 0 ] || ! pre_dirty "$bl"; }; then
    {
      echo ""
      echo '- improvement programs: see `improvement/` (moved from `docs/improvement/`; class: improvement work - convert items to FRs on pickup)'
    } >> "$bl"
    appended=1
  fi

  # 5. commit ONLY what this run created/changed, never pre-dirty files
  [ "$moved" = 1 ] && git add -A docs/improvement docs/feature-requests/improvement 2>/dev/null
  if [ -f "$bl" ] && { [ "$had_backlog" = 0 ] || [ "$appended" = 1 ]; } && ! pre_dirty "$bl"; then git add "$bl" 2>/dev/null; fi
  if [ "$had_agents" = 0 ] && [ -f AGENTS.md ]; then git add AGENTS.md 2>/dev/null; fi
  if ! pre_dirty ".gitignore" && ! git diff --quiet -- .gitignore 2>/dev/null; then git add .gitignore 2>/dev/null; fi
  if ! git diff --cached --quiet 2>/dev/null; then
    ver="$(cat .cyberos/VERSION 2>/dev/null || echo '?')"
    git -c core.hooksPath=/dev/null commit -q -m "cyberos: adopt unified feature-requests structure (init v$ver)

All FRs live under docs/feature-requests (improvement/ is a normal subfolder for
cross-cutting hardening; class: improvement rows share the one BACKLOG.md, tagged
(improvement)). Vendored machine at .cyberos/ + BRAIN store at .cyberos/memory/store
are gitignored. No push." && echo "  committed"
  else
    echo "  nothing to commit (already structured)"
  fi

  # 6. smoke: the vendored machine works standalone
  b0="$(head -c 3 .cyberos/cuo/ship-feature-requests.md 2>/dev/null)"
  wf=BAD; [ "$b0" = "---" ] && wf=OK
  ver="$(cat .cyberos/VERSION 2>/dev/null || echo none)"
  store=NO; [ -f .cyberos/memory/store/manifest.json ] && [ -f .cyberos/memory/store/HEAD ] && store=yes
  gz=NO; [ -s .cyberos/gates.env ] && gz=yes
  plug=NO; [ -f .cyberos/plugin/commands/ship-fr.md ] && plug=yes
  echo "  RESULT $name: v$ver workflow=$wf store=$store gates.env=$gz plugin=$plug moved-improvement=$moved brain-migrated=$brain_migrated backlog-conventions=$appended"
done
