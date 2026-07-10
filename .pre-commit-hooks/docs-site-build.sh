#!/usr/bin/env bash
# Verify the docs site still BUILDS whenever a documentation source is staged. The site is
# generated into gitignored dist/website (nothing generated is committed), so the check is
# "the build is green", not a drift diff. CI re-runs the same build on every docs PR.
set -euo pipefail

root="$(git rev-parse --show-toplevel)"

if ! git diff --cached --name-only | grep -Eq \
  '^(docs/|modules/[^/]+/docs/|services/[^/]+/docs/|tools/docs-site/|CHANGELOG\.md|modules/[^/]+/CHANGELOG\.md|services/[^/]+/CHANGELOG\.md|docs/feature-requests/|docs/non-functional-requirements/)'; then
  exit 0
fi

echo "docs: a documentation source changed - verifying the site builds ..."
bash "$root/tools/docs-site/build.sh" >/dev/null
echo "docs: site build green (output at dist/website, gitignored)."
