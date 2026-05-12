# cyberos fish completion — Aspect 1.4 of the Layer-1 catalog.
#
# Install:
#   cp runtime/completions/cyberos.fish ~/.config/fish/completions/

function __cyberos_seen_subcommand
    set -l cmd (commandline -opc)
    test (count $cmd) -gt 1
    and string match -q $argv[1] $cmd[2]
end

# Top-level subcommands
complete -c cyberos -n '__fish_use_subcommand' -a 'status' -d '4-question operator dashboard'
complete -c cyberos -n '__fish_use_subcommand' -a 'verify' -d 'run full validator pass'
complete -c cyberos -n '__fish_use_subcommand' -a 'doctor' -d 'recovery + repair ops'
complete -c cyberos -n '__fish_use_subcommand' -a 'export' -d 'deterministic export bundle'
complete -c cyberos -n '__fish_use_subcommand' -a 'search' -d 'SQLite-backed search'
complete -c cyberos -n '__fish_use_subcommand' -a 'stats' -d 'bucket / class counts'
complete -c cyberos -n '__fish_use_subcommand' -a 'show' -d 'list memories'
complete -c cyberos -n '__fish_use_subcommand' -a 'add' -d 'interactive memory wizard'
complete -c cyberos -n '__fish_use_subcommand' -a 'eval' -d 'run capability + regression evals'
complete -c cyberos -n '__fish_use_subcommand' -a 'council' -d 'opt-in 4-voice synthesis'
complete -c cyberos -n '__fish_use_subcommand' -a 'sync' -d 'multi-machine sync scaffolding'
complete -c cyberos -n '__fish_use_subcommand' -a 'mcp' -d 'read-only MCP server'
complete -c cyberos -n '__fish_use_subcommand' -a 'voice' -d 'voice linter'
complete -c cyberos -n '__fish_use_subcommand' -a 'doc-consistency' -d 'cross-doc check'
complete -c cyberos -n '__fish_use_subcommand' -a 'panic' -d 'emergency stop'
complete -c cyberos -n '__fish_use_subcommand' -a 'onboard' -d 'new-contributor bootstrap'
complete -c cyberos -n '__fish_use_subcommand' -a 'analytics' -d 'local-only usage analytics'
complete -c cyberos -n '__fish_use_subcommand' -a 'security' -d 'security posture audit'
complete -c cyberos -n '__fish_use_subcommand' -a 'drift' -d 'list drift candidates'
complete -c cyberos -n '__fish_use_subcommand' -a 'help' -d 'detailed help'
complete -c cyberos -n '__fish_use_subcommand' -a 'version' -d 'print version'
complete -c cyberos -n '__fish_use_subcommand' -a 'repl' -d 'interactive REPL'
complete -c cyberos -n '__fish_use_subcommand' -a 'dedup' -d 'detect duplicate memories'

# add subcommand
complete -c cyberos -n '__cyberos_seen_subcommand add' -f -a 'DEC REF FACT PERSON PROJECT PREFERENCE DRIFT'
complete -c cyberos -n '__cyberos_seen_subcommand add' -l slug -d 'slug (kebab-case)'
complete -c cyberos -n '__cyberos_seen_subcommand add' -l classification -xa 'personnel client operational public'
complete -c cyberos -n '__cyberos_seen_subcommand add' -l authority -xa 'human-edited human-confirmed llm-explicit llm-implicit'
complete -c cyberos -n '__cyberos_seen_subcommand add' -l sync-class -xa 'local-only publishable shared client-visible'
complete -c cyberos -n '__cyberos_seen_subcommand add' -l prov-source -xa 'chat doc code inference manual imported conflict_resolution'
complete -c cyberos -n '__cyberos_seen_subcommand add' -l auto-tags -d 'GLOSSARY auto-tagging'
complete -c cyberos -n '__cyberos_seen_subcommand add' -l dry-run
complete -c cyberos -n '__cyberos_seen_subcommand add' -l non-interactive

# sync
complete -c cyberos -n '__cyberos_seen_subcommand sync' -f -a 'export import conflicts'

# mcp
complete -c cyberos -n '__cyberos_seen_subcommand mcp' -f -a 'serve info'

# status flags
complete -c cyberos -n '__cyberos_seen_subcommand status' -l verbose
complete -c cyberos -n '__cyberos_seen_subcommand status' -l weekly
complete -c cyberos -n '__cyberos_seen_subcommand status' -l watch
complete -c cyberos -n '__cyberos_seen_subcommand status' -l security
complete -c cyberos -n '__cyberos_seen_subcommand status' -l interval
