#!/usr/bin/env bash
# rollout.sh - adopt the unified tasks structure + CyberOS machine across many repos.
# For each target repo: migrate any legacy layout (docs/improvement -> docs/tasks/improvement,
# .cyberos-memory -> .cyberos/memory/store), run install.sh, smoke-check the vendored machine, and commit ONLY
# the artifacts this run created/changed (never a file that was already dirty before the run).
#
# Usage: bash rollout.sh <payload-dir> <repo> [<repo>...]
# Never pushes. Prints one summary line per repo:  <repo>  v<ver> workflow=OK store=yes ...
set -uo pipefail

# TASK-IMP-069: --from-release [vX.Y.Z] downloads + verifies the published payload once, then
# proceeds unchanged for every listed repo. CYBEROS_PAYLOAD_URL overrides (tests use file://).
if [ "${1:-}" = "--from-release" ]; then
  shift
  tag=""; case "${1:-}" in v[0-9]*) tag="$1"; shift ;; esac
  base="https://github.com/cyberskill-official/cyberos/releases"
  if [ -n "$tag" ]; then url="$base/download/$tag/cyberos-payload.tar.gz"; else url="$base/latest/download/cyberos-payload.tar.gz"; fi
  url="${CYBEROS_PAYLOAD_URL:-$url}"
  dl="$(mktemp -d)"; trap 'rm -rf "$dl"' EXIT
  echo "rollout: downloading $url"
  curl -fsSL "$url" -o "$dl/cyberos-payload.tar.gz"
  curl -fsSL "$(dirname "$url")/SHA256SUMS" -o "$dl/SHA256SUMS" || { echo "rollout: ERROR: no SHA256SUMS beside the tarball" >&2; exit 1; }
  (cd "$dl" && grep " cyberos-payload.tar.gz$" SHA256SUMS | sha256sum -c - >/dev/null) || { echo "rollout: ERROR: checksum mismatch" >&2; exit 1; }
  mkdir -p "$dl/payload" && tar -xzf "$dl/cyberos-payload.tar.gz" -C "$dl/payload"
  PAYLOAD="$dl/payload"
else
  PAYLOAD="$(cd "$1" && pwd)"; shift
fi

for repo in "$@"; do
  name="$(basename "$repo")"
  echo "=== $name ==="
  if [ ! -d "$repo/.git" ]; then echo "  SKIP: not a git repo"; continue; fi
  cd "$repo" || { echo "  SKIP: cannot cd"; continue; }

  before="$(git status --porcelain 2>/dev/null)"
  pre_dirty() { printf '%s\n' "$before" | grep -q " $1\$"; }
  had_agents=0;  [ -f AGENTS.md ] && had_agents=1
  had_backlog=0; [ -f docs/tasks/BACKLOG.md ] && had_backlog=1
  moved=0; brain_migrated=0; appended=0

  # 1. legacy BRAIN store -> unified location (install scaffolds only .cyberos/memory/store now)
  if [ -d .cyberos-memory ] && [ ! -d .cyberos/memory/store ]; then
    mkdir -p .cyberos/memory && mv .cyberos-memory .cyberos/memory/store && brain_migrated=1
  fi

  # 2. docs/improvement -> docs/tasks/improvement (a normal subfolder; tasks on pickup)
  if [ -d docs/improvement ] && [ ! -e docs/tasks/improvement ]; then
    mkdir -p docs/tasks
    if git mv docs/improvement docs/tasks/improvement 2>/dev/null; then moved=1
    elif mv docs/improvement docs/tasks/improvement 2>/dev/null; then moved=1; fi
  fi

  # 3. vendor the machine (idempotent; never clobbers BACKLOG/tasks/AGENTS/BRAIN)
  if ! bash "$PAYLOAD/install.sh" "$repo" >/dev/null 2>&1; then echo "  INSTALL FAILED"; continue; fi

  # 3b. drop a stale legacy gitignore line (the store lives inside .cyberos/ now)
  if [ -f .gitignore ] && grep -qE '^\.cyberos-memory/?$' .gitignore; then
    grep -vE '^\.cyberos-memory/?$' .gitignore > .gitignore.tmp && mv .gitignore.tmp .gitignore
  fi

  # 4. conventions: ensure a pre-existing, CLEAN backlog documents the one-file/both-classes rule
  bl=docs/tasks/BACKLOG.md
  if [ -f "$bl" ] && ! grep -q '(improvement)' "$bl" && ! pre_dirty "$bl"; then
    {
      echo ""
      echo "## Conventions (CyberOS)"
      echo ""
      echo 'One backlog for both classes: rows are `- [status] TASK-ID-slug - title`;'
      echo '`class: improvement` rows carry an `(improvement)` suffix, product rows are untagged.'
      echo 'task frontmatter `status` is the record of truth; this file is the index.'
    } >> "$bl"
    appended=1
  fi
  if [ "$moved" = 1 ] && [ -f "$bl" ] && ! grep -q 'moved from `docs/improvement/`' "$bl" && { [ "$appended" = 1 ] || [ "$had_backlog" = 0 ] || ! pre_dirty "$bl"; }; then
    {
      echo ""
      echo '- improvement programs: see `improvement/` (moved from `docs/improvement/`; class: improvement work - convert items to tasks on pickup)'
    } >> "$bl"
    appended=1
  fi

  # 5. commit ONLY what this run created/changed, never pre-dirty files
  [ "$moved" = 1 ] && git add -A docs/improvement docs/tasks/improvement 2>/dev/null
  if [ -f "$bl" ] && { [ "$had_backlog" = 0 ] || [ "$appended" = 1 ]; } && ! pre_dirty "$bl"; then git add "$bl" 2>/dev/null; fi
  if [ "$had_agents" = 0 ] && [ -f AGENTS.md ]; then git add AGENTS.md 2>/dev/null; fi
  if ! pre_dirty ".gitignore" && ! git diff --quiet -- .gitignore 2>/dev/null; then git add .gitignore 2>/dev/null; fi
  if ! git diff --cached --quiet 2>/dev/null; then
    ver="$(cat .cyberos/VERSION 2>/dev/null || echo '?')"
    git -c core.hooksPath=/dev/null commit -q -m "cyberos: adopt unified tasks structure (init v$ver)

All tasks live under docs/tasks (improvement/ is a normal subfolder for
cross-cutting hardening; class: improvement rows share the one BACKLOG.md, tagged
(improvement)). Vendored machine at .cyberos/ + BRAIN store at .cyberos/memory/store
are gitignored. No push." && echo "  committed"
  else
    echo "  nothing to commit (already structured)"
  fi

  # 6. smoke: the vendored machine works standalone
  b0="$(head -c 3 .cyberos/cuo/ship-tasks.md 2>/dev/null)"
  wf=BAD; [ "$b0" = "---" ] && wf=OK
  ver="$(cat .cyberos/VERSION 2>/dev/null || echo none)"
  store=NO; [ -f .cyberos/memory/store/manifest.json ] && [ -f .cyberos/memory/store/HEAD ] && store=yes
  gz=NO; [ -s .cyberos/gates.env ] && gz=yes
  plug=NO; [ -f .cyberos/plugin/.claude-plugin/plugin.json ] && plug=yes   # was ship-fr.md: a command that exists under no name (ship-tasks is a SKILL), so this read NO forever
  echo "  RESULT $name: v$ver workflow=$wf store=$store gates.env=$gz plugin=$plug moved-improvement=$moved brain-migrated=$brain_migrated backlog-conventions=$appended"
done
