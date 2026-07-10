#!/usr/bin/env bash
# Rebuild the generated website whenever a documentation source is staged, and stage the
# regenerated output with it — so a commit can never ship stale generated HTML. The CI
# docs-prerender-gate re-verifies the same property on every PR. Fast no-op when no doc
# source changed.
set -euo pipefail

root="$(git rev-parse --show-toplevel)"

if ! git diff --cached --name-only | grep -Eq \
  '^(docs/|modules/[^/]+/docs/|services/[^/]+/docs/|website/build/|CHANGELOG\.md|modules/[^/]+/CHANGELOG\.md|services/[^/]+/CHANGELOG\.md|docs/feature-requests/|docs/non-functional-requirements/)'; then
  exit 0
fi

echo "docs: a documentation source changed - rebuilding the generated site ..."
bash "$root/website/build/build.sh" >/dev/null
git add "$root/website/docs"
echo "docs: website/docs regenerated + staged."
