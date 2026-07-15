---
artefact: architecture-decision-record@1
adr_id: ADR-IMP-068-001
task_id: TASK-IMP-068
status: accepted
created: 2026-07-12
verdict: pass (architecture-decision-record-audit)
---
# ADR-IMP-068-001: Prove payload-version sync by rebuilding in CI, not by committing dist/

## Context
VERSION auto-bumps in CI; the payload build is manual and local; dist/ is gitignored. The installed plugin drifted to 1.2.0 while VERSION reached 1.7.0. Five files outside tools/cyberos-install are touched, tripping the ADR condition.

## Options considered
1. Commit a stamped dist/ to the repo and diff it in CI - rejected: noisy diffs on every source touch, merge conflicts in generated files, and the operator explicitly ruled dist stays gitignored (2026-07-12 plan approval).
2. Derive the payload version at install time instead of stamping - rejected: consumers (marketplace add, .plugin file) read static manifests; no execution hook exists at install.
3. Rebuild in CI + compare stamps + wire the local hook into .githooks + prove the bump inline in version.yml - CHOSEN.

## Decision
Option 3. One read-only comparator (check-version-sync.sh) shared by CI gate, git hook, the version-bump job, and later the release publisher (TASK-IMP-069).

## Consequences
- dist/ stays gitignored; the repo never carries stale generated stamps.
- The bump commit itself is proven buildable ([skip ci] blindness closed inline in version.yml).
- A contributor bypassing hooks (--no-verify) is still caught by payload-gate.yml on push/PR.
- New CI job cost: < 3 min, no network beyond checkout.
