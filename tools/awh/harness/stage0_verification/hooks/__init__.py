"""Claude Code hook entry points (PreToolUse, Stop, PostToolUse).

Each module exposes a testable ``main(stdin_text, *, cwd) -> (exit_code, stdout, stderr)``
and a ``__main__`` guard that reads real stdin and exits with the returned code.
Exit code 2 is the canonical "block" signal: Claude Code feeds the hook's stderr
back to the model as the reason.
"""
