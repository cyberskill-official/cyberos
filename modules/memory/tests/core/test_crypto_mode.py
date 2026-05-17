"""Tests for cyberos.core.crypto_mode (PROPOSAL.md P2 Stage 3)."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cyberos.core import crypto_mode as cm


def _make_store(tmp_path: Path) -> Path:
    store = tmp_path / ".cyberos-memory"
    store.mkdir()
    (store / "audit").mkdir()
    (store / "manifest.json").write_text("{}", encoding="utf-8")
    return store


def _make_signed_store(tmp_path: Path) -> Path:
    """A store with at least one persisted STH (passes safety check #1)."""
    store = _make_store(tmp_path)
    sth_dir = store / "audit" / "sth"
    sth_dir.mkdir(parents=True, exist_ok=True)
    (sth_dir / "0001.json").write_text("{}", encoding="utf-8")
    return store


# ---------------------------------------------------------------------------
# current_mode default
# ---------------------------------------------------------------------------


def test_current_mode_default_chained(tmp_path):
    store = _make_store(tmp_path)
    assert cm.current_mode(store) == "chained"


def test_current_mode_when_no_manifest(tmp_path):
    store = tmp_path / ".cyberos-memory"
    store.mkdir()
    assert cm.current_mode(store) == "chained"


def test_current_mode_returns_persisted_value(tmp_path):
    store = _make_store(tmp_path)
    manifest_path = store / "manifest.json"
    manifest_path.write_text(
        json.dumps({"crypto_mode": "sth_only"}),
        encoding="utf-8",
    )
    assert cm.current_mode(store) == "sth_only"


def test_is_sth_only_predicate(tmp_path):
    store = _make_store(tmp_path)
    assert not cm.is_sth_only(store)
    (store / "manifest.json").write_text(
        json.dumps({"crypto_mode": "sth_only"}),
        encoding="utf-8",
    )
    assert cm.is_sth_only(store)


# ---------------------------------------------------------------------------
# upgrade — gating
# ---------------------------------------------------------------------------


def test_upgrade_refuses_wrong_phrase(tmp_path):
    store = _make_signed_store(tmp_path)
    with pytest.raises(cm.CryptoModeError, match="approval_phrase"):
        cm.upgrade_to_sth_only(
            store, approval_phrase="not the right phrase",
            skip_safety_checks=True,
        )


def test_upgrade_refuses_when_no_sth(tmp_path):
    store = _make_store(tmp_path)
    with pytest.raises(cm.CryptoModeError, match="STH"):
        cm.upgrade_to_sth_only(store, approval_phrase=cm.APPROVAL_PHRASE)


def test_upgrade_refuses_when_no_manifest(tmp_path):
    store = tmp_path / "empty"
    store.mkdir()
    (store / "audit" / "sth").mkdir(parents=True)
    (store / "audit" / "sth" / "1.json").write_text("{}", encoding="utf-8")
    with pytest.raises(cm.CryptoModeError, match="manifest"):
        cm.upgrade_to_sth_only(
            store,
            approval_phrase=cm.APPROVAL_PHRASE,
            skip_safety_checks=True,
        )


# ---------------------------------------------------------------------------
# upgrade — happy path
# ---------------------------------------------------------------------------


def test_upgrade_flips_manifest(tmp_path):
    store = _make_signed_store(tmp_path)
    summary = cm.upgrade_to_sth_only(
        store, approval_phrase=cm.APPROVAL_PHRASE,
        skip_safety_checks=True,
    )
    assert summary["status"] == "upgraded"
    assert summary["previous_mode"] == "chained"
    assert summary["current_mode"] == "sth_only"

    manifest = json.loads((store / "manifest.json").read_text(encoding="utf-8"))
    assert manifest["crypto_mode"] == "sth_only"
    assert manifest["crypto_mode_history"][-1]["to"] == "sth_only"
    assert manifest["crypto_mode_history"][-1]["from"] == "chained"


def test_upgrade_idempotent(tmp_path):
    store = _make_signed_store(tmp_path)
    cm.upgrade_to_sth_only(
        store, approval_phrase=cm.APPROVAL_PHRASE,
        skip_safety_checks=True,
    )
    # Second time → already-upgraded
    summary = cm.upgrade_to_sth_only(
        store, approval_phrase=cm.APPROVAL_PHRASE,
        skip_safety_checks=True,
    )
    assert summary["status"] == "already-upgraded"


# ---------------------------------------------------------------------------
# downgrade
# ---------------------------------------------------------------------------


def test_downgrade_refuses_wrong_phrase(tmp_path):
    store = _make_signed_store(tmp_path)
    cm.upgrade_to_sth_only(
        store, approval_phrase=cm.APPROVAL_PHRASE,
        skip_safety_checks=True,
    )
    with pytest.raises(cm.CryptoModeError):
        cm.downgrade_to_chained(store, approval_phrase="nope")


def test_downgrade_happy_path(tmp_path):
    store = _make_signed_store(tmp_path)
    cm.upgrade_to_sth_only(
        store, approval_phrase=cm.APPROVAL_PHRASE,
        skip_safety_checks=True,
    )
    summary = cm.downgrade_to_chained(store, approval_phrase=cm.APPROVAL_PHRASE)
    assert summary["status"] == "downgraded"
    assert summary["previous_mode"] == "sth_only"
    assert summary["current_mode"] == "chained"

    manifest = json.loads((store / "manifest.json").read_text(encoding="utf-8"))
    assert manifest["crypto_mode"] == "chained"
    # Both transitions are in history
    history = manifest["crypto_mode_history"]
    assert len(history) >= 2
    assert history[-1]["to"] == "chained"


def test_downgrade_when_already_chained(tmp_path):
    store = _make_store(tmp_path)
    summary = cm.downgrade_to_chained(store, approval_phrase=cm.APPROVAL_PHRASE)
    assert summary["status"] == "already-chained"


# ---------------------------------------------------------------------------
# manifest atomicity
# ---------------------------------------------------------------------------


def test_upgrade_writes_canonical_manifest_json(tmp_path):
    store = _make_signed_store(tmp_path)
    cm.upgrade_to_sth_only(
        store, approval_phrase=cm.APPROVAL_PHRASE,
        skip_safety_checks=True,
    )
    # Manifest must remain valid JSON
    json.loads((store / "manifest.json").read_text(encoding="utf-8"))
    # No leftover .tmp
    assert not (store / "manifest.tmp").exists()


# ---------------------------------------------------------------------------
# invariant integration — sth_only mode demotes link to advisory
# ---------------------------------------------------------------------------


def test_link_invariant_passes_advisory_in_sth_only(tmp_path):
    """check_ledger_link returns PASS even in sth_only mode with no records."""
    from cyberos.core.invariants import check_ledger_link
    store = _make_signed_store(tmp_path)
    cm.upgrade_to_sth_only(
        store, approval_phrase=cm.APPROVAL_PHRASE,
        skip_safety_checks=True,
    )
    passed, details = check_ledger_link(store)
    assert passed
    # Either "no binlog segments" (empty store) or note about sth_only mode.
    # Empty store path returns "no binlog segments" before mode check kicks in.
    assert "no binlog segments" in details or "sth_only" in details


def test_link_invariant_records_sth_only_advisory_with_data(tmp_path):
    """With actual binlog data, the details string mentions sth_only mode."""
    from cyberos.core.invariants import check_ledger_link
    from cyberos.core.ops import put
    from cyberos.core.writer import Writer

    store = _make_store(tmp_path)
    # Write at least one row so check_ledger_link goes past the empty-segs guard.
    (store / "memories" / "facts").mkdir(parents=True, exist_ok=True)
    with Writer(store) as w:
        put(w, "memories/facts/a.md", b"a", actor="s", kind="fact")
    # Add a fake STH so upgrade safety check #1 passes
    sth_dir = store / "audit" / "sth"
    sth_dir.mkdir(parents=True, exist_ok=True)
    (sth_dir / "1.json").write_text("{}", encoding="utf-8")

    # Switch to sth_only
    cm.upgrade_to_sth_only(
        store, approval_phrase=cm.APPROVAL_PHRASE,
        skip_safety_checks=True,
    )
    passed, details = check_ledger_link(store)
    assert passed
    assert "sth_only" in details
