#!/usr/bin/env bash
# bootstrap.sh — commit all staged memories into BRAIN via brain_writer.py
#
# Discovered signature: python3 outputs/brain_writer.py write <actor> <relpath> <content_file>
#
# Run from cyberos repo root:
#   cd ~/Projects/CyberSkill/cyberos
#   bash outputs/staged-memories/bootstrap.sh
#
# Set CYBEROS_HOST_MOUNT_PREFIX env var if running under Cowork sandbox mount.

set -euo pipefail

REPO_ROOT="${1:-$(pwd)}"
STAGED_DIR="$REPO_ROOT/outputs/staged-memories"
ACTOR="${CYBEROS_ACTOR:-agent:claude-sonnet-4.7}"

echo "═══════════════════════════════════════════════════════"
echo "  CyberOS Bootstrap — Seed memories (Aspect 4.3-4.6)"
echo "═══════════════════════════════════════════════════════"
echo "Repo root:     $REPO_ROOT"
echo "Staged dir:    $STAGED_DIR"
echo "Actor:         $ACTOR"
echo "Host mount:    ${CYBEROS_HOST_MOUNT_PREFIX:-(none)}"
echo ""

# Verify protocol pin (§0.5 sanity)
PINNED=$(python3 -c "import json; print(json.load(open('$REPO_ROOT/.cyberos-memory/manifest.json'))['protocol']['sha256'])")
LIVE=$(python3 "$REPO_ROOT/runtime/tools/canonical_sha.py" "$REPO_ROOT/docs/CyberOS-AGENTS.md" 2>/dev/null | tail -1)
if [ "$PINNED" != "$LIVE" ]; then
  echo "✗ Protocol pin mismatch: pinned=$PINNED live=$LIVE"
  exit 1
fi
echo "✓ Protocol pin verified"
echo ""

# session.start
echo "→ session.start"
python3 "$REPO_ROOT/outputs/brain_writer.py" session-start "$ACTOR" 2>&1 | tail -1

# Find next available NNN per bucket
next_nnn() {
  local bucket=$1
  local dir="$REPO_ROOT/.cyberos-memory/memories/$bucket"
  local max=0
  if [ -d "$dir" ]; then
    for f in "$dir"/*.md; do
      [ -f "$f" ] || continue
      local n=$(basename "$f" | grep -oE '^[A-Z]+-([0-9]+)' | grep -oE '[0-9]+$')
      [ -n "$n" ] && [ "$((10#$n))" -gt "$((10#$max))" ] && max=$n
    done
  fi
  printf "%03d" $((10#$max + 1))
}

written=0
skipped=0
failed=0

for staged in "$STAGED_DIR"/facts/*.md "$STAGED_DIR"/people/*.md "$STAGED_DIR"/preferences/*.md; do
  [ -f "$staged" ] || continue
  basename=$(basename "$staged")
  if   [[ "$basename" == FACT-* ]];   then bucket=facts;       prefix=FACT
  elif [[ "$basename" == PERSON-* ]]; then bucket=people;      prefix=PERSON
  elif [[ "$basename" == PREF-* ]];   then bucket=preferences; prefix=PREF
  else
    echo "  ✗ unknown prefix: $basename"
    failed=$((failed+1))
    continue
  fi

  nnn=$(next_nnn "$bucket")
  slug=$(echo "$basename" | sed -E "s/^${prefix}-[0-9]+-//;s/\.md$//")
  target_rel="memories/$bucket/${prefix}-${nnn}-${slug}.md"
  target_abs="$REPO_ROOT/.cyberos-memory/$target_rel"

  if [ -f "$target_abs" ]; then
    echo "  · skip (exists): $target_rel"
    skipped=$((skipped+1))
    continue
  fi

  # brain_writer write <actor> <relpath> <content_file>
  if python3 "$REPO_ROOT/outputs/brain_writer.py" write "$ACTOR" "$target_rel" "$staged" 2>&1 | tail -1; then
    echo "  ✓ $target_rel"
    written=$((written+1))
  else
    echo "  ✗ FAILED: $target_rel"
    failed=$((failed+1))
  fi
done

# session.end (no manifest-bump subcommand — brain_writer auto-bumps via self-audit)
echo ""
echo "→ session.end"
python3 "$REPO_ROOT/outputs/brain_writer.py" session-end "$ACTOR" 2>&1 | tail -1

echo ""
echo "═══════════════════════════════════════════════════════"
echo "  Bootstrap complete: written=$written  skipped=$skipped  failed=$failed"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "→ cyberos status"
python3 "$REPO_ROOT/runtime/tools/cyberos" status 2>&1 | head -22
