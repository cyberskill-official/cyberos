#!/usr/bin/env python3
"""
cyberos-index — local search index for `.cyberos-memory/` stores.

Builds and maintains a regenerable SQLite-backed index covering the four
high-traffic lookup patterns that grep-walking gets slow at past ~500 memories:

  - tag         → memory_id (multi-value)
  - relates-to  → memory_id (with relationship kind)
  - source-sha  → memory_id (ingestion dedup)
  - audit       → audit_id by path / memory_id / ts range

Plus tombstone-set membership and supersedes-graph traversal.

The index is **derived state**, not authoritative. AGENTS.md §11.1 excludes
`index/` from export bundles. Safe to delete at any time — `cyberos-index
build` regenerates from canonical `.cyberos-memory/` ground truth.

Usage
-----
    cyberos-index <store>                              # incremental update
    cyberos-index <store> build                        # full rebuild
    cyberos-index <store> update                       # explicit incremental
    cyberos-index <store> verify                       # check integrity
    cyberos-index <store> stats                        # show index stats
    cyberos-index <store> query tag <tag>
    cyberos-index <store> query relates-to <memory_id>
    cyberos-index <store> query source-sha <sha>
    cyberos-index <store> query audit-by-path <path>
    cyberos-index <store> query tombstoned <memory_id>

The index lives at `<store>/index/cyberos.db` and is excluded from exports.

Author: CyberOS local-optimization Stage 3
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
import sqlite3
import sys
import time
from pathlib import Path
from typing import Any, Iterable

try:
    import yaml  # type: ignore
except ImportError:
    yaml = None  # type: ignore


SCHEMA_VERSION = "1"

SCHEMA_SQL = """
CREATE TABLE IF NOT EXISTS memories (
    memory_id        TEXT PRIMARY KEY,
    file_path        TEXT NOT NULL UNIQUE,
    scope            TEXT,
    classification   TEXT,
    authority        TEXT,
    version          INTEGER,
    created_at       TEXT,
    last_updated_at  TEXT,
    body_sha         TEXT,
    tombstoned       INTEGER NOT NULL DEFAULT 0,
    source_sha       TEXT
);
CREATE INDEX IF NOT EXISTS idx_memories_path ON memories(file_path);
CREATE INDEX IF NOT EXISTS idx_memories_scope ON memories(scope);
CREATE INDEX IF NOT EXISTS idx_memories_classification ON memories(classification);
CREATE INDEX IF NOT EXISTS idx_memories_tombstoned ON memories(tombstoned);
CREATE INDEX IF NOT EXISTS idx_memories_source_sha ON memories(source_sha)
    WHERE source_sha IS NOT NULL;

