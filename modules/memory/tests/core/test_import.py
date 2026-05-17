"""
P6 cross-BRAIN import tests.

Covers:
* basic import — files copied, audit-row sandwich, manifest watermark.
* idempotent re-import — second run is a no-op.
* filter — sync_class=shareable yields only matching files.
* conflict policy — skip/overwrite/branch each behave correctly.
* map-actor — rewrites actor field on import rows.
* delete propagation — tombstones in source produce tombstones locally.
* dry-run — reports plan without writes.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cyberos.core import ops
from cyberos.core.import_ import (
    _fingerprint, build_plans, format_report, get_last_imported_seq,
    parse_filters, run,
)
from cyberos.core.walker import MmapWalker
from cyberos.core.writer import Writer


def _make_store(root: Path, name: str) -> Path:
    s = root / name / ".cyberos-memory"
    (s / "audit").mkdir(parents=True)
    s.joinpath("manifest.json").write_text("{}", encoding="utf-8")
    return s


def _body(fm_id: str, *, kind: str = "fact", extra: dict | None = None) -> bytes:
    e = extra or {}
    fm = {
        "actor": "src",
        "extra": e,
        "id": fm_id,
        "kind": kind,
        "tags": [],
        "ts_ns": 1,
    }
    head = ("---\n" + json.dumps(fm, sort_keys=True) + "\n---\n").encode("utf-8")
    return head + f"# {fm_id}\n".encode("utf-8")


# ---------------------------------------------------------------------- fixtures


@pytest.fixture()
def source_store(tmp_path: Path) -> Path:
    store = _make_store(tmp_path, "source")
    with Writer(store) as w:
        ops.put(w, "memories/facts/F-001.md", _body("F-001"), actor="alice")
        ops.put(w, "memories/decisions/D-001.md", _body("D-001", kind="decision",
                                                       extra={"sync_class": "shareable"}),
                actor="alice")
        ops.put(w, "memories/decisions/D-002.md", _body("D-002", kind="decision",
                                                       extra={"sync_class": "private"}),
                actor="alice")
    return store


@pytest.fixture()
def target_store(tmp_path: Path) -> Path:
    return _make_store(tmp_path, "target")


# ---------------------------------------------------------------------- basic


def test_basic_import_copies_files(target_store: Path, source_store: Path) -> None:
    report = run(target_store, source_store)
    assert report.ok
    assert len(report.imported) == 3
    for rel in [
        "memories/facts/F-001.md",
        "memories/decisions/D-001.md",
        "memories/decisions/D-002.md",
    ]:
        assert (target_store / rel).is_file()


def test_import_brackets_session_audit_rows(target_store: Path, source_store: Path) -> None:
    run(target_store, source_store)
    with MmapWalker(target_store / "audit" / "current.binlog") as w:
        records = [r for _o, r in w.iter_records()]
    ops_list = [r.op for r in records]
    assert ops_list[0] == "session.start"
    assert ops_list[-1] == "session.end"
    assert records[0].actor.startswith("cyberos-import:")
    assert records[-1].extra["imported"] == 3


def test_idempotent_reimport_is_noop(target_store: Path, source_store: Path) -> None:
    run(target_store, source_store)
    second = run(target_store, source_store)
    assert second.ok
    assert len(second.imported) == 0


def test_manifest_records_high_watermark(target_store: Path, source_store: Path) -> None:
    run(target_store, source_store)
    fp = _fingerprint(source_store)
    assert get_last_imported_seq(target_store, fp) > 0


# ---------------------------------------------------------------------- filters


def test_filter_sync_class_shareable_only(target_store: Path, source_store: Path) -> None:
    report = run(target_store, source_store, filters=["sync_class=shareable"])
    assert report.ok
    assert report.imported == ["memories/decisions/D-001.md"]
    # The private and unmarked files should NOT be present
    assert not (target_store / "memories/decisions/D-002.md").exists()
    assert not (target_store / "memories/facts/F-001.md").exists()


def test_filter_kind_decision(target_store: Path, source_store: Path) -> None:
    report = run(target_store, source_store, filters=["kind=decision"])
    assert len(report.imported) == 2
    assert "memories/decisions/D-001.md" in report.imported
    assert "memories/decisions/D-002.md" in report.imported


def test_parse_filters_rejects_bad_syntax() -> None:
    with pytest.raises(ValueError, match="unrecognised filter"):
        parse_filters(["this-is-not-a-filter"])


def test_parse_filters_rejects_unknown_key() -> None:
    with pytest.raises(ValueError, match="unknown filter key"):
        parse_filters(["spaghetti=true"])


# ---------------------------------------------------------------------- conflicts


def test_conflict_skip_leaves_local_intact(target_store: Path, source_store: Path) -> None:
    # Pre-populate the target with a different body at the same path.
    with Writer(target_store) as w:
        ops.put(w, "memories/facts/F-001.md", _body("LOCAL-VERSION"), actor="bob")
    local_before = (target_store / "memories/facts/F-001.md").read_bytes()

    report = run(target_store, source_store, on_conflict="skip")
    assert any("F-001.md" in s for s, _ in report.skipped)
    assert (target_store / "memories/facts/F-001.md").read_bytes() == local_before


def test_conflict_overwrite_replaces(target_store: Path, source_store: Path) -> None:
    with Writer(target_store) as w:
        ops.put(w, "memories/facts/F-001.md", _body("LOCAL-VERSION"), actor="bob")

    run(target_store, source_store, on_conflict="overwrite")
    body = (target_store / "memories/facts/F-001.md").read_bytes()
    assert b"F-001" in body
    assert b"LOCAL-VERSION" not in body


def test_conflict_branch_creates_alt_path(target_store: Path, source_store: Path) -> None:
    with Writer(target_store) as w:
        ops.put(w, "memories/facts/F-001.md", _body("LOCAL-VERSION"), actor="bob")

    report = run(target_store, source_store, on_conflict="branch")
    assert report.branched
    # Original local copy still there
    assert b"LOCAL-VERSION" in (target_store / "memories/facts/F-001.md").read_bytes()
    # Branched copy exists with .from-<fp>.md suffix
    branched_paths = [new for _, new in report.branched]
    assert any("from-" in p for p in branched_paths)


# ---------------------------------------------------------------------- map-actor


def test_map_actor_rewrites_audit_rows(target_store: Path, source_store: Path) -> None:
    run(target_store, source_store, map_actor={"alice": "alice@cyberskill.world"})
    with MmapWalker(target_store / "audit" / "current.binlog") as w:
        records = [r for _o, r in w.iter_records() if r.op == "put"]
    assert all(r.actor == "alice@cyberskill.world" for r in records)


# ---------------------------------------------------------------------- delete propagation


def test_delete_in_source_propagates(target_store: Path, source_store: Path) -> None:
    # Initial import
    run(target_store, source_store)
    # Source-side delete
    with Writer(source_store) as w:
        ops.delete(w, "memories/facts/F-001.md", actor="alice")
    # Re-import
    report = run(target_store, source_store)
    assert any(p.endswith("F-001.md") for p in report.imported)
    # Audit row on local chain marks the tombstone
    with MmapWalker(target_store / "audit" / "current.binlog") as w:
        last_delete = [
            r for _o, r in w.iter_records()
            if r.op == "delete" and r.path == "memories/facts/F-001.md"
        ]
    assert last_delete


# ---------------------------------------------------------------------- dry-run


def test_dry_run_writes_nothing(target_store: Path, source_store: Path) -> None:
    report = run(target_store, source_store, dry_run=True)
    assert report.ok
    assert len(report.imported) == 3
    # No files moved, no audit rows appended
    assert not (target_store / "memories/facts/F-001.md").exists()
    binlog = target_store / "audit" / "current.binlog"
    assert not binlog.exists() or binlog.stat().st_size == 0


# ---------------------------------------------------------------------- zip source


def test_import_from_zip(target_store: Path, source_store: Path, tmp_path: Path) -> None:
    from cyberos.core.export import export_zip
    zip_path = tmp_path / "source.zip"
    export_zip(source_store, zip_path)

    report = run(target_store, zip_path)
    assert report.ok
    assert len(report.imported) == 3
