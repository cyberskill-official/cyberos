#!/bin/bash
# cyberos cleanup script — generated 2026-05-12T12:34:15+07:00
# Run from: /sessions/trusting-zen-mccarthy/mnt/cyberos
set -e

rm -f "outputs/test-sync-bundle-a.zip"  # outputs/test-* dev scratch
rm -f "outputs/test-sync-bundle-conflict.zip"  # outputs/test-* dev scratch
rm -f "outputs/test-sync-bundle-b.zip"  # outputs/test-* dev scratch
rm -f "outputs/test-sync-bundle-2.zip"  # outputs/test-* dev scratch
rm -f "outputs/test-sync-bundle.zip"  # outputs/test-* dev scratch
rm -f "outputs/test-via-umbrella.zip"  # outputs/test-* dev scratch
rm -f "outputs/test-claude-settings.json"  # outputs/test-* dev scratch
rm -f "outputs/audit-bundle.zip"  # audit-script leftover bundle
rm -rf "outputs/cold-test"  # test cold-storage archives
rm -rf "outputs/site"  # static-site render (regenerable)
rm -f "outputs/sync/20260512-074156.md"  # sync import report (regenerable)
rm -f "outputs/sync/20260512-095743.md"  # sync import report (regenerable)
rm -f "outputs/sync/20260512-074212.md"  # sync import report (regenerable)
rm -rf ".cyberos-memory/.branches/experiment-tier-b"  # .branches/ experimental snapshot
