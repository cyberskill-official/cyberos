# TASK-IMP-095 implementation plan

1. **Capture the backup path** (enables 1.1) - install.sh:156-161: `[ -f ] && cp` becomes an if-block setting `env_bak="$env_file.bak.$(date +%s)"` before the copy; empty when no prior file existed. Filesystem behavior byte-identical to today.
2. **The notice** (clauses 1.1, 1.2) - install.sh:184-191, immediately after the heredoc that writes the new file: `if [ -n "$env_bak" ] && ! cmp -s "$env_bak" "$env_file"` -> echo `cyberos install: gates.env regenerated (previous kept at <bak>); durable overrides belong in .cyberos/config.yaml`. Both silent arms fall through the guard.
3. **Coverage** (clause 1.3) - hygiene t08_gates_env_regen_notice: fresh-silent, unedited-silent, edited-notice (exactly once, names an existing .bak that carries the operator edit, names config.yaml). Speed flags fine - step 3 runs before migrate/memory/MCP.
4. **Gates** - hygiene suite 19/19; live scratch capture of the exact line in the gate log.

Order: after TASK-IMP-094's step-5b work (same file, serial agent), before TASK-IMP-096.
