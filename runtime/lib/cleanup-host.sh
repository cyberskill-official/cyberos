#!/bin/bash
# cyberos cleanup script — generated 2026-05-12T12:34:15+07:00
# Run from: /sessions/trusting-zen-mccarthy/mnt/cyberos
set -e

rm -f ".cyberos-memory/cache/test-fixtures/test-sync-bundle-a.zip"  # .cyberos-memory/cache/test-fixtures/test-* dev scratch
rm -f ".cyberos-memory/cache/test-fixtures/test-sync-bundle-conflict.zip"  # .cyberos-memory/cache/test-fixtures/test-* dev scratch
rm -f ".cyberos-memory/cache/test-fixtures/test-sync-bundle-b.zip"  # .cyberos-memory/cache/test-fixtures/test-* dev scratch
rm -f ".cyberos-memory/cache/test-fixtures/test-sync-bundle-2.zip"  # .cyberos-memory/cache/test-fixtures/test-* dev scratch
rm -f ".cyberos-memory/cache/test-fixtures/test-sync-bundle.zip"  # .cyberos-memory/cache/test-fixtures/test-* dev scratch
rm -f ".cyberos-memory/cache/test-fixtures/test-via-umbrella.zip"  # .cyberos-memory/cache/test-fixtures/test-* dev scratch
rm -f ".cyberos-memory/cache/test-fixtures/test-claude-settings.json"  # .cyberos-memory/cache/test-fixtures/test-* dev scratch
rm -f ".cyberos-memory/cache/test-fixtures/audit-bundle.zip"  # audit-script leftover bundle
rm -rf ".cyberos-memory/cache/test-fixtures/cold-test"  # test cold-storage archives
rm -rf ".cyberos-memory/cache/site"  # static-site render (regenerable)
rm -f ".cyberos-memory/cache/test-fixtures/sync/20260512-074156.md"  # sync import report (regenerable)
rm -f ".cyberos-memory/cache/test-fixtures/sync/20260512-095743.md"  # sync import report (regenerable)
rm -f ".cyberos-memory/cache/test-fixtures/sync/20260512-074212.md"  # sync import report (regenerable)
rm -rf ".cyberos-memory/.branches/experiment-tier-b"  # .branches/ experimental snapshot
