# cyberos bash completion — Aspect 1.4 of the Layer-1 catalog.
#
# Install:
#     source /path/to/cyberos/runtime/completions/cyberos.bash
# or:
#     cp runtime/completions/cyberos.bash /etc/bash_completion.d/cyberos
#     # then restart your shell
#
# Verified against bash 5.x. Should also work under bash 4.x.

_cyberos_complete() {
    local cur prev cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Top-level subcommands
    local cmds="status verify doctor export search stats show add eval council sync mcp voice doc-consistency panic onboard analytics security drift help version prune hooks repl conflicts dedup"

    # First-position completion: subcommand
    if [[ $COMP_CWORD -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "$cmds" -- "$cur") )
        return 0
    fi

    cmd="${COMP_WORDS[1]}"

    case "$cmd" in
        add)
            if [[ $COMP_CWORD -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "DEC REF FACT PERSON PROJECT PREFERENCE DRIFT" -- "$cur") )
                return 0
            fi
            case "$prev" in
                --classification) COMPREPLY=( $(compgen -W "personnel client operational public" -- "$cur") ); return 0 ;;
                --authority) COMPREPLY=( $(compgen -W "human-edited human-confirmed llm-explicit llm-implicit" -- "$cur") ); return 0 ;;
                --sync-class) COMPREPLY=( $(compgen -W "local-only publishable shared client-visible" -- "$cur") ); return 0 ;;
                --prov-source) COMPREPLY=( $(compgen -W "chat doc code inference manual imported conflict_resolution" -- "$cur") ); return 0 ;;
            esac
            COMPREPLY=( $(compgen -W "--slug --classification --authority --tags --sync-class --prov-source --prov-source-ref --freshness-tier --auto-tags --non-interactive --dry-run" -- "$cur") )
            return 0
            ;;
        show)
            COMPREPLY=( $(compgen -W "--scope --tag --class --tombstoned --recent" -- "$cur") )
            return 0
            ;;
        sync)
            if [[ $COMP_CWORD -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "export import conflicts" -- "$cur") )
                return 0
            fi
            COMPREPLY=( $(compgen -W "--to --from --include --dry-run" -- "$cur") )
            return 0
            ;;
        mcp)
            if [[ $COMP_CWORD -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "serve info" -- "$cur") )
                return 0
            fi
            ;;
        council)
            if [[ $COMP_CWORD -eq 2 ]]; then
                # complete REF-NNN slugs from .cyberos-memory
                local refs=$(ls .cyberos-memory/memories/refinements/REF-*.md 2>/dev/null | xargs -n1 basename | sed 's/\.md$//' | head -50)
                COMPREPLY=( $(compgen -W "$refs" -- "$cur") )
                return 0
            fi
            COMPREPLY=( $(compgen -W "--voices --print" -- "$cur") )
            return 0
            ;;
        eval)
            if [[ $COMP_CWORD -eq 2 ]]; then
                local refs=$(ls runtime/tests/refinements/ 2>/dev/null | grep '^REF-' | head -50)
                COMPREPLY=( $(compgen -W "$refs" -- "$cur") )
                return 0
            fi
            ;;
        verify)
            COMPREPLY=( $(compgen -W "--self-test --denylist --pre-commit" -- "$cur") )
            return 0
            ;;
        status)
            COMPREPLY=( $(compgen -W "--verbose --weekly --watch --security --interval" -- "$cur") )
            return 0
            ;;
        doctor)
            if [[ $COMP_CWORD -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "rebuild-chain tombstone-orphan resolve-conflict manual-rollback fix-frontmatter compact-ledger decompact-ledger" -- "$cur") )
                return 0
            fi
            COMPREPLY=( $(compgen -W "--repair --reason --dry-run" -- "$cur") )
            return 0
            ;;
        export)
            COMPREPLY=( $(compgen -W "-o --to --sanitize-level --dry-run --daemon --interval --verify" -- "$cur") )
            return 0
            ;;
        analytics)
            if [[ $COMP_CWORD -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "log report purge" -- "$cur") )
                return 0
            fi
            COMPREPLY=( $(compgen -W "--period --format" -- "$cur") )
            return 0
            ;;
        help)
            if [[ $COMP_CWORD -eq 2 ]]; then
                COMPREPLY=( $(compgen -W "$cmds" -- "$cur") )
                return 0
            fi
            ;;
        panic)
            COMPREPLY=( $(compgen -W "--reason --resolve" -- "$cur") )
            return 0
            ;;
        onboard)
            COMPREPLY=( $(compgen -W "--shared --persona --non-interactive" -- "$cur") )
            return 0
            ;;
    esac

    return 0
}

complete -F _cyberos_complete cyberos
