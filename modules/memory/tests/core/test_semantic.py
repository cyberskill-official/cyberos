"""Tests for cyberos.core.semantic (PROPOSAL.md P7).

We do NOT require sentence-transformers in CI — it's a 100 MB+ optional
dep. Instead the heavy tests (sync / search) inject a deterministic fake
model via monkeypatch so the path is fully exercised on every test run.
"""

from __future__ import annotations

import hashlib
import sqlite3
from pathlib import Path

import pytest

from cyberos.core import semantic


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------


def _store(tmp_path) -> Path:
    s = tmp_path / ".cyberos/memory/store"
    s.mkdir(parents=True, exist_ok=True)
    (s / "audit").mkdir(parents=True, exist_ok=True)
    (s / "manifest.json").write_text('{}', encoding="utf-8")
    return s


def _write_memory(store: Path, rel: str, body: str) -> None:
    import msgspec
    fm = {"id": rel, "kind": "fact", "ts_ns": 1, "actor": "s", "tags": []}
    fm_bytes = msgspec.json.encode(fm, order="sorted")
    path = store / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"---\n" + fm_bytes + b"\n---\n" + body.encode("utf-8"))


class _FakeModel:
    """Deterministic hash-based 'embeddings' for testing.

    Maps each text to an 8-dim float vector derived from the SHA-256 of
    the text. Stable, deterministic, no external dependency.
    """
    def encode(self, texts, show_progress_bar=False):
        import numpy as np
        out = np.zeros((len(texts), 8), dtype=np.float32)
        for i, t in enumerate(texts):
            digest = hashlib.sha256(t.encode("utf-8")).digest()
            # 8 floats from the first 32 bytes of the digest
            for j in range(8):
                out[i, j] = (digest[j * 4] - 128) / 128.0
        return out


@pytest.fixture
def fake_semantic(monkeypatch, tmp_path):
    """Force semantic.available() True and inject a deterministic fake model.

    Also redirect the cache dir to tmp_path so the SQLite DB lives in the
    test sandbox and doesn't pollute the user's real cache.
    """
    monkeypatch.setattr(semantic, "_AVAILABLE", True)
    monkeypatch.setattr(semantic, "_MODEL_CACHE", _FakeModel())
    # Redirect cache_dir() to point inside tmp_path. The function is
    # imported at runtime in semantic.open_index, so we patch the source.
    from cyberos.core import index as _idx
    monkeypatch.setattr(_idx, "cache_dir", lambda: tmp_path / "cache")
    yield
    # Reset module-level cache so the next test re-probes cleanly.
    semantic._AVAILABLE = None
    semantic._MODEL_CACHE = None


# ---------------------------------------------------------------------------
# probe
# ---------------------------------------------------------------------------


def test_available_returns_bool():
    # Don't mutate state — call available() and accept either result.
    result = semantic.available()
    assert isinstance(result, bool)


def test_sync_raises_when_unavailable(tmp_path, monkeypatch):
    monkeypatch.setattr(semantic, "_AVAILABLE", False)
    monkeypatch.setattr(semantic, "_MODEL_CACHE", None)
    store = _store(tmp_path)
    with pytest.raises(RuntimeError, match="dependencies missing"):
        semantic.sync(store)
    semantic._AVAILABLE = None


def test_search_returns_empty_when_unavailable(tmp_path, monkeypatch):
    monkeypatch.setattr(semantic, "_AVAILABLE", False)
    monkeypatch.setattr(semantic, "_MODEL_CACHE", None)
    store = _store(tmp_path)
    assert semantic.search(store, "anything") == []
    semantic._AVAILABLE = None


# ---------------------------------------------------------------------------
# quantisation
# ---------------------------------------------------------------------------


def test_quantise_int8_round_trip(fake_semantic):
    import numpy as np
    vec = np.array([0.1, -0.5, 0.3, 1.0, -1.0], dtype=np.float32)
    scale, blob = semantic._quantise_int8(vec)
    rec = semantic._dequantise_int8(scale, blob)
    # Loose tolerance — int8 quantisation has roughly ±1/127 error.
    assert len(rec) == 5
    for got, want in zip(rec, vec):
        assert abs(got - want) <= 0.02


