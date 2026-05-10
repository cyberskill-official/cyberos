#!/usr/bin/env bash
# outputs/apply-bundle-Q.sh — completes the §0.5 + §0.6 protocol-upgrade flow
# for Bundle Q (2026-05-11). Run this from your macOS terminal at the project
# repo root, AFTER the AGENTS.md / CHANGELOG / README edits have been made.
#
# This script performs the BRAIN-touching steps that the cowork sandbox
# could not perform (§0.1 forbids the writer from running under /sessions/):
#
#   1. session.start                                                        (op:session.start)
#   2. archive prior AGENTS.md → meta/protocol-history/AGENTS-sha256-<old>.md (op:create)
#   3. write DEC-109                                                        (op:create)
#   4. write REF-041                                                        (op:create)
#   5. protocol-upgrade — manifest.protocol pin update                      (op:protocol_upgrade)
#   6. session.end + final manifest str_replace (audit_chain_head update)    (op:session.end + op:str_replace)
#
# Each step is idempotent within a single run via the BrainLock and the
# writer's own validators. If the script fails midway, the chain remains
# consistent (no partial mutations); you can fix the cause and rerun.
#
# Verify after with:
#   python3 outputs/brain_writer.py status
#   python3 outputs/brain_writer.py verify --bit-perfect

set -euo pipefail

# ─── Config ──────────────────────────────────────────────────────────────
OLD_SHA="sha256:617f5aef1a49c394f6d17be072c8b29dbeb84c3265b80f3de3cb00a0f1c07759"
NEW_SHA="sha256:71a276c74fe5a1fb65dbe24c6073f74d4cc7168b02aef1b577db9e01ccb13688"
ACTOR="subject:stephen-cheng"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WRITER="$REPO_ROOT/outputs/brain_writer.py"
TPL_DIR="$REPO_ROOT/outputs/refinements/2026-05-11-bundle-Q"
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

# Recover prior AGENTS.md from git HEAD (verify SHA matches old pin)
PRIOR_TMP="$(mktemp -t agents-prior.XXXXXX).md"
git show HEAD:docs/CyberOS-AGENTS.md > "$PRIOR_TMP"
COMPUTED_OLD=$(python3 - <<PY
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
)
if [ "$COMPUTED_OLD" != "$OLD_SHA" ]; then
    echo "FATAL: git HEAD AGENTS.md SHA mismatch." >&2
    echo "  expected (manifest pin): $OLD_SHA" >&2
    echo "  computed (git HEAD):     $COMPUTED_OLD" >&2
    echo "This usually means you committed the AGENTS.md edits already; the apply"  >&2
    echo "script needs the PRE-edit version. Run: git show HEAD~1:docs/CyberOS-AGENTS.md" >&2
    rm -f "$PRIOR_TMP"
    exit 2
fi
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
    --reason "Approve protocol upgrade $OLD_SHA → $NEW_SHA per §0.5; approved by $ACTOR in chat (Bundle Q: §0.6 implementation-files clause, §4.7 post-terminator close exemption, §13.1 BRAIN-not-versioned warn, §15 relative-symlink rule)."

# ─── 6. session.end + final manifest str_replace ─────────────────────────
echo
echo "── 6. session.end + final manifest str_replace ──"
python3 "$WRITER" session-end "$ACTOR"

# ─── Cleanup ─────────────────────────────────────────────────────────────
rm -f "$PRIOR_TMP" "$DEC_RENDERED" "$REF_RENDERED"

echo
echo "── done ──"
echo "Bundle Q applied. Next step: verify the chain is healthy."
echo "  python3 outputs/brain_writer.py status"
echo "  python3 outputs/brain_writer.py verify --bit-perfect"
echo
echo "If verify reports 'LINK invariant breaks: 0' and the chain head matches"
echo "the manifest, you're done. The post-upgrade §8.7 self-audit is a "
echo "follow-up: run it manually when you're ready for a deeper sweep."
