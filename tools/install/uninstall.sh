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

# What THIS RUN removed (TASK-IMP-106 §1.1). Each removal appends here at the moment it happens,
# beside the line that reports it - so a branch that did not run contributes nothing, and the
# summary cannot claim an action the run did not take. Newline-delimited string rather than an
# array on purpose: `set -u` plus an empty array is an error on bash < 4.4, and this script runs
# on whatever bash the operator has (macOS ships 3.2).
_removed_list=""
_note_removed() { _removed_list="${_removed_list}$1
"; }

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
    _note_removed "${hk#$root/} (managed pre-commit hook)"
  elif grep -q "cyberos-status-hook" "$hk" 2>/dev/null; then
    tmp="$hk.cyberos.tmp"
    # Strip the managed block AND the single blank separator install writes immediately BEFORE our
    # `# >>> cyberos-status-hook` marker (install.sh appends the block via a heredoc whose first line
    # is that blank - TASK-IMP-126 §1.3). The old strip deleted only the >>>..<<< range inclusive, so
    # the separator survived and every install/uninstall cycle on a foreign hook accumulated one blank
    # line. awk, not sed, because the fix needs one line of look-behind: hold a pending blank and drop
    # it ONLY when the very next line is our marker - so an operator's own blank line anywhere else in
    # the hook is never touched (§1.4). The start pattern is the shared `# >>> cyberos-status-hook`
    # prefix, exactly as the old strip matched, so a v1 leftover (install.sh:856 upgrade shares the
    # shape) is healed the same as a v2 block.
    awk '
      inblk { if ($0 ~ /# <<< cyberos-status-hook <<</) inblk=0; next }
      /# >>> cyberos-status-hook/ { inblk=1; held=0; next }
      /^[[:space:]]*$/ { if (held) print sep; sep=$0; held=1; next }
      { if (held) { print sep; held=0 } print }
      END { if (held) print sep }
    ' "$hk" > "$tmp" && mv "$tmp" "$hk"
    chmod +x "$hk"
    echo "  stripped cyberos block from pre-commit"
    _note_removed "${hk#$root/} (our block only - the hook itself is yours and stays)"
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
    _note_removed ".gitignore (managed block only - your rules stay)"
  fi
fi

# 2b. managed skill entries install writes - the shared .agents/skills entries (TASK-IMP-094) in the
# loop here, then every per-agent family link in the loop below (extended by TASK-IMP-126 §1.2).
# Removed only when OURS by construction: a symlink whose target is the
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
        rm -f "$_p"; echo "  removed .agents/skills/$_sc (managed entry)"
        _note_removed ".agents/skills/$_sc (managed entry)";;
    esac
  elif [ -d "$_p" ] && [ -f "$_p/.cyberos-owned" ]; then
    rm -rf "$_p"; echo "  removed .agents/skills/$_sc (installer copy)"
    _note_removed ".agents/skills/$_sc (installer copy)"
  elif [ -d "$_p" ] && [ -f "$_p/SKILL.md" ]; then
    # A skill dir we did NOT mark: either an operator's own, or a copy from an install that
    # predates the marker (TASK-IMP-094 PR-review fix). Ambiguous ownership is not a licence
    # to rm -rf - say what we see and leave it. Spec §1.3: never touch operator files.
    echo "  kept .agents/skills/$_sc (unmarked skill dir - not an installer copy we can prove;"
    echo "       remove it by hand if it is a leftover from a pre-marker install)"
  fi
done
rmdir "$root/.agents/skills" 2>/dev/null || true
rmdir "$root/.agents" 2>/dev/null || true

# Then the per-agent native skill entries install writes (install.sh install_skill calls, ~L632-641):
# claude-code gets ship-tasks + the /create-tasks pair (task-author, task-audit); grok, command-code,
# codex and opencode each get ship-tasks. Each is a relative symlink into .cyberos/plugin/skills/<skill>
# (a copy-fallback lands only where a link cannot be made). Before TASK-IMP-126, uninstall touched only
# the .claude/skills create-tasks pair and LEFT .claude/skills/ship-tasks plus the grok/command-code/
# codex/opencode entries pointing into the machine it was about to remove - every one a dangling link
# (§1.2). The family/skill list below mirrors install.sh exactly; do not invent names here.
for _fs in ".claude/skills:ship-tasks" ".claude/skills:task-author" ".claude/skills:task-audit" \
           ".grok/skills:ship-tasks" ".commandcode/skills:ship-tasks" \
           ".codex/skills:ship-tasks" ".opencode/skill:ship-tasks"; do
  _sd="${_fs%%:*}"; _sk="${_fs##*:}"; _p="$root/$_sd/$_sk"
  if [ -L "$_p" ]; then
    case "$(readlink "$_p" 2>/dev/null)" in
      *".cyberos/plugin/skills/$_sk")
        rm -f "$_p"; echo "  removed $_sd/$_sk (managed skill link)"
        _note_removed "$_sd/$_sk (managed skill link)";;
    esac
  elif [ -d "$_p" ] && [ -f "$_p/.cyberos-owned" ]; then
    rm -rf "$_p"; echo "  removed $_sd/$_sk (installer skill copy)"
    _note_removed "$_sd/$_sk (installer skill copy)"
  fi
  rmdir "$root/$_sd" 2>/dev/null || true
done

# 2c. MCP registration files (TASK-IMP-126 §1.1). install.sh writes .mcp.json (always) and, when
# the cursor agent was selected, .cursor/mcp.json - each a FIXED JSON string emitted by mcp_json(),
# pointing at .cyberos/mcp/cyberos-mcp.mjs, the entry point this run is about to remove. Remove each
# ONLY when its content byte-matches what install writes (recreate the string, cmp -s) - the same
# ownership-by-construction the skill and hook sections use. An operator's own file of either name,
# any other content, STAYS (§1.4). Absent files are not an error (§3: the cursor file exists only
# when the cursor agent was selected at install). Never rmdir .cursor/: its rules/ pointer is tracked
# agent surface and is kept.
mcp_json() { printf '{\n  "mcpServers": {\n    "cyberos": { "command": "node", "args": [".cyberos/mcp/cyberos-mcp.mjs"] }\n  }\n}\n'; }
_mcp_ours="$(mktemp "${TMPDIR:-/tmp}/cyberos-mcp.XXXXXX")"
mcp_json > "$_mcp_ours"
for _mf in .mcp.json .cursor/mcp.json; do
  _mp="$root/$_mf"
  if [ -f "$_mp" ] && cmp -s "$_mp" "$_mcp_ours"; then
    rm -f "$_mp"; echo "  removed $_mf (cyberos MCP registration)"
    _note_removed "$_mf (cyberos MCP registration)"
  fi
done
rm -f "$_mcp_ours"

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
_note_removed ".cyberos/ (the vendored machine: workflows, skills, plugin, docs-tools)"

# 5. optional restore brain only (minimal rehydrate)
if [ -n "${KEEP_BRAIN_STASH:-}" ] && [ -d "$KEEP_BRAIN_STASH" ]; then
  mkdir -p "$root/.cyberos/memory"
  mv "$KEEP_BRAIN_STASH" "$root/.cyberos/memory/store"
  rmdir "$(dirname "$KEEP_BRAIN_STASH")" 2>/dev/null || true
  echo "  restored BRAIN at .cyberos/memory/store/ (machine removed; re-install to restore workflow)"
fi

# 6. summary — what went, what stayed and why, and how to finish the job by hand (TASK-IMP-106).
#
# This block REPORTS; it MUST NOT mutate. Everything above has already run, so a bug here can
# never leave a half-removed machine — that is why the summary is last and why nothing follows it.
#
# The kept list is DERIVED from what is on disk right now, never recited from memory (§1.4). The
# line this replaced read `kept: docs/tasks/, docs/status/, CHANGELOG.md, AGENTS.md / pointer
# files` unconditionally — so it claimed docs/status/ on repos that had never rendered a page, and
# told the operator their corpus was safe without ever looking for it. A summary that is right by
# luck is wrong; it drifts the moment the state machine changes and nothing notices, because a
# hard-coded list has no way to be wrong out loud.
#
# Managed skill links into .cyberos/plugin/skills are removed above (§1.2, every family install
# writes); an unmarked operator skill dir is left in place and is the operator's to clean.
echo "cyberos uninstall: done."

if [ -n "$_removed_list" ]; then
  echo "  removed:"
  # `|| continue`, not `&& echo`: an && list whose left side fails leaves a non-zero status as
  # the loop body's last command, and `set -e` would abort the script HERE - after the machine
  # is already gone. The summary must never be able to fail the run it is reporting on.
  printf '%s' "$_removed_list" | while IFS= read -r _r; do
    [ -n "$_r" ] || continue
    echo "    $_r"
  done
fi

# Kept paths: a present-tense probe of the tree we just finished editing. The four NAMES are
# string literals here — the probe decides only WHETHER a literal prints, never what prints, so
# nothing from the tree or from "$1" can reach the rm line below. Directories are probed with a
# trailing slash: `[ -e docs/tasks/ ]` is false for a regular FILE of that name, so a stray file
# is never announced as the operator's corpus.
_kept_lines=""
_kept_paths=""
_keep() {   # $1 = path (as displayed, probed, and removed)   $2 = the one-line reason
  [ -e "$root/$1" ] || return 0
  _kept_lines="${_kept_lines}$(printf '    %-16s %s' "$1" "$2")
"
  _kept_paths="${_kept_paths:+$_kept_paths }$1"
  return 0
}
_keep "docs/tasks/"     "your task corpus - specs, audits, the backlog"
_keep "docs/status/"    "the rendered status page for that corpus"
_keep "CHANGELOG.md"    "your release history"
_keep ".cyberos/memory" "the BRAIN store - everything the machine remembered for you"

if [ -n "$_kept_paths" ]; then
  echo "  kept on purpose - this is your work, not the machine's, so uninstall never removes it:"
  printf '%s' "$_kept_lines"
  echo "  to remove the kept material yourself, run this from $root:"
  echo "    rm -rf $_kept_paths"
fi
echo "  re-install: bash <payload>/install.sh $root"