CREATE TABLE IF NOT EXISTS tags (
    memory_id TEXT NOT NULL,
    tag       TEXT NOT NULL,
    PRIMARY KEY (memory_id, tag),
    FOREIGN KEY (memory_id) REFERENCES memories(memory_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);

CREATE TABLE IF NOT EXISTS relationships (
    from_id TEXT NOT NULL,
    to_id   TEXT NOT NULL,
    kind    TEXT NOT NULL,
    PRIMARY KEY (from_id, to_id, kind),
    FOREIGN KEY (from_id) REFERENCES memories(memory_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_relationships_to ON relationships(to_id);
CREATE INDEX IF NOT EXISTS idx_relationships_kind ON relationships(kind);

CREATE TABLE IF NOT EXISTS supersedes (
    from_id TEXT NOT NULL,
    to_id   TEXT NOT NULL,
    PRIMARY KEY (from_id, to_id),
    FOREIGN KEY (from_id) REFERENCES memories(memory_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_supersedes_to ON supersedes(to_id);

CREATE TABLE IF NOT EXISTS audit_rows (
    audit_id    TEXT PRIMARY KEY,
    ts          TEXT NOT NULL,
    op          TEXT NOT NULL,
    path        TEXT,
    memory_id   TEXT,
    chain       TEXT NOT NULL,
    prev_chain  TEXT NOT NULL,
    actor_kind  TEXT,
    actor_id    TEXT,
    ledger      TEXT NOT NULL  -- which audit/<YYYY-MM>.jsonl this came from
);
CREATE INDEX IF NOT EXISTS idx_audit_ts ON audit_rows(ts);
CREATE INDEX IF NOT EXISTS idx_audit_path ON audit_rows(path);
CREATE INDEX IF NOT EXISTS idx_audit_memory ON audit_rows(memory_id);
CREATE INDEX IF NOT EXISTS idx_audit_op ON audit_rows(op);

CREATE TABLE IF NOT EXISTS merkle_checkpoints (
    audit_id              TEXT PRIMARY KEY,
    ts                    TEXT NOT NULL,
    root                  TEXT NOT NULL,
    period_start_audit_id TEXT,
    period_end_audit_id   TEXT,
    leaves_count          INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (audit_id) REFERENCES audit_rows(audit_id)
);
CREATE INDEX IF NOT EXISTS idx_merkle_ts ON merkle_checkpoints(ts);

CREATE TABLE IF NOT EXISTS index_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"""

# Frontmatter exemption set — same as cyberos_validate.py
_EXEMPT_PREFIXES = ("meta/protocol-history/", "meta/health/")
_EXEMPT_FILES = {
    "meta/legacy-ids.md", "meta/tombstones.md",
    "meta/classification-rules.md", "meta/retention-rules.md",
    "meta/conflict-resolutions.md",
}


def is_exempt(rel: str) -> bool:
    if rel in _EXEMPT_FILES:
        return True
    return any(rel.startswith(p) for p in _EXEMPT_PREFIXES)


# ---------------------------------------------------------------------------
# Frontmatter parsing
# ---------------------------------------------------------------------------

def split_frontmatter(text: str) -> tuple[str | None, str]:
    if not text.startswith("---\n"):
        return None, text
    rest = text[4:]
    m = re.search(r"\n---(\n|$)", rest)
    if not m:
        return None, text
    return rest[:m.start()], rest[m.end():]


# ---------------------------------------------------------------------------
# Indexer
# ---------------------------------------------------------------------------

class Indexer:
    def __init__(self, store: Path, cache_dir: Path | None = None):
        self.store = store

        # Default cache_dir to ~/Library/Caches/cyberos on macOS (or
        # ~/.cache/cyberos elsewhere) when no explicit path given.
        # Reason: SQLite + iCloud/Dropbox/OneDrive sync conflict on the .db
        # file. ~/Library/Caches/ is excluded from cloud sync by default.
        # Users who want the index inside .cyberos-memory/ pass --cache-dir
        # explicitly OR set the legacy env var CYBEROS_INDEX_IN_STORE=1.
        if cache_dir is None and not os.environ.get("CYBEROS_INDEX_IN_STORE"):
            import sys
            if sys.platform == "darwin":
                cache_dir = Path.home() / "Library" / "Caches" / "cyberos"
            else:
                cache_dir = Path.home() / ".cache" / "cyberos"

        if cache_dir is not None:
            # Per-store fingerprint so multiple stores don't collide
            fp = hashlib.sha256(
                str(store.resolve()).encode("utf-8")).hexdigest()[:16]
            self.index_dir = cache_dir / fp
        else:
            self.index_dir = store / "index"
        self.db_path = self.index_dir / "cyberos.db"

    def connect(self) -> sqlite3.Connection:
        self.index_dir.mkdir(parents=True, exist_ok=True)
        conn = sqlite3.connect(self.db_path)
        conn.execute("PRAGMA foreign_keys = ON")
        # WAL would be faster but isn't supported on all sync mounts
        # (iCloud/Dropbox FUSE drivers occasionally fail mmap-based WAL).
        # DELETE journal works universally; index is regenerable so durability
        # tradeoffs are acceptable.
        conn.execute("PRAGMA journal_mode = DELETE")
        conn.executescript(SCHEMA_SQL)
        # Set schema version on first run
        conn.execute(
            "INSERT OR IGNORE INTO index_meta (key, value) VALUES (?, ?)",
            ("schema_version", SCHEMA_VERSION),
        )
        return conn

    def get_meta(self, conn: sqlite3.Connection, key: str) -> str | None:
        row = conn.execute(
            "SELECT value FROM index_meta WHERE key = ?", (key,)
        ).fetchone()
        return row[0] if row else None

    def set_meta(self, conn: sqlite3.Connection, key: str, value: str) -> None:
        conn.execute(
            "INSERT OR REPLACE INTO index_meta (key, value) VALUES (?, ?)",
            (key, value),
        )

    # -- full build ----------------------------------------------------------

    def build(self, *, full: bool = True) -> dict:
        """Full rebuild from canonical store."""
        if full and self.db_path.exists():
            # On some sandbox/FUSE mounts unlink is denied; fall back to
            # opening + dropping all tables, which is functionally equivalent
            # and works regardless of mount semantics.
            try:
                self.db_path.unlink()
                for f in self.index_dir.glob("cyberos.db-*"):
                    f.unlink()
            except (PermissionError, OSError):
                conn = sqlite3.connect(self.db_path)
                try:
                    cur = conn.execute(
                        "SELECT name FROM sqlite_master WHERE type='table'")
                    for (name,) in cur.fetchall():
                        conn.execute(f"DROP TABLE IF EXISTS {name}")
                    conn.commit()
                finally:
                    conn.close()
        conn = self.connect()
        t0 = time.perf_counter()
        try:
            with conn:
                memories_indexed = self._index_memories(conn)
                audit_indexed = self._index_audit(conn)
                merkle_indexed = self._index_merkle_checkpoints(conn)
                self.set_meta(conn, "last_built_at",
                              dt.datetime.now(dt.timezone.utc)
                              .isoformat(timespec="seconds"))
            elapsed_ms = (time.perf_counter() - t0) * 1000
            return {
                "memories_indexed": memories_indexed,
                "audit_rows_indexed": audit_indexed,
                "merkle_checkpoints_indexed": merkle_indexed,
                "elapsed_ms": round(elapsed_ms, 1),
                "db_size_bytes": self.db_path.stat().st_size,
            }
        finally:
            conn.close()

    def _index_memories(self, conn: sqlite3.Connection) -> int:
        if yaml is None:
            print("warning: pyyaml not installed; cannot index memories")
            return 0
        scope_dirs = ["company", "module", "member", "client", "project",
                      "persona", "memories", "meta"]
        count = 0
        for scope in scope_dirs:
            scope_path = self.store / scope
            if not scope_path.exists():
                continue
            for md in scope_path.rglob("*.md"):
                rel = md.relative_to(self.store).as_posix()
                if md.name == "README.md" or is_exempt(rel):
                    continue
                if self._index_memory_file(conn, md, rel):
                    count += 1
        return count

    def _index_memory_file(self, conn: sqlite3.Connection,
                            path: Path, rel: str) -> bool:
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            return False
        fm_yaml, _body = split_frontmatter(text)
        if fm_yaml is None:
            return False
        try:
            fm = yaml.safe_load(fm_yaml)
        except yaml.YAMLError:
            return False
        if not isinstance(fm, dict):
            return False
        mid = fm.get("memory_id")
        if not mid or not isinstance(mid, str):
            return False

        # Source SHA for ingestion dedup (DEC-080 + ingestion_coverage)
        source_sha = None
        ic = fm.get("ingestion_coverage")
        if isinstance(ic, dict):
            source_sha = ic.get("source_sha256")

        body_sha = "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()

        # Coerce datetime objects to ISO strings (DEC-088)
        def _ts(v):
            if isinstance(v, dt.datetime):
                return v.isoformat()
            return v if isinstance(v, str) else None

        conn.execute(
            "INSERT OR REPLACE INTO memories "
            "(memory_id, file_path, scope, classification, authority, "
            " version, created_at, last_updated_at, body_sha, "
            " tombstoned, source_sha) "
            "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                mid, rel,
                fm.get("scope") if isinstance(fm.get("scope"), str) else None,
                fm.get("classification") if isinstance(
                    fm.get("classification"), str) else None,
                fm.get("authority") if isinstance(
                    fm.get("authority"), str) else None,
                int(fm["version"]) if isinstance(
                    fm.get("version"), int) else None,
                _ts(fm.get("created_at")),
                _ts(fm.get("last_updated_at")),
                body_sha,
                1 if fm.get("tombstoned") else 0,
                source_sha,
            ),
        )

        # Tags
        conn.execute("DELETE FROM tags WHERE memory_id = ?", (mid,))
        tags = fm.get("tags")
        if isinstance(tags, list):
            for t in tags:
                if isinstance(t, str):
                    conn.execute(
                        "INSERT OR IGNORE INTO tags (memory_id, tag) "
                        "VALUES (?, ?)", (mid, t))

        # Relationships
        conn.execute("DELETE FROM relationships WHERE from_id = ?", (mid,))
        rels = fm.get("relationships")
        if isinstance(rels, list):
            for r in rels:
                if isinstance(r, dict):
                    to_id = r.get("relates_to")
                    kind = r.get("kind")
                    if isinstance(to_id, str) and isinstance(kind, str):
                        conn.execute(
                            "INSERT OR IGNORE INTO relationships "
                            "(from_id, to_id, kind) VALUES (?, ?, ?)",
                            (mid, to_id, kind),
                        )

        # Supersedes
        conn.execute("DELETE FROM supersedes WHERE from_id = ?", (mid,))
        sup = fm.get("supersedes")
        if isinstance(sup, str):
            conn.execute("INSERT OR IGNORE INTO supersedes "
                         "(from_id, to_id) VALUES (?, ?)", (mid, sup))
        elif isinstance(sup, list):
            for s in sup:
                if isinstance(s, str):
                    conn.execute("INSERT OR IGNORE INTO supersedes "
                                 "(from_id, to_id) VALUES (?, ?)", (mid, s))
        return True

    def _index_merkle_checkpoints(self, conn: sqlite3.Connection) -> int:
        """Walk audit rows; record every consolidation_run row with merkle_root."""
        rows = conn.execute(
            "SELECT audit_id, ts, op FROM audit_rows ORDER BY ts"
        ).fetchall()
        # Need access to merkle_root field — re-walk JSONL because audit_rows
        # table doesn't carry it
        audit_dir = self.store / "audit"
        if not audit_dir.exists():
            return 0
        prev_audit_id = None
        count = 0
        for ledger in sorted(audit_dir.glob("*.jsonl")):
            if ledger.name.endswith(".compacted.jsonl"):
                continue
            try:
                with ledger.open("r", encoding="utf-8") as f:
                    period_start_audit_id = None
                    leaves = 0
                    for line in f:
                        if not line.strip():
                            continue
                        try:
                            row = json.loads(line)
                        except json.JSONDecodeError:
                            continue
                        if period_start_audit_id is None:
                            period_start_audit_id = row.get("audit_id")
                        if row.get("op") == "consolidation_run" \
                                and "merkle_root" in row:
                            conn.execute(
                                "INSERT OR REPLACE INTO merkle_checkpoints "
                                "(audit_id, ts, root, period_start_audit_id, "
                                " period_end_audit_id, leaves_count) "
                                "VALUES (?, ?, ?, ?, ?, ?)",
                                (
                                    row["audit_id"], row.get("ts", ""),
                                    row["merkle_root"],
                                    period_start_audit_id,
                                    prev_audit_id, leaves,
                                )
                            )
                            count += 1
                            period_start_audit_id = row["audit_id"]
                            leaves = 0
                        else:
                            leaves += 1
                        prev_audit_id = row.get("audit_id")
            except OSError:
                continue
        return count

    def _index_audit(self, conn: sqlite3.Connection) -> int:
        audit_dir = self.store / "audit"
        if not audit_dir.exists():
            return 0
        last_chain: str | None = None
        last_audit_id: str | None = None
        count = 0
        for ledger in sorted(audit_dir.glob("*.jsonl")):
            try:
                with ledger.open("r", encoding="utf-8") as f:
                    for line in f:
                        if not line.strip():
                            continue
                        try:
                            row = json.loads(line)
                        except json.JSONDecodeError:
                            continue
                        aid = row.get("audit_id")
                        if not aid:
                            continue
                        conn.execute(
                            "INSERT OR REPLACE INTO audit_rows "
                            "(audit_id, ts, op, path, memory_id, chain, "
                            " prev_chain, actor_kind, actor_id, ledger) "
                            "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                            (
                                aid, row.get("ts"), row.get("op"),
                                row.get("path"), row.get("memory_id"),
                                row.get("chain"), row.get("prev_chain"),
                                row.get("actor_kind"), row.get("actor_id"),
                                ledger.name,
                            ),
                        )
                        last_chain = row.get("chain")
                        last_audit_id = aid
                        count += 1
            except OSError:
                continue
        if last_audit_id:
            self.set_meta(conn, "last_indexed_audit_id", last_audit_id)
        if last_chain:
            self.set_meta(conn, "last_indexed_chain", last_chain)
        return count

    # -- incremental update --------------------------------------------------

    def update(self) -> dict:
        """Incremental update: index audit rows newer than checkpoint AND
        re-index any memory file whose body_sha has changed."""
        conn = self.connect()
        t0 = time.perf_counter()
        try:
            with conn:
                last_aid = self.get_meta(conn, "last_indexed_audit_id")
                if not last_aid:
                    # Nothing indexed yet — fall back to full
                    return self.build(full=False)

                # 1. Walk audit ledger; index rows with audit_id > last_aid
                #    (we use ts ordering as a proxy since audit_id is UUIDv7
                #    which is time-ordered within ms precision; ts is the
                #    canonical ordering)
                last_row = conn.execute(
                    "SELECT ts FROM audit_rows WHERE audit_id = ?",
                    (last_aid,)
                ).fetchone()
                last_ts = last_row[0] if last_row else "1970-01-01T00:00:00Z"

                audit_indexed = 0
                latest_aid = last_aid
                latest_chain = self.get_meta(conn, "last_indexed_chain")
                audit_dir = self.store / "audit"
                if audit_dir.exists():
                    for ledger in sorted(audit_dir.glob("*.jsonl")):
                        try:
                            with ledger.open("r", encoding="utf-8") as f:
                                for line in f:
                                    if not line.strip():
                                        continue
                                    try:
                                        row = json.loads(line)
                                    except json.JSONDecodeError:
                                        continue
                                    aid = row.get("audit_id")
                                    rts = row.get("ts", "")
                                    if not aid:
                                        continue
                                    # Skip rows already in DB (use ts as
                                    # primary cutoff; INSERT OR IGNORE handles
                                    # exact matches)
                                    if rts <= last_ts and aid != last_aid:
                                        # Verify it's already present
                                        exists = conn.execute(
                                            "SELECT 1 FROM audit_rows "
                                            "WHERE audit_id = ?", (aid,)
                                        ).fetchone()
                                        if exists:
                                            continue
                                    conn.execute(
                                        "INSERT OR IGNORE INTO audit_rows "
                                        "(audit_id, ts, op, path, memory_id, "
                                        " chain, prev_chain, actor_kind, "
                                        " actor_id, ledger) "
                                        "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                        (
                                            aid, row.get("ts"),
                                            row.get("op"), row.get("path"),
                                            row.get("memory_id"),
                                            row.get("chain"),
                                            row.get("prev_chain"),
                                            row.get("actor_kind"),
                                            row.get("actor_id"),
                                            ledger.name,
                                        ),
                                    )
                                    if conn.total_changes:
                                        audit_indexed += 1
                                    if rts > last_ts:
                                        latest_aid = aid
                                        latest_chain = row.get("chain")
                        except OSError:
                            continue

                # 2. Walk memory files; re-index any whose SHA changed
                memories_reindexed = 0
                if yaml:
                    scope_dirs = ["company", "module", "member", "client",
                                  "project", "persona", "memories", "meta"]
                    seen_paths: set[str] = set()
                    for scope in scope_dirs:
                        scope_path = self.store / scope
                        if not scope_path.exists():
                            continue
                        for md in scope_path.rglob("*.md"):
                            rel = md.relative_to(self.store).as_posix()
                            if md.name == "README.md" or is_exempt(rel):
                                continue
                            seen_paths.add(rel)
                            try:
                                text = md.read_text(encoding="utf-8")
                            except (OSError, UnicodeDecodeError):
                                continue
                            cur_sha = "sha256:" + hashlib.sha256(
                                text.encode("utf-8")).hexdigest()
                            row = conn.execute(
                                "SELECT body_sha FROM memories "
                                "WHERE file_path = ?", (rel,)
                            ).fetchone()
                            if row and row[0] == cur_sha:
                                continue  # unchanged
                            if self._index_memory_file(conn, md, rel):
                                memories_reindexed += 1

                    # Detect deleted files (no longer on disk but in index)
                    db_paths = {
                        r[0] for r in conn.execute(
                            "SELECT file_path FROM memories"
                        ).fetchall()
                    }
                    for stale in db_paths - seen_paths:
                        conn.execute(
                            "DELETE FROM memories WHERE file_path = ?",
                            (stale,))

                # 3. Update meta
                if latest_aid:
                    self.set_meta(conn, "last_indexed_audit_id", latest_aid)
                if latest_chain:
                    self.set_meta(conn, "last_indexed_chain", latest_chain)
                self.set_meta(
                    conn, "last_built_at",
                    dt.datetime.now(dt.timezone.utc)
                    .isoformat(timespec="seconds"))

            elapsed_ms = (time.perf_counter() - t0) * 1000
            return {
                "memories_reindexed": memories_reindexed,
                "audit_rows_indexed": audit_indexed,
                "elapsed_ms": round(elapsed_ms, 1),
                "db_size_bytes": self.db_path.stat().st_size,
            }
        finally:
            conn.close()

    # -- verify --------------------------------------------------------------

    def verify(self) -> list[str]:
        """Return list of integrity issues (empty if healthy)."""
        if not self.db_path.exists():
            return ["index does not exist; run `build`"]
        conn = self.connect()
        issues: list[str] = []
        try:
            # Check schema version
            sv = self.get_meta(conn, "schema_version")
            if sv != SCHEMA_VERSION:
                issues.append(
                    f"schema version mismatch (db={sv} tool={SCHEMA_VERSION})")

            # Count memories on disk vs in index
            disk_count = 0
            scope_dirs = ["company", "module", "member", "client", "project",
                          "persona", "memories", "meta"]
            for scope in scope_dirs:
                scope_path = self.store / scope
                if not scope_path.exists():
                    continue
                for md in scope_path.rglob("*.md"):
                    rel = md.relative_to(self.store).as_posix()
                    if md.name == "README.md" or is_exempt(rel):
                        continue
                    disk_count += 1
            db_count = conn.execute(
                "SELECT COUNT(*) FROM memories").fetchone()[0]
            if disk_count != db_count:
                issues.append(
                    f"memory count mismatch: disk={disk_count} db={db_count}")

            # Spot-check 5 random memories' body_sha vs disk
            rows = conn.execute(
                "SELECT memory_id, file_path, body_sha FROM memories "
                "ORDER BY RANDOM() LIMIT 5").fetchall()
            for mid, rel, db_sha in rows:
                disk_path = self.store / rel
                if not disk_path.exists():
                    issues.append(f"index has {mid} but {rel} missing on disk")
                    continue
                cur_sha = "sha256:" + hashlib.sha256(
                    disk_path.read_bytes()).hexdigest()
                if cur_sha != db_sha:
                    issues.append(
                        f"body_sha drift for {mid}: db={db_sha[:24]}... "
                        f"disk={cur_sha[:24]}...")

            # Audit chain LINK invariant verification is intentionally NOT
            # done here — the canonical check is cyberos_validate.py walking
            # the JSONL in file-order. The index uses INSERT OR REPLACE on
            # audit_id which doesn't preserve insertion order across the
            # mixed UUIDv7/ULID audit_ids in older rows. Run cyberos_validate
            # for chain integrity; this verify only checks index consistency.
            audit_count_disk = 0
            audit_dir = self.store / "audit"
            if audit_dir.exists():
                for ledger in audit_dir.glob("*.jsonl"):
                    try:
                        with ledger.open("r", encoding="utf-8") as f:
                            for line in f:
                                if line.strip():
                                    try:
                                        json.loads(line)
                                        audit_count_disk += 1
                                    except json.JSONDecodeError:
                                        pass
                    except OSError:
                        pass
            audit_count_db = conn.execute(
                "SELECT COUNT(*) FROM audit_rows").fetchone()[0]
            if audit_count_disk != audit_count_db:
                issues.append(
                    f"audit row count mismatch: disk={audit_count_disk} "
                    f"db={audit_count_db} (run `build` to refresh)")
        finally:
            conn.close()
        return issues

    # -- stats ---------------------------------------------------------------

    def stats(self) -> dict:
        if not self.db_path.exists():
            return {"error": "index not built; run `build`"}
        conn = self.connect()
        try:
            return {
                "memories_total": conn.execute(
                    "SELECT COUNT(*) FROM memories").fetchone()[0],
                "memories_tombstoned": conn.execute(
                    "SELECT COUNT(*) FROM memories WHERE tombstoned = 1"
                ).fetchone()[0],
                "tags_distinct": conn.execute(
                    "SELECT COUNT(DISTINCT tag) FROM tags").fetchone()[0],
                "tags_total_links": conn.execute(
                    "SELECT COUNT(*) FROM tags").fetchone()[0],
                "relationships": conn.execute(
                    "SELECT COUNT(*) FROM relationships").fetchone()[0],
                "supersedes": conn.execute(
                    "SELECT COUNT(*) FROM supersedes").fetchone()[0],
                "audit_rows": conn.execute(
                    "SELECT COUNT(*) FROM audit_rows").fetchone()[0],
                "last_indexed_audit_id":
                    self.get_meta(conn, "last_indexed_audit_id"),
                "last_built_at":
                    self.get_meta(conn, "last_built_at"),
                "schema_version":
                    self.get_meta(conn, "schema_version"),
                "db_size_bytes": self.db_path.stat().st_size,
            }
        finally:
            conn.close()

    # -- queries -------------------------------------------------------------

    def query_tag(self, tag: str, *, include_tombstoned: bool = False) -> list[dict]:
        conn = self.connect()
        try:
            sql = """
                SELECT m.memory_id, m.file_path, m.scope, m.classification,
                       m.authority
                  FROM tags t
                  JOIN memories m ON m.memory_id = t.memory_id
                 WHERE t.tag = ?
            """
            if not include_tombstoned:
                sql += " AND m.tombstoned = 0"
            sql += " ORDER BY m.last_updated_at DESC"
            return [dict(zip(
                ("memory_id", "file_path", "scope", "classification",
                 "authority"), r))
                    for r in conn.execute(sql, (tag,)).fetchall()]
        finally:
            conn.close()

    def query_relates_to(self, memory_id: str) -> dict:
        """Return inbound + outbound relationships for memory_id."""
        conn = self.connect()
        try:
            inbound = [dict(zip(("from_id", "kind"), r))
                       for r in conn.execute(
                "SELECT from_id, kind FROM relationships WHERE to_id = ?",
                (memory_id,)).fetchall()]
            outbound = [dict(zip(("to_id", "kind"), r))
                        for r in conn.execute(
                "SELECT to_id, kind FROM relationships WHERE from_id = ?",
                (memory_id,)).fetchall()]
            supersedes = [r[0] for r in conn.execute(
                "SELECT to_id FROM supersedes WHERE from_id = ?",
                (memory_id,)).fetchall()]
            superseded_by = [r[0] for r in conn.execute(
                "SELECT from_id FROM supersedes WHERE to_id = ?",
                (memory_id,)).fetchall()]
            return {
                "memory_id": memory_id,
                "inbound": inbound,
                "outbound": outbound,
                "supersedes": supersedes,
                "superseded_by": superseded_by,
            }
        finally:
            conn.close()

    def query_source_sha(self, source_sha: str) -> list[dict]:
        conn = self.connect()
        try:
            return [dict(zip(("memory_id", "file_path"), r))
                    for r in conn.execute(
                "SELECT memory_id, file_path FROM memories "
                "WHERE source_sha = ?", (source_sha,)).fetchall()]
        finally:
            conn.close()

    def query_audit_by_path(self, path: str, *, limit: int = 20) -> list[dict]:
        conn = self.connect()
        try:
            # Accept both ".cyberos-memory/foo.md" and "foo.md" forms
            patterns = [path, f".cyberos-memory/{path}"]
            placeholders = ",".join("?" * len(patterns))
            sql = (f"SELECT audit_id, ts, op, chain FROM audit_rows "
                   f"WHERE path IN ({placeholders}) "
                   f"ORDER BY ts DESC LIMIT ?")
            return [dict(zip(
                ("audit_id", "ts", "op", "chain"), r))
                    for r in conn.execute(sql, (*patterns, limit)).fetchall()]
        finally:
            conn.close()

    def query_merkle_proof(self, chain: str) -> dict:
        """Return Merkle inclusion proof + checkpoint root for a given chain."""
        conn = self.connect()
        try:
            # Find the row carrying this chain
            row = conn.execute(
                "SELECT audit_id, ts FROM audit_rows WHERE chain = ?",
                (chain,)
            ).fetchone()
            if not row:
                return {"error": f"chain {chain} not found in audit_rows"}
            target_audit_id, target_ts = row

            # Find the next checkpoint after this row
            checkpoint = conn.execute(
                "SELECT audit_id, root, period_start_audit_id "
                "FROM merkle_checkpoints WHERE ts >= ? "
                "ORDER BY ts ASC LIMIT 1",
                (target_ts,)
            ).fetchone()
            if not checkpoint:
                return {
                    "chain": chain,
                    "audit_id": target_audit_id,
                    "error": "no Merkle checkpoint exists for this row yet "
                             "(run a consolidation_run first)",
                }
            cp_audit_id, cp_root, period_start = checkpoint

            # Collect all leaves in the period
            period_rows = conn.execute(
                "SELECT chain FROM audit_rows "
                "WHERE ts > (SELECT ts FROM audit_rows WHERE audit_id = ?) "
                "AND ts < ? "
                "ORDER BY ts ASC",
                (period_start, cp_audit_id_to_ts := target_ts.split('@')[0])
            ).fetchall() if period_start else []
            leaves = [r[0] for r in period_rows]

            # Build proof: walk Merkle tree, record sibling at each level
            try:
                idx = leaves.index(chain)
            except ValueError:
                return {
                    "chain": chain,
                    "audit_id": target_audit_id,
                    "error": "chain not in any indexed Merkle period (compacted?)",
                }

            level = [bytes.fromhex(c.replace("sha256:", "")) for c in leaves]
            proof = []
            cur_idx = idx
            while len(level) > 1:
                if len(level) % 2:
                    level.append(level[-1])
                # Sibling
                sibling_idx = cur_idx ^ 1  # XOR 1 flips lowest bit
                sibling = level[sibling_idx]
                proof.append({
                    "hash": "sha256:" + sibling.hex(),
                    "position": "left" if sibling_idx < cur_idx else "right",
                })
                # Build next level
                level = [hashlib.sha256(level[i] + level[i + 1]).digest()
                         for i in range(0, len(level), 2)]
                cur_idx //= 2
            return {
                "chain": chain,
                "audit_id": target_audit_id,
                "checkpoint_audit_id": cp_audit_id,
                "checkpoint_root": cp_root,
                "proof": proof,
                "leaves_in_period": len(leaves),
            }
        finally:
            conn.close()

    def query_tombstoned(self, memory_id: str) -> bool:
        conn = self.connect()
        try:
            row = conn.execute(
                "SELECT tombstoned FROM memories WHERE memory_id = ?",
                (memory_id,)).fetchone()
            return bool(row and row[0])
        finally:
            conn.close()


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="cyberos-index",
        description="Local search index for .cyberos-memory/ stores.",
    )
    parser.add_argument("path")
    parser.add_argument(
        "--cache-dir",
        help="Store index outside .cyberos-memory/index/ (e.g. "
             "~/.cache/cyberos). Useful when the filesystem under "
             "the store doesn't support SQLite (some FUSE/cloud mounts).")
    sub = parser.add_subparsers(dest="cmd")
    sub.add_parser("build", help="Full rebuild from canonical store")
    sub.add_parser("update", help="Incremental update (default)")
    sub.add_parser("verify", help="Check index integrity")
    sub.add_parser("stats", help="Show index statistics")

    qp = sub.add_parser("query", help="Query the index")
    qsub = qp.add_subparsers(dest="qcmd")
    q_tag = qsub.add_parser("tag")
    q_tag.add_argument("tag")
    q_tag.add_argument("--include-tombstoned", action="store_true")
    q_rel = qsub.add_parser("relates-to")
    q_rel.add_argument("memory_id")
    q_src = qsub.add_parser("source-sha")
    q_src.add_argument("sha")
    q_aud = qsub.add_parser("audit-by-path")
    q_aud.add_argument("target_path", help="Path inside .cyberos-memory/")
    q_aud.add_argument("--limit", type=int, default=20)
    q_tomb = qsub.add_parser("tombstoned")
    q_tomb.add_argument("memory_id")
    q_mp = qsub.add_parser("merkle-proof")
    q_mp.add_argument("chain", help="sha256:... chain value to derive proof for")

    args = parser.parse_args(argv)

    store = Path(args.path).resolve()
    if (store / ".cyberos-memory").is_dir():
        store = store / ".cyberos-memory"
    if not store.is_dir():
        print(f"error: {store} not a directory", file=sys.stderr)
        return 3

    cache_dir = Path(args.cache_dir).expanduser() if args.cache_dir else None
    idx = Indexer(store, cache_dir=cache_dir)
    cmd = args.cmd or "update"

    if cmd == "build":
        result = idx.build(full=True)
        print(json.dumps(result, indent=2))
        return 0
    if cmd == "update":
        result = idx.update()
        print(json.dumps(result, indent=2))
        return 0
    if cmd == "verify":
        issues = idx.verify()
        if issues:
            print(f"❌ {len(issues)} issue(s):")
            for i in issues:
                print(f"  - {i}")
            return 2
        print("✅ index is consistent with canonical store")
        return 0
    if cmd == "stats":
        print(json.dumps(idx.stats(), indent=2))
        return 0
    if cmd == "query":
        if args.qcmd == "tag":
            r = idx.query_tag(args.tag,
                              include_tombstoned=args.include_tombstoned)
        elif args.qcmd == "relates-to":
            r = idx.query_relates_to(args.memory_id)
        elif args.qcmd == "source-sha":
            r = idx.query_source_sha(args.sha)
        elif args.qcmd == "audit-by-path":
            r = idx.query_audit_by_path(args.target_path, limit=args.limit)
        elif args.qcmd == "tombstoned":
            r = {"memory_id": args.memory_id,
                 "tombstoned": idx.query_tombstoned(args.memory_id)}
        elif args.qcmd == "merkle-proof":
            r = idx.query_merkle_proof(args.chain)
        else:
            qp.print_help()
            return 3
        print(json.dumps(r, indent=2, default=str))
        return 0

    parser.print_help()
    return 3


if __name__ == "__main__":
    sys.exit(main())
