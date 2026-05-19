#!/usr/bin/env bash
# tools/cleanup-folder-layout.sh — repo-layout cleanup per the 2026-05-19 audit.
#
# What this does (operator runs on macOS-side; sandbox can't git mv):
#
#   1. runtime/                 → modules/memory/runtime/
#      Rationale: runtime/ is Python operational tooling that operates on
#      .cyberos-memory/ — strictly the memory module's domain.
#
#   2. apps/memory/              → services/memory/desktop/
#      Rationale: apps/memory/ is the Tauri desktop client for the memory
#      service. Co-locating with services/memory/ keeps the build chain
#      (Rust service + Tauri shell) together. Updates FR-MEMORY-104 + tours
#      reference paths.
#
#   3. outputs/portable-fr-prompts.md → tools/portable-fr-prompts.md
#      Rationale: scratch artefact lived in outputs/; tools/ is the right
#      home for one-off operational artefacts. outputs/ folder removed.
#
#   4. WAVE-1-2-CONTINUATION.md → docs/sessions/2026-05-18-wave-1-2.md
#      Rationale: 634-line session log belongs in docs/sessions/ rather
#      than cluttering repo root. New docs/sessions/ folder serves as the
#      home for future session-log archives.
#
# Usage:
#   bash tools/cleanup-folder-layout.sh --dry-run    # preview
#   bash tools/cleanup-folder-layout.sh --apply      # do it
#
# After --apply, the script runs sed across affected files to update path
# references (runtime/ → modules/memory/runtime/, apps/memory/ → services/
# memory/desktop/). Review the diff before committing.

# Note: NOT using `set -e` because we want each step to attempt independently
# even if a prior step hits a recoverable issue (e.g. one folder already moved,
# the next file gitignored, etc.). We still use `set -uo pipefail` for unset-var
# protection + pipe-failure visibility.
set -uo pipefail

MODE="${1:-}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

if [ "$MODE" != "--dry-run" ] && [ "$MODE" != "--apply" ]; then
  echo "Usage: $0 --dry-run | --apply"
  exit 2
fi

# Helper: move a file via git mv when tracked, falling back to plain mv + git add
# for gitignored sources (e.g. outputs/ which is in .gitignore).
do_mv() {
  local src="$1"
  local dst="$2"
  if [ "$MODE" = "--dry-run" ]; then
    if git ls-files --error-unmatch "$src" >/dev/null 2>&1; then
      echo "  would: git mv $src $dst"
    else
      echo "  would: mv $src $dst   (source is gitignored — plain mv)"
    fi
  else
    mkdir -p "$(dirname "$dst")"
    # Try tracked path first; fall back to plain mv if untracked / gitignored.
    if git ls-files --error-unmatch "$src" >/dev/null 2>&1; then
      git mv "$src" "$dst"
      echo "  done: git mv $src → $dst"
    else
      mv "$src" "$dst"
      # Best-effort: try to add the destination (will silently no-op if it's
      # also gitignored at the new location).
      git add "$dst" 2>/dev/null || true
      echo "  done: mv $src → $dst (was gitignored; staged at destination)"
    fi
  fi
}

# Helper: sed-replace across affected files (if --apply).
update_refs() {
  local old="$1"
  local new="$2"
  if [ "$MODE" = "--dry-run" ]; then
    local count
    count=$(grep -rl "$old" --include='*.md' --include='*.py' --include='*.rs' --include='*.toml' --include='*.yml' --include='*.html' . 2>/dev/null | grep -v node_modules | grep -v '.cyberos-memory' | grep -v '.git/' | wc -l | tr -d ' ')
    echo "  would: update $count file(s) referencing '$old' → '$new'"
  else
    local files
    files=$(grep -rl "$old" --include='*.md' --include='*.py' --include='*.rs' --include='*.toml' --include='*.yml' --include='*.html' . 2>/dev/null | grep -v node_modules | grep -v '.cyberos-memory' | grep -v '.git/' || true)
    local count=0
    while IFS= read -r f; do
      [ -z "$f" ] && continue
      sed -i.bak "s|$old|$new|g" "$f"
      rm -f "${f}.bak"
      count=$((count + 1))
    done <<< "$files"
    echo "  done: updated $count file(s) ('$old' → '$new')"
  fi
}

