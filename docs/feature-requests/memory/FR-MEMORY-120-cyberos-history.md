---
id: FR-MEMORY-120
title: "memory history — `cyberos history <path>` surfaces per-file version + attribution from the audit chain; REST `/api/v2/memories/<path>/history`; dream-applied + session_id annotations rendered inline"
module: memory
priority: SHOULD
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_frs: [FR-MEMORY-108, FR-MEMORY-112, FR-MEMORY-113, FR-MEMORY-115, FR-MEMORY-119]
depends_on: []
blocks: []

source_pages:
  - playground/extracts/memory-and-dreaming.transcript.txt  # see "version history" segment [499..540]
source_decisions:
  - DEC-260 (history is read-only — pure projection over existing audit-chain rows; no new audit rows emitted by history operations)
  - DEC-261 (Default rendering is the most-recent-first chronology; older-first available via --chronological; both render the SAME data, different orders)
  - DEC-262 (Diff rendering for body changes uses unified diff format by default; structured JSON via --json; both backed by the same per-version snapshot)
  - DEC-263 (For paths that have undergone moves, history follows the move chain — `cyberos history dst.md` shows entries from both src.md and dst.md since the move)

language: python 3.10
service: modules/memory/cyberos/
new_files:
  - modules/memory/cyberos/core/history.py
  - modules/memory/cyberos/cli/history.py
  - modules/memory/tests/core/test_history.py
  - modules/memory/tests/core/test_history.py
modified_files:
  - modules/memory/cyberos/__main__.py            # wire `cyberos history <path>` subcommand
  - modules/memory/cyberos/core/serve.py          # add GET /api/v2/memories/<path>/history endpoint
