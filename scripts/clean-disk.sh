#!/usr/bin/env bash
# Reclaim local disk from regeneratable build artifacts. Everything this removes is gitignored and
# rebuilt on demand; it NEVER touches tracked source. Do NOT wire this into a git hook - cleaning
# target/ forces a full recompile on the next build. Run it occasionally, or schedule it weekly
# (launchd snippet at the bottom of this file).
#
#   bash scripts/clean-disk.sh           # gentle: prune build artifacts older than 15 days (needs cargo-sweep)
#   bash scripts/clean-disk.sh --hard    # full: cargo clean every workspace (biggest reclaim, slowest next build)
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"
MODE="${1:-soft}"

# Each of these is a separate Rust workspace with its own target/.
for w in services modules/skill apps/desktop/src-tauri .; do
  [ -d "$w/target" ] || continue
  before="$(du -sh "$w/target" 2>/dev/null | cut -f1)"
  if [ "$MODE" = "--hard" ]; then
    ( cd "$w" && cargo clean 2>/dev/null ) || rm -rf "$w/target"
    echo "cleaned  $w/target   (was ${before:-?})"
  elif command -v cargo-sweep >/dev/null 2>&1; then
    ( cd "$w" && cargo sweep --time 15 . >/dev/null 2>&1 ) || true
    echo "swept    $w/target   (was ${before:-?}, kept last 15 days)"
  else
    rm -rf "$w/target"
    echo "removed  $w/target   (was ${before:-?})  tip: 'cargo install cargo-sweep' for a gentler prune"
  fi
done

# Python / JS build cruft (all gitignored).
find . -type d \( -name __pycache__ -o -name .pytest_cache -o -name .next \) -prune -exec rm -rf {} + 2>/dev/null || true
echo "pruned   __pycache__ / .pytest_cache / .next"

echo "done. Docker volumes are separate from this folder - run 'docker system df' and"
echo "'docker system prune' if Docker's own disk is also tight."

# --- optional: run weekly via launchd (macOS) ---------------------------------
# Save the plist below to ~/Library/LaunchAgents/world.cyberskill.clean-disk.plist, edit the path,
# then: launchctl load ~/Library/LaunchAgents/world.cyberskill.clean-disk.plist
#
# <?xml version="1.0" encoding="UTF-8"?>
# <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
# <plist version="1.0"><dict>
#   <key>Label</key><string>world.cyberskill.clean-disk</string>
#   <key>ProgramArguments</key>
#     <array><string>/bin/bash</string>
#            <string>/Users/stephencheng/Projects/CyberSkill/cyberos/scripts/clean-disk.sh</string></array>
#   <key>StartCalendarInterval</key><dict><key>Weekday</key><integer>0</integer><key>Hour</key><integer>3</integer></dict>
# </dict></plist>
