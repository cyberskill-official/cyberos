---
artefact: repo-context-map@1
task_id: TASK-IMP-088
created: 2026-07-16
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated, ADR trigger evaluated)
---
# Repo context map - TASK-IMP-088

## Baseline patterns the new code must follow
- create-once scaffold shape: step 3b writes .cyberos/config.yaml through a `[ ! -f ]`-guarded `cat > <<EOF` heredoc and never clobbers - pinned_in: tools/install/install.sh step 3b (TASK-CUO-207 §1 #3); the change stays inside that guard and heredoc
- platform detection: `is_platform_repo()` already exists - marker file `modules/memory/memory.schema.json` under `$root` - pinned_in: install.sh (definition formerly at :362-365, sole caller the AGENTS.md handling at :375). ORDERING HAZARD: the definition sat ~190 lines BELOW step 3b; calling it there would hit `command not found` (127) inside a non-final `&&` position, which `set -e` forgives - i.e. the platform repo would SILENTLY scaffold the consumer line. Hoisted above step 3b with a comment; old site keeps a pointer comment
- short-circuit conditional assignment under `set -euo pipefail`: `cmd && var=...` with a failing non-final command is the file's own idiom - pinned_in: install.sh:44 (seed guard), :137-141 (Makefile claims)
- hygiene-suite harness shape: mkrepo scratch git repos, ok/fail counters, `pass=N fail=N` summary, non-zero exit on fail, speed flags for scenarios that skip migrate/memory/MCP - pinned_in: tools/install/tests/test_install_hygiene.sh:14-24 and `_t05_install`:160
- suite discovery: scripts/tests/run_all.sh globs `tools/install/tests/test_*.sh` - the extended suite needs zero wiring

## Schemas / interfaces in scope
- config.yaml scaffold contract: consumer repos get the LIVE line `task_template: task@1`; the platform repo keeps today's commented `# task_template: engineering-spec@1`; every other scaffold line byte-identical to today (spec §1 #1.1/#1.2)
- the task-author resolution chain READS `task_template` from .cyberos/config.yaml; the chain itself is out of scope and untouched (spec Out of scope)

## Files outside the immediate domain (tools/install/)
None. Both touched files (tools/install/install.sh, tools/install/tests/test_install_hygiene.sh) are spec-declared in `modified_files`.

files_outside_immediate_domain: 0 (<= 3 -> no ADR trigger)

## Blast radius
file_count: 2 modified (install.sh ~+16/-5 incl. hoist; test suite +~45) | module_count: 1 (tools/install) | cross_module_edges: none new (the hoisted function keeps serving the AGENTS.md handling below) module_placement_warning: null (spec declares `service: tools/install`) Behavioral radius: FRESH installs only - the create-once guard means every existing config.yaml (consumer or platform) is untouched on re-install (t06_existing_config_untouched); the platform repo's own future re-installs keep today's commented bytes via the existing detector (t06_platform_keeps_comment).
