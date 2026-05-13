"""
Tests for the v2 canonical ops (PROPOSAL.md P1 + P4): ``put``, ``move``,
and ``delete(mode="purge")``.

Pins the audit-row op-name contract (canonical: `put`, `move`, `delete`)
and the GDPR purge gate (refusal modes + magic-phrase success).
"""

from __future__ import annotations

from pathlib import Path

import pytest

from cyberos.core import ops
from cyberos.core.walker import MmapWalker
from cyberos.core.writer import Writer


@pytest.fixture()
def store(tmp_path: Path) -> Path:
    s = tmp_path / ".cyberos-memory"
    (s / "audit").mkdir(parents=True)
    return s


def _body(body_text: bytes = b"# body\n") -> bytes:
    return (
        b'---\n{"actor":"t","extra":{},"id":"X","kind":"fact","tags":[],"ts_ns":1}\n---\n'
        + body_text
    )


# --- P1: put / move canonical op names ------------------------------------


def test_put_on_new_file_emits_op_put(store: Path) -> None:
    with Writer(store) as w:
        ops.put(w, "memories/facts/X.md", _body(), actor="t")
    with MmapWalker(store / "audit" / "current.binlog") as walker:
        records = list(walker.iter_records())
    assert len(records) == 1
    assert records[0][1].op == "put"


def test_put_idempotent_on_existing_file(store: Path) -> None:
    """put doesn't care whether the file existed — same op name either way."""
    with Writer(store) as w:
        ops.put(w, "memories/facts/X.md", _body(b"first"), actor="t")
        ops.put(w, "memories/facts/X.md", _body(b"second"), actor="t")
    with MmapWalker(store / "audit" / "current.binlog") as walker:
        records = list(walker.iter_records())
    assert [r.op for _o, r in records] == ["put", "put"]
    assert (store / "memories/facts/X.md").read_bytes().endswith(b"second")


def test_move_emits_op_move(store: Path) -> None:
    with Writer(store) as w:
        ops.put(w, "memories/facts/X.md", _body(), actor="t")
        ops.move(w, "memories/facts/X.md", "memories/facts/Y.md", actor="t")
    with MmapWalker(store / "audit" / "current.binlog") as walker:
        records = list(walker.iter_records())
    assert records[1][1].op == "move"
    assert records[1][1].extra["to"] == "memories/facts/Y.md"
    assert not (store / "memories/facts/X.md").exists()
    assert (store / "memories/facts/Y.md").exists()


def test_v1_aliases_still_emit_v1_op_names(store: Path) -> None:
    """create/str_replace/insert/rename keep emitting their v1 op names."""
    with Writer(store) as w:
        ops.create(w, "memories/facts/A.md", _body(), actor="t")
        ops.rename(w, "memories/facts/A.md", "memories/facts/B.md", actor="t")
    with MmapWalker(store / "audit" / "current.binlog") as walker:
        records = list(walker.iter_records())
    assert [r.op for _o, r in records] == ["create", "rename"]


# --- P4: GDPR purge --------------------------------------------------------


_MAGIC = "APPROVE protocol change P4 §3.6"


def test_tombstone_keeps_body_on_disk(store: Path) -> None:
    with Writer(store) as w:
        ops.put(w, "memories/facts/X.md", _body(b"secret"), actor="t")
        ops.delete(w, "memories/facts/X.md", actor="t")
    assert (store / "memories/facts/X.md").is_file()
    assert b"secret" in (store / "memories/facts/X.md").read_bytes()


def test_purge_without_phrase_refused(store: Path) -> None:
    with Writer(store) as w:
        ops.put(w, "memories/facts/X.md", _body(b"secret"), actor="t")
        with pytest.raises(ops.PurgeRefused):
            ops.delete(w, "memories/facts/X.md", actor="t", mode="purge", reason="gdpr")


