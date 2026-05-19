"""
cyberos.core.semantic — optional local semantic search (PROPOSAL.md P7).

This module is OPTIONAL. Without ``sentence-transformers`` installed,
``available()`` returns False and the CLI falls back to FTS5. With the
dependency present, ``cyberos search --semantic <query>`` runs a local
cosine-similarity search over int8-quantized embeddings stored beside
the FTS5 DB.

Design choices:

* Model: ``sentence-transformers/all-MiniLM-L6-v2`` — 22 MB, 384-dim,
  CPU-fast, well-rounded on English short text. Configurable via
  ``CYBEROS_EMBED_MODEL``.
* Storage: SQLite ``embeddings`` table colocated with the FTS5 index. We
  store the body SHA-256 so we only re-embed memories whose body changed
  since the last run.
* Quantization: int8 vectors (~ 1/4 the disk vs float32) with per-vector
  scale; cosine similarity reconstructed at query time. 384*1 byte +
  4-byte scale = 388 bytes per memory — a 10k-memory memory is ~3.8 MB.
* Privacy: 100% local. The model is downloaded once into the user's HF
  cache. No network at query time.

Falling back gracefully: every public function checks ``available()`` and
returns a typed result that the CLI can render as either "found 7 hits"
or "semantic search unavailable; install with `pip install
sentence-transformers`".
"""

from __future__ import annotations

import hashlib
import json
import os
import sqlite3
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

# ---------------------------------------------------------------------------
# soft dependency probe
# ---------------------------------------------------------------------------


_AVAILABLE: bool | None = None
_MODEL_CACHE: object | None = None


def available() -> bool:
    """True iff ``sentence-transformers`` and ``numpy`` are importable.

    Result is memoised — repeated CLI calls don't re-probe imports.
    """
    global _AVAILABLE
    if _AVAILABLE is not None:
        return _AVAILABLE
    try:
        import sentence_transformers  # noqa: F401
        import numpy  # noqa: F401
    except ImportError:
        _AVAILABLE = False
    else:
        _AVAILABLE = True
    return _AVAILABLE


def _default_model_name() -> str:
    return os.environ.get(
        "CYBEROS_EMBED_MODEL",
        "sentence-transformers/all-MiniLM-L6-v2",
    )


def _get_model() -> object:
    """Load (lazy, cached) the sentence-transformer model."""
    global _MODEL_CACHE
    if _MODEL_CACHE is not None:
        return _MODEL_CACHE
    if not available():
        raise RuntimeError(
            "semantic search dependencies missing; install with "
            "`pip install sentence-transformers --break-system-packages`"
        )
    from sentence_transformers import SentenceTransformer
    _MODEL_CACHE = SentenceTransformer(_default_model_name())
    return _MODEL_CACHE


# ---------------------------------------------------------------------------
# storage
# ---------------------------------------------------------------------------


_EMBED_SCHEMA = """
CREATE TABLE IF NOT EXISTS embeddings(
    rel_path       TEXT PRIMARY KEY,
    body_sha256    TEXT NOT NULL,
    model_name     TEXT NOT NULL,
    dim            INTEGER NOT NULL,
    scale          REAL NOT NULL,
    vector_int8    BLOB NOT NULL,
    indexed_at_ns  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_embed_sha ON embeddings(body_sha256);
"""


