#!/usr/bin/env bash
# Rebuild the vendorable CyberOS payload (dist/cyberos) whenever a source that
# feeds it is staged. dist/ is gitignored, so this only refreshes the local
# artifact - nothing is committed. It keeps tools/install/init.sh vendoring
# current bits during implement/development, per the "auto-build on module
# update" rule. Fast no-op when nothing relevant changed.
#
# Wired into git via .githooks/pre-commit (core.hooksPath=.githooks), which also runs
# tools/install/check-version-sync.sh after the rebuild (TASK-IMP-068). The trigger
# regex below is mirrored there - keep both in sync when adding a payload source.
set -euo pipefail

root="$(git rev-parse --show-toplevel)"

# The sources build.sh reads into the payload (cuo workflow + doctrine + status
# contract + author/audit skills + caf + memory protocol/schema + the init tool +
# the single VERSION). Touch any of these and the payload is stale.
if ! git diff --cached --name-only | grep -Eq \
  '^(modules/cuo/|modules/skill/|modules/memory/memory\.(schema\.json|invariants\.yaml)|AGENTS\.md|tools/install/|tools/caf/|scripts/caf_gate\.sh|VERSION)'; then
  exit 0
fi

echo "cyberos: a vendored source changed - rebuilding dist/cyberos ..."
bash "$root/tools/install/build.sh" >/dev/null
echo "cyberos: dist/cyberos refreshed (gitignored, not committed)."
