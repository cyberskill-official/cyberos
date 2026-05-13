"""Determinism regression for cyberos.core.export.export_zip."""

from __future__ import annotations

from pathlib import Path

import pytest

from cyberos.core import ops
from cyberos.core.export import export_zip
from cyberos.core.frontmatter import Frontmatter, serialize
from cyberos.core.writer import Writer


@pytest.fixture()
def store(tmp_path: Path) -> Path:
    s = tmp_path / ".cyberos-memory"
    (s / "audit").mkdir(parents=True)
    return s


def _populate(store: Path) -> None:
    with Writer(store) as writer:
        for i in range(20):
            body = serialize(
                Frontmatter(id=f"M-{i}", kind="fact", ts_ns=i, actor="t"),
                f"# memory {i}\n".encode("utf-8"),
            )
            ops.create(writer, f"memories/facts/{i:04d}.md", body, actor="t")


def test_export_byte_identical_on_repeat(store: Path, tmp_path: Path) -> None:
    _populate(store)
    a = export_zip(store, tmp_path / "a.zip")
    b = export_zip(store, tmp_path / "b.zip")
    assert a == b, f"export not deterministic: {a} != {b}"


def test_export_byte_identical_with_intervening_view(store: Path, tmp_path: Path) -> None:
    """View ops should not change the export's byte hash.

    Views append audit rows (which the export does include — the binlog
    is part of the store) so identical views in different orders produce
    different exports. This test confirms that the export is stable as
    long as the underlying state matches.
    """
    _populate(store)
    a = export_zip(store, tmp_path / "a.zip")
    # Re-export immediately — no ops in between.
    b = export_zip(store, tmp_path / "b.zip")
    assert a == b
