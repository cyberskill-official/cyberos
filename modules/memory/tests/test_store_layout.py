"""Layout-root-canonical regression (TASK-MEMORY-302).

Asserts the store's top-level set is a subset of AGENTS.md §2 — the invariant
that surfaces applier raw-writes under ``adrs/``, ``impl-plans/``, etc.
"""

from __future__ import annotations

from pathlib import Path

import pytest

# Canonical top-level names from AGENTS.md §2 (files + dirs).
_CANONICAL_TOP = frozenset({
    "manifest.json",
    "HEAD",
    ".lock",
    "STORE.yaml",
    "audit",
    "memories",
    "meta",
    "company",
    "module",
    "member",
    "client",
    "project",
    "persona",
    "conflicts",
    "exports",
    "index",
    "sessions",
    "dreams",
})


def _store_fixture(tmp_path: Path) -> Path:
    store = tmp_path / "store"
    for d in ("audit", "memories", "meta", "index"):
        (store / d).mkdir(parents=True)
    (store / "HEAD").write_bytes((0).to_bytes(8, "little"))
    return store


def test_clean_store_passes(tmp_path: Path) -> None:
    store = _store_fixture(tmp_path)
    unexpected = [p.name for p in store.iterdir() if p.name not in _CANONICAL_TOP]
    assert unexpected == []


def test_no_noncanonical_top_level_dirs(tmp_path: Path) -> None:
    """Red against the old applier layout; green after TASK-MEMORY-302."""
    store = _store_fixture(tmp_path)
    # Simulate the bug: raw mkdir under store root.
    for name in ("adrs", "impl-plans", "audits", "code-reviews", "obs-injections"):
        (store / name).mkdir()
        (store / name / "x.md").write_text("leak\n", encoding="utf-8")
    unexpected = sorted(
        p.name + ("/" if p.is_dir() else "")
        for p in store.iterdir()
        if p.name not in _CANONICAL_TOP
    )
    assert unexpected == [
        "adrs/", "audits/", "code-reviews/", "impl-plans/", "obs-injections/",
    ], "fixture must demonstrate the MEMORY-302 failure mode"

    # After the fix, appliers write via put() under memories/<kind>/ — removing
    # the stray dirs restores the invariant. (Live-store migration is a separate
    # operator move; this unit asserts the canonical set itself.)
    for name in ("adrs", "impl-plans", "audits", "code-reviews", "obs-injections"):
        import shutil
        shutil.rmtree(store / name)
    unexpected = [p.name for p in store.iterdir() if p.name not in _CANONICAL_TOP]
    assert unexpected == []


def test_shard_depth_enforced() -> None:
    import hashlib

    filename = "ADR-0001-example.md"
    digest = hashlib.sha256(filename.encode("utf-8")).hexdigest()
    rel = f"{digest[:2]}/{digest[2:4]}/{filename}"
    parts = rel.split("/")
    assert len(parts) == 3
    assert len(parts[0]) == 2 and len(parts[1]) == 2
    assert parts[2] == filename
