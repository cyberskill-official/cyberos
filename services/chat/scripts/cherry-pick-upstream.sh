#!/usr/bin/env bash
# FR-CHAT-001 §1 #5 — operator workflow for cherry-picking upstream security
# fixes into the CyberOS patch series.
#
# Per DEC-422 we DO NOT rebase from upstream master. Each upstream commit
# we want is extracted as a patch via `git format-patch`, dropped into
# services/chat/patches/, and merged through a PR that the legal-team must
# label `legal-reviewed` before CI lets it merge.
#
# Usage:
#   bash services/chat/scripts/cherry-pick-upstream.sh <upstream-sha> [patch-name]
#
# Example:
#   bash services/chat/scripts/cherry-pick-upstream.sh abc123def456 cve-2026-12345
#
# Side effects:
#   - Clones upstream into /tmp/cyberos-chat-cherry-pick-XXXX (shallow).
#   - Writes services/chat/patches/NNN-<patch-name>.patch.
#   - Updates CHANGELOG.cyberos.md under `## [unreleased]`.
#   - Opens a PR via `gh pr create` (if gh available + GH_TOKEN set).
#
# Required labels for merge: `legal-reviewed` + `security` (auto-applied).

set -euo pipefail

UPSTREAM_SHA="${1:-}"
PATCH_NAME="${2:-}"

if [[ -z "$UPSTREAM_SHA" ]]; then
    cat >&2 <<EOF
Usage: $0 <upstream-sha> [patch-name]

Required: 40-char SHA of the upstream commit to cherry-pick.
Optional: human-readable patch name (e.g. "cve-2026-12345"); defaults to the SHA.
EOF
    exit 2
fi

if [[ ! "$UPSTREAM_SHA" =~ ^[0-9a-f]{7,40}$ ]]; then
    echo "::error::Argument is not a valid git SHA: $UPSTREAM_SHA" >&2
    exit 2
fi

PATCH_NAME="${PATCH_NAME:-${UPSTREAM_SHA:0:12}}"
UPSTREAM_REPO="${UPSTREAM_REPO:-mattermost/mattermost-server}"
UPSTREAM_URL="${UPSTREAM_URL:-https://github.com/${UPSTREAM_REPO}.git}"

# Always operate from the cyberos repo root so the patches/ path resolves
# correctly regardless of where the operator invokes the script.
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

PATCHES_DIR="services/chat/patches"
mkdir -p "$PATCHES_DIR"

# Pick the next patch number — sort existing NNN-*.patch files and increment.
LAST_NUM=$(ls -1 "$PATCHES_DIR" 2>/dev/null \
    | grep -E '^[0-9]{3}-' \
    | sort -n \
    | tail -1 \
    | sed -E 's/^([0-9]{3})-.*$/\1/' \
    || echo "000")
LAST_NUM=${LAST_NUM:-000}
NEXT_NUM=$(printf "%03d" "$((10#$LAST_NUM + 1))")
PATCH_FILE="${PATCHES_DIR}/${NEXT_NUM}-${PATCH_NAME}.patch"

# Clone shallowly, fetch the specific commit's tree, and emit format-patch.
WORK=$(mktemp -d /tmp/cyberos-chat-cherry-pick.XXXXXXXX)
trap 'rm -rf "$WORK"' EXIT
git clone --no-checkout --filter=blob:none "$UPSTREAM_URL" "$WORK/upstream" >/dev/null 2>&1
cd "$WORK/upstream"
git fetch --depth=2 origin "$UPSTREAM_SHA" >/dev/null
git format-patch -1 --stdout "$UPSTREAM_SHA" > "$REPO_ROOT/$PATCH_FILE"
cd "$REPO_ROOT"

# Try to extract a CVE number from the commit message for the PR title.
COMMIT_MSG_FIRST_LINE=$(head -50 "$PATCH_FILE" | sed -n 's/^Subject: \[PATCH\] //p' | head -1)
CVE_HINT=$(echo "$COMMIT_MSG_FIRST_LINE" | grep -oE 'CVE-[0-9]{4}-[0-9]+' | head -1 || true)

