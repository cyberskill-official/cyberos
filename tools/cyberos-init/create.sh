#!/usr/bin/env bash
# create.sh <new-project-dir> - scaffold a fresh CyberOS project from the template skeleton,
# then run init.sh (agent surface, gates autodetect, backlog, BRAIN). git-inits if needed.
# Also the content of the "GitHub template repo" channel: host template/ as a repo and click
# "Use this template", then run init.sh once. Never clobbers files already present in the target.
set -euo pipefail

src="$(cd "$(dirname "$0")" && pwd)"
dst="${1:?usage: bash create.sh <new-project-dir>}"
mkdir -p "$dst"; dst="$(cd "$dst" && pwd)"

[ -d "$dst/.git" ] || git init -q "$dst" 2>/dev/null || true

# seed the skeleton without overwriting anything already there
if [ -d "$src/template" ]; then
  cp -Rn "$src/template/." "$dst/" 2>/dev/null || cp -R -n "$src/template/." "$dst/" 2>/dev/null || {
    # portable no-clobber fallback for cp implementations lacking -n
    ( cd "$src/template" && find . -type f | while read -r f; do [ -e "$dst/$f" ] || { mkdir -p "$dst/$(dirname "$f")"; cp "$f" "$dst/$f"; }; done )
  }
fi

# fill the {{PROJECT}} placeholder in the freshly-seeded README (basename has no slashes)
proj="$(basename "$dst")"
if [ -f "$dst/README.md" ] && grep -q '{{PROJECT}}' "$dst/README.md" 2>/dev/null; then
  sed "s/{{PROJECT}}/$proj/g" "$dst/README.md" > "$dst/README.md.tmp" && mv "$dst/README.md.tmp" "$dst/README.md"
fi

bash "$src/init.sh" "$dst"
echo "cyberos create: new project ready at $dst"
