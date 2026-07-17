#!/usr/bin/env bash
# uninstall.sh — remove the vendored CyberOS machine from a repo (once / on demand).
# Keeps operator work: docs/tasks/, docs/status/, CHANGELOG.md, agent files.
# BRAIN store kept by default (CYBEROS_UNINSTALL_KEEP_BRAIN=0 to drop it).
#
#   bash .cyberos/uninstall.sh [repo]
#   bash <payload>/uninstall.sh [repo]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
target="${1:-$(pwd)}"
# Explicit grouping (TASK-IMP-083). The ungrouped form parsed as ((cd && rev-parse) || cd)
# && pwd - so after a SUCCESSFUL rev-parse the trailing pwd still ran and $root captured
# TWO newline-joined paths. "$root/.cyberos" then never existed, every uninstall on a git
# repo exited "nothing to do", and the hook/gitignore/BRAIN sections were unreachable.
root="$( (cd "$target" 2>/dev/null && git rev-parse --show-toplevel 2>/dev/null) || (cd "$target" && pwd) )"
CY="$root/.cyberos"

echo "cyberos uninstall: target=$root"

# Soft update check is irrelevant when removing — skip

if [ ! -d "$CY" ]; then
  echo "cyberos uninstall: nothing to do (no .cyberos/)"
  exit 0
fi

# 1. pre-commit: strip cyberos blocks / managed hook
# Resolve the EFFECTIVE hooks directory exactly as install.sh step 6b does (TASK-IMP-083):
# git runs hooks from core.hooksPath when set (relative anchors at the repo root, absolute
# used as is), else .git/hooks - so we remove/unappend from where install actually wrote,
# and never touch .git/hooks/pre-commit when hooksPath points elsewhere.
hooks_path="$(git -C "$root" config core.hooksPath 2>/dev/null || true)"
if [ -z "$hooks_path" ]; then
  hooks_dir="$root/.git/hooks"
