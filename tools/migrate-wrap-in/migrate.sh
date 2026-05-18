#!/usr/bin/env bash
# tools/migrate-wrap-in/migrate.sh — FR-SKILL-113 mechanical sweep.
#
# Renames legacy frontmatter form `wrap_in: <untrusted_content/>` →
# new form `wrap_in_marker: "untrusted_content"` across all
# modules/skill/**/SKILL.md files (post registry v0.2.5).
#
# Usage:
#   bash migrate.sh --dry-run         # preview only — list files that would change
#   bash migrate.sh --apply           # do it — perl -i edit
#   bash migrate.sh --verify          # post-sweep invariants
#
# Idempotent: re-running --apply after a successful sweep is a no-op (0 files changed).
# Body XML form `<untrusted_content source="...">…</untrusted_content>` is untouched
# (the regex only matches the legacy frontmatter line pattern).

set -euo pipefail

MODE="${1:-}"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SKILLS_DIR="$REPO_ROOT/modules/skill"

case "$MODE" in
  --dry-run)
    echo "DRY RUN — files that would change:"
    matches=$(grep -rln 'wrap_in: <untrusted_content/>' "$SKILLS_DIR" --include='SKILL.md' 2>/dev/null || true)
    if [ -z "$matches" ]; then
      echo "  (none — sweep already applied or no matching files)"
    else
      echo "$matches"
      count=$(echo "$matches" | wc -l | tr -d ' ')
      echo "Total: $count file(s)"
    fi
    ;;
  --apply)
    matches=$(grep -rln 'wrap_in: <untrusted_content/>' "$SKILLS_DIR" --include='SKILL.md' 2>/dev/null || true)
    if [ -z "$matches" ]; then
      echo "Migrated 0 files (nothing to do — sweep already applied)."
      exit 0
    fi
    count=0
    while IFS= read -r f; do
      [ -z "$f" ] && continue
      # Preserve YAML whitespace + indentation.
      # Match: leading whitespace + 'wrap_in:' + optional whitespace + '<untrusted_content/>' + optional trailing spaces/tabs
      # NOTE: use [ \t]* (space/tab only) NOT \s* — \s eats newlines which collapses adjacent lines.
      perl -i -pe 's/^([ \t]*)wrap_in:[ \t]*<untrusted_content\/>[ \t]*$/$1wrap_in_marker: "untrusted_content"/' "$f"
      count=$((count + 1))
    done <<< "$matches"
    echo "Migrated $count file(s)."
    ;;
  --verify)
    # Invariant (a): no residual `wrap_in:\s*<` anywhere in skill SKILL.md files
    residual=$(grep -rn 'wrap_in:[[:space:]]*<' "$SKILLS_DIR" --include='SKILL.md' 2>/dev/null || true)
    if [ -n "$residual" ]; then
      echo "FAIL: residual 'wrap_in: <...>' found:"
      echo "$residual"
      exit 1
    fi
    # Invariant (b): wrap_in_marker present where untrusted_inputs block exists
    # Find files that have `untrusted_inputs:` but no `wrap_in_marker:`
    missing=""
    while IFS= read -r f; do
      if grep -q '^untrusted_inputs:' "$f" 2>/dev/null && ! grep -q 'wrap_in_marker:' "$f" 2>/dev/null; then
        missing="$missing$f\n"
      fi
    done < <(find "$SKILLS_DIR" -name 'SKILL.md' -type f 2>/dev/null)
    if [ -n "$missing" ]; then
      echo "FAIL: files have untrusted_inputs but no wrap_in_marker:"
      printf "%b" "$missing"
      exit 1
    fi
    echo "PASS — sweep verified (zero residual legacy form; all untrusted_inputs blocks have wrap_in_marker)."
    ;;
  *)
    echo "Usage: $0 --dry-run | --apply | --verify"
    echo ""
    echo "  --dry-run   List files that would be changed (no edits)."
    echo "  --apply     Perform the sweep (perl -i -pe; preserves whitespace)."
    echo "  --verify    Post-sweep invariants: zero residual legacy form."
    exit 2
    ;;
esac