allowed_tools:
  - file_read: modules/memory/**
  - file_write: modules/memory/cyberos/**, modules/memory/tests/**
  - bash: cd modules/memory && python -m pytest tests/test_history_*.py -v
  - bash: cd modules/memory && python -m cyberos history memories/facts/x.md
disallowed_tools:
  - emit any new audit row from history operations (per §1 #1, DEC-260 — pure read-only projection)
  - modify any memory file as a side-effect of history rendering (per §1 #1)

effort_hours: 8
sub_tasks:
  - "0.5h: cyberos/core/history.py — `HistoryEntry` dataclass (seq, ts, kind, actor, body_hash, body_diff?, frontmatter_diff?, extra)"
  - "2.0h: cyberos/core/history.py — `walk(path, follow_moves=True) -> list[HistoryEntry]`; iterates audit chain backward, collects rows touching this path (or its prior move-source(s))"
  - "1.0h: diff computation — for adjacent put rows on the same path, compute unified diff of body bytes + structured frontmatter diff"
  - "1.5h: cyberos/cli/history.py — `cyberos history <path> [--limit 10] [--chronological] [--json] [--no-follow-moves] [--show-body]`"
  - "1.0h: cyberos/core/serve.py — REST endpoint with same parameters as CLI; returns JSON"
  - "1.0h: modules/memory/tests/core/test_history.py — 12 cases (single-write, multi-write, diff between versions, tombstone shown, dream-applied annotation, session_id annotation, importance-scored annotation, JSON format, limit cap, chronological order)"
  - "1.0h: modules/memory/tests/core/test_history.py — 8 cases (move-once history, move-chain history, --no-follow-moves boundary, deleted-then-recreated, never-existed path, history for tombstoned path)"
  - "0.5h: integration test against seeded memory with all FR-MEMORY-112..119 row kinds present"
  - "0.5h: CHANGELOG entry + sample-usage doc"
risk_if_skipped: "Without `cyberos history`, the audit chain's rich per-path provenance is INVISIBLE to operators and agents. Two failure modes follow: (1) Debugging 'why does this memory say X?' requires manually scanning the raw audit binlog — operationally unusable. (2) FR-MEMORY-115 dream-applied annotations, FR-MEMORY-119 session_id linkage, FR-MEMORY-114 importance-scored attribution all become dead-weight data that operators can't access without writing custom scripts. The talk's demo segment showcases this exact feature: clicking through to see 'when was each entry written, by which agent, with what change?' The 8-hour effort is dominantly projection logic (no new data) and CLI/REST glue. Skipping means we've shipped the data layer of memory governance (FR-MEMORY-115/117/118/119) without the visibility layer. The infrastructure becomes hard to operate."
---

## §1 — Description (BCP-14 normative)

The history projection is a **read-only** view over the existing audit chain that surfaces per-file version + attribution. No new audit data is created. The contract:

1. **MUST** be read-only. Every history operation reads existing audit-chain rows; no new put/move/delete/aux row is emitted by `cyberos history`. (DEC-260.)
2. **MUST** support `cyberos history <path> [--limit <N>] [--chronological] [--json] [--no-follow-moves] [--show-body] [--since <duration>]`. Default: most-recent-first, 10 entries, no body bytes, follow moves.
3. **MUST** return entries with at minimum these fields per row:
    ```python
    @dataclass
    class HistoryEntry:
        seq:             int
        ts:              datetime         # UTC
        kind:            str              # "put" | "move" | "delete" | aux-row kinds
        actor:           str
        body_hash:       Optional[str]    # absent for non-mutating rows
        frontmatter_diff: Optional[dict]  # field-level diff vs previous version
        body_diff:       Optional[str]    # unified diff vs previous version (if --show-body)
        extra:           dict             # extra fields from the row (dream_id, session_id, etc.)
    ```
4. **MUST** render the `extra` field's content inline in the human-formatted output, with these recognised annotation patterns:
    - `extra.dream_id` → "via dream <id>"
    - `extra.proposal_id` → "(proposal <id>)"
    - `extra.session_id` → "during session <id>"
    - `extra.invocation` (e.g. "consolidate") → "via <invocation>"
    - `extra.imported_from` → "imported from <fingerprint>"
    - `extra.merged_into` → "merged into <path>"
    These annotations make the audit-chain richness operator-readable without requiring `--json` raw inspection.
5. **MUST** follow move chains by default. `cyberos history dst.md` walks back; when a `move(src, dst)` row is encountered, history continues walking under `src.md`'s prior history. Repeats for chained moves. `--no-follow-moves` cuts the walk at the move boundary.
6. **MUST** support `--since 24h | 7d | 30d | <ISO timestamp>` to bound the walk. Default: no time bound (full chain history for that path).
7. **MUST** compute per-version body diffs from adjacent `put` rows on the same path. The diff format default is **unified-diff** (operator-readable); `--json` returns structured `{added: [line, ...], removed: [line, ...], context: [...]}`. Frontmatter diffs are always structured (field-level adds / removes / changes).
8. **MUST** include rows for ALL chain kinds that touch the path, not just `put/move/delete`. Specifically: `episode.logged` (FR-MEMORY-112), `memory.importance_scored` (FR-MEMORY-114), `dream.proposal_applied` (FR-MEMORY-115), `memory.acl_denied` and `memory.precondition_failed` (FR-MEMORY-117/118 attempt rows). These rows surface as "annotation events" — they appear in the history but don't trigger a body diff.
9. **MUST** render tombstone correctly — a `delete(path, tombstone)` row appears as a history entry with `kind: "delete"` and `extra.mode: "tombstone"`; subsequent re-creation via `put` is the next entry. `delete(path, purge)` rows surface but with body redacted (per §3.6 the *fact* of purge is auditable; the body bytes are gone).
10. **MUST** add a REST endpoint `GET /api/v2/memories/<path>/history` on `cyberos serve` that returns JSON. Same query parameters as CLI (`limit`, `chronological`, `follow_moves`, `since`, `show_body`).
11. **MUST** complete `cyberos history <path>` in ≤ 200 ms p95 on a 100,000-row chain (chain-walk is O(N) but bounded by index; the existing `cyberos search` performance budget is the benchmark — history's projection should not be slower per-row than search).
12. **MUST** support `cyberos history --all-paths [--limit 100]` for cluster-level view: emits a chronological/reverse list of every chain row touching any memory file. Useful for "what's been happening lately?" overview.
13. **MUST** display correctly for paths that never existed — `cyberos history memories/never-was.md` returns empty list (not an error). The CLI prints "No history for this path." `--json` returns `[]`.
14. **SHOULD** support `cyberos history --filter actor=<x>` and `--filter session_id=<y>` for narrow operator queries. Slice-4 polish — basic implementation acceptable in slice-3, full grammar in slice-4.
15. **SHOULD** support a `tail`-style mode `cyberos history --follow` that watches the chain for new rows touching the path and prints them live. Slice-4+ stretch.

---

## §2 — Why this design (rationale for humans)

**Why read-only (§1 #1, DEC-260).** History is *querying* state, not changing it. Emitting audit rows for "I looked at history" creates audit-row noise without value — every CI run, every operator `--help`, every dashboard refresh would pollute the chain. The exception (per AGENTS.md §3.2) is that `view` MAY emit a row, but history doesn't trigger that either since it's a pure projection, not a memory file read.

**Why follow moves by default (§1 #5, DEC-263).** Operators thinking about `memories/facts/x.md` mentally include "what this memory used to be called." If history stops at the move boundary, the operator has to remember the old path to see the prior history — defeats the purpose. Following moves transparently joins the chain so the operator sees the full history as one timeline. `--no-follow-moves` is the explicit "I want just the post-move history" escape.

**Why render `extra` annotations inline (§1 #4).** The audit chain accumulates rich provenance under `extra` (dream_id, session_id, invocation, imported_from, ...). If the human-format output is just `[seq | ts | kind | actor]` and the operator has to `--json` to see the extras, the rich data is hidden. Inline rendering makes it discoverable — "ah, this row was written by dream X during session Y."

**Why include non-mutation rows (§1 #8).** A memory's history isn't just its writes; it's the events around it. `memory.importance_scored` happened at write time. `memory.acl_denied` records that someone TRIED to write but couldn't. `dream.proposal_applied` records a logical change even when no body bytes changed. Operators investigating "what's happened to this memory?" want all of these. The full event stream is the history.

**Why unified-diff for body (§1 #7, DEC-262).** Default to operator-readable. Most operators see history in a terminal; unified diff is what `git log -p` shows, which is the mental model. `--json` is for tooling. Structured frontmatter diff is always structured because frontmatter is structured data (field-level diff is more informative than line-level diff of YAML).

**Why ≤ 200 ms p95 (§1 #11).** History is an interactive command; operators run it in a tight loop while debugging. > 200 ms feels laggy. The existing audit binlog reader does ~50 µs per row; 100K rows = ~5 ms in the best case. Allowing 200 ms is 40× headroom for projection / diff / formatting.

**Why `--all-paths` cluster view (§1 #12).** Common operator question: "what's been happening in this memory over the last hour?" Without `--all-paths`, the operator has to do per-path history or read the raw binlog. Cluster view is one command + filter.

**Why no protocol amendment (§1 — implicit).** This FR adds zero new normative writer behaviour, no new audit kinds, no new schema fields. It's pure projection over existing data. AGENTS.md unchanged.

---

## §3 — API contract

### History dataclass

```python
# modules/memory/cyberos/core/history.py
from __future__ import annotations
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional


@dataclass
class HistoryEntry:
    seq:             int
    ts:              datetime
    kind:            str
    actor:           str
    body_hash:       Optional[str]   = None
    frontmatter_diff: Optional[dict]  = None
    body_diff:       Optional[str]    = None
    extra:           dict            = field(default_factory=dict)
```

### Walk function

```python
# modules/memory/cyberos/core/history.py — continued
from cyberos.core.reader import AuditChainReader


def walk(
    store_path:      Path,
    target_path:     str,
    follow_moves:    bool = True,
    since:           Optional[datetime] = None,
    limit:           Optional[int] = None,
    show_body:       bool = False,
) -> list[HistoryEntry]:
    chain  = AuditChainReader(store_path)
    paths  = {target_path}                            # accumulates moved-from paths if follow_moves
    entries: list[HistoryEntry] = []
    prev_bodies: dict[str, str] = {}                  # path → previous body text for diffing

    for row in chain.iter_rows(reverse=True):
        ts = datetime.fromisoformat(row["ts"]) if isinstance(row["ts"], str) else row["ts"]
        if since and ts < since:
            break
        if not _touches_path(row, paths):
            continue
        entry = _row_to_entry(row, prev_bodies, show_body)
        entries.append(entry)
        if follow_moves and row["kind"] == "move":
            # The move's prior path is also part of the chain
            paths.add(row["payload"]["src"])
        if limit and len(entries) >= limit:
            break

    # We collected reverse-chronologically; default render order in entries is most-recent-first
    return entries


def _touches_path(row: dict, paths: set[str]) -> bool:
    kind = row["kind"]
    payload = row.get("payload", {})
    if kind in ("put", "delete"):
        return payload.get("path") in paths
    if kind == "move":
        return payload.get("src") in paths or payload.get("dst") in paths
    # aux kinds carrying a `path` field
    return payload.get("path") in paths
```

### CLI

```python
# modules/memory/cyberos/cli/history.py
import argparse, json, sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

from cyberos.core.history import walk, HistoryEntry


def add_args(sub: argparse.ArgumentParser) -> None:
    sub.add_argument("path", help="Memory path under <memory-root>/")
    sub.add_argument("--limit",          type=int, default=10)
    sub.add_argument("--chronological",  action="store_true")
    sub.add_argument("--json",           action="store_true")
    sub.add_argument("--no-follow-moves", action="store_true")
    sub.add_argument("--show-body",      action="store_true")
    sub.add_argument("--since",          help="24h | 7d | 30d | ISO timestamp")


def _parse_since(s: str) -> datetime:
    if s.endswith("h"): return datetime.now(timezone.utc) - timedelta(hours=int(s[:-1]))
    if s.endswith("d"): return datetime.now(timezone.utc) - timedelta(days=int(s[:-1]))
    return datetime.fromisoformat(s)


def run(store: Path, args: argparse.Namespace) -> int:
    since = _parse_since(args.since) if args.since else None
    entries = walk(
        store_path=store, target_path=args.path,
        follow_moves=not args.no_follow_moves,
        since=since, limit=args.limit, show_body=args.show_body,
    )
    if args.chronological:
        entries.reverse()
    if args.json:
        print(json.dumps([_entry_to_json(e) for e in entries], default=str, indent=2))
        return 0
    if not entries:
        print(f"No history for {args.path!r}.")
        return 0
    for e in entries:
        _render_human(e, show_body=args.show_body)
    return 0


def _render_human(e: HistoryEntry, show_body: bool) -> None:
    annotations = []
    if "dream_id" in e.extra:    annotations.append(f"via dream {e.extra['dream_id'][:8]}…")
    if "proposal_id" in e.extra: annotations.append(f"(proposal {e.extra['proposal_id']})")
    if "session_id" in e.extra:  annotations.append(f"during session {e.extra['session_id']}")
    if "invocation" in e.extra:  annotations.append(f"via {e.extra['invocation']}")
    if "imported_from" in e.extra: annotations.append(f"imported from {e.extra['imported_from']}")
    if "merged_into" in e.extra: annotations.append(f"merged into {e.extra['merged_into']}")
    annot_str = " " + " ".join(annotations) if annotations else ""
    body_hash_short = (e.body_hash[:8] + "…") if e.body_hash else "—"
    print(f"[{e.seq:>6}] {e.ts.isoformat()} {e.kind:<28} {e.actor:<16} body={body_hash_short}{annot_str}")
    if e.frontmatter_diff:
        for op, fields in e.frontmatter_diff.items():
            for field, val in fields.items():
                marker = {"added": "+", "removed": "-", "changed": "~"}.get(op, "?")
                print(f"         {marker} {field}: {val!r}")
    if show_body and e.body_diff:
        for line in e.body_diff.splitlines():
            print(f"         {line}")
```

### REST endpoint

```python
# modules/memory/cyberos/core/serve.py — addition
@app.get("/api/v2/memories/{path:path}/history")
async def memory_history(
    path:          str,
    limit:         int  = 10,
    chronological: bool = False,
    follow_moves:  bool = True,
    since:         Optional[str] = None,
    show_body:     bool = False,
):
    since_dt = _parse_since(since) if since else None
    entries = walk(
        store_path=app.state.store_path, target_path=path,
        follow_moves=follow_moves, since=since_dt, limit=limit, show_body=show_body,
    )
    if chronological:
        entries.reverse()
    return {"path": path, "entries": [_entry_to_json(e) for e in entries]}
```

---

## §4 — Acceptance criteria

1. **Single-write memory** — write once + history → 1 entry, kind=put, body_diff=None (no prior version). *(traces_to: §1 #2, §1 #3)*
2. **Multi-write memory** — write 3 times → 3 entries (most-recent-first); entries 1 and 2 have body_diff against the prior version; entry 3 has body_diff=None. *(traces_to: §1 #3, §1 #7)*
3. **`--chronological` reverses order** — same fixture → 3 entries oldest-first. *(traces_to: §1 #2, DEC-261)*
4. **`--limit N` caps results** — write 5 times → `--limit 2` returns 2 entries. *(traces_to: §1 #2)*
5. **`--since 24h` filters** — write today + write 2 days ago + history --since 24h → only today's entry. *(traces_to: §1 #6)*
6. **`--show-body` includes unified diff** — multi-version fixture → diff text present in output; format is unified-diff (headers like `--- a/...`, `+++ b/...`). *(traces_to: §1 #7, DEC-262)*
7. **`--json` returns structured** — output parses as JSON array; each entry has the documented HistoryEntry fields. *(traces_to: §1 #2, §1 #7)*
8. **Follow-moves on default** — write to src.md, move src.md → dst.md, write to dst.md, history dst.md → 3 entries (covers both paths). *(traces_to: §1 #5, DEC-263)*
9. **`--no-follow-moves` cuts at boundary** — same fixture + flag → only the post-move entries (2 of 3). *(traces_to: §1 #5)*
10. **Dream-applied annotation rendered** — fixture with a `delete` row carrying `extra.dream_id` → human output contains "via dream <id>…". *(traces_to: §1 #4)*
11. **Session_id annotation rendered** — fixture with put row carrying `extra.session_id` → output contains "during session <id>". *(traces_to: §1 #4)*
12. **Importance_scored annotation surfaces** — fixture with `memory.importance_scored` aux row for path → appears as history entry with kind == `memory.importance_scored`. *(traces_to: §1 #4, §1 #8)*
13. **Tombstone renders correctly** — write + delete(tombstone) → history has 2 entries; delete entry has `extra.mode: "tombstone"`. *(traces_to: §1 #9)*
14. **Purged row body redacted** — write + delete(purge) → delete entry present; body_hash present but body_diff is None (body bytes gone). *(traces_to: §1 #9)*
15. **Frontmatter diff structured** — write with `importance: 0.5`, then update to `importance: 0.8` → second entry's frontmatter_diff contains `{changed: {importance: {old: 0.5, new: 0.8}}}`. *(traces_to: §1 #7)*
16. **Multi-kind events appear** — fixture with put + episode.logged + memory.importance_scored + memory.acl_denied for same path → all 4 appear in history (sorted by seq descending). *(traces_to: §1 #8)*
17. **REST endpoint matches CLI** — `GET /api/v2/memories/<path>/history?limit=3` returns JSON with same shape as `cyberos history <path> --limit 3 --json`. *(traces_to: §1 #10)*
18. **Latency p95 ≤ 200 ms on 100K rows** — bench script measures CLI invocation on fixture. *(traces_to: §1 #11)*
19. **`--all-paths`** — option works; returns chronological list across all paths. *(traces_to: §1 #12)*
20. **Path never existed** — `cyberos history memories/never.md` returns empty list (CLI prints "No history…"; JSON returns `[]`). *(traces_to: §1 #13)*
21. **Read-only — no new audit rows** — head_seq before/after `cyberos history ...` is identical. *(traces_to: §1 #1, DEC-260)*

---

## §5 — Verification

```python
# modules/memory/tests/core/test_history.py
import json, pytest, subprocess
from datetime import datetime, timezone
from cyberos.core.history import walk


def test_single_write(seeded_memory):
    """AC #1"""
    seeded_memory.put("memories/facts/single.md", "body v1", meta={})
    entries = walk(seeded_memory.store_path, "memories/facts/single.md")
    assert len(entries) == 1
    assert entries[0].kind == "put"
    assert entries[0].body_diff is None


def test_multi_write_diffs(seeded_memory):
    """AC #2"""
    seeded_memory.put("memories/facts/x.md", "v1", meta={})
    seeded_memory.put("memories/facts/x.md", "v2", meta={})
    seeded_memory.put("memories/facts/x.md", "v3", meta={})
    entries = walk(seeded_memory.store_path, "memories/facts/x.md")
    assert len(entries) == 3
    # Most-recent-first; entry 0 has no diff (it's the latest, no NEWER version to diff)
    assert entries[2].body_diff is None
    assert entries[1].body_diff is not None
    assert entries[0].body_diff is not None


def test_chronological_reverses(seeded_memory):
    """AC #3"""
    seeded_memory.put("memories/facts/x.md", "v1", meta={})
    seeded_memory.put("memories/facts/x.md", "v2", meta={})
    e = walk(seeded_memory.store_path, "memories/facts/x.md")
    e_rev = list(reversed(e))
    assert e[0].seq > e[1].seq
    assert e_rev[0].seq < e_rev[1].seq


def test_limit_caps(seeded_memory):
    """AC #4"""
    for i in range(5):
        seeded_memory.put("memories/facts/lim.md", f"v{i}", meta={})
    e = walk(seeded_memory.store_path, "memories/facts/lim.md", limit=2)
    assert len(e) == 2


def test_since_filters(seeded_memory, mock_time):
    """AC #5"""
    mock_time.set("2026-05-17T00:00:00Z")
    seeded_memory.put("memories/facts/old.md", "old", meta={})
    mock_time.set("2026-05-19T00:00:00Z")
    seeded_memory.put("memories/facts/old.md", "new", meta={})
    e = walk(seeded_memory.store_path, "memories/facts/old.md",
             since=datetime(2026, 5, 18, tzinfo=timezone.utc))
    assert len(e) == 1
    assert e[0].seq > 1


def test_show_body_diff(seeded_memory):
    """AC #6"""
    seeded_memory.put("memories/facts/d.md", "line a\nline b", meta={})
    seeded_memory.put("memories/facts/d.md", "line a\nline c", meta={})
    e = walk(seeded_memory.store_path, "memories/facts/d.md", show_body=True)
    assert "+line c" in e[1].body_diff
    assert "-line b" in e[1].body_diff


def test_json_output(seeded_memory, tmp_path):
    """AC #7"""
    seeded_memory.put("memories/facts/j.md", "x", meta={})
    result = subprocess.run([
        "python", "-m", "cyberos", "--store", str(seeded_memory.store_path),
        "history", "memories/facts/j.md", "--json"
    ], capture_output=True, text=True)
    data = json.loads(result.stdout)
    assert isinstance(data, list)
    assert data[0]["kind"] == "put"


def test_dream_annotation(seeded_memory_with_dream_apply):
    """AC #10"""
    e = walk(seeded_memory_with_dream_apply.store_path, "memories/facts/dream-touched.md")
    assert any("dream_id" in entry.extra for entry in e)


def test_session_annotation(seeded_memory_with_active_session):
    """AC #11"""
    e = walk(seeded_memory_with_active_session.store_path, "memories/facts/sess-touched.md")
    assert any("session_id" in entry.extra for entry in e)


def test_importance_scored_appears(seeded_memory_with_scoring):
    """AC #12"""
    e = walk(seeded_memory_with_scoring.store_path, "memories/facts/scored.md")
    assert any(entry.kind == "memory.importance_scored" for entry in e)


def test_tombstone_render(seeded_memory):
    """AC #13"""
    seeded_memory.put("memories/facts/t.md", "body", meta={})
    seeded_memory.delete("memories/facts/t.md", mode="tombstone")
    e = walk(seeded_memory.store_path, "memories/facts/t.md")
    assert any(entry.kind == "delete" and entry.extra.get("mode") == "tombstone" for entry in e)


def test_purged_body_redacted(seeded_memory):
    """AC #14"""
    seeded_memory.put("memories/facts/p.md", "secret", meta={})
    seeded_memory.delete("memories/facts/p.md", mode="purge",
                         reason="GDPR Article 17 request")
    e = walk(seeded_memory.store_path, "memories/facts/p.md")
    delete_entry = next(entry for entry in e if entry.kind == "delete")
    assert delete_entry.body_diff is None         # body bytes gone


def test_frontmatter_diff_structured(seeded_memory):
    """AC #15"""
    seeded_memory.put("memories/facts/fm.md", "body", meta={"importance": 0.5})
    seeded_memory.put("memories/facts/fm.md", "body", meta={"importance": 0.8})
    e = walk(seeded_memory.store_path, "memories/facts/fm.md")
    diff = e[1].frontmatter_diff
    assert diff["changed"]["importance"]["old"] == 0.5
    assert diff["changed"]["importance"]["new"] == 0.8


def test_multi_kind_events(seeded_memory_with_all_kinds):
    """AC #16"""
    e = walk(seeded_memory_with_all_kinds.store_path, "memories/facts/multi.md")
    kinds = {entry.kind for entry in e}
    expected = {"put", "episode.logged", "memory.importance_scored", "memory.acl_denied"}
    assert expected.issubset(kinds)


def test_never_existed_path(empty_memory):
    """AC #20"""
    e = walk(empty_memory.store_path, "memories/facts/never.md")
    assert e == []


def test_read_only_no_new_rows(seeded_memory):
    """AC #21"""
    head_before = seeded_memory.head_seq()
    walk(seeded_memory.store_path, "memories/facts/anything.md")
    assert seeded_memory.head_seq() == head_before


def test_rest_endpoint_matches_cli(running_cyberos_serve):
    """AC #17"""
    import requests
    cli_json = subprocess.run([
        "python", "-m", "cyberos", "--store", str(running_cyberos_serve.store_path),
        "history", "memories/facts/x.md", "--limit", "3", "--json"
    ], capture_output=True, text=True).stdout
    rest_json = requests.get(
        f"{running_cyberos_serve.url}/api/v2/memories/memories/facts/x.md/history?limit=3"
    ).text
    assert json.loads(cli_json) == json.loads(rest_json)["entries"]
```

```python
# modules/memory/tests/core/test_history.py
import pytest
from cyberos.core.history import walk


def test_follow_moves_default(seeded_memory):
    """AC #8"""
    seeded_memory.put("memories/facts/src.md", "v1", meta={})
    seeded_memory.move("memories/facts/src.md", "memories/facts/dst.md")
    seeded_memory.put("memories/facts/dst.md", "v2", meta={})
    e = walk(seeded_memory.store_path, "memories/facts/dst.md", follow_moves=True)
    assert len(e) == 3                                      # put-src + move + put-dst


def test_no_follow_moves(seeded_memory):
    """AC #9"""
    seeded_memory.put("memories/facts/src.md", "v1", meta={})
    seeded_memory.move("memories/facts/src.md", "memories/facts/dst.md")
    seeded_memory.put("memories/facts/dst.md", "v2", meta={})
    e = walk(seeded_memory.store_path, "memories/facts/dst.md", follow_moves=False)
    # Only post-move events (the move row itself + the new put on dst)
    assert all("src" not in (entry.extra.get("from", "") or "") for entry in e)


def test_chained_moves(seeded_memory):
    """Chained: a.md -> b.md -> c.md"""
    seeded_memory.put("memories/facts/a.md", "v1", meta={})
    seeded_memory.move("memories/facts/a.md", "memories/facts/b.md")
    seeded_memory.move("memories/facts/b.md", "memories/facts/c.md")
    seeded_memory.put("memories/facts/c.md", "v2", meta={})
    e = walk(seeded_memory.store_path, "memories/facts/c.md", follow_moves=True)
    # put on a + 2 moves + put on c = 4
    assert len(e) == 4


def test_deleted_then_recreated(seeded_memory):
    seeded_memory.put("memories/facts/r.md", "v1", meta={})
    seeded_memory.delete("memories/facts/r.md", mode="tombstone")
    seeded_memory.put("memories/facts/r.md", "v2", meta={})
    e = walk(seeded_memory.store_path, "memories/facts/r.md")
    assert len(e) == 3
    kinds = [entry.kind for entry in e]
    assert kinds == ["put", "delete", "put"][::-1]            # most-recent-first
```

---

## §6 — Implementation skeleton

API + tests above are the skeleton. Order:

1. `cyberos/core/history.py` — `walk()` + `HistoryEntry`.
2. Diff computation helpers (unified diff for body, field-level for frontmatter).
3. `cyberos/cli/history.py` — CLI subcommand.
4. `__main__.py` — wire `cyberos history`.
5. `cyberos/core/serve.py` — REST endpoint.
6. Tests + fixtures.
7. CHANGELOG.

---

## §7 — Dependencies

- **FR-MEMORY-108 (related)** — audit chain reader infrastructure is reused. History is a different projection over the same data.
- **FR-MEMORY-112 (related)** — `episode.logged` aux rows are rendered as history entries.
- **FR-MEMORY-113 (related)** — `meta.importance` field diffs surface in `frontmatter_diff`.
- **FR-MEMORY-114 (related)** — `memory.importance_scored` rows are rendered as history entries.
- **FR-MEMORY-115 (related)** — `dream.proposal_applied` rows surface; `extra.dream_id` annotation rendered.
- **FR-MEMORY-117 (related)** — `memory.acl_denied` rows surface as attempt-history.
- **FR-MEMORY-118 (related)** — `memory.precondition_failed` rows surface similarly.
- **FR-MEMORY-119 (related)** — `extra.session_id` annotation rendered.

---

## §8 — Example payloads

### CLI human output

```text
$ cyberos history memories/sre/dispatch-1.md --limit 6
[  4319] 2026-05-19T08:00:42Z put                          stephen          body=f3a9b2c1…
         ~ importance: 0.5 → 0.78
         + last_verified_at: '2026-05-19T08:00:42Z'
[  4316] 2026-05-19T07:30:00Z memory.importance_scored      claude-code-hook body=—
[  4291] 2026-05-18T20:55:13Z put                          stephen          body=abc123de… during session sre-investigation-2026-05-19
[  4275] 2026-05-18T20:45:00Z delete                       dream-applier    body=def456…  via dream 01HJ8XVK… (proposal P3FQ8K2X)
[  4274] 2026-05-18T20:44:55Z dream.proposal_applied       dream-applier    body=—
[  4112] 2026-05-15T14:20:00Z put                          stephen          body=def456…
```

### CLI --json output

```json
[
  {
    "seq": 4319,
    "ts": "2026-05-19T08:00:42Z",
    "kind": "put",
    "actor": "stephen",
    "body_hash": "f3a9b2c1d4e5f6789a0b1c2d3e4f56789a0b1c2d3e4f56789a0b1c2d3e4f5678",
    "frontmatter_diff": {
      "changed": {"importance": {"old": 0.5, "new": 0.78}},
      "added":   {"last_verified_at": "2026-05-19T08:00:42Z"}
    },
    "extra": {"actor": "stephen"}
  }
]
```

### REST response

```json
GET /api/v2/memories/memories/sre/dispatch-1.md/history?limit=3

{
  "path": "memories/sre/dispatch-1.md",
  "entries": [ ... ]
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Filter grammar `--filter actor=x` — §1 #14; slice 4 polish.
- `--follow` tail mode — §1 #15; slice 4+.
- Two-way diff against arbitrary timestamps (`--from X --to Y`) — slice 4+.
- Graph visualization of move chains — slice 4+ (could go on `cyberos serve` dashboard).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Path never existed | walk returns empty | CLI prints "No history" | None — by design |
| Path under symlink | resolver follows | works | None |
| Audit chain partial corruption | reader raises on bad frame | partial history rendered with WARN | Operator runs `cyberos doctor` |
| Very long history (>100K rows) | `--limit` + lazy walk | bounded output | Operator uses `--limit` + `--since` |
| Body too large for diff (>1MB) | diff fallback to byte-level | "binary file (use --no-show-body)" | Operator skips --show-body |
| Frontmatter unparseable | diff fallback to "frontmatter unparseable" | history still renders | Operator inspects raw row via `cyberos read` |
| Move chain with cycle | bounded by N-row walk; cycle caught | warning printed; first cycle break | walker `chain-no-cycles` already covers |
| `extra` dict with arbitrary keys | renderer ignores unknown keys | works | None |
| Concurrent write during history | reader snapshots HEAD; consistent view | works | None |
| `--since` future-dated | walk returns empty | "No history" | Operator fixes timestamp |
| `--limit 0` | edge case; returns empty | "No history" | Operator picks N ≥ 1 |
| Tombstone followed by re-create | both entries rendered | works | None |
| Purge body redacted | body_diff = None | rendered correctly | None |
| REST endpoint with unknown `path` | 200 + `entries: []` | empty array | None — by design |
| Slash-encoding in REST path | FastAPI `path:path` handles | works | None |
| Time zone mismatch | walker uses UTC; CLI prints ISO with offset | unambiguous | None |
| --json with --show-body on huge memory | bytes returned | works; user might want to omit --show-body | Operator inspects --json --no-show-body first |
| `--all-paths` on massive memory | bounded by `--limit` | works | Operator pages with `--since` |
| Filter with unknown key | error in slice 4 surface (slice-3 ignores unknown filters) | works in slice-3 | None until slice 4 |

---

## §11 — Implementation notes

- **Walk is reverse-chronological by default** because the most-recent-first ordering is what operators want; reversing once at the end is O(N) and cheap.
- **Move-following is the natural way operators think about history.** Counter-intuitively, the easier mental model is "history of this name" but operators thinking about content want "history of this concept" — which moves and re-renames are part of. Hence default = follow.
- **Frontmatter diff** uses dict-based field-level comparison; line-level diff would be misleading because YAML serialisation is non-canonical (key order, whitespace).
- **Body diff via Python's `difflib.unified_diff`** — bounded, well-known format. JSON-mode delegates to the same engine but emits structured `{added: [], removed: [], context: []}` arrays.
- **Purged rows surface but with redacted body** — required by §3.6 ("the fact of purge is itself a ledger leaf and is not erasable"). History honours this.
- **`--all-paths`** uses the same `walk()` engine without the path filter — single-pass over the chain. The CLI applies path filtering downstream if needed.
- **REST endpoint shares the CLI's `walk()` engine** so behaviour is identical; AC #17 asserts this.
- **The `_render_human` helper** is intentionally simple; rich formatting (colour, alignment) lives in a separate `--pretty` flag for slice-4. Slice-3 ships terminal-readable plain text.

---

*End of FR-MEMORY-120.*
