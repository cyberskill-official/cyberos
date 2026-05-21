"""
cyberos.core.index — the derived SQLite index. WAL mode; concurrent reads.

The hot SQLite file lives OUTSIDE the store (in the OS cache dir) to avoid
iCloud / Dropbox / OneDrive sync conflicts with the ``.cyberos-memory/``
directory the user actually wants synced. That pre-existing decision from
the legacy writer is preserved here; what changes is:

* journal mode flipped from ``DELETE`` (writer blocks readers) to ``WAL``
  ("writers cannot block readers, and readers don't block writers"
  — sqlite.org/wal.html);
* tuned PRAGMA set: ``synchronous=NORMAL``, ``mmap_size=256MiB``,
  ``cache_size=64MiB``, ``temp_store=MEMORY``;
* ``fullfsync=1`` for macOS — on Apple's bundled SQLite this maps to
  ``F_BARRIERFSYNC``; on upstream SQLite (bundled via ``pysqlite3-binary``)
  it is ``F_FULLFSYNC``. See bonsaidb.io/blog/acid-on-apple.

The index is **fully rebuildable from the binlog**, so losing the WAL tail
on power loss is acceptable: at worst, the next ``open_index`` replays
from the last applied ``seq`` recorded in ``index/manifest.json``. The
ledger and manifest are the sources of truth; this table is a cache.
"""

from __future__ import annotations

import json
import os
import sqlite3
import sys
from pathlib import Path
from typing import Final, Iterable


_SCHEMA: Final[str] = """
CREATE TABLE IF NOT EXISTS memories(
    rel_path        TEXT PRIMARY KEY,
    kind            TEXT NOT NULL,
    actor           TEXT NOT NULL,
    ts_ns           INTEGER NOT NULL,
    content_sha256  TEXT NOT NULL,
    last_seq        INTEGER NOT NULL,
    tombstoned      INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_memories_kind  ON memories(kind);
CREATE INDEX IF NOT EXISTS idx_memories_actor ON memories(actor);
CREATE INDEX IF NOT EXISTS idx_memories_ts    ON memories(ts_ns);
CREATE INDEX IF NOT EXISTS idx_memories_tomb  ON memories(tombstoned);

CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
    rel_path UNINDEXED,
    body,
    tokenize = 'unicode61'
);

CREATE TABLE IF NOT EXISTS sync_state(
    k TEXT PRIMARY KEY,
    v TEXT NOT NULL
);
"""


def cache_dir() -> Path:
    """OS-native cache directory for the index DB.

    * macOS:   ``~/Library/Caches/cyberos``
    * Windows: ``%LOCALAPPDATA%\\cyberos``
    * Linux/other POSIX: ``$XDG_CACHE_HOME/cyberos`` (or ``~/.cache/cyberos``)
    """
    if sys.platform == "darwin":
        return Path.home() / "Library" / "Caches" / "cyberos"
    if sys.platform == "win32":
        base = os.environ.get("LOCALAPPDATA") or str(Path.home())
        return Path(base) / "cyberos"
    xdg = os.environ.get("XDG_CACHE_HOME") or str(Path.home() / ".cache")
    return Path(xdg) / "cyberos"


def db_path_for(store_fingerprint: str) -> Path:
    """Return the absolute path to the index DB for a given store fingerprint."""
    return cache_dir() / store_fingerprint / "cyberos.db"


def open_index(store_fingerprint: str) -> sqlite3.Connection:
    """Open the index DB in WAL mode with the tuned PRAGMA set.

    ``isolation_level=None`` gives manual transaction control — we want
    explicit ``BEGIN IMMEDIATE`` / ``COMMIT`` around the replay batch,
    not Python's default auto-commit behaviour.

    ``check_same_thread=False`` permits the writer thread and reader
    threads to share the connection. SQLite itself serialises calls; for
    high-throughput multi-thread scenarios, prefer one connection per
    thread.
    """
    db = db_path_for(store_fingerprint)
    db.parent.mkdir(parents=True, exist_ok=True)

    conn = sqlite3.connect(str(db), isolation_level=None, check_same_thread=False)
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA synchronous = NORMAL")
    conn.execute("PRAGMA wal_autocheckpoint = 1000")
    conn.execute("PRAGMA mmap_size = 268435456")        # 256 MiB
    conn.execute("PRAGMA temp_store = MEMORY")
    conn.execute("PRAGMA cache_size = -65536")          # 64 MiB
    conn.execute("PRAGMA busy_timeout = 5000")
    # On Apple-bundled SQLite this maps to F_BARRIERFSYNC; on upstream
    # SQLite (via pysqlite3-binary) it is F_FULLFSYNC.
    conn.execute("PRAGMA fullfsync = 1")
    conn.executescript(_SCHEMA)
    return conn


