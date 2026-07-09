#!/usr/bin/env bash
#
# scripts/install.sh — bootstrap a target project for CyberOS memory.
#
# Creates `.cyberos/memory/store/` in the target project with the protocol file
# copied inside (self-contained store). Symlinks AGENTS.md from project
# root into the store. Adds .cyberos/memory/store to .gitignore.
#
# The cyberos engine is installed once via `pip install -e .` in the
# source repo; all projects share it.
#
# Usage:
#     /path/to/cyberos/modules/memory/scripts/install.sh [TARGET]
#
#     # Force overwrite existing .cyberos/memory/store/:
#     install.sh --force
#
#     # Skip the agent symlink (if you wire AGENTS.md manually):
#     install.sh --no-agent-symlink
#
#     # Set up host-side automation (macOS only):
#     install.sh --with-automation
#
#     # Install the git pre-commit hook:
#     install.sh --with-pre-commit

set -euo pipefail

# ---------------------------------------------------------------------- args

TARGET=""
WITH_AUTOMATION=0
WITH_PRE_COMMIT=0
NO_AGENT_SYMLINK=0
FORCE=0
AUTO_INDEX=0
AUTO_DIGEST=0
DIGEST_LIMIT=50
SOURCE_REPO="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="$(cd "$SOURCE_REPO/../.." && pwd)"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --with-automation)   WITH_AUTOMATION=1; shift ;;
        --with-pre-commit)   WITH_PRE_COMMIT=1; shift ;;
        --no-agent-symlink)  NO_AGENT_SYMLINK=1; shift ;;
        --force)             FORCE=1; shift ;;
        --auto-index)        AUTO_INDEX=1; shift ;;
        --auto-digest)       AUTO_DIGEST=1; shift ;;
        --digest-limit)      DIGEST_LIMIT="$2"; shift 2 ;;
        -h|--help)
            grep '^#' "$0" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *)
            if [[ -z "$TARGET" ]]; then TARGET="$1"
            else echo "unknown arg: $1" >&2; exit 2; fi
            shift ;;
    esac
done

if [[ -z "$TARGET" ]]; then
    TARGET="$(pwd)"
fi
TARGET="$(cd "$TARGET" && pwd)"

echo "=== cyberos memory install ==="
echo "  source : $SOURCE_REPO"
echo "  target : $TARGET"
echo "  options: $([[ $WITH_AUTOMATION == 1 ]] && echo -n '+automation ')$([[ $WITH_PRE_COMMIT == 1 ]] && echo -n '+pre-commit ')$([[ $NO_AGENT_SYMLINK == 1 ]] && echo -n '+no-agent-symlink ')"
echo

# ---------------------------------------------------------------------- 1. check engine

echo "→ step 1/5: check cyberos engine"
if command -v cyberos >/dev/null 2>&1; then
    echo "  ✓ cyberos CLI found: $(command -v cyberos)"
elif python -c "import cyberos" 2>/dev/null; then
    echo "  ✓ cyberos Python package importable"
else
    echo "  ✗ cyberos engine not found."
    echo "    Install it first:"
    echo "      cd $SOURCE_REPO && pip install -e ."
    exit 1
fi
echo

# ---------------------------------------------------------------------- 2. .cyberos/memory/store skeleton + protocol file

echo "→ step 2/5: initialise .cyberos/memory/store/"
memory="$TARGET/.cyberos/memory/store"

# Build cyberos init command
init_cmd="python -m cyberos --store .cyberos/memory/store init"
if [[ "$FORCE" == "1" ]]; then
    init_cmd="$init_cmd --force"
fi
if [[ "$AUTO_INDEX" == "1" ]]; then
    init_cmd="$init_cmd --auto-index"
fi
if [[ "$AUTO_DIGEST" == "1" ]]; then
    init_cmd="$init_cmd --auto-digest --digest-limit $DIGEST_LIMIT"
fi

cd "$TARGET"
if [[ -d "$memory" && "$FORCE" != "1" ]]; then
    echo "  – $memory already exists; skipping store init (use --force to re-init)"