def open_index(fingerprint: str) -> sqlite3.Connection:
    """Open the embeddings DB beside the FTS5 index, creating it if needed."""
    from cyberos.core.index import cache_dir
    dbpath = cache_dir() / f"embeddings-{fingerprint}.sqlite"
    dbpath.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(dbpath))
    conn.executescript("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
    conn.executescript(_EMBED_SCHEMA)
    return conn


# ---------------------------------------------------------------------------
# quantisation
# ---------------------------------------------------------------------------


def _quantise_int8(vec):
    """Map a float32 vector to int8 with one shared scale.

    Returns (scale: float, bytes). Reconstruction: ``float = int / scale``.
    """
    import numpy as np
    abs_max = float(np.max(np.abs(vec))) or 1.0
    scale = 127.0 / abs_max
    q = np.clip(np.round(vec * scale), -127, 127).astype(np.int8)
    return float(scale), q.tobytes()


def _dequantise_int8(scale: float, blob: bytes):
    import numpy as np
    return np.frombuffer(blob, dtype=np.int8).astype(np.float32) / scale


def _cosine(a, b) -> float:
    import numpy as np
    na = float(np.linalg.norm(a))
    nb = float(np.linalg.norm(b))
    if na == 0.0 or nb == 0.0:
        return 0.0
    return float(np.dot(a, b) / (na * nb))


# ---------------------------------------------------------------------------
# embed + sync
# ---------------------------------------------------------------------------


@dataclass
class EmbeddingHit:
    rel_path: str
    score: float
    snippet: str


@dataclass
class SyncReport:
    indexed: int
    skipped_unchanged: int
    removed: int
    total_in_index: int


def _iter_memory_files(store: Path) -> Iterable[Path]:
    """Yield memory .md files under the canonical roots."""
    roots = ("memories", "company", "module", "member", "client", "project", "persona")
    for r in roots:
        base = store / r
        if not base.is_dir():
            continue
        for p in sorted(base.rglob("*.md")):
            if p.is_file():
                yield p


def _parse_body(path: Path) -> str | None:
    """Read the memory body (everything after the closing `---`)."""
    try:
        raw = path.read_bytes()
    except OSError:
        return None
    # Split on the first two "---\n" markers — the simple textual frame.
    parts = raw.split(b"\n---\n", 1)
    if len(parts) != 2:
        # Try opening "---\n" prefix
        if raw.startswith(b"---\n"):
            inner = raw[4:].split(b"\n---\n", 1)
            if len(inner) == 2:
                return inner[1].decode("utf-8", errors="replace")
        return raw.decode("utf-8", errors="replace")
    return parts[1].decode("utf-8", errors="replace")


def sync(store: Path, *, model_name: str | None = None, batch_size: int = 32) -> SyncReport:
    """Re-embed memories whose body_sha256 doesn't match the cached vector.

    Memories that no longer exist on disk are dropped from the embeddings
    table. Returns a SyncReport so the CLI can report counts.

    Raises ``RuntimeError`` if the optional deps are missing.
    """
    if not available():
        raise RuntimeError(
            "semantic search dependencies missing; install with "
            "`pip install sentence-transformers --break-system-packages`"
        )
    import time
    model = _get_model()
    name = model_name or _default_model_name()
    fingerprint = hashlib.sha256(str(store.resolve()).encode("utf-8")).hexdigest()[:16]
    conn = open_index(fingerprint)

    # Build (rel_path, body_sha, body) lists; skip unparseable files.
    on_disk: dict[str, tuple[str, str]] = {}
    for path in _iter_memory_files(store):
        body = _parse_body(path)
        if body is None:
            continue
        rel = path.relative_to(store).as_posix()
        sha = hashlib.sha256(body.encode("utf-8")).hexdigest()
        on_disk[rel] = (sha, body)

    # Current embedding state
    cur = conn.execute(
        "SELECT rel_path, body_sha256, model_name FROM embeddings"
    ).fetchall()
    cur_map = {r[0]: (r[1], r[2]) for r in cur}

    # Decide what to (re-)embed
    to_index: list[tuple[str, str, str]] = []  # (rel_path, sha, body)
    skipped = 0
    for rel, (sha, body) in on_disk.items():
        existing = cur_map.get(rel)
        if existing and existing[0] == sha and existing[1] == name:
            skipped += 1
            continue
        to_index.append((rel, sha, body))

    # Embed in batches
    indexed = 0
    if to_index:
        for i in range(0, len(to_index), batch_size):
            chunk = to_index[i:i + batch_size]
            bodies = [b for _r, _s, b in chunk]
            vectors = model.encode(bodies, show_progress_bar=False)
            now_ns = time.time_ns()
            with conn:
                for (rel, sha, _body), vec in zip(chunk, vectors):
                    scale, blob = _quantise_int8(vec)
                    conn.execute(
                        "INSERT OR REPLACE INTO embeddings"
                        " (rel_path, body_sha256, model_name, dim, scale,"
                        "  vector_int8, indexed_at_ns)"
                        " VALUES (?, ?, ?, ?, ?, ?, ?)",
                        (rel, sha, name, len(vec), scale, blob, now_ns),
                    )
                    indexed += 1

    # Remove stale entries
    on_disk_paths = set(on_disk.keys())
    stale = [rel for rel in cur_map if rel not in on_disk_paths]
    if stale:
        with conn:
            conn.executemany(
                "DELETE FROM embeddings WHERE rel_path = ?",
                [(s,) for s in stale],
            )

    total = conn.execute("SELECT COUNT(*) FROM embeddings").fetchone()[0]
    conn.close()
    return SyncReport(
        indexed=indexed,
        skipped_unchanged=skipped,
        removed=len(stale),
        total_in_index=total,
    )


def search(store: Path, query: str, *, limit: int = 20) -> list[EmbeddingHit]:
    """Cosine-similarity search the embeddings index for ``query``.

    Returns hits sorted by descending score. Empty list if the index is
    empty or the optional deps are missing — callers should check
    ``available()`` first if they want to distinguish "no hits" from
    "feature unavailable".
    """
    if not available():
        return []
    import numpy as np
    fingerprint = hashlib.sha256(str(store.resolve()).encode("utf-8")).hexdigest()[:16]
    conn = open_index(fingerprint)
    rows = conn.execute(
        "SELECT rel_path, scale, vector_int8 FROM embeddings"
    ).fetchall()
    conn.close()
    if not rows:
        return []

    model = _get_model()
    qvec = model.encode([query], show_progress_bar=False)[0]
    qvec = np.asarray(qvec, dtype=np.float32)

    scored: list[tuple[float, str]] = []
    for rel_path, scale, blob in rows:
        vec = _dequantise_int8(scale, blob)
        score = _cosine(qvec, vec)
        scored.append((score, rel_path))
    scored.sort(key=lambda x: -x[0])

    # Synthesize a snippet from disk for the top-K
    hits: list[EmbeddingHit] = []
    for score, rel_path in scored[:limit]:
        body = _parse_body(store / rel_path) or ""
        snippet = body.strip().split("\n", 1)[0][:200]
        hits.append(EmbeddingHit(
            rel_path=rel_path, score=score, snippet=snippet,
        ))
    return hits


__all__ = [
    "EmbeddingHit",
    "SyncReport",
    "available",
    "open_index",
    "search",
    "sync",
]
