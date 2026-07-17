#!/usr/bin/env bash
# TASK-CHAT-001 §4 #3 + §4 #5 — verify the GitHub Actions workflows exist
# and carry the right cron / trigger / label-gate semantics.
#
# We parse the YAML conservatively with grep — the workflow harness will
# do the deeper validation at runtime.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

fail() { echo "::error::$*" >&2; exit 1; }
ok()   { echo "  ✓ $*"; }

DRIFT_WF="$REPO_ROOT/.github/workflows/chat-license-drift-watcher.yml"
CHERRY_WF="$REPO_ROOT/.github/workflows/chat-cherry-pick-review.yml"

[[ -f "$DRIFT_WF" ]] || fail "missing $DRIFT_WF"
[[ -f "$CHERRY_WF" ]] || fail "missing $CHERRY_WF"
ok "Both workflow files present"

# Drift watcher: must have a schedule cron AND workflow_dispatch.
grep -qE "^[[:space:]]*-?[[:space:]]*cron:[[:space:]]*'0 0 \* \* 1'" "$DRIFT_WF" \
    || fail "drift watcher missing 'cron: 0 0 * * 1' (Monday 00:00 UTC)"
ok "Drift watcher cron = Monday 00:00 UTC"

grep -q "workflow_dispatch:" "$DRIFT_WF" \
    || fail "drift watcher missing workflow_dispatch trigger"
ok "Drift watcher supports manual trigger"

# Drift watcher: must reference the script.
grep -q "scripts/check-license-drift.sh" "$DRIFT_WF" \
    || fail "drift watcher does not invoke services/chat/scripts/check-license-drift.sh"
ok "Drift watcher invokes the check script"

# Cherry-pick gate: must trigger on services/chat/patches/** paths AND on
# label changes (so legal can flip the gate by adding the label).
grep -q "services/chat/patches/\*\*" "$CHERRY_WF" \
    || fail "cherry-pick gate does not watch services/chat/patches/**"
ok "Cherry-pick gate triggers on services/chat/patches/**"

grep -qE "(labeled|unlabeled)" "$CHERRY_WF" \
    || fail "cherry-pick gate does not respond to label changes"
ok "Cherry-pick gate responds to label changes"

# Cherry-pick gate: must require legal-reviewed label.
grep -q "legal-reviewed" "$CHERRY_WF" \
    || fail "cherry-pick gate does not check for 'legal-reviewed' label"
ok "Cherry-pick gate requires 'legal-reviewed' label"

echo "✓ workflows_present_test: drift + cherry-pick workflows have required shape"
