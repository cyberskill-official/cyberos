#!/usr/bin/env bash
# Advisory commit-msg hook: nudge Conventional Commits (so the auto-version workflow can
# classify the change) and print the projected next CyberOS version. It does NOT block by
# default - set CYBEROS_STRICT_COMMITS=1 to fail on a non-conventional subject.
#
# Install the commit-msg hook type once:  pre-commit install --hook-type commit-msg
set -euo pipefail

msg_file="${1:-}"
[ -n "$msg_file" ] && [ -f "$msg_file" ] || exit 0
subject="$(sed -n '1p' "$msg_file")"

# allow: conventional types (optional scope, optional !), merges, reverts, fixups, and the
# bot's own release commits.
conv='^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([^)]+\))?!?: .+'
if printf '%s' "$subject" | grep -Eq "$conv" \
   || printf '%s' "$subject" | grep -Eq '^(Merge |Revert |fixup!|squash!|amend!)'; then
  : # good
else
  echo "cyberos commit-msg: '$subject'" >&2
  echo "  not a Conventional Commit. Use e.g. 'feat(cyberos-install): ...' / 'fix(memory): ...'." >&2
  echo "  Types: feat(minor) fix|perf|refactor|revert(patch) '!' or 'BREAKING CHANGE:'(major); chore/docs/ci/test/build/style don't bump." >&2
  if [ "${CYBEROS_STRICT_COMMITS:-0}" = "1" ]; then echo "  CYBEROS_STRICT_COMMITS=1 -> rejecting." >&2; exit 1; fi
  echo "  (advisory - committing anyway; set CYBEROS_STRICT_COMMITS=1 to enforce.)" >&2
fi

# best-effort: show what version this change would produce (never fails the commit)
root="$(git rev-parse --show-toplevel 2>/dev/null || echo .)"
if command -v node >/dev/null 2>&1 && [ -f "$root/scripts/cyberos-version.mjs" ]; then
  proj="$(node "$root/scripts/cyberos-version.mjs" --json 2>/dev/null || true)"
  next="$(printf '%s' "$proj" | sed -n 's/.*"next":"\([^"]*\)".*/\1/p')"
  cur="$(printf '%s' "$proj" | sed -n 's/.*"current":"\([^"]*\)".*/\1/p')"
  [ -n "$next" ] && [ "$next" != "$cur" ] && echo "cyberos: projected next release version -> $next (currently $cur; tag v$next to release)." >&2
fi
exit 0
