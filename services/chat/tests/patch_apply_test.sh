#!/usr/bin/env bash
# FR-CHAT-001 §4 #10 — patches/ apply in lexicographic order.
#
# This test asserts a structural invariant: every file in services/chat/patches/
# ending in .patch is a valid git-format-patch (it has at minimum the
# `From:` / `Subject:` / `---` separators or the simpler `diff --git` form).
# An empty patches/ directory is also accepted.
#
# We do NOT actually `git am` apply them here (that requires the upstream
# checkout and git context); the Dockerfile build is the integration test.
# Instead we sanity-check shape so a malformed patch surfaces at PR time.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

fail() { echo "::error::$*" >&2; exit 1; }
ok()   { echo "  ✓ $*"; }

PATCHES_DIR="$ROOT/patches"

[[ -d "$PATCHES_DIR" ]] || fail "patches/ directory missing"
ok "patches/ directory present"

shopt -s nullglob
patches=("$PATCHES_DIR"/*.patch)

if [[ ${#patches[@]} -eq 0 ]]; then
    echo "  (no patches present; that is a valid state at slice 1)"
    echo "✓ patch_apply_test: no-op"
    exit 0
fi

ok "Found ${#patches[@]} patch file(s)"

# Verify lexicographic naming convention: NNN-name.patch
for p in "${patches[@]}"; do
    base=$(basename "$p")
    if [[ ! "$base" =~ ^[0-9]{3}-[A-Za-z0-9._-]+\.patch$ ]]; then
        fail "Patch '$base' does not match NNN-name.patch convention"
    fi
done
ok "All patches follow NNN-name.patch convention"

# Verify each patch is shaped like a git-format-patch or diff --git output.
for p in "${patches[@]}"; do
    if ! grep -qE '^(From [0-9a-f]{40}|diff --git)' "$p"; then
        fail "Patch '$(basename $p)' does not look like a git patch (no 'From <sha>' or 'diff --git' line)"
    fi
done
ok "All patches have git-format-patch / diff --git shape"

# Verify no two patches share the same NNN prefix. Keep this bash-3
# compatible for macOS runners.
prefixes=$(mktemp)
for p in "${patches[@]}"; do
    basename "$p" | cut -c1-3 >> "$prefixes"
done
dupe=$(sort "$prefixes" | uniq -d | head -1)
rm -f "$prefixes"
if [[ -n "$dupe" ]]; then
    fail "Two patches share NNN prefix $dupe"
fi
ok "All patch NNN prefixes are unique"

echo "✓ patch_apply_test: ${#patches[@]} patches verified"
