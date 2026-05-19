#!/usr/bin/env bash
#
# scripts/automation/cyberos-nightly.sh — nightly soak run.
#
# Wired by ``scripts/automation-install.sh`` as a macOS LaunchAgent.
# Runs on the host (not in any agent sandbox), against the real memory.
#
# Steps:
#   1. cyberos doctor — must return overall OK
#   2. cyberos consolidate --dry-run — confirms a real consolidate would
#      proceed cleanly; doesn't actually sign / publish
#
# Output goes to ~/Library/Logs/cyberos/nightly.log . On regression a
# notification posts via osascript (best-effort; silent on machines
# without GUI).

set -euo pipefail

PROJECT="${CYBEROS_PROJECT:-${1:-}}"
if [[ -z "$PROJECT" || ! -d "$PROJECT" ]]; then
    echo "usage: cyberos-nightly.sh <project-root> (or set CYBEROS_PROJECT)" >&2
    exit 2
fi

LOG_DIR="$HOME/Library/Logs/cyberos"
mkdir -p "$LOG_DIR"
LOG="$LOG_DIR/nightly.log"

notify() {
    local title="$1"; shift
    local msg="$*"
    if command -v osascript >/dev/null 2>&1; then
        osascript -e "display notification \"$msg\" with title \"$title\"" \
            >/dev/null 2>&1 || true
    fi
}

ts() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

{
    echo "=== nightly $(ts) ==="
    cd "$PROJECT"

    if [[ ! -d .cyberos-memory ]]; then
        echo "no .cyberos-memory in $PROJECT — skipping"
        exit 0
    fi

    echo "→ cyberos doctor"
    if python -m cyberos --store .cyberos-memory doctor; then
        echo "doctor: OK"
    else
        echo "doctor: FAIL"
        notify "cyberos nightly: FAIL" \
            "cyberos doctor reported errors. See $LOG"
        exit 1
    fi

    echo
    echo "→ cyberos consolidate --dry-run"
    if python -m cyberos --store .cyberos-memory consolidate --dry-run; then
        echo "consolidate dry-run: OK"
    else
        echo "consolidate dry-run: FAIL"
        notify "cyberos nightly: consolidate dry-run failed" \
            "See $LOG"
        exit 1
    fi
} >> "$LOG" 2>&1
