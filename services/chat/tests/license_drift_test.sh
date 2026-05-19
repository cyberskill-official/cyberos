#!/usr/bin/env bash
# FR-CHAT-001 §4 #4 — drift watcher detects synthetic license commit.
#
# Strategy:
#   - Stage a temporary "mock commits" file with one fake SHA.
#   - Stage a temporary directory mapping that SHA → a file-list containing
#     "LICENSE.md" (i.e. the kind of commit we want to flag).
#   - Run the drift watcher with MOCK_COMMITS_FILE + MOCK_FILES_FOR_COMMIT
#     env vars; it MUST emit the legal-review-needed warning to stderr
#     and exit 1.
#
# This test does NOT need network access. It exercises the filter logic
# in isolation.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

fail() { echo "::error::$*" >&2; exit 1; }
ok()   { echo "  ✓ $*"; }

echo "Running license-drift detection tests..."

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# ---- Case 1: a commit touching LICENSE.md is flagged. ----------------------

cat > "$TMP/commits.txt" <<EOF
abc1234567890abcdef1234567890abcdef12345
EOF
mkdir -p "$TMP/files"
echo "LICENSE.md" > "$TMP/files/abc1234567890abcdef1234567890abcdef12345"

set +e
GH_TOKEN=fake-token-not-used \
PINNED_COMMIT_FILE="$ROOT/PINNED_COMMIT" \
MOCK_COMMITS_FILE="$TMP/commits.txt" \
MOCK_FILES_FOR_COMMIT="$TMP/files" \
SINCE_ISO="2026-01-01T00:00:00Z" \
bash "$ROOT/scripts/check-license-drift.sh" 2> "$TMP/case1-err.log"
RC=$?
set -e

[[ $RC -eq 1 ]] || fail "Case 1: expected exit 1 (drift detected) but got $RC. stderr: $(cat $TMP/case1-err.log)"
grep -q "legal-review-needed" "$TMP/case1-err.log" || fail "Case 1: expected stderr to mention 'legal-review-needed'. stderr was: $(cat $TMP/case1-err.log)"
ok "License-touching commit triggers legal-review-needed signal"

# ---- Case 2: a commit touching only normal source is NOT flagged. ----------

cat > "$TMP/commits.txt" <<EOF
def5678901234567890abcdef1234567890abcde
EOF
mkdir -p "$TMP/files"
echo "model/user.go" > "$TMP/files/def5678901234567890abcdef1234567890abcde"

set +e
GH_TOKEN=fake-token-not-used \
PINNED_COMMIT_FILE="$ROOT/PINNED_COMMIT" \
MOCK_COMMITS_FILE="$TMP/commits.txt" \
MOCK_FILES_FOR_COMMIT="$TMP/files" \
SINCE_ISO="2026-01-01T00:00:00Z" \
bash "$ROOT/scripts/check-license-drift.sh" 2> "$TMP/case2-err.log"
RC=$?
set -e

[[ $RC -eq 0 ]] || fail "Case 2: expected exit 0 (no drift) but got $RC. stderr: $(cat $TMP/case2-err.log)"
ok "Non-license-touching commit does not trigger drift signal"

# ---- Case 3: an empty commit list returns 0. -------------------------------

: > "$TMP/commits.txt"
set +e
GH_TOKEN=fake-token-not-used \
PINNED_COMMIT_FILE="$ROOT/PINNED_COMMIT" \
MOCK_COMMITS_FILE="$TMP/commits.txt" \
MOCK_FILES_FOR_COMMIT="$TMP/files" \
SINCE_ISO="2026-01-01T00:00:00Z" \
bash "$ROOT/scripts/check-license-drift.sh" 2> "$TMP/case3-err.log"
RC=$?
set -e

[[ $RC -eq 0 ]] || fail "Case 3: empty commit list expected exit 0 but got $RC"
ok "Empty commit list returns clean exit"

# ---- Case 4: COPYING + NOTICE files also flagged. --------------------------

cat > "$TMP/commits.txt" <<EOF
f001234567890abcdef1234567890abcdef12345
EOF
echo -e "COPYING\nNOTICE" > "$TMP/files/f001234567890abcdef1234567890abcdef12345"

set +e
GH_TOKEN=fake-token-not-used \
PINNED_COMMIT_FILE="$ROOT/PINNED_COMMIT" \
MOCK_COMMITS_FILE="$TMP/commits.txt" \
MOCK_FILES_FOR_COMMIT="$TMP/files" \
SINCE_ISO="2026-01-01T00:00:00Z" \
bash "$ROOT/scripts/check-license-drift.sh" 2> "$TMP/case4-err.log"
RC=$?
set -e

[[ $RC -eq 1 ]] || fail "Case 4: COPYING/NOTICE-touching commit expected exit 1 but got $RC"
grep -q "legal-review-needed" "$TMP/case4-err.log" || fail "Case 4: expected legal-review-needed signal"
ok "COPYING / NOTICE file changes also trigger drift signal"

echo "✓ All license-drift detection scenarios pass."