echo "✓ Patch extracted: $PATCH_FILE"
echo "  Upstream commit:  $UPSTREAM_SHA"
echo "  First-line:       $COMMIT_MSG_FIRST_LINE"
[[ -n "$CVE_HINT" ]] && echo "  CVE detected:    $CVE_HINT"

# Append CHANGELOG entry under [unreleased]. We use a marker-line strategy:
# the first line equal to "## [unreleased]" gets a new entry inserted after it.
CHANGELOG="services/chat/CHANGELOG.cyberos.md"
ENTRY="- license-cherry-pick / security: \`${UPSTREAM_SHA:0:12}\` — ${COMMIT_MSG_FIRST_LINE}"
[[ -n "$CVE_HINT" ]] && ENTRY="$ENTRY (${CVE_HINT})"

python3 - <<PYEOF
import io, pathlib
path = pathlib.Path("$CHANGELOG")
src = path.read_text(encoding="utf-8")
marker = "## [unreleased]"
if marker not in src:
    raise SystemExit("CHANGELOG missing [unreleased] section")
# Insert the entry after the marker block. We add a blank line + bullet.
before, _, after = src.partition(marker)
# After the marker, find the next blank line or section and inject.
lines = after.splitlines()
new_lines = [lines[0]] if lines else []
new_lines.append("")
new_lines.append("""$ENTRY""")
new_lines.extend(lines[1:] if len(lines) > 1 else [])
out = before + marker + "\n" + "\n".join(new_lines[1:]).lstrip("\n")
path.write_text(out, encoding="utf-8")
print(f"✓ CHANGELOG updated at {path}")
PYEOF

# Open the PR if gh is available + we're in a git-tracked branch.
if command -v gh >/dev/null 2>&1 && [[ -n "${GH_TOKEN:-${GITHUB_TOKEN:-}}" ]]; then
    BRANCH="chat-cherry-pick/${NEXT_NUM}-${PATCH_NAME}"
    git checkout -b "$BRANCH" 2>/dev/null || git checkout "$BRANCH"
    git add "$PATCH_FILE" "$CHANGELOG"
    git -c user.email="bot@cyberos.local" -c user.name="cyberos-bot" \
        commit -m "chat: cherry-pick upstream ${UPSTREAM_SHA:0:12}${CVE_HINT:+ ($CVE_HINT)}"
    git push -u origin "$BRANCH"
    gh pr create \
        --title "chat: cherry-pick upstream ${UPSTREAM_SHA:0:12}${CVE_HINT:+ ($CVE_HINT)}" \
        --body "Cherry-pick of upstream \`${UPSTREAM_SHA}\` from \`${UPSTREAM_REPO}\`.

Subject: ${COMMIT_MSG_FIRST_LINE}
${CVE_HINT:+CVE: $CVE_HINT}

This PR requires the \`legal-reviewed\` label before merge per
\`.github/workflows/chat-cherry-pick-review.yml\`. Legal-team: please verify
this commit's content does not affect upstream license terms (i.e. does
not touch LICENSE, licensing/, or root-level package metadata)." \
        --label "legal-review-needed,chat,security"
else
    cat <<EOF

Patch staged at $PATCH_FILE; PR not opened (gh CLI not available or GH_TOKEN unset).
To open the PR manually:

  git checkout -b chat-cherry-pick/${NEXT_NUM}-${PATCH_NAME}
  git add $PATCH_FILE services/chat/CHANGELOG.cyberos.md
  git commit -m "chat: cherry-pick upstream ${UPSTREAM_SHA:0:12}"
  git push -u origin chat-cherry-pick/${NEXT_NUM}-${PATCH_NAME}
  gh pr create --title 'chat: cherry-pick upstream ${UPSTREAM_SHA:0:12}' \\
               --label 'legal-review-needed,chat,security'

Legal-team must then add the \`legal-reviewed\` label before merge.

EOF
fi