def last_applied_seq(conn: sqlite3.Connection) -> int:
    """Return the highest binlog seq already applied to this index."""
    row = conn.execute(
        "SELECT v FROM sync_state WHERE k = 'last_applied_seq'"
    ).fetchone()
    if row is None:
        return 0
    try:
        return int(row[0])
    except (TypeError, ValueError):
        return 0


def replay_from_binlog(
    conn: sqlite3.Connection,
    store: Path,
    from_seq: int | None = None,
) -> int:
    """Forward-replay binlog records into the index.

    Idempotent: re-running with a non-zero ``from_seq`` skips records
    already applied. Returns the new high-water-mark seq.

    The replay opens one transaction and commits at the end so a partial
    crash leaves the index unchanged (the binlog is the source of truth;
    we can always replay again).
    """
    from cyberos.core.walker import MmapWalker  # local import — heavy

    if from_seq is None:
        from_seq = last_applied_seq(conn)
    applied = from_seq

    # Order: sealed monthly segments lexicographically, then current.binlog.
    audit_dir = store / "audit"
    segments: list[Path] = sorted(
        p for p in audit_dir.glob("*.binlog") if p.name != "current.binlog"
    )
    current = audit_dir / "current.binlog"
    if current.exists():
        segments.append(current)

    conn.execute("BEGIN IMMEDIATE")
    try:
        for path in segments:
            with MmapWalker(path) as walker:
                for _offset, rec in walker.iter_records():
                    seq = int(rec.extra.get("_seq", 0))
                    if seq <= applied:
                        continue
                    _apply_record(conn, rec, seq, store)
                    applied = seq
        conn.execute(
            "INSERT INTO sync_state(k, v) VALUES('last_applied_seq', ?) "
            "ON CONFLICT(k) DO UPDATE SET v = excluded.v",
            (str(applied),),
        )
        conn.execute("COMMIT")
    except BaseException:
        conn.execute("ROLLBACK")
        raise
    return applied


def _apply_record(
    conn: sqlite3.Connection,
    rec,
    seq: int,
    store: Path | None = None,
) -> None:
    """Apply one AuditRecord to the index. Match on rec.op.

    ``store`` is required to populate the FTS body for create/str_replace/
    insert/put records (the binlog only carries content_sha256, not the
    body text). If ``store`` is None the FTS body is left empty — this
    preserves backward-compatibility for any test path that didn't pass it.
    """
    if rec.op == "delete":
        conn.execute(
            "UPDATE memories SET tombstoned = 1, last_seq = ? WHERE rel_path = ?",
            (seq, rec.path),
        )
        # Mirror the tombstone into FTS so the row stops matching searches.
        conn.execute(
            "DELETE FROM memories_fts WHERE rel_path = ?",
            (rec.path,),
        )
        return

    if rec.op == "rename":
        new_path = rec.extra.get("to", rec.path)
        conn.execute(
            "UPDATE memories SET rel_path = ?, last_seq = ? WHERE rel_path = ?",
            (new_path, seq, rec.path),
        )
        # FTS keys on rel_path (UNINDEXED); update there too.
        conn.execute(
            "UPDATE memories_fts SET rel_path = ? WHERE rel_path = ?",
            (new_path, rec.path),
        )
        return

    # ``put`` is the canonical cross-memory import op per AGENTS.md §14.2.
    # Treat it identically to a create/str_replace from the index's POV —
    # without this, the 708 memories imported via cyberos.core.import_ on
    # 2026-05-19 never made it into the FTS index. (Bug pre-dated the
    # workbench-consume; fix lands as part of the v0-compat sweep.)
    if rec.op in ("create", "str_replace", "insert", "put"):
        kind = rec.extra.get("kind", "unknown")
        conn.execute(
            """
            INSERT INTO memories(rel_path, kind, actor, ts_ns, content_sha256, last_seq, tombstoned)
            VALUES(?, ?, ?, ?, ?, ?, 0)
            ON CONFLICT(rel_path) DO UPDATE SET
                kind            = excluded.kind,
                actor           = excluded.actor,
                ts_ns           = excluded.ts_ns,
                content_sha256  = excluded.content_sha256,
                last_seq        = excluded.last_seq,
                tombstoned      = 0
            """,
            (rec.path, kind, rec.actor, rec.ts_ns, rec.content_sha256, seq),
        )
        # Populate the FTS body table by reading the file from the store.
        # The binlog only carries content_sha256, so we have to load the
        # current bytes. Best-effort: tolerate read failures so a missing
        # file (rare; concurrent delete) doesn't abort the whole replay.
        if store is not None:
            body_text = _load_body_text(store, rec.path)
            if body_text is not None:
                conn.execute(
                    "DELETE FROM memories_fts WHERE rel_path = ?",
                    (rec.path,),
                )
                conn.execute(
                    "INSERT INTO memories_fts(rel_path, body) VALUES(?, ?)",
                    (rec.path, body_text),
                )
        return

    # 'view', 'session.start', 'session.end' rows do not mutate the
    # derived index.


