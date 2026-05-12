#!/usr/bin/env python3
"""
cyberos_repl.py — interactive REPL for chained cyberos operations.

Aspect 1.6 of the Layer-1 improvement catalog.

Avoids re-running session.start for every individual memory op. Useful when
you want to: add 5 FACTs + verify + show + export all in one shell session
without spawning 5 child processes.

Commands inside the REPL match the umbrella `cyberos` CLI exactly:
    > status
    > add FACT --slug pricing-tier-3 --auto-tags
    > verify
    > show --tag pricing
    > help
    > exit | quit | q | EOF

Special REPL meta-commands:
    > .cd <path>             change working directory
    > .pwd                   print working directory
    > .last                  show last command + return code
    > .history               show command history this session
    > .reload                reload from disk (e.g. if external edit happened)
    > .save <path>           write history to file (resumeable session log)

Pattern from `nanoclaw-repl` / autonomous-loops skill — keep state in one
process; each `cyberos <subcmd>` is a fresh subprocess (so we get fresh
brain_root resolution + clean stdout). Saves session.start overhead, not
process spawn.
"""
from __future__ import annotations
import os
import shlex
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))


def find_cyberos_bin() -> Path:
    """Locate the umbrella `cyberos` binary."""
    cur = Path(__file__).resolve().parent
    cand = cur / "cyberos"
    if cand.exists():
        return cand
    raise SystemExit(f"could not find cyberos binary near {__file__}")


BANNER = r"""
  cyberos REPL — interactive shell for .cyberos-memory ops
  Type a cyberos subcommand (e.g. `status`, `add FACT --slug ...`),
  `help` for command list, or `.help` for REPL meta-commands.
  Exit with `exit`, `quit`, `q`, or Ctrl-D.
"""


META_HELP = """
  REPL meta-commands:
    .help                show this help
    .cd <path>           change working directory
    .pwd                 print working directory
    .last                show last cyberos cmd + rc
    .history             show command history
    .reload              re-resolve cyberos binary
    .save <path>         save history to a .cyberos-repl.log file
    .clear               clear history
    .env [KEY]           show CYBEROS_* env vars (all or one)
    .env KEY=VAL         set an env var (e.g. .env CYBEROS_SUBJECT_ID=stephen)
"""


def _setup_readline():
    """Tier E.6 — persistent history + up-arrow recall via libreadline."""
    try:
        import readline
    except ImportError:
        return None
    hist_file = Path.home() / ".cyberos" / "repl-history"
    hist_file.parent.mkdir(parents=True, exist_ok=True)
    if hist_file.exists():
        try:
            readline.read_history_file(str(hist_file))
        except Exception:
            pass
    readline.set_history_length(1000)
    # Tab completion: complete on cyberos subcommands
    try:
        SUBCMDS = ("status verify doctor export search stats show add eval council sync "
                   "mcp voice doc-consistency panic onboard analytics security drift help "
                   "version prune hooks repl conflicts dedup graph refinements explain "
                   "compact-stats mutation-test lock cold-storage skill semantic-search tui "
                   "history branch ref-from-drift autorepair serve replicate tenant crdt sign "
                   "parallel-validate static migrate edit bulk-set bulk-unset hybrid-search "
                   "audit-stream alert").split()
        def completer(text, state):
            opts = [c for c in SUBCMDS if c.startswith(text)]
            if state < len(opts):
                return opts[state]
            return None
        readline.set_completer(completer)
        readline.parse_and_bind("tab: complete")
    except Exception:
        pass
    return hist_file


def repl():
    binary = find_cyberos_bin()
    history: list[dict] = []
    last_rc = 0
    hist_file = _setup_readline()
    print(BANNER)

    while True:
        try:
            line = input("\033[1;36mcyberos>\033[0m ").strip()
        except EOFError:
            print()
            break
        except KeyboardInterrupt:
            print("^C (type exit to quit)")
            continue

        if not line:
            continue

        if line in ("exit", "quit", "q"):
            break

        # Meta-commands
        if line.startswith("."):
            parts = line.split(None, 1)
            meta = parts[0]
            rest = parts[1] if len(parts) > 1 else ""
            if meta == ".help":
                print(META_HELP)
            elif meta == ".cd":
                if not rest:
                    print("usage: .cd <path>")
                else:
                    try:
                        os.chdir(os.path.expanduser(rest))
                        print(f"  cwd: {os.getcwd()}")
                    except OSError as e:
                        print(f"  error: {e}")
            elif meta == ".pwd":
                print(f"  cwd: {os.getcwd()}")
            elif meta == ".last":
                if not history:
                    print("  (no commands yet)")
                else:
                    h = history[-1]
                    print(f"  ts:  {h['ts']}")
                    print(f"  cmd: {h['cmd']}")
                    print(f"  rc:  {h['rc']}")
            elif meta == ".history":
                if not history:
                    print("  (empty)")
                for i, h in enumerate(history[-30:], 1):
                    marker = "✗" if h["rc"] else "✓"
                    print(f"  {i:3d}. {marker} {h['cmd']}")
            elif meta == ".reload":
                binary = find_cyberos_bin()
                print(f"  cyberos binary: {binary}")
            elif meta == ".save":
                if not rest:
                    out = Path(".cyberos-repl.log")
                else:
                    out = Path(os.path.expanduser(rest))
                lines = [f"# cyberos REPL session — {datetime.now(ICT).isoformat(timespec='seconds')}\n"]
                for h in history:
                    lines.append(f"# {h['ts']}  rc={h['rc']}\n")
                    lines.append(f"{h['cmd']}\n")
                out.write_text("".join(lines), encoding="utf-8")
                print(f"  saved {len(history)} commands → {out}")
            elif meta == ".clear":
                history.clear()
                print("  history cleared")
            elif meta == ".env":
                rest = rest.strip()
                if "=" in rest:
                    k, v = rest.split("=", 1)
                    os.environ[k.strip()] = v.strip()
                    print(f"  set {k.strip()}={v.strip()}")
                elif rest:
                    print(f"  {rest}={os.environ.get(rest, '(unset)')}")
                else:
                    for k, v in sorted(os.environ.items()):
                        if k.startswith("CYBEROS_"):
                            print(f"  {k}={v}")
            else:
                print(f"  unknown meta command: {meta}; try .help")
            continue

        # Forward to umbrella binary as one cyberos subprocess invocation
        try:
            argv = shlex.split(line)
        except ValueError as e:
            print(f"  parse error: {e}")
            continue
        cmd = [sys.executable, str(binary), *argv]
        ts = datetime.now(ICT).isoformat(timespec="seconds")
        try:
            rc = subprocess.run(cmd).returncode
        except FileNotFoundError as e:
            print(f"  spawn error: {e}")
            rc = -1
        history.append({"ts": ts, "cmd": line, "rc": rc})
        last_rc = rc

    # Persist readline history on exit
    if hist_file:
        try:
            import readline
            readline.write_history_file(str(hist_file))
        except Exception:
            pass

    print(f"  bye. {len(history)} commands run this session.")
    return last_rc


if __name__ == "__main__":
    sys.exit(repl())
