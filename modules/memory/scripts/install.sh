#!/usr/bin/env bash
#
# scripts/install.sh — drop the cyberos protocol into a fresh project.
#
# One command bootstraps a new project from zero to "AGENTS.md loaded
# by the agent, .cyberos-memory/ ready to write, doctor reports READY".
#
# Usage:
#     # From inside the target project root:
#     curl -fsSL https://raw.githubusercontent.com/.../scripts/install.sh | bash
#
#     # Or, after cloning the cyberos repo, point install at any project:
#     /path/to/cyberos/scripts/install.sh ~/Projects/my-other-project
#
#     # Skip the agent symlink (if you wire AGENTS.md manually):
#     install.sh --no-agent-symlink
#
#     # Set up host-side automation at the same time (macOS only):
#     install.sh --with-automation
#
#     # Install the git pre-commit hook too:
#     install.sh --with-pre-commit

set -euo pipefail

# ---------------------------------------------------------------------- args

TARGET=""
WITH_AUTOMATION=0
WITH_PRE_COMMIT=0
NO_AGENT_SYMLINK=0
FORCE=0
SOURCE_REPO="$(cd "$(dirname "$0")/../.." && pwd)"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --with-automation)   WITH_AUTOMATION=1; shift ;;
        --with-pre-commit)   WITH_PRE_COMMIT=1; shift ;;
        --no-agent-symlink)  NO_AGENT_SYMLINK=1; shift ;;
        --force)             FORCE=1; shift ;;
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

echo "=== cyberos install ==="
echo "  source : $SOURCE_REPO"
echo "  target : $TARGET"
echo "  options: $([[ $WITH_AUTOMATION == 1 ]] && echo -n '+automation ')$([[ $WITH_PRE_COMMIT == 1 ]] && echo -n '+pre-commit ')$([[ $NO_AGENT_SYMLINK == 1 ]] && echo -n '+no-agent-symlink ')"
echo

# ---------------------------------------------------------------------- 1. python deps

echo "→ step 1/6: python dependencies"
PIP_FLAGS=""
if python -c "import sys; sys.exit(0 if sys.version_info >= (3,11) else 1)" 2>/dev/null; then
    PIP_FLAGS="--break-system-packages"
fi
python -m pip install $PIP_FLAGS --quiet \
    msgspec cryptography crc32c rfc8785 'pyyaml>=6' jsonschema zstandard \
    || { echo "pip install failed; see above"; exit 1; }
echo "  ✓ msgspec, cryptography, crc32c, rfc8785, pyyaml, jsonschema, zstandard"
echo

# ---------------------------------------------------------------------- 2. pandoc (optional)

echo "→ step 2/6: pandoc (optional, for PRD/SRS docx ↔ md round-trip)"
if command -v pandoc >/dev/null 2>&1; then
    echo "  ✓ pandoc $(pandoc --version | head -1 | awk '{print $2}')"
else
    echo "  – pandoc not found; run \`brew install pandoc\` if you need docx conversion"
fi
echo

# ---------------------------------------------------------------------- 3. protocol files

echo "→ step 3/6: install protocol files"
mkdir -p "$TARGET/memory/docs"
for f in AGENTS.md INTEROP.md memory.schema.json memory.invariants.yaml; do
    src="$SOURCE_REPO/memory/docs/$f"
    dst="$TARGET/memory/docs/$f"
    if [[ -f "$dst" && "$FORCE" != "1" ]]; then
        echo "  – $f already exists (use --force to overwrite)"
    else
        cp "$src" "$dst"
        echo "  ✓ $dst"
    fi
done
# Copy the cyberos Python package alongside so `python -m cyberos` works
if [[ ! -d "$TARGET/memory/cyberos" || "$FORCE" == "1" ]]; then
    mkdir -p "$TARGET/memory"
    cp -r "$SOURCE_REPO/memory/cyberos" "$TARGET/memory/cyberos"
    echo "  ✓ $TARGET/memory/cyberos/"
fi
echo

# ---------------------------------------------------------------------- 4. .cyberos-memory skeleton

echo "→ step 4/6: initialise .cyberos-memory/"
memory="$TARGET/.cyberos-memory"
if [[ -d "$memory" && "$FORCE" != "1" ]]; then
    echo "  – $memory already exists; skipping (use --force to re-init)"
else
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
    echo "  ✓ $memory/manifest.json (schema_version=2)"
fi
echo

# ---------------------------------------------------------------------- 5. agent symlink

if [[ "$NO_AGENT_SYMLINK" == "0" ]]; then
    echo "→ step 5/6: wire AGENTS.md for your agent"
    cd "$TARGET"
    # Symlinks work on macOS/Linux; on Git Bash / MSYS / Cygwin under
    # Windows the symlink may degrade to a copy unless dev mode is on.
    # Detect Windows and copy instead so the end-state is the same.
    case "$(uname -s 2>/dev/null || echo unknown)" in
        MINGW*|MSYS*|CYGWIN*) USE_COPY=1 ;;
        *)                    USE_COPY=0 ;;
    esac
    for link_name in AGENTS.md CLAUDE.md; do
        if [[ -e "$link_name" && "$FORCE" != "1" ]]; then
            echo "  – $link_name already exists; skipping"
        elif [[ "$USE_COPY" == "1" ]]; then
            rm -f "$link_name"
            cp memory/docs/AGENTS.md "$link_name"
            echo "  ✓ $link_name (copy; on Windows symlinks require dev mode)"
        else
            rm -f "$link_name"
            ln -s memory/docs/AGENTS.md "$link_name"
            echo "  ✓ $link_name → memory/docs/AGENTS.md"
        fi
    done
    echo
else
    echo "→ step 5/6: agent symlink skipped (--no-agent-symlink)"
    echo
fi

# ---------------------------------------------------------------------- 6. verify

echo "→ step 6/6: verify"
cd "$TARGET"
if python -m cyberos --store .cyberos-memory doctor > /tmp/cyberos-install-doctor.log 2>&1; then
    tail -3 /tmp/cyberos-install-doctor.log | sed 's/^/  /'
else
    echo "  ⚠ cyberos doctor failed:"
    sed 's/^/    /' /tmp/cyberos-install-doctor.log
fi
echo

# ---------------------------------------------------------------------- optional automation

if [[ "$WITH_AUTOMATION" == "1" ]]; then
    echo "→ extra: macOS automation (launchd)"
    "$SOURCE_REPO/memory/scripts/automation-install.sh" --target "$TARGET"
    echo
fi

if [[ "$WITH_PRE_COMMIT" == "1" ]]; then
    echo "→ extra: git pre-commit hook"
    "$SOURCE_REPO/memory/scripts/install-pre-commit.sh" "$TARGET"
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
echo "  python -m cyberos --store .cyberos-memory state"
echo "  python -m cyberos --store .cyberos-memory doctor"