def test_purge_without_reason_refused(store: Path) -> None:
    with Writer(store) as w:
        ops.put(w, "memories/facts/X.md", _body(b"secret"), actor="t")
        with pytest.raises(ops.PurgeRefused):
            ops.delete(
                w, "memories/facts/X.md", actor="t",
                mode="purge", reason="", approval_phrase=_MAGIC,
            )


def test_purge_with_wrong_phrase_refused(store: Path) -> None:
    with Writer(store) as w:
        ops.put(w, "memories/facts/X.md", _body(b"secret"), actor="t")
        with pytest.raises(ops.PurgeRefused):
            ops.delete(
                w, "memories/facts/X.md", actor="t",
                mode="purge", reason="gdpr", approval_phrase="wrong",
            )


def test_purge_with_correct_phrase_redacts_body(store: Path) -> None:
    target = "memories/facts/X.md"
    with Writer(store) as w:
        ops.put(w, target, _body(b"sensitive personal data"), actor="t")
        ops.delete(
            w, target, actor="t",
            mode="purge", reason="data-subject request 2026-05-13",
            approval_phrase=_MAGIC,
        )
    # File still exists but body is the redaction marker.
    body = (store / target).read_bytes()
    assert b"<<<CYBEROS:PURGED" in body
    assert b"sensitive personal data" not in body


def test_purge_audit_row_carries_redaction_metadata(store: Path) -> None:
    target = "memories/facts/X.md"
    with Writer(store) as w:
        ops.put(w, target, _body(b"x"), actor="t")
        ops.delete(
            w, target, actor="t",
            mode="purge", reason="legal", approval_phrase=_MAGIC,
        )
    with MmapWalker(store / "audit" / "current.binlog") as walker:
        records = list(walker.iter_records())
    purge_row = records[-1][1]
    assert purge_row.op == "delete"
    assert purge_row.extra["mode"] == "purge"
    assert purge_row.extra["reason"] == "legal"
    assert "redacted_sha256" in purge_row.extra
    # The audit row's content_sha256 records the ORIGINAL bytes' hash.
    assert purge_row.content_sha256 == purge_row.extra["redacted_sha256"]


def test_purge_via_env_var_phrase(store: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Setting CYBEROS_PURGE_APPROVAL provides the phrase implicitly."""
    monkeypatch.setenv("CYBEROS_PURGE_APPROVAL", _MAGIC)
    target = "memories/facts/X.md"
    with Writer(store) as w:
        ops.put(w, target, _body(), actor="t")
        ops.delete(w, target, actor="t", mode="purge", reason="env-var test")
    assert b"<<<CYBEROS:PURGED" in (store / target).read_bytes()


# --- P3 surface: sidecar parser --------------------------------------------


def test_parse_sidecar_validates_body_hash(tmp_path: Path) -> None:
    import hashlib
    from cyberos.core.frontmatter import Frontmatter, parse_sidecar
    import msgspec

    body = b"# pure body content\n"
    body_hash = hashlib.sha256(body).hexdigest()
    fm = Frontmatter(
        id="F-001", kind="fact", ts_ns=1, actor="t",
        extra={"body_hash": body_hash},
    )
    sidecar_bytes = msgspec.json.encode(fm, order="sorted")

    fm_out, body_out = parse_sidecar(sidecar_bytes, body)
    assert fm_out.id == "F-001"
    assert body_out == body


def test_parse_sidecar_rejects_hash_mismatch(tmp_path: Path) -> None:
    from cyberos.core.frontmatter import Frontmatter, parse_sidecar
    import msgspec

    body = b"# body\n"
    fm = Frontmatter(
        id="X", kind="fact", ts_ns=1, actor="t",
        extra={"body_hash": "0" * 64},  # deliberately wrong
    )
    sidecar_bytes = msgspec.json.encode(fm, order="sorted")
    with pytest.raises(ValueError, match="body_hash mismatch"):
        parse_sidecar(sidecar_bytes, body)
