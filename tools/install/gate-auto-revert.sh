#!/usr/bin/env bash
# gate-auto-revert.sh — opt-in revert-PR helper on gate regression (TASK-IMP-026).
#
# NEVER force-pushes. NEVER merges. Opens a revert PR via `gh` when live;
# `--dry-run` prints the plan without mutating remotes.
#
# Usage:
#   bash tools/install/gate-auto-revert.sh --dry-run [--goldenset-case <id>] \
#     [--failure-json <path>] <bad-sha> [base-branch]
#   CYBEROS_AUTO_REVERT=1 bash tools/install/gate-auto-revert.sh <bad-sha> [base-branch]
#
# Exit: 0 plan/PR ok; 2 usage / missing opt-in / bad sha; 1 gh/git failure
set -uo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root" || exit 2

DRY=0
GOLDENSET_CASE=""
FAILURE_JSON=""
BAD_SHA=""
BASE_BRANCH="main"

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run) DRY=1; shift ;;
    --goldenset-case) GOLDENSET_CASE="${2:-}"; shift 2 ;;
    --failure-json) FAILURE_JSON="${2:-}"; shift 2 ;;
    --help|-h)
      sed -n '2,14p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    -*)
      echo "gate-auto-revert: unknown flag $1" >&2
      exit 2
      ;;
    *)
      if [ -z "$BAD_SHA" ]; then BAD_SHA="$1"
      else BASE_BRANCH="$1"
      fi
      shift
      ;;
  esac
done

if [ -z "$BAD_SHA" ]; then
  echo "gate-auto-revert: usage: gate-auto-revert.sh [--dry-run] <bad-sha> [base-branch]" >&2
  exit 2
fi

if [ "$DRY" -ne 1 ] && [ "${CYBEROS_AUTO_REVERT:-}" != "1" ]; then
  echo "gate-auto-revert: refused — set CYBEROS_AUTO_REVERT=1 for live mode, or pass --dry-run" >&2
  echo "  (never force-pushes; never merges; opens a revert PR for an operator to review)" >&2
  exit 2
fi

if ! git cat-file -e "${BAD_SHA}^{commit}" 2>/dev/null; then
  echo "gate-auto-revert: not a commit: $BAD_SHA" >&2
  exit 2
fi

SHORT="$(git rev-parse --short "$BAD_SHA")"
FULL="$(git rev-parse "$BAD_SHA")"
BRANCH="revert/${SHORT}"
SUBJECT="$(git log -1 --format=%s "$FULL")"
TITLE="revert: ${SHORT} — ${SUBJECT}"

BODY_FILE="$(mktemp)"
{
  echo "## Summary"
  echo
  echo "Proposed revert of \`${FULL}\` (\`${SUBJECT}\`) after a gate regression."
  echo
  echo "This PR was opened by \`tools/install/gate-auto-revert.sh\` (TASK-IMP-026)."
  echo "It does **not** force-push and does **not** merge — an operator must review."
  echo
  if [ -n "$GOLDENSET_CASE" ]; then
    echo "## Goldenset case"
    echo
    echo "- Failing case: \`${GOLDENSET_CASE}\`"
    echo "- Goldenset: \`tools/install/.awh/goldenset.yaml\` (TASK-IMP-008)"
    echo
  fi
  if [ -n "$FAILURE_JSON" ] && [ -f "$FAILURE_JSON" ]; then
    echo "## Gate failure artifact"
    echo
    echo '```json'
    cat "$FAILURE_JSON"
    echo
    echo '```'
    echo
  elif [ -f "$root/.cyberos/last-gate-failure.json" ]; then
    echo "## Gate failure artifact"
    echo
    echo '```json'
    cat "$root/.cyberos/last-gate-failure.json"
    echo
    echo '```'
    echo
  fi
  echo "## Operator checklist"
  echo
  echo "- [ ] Confirm the bad SHA is the regression"
  echo "- [ ] Review the revert diff"
  echo "- [ ] Merge manually if accepted (script will never merge)"
} > "$BODY_FILE"

echo "gate-auto-revert: bad_sha=$FULL"
echo "gate-auto-revert: branch=$BRANCH (from $BASE_BRANCH)"
echo "gate-auto-revert: title=$TITLE"
echo "gate-auto-revert: plan=git revert --no-edit $FULL && git push -u origin $BRANCH && gh pr create"

if [ "$DRY" -eq 1 ]; then
  echo "---- dry-run PR body ----"
  cat "$BODY_FILE"
  echo "---- end dry-run ----"
  rm -f "$BODY_FILE"
  exit 0
fi

# Live mode — still never force-push, never merge.
git fetch origin "$BASE_BRANCH" 2>/dev/null || true
if ! git show-ref --verify --quiet "refs/heads/$BASE_BRANCH" \
  && ! git show-ref --verify --quiet "refs/remotes/origin/$BASE_BRANCH"; then
  echo "gate-auto-revert: base branch not found: $BASE_BRANCH" >&2
  rm -f "$BODY_FILE"
  exit 2
fi

git checkout -B "$BRANCH" "origin/${BASE_BRANCH}" 2>/dev/null \
  || git checkout -B "$BRANCH" "$BASE_BRANCH" || {
  echo "gate-auto-revert: failed to create branch $BRANCH" >&2
  rm -f "$BODY_FILE"
  exit 1
}

if ! git revert --no-edit "$FULL"; then
  echo "gate-auto-revert: git revert failed (conflicts?); aborting, leaving branch for operator" >&2
  rm -f "$BODY_FILE"
  exit 1
fi

# Fast-forward push only — no --force, no -f.
if ! git push -u origin "HEAD:refs/heads/${BRANCH}"; then
  echo "gate-auto-revert: push failed" >&2
  rm -f "$BODY_FILE"
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "gate-auto-revert: gh not installed; branch pushed — open PR manually" >&2
  rm -f "$BODY_FILE"
  exit 1
fi

gh pr create --base "$BASE_BRANCH" --head "$BRANCH" --title "$TITLE" --body-file "$BODY_FILE"
rc=$?
rm -f "$BODY_FILE"
exit "$rc"
