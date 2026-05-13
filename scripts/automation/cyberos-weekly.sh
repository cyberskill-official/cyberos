#!/usr/bin/env bash
#
# scripts/automation/cyberos-weekly.sh — weekly maintenance run.
#
# Runs on Sundays at 02:00 local time via macOS LaunchAgent.
#
# Steps:
#   1. cyberos backup     — incremental hard-link snapshot to ~/cyberos-backups/<project>/
#   2. cyberos consolidate — Walk → Compact → Sign → Publish (real, not dry-run)
#   3. determinism guard   — two exports must be byte-identical

set -euo pipefail

PROJECT="${CYBEROS_PROJECT:-${1:-}}"
if [[ -z "$PROJECT" || ! -d "$PROJECT" ]]; then
    echo "usage: cyberos-weekly.sh <project-root> (or set CYBEROS_PROJECT)" >&2
    exit 2
fi

PROJECT_NAME="$(basename "$PROJECT")"
BACKUP_ROOT="${CYBEROS_BACKUP_ROOT:-$HOME/cyberos-backups}"
BACKUP_TARGET="$BACKUP_ROOT/$PROJECT_NAME"

LOG_DIR="$HOME/Library/Logs/cyberos"
mkdir -p "$LOG_DIR"
LOG="$LOG_DIR/weekly.log"

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
    echo "=== weekly $(ts) ==="
    cd "$PROJECT"

    if [[ ! -d .cyberos-memory ]]; then
        echo "no .cyberos-memory in $PROJECT — skipping"
        exit 0
    fi

    echo "→ cyberos backup → $BACKUP_TARGET"
    mkdir -p "$BACKUP_TARGET"
    python -m cyberos --store .cyberos-memory backup \
        --target "$BACKUP_TARGET" --label "weekly-$(ts)" \
        || { echo "backup FAIL"; notify "cyberos weekly: backup failed" "See $LOG"; exit 1; }

    echo
    echo "→ cyberos consolidate"
    python -m cyberos --store .cyberos-memory consolidate \
        || { echo "consolidate FAIL"; notify "cyberos weekly: consolidate failed" "See $LOG"; exit 1; }

    echo
    echo "→ determinism guard (two exports → byte-identical?)"
    SCRATCH="$(mktemp -d)"
    trap 'rm -rf "$SCRATCH"' EXIT
    python -m cyberos --store .cyberos-memory export "$SCRATCH/a.zip" > "$SCRATCH/a.sha"
    python -m cyberos --store .cyberos-memory export "$SCRATCH/b.zip" > "$SCRATCH/b.sha"
    if cmp -s "$SCRATCH/a.sha" "$SCRATCH/b.sha"; then
        echo "determinism: OK ($(cat "$SCRATCH/a.sha"))"
    else
        echo "determinism: REGRESSION"
        cat "$SCRATCH/a.sha" "$SCRATCH/b.sha"
        notify "cyberos weekly: NON-DETERMINISTIC EXPORT" \
            "Export round-trip diverged. See $LOG"
        exit 1
    fi
} >> "$LOG" 2>&1