def test_cosine_orthogonal_vectors(fake_semantic):
    import numpy as np
    a = np.array([1.0, 0.0, 0.0], dtype=np.float32)
    b = np.array([0.0, 1.0, 0.0], dtype=np.float32)
    assert abs(semantic._cosine(a, b)) < 1e-6


def test_cosine_identical_vectors(fake_semantic):
    import numpy as np
    a = np.array([1.0, 2.0, 3.0], dtype=np.float32)
    assert abs(semantic._cosine(a, a) - 1.0) < 1e-6


def test_cosine_zero_vectors_safe(fake_semantic):
    import numpy as np
    z = np.zeros(3, dtype=np.float32)
    assert semantic._cosine(z, z) == 0.0


# ---------------------------------------------------------------------------
# sync
# ---------------------------------------------------------------------------


def test_sync_indexes_all_memories(fake_semantic, tmp_path):
    store = _store(tmp_path)
    _write_memory(store, "memories/facts/a.md", "first memory")
    _write_memory(store, "memories/facts/b.md", "second memory")
    _write_memory(store, "memories/decisions/c.md", "third decision")

    report = semantic.sync(store)
    assert report.indexed == 3
    assert report.skipped_unchanged == 0
    assert report.removed == 0
    assert report.total_in_index == 3


def test_sync_is_incremental(fake_semantic, tmp_path):
    store = _store(tmp_path)
    _write_memory(store, "memories/facts/a.md", "original")
    _write_memory(store, "memories/facts/b.md", "second")

    r1 = semantic.sync(store)
    assert r1.indexed == 2

    # Second run with no changes
    r2 = semantic.sync(store)
    assert r2.indexed == 0
    assert r2.skipped_unchanged == 2


def test_sync_re_embeds_modified_body(fake_semantic, tmp_path):
    store = _store(tmp_path)
    _write_memory(store, "memories/facts/a.md", "v1 body")
    semantic.sync(store)

    _write_memory(store, "memories/facts/a.md", "v2 body changed")
    r = semantic.sync(store)
    assert r.indexed == 1
    assert r.skipped_unchanged == 0


def test_sync_removes_deleted_memories(fake_semantic, tmp_path):
    store = _store(tmp_path)
    _write_memory(store, "memories/facts/a.md", "alpha")
    _write_memory(store, "memories/facts/b.md", "beta")
    semantic.sync(store)

    (store / "memories" / "facts" / "b.md").unlink()
    r = semantic.sync(store)
    assert r.removed == 1
    assert r.total_in_index == 1


def test_sync_skips_unparseable(fake_semantic, tmp_path):
    store = _store(tmp_path)
    _write_memory(store, "memories/facts/good.md", "good body")
    (store / "memories" / "facts" / "bad.md").write_bytes(b"")  # empty body still parses

    r = semantic.sync(store)
    # Both files are technically "readable" — the parser returns the raw
    # content unchanged. We just verify sync doesn't crash.
    assert r.indexed >= 1


# ---------------------------------------------------------------------------
# search
# ---------------------------------------------------------------------------


def test_search_returns_sorted_hits(fake_semantic, tmp_path):
    store = _store(tmp_path)
    _write_memory(store, "memories/facts/a.md", "the quick brown fox")
    _write_memory(store, "memories/facts/b.md", "the lazy dog")
    semantic.sync(store)

    hits = semantic.search(store, "the quick brown fox", limit=10)
    assert len(hits) == 2
    # Best match should be the identical text — score order is
    # monotonically decreasing
    for i in range(len(hits) - 1):
        assert hits[i].score >= hits[i + 1].score


def test_search_limit_respected(fake_semantic, tmp_path):
    store = _store(tmp_path)
    for i in range(10):
        _write_memory(store, f"memories/facts/m{i}.md", f"memory number {i}")
    semantic.sync(store)

    hits = semantic.search(store, "memory", limit=3)
    assert len(hits) == 3


def test_search_empty_when_index_empty(fake_semantic, tmp_path):
    store = _store(tmp_path)
    # No memories, but also no sync called
    hits = semantic.search(store, "anything")
    assert hits == []


def test_search_includes_snippet(fake_semantic, tmp_path):
    store = _store(tmp_path)
    body = "first line of body\nsecond line"
    _write_memory(store, "memories/facts/a.md", body)
    semantic.sync(store)

    hits = semantic.search(store, "first line")
    assert hits
    assert "first line" in hits[0].snippet
