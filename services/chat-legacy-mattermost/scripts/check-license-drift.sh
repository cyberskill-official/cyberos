#!/usr/bin/env bash
# FR-CHAT-001 §1 #4 — license-drift watcher.
#
# Queries the upstream mattermost-server repository for commits since the
# pinned SHA + filters for any commit that touches a LICENSE file or the
# licensing/ tree. If drift is detected, files a GitHub issue with the
# `legal-review-needed` label and exits non-zero.
#
# Usage:
#   GH_TOKEN=ghp_... bash services/chat/scripts/check-license-drift.sh [since-window]
#
# Args:
#   since-window — optional, defaults to "7 days ago". Useful for catching
#                  up after a missed cron run.
#
# Exit codes:
#   0 — no drift detected; no action taken.
#   1 — drift detected; GitHub issue filed (or attempted).
#   2 — environment misconfigured (missing GH_TOKEN, missing PINNED_COMMIT).
#
# Tests:
#   services/chat/tests/license_drift_test.sh — synthesises a mock commit
#   stream + asserts the script flags it.

set -euo pipefail

# ----- environment guard -----------------------------------------------------
GH_TOKEN="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
if [[ -z "$GH_TOKEN" ]]; then
    echo "::error::GH_TOKEN (or GITHUB_TOKEN) not set" >&2
    exit 2
fi

PINNED_FILE="${PINNED_COMMIT_FILE:-services/chat/PINNED_COMMIT}"
if [[ ! -f "$PINNED_FILE" ]]; then
    echo "::error::PINNED_COMMIT file not found at $PINNED_FILE" >&2
    exit 2
fi

# Strip comments + take first non-empty line; tolerate trailing whitespace.
PINNED=$(grep -v '^[[:space:]]*#' "$PINNED_FILE" | awk 'NF{print $1; exit}')
if [[ ! "$PINNED" =~ ^[0-9a-f]{40}$ ]]; then
    echo "::error::Pinned SHA in $PINNED_FILE is not a 40-char hex string: '$PINNED'" >&2
    exit 2
fi

# ----- params ----------------------------------------------------------------
SINCE_WINDOW="${1:-7 days ago}"
UPSTREAM_REPO="${UPSTREAM_REPO:-mattermost/mattermost-server}"
LABEL_NEW="${LICENSE_LABEL:-legal-review-needed}"
LABEL_AREA="${AREA_LABEL:-chat}"

# date arg is GNU-flavour. macOS users can install coreutils + use gdate, or
# pass the ISO date directly via SINCE_ISO env override.
if [[ -n "${SINCE_ISO:-}" ]]; then
    SINCE="$SINCE_ISO"
else
    SINCE=$(date -u -d "$SINCE_WINDOW" +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || \
            date -u -v -7d +"%Y-%m-%dT%H:%M:%SZ")
fi

# ----- commit-list mode (injectable for tests) -------------------------------
# When MOCK_COMMITS_FILE is set, the script reads SHA list from there instead
# of hitting the GitHub API. Used by the test harness.
if [[ -n "${MOCK_COMMITS_FILE:-}" ]] && [[ -f "$MOCK_COMMITS_FILE" ]]; then
    # awk strips blank lines without erroring on an empty input (unlike `grep -v`
    # which returns exit 1 when nothing matches and trips `set -e` via the
    # variable-assignment substitution on some bash builds).
    COMMITS=$(awk 'NF' "$MOCK_COMMITS_FILE")
else
    # Real-world path: paginate the upstream commits endpoint.
    COMMITS=$(curl -fsSL \
        -H "Authorization: Bearer $GH_TOKEN" \
        -H "Accept: application/vnd.github+json" \
        "https://api.github.com/repos/${UPSTREAM_REPO}/commits?since=${SINCE}&per_page=100" \
        | python3 -c 'import sys, json; [print(c["sha"]) for c in json.load(sys.stdin)]' 2>/dev/null || true)
fi

