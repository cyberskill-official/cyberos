---
artefact: repo-context-map@1
task_id: TASK-IMP-068
created: 2026-07-12
verdict: pass (repo-context-map-audit)
---
# Repo context map - TASK-IMP-068

## Baseline patterns the new code must follow
- error_type: bash `set -euo pipefail`, explicit numeric exit codes (0 ok / 10 policy fail / 2 unreadable) - pinned_in: tools/cyberos-init/init.sh, scripts/caf_gate.sh
- logging: `cyberos-init:` / `cyberos:` prefixed echo lines, errors to stderr - pinned_in: tools/cyberos-init/build.sh, .pre-commit-hooks/cyberos-payload-build.sh
- test_framework: standalone bash test scripts, function-per-case, no framework - pinned_in: deploy/obs/tests, scripts/local_verify.sh style

## Schemas / interfaces in scope
- Payload stamp surface (6 artifacts): VERSION, plugin/.claude-plugin/plugin.json (.version), .claude-plugin/marketplace.json (plugins[0].version), mcp/package.json (.version), manifest.yaml (cyberos_version), cyberos.plugin!.claude-plugin/plugin.json
- Workflow triggers: version.yml bump job (bot commit carries [skip ci])

## Files outside the immediate domain (tools/cyberos-init/)
1. .github/workflows/payload-gate.yml (new)
2. .github/workflows/version.yml (modified - inline proof step)
3. .githooks/pre-commit (new - core.hooksPath is .githooks)
4. .pre-commit-hooks/cyberos-payload-build.sh (modified - cross-reference)
5. docs/deploy/RELEASE.md (modified - enforcement claim)

files_outside_immediate_domain: 5 (> 3 -> ADR required, steps 3-4 fire)

## Blast radius
file_count: 9 | module_count: 3 (tools/cyberos-init, .github, .githooks) | cross_module_edges: hooks->build.sh, CI->build.sh+check
module_placement_warning: null (TASK-IMP-068 declared module improvement = cross-cutting; correct)
