#!/usr/bin/env bash
#
# scripts/automation-install-linux.sh — Linux systemd-user automation.
#
# Installs two systemd --user units:
#   - cyberos-nightly.service + .timer  → daily 01:09 local
#   - cyberos-weekly.service  + .timer  → Sundays 02:07 local
#
# Falls back to a crontab entry if systemd --user isn't available
# (e.g. headless containers, WSL1, very old distros).
#
# Logs land at ~/.local/state/cyberos/{nightly,weekly}.log .
#
# Usage:
#     ./scripts/automation-install-linux.sh --target <project>
#     ./scripts/automation-install-linux.sh --uninstall

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TARGET=""
ACTION="install"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)    TARGET="$2"; shift 2 ;;
        --uninstall) ACTION="uninstall"; shift ;;
        -h|--help)   grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
done

NIGHTLY_UNIT="cyberos-nightly"
WEEKLY_UNIT="cyberos-weekly"
SYSTEMD_DIR="$HOME/.config/systemd/user"
LOG_DIR="$HOME/.local/state/cyberos"
mkdir -p "$LOG_DIR"

has_systemd_user() {
    command -v systemctl >/dev/null 2>&1 && \
        systemctl --user is-active --quiet default.target 2>/dev/null
}

# ---------------------------------------------------------------------- uninstall

if [[ "$ACTION" == "uninstall" ]]; then
    echo "=== uninstalling Linux automation ==="
    if has_systemd_user; then
        for unit in "$NIGHTLY_UNIT" "$WEEKLY_UNIT"; do
            systemctl --user disable --now "${unit}.timer" 2>/dev/null || true
            rm -f "$SYSTEMD_DIR/${unit}.service" "$SYSTEMD_DIR/${unit}.timer"
            echo "  removed: $SYSTEMD_DIR/${unit}.{service,timer}"
        done
        systemctl --user daemon-reload || true
    else
        echo "  systemd --user unavailable; uninstalling crontab entries"
        crontab -l 2>/dev/null \
            | grep -v "# cyberos-automation-marker$" \
            | crontab - 2>/dev/null || true
    fi
    echo "done"
    exit 0
fi

# ---------------------------------------------------------------------- install

if [[ -z "$TARGET" ]]; then TARGET="$(pwd)"; fi
TARGET="$(cd "$TARGET" && pwd)"
if [[ ! -d "$TARGET/.cyberos-memory" ]]; then
    echo "error: $TARGET has no .cyberos-memory/" >&2
    exit 2
fi

PYTHON_BIN="$(command -v python || command -v python3)"
PYTHON_DIR="$(dirname "$PYTHON_BIN")"

chmod +x "$REPO_ROOT/memory/scripts/automation/cyberos-nightly.sh" \
         "$REPO_ROOT/memory/scripts/automation/cyberos-weekly.sh"

if has_systemd_user; then
    echo "=== installing systemd --user units ==="
    mkdir -p "$SYSTEMD_DIR"

    cat > "$SYSTEMD_DIR/${NIGHTLY_UNIT}.service" <<UNIT
[Unit]
Description=CyberOS nightly soak (doctor + consolidate dry-run)

[Service]
Type=oneshot
Environment=PATH=${PYTHON_DIR}:/usr/local/bin:/usr/bin:/bin
Environment=CYBEROS_PROJECT=${TARGET}
ExecStart=/bin/bash ${REPO_ROOT}/memory/scripts/automation/cyberos-nightly.sh ${TARGET}
WorkingDirectory=${TARGET}
UNIT

    cat > "$SYSTEMD_DIR/${NIGHTLY_UNIT}.timer" <<UNIT
[Unit]
Description=CyberOS nightly soak — daily 01:09 local

[Timer]
OnCalendar=*-*-* 01:09:00
Persistent=true
RandomizedDelaySec=120

[Install]
WantedBy=timers.target
UNIT

    cat > "$SYSTEMD_DIR/${WEEKLY_UNIT}.service" <<UNIT
[Unit]
Description=CyberOS weekly maintenance (backup + consolidate + determinism)

[Service]
Type=oneshot
Environment=PATH=${PYTHON_DIR}:/usr/local/bin:/usr/bin:/bin
Environment=CYBEROS_PROJECT=${TARGET}
ExecStart=/bin/bash ${REPO_ROOT}/memory/scripts/automation/cyberos-weekly.sh ${TARGET}
WorkingDirectory=${TARGET}
UNIT

    cat > "$SYSTEMD_DIR/${WEEKLY_UNIT}.timer" <<UNIT
[Unit]
Description=CyberOS weekly maintenance — Sundays 02:07 local

[Timer]
OnCalendar=Sun *-*-* 02:07:00
Persistent=true
RandomizedDelaySec=300

[Install]
WantedBy=timers.target
UNIT

    systemctl --user daemon-reload
    systemctl --user enable --now "${NIGHTLY_UNIT}.timer" "${WEEKLY_UNIT}.timer"
    echo "  enabled timers:"
    systemctl --user list-timers "${NIGHTLY_UNIT}.timer" "${WEEKLY_UNIT}.timer" --no-pager
    echo
    echo "to run one immediately:"
    echo "  systemctl --user start ${NIGHTLY_UNIT}.service"
    echo "  journalctl --user -u ${NIGHTLY_UNIT}.service -n 50"
else
    echo "=== systemd --user unavailable; installing crontab entries ==="
    crontab_existing=$(crontab -l 2>/dev/null || true)
    new_entries=$(cat <<CRON
9 1 * * *     CYBEROS_PROJECT=${TARGET} ${REPO_ROOT}/memory/scripts/automation/cyberos-nightly.sh ${TARGET}    # cyberos-automation-marker
7 2 * * 0     CYBEROS_PROJECT=${TARGET} ${REPO_ROOT}/memory/scripts/automation/cyberos-weekly.sh ${TARGET}    # cyberos-automation-marker
CRON
    )
    {
        echo "$crontab_existing" | grep -v "# cyberos-automation-marker$" || true
        echo "$new_entries"
    } | crontab -
    echo "  crontab entries installed; review with: crontab -l"
fi

echo
echo "logs land in: $LOG_DIR"
echo "to remove: $0 --uninstall"
