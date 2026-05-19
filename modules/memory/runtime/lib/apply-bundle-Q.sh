#!/usr/bin/env bash
# runtime/lib/apply-bundle-Q.sh — completes the §0.5 + §0.6 protocol-upgrade flow
# for Bundle Q (2026-05-11). Run this from your macOS terminal at the project
# repo root, AFTER the AGENTS.md / CHANGELOG / README edits have been made.
#
# This script performs the memory-touching steps that the cowork sandbox
# could not perform (§0.1 forbids the writer from running under /sessions/):
#
#   1. session.start                                                        (op:session.start)
#   2. archive prior AGENTS.md → meta/protocol-history/AGENTS-sha256-<old>.md (op:create)
#   3. write DEC-109                                                        (op:create)
#   4. write REF-041                                                        (op:create)
#   5. protocol-upgrade — manifest.protocol pin update                      (op:protocol_upgrade)
#   6. §8.7 post-upgrade self-audit (auto per §0.5 step 4)                  (op:health_check)
#   7. session.end + final manifest str_replace (audit_chain_head update)   (op:session.end + op:str_replace)
#
# Each step is idempotent within a single run via the MemoryLock and the
# writer's own validators. If the script fails midway, the chain remains
# consistent (no partial mutations); you can fix the cause and rerun.
#
# The §8.7 self-audit at step 6 is the §0.5-mandated post-upgrade
# migration check. CRITICAL findings cause the script to bail before
# session.end (the upgrade is recorded but writes stay frozen until
# repaired in MAINTENANCE mode). WARN/INFO findings are reported but
# don't block.
#
# Verify after with:
#   python3 runtime/lib/memory_writer.py status
#   python3 runtime/lib/memory_writer.py verify --bit-perfect

set -euo pipefail

# ─── Config ──────────────────────────────────────────────────────────────
OLD_SHA="sha256:617f5aef1a49c394f6d17be072c8b29dbeb84c3265b80f3de3cb00a0f1c07759"
NEW_SHA="sha256:71a276c74fe5a1fb65dbe24c6073f74d4cc7168b02aef1b577db9e01ccb13688"
ACTOR="subject:stephen-cheng"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WRITER="$REPO_ROOT/runtime/lib/memory_writer.py"
TPL_DIR="$REPO_ROOT/.cyberos-memory/refinements/2026-05-11-bundle-Q"
ARCHIVE_PATH="meta/protocol-history/AGENTS-${OLD_SHA/:/-}.md"
DEC_PATH="memories/decisions/DEC-109-implementation-files-in-source-tree.md"
REF_PATH="memories/refinements/REF-041-bundle-q-impl-files-and-close-pattern.md"

cd "$REPO_ROOT"

# ─── Sanity checks ───────────────────────────────────────────────────────
echo "── Bundle Q apply script ──"
echo "Repo root: $REPO_ROOT"
echo

if [ ! -f "$WRITER" ]; then
    echo "FATAL: writer not found at $WRITER" >&2
    exit 2
fi

if [ ! -d ".cyberos-memory" ]; then
    echo "FATAL: .cyberos-memory/ not found at $REPO_ROOT" >&2
    exit 2
fi

# Preflight Python dependencies — fail before any chain mutations
if ! python3 -c "import rfc8785, yaml" 2>/dev/null; then
    echo "FATAL: missing Python dependencies for the writer." >&2
    echo "  Required: rfc8785 (RFC 8785 JCS), PyYAML." >&2
    echo "  Install with:" >&2
    echo "    python3 -m pip install rfc8785 PyYAML --break-system-packages" >&2
    echo "  (macOS Python 3.11+ enforces PEP 668; the --break-system-packages" >&2
    echo "   flag is required when installing into the system Python.)" >&2
    exit 2
fi
echo "✓ Python deps present (rfc8785, PyYAML)"

# Verify new AGENTS.md canonical SHA matches the approved target
COMPUTED_NEW=$(python3 - <<'PY'
import hashlib, unicodedata
with open('docs/CyberOS-AGENTS.md','rb') as f: data = f.read()
BOM = b'\xef\xbb\xbf'
if data.startswith(BOM): data = data[3:]
data = data.replace(BOM, b'')
text = data.decode('utf-8').replace('\r\n','\n').replace('\r','\n')
text = unicodedata.normalize('NFC', text)
lines = [l.rstrip() for l in text.split('\n')]
while lines and lines[-1] == '': lines.pop()
print('sha256:' + hashlib.sha256(('\n'.join(lines)+'\n').encode('utf-8')).hexdigest())
PY
)

