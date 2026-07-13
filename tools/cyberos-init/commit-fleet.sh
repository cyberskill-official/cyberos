#!/usr/bin/env bash
# commit-fleet.sh - commit ONLY the CyberOS-owned paths in every repo under the given roots, and
# push to main. A repo's own uncommitted work is never staged: the allowlist below is exactly what
# init + migration write as TRACKED files, and the run ABORTS a repo if anything outside it got
# staged. Repos with no remote are committed and reported as unpushable; repos on a branch other
# than main are committed but never pushed (this script does not decide someone's branching).
# Usage: bash tools/cyberos-init/commit-fleet.sh <msg-file> <root-dir> [<root-dir> ...]
set -uo pipefail
MSG="${1:?usage: commit-fleet.sh <msg-file> <root> [...]}"; shift
[ -f "$MSG" ] || { echo "commit-fleet: no such message file: $MSG"; exit 2; }

# exactly what cyberos init + migrate-frs write as tracked content
OWNED=(docs/status docs/feature-requests AGENTS.md CLAUDE.md GEMINI.md .cursorrules
       .windsurfrules .gitignore CHANGELOG.md .mcp.json .agents)
ALLOW='^(docs/status/|docs/feature-requests/|AGENTS\.md$|CLAUDE\.md$|GEMINI\.md$|\.cursorrules$|\.windsurfrules$|\.gitignore$|CHANGELOG\.md$|\.mcp\.json$|\.agents/)'

SKIP=0; DONE=0; PUSHED=0; FAILED=0

for base in "$@"; do
  for r in "$base"/*; do
    [ -d "$r" ] || continue
    name="$(basename "$base")/$(basename "$r")"
    [ "$(basename "$r")" = "cyberos" ] && continue          # self-host: released on its own
    if [ ! -d "$r/.git" ]; then
      printf 'SKIP  %-46s no git repo\n' "$name"; SKIP=$((SKIP+1)); continue
    fi

    git -C "$r" reset -q                                     # start from a clean index
    for p in "${OWNED[@]}"; do
      [ -e "$r/$p" ] && git -C "$r" add -A -- "$p" 2>/dev/null
    done
    staged="$(git -C "$r" diff --cached --name-only)"
    if [ -z "$staged" ]; then
      printf 'SKIP  %-46s nothing CyberOS-owned to commit\n' "$name"; SKIP=$((SKIP+1)); continue
    fi

    # the guard: refuse to commit anything the allowlist does not cover
    stray="$(grep -Ev "$ALLOW" <<<"$staged" || true)"
    if [ -n "$stray" ]; then
      git -C "$r" reset -q
      printf 'FAIL  %-46s staged a path outside the allowlist: %s\n' "$name" "$(head -1 <<<"$stray")"
      FAILED=$((FAILED+1)); continue
    fi

    n="$(wc -l <<<"$staged" | tr -d ' ')"
    if ! git -C "$r" commit -q -F "$MSG" 2>/dev/null; then
      printf 'FAIL  %-46s commit failed\n' "$name"; FAILED=$((FAILED+1)); continue
    fi
    DONE=$((DONE+1))

    br="$(git -C "$r" branch --show-current)"
    rem="$(git -C "$r" remote | head -1)"
    left="$(git -C "$r" status --porcelain | wc -l | tr -d ' ')"
    if [ -z "$rem" ]; then
      printf 'OK    %-46s committed %-3s paths | NO REMOTE (nothing to push) | %s left dirty\n' "$name" "$n" "$left"
    elif [ "$br" != "main" ]; then
      printf 'OK    %-46s committed %-3s paths | on branch %s, NOT pushed | %s left dirty\n' "$name" "$n" "$br" "$left"
    elif git -C "$r" push -q "$rem" main 2>/dev/null; then
      PUSHED=$((PUSHED+1))
      printf 'OK    %-46s committed %-3s paths | pushed to main | %s left dirty\n' "$name" "$n" "$left"
    else
      printf 'WARN  %-46s committed %-3s paths | PUSH REJECTED (behind/protected?) | %s left dirty\n' "$name" "$n" "$left"
      FAILED=$((FAILED+1))
    fi
  done
done

echo "----"
echo "commit-fleet: committed=$DONE pushed=$PUSHED skipped=$SKIP failed=$FAILED"
[ "$FAILED" -eq 0 ]