if [[ -z "$COMMITS" ]]; then
    echo "✓ No new upstream commits since $SINCE; pinned $PINNED unchanged."
    exit 0
fi

# ----- filter for LICENSE-touching commits ----------------------------------
FLAGGED=()
for sha in $COMMITS; do
    if [[ -n "${MOCK_FILES_FOR_COMMIT:-}" ]] && [[ -d "${MOCK_FILES_FOR_COMMIT}" ]]; then
        # Test mode: the mock harness puts a file at MOCK_FILES_FOR_COMMIT/$sha
        # listing one filename per line.
        files=$(cat "${MOCK_FILES_FOR_COMMIT}/$sha" 2>/dev/null || true)
    else
        files=$(curl -fsSL \
            -H "Authorization: Bearer $GH_TOKEN" \
            -H "Accept: application/vnd.github+json" \
            "https://api.github.com/repos/${UPSTREAM_REPO}/commits/${sha}" \
            | python3 -c 'import sys, json; d=json.load(sys.stdin); [print(f["filename"]) for f in d.get("files",[])]' \
            2>/dev/null || true)
    fi

    if echo "$files" | grep -qE '^(LICENSE|LICENSE\.md|licensing/|.*\.LICENSE|COPYING|NOTICE)'; then
        FLAGGED+=("$sha")
    fi
done

if [[ ${#FLAGGED[@]} -eq 0 ]]; then
    echo "✓ No license-affecting commits since pinned $PINNED."
    exit 0
fi

# ----- file a legal-review issue --------------------------------------------
echo "::warning::License-affecting commits detected since pinned $PINNED:"
for s in "${FLAGGED[@]}"; do
    echo "  - https://github.com/${UPSTREAM_REPO}/commit/${s}"
done

BODY=$(cat <<EOF
The chat license-drift watcher detected commits affecting LICENSE files in
\`${UPSTREAM_REPO}\` since the pinned SHA \`${PINNED}\`.

**Pinned commit:** \`${PINNED}\`
**Scan window:** since \`${SINCE}\`
**Flagged commits:**

$(for s in "${FLAGGED[@]}"; do
    echo "- [\`${s:0:12}\`](https://github.com/${UPSTREAM_REPO}/commit/${s})"
done)

**Action required:** legal-team review of each flagged commit. If a commit
genuinely changes the upstream license terms, follow the procedure in
\`services/chat/README.md §2\` for updating \`PINNED_COMMIT\` (which
requires CODEOWNERS approval and a fresh DEC entry if posture changes).

cc @cyberos/legal-team

— filed automatically by services/chat/scripts/check-license-drift.sh
EOF
)

# Skip issue creation in mock mode; tests assert via the stderr signal.
if [[ -n "${MOCK_COMMITS_FILE:-}" ]]; then
    echo "::warning::legal-review-needed (mock mode, issue creation skipped)" >&2
    exit 1
fi

# Real mode: file the GitHub issue. We prefer `gh` if available (concise),
# fall back to a raw API POST otherwise.
TITLE="Upstream license drift since pinned commit ${PINNED:0:12}"
REPO_SLUG="${GITHUB_REPOSITORY:-cyberskill/cyberos}"
if command -v gh >/dev/null 2>&1; then
    gh issue create \
        --repo "$REPO_SLUG" \
        --title "$TITLE" \
        --body "$BODY" \
        --label "${LABEL_NEW},${LABEL_AREA}" \
        >&2
else
    curl -fsSL -X POST \
        -H "Authorization: Bearer $GH_TOKEN" \
        -H "Accept: application/vnd.github+json" \
        "https://api.github.com/repos/${REPO_SLUG}/issues" \
        -d "$(python3 -c "
import json,sys
print(json.dumps({
  'title': sys.argv[1],
  'body':  sys.argv[2],
  'labels': sys.argv[3].split(','),
}))" "$TITLE" "$BODY" "${LABEL_NEW},${LABEL_AREA}")" \
        >&2
fi

exit 1