if [ "$COMPUTED_NEW" != "$NEW_SHA" ]; then
    echo "FATAL: AGENTS.md canonical SHA mismatch." >&2
    echo "  expected: $NEW_SHA"  >&2
    echo "  computed: $COMPUTED_NEW"  >&2
    echo "Refusing to upgrade — re-edit AGENTS.md or re-run with the matching SHA." >&2
    exit 2
fi
echo "✓ AGENTS.md canonical SHA matches approved target ($NEW_SHA)"

# Recover prior AGENTS.md by walking git history backwards until the canonical
# SHA matches the OLD pin. This handles three cases robustly:
#   - edits not yet committed (HEAD has prior content)
#   - bundle committed in one commit (HEAD~1 has prior content)
#   - bundle split across multiple commits (HEAD~N has prior content)
PRIOR_TMP="$(mktemp -t agents-prior.XXXXXX).md"

canonical_sha_of_blob() {
    local rev="$1"
    git show "$rev:docs/CyberOS-AGENTS.md" 2>/dev/null > "$PRIOR_TMP" || return 1
    python3 - <<PY
import hashlib, unicodedata
with open('$PRIOR_TMP','rb') as f: data = f.read()
BOM = b'\xef\xbb\xbf'
if data.startswith(BOM): data = data[3:]
data = data.replace(BOM, b'')
text = data.decode('utf-8').replace('\r\n','\n').replace('\r','\n')
text = unicodedata.normalize('NFC', text)
lines = [l.rstrip() for l in text.split('\n')]
while lines and lines[-1] == '': lines.pop()
print('sha256:' + hashlib.sha256(('\n'.join(lines)+'\n').encode('utf-8')).hexdigest())
PY
}

FOUND_REV=""
for n in 0 1 2 3 4 5 6 7 8 9 10; do
    REV="HEAD~$n"
    [ "$n" -eq 0 ] && REV="HEAD"
    if ! git rev-parse "$REV" >/dev/null 2>&1; then
        break
    fi
    SHA=$(canonical_sha_of_blob "$REV" 2>/dev/null) || continue
    if [ "$SHA" = "$OLD_SHA" ]; then
        FOUND_REV="$REV"
        break
    fi
done

if [ -z "$FOUND_REV" ]; then
    echo "FATAL: could not locate the pre-Q AGENTS.md in git history." >&2
    echo "  expected (manifest pin): $OLD_SHA" >&2
    echo "  walked HEAD..HEAD~10 — no canonical SHA match" >&2
    echo "Possible causes:" >&2
    echo "  - the memory's manifest pin doesn't reflect what was actually" >&2
    echo "    committed before Bundle Q (rare; chain corruption);" >&2
    echo "  - Bundle Q's commits are deeper than HEAD~10 (unlikely)." >&2
    echo "Manual recovery: find the commit pre-edit, run" >&2
    echo "  git show <commit>:docs/CyberOS-AGENTS.md > /tmp/prior.md" >&2
    echo "  python3 runtime/lib/memory_writer.py write $ACTOR \\" >&2
    echo "    meta/protocol-history/AGENTS-${OLD_SHA/:/-}.md /tmp/prior.md" >&2
    rm -f "$PRIOR_TMP"
    exit 2
fi
echo "✓ Prior AGENTS.md (from git $FOUND_REV) matches old pin ($OLD_SHA)"
echo "✓ Prior AGENTS.md (from git HEAD) matches old pin ($OLD_SHA)"

# ─── Helper: render a memory template with fresh memory_id + ts ──────────
render_template() {
    local tpl="$1"
    local out="$2"
    python3 - <<PY
import datetime, secrets, time
from zoneinfo import ZoneInfo

# UUIDv7 generation (RFC 9562 layout)
ts_ms = int(time.time() * 1000) & ((1 << 48) - 1)
rand_a = secrets.randbits(12)
rand_b = secrets.randbits(62)
n = ts_ms << 80
n |= 0x7 << 76
n |= rand_a << 64
n |= 0b10 << 62
n |= rand_b
hex_str = f'{n:032x}'
mem_id = f'mem_{hex_str[0:8]}-{hex_str[8:12]}-{hex_str[12:16]}-{hex_str[16:20]}-{hex_str[20:32]}'

# ISO-8601 with offset, second precision, in manifest's tz
import json
with open('.cyberos-memory/manifest.json') as f:
    m = json.load(f)
tz = ZoneInfo(m.get('timezone') or 'UTC')
iso_ts = datetime.datetime.now(tz).replace(microsecond=0).isoformat()

with open('$tpl') as f: tpl_text = f.read()
out = tpl_text.replace('__MEM_ID__', mem_id).replace('__ISO_TS__', iso_ts)
with open('$out', 'w') as f: f.write(out)
print(f'rendered {mem_id} at {iso_ts}')
PY
}

