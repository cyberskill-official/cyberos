"""Tests for cyberos.core.conflicts (PROPOSAL.md P9)."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cyberos.core import conflicts


# ---------------------------------------------------------------------------
# pattern classifier
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("filename,expected_base,expected_source", [
    # Syncthing
    ("notes.sync-conflict-20251225-103145-AB12CD3.md", "notes.md", "syncthing"),
    # Dropbox
    ("notes (conflict).md", "notes.md", "dropbox-or-gdrive"),
    ("notes (conflict 2).md", "notes.md", "dropbox-or-gdrive"),
    ("notes (conflicted copy).md", "notes.md", "dropbox-or-gdrive"),
    ("notes (Stephen's conflicted copy 2025-12-25).md", "notes.md", "dropbox"),
    # Google Drive
    ("notes (Conflicted copy 2025-12-25).md", "notes.md", "dropbox-or-gdrive"),
    # Box
    ("notes (Conflicted copy with Stephen Cheng 2025-12-25).md", "notes.md",
     "dropbox-or-gdrive"),
    # OneDrive
    ("notes - Stephens-MacBook.md", "notes.md", "onedrive"),
    # iCloud / Mac suffix
    ("notes (Mac).md", "notes.md", "icloud"),
    ("notes (iPhone).md", "notes.md", "icloud"),
    # Resilio / .bak suffix
    ("notes.md.bak", "notes.md", "backup"),
])
def test_classify_known_patterns(filename, expected_base, expected_source):
    base, source = conflicts.classify_sibling(filename)
    assert base == expected_base
    assert source == expected_source


@pytest.mark.parametrize("filename", [
    "notes.md",
    "README.md",
    "manifest.json",
    "audit-2025-12.binlog",
    # Plain trailing number (Finder) — too noisy, we deliberately don't match
    "notes 2.md",
    # OneDrive pattern needs a dash in hostname — "foo - bar.md" alone shouldn't match
    "notes - companion.md",
])
def test_classify_non_conflicts(filename):
    base, source = conflicts.classify_sibling(filename)
    assert base is None, f"unexpectedly classified {filename!r} as {base!r}/{source!r}"
    assert source is None


# ---------------------------------------------------------------------------
# scan over a synthetic store
# ---------------------------------------------------------------------------


def _make_store(tmp_path: Path) -> Path:
    """Make a minimal store skeleton with a memories dir."""
    store = tmp_path / ".cyberos-memory"
    (store / "audit").mkdir(parents=True)
    (store / "memories" / "facts").mkdir(parents=True)
    (store / "memories" / "decisions").mkdir(parents=True)
    (store / "conflicts").mkdir(parents=True)
    (store / "index").mkdir(parents=True)
    (store / "exports").mkdir(parents=True)
    (store / "manifest.json").write_text('{"schema_version":2}', encoding="utf-8")
    return store


def test_scan_groups_siblings_by_canonical(tmp_path):
    store = _make_store(tmp_path)
    facts = store / "memories" / "facts"

    canonical = facts / "notes.md"
    canonical.write_text("# canonical\nhello\n", encoding="utf-8")

    sibling1 = facts / "notes (conflict).md"
    sibling1.write_text("# canonical\nhello from machine A\n", encoding="utf-8")

    sibling2 = facts / "notes.sync-conflict-20251225-103145-AB12CD3.md"
    sibling2.write_text("# canonical\nhello from machine B\n", encoding="utf-8")

    pairs = conflicts.scan(store)
    assert len(pairs) == 1
    pair = pairs[0]
    assert pair.canonical == canonical
    sources = {s for (_, s) in pair.siblings}
    assert sources == {"dropbox-or-gdrive", "syncthing"}


def test_scan_skips_audit_index_exports(tmp_path):
    """conflict-shaped names in audit/index/exports/conflicts MUST be ignored."""
    store = _make_store(tmp_path)
    # Drop conflict-looking files where the scanner MUST NOT find them.
    (store / "audit" / "trace (conflict).md").write_text("ignored", encoding="utf-8")
    (store / "index" / "x (Mac).md").write_text("ignored", encoding="utf-8")
    (store / "exports" / "x.sync-conflict-20251225-103145-AB12CD3.md").write_text(
        "ignored", encoding="utf-8")
    (store / "conflicts" / "x.md.bak").write_text("already archived", encoding="utf-8")

    pairs = conflicts.scan(store)
    assert pairs == []


def test_scan_ignores_root_hygiene_files(tmp_path):
    store = _make_store(tmp_path)
    # These have no conflict-shape names, but verify scan() doesn't choke
    (store / ".DS_Store").write_bytes(b"\x00")
    (store / "README.md").write_text("readme", encoding="utf-8")
    assert conflicts.scan(store) == []


# ---------------------------------------------------------------------------
# diff
# ---------------------------------------------------------------------------


def test_diff_reports_changes(tmp_path):
    store = _make_store(tmp_path)
    facts = store / "memories" / "facts"
    canonical = facts / "notes.md"
    sibling = facts / "notes (conflict).md"
    canonical.write_text("alpha\nbeta\ngamma\n", encoding="utf-8")
    sibling.write_text("alpha\nBETA\ngamma\n", encoding="utf-8")

    d = conflicts.diff(canonical, sibling)
    assert "-beta" in d
    assert "+BETA" in d


def test_diff_empty_when_identical(tmp_path):
    store = _make_store(tmp_path)
    facts = store / "memories" / "facts"
    canonical = facts / "notes.md"
    sibling = facts / "notes (conflict).md"
    canonical.write_text("alpha\nbeta\n", encoding="utf-8")
    sibling.write_text("alpha\nbeta\n", encoding="utf-8")
    assert conflicts.diff(canonical, sibling) == ""


def test_diff_when_canonical_missing(tmp_path):
    store = _make_store(tmp_path)
    facts = store / "memories" / "facts"
    sibling = facts / "notes (conflict).md"
    sibling.write_text("orphaned", encoding="utf-8")
    canonical = facts / "notes.md"
    d = conflicts.diff(canonical, sibling)
    assert "canonical" in d.lower() and "missing" in d.lower()


# ---------------------------------------------------------------------------
# resolve_conflict
# ---------------------------------------------------------------------------


def test_resolve_keep_canonical_archives_siblings(tmp_path):
    store = _make_store(tmp_path)
    facts = store / "memories" / "facts"
    canonical = facts / "notes.md"
    canonical.write_text("# canonical\n", encoding="utf-8")
    sibling1 = facts / "notes (conflict).md"
    sibling1.write_text("# sibling A\n", encoding="utf-8")
    sibling2 = facts / "notes.md.bak"
    sibling2.write_text("# sibling B\n", encoding="utf-8")

    result = conflicts.resolve_conflict(store, canonical, keep="canonical")
    assert result["status"] == "resolved"
    assert canonical.exists()
    assert canonical.read_text(encoding="utf-8") == "# canonical\n"
    assert not sibling1.exists()
    assert not sibling2.exists()
    assert len(result["archived"]) == 2

    # The conflicts/<ts>/ directory has the archived siblings + manifest
    archived_paths = [store / p for p in result["archived"]]
    for p in archived_paths:
        assert p.exists(), p
    manifest = archived_paths[0].parent / "manifest.json"
    assert manifest.exists()
    decoded = json.loads(manifest.read_text(encoding="utf-8"))
    assert decoded["keep"] == "canonical"


def test_resolve_keep_sibling_replaces_canonical(tmp_path):
    store = _make_store(tmp_path)
    facts = store / "memories" / "facts"
    canonical = facts / "notes.md"
    canonical.write_text("# old\n", encoding="utf-8")
    # Two siblings — sorted by filename: "notes (conflict).md" < "notes.md.bak"
    sibling1 = facts / "notes (conflict).md"
    sibling1.write_text("# winner\n", encoding="utf-8")
    sibling2 = facts / "notes.md.bak"
    sibling2.write_text("# loser\n", encoding="utf-8")

    result = conflicts.resolve_conflict(store, canonical, keep="sibling:1")
    assert result["status"] == "resolved"
    assert result["winner_source"] == "dropbox-or-gdrive"
    assert canonical.read_text(encoding="utf-8") == "# winner\n"
    assert not sibling1.exists()
    assert not sibling2.exists()
    assert "next_step" in result


def test_resolve_sibling_index_out_of_range(tmp_path):
    store = _make_store(tmp_path)
    facts = store / "memories" / "facts"
    canonical = facts / "notes.md"
    canonical.write_text("c", encoding="utf-8")
    sibling = facts / "notes (conflict).md"
    sibling.write_text("s", encoding="utf-8")
    with pytest.raises(ValueError, match="out of range"):
        conflicts.resolve_conflict(store, canonical, keep="sibling:99")


def test_resolve_dry_run_no_filesystem_change(tmp_path):
    store = _make_store(tmp_path)
    facts = store / "memories" / "facts"
    canonical = facts / "notes.md"
    canonical.write_text("c", encoding="utf-8")
    sibling = facts / "notes (conflict).md"
    sibling.write_text("s", encoding="utf-8")

    result = conflicts.resolve_conflict(store, canonical, keep="canonical", dry_run=True)
    assert result["status"] == "resolved"
    # Files MUST still exist
    assert canonical.exists()
    assert sibling.exists()
    # No conflicts/<ts>/manifest.json should exist
    conflicts_dirs = list((store / "conflicts").iterdir())
    assert conflicts_dirs == []


def test_resolve_no_conflicts(tmp_path):
    store = _make_store(tmp_path)
    facts = store / "memories" / "facts"
    canonical = facts / "notes.md"
    canonical.write_text("solo", encoding="utf-8")
    result = conflicts.resolve_conflict(store, canonical)
    assert result["status"] == "no-conflicts"


# ---------------------------------------------------------------------------
# invariant integration
# ---------------------------------------------------------------------------


def test_doctor_invariant_passes_on_clean_store(tmp_path):
    from cyberos.core.invariants import check_layout_no_sync_conflict_siblings
    store = _make_store(tmp_path)
    passed, details = check_layout_no_sync_conflict_siblings(store)
    assert passed, details
    assert "no sync-FS conflict siblings" in details


def test_doctor_invariant_fails_on_conflict_store(tmp_path):
    from cyberos.core.invariants import check_layout_no_sync_conflict_siblings
    store = _make_store(tmp_path)
    facts = store / "memories" / "facts"
    (facts / "notes.md").write_text("canonical", encoding="utf-8")
    (facts / "notes (conflict).md").write_text("sibling", encoding="utf-8")

    passed, details = check_layout_no_sync_conflict_siblings(store)
    assert not passed
    assert "conflict sibling" in details
    assert "resolve-conflict" in details
