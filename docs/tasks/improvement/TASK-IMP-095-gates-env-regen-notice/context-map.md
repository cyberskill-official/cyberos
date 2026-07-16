# TASK-IMP-095 repo context map

## Cone
- `tools/install/install.sh` step 3 (gates.env write: capture the .bak path, compare, one notice line - lines 156-191)
- `tools/install/tests/test_install_hygiene.sh` (new t08_gates_env_regen_notice)

## Patterns the change must follow
- **gates.env is machine-owned** (TASK-CUO-207): regeneration and the timestamped `.bak` are the designed behavior and stay byte-identical; this change adds a message, never a semantics change. `.cyberos/config.yaml` is the durable override home the message points at.
- **Backup churn guard stays**: step 1's `rm -f "$CY"/gates.env.bak.*` (install.sh:72) runs BEFORE step 3, so after any install exactly one `.bak` exists - the one the notice names.
- **Non-interactive always**: install runs in CI and under agents; a printed line is the ceiling (the spec's rejected alternative was prompting).
- **cmp -s for byte-compare**: the house idiom (t06_existing_config_untouched, t05_short_foreign uses it) - not diff, not checksums.

## Blast radius
- Files: 2 modified. One new variable (`env_bak`), one guarded echo; the `[ -f ] && cp` one-liner became an if-block so the path is capturable - identical filesystem effects.
- Consumer impact: an operator whose edit vanishes now learns the .bak path and the durable home at the moment of clobber; unedited/fresh installs see zero new output.
- Cross-module edges: install.sh + hygiene suite shared with TASK-IMP-094/096 - one agent, serial (this landed second).

## Module placement
Correct. `improvement` - installer trust/hygiene, not a product surface.