echo "═══════════════════════════════════════════════════════════════"
echo "  CyberOS folder cleanup — $MODE"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# ── 1. runtime/ → modules/memory/runtime/ ──────────────────────────────────────
echo "Step 1: runtime/ → modules/memory/runtime/"
if [ -d runtime ]; then
  if [ "$MODE" = "--apply" ]; then
    mkdir -p modules/memory
    git mv runtime modules/memory/runtime
    echo "  done: runtime/ → modules/memory/runtime/"
  else
    echo "  would: git mv runtime modules/memory/runtime"
  fi
  update_refs "\bruntime/" "modules/memory/runtime/"
  # The grep would also have matched bare `runtime/` mentions — be more careful in --apply.
  # If sed misfires, the .bak files we removed already are gone; use --dry-run first!
else
  echo "  SKIP — runtime/ does not exist at repo root (already moved?)"
fi
echo ""

# ── 2. apps/memory/ → services/memory/desktop/ ───────────────────────────────────
echo "Step 2: apps/memory/ → services/memory/desktop/"
if [ -d apps/memory ]; then
  if [ "$MODE" = "--apply" ]; then
    mkdir -p services/memory
    git mv apps/memory services/memory/desktop
    rmdir apps 2>/dev/null || true
    echo "  done: apps/memory/ → services/memory/desktop/ (apps/ removed if empty)"
  else
    echo "  would: git mv apps/memory services/memory/desktop && rmdir apps"
  fi
  update_refs "apps/memory/" "services/memory/desktop/"
else
  echo "  SKIP — apps/memory/ does not exist (already moved?)"
fi
echo ""

# ── 3. outputs/portable-fr-prompts.md → tools/ ─────────────────────────────────
echo "Step 3: outputs/portable-fr-prompts.md → tools/"
if [ -f outputs/portable-fr-prompts.md ]; then
  do_mv outputs/portable-fr-prompts.md tools/portable-fr-prompts.md
  if [ "$MODE" = "--apply" ]; then
    rmdir outputs 2>/dev/null || true
    echo "  done: outputs/ removed if empty"
  fi
else
  echo "  SKIP — outputs/portable-fr-prompts.md does not exist (already moved?)"
fi
echo ""

# ── 4. WAVE-1-2-CONTINUATION.md → docs/sessions/ ───────────────────────────────
echo "Step 4: WAVE-1-2-CONTINUATION.md → docs/sessions/2026-05-18-wave-1-2.md"
if [ -f WAVE-1-2-CONTINUATION.md ]; then
  do_mv WAVE-1-2-CONTINUATION.md docs/sessions/2026-05-18-wave-1-2.md
else
  echo "  SKIP — WAVE-1-2-CONTINUATION.md does not exist (already moved?)"
fi
echo ""

# ── 5. Add services/target/ to .gitignore if missing ───────────────────────────
echo "Step 5: ensure services/target/ in .gitignore"
if ! grep -q 'services/target' .gitignore 2>/dev/null; then
  if [ "$MODE" = "--apply" ]; then
    echo "" >> .gitignore
    echo "# Cargo build artefacts (per repo-layout-doctrine in README — services/ is a workspace)" >> .gitignore
    echo "services/target/" >> .gitignore
    echo "  done: appended services/target/ to .gitignore"
  else
    echo "  would: append 'services/target/' to .gitignore"
  fi
else
  echo "  SKIP — already in .gitignore"
fi
echo ""

# ── 6. Update README repo-layout map ───────────────────────────────────────────
if [ "$MODE" = "--apply" ]; then
  echo "Step 6: update root README.md repo-layout block"
  # Replace 'runtime/' line in the layout map
  sed -i.bak 's|├── runtime/                ← Rust runtime artefacts (separate from skill host)|├── docs/sessions/         ← per-session log archives (new — see docs/sessions/2026-05-18-wave-1-2.md)|' README.md
  rm -f README.md.bak
  echo "  done: README.md repo-layout map updated"
else
  echo "Step 6: would update README.md repo-layout map (runtime/ entry replaced)"
fi
echo ""

echo "═══════════════════════════════════════════════════════════════"
echo "  Cleanup $MODE complete."
echo ""
echo "Next steps after --apply:"
echo "  1. git status                 # review the staged moves"
echo "  2. git diff --stat            # see scope of path-reference updates"
echo "  3. cd services && cargo build # smoke-check no Rust paths broken"
echo "  4. cd modules/memory && pytest # smoke-check Python paths"
echo "  5. git commit -m 'refactor(repo): folder-layout cleanup per 2026-05-19 audit'"
echo "═══════════════════════════════════════════════════════════════"
