#!/usr/bin/env bash
#
# scripts/automation-install.sh — install macOS LaunchAgents for the
# cyberos nightly + weekly automation jobs.
#
# After running this, two jobs run on the host:
#   * os.cyberskill.world.nightly  — daily 01:09 local
#   * os.cyberskill.world.weekly   — Sundays 02:07 local
#
# Logs land at ~/Library/Logs/cyberos/{nightly,weekly}.log .
#
# Usage:
#     ./scripts/automation-install.sh --target /path/to/project
#     ./scripts/automation-install.sh --uninstall

set -euo pipefail

# OS dispatch — this script handles macOS (launchd) directly; on other
# platforms it delegates to the per-OS installer.
case "$(uname -s)" in
    Darwin)
        : # macOS launchd path follows
        ;;
    Linux)
        exec "$(dirname "$0")/automation-install-linux.sh" "$@"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        echo "automation-install.sh: detected Windows; please run instead:" >&2
        echo "  powershell -ExecutionPolicy Bypass -File scripts\\automation-install.ps1 -Target <project>" >&2
        exit 2
        ;;
    *)
        echo "automation-install.sh: unsupported OS '$(uname -s)'" >&2
        echo "  supported: macOS (launchd), Linux (systemd-user or cron), Windows (Task Scheduler)" >&2
        exit 2
        ;;
esac

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TARGET=""
ACTION="install"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)    TARGET="$2"; shift 2 ;;
        --uninstall) ACTION="uninstall"; shift ;;
        -h|--help)
            grep '^#' "$0" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
done

NIGHTLY_LABEL="os.cyberskill.world.nightly"
WEEKLY_LABEL="os.cyberskill.world.weekly"
AGENTS_DIR="$HOME/Library/LaunchAgents"
NIGHTLY_PLIST="$AGENTS_DIR/$NIGHTLY_LABEL.plist"
WEEKLY_PLIST="$AGENTS_DIR/$WEEKLY_LABEL.plist"

# ---------------------------------------------------------------------- uninstall

if [[ "$ACTION" == "uninstall" ]]; then
    echo "=== uninstalling cyberos LaunchAgents ==="
    for plist in "$NIGHTLY_PLIST" "$WEEKLY_PLIST"; do
        if [[ -f "$plist" ]]; then
            launchctl unload "$plist" 2>/dev/null || true
            rm -f "$plist"
            echo "  removed: $plist"
        fi
    done
    echo "done"
    exit 0
fi

# ---------------------------------------------------------------------- install

if [[ -z "$TARGET" ]]; then
    TARGET="$(pwd)"
fi
TARGET="$(cd "$TARGET" && pwd)"

if [[ ! -d "$TARGET/.cyberos/memory/store" ]]; then
    echo "error: $TARGET has no .cyberos/memory/store/ — run scripts/install.sh first" >&2
    exit 2
fi

echo "=== installing cyberos LaunchAgents ==="
echo "  target project : $TARGET"
echo "  nightly script : $REPO_ROOT/memory/scripts/automation/cyberos-nightly.sh"
echo "  weekly script  : $REPO_ROOT/memory/scripts/automation/cyberos-weekly.sh"
echo "  agents dir     : $AGENTS_DIR"
echo "  log dir        : ~/Library/Logs/cyberos/"
echo

mkdir -p "$AGENTS_DIR"
mkdir -p "$HOME/Library/Logs/cyberos"

# Resolve python — prefer the one the user installed cyberos into.
PYTHON_BIN="$(command -v python || command -v python3)"

write_plist() {
    local label="$1" script="$2" hour="$3" minute="$4" weekday="${5:-}"
    local plist="$AGENTS_DIR/$label.plist"

    cat > "$plist" <<XML
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>          <string>$label</string>
    <key>RunAtLoad</key>      <false/>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>$script</string>
        <string>$TARGET</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>$(dirname "$PYTHON_BIN"):/usr/local/bin:/usr/bin:/bin</string>
        <key>CYBEROS_PROJECT</key>
        <string>$TARGET</string>
    </dict>
    <key>WorkingDirectory</key><string>$TARGET</string>
    <key>StandardOutPath</key> <string>$HOME/Library/Logs/cyberos/$label.out.log</string>
    <key>StandardErrorPath</key><string>$HOME/Library/Logs/cyberos/$label.err.log</string>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>   <integer>$hour</integer>
        <key>Minute</key> <integer>$minute</integer>$([ -n "$weekday" ] && printf '\n        <key>Weekday</key> <integer>%s</integer>' "$weekday")
    </dict>
</dict>
</plist>
XML
    echo "  wrote: $plist"
}

write_plist "$NIGHTLY_LABEL" \
    "$REPO_ROOT/memory/scripts/automation/cyberos-nightly.sh" \
    1 9

write_plist "$WEEKLY_LABEL" \
    "$REPO_ROOT/memory/scripts/automation/cyberos-weekly.sh" \
    2 7 0

chmod +x \
    "$REPO_ROOT/memory/scripts/automation/cyberos-nightly.sh" \
    "$REPO_ROOT/memory/scripts/automation/cyberos-weekly.sh"

# Load (unload first in case they exist; idempotent).
launchctl unload "$NIGHTLY_PLIST" 2>/dev/null || true
launchctl unload "$WEEKLY_PLIST" 2>/dev/null || true
launchctl load "$NIGHTLY_PLIST"
launchctl load "$WEEKLY_PLIST"

echo
echo "loaded jobs:"
launchctl list | grep "world.cyberskill" || true
echo
echo "next runs:"
echo "  nightly : 01:09 local time (every day)"
echo "  weekly  : 02:07 local time (Sundays)"
echo
echo "to test the nightly job immediately:"
echo "  launchctl start $NIGHTLY_LABEL"
echo "  tail -f ~/Library/Logs/cyberos/nightly.log"
echo
echo "to remove:"
echo "  $0 --uninstall"