def _load_body_text(store: Path, rel_path: str) -> str | None:
    """Load the current body bytes for ``rel_path`` from ``store``, decoded
    as UTF-8 with replacement for non-UTF-8 chunks (so binary blobs don't
    abort the index replay). Returns None on read failure."""
    abs_path = store / rel_path
    try:
        raw = abs_path.read_bytes()
    except OSError:
        return None
    # Strip frontmatter block if present so FTS indexes the body only.
    if raw.startswith(b"---\n"):
        end = raw.find(b"\n---\n", 4)
        if end > 0:
            raw = raw[end + 5:]
    return raw.decode("utf-8", errors="replace")


def write_index_manifest(store: Path, last_seq: int) -> None:
    """Write ``<store>/index/manifest.json`` — the cold-start rebuild marker.

    The manifest lives INSIDE the store (so it travels with the zip
    export) and records the seq up to which the external SQLite index was
    last sync'd. The protocol is unversioned; this is just a cold-start
    pointer for the derived index.
    """
    target = store / "index" / "manifest.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    body = json.dumps(
        {"last_applied_seq": last_seq},
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    tmp = target.with_suffix(".tmp")
    fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    try:
        os.write(fd, body)
        if hasattr(os, "fdatasync"):
            os.fdatasync(fd)
        else:  # pragma: no cover
            os.fsync(fd)
    finally:
        os.close(fd)
    os.replace(tmp, target)


def read_index_manifest(store: Path) -> dict | None:
    """Read ``<store>/index/manifest.json`` if present."""
    target = store / "index" / "manifest.json"
    if not target.is_file():
        return None
    try:
        return json.loads(target.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None


def _sanitize_fts5(query: str) -> str:
    """Escape FTS5 special characters by quoting terms.

    FTS5 treats ``-``, ``*``, ``"``, ``(``, ``)``, ``:``, ``^``, ``AND``,
    ``OR``, ``NOT`` as operators.  A bare ``dual-sku`` is parsed as
    ``dual NOT sku`` which fails when ``sku`` isn't a column.  Quoting
    each whitespace-delimited token makes it a literal phrase match.
    """
    FTS5_SPECIAL = set('-* "()^:')
    words = query.split()
    out = []
    for w in words:
        if any(c in FTS5_SPECIAL for c in w) or w.upper() in ("AND", "OR", "NOT"):
            out.append(f'"{w}"')
        else:
            out.append(w)
    return " ".join(out)


def search_memories(
    conn: sqlite3.Connection,
    query: str,
    *,
    limit: int = 50,
) -> Iterable[tuple]:
    """FTS5 search over memory bodies. Returns (rel_path, snippet) tuples."""
    safe = _sanitize_fts5(query)
    return conn.execute(
        """
        SELECT m.rel_path,
               snippet(memories_fts, 1, '<<', '>>', '...', 16) AS snip
        FROM memories_fts
        JOIN memories m ON m.rel_path = memories_fts.rel_path
        WHERE memories_fts MATCH ? AND m.tombstoned = 0
        ORDER BY rank
        LIMIT ?
        """,
        (safe, limit),
    )


__all__ = [
    "cache_dir",
    "db_path_for",
    "open_index",
    "last_applied_seq",
    "replay_from_binlog",
    "write_index_manifest",
    "read_index_manifest",
    "search_memories",
]