else
  case "$hooks_path" in
    /*) hooks_dir="$hooks_path" ;;
    *)  hooks_dir="$root/${hooks_path%/}" ;;
  esac
fi
hk="$hooks_dir/pre-commit"

# Do we own this file OUTRIGHT? Exact line-2 test, copied from install.sh step 6b, which
# found and fixed this bug class on the install side. The heuristic it replaces here -
# `head -5 "$hk" | grep -q cyberos-status-hook` - asked "is our marker near the top?",
# not "is this our file?": for a FOREIGN hook shorter than five lines carrying our
# appended block, the block's `>>>` marker lands inside head -5, the heuristic classified
# the file as ours, and rm -f deleted the user's hook WHOLE. Our standalone form always
# carries the managed header on line 2; the appended form is marked `>>>` and belongs to
# whoever owns the lines above it. Line 2 + the `>>>` exclusion separates them exactly,
# at any file length.
_cyberos_owns_hook() {
  [ -f "$1" ] || return 1
  local l2; l2="$(sed -n '2p' "$1" 2>/dev/null)"
  case "$l2" in
    *'>>>'*)                    return 1 ;;   # the APPENDED form — the file is theirs
    '# cyberos-status-hook'*)   return 0 ;;   # our managed standalone header
    *)                          return 1 ;;
  esac
}

if [ -f "$hk" ]; then
  if _cyberos_owns_hook "$hk"; then
    rm -f "$hk"
    echo "  removed managed pre-commit hook"
  elif grep -q "cyberos-status-hook" "$hk" 2>/dev/null; then
    tmp="$hk.cyberos.tmp"
    sed '/# >>> cyberos-status-hook/,/# <<< cyberos-status-hook <<</d' "$hk" > "$tmp" && mv "$tmp" "$hk"
    chmod +x "$hk"
    echo "  stripped cyberos block from pre-commit"
  fi
fi

# 2. managed .gitignore block
gi="$root/.gitignore"
if [ -f "$gi" ] && grep -q 'cyberos' "$gi" 2>/dev/null; then
  tmp="$gi.cyberos.tmp"
  # strip marked block if present
  if grep -q '>>> cyberos' "$gi" 2>/dev/null; then
    sed '/# >>> cyberos/,/# <<< cyberos <<</d' "$gi" > "$tmp" && mv "$tmp" "$gi"
    echo "  removed managed .gitignore block"
  fi
fi

# 2b. shared .agents/skills entries + the /create-tasks pair's .claude/skills counterparts
# (TASK-IMP-094). Removed only when OURS by construction: a symlink whose target is the
# vendored machine (directly, or chained via .claude/skills/<cmd>), or the installer's
# copy-fallback (a dir carrying our .cyberos-owned marker). A dir with only a SKILL.md is
# NOT proof of ownership - the installer's copy is byte-indistinguishable from an operator's,
# so the marker is the copy's equivalent of the symlink's readlink target. Anything else under
# .agents/skills/ is operator work and stays; dirs are pruned only when emptied. The tracked
# rules pointers (.devin/rules/, .windsurf/rules/, .windsurfrules) are agent surface and are
# kept, same as CLAUDE.md and the other pointer files.
for _sc in ship-tasks task-author task-audit; do
  _p="$root/.agents/skills/$_sc"
  if [ -L "$_p" ]; then
    case "$(readlink "$_p" 2>/dev/null)" in
      *".claude/skills/$_sc"|*".cyberos/plugin/skills/$_sc")
        rm -f "$_p"; echo "  removed .agents/skills/$_sc (managed entry)";;
    esac
  elif [ -d "$_p" ] && [ -f "$_p/.cyberos-owned" ]; then
    rm -rf "$_p"; echo "  removed .agents/skills/$_sc (installer copy)"
  elif [ -d "$_p" ] && [ -f "$_p/SKILL.md" ]; then
    # A skill dir we did NOT mark: either an operator's own, or a copy from an install that
    # predates the marker (TASK-IMP-094 PR-review fix). Ambiguous ownership is not a licence
    # to rm -rf - say what we see and leave it. Spec §1.3: never touch operator files.
    echo "  kept .agents/skills/$_sc (unmarked skill dir - not an installer copy we can prove;"
    echo "       remove it by hand if it is a leftover from a pre-marker install)"
  fi
  # the pair under .claude/skills is machine-pointing and new with TASK-IMP-094;
  # .claude/skills/ship-tasks keeps today's leave-in-place behavior (section 6).
  if [ "$_sc" != "ship-tasks" ] && [ -L "$root/.claude/skills/$_sc" ]; then
    case "$(readlink "$root/.claude/skills/$_sc" 2>/dev/null)" in
      *".cyberos/plugin/skills/$_sc") rm -f "$root/.claude/skills/$_sc"; echo "  removed .claude/skills/$_sc (managed entry)";;
    esac
  fi
done
rmdir "$root/.agents/skills" 2>/dev/null || true
rmdir "$root/.agents" 2>/dev/null || true

# 3. BRAIN store
brain="$CY/memory/store"
if [ "${CYBEROS_UNINSTALL_KEEP_BRAIN:-1}" = "1" ] && [ -d "$brain" ]; then
  stash="$(mktemp -d "${TMPDIR:-/tmp}/cyberos-brain.XXXXXX")"
  mv "$brain" "$stash/store"
  echo "  BRAIN stashed at $stash/store (restore under .cyberos/memory/store/ if needed)"
  KEEP_BRAIN_STASH="$stash/store"
else
  KEEP_BRAIN_STASH=""
  echo "  dropping BRAIN store (CYBEROS_UNINSTALL_KEEP_BRAIN=0 or absent)"
fi

# 4. remove machine
# The install lock (TASK-IMP-103) lives inside the machine, so removing $CY removes it.
# But if a lock is held by a LIVE install on this host right now, tearing the tree out from
# under it is how you get a half-removed machine and a very confused operator. Same rule as
# the .cyberos-owned marker above: what we did not create, we do not silently destroy.
_ul="$CY/.install.lock"
if [ -d "$_ul" ]; then
  _ulp=""; _ulh=""
  if [ -r "$_ul/owner" ]; then
    _ulp="$(sed -n 's/^pid=//p'  "$_ul/owner" 2>/dev/null | head -1)"
    _ulh="$(sed -n 's/^host=//p' "$_ul/owner" 2>/dev/null | head -1)"
  fi
  if [ -n "$_ulp" ] && [ "$_ulh" = "$(hostname 2>/dev/null || echo unknown)" ] && kill -0 "$_ulp" 2>/dev/null; then
    echo "cyberos uninstall: an install is running (pid $_ulp holds $_ul). Refusing to remove the machine underneath it." >&2
    exit 1
  fi
  echo "  removing stale install lock (pid ${_ulp:-unknown})"
fi
rm -rf "$CY"
echo "  removed .cyberos/"

# 5. optional restore brain only (minimal rehydrate)
if [ -n "${KEEP_BRAIN_STASH:-}" ] && [ -d "$KEEP_BRAIN_STASH" ]; then
  mkdir -p "$root/.cyberos/memory"
  mv "$KEEP_BRAIN_STASH" "$root/.cyberos/memory/store"
  rmdir "$(dirname "$KEEP_BRAIN_STASH")" 2>/dev/null || true
  echo "  restored BRAIN at .cyberos/memory/store/ (machine removed; re-install to restore workflow)"
fi

# 6. skill symlinks into .cyberos (dangling) — leave dirs; operator cleans
echo "cyberos uninstall: done."
echo "  kept: docs/tasks/, docs/status/, CHANGELOG.md, AGENTS.md / pointer files"
echo "  re-install: bash <payload>/install.sh $root"