else
    echo "  Running: $init_cmd"
    if eval "$init_cmd"; then
        echo "  ✓ $memory/ initialized via cyberos init"
    else
        echo "  ⚠ cyberos init failed, falling back to manual initialization"
        mkdir -p "$memory"/{audit,memories/decisions,memories/facts,memories/people,memories/projects,memories/preferences,memories/drift,memories/refinements,meta,company,module,member,client,project,persona,conflicts,exports,index}
        cat > "$memory/manifest.json" <<EOF
{
  "schema_version": 2,
  "project": {
    "root_path": "$TARGET"
  },
  "created_at_ns": $(date +%s)000000000
}
EOF
        # Create HEAD (8-byte LE u64 zeroed)
        python3 -c "import sys; sys.stdout.buffer.write(b'\x00'*8)" > "$memory/HEAD"
        echo "  ✓ $memory/ (manifest.json + HEAD + directory skeleton)"
    fi
fi

# Copy protocol files into the store (self-contained)
cp "$REPO_ROOT/AGENTS.md" "$memory/AGENTS.md"
for f in memory.schema.json memory.invariants.yaml; do
    cp "$SOURCE_REPO/$f" "$memory/$f"
done
echo "  ✓ $memory/AGENTS.md + memory.schema.json + memory.invariants.yaml (protocol files)"
echo

# ---------------------------------------------------------------------- 3. agent symlink

if [[ "$NO_AGENT_SYMLINK" == "0" ]]; then
    echo "→ step 3/5: wire AGENTS.md for your agent"
    cd "$TARGET"
    case "$(uname -s 2>/dev/null || echo unknown)" in
        MINGW*|MSYS*|CYGWIN*) USE_COPY=1 ;;
        *)                    USE_COPY=0 ;;
    esac
    for link_name in AGENTS.md CLAUDE.md; do
        if [[ -e "$link_name" && ! -L "$link_name" && "$FORCE" != "1" ]]; then
            echo "  – $link_name already exists (not a symlink); skipping"
        elif [[ "$USE_COPY" == "1" ]]; then
            rm -f "$link_name"
            cp "$memory/AGENTS.md" "$link_name"
            echo "  ✓ $link_name (copy; on Windows symlinks require dev mode)"
        else
            rm -f "$link_name"
            ln -s .cyberos/memory/store/AGENTS.md "$link_name"
            echo "  ✓ $link_name → .cyberos/memory/store/AGENTS.md"
        fi
    done
    echo
else
    echo "→ step 3/5: agent symlink skipped (--no-agent-symlink)"
    echo
fi

# ---------------------------------------------------------------------- 4. .gitignore

echo "→ step 4/5: .gitignore"
gitignore="$TARGET/.gitignore"
marker=".cyberos/memory/store/"
if [[ -f "$gitignore" ]] && grep -qF "$marker" "$gitignore"; then
    echo "  – $marker already in .gitignore; skipping"
else
    {
        echo ""
        echo "# CyberOS memory store (runtime data, not version-controlled)"
        echo "$marker"
    } >> "$gitignore"
    echo "  ✓ added $marker to .gitignore"
fi
echo

# ---------------------------------------------------------------------- 5. verify

echo "→ step 5/5: verify"
cd "$TARGET"
if python -m cyberos --store .cyberos/memory/store doctor > /tmp/cyberos-install-doctor.log 2>&1; then
    tail -3 /tmp/cyberos-install-doctor.log | sed 's/^/  /'
else
    echo "  ⚠ cyberos doctor failed:"
    sed 's/^/    /' /tmp/cyberos-install-doctor.log
fi
echo

# ---------------------------------------------------------------------- optional automation

if [[ "$WITH_AUTOMATION" == "1" ]]; then
    echo "→ extra: macOS automation (launchd)"
    "$SOURCE_REPO/scripts/automation-install.sh" --target "$TARGET"
    echo
fi

if [[ "$WITH_PRE_COMMIT" == "1" ]]; then
    echo "→ extra: git pre-commit hook"
    "$SOURCE_REPO/scripts/install-pre-commit.sh" "$TARGET"
    echo
fi

echo "=== done ==="
echo
echo "next steps:"
echo "  1. open the project in your agent (Claude Code / Cursor / Cowork)"
echo "  2. AGENTS.md is loaded automatically via the symlink"
echo "  3. the agent will start building $memory/memories/ as you work"
echo
echo "verify anytime with:"
echo "  cd $TARGET"
echo "  python -m cyberos --store .cyberos/memory/store state"
echo "  python -m cyberos --store .cyberos/memory/store doctor"
