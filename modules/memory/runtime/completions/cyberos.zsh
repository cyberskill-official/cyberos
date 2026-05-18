#compdef cyberos
# cyberos zsh completion — Aspect 1.4 of the Layer-1 catalog.
#
# Install (one of):
#   1) source this file from .zshrc
#   2) place in $fpath as `_cyberos`; rebuild compinit cache.
#
# Verified against zsh 5.8 / 5.9.

_cyberos() {
    local context state state_descr line
    typeset -A opt_args

    local -a subcmds=(
        'status:operator dashboard (4 questions)'
        'verify:run full validator pass'
        'doctor:recovery + repair ops (MAINTENANCE mode)'
        'export:deterministic export bundle'
        'search:SQLite-backed search'
        'stats:bucket / class / authority counts'
        'show:list memories with metadata'
        'add:interactive memory wizard'
        'eval:run capability + regression evals for a REF'
        'council:opt-in 4-voice synthesis for ambiguous REFs'
        'sync:multi-machine sync scaffolding'
        'mcp:read-only MCP server'
        'voice:em-dash + AI-vocab linter'
        'doc-consistency:cross-doc consistency check'
        'panic:emergency stop'
        'onboard:interactive new-contributor bootstrap'
        'analytics:local-only usage analytics'
        'security:posture audit (encryption, denylist, perms)'
        'drift:list and resolve drift candidates'
        'help:detailed help'
        'version:print version'
        'prune:(not yet implemented)'
        'hooks:(not yet implemented)'
        'repl:interactive REPL for chained ops'
        'conflicts:(stub — use sync conflicts)'
        'dedup:detect duplicate memories by content fingerprint'
    )

    _arguments -C \
        '1: :->cmd' \
        '*: :->args'

    case $state in
        cmd)
            _describe 'cyberos subcommand' subcmds
            ;;
        args)
            case $words[2] in
                add)
                    if (( CURRENT == 3 )); then
                        _values 'memory type' DEC REF FACT PERSON PROJECT PREFERENCE DRIFT
                    else
                        _arguments \
                            '--slug[slug]' \
                            '--classification[classification]:cls:(personnel client operational public)' \
                            '--authority[authority]:auth:(human-edited human-confirmed llm-explicit llm-implicit)' \
                            '--tags[tags]' \
                            '--sync-class[sync class]:sc:(local-only publishable shared client-visible)' \
                            '--prov-source[prov source]:src:(chat doc code inference manual imported conflict_resolution)' \
                            '--prov-source-ref[prov ref]' \
                            '--freshness-tier[freshness]' \
                            '--auto-tags[auto-suggest tags from GLOSSARY]' \
                            '--non-interactive[no prompts]' \
                            '--dry-run[do not write]'
                    fi
                    ;;
                sync)
                    if (( CURRENT == 3 )); then
                        _values 'sync subcommand' export import conflicts
                    fi
                    ;;
                mcp)
                    if (( CURRENT == 3 )); then
                        _values 'mcp subcommand' serve info
                    fi
                    ;;
                council)
                    if (( CURRENT == 3 )); then
                        local -a refs
                        refs=(${(@f)"$(ls .cyberos-memory/memories/refinements/REF-*.md 2>/dev/null | xargs -n1 basename | sed 's/\.md$//')"})
                        _describe 'REF' refs
                    fi
                    ;;
                eval)
                    if (( CURRENT == 3 )); then
                        local -a refs
                        refs=(${(@f)"$(ls runtime/tests/refinements/ 2>/dev/null | grep '^REF-')"})
                        _describe 'REF eval' refs
                    fi
                    ;;
                status)
                    _arguments \
                        '--verbose[show all findings]' \
                        '--weekly[weekly digest]' \
                        '--watch[continuous]' \
                        '--security[encryption posture]' \
                        '--interval[seconds between refreshes]'
                    ;;
                doctor)
                    if (( CURRENT == 3 )); then
                        _values 'doctor op' rebuild-chain tombstone-orphan resolve-conflict manual-rollback fix-frontmatter compact-ledger decompact-ledger
                    fi
                    ;;
                help)
                    if (( CURRENT == 3 )); then
                        _describe 'subcommand' subcmds
                    fi
                    ;;
            esac
            ;;
    esac
}

_cyberos "$@"
