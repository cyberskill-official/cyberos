#!/usr/bin/env bash
# Ship the generated docs site (dist/website) to the VPS docs mount - the ONE implementation
# behind both callers (.github/workflows/deploy.yml `docs` job + .github/workflows/release.yml
# `docs` job).
#
# Why a shared script with a lock (FR-IMP-079): the two workflows used to inline the same
# snippet, both staging into the SAME ~/cyberos/apps/console/docs.new. A branch push followed
# seconds later by the v1.0.0 tag push ran both docs jobs concurrently; the release shipper's
# `rm -rf docs.new` deleted the deploy shipper's staging between its tar extract and its mv -
# "mv: cannot stat '~/cyberos/apps/console/docs.new'" (observed 2026-07-13, deploy run #127,
# release run green). Worse interleavings could leave the live `docs` dir deleted with the
# replacement gone. Per-run staging dirs make extraction collision-free; the flock'd swap
# serializes the replace, so N concurrent shippers all go green and the last swap wins (same
# content either way - every shipper builds from the same main).
#
# Transport notes inherited from the inline era: stream tar over ssh - no runner tgz, no VPS
# /tmp copy (the scp+/tmp path once shipped a truncated archive: "gzip: stdin: unexpected end
# of file" on extract). pipefail makes a broken pipe fail the step honestly; the size echo
# feeds the deploy log.
#
# Stale-staging sweep: the flock'd section removes docs.new* dirs untouched for 2h+ (abandoned
# by a killed run). An in-flight extract refreshes its dir mtime continuously and finishes in
# seconds, so a live concurrent shipper is never swept.
#
# Requires: dist/website built (tools/docs-site/build.sh); env VPS_HOST / VPS_USER / VPS_SSH_KEY.
set -euo pipefail
: "${VPS_HOST:?VPS_HOST missing}" "${VPS_USER:?VPS_USER missing}" "${VPS_SSH_KEY:?VPS_SSH_KEY missing}"
[ -d dist/website ] || { echo "ship-docs: dist/website missing - run tools/docs-site/build.sh first" >&2; exit 1; }

keydir="${RUNNER_TEMP:-$(mktemp -d)}"
install -m 600 /dev/stdin "$keydir/deploy_key" <<< "$VPS_SSH_KEY"

echo "site: $(du -sh dist/website | cut -f1), $(find dist/website -type f | wc -l) files"
stage="docs.new.${GITHUB_RUN_ID:-local$$}.${GITHUB_RUN_ATTEMPT:-0}"
tar -czf - -C dist/website . | ssh -4 -o StrictHostKeyChecking=accept-new -i "$keydir/deploy_key" "$VPS_USER@$VPS_HOST" \
  "set -e; mkdir -p ~/cyberos/apps/console && cd ~/cyberos/apps/console \
   && mkdir '$stage' && tar -xzf - -C '$stage' \
   && flock .docs-ship.lock -c 'rm -rf docs && mv \"$stage\" docs && find . -maxdepth 1 -name \"docs.new*\" -mmin +120 -exec rm -rf {} +'"
echo "ship-docs: live at ~/cyberos/apps/console/docs (staged as $stage)"