# ─── 1. session-start ────────────────────────────────────────────────────
echo
echo "── 1. session.start ──"
python3 "$WRITER" session-start "$ACTOR"

# ─── 2. archive prior AGENTS.md ──────────────────────────────────────────
echo
echo "── 2. archive prior AGENTS.md → $ARCHIVE_PATH ──"
python3 "$WRITER" write "$ACTOR" "$ARCHIVE_PATH" "$PRIOR_TMP"

# ─── 3. write DEC-109 ────────────────────────────────────────────────────
echo
echo "── 3. write DEC-109 ──"
DEC_RENDERED="$(mktemp -t dec109.XXXXXX).md"
render_template "$TPL_DIR/DEC-109-implementation-files-in-source-tree.md.tpl" "$DEC_RENDERED"
python3 "$WRITER" write "$ACTOR" "$DEC_PATH" "$DEC_RENDERED"

# ─── 4. write REF-041 ────────────────────────────────────────────────────
echo
echo "── 4. write REF-041 ──"
REF_RENDERED="$(mktemp -t ref041.XXXXXX).md"
render_template "$TPL_DIR/REF-041-bundle-q-impl-files-and-close-pattern.md.tpl" "$REF_RENDERED"
python3 "$WRITER" write "$ACTOR" "$REF_PATH" "$REF_RENDERED"

# ─── 5. protocol-upgrade — manifest pin update ───────────────────────────
echo
echo "── 5. protocol-upgrade — manifest.protocol pin ──"
python3 "$WRITER" protocol-upgrade "$ACTOR" "$OLD_SHA" "$NEW_SHA" \
    --reason "Approve protocol upgrade $OLD_SHA → $NEW_SHA per §0.5; approved by $ACTOR in chat (Bundle Q: §0.6 implementation-files clause, §4.7 post-terminator close exemption, §13.1 memory-not-versioned warn, §15 relative-symlink rule)."

# ─── 6. §8.7 post-upgrade self-audit (§0.5 step 4 auto-trigger) ──────────
echo
echo "── 6. §8.7 post-upgrade self-audit ──"
set +e
python3 "$WRITER" self-audit "$ACTOR" --post-upgrade
SA_STATUS=$?
set -e
if [ "$SA_STATUS" -eq 2 ]; then
    echo
    echo "✗ §8.7 self-audit reported CRITICAL findings — halting before session.end."  >&2
    echo "  The protocol-upgrade row landed; writes will remain frozen until"  >&2
    echo "  repaired in MAINTENANCE mode (§8.8). Review the latest report at"  >&2
    echo "  .cyberos-memory/meta/health/, address findings, then run:"  >&2
    echo "    python3 $WRITER session-end $ACTOR"  >&2
    rm -f "$PRIOR_TMP" "$DEC_RENDERED" "$REF_RENDERED"
    exit 2
fi
echo "✓ Self-audit clean (no CRITICAL). Proceeding to session.end."

# ─── 7. session.end + final manifest str_replace ─────────────────────────
echo
echo "── 7. session.end + final manifest str_replace ──"
python3 "$WRITER" session-end "$ACTOR"

# ─── Cleanup ─────────────────────────────────────────────────────────────
rm -f "$PRIOR_TMP" "$DEC_RENDERED" "$REF_RENDERED"

echo
echo "── done ──"
echo "Bundle Q applied. Verifying the chain is healthy:"
python3 "$WRITER" verify --bit-perfect
echo
echo "If 'LINK invariant breaks: 0' above and the manifest pin matches"
echo "$NEW_SHA, you're done. Commit the bundle as one logical unit:"
echo "  git add .gitignore AGENTS.md docs/ runtime/ .cyberos-memory/cache/"
echo "  git commit -m 'Bundle Q: §0.6 + §4.7 + §13.1 + §15 amendments'"
