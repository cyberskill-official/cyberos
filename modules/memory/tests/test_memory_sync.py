"""Tests for FR-MEMORY-103 multi-device sync daemon (pure-function pieces)."""

from __future__ import annotations

import json
import os
from pathlib import Path
from unittest.mock import patch

import pytest

from cyberos.core.memory_sync import (
    DEAD_LETTER_REL,
    LAST_STATUS_REL,
    PULL_CURSOR_REL,
    MemorySync,
    RetryPolicy,
    SyncTransportError,
    build_push_batch,
    should_admit_pulled_row,
    _read_pull_cursor,
    _write_pull_cursor,
    _append_dead_letter,
)


# ---------------------------------------------------------------------------
# RetryPolicy
# ---------------------------------------------------------------------------

def test_retry_backoff_grows() -> None:
    p = RetryPolicy(base_secs=1.0, factor=2.0, cap_secs=10.0)
    assert p.backoff_for_attempt(0) == 0.0
    assert p.backoff_for_attempt(1) == 1.0
    assert p.backoff_for_attempt(2) == 2.0
    assert p.backoff_for_attempt(3) == 4.0
    assert p.backoff_for_attempt(5) == 10.0  # capped


# ---------------------------------------------------------------------------
# build_push_batch
# ---------------------------------------------------------------------------

def test_push_batch_filters_private() -> None:
    rows = [
        {"seq": 1, "path": "a", "frontmatter": {"sync_class": "private"}},
        {"seq": 2, "path": "b", "frontmatter": {"sync_class": "shareable"}},
    ]
    out = build_push_batch(rows)
    assert [r["path"] for r in out] == ["b"]


def test_push_batch_caps_at_max_size() -> None:
    rows = [
        {"seq": i, "path": f"r{i}", "frontmatter": {"sync_class": "shareable"}}
        for i in range(100)
    ]
    out = build_push_batch(rows, max_size=10)
    assert len(out) == 10


def test_push_batch_envelope_has_sync_class() -> None:
    rows = [{"seq": 1, "path": "x", "frontmatter": {"sync_class": "team"}}]
    out = build_push_batch(rows)
    assert out[0]["sync_class"] == "team"


# ---------------------------------------------------------------------------
# should_admit_pulled_row
# ---------------------------------------------------------------------------

def test_admit_shareable_with_anchor() -> None:
    row = {"sync_class": "shareable", "chain_anchor": "abc"}
    assert should_admit_pulled_row(row) is True


def test_admit_team_with_anchor() -> None:
    row = {"sync_class": "team", "chain_anchor": "abc"}
    assert should_admit_pulled_row(row) is True


def test_reject_private_row() -> None:
    row = {"sync_class": "private", "chain_anchor": "abc"}
    assert should_admit_pulled_row(row) is False


def test_reject_row_missing_anchor() -> None:
    row = {"sync_class": "shareable"}
    assert should_admit_pulled_row(row) is False


def test_reject_unknown_sync_class() -> None:
    row = {"sync_class": "leaky", "chain_anchor": "abc"}
    assert should_admit_pulled_row(row) is False


def test_admit_anchor_check_can_be_disabled() -> None:
    row = {"sync_class": "shareable"}
    assert should_admit_pulled_row(row, require_chain_anchor=False) is True


# ---------------------------------------------------------------------------
# Cursor + dead-letter persistence
# ---------------------------------------------------------------------------

def test_read_cursor_returns_zero_when_absent(tmp_path: Path) -> None:
    assert _read_pull_cursor(tmp_path) == 0


def test_cursor_round_trip(tmp_path: Path) -> None:
    _write_pull_cursor(tmp_path, 42)
    assert _read_pull_cursor(tmp_path) == 42


def test_cursor_recovers_from_corrupt_file(tmp_path: Path) -> None:
    p = tmp_path / PULL_CURSOR_REL
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text("not valid json")
    assert _read_pull_cursor(tmp_path) == 0


def test_dead_letter_appends_lines(tmp_path: Path) -> None:
    _append_dead_letter(tmp_path, {"path": "x", "seq": 1}, "test reason")
    _append_dead_letter(tmp_path, {"path": "y", "seq": 2}, "other reason")
    p = tmp_path / DEAD_LETTER_REL
    lines = p.read_text().strip().split("\n")
    assert len(lines) == 2
    parsed = [json.loads(line) for line in lines]
    assert parsed[0]["reason"] == "test reason"
    assert parsed[1]["row_path"] == "y"


# ---------------------------------------------------------------------------
# Orchestrator — push + pull behaviour with HTTP stubbed
# ---------------------------------------------------------------------------

def test_push_once_writes_status_and_counts(tmp_path: Path) -> None:
    rows = [
        {"seq": 1, "path": "a", "frontmatter": {"sync_class": "shareable"}, "chain_anchor": "h1"},
        {"seq": 2, "path": "b", "frontmatter": {"sync_class": "private"}, "chain_anchor": "h2"},
    ]
    sync = MemorySync(store=tmp_path, lumi_url="https://lumi.test", lumi_token="t")
    with patch("cyberos.core.memory_sync._http_post_json", return_value={"ok": True}):
        counters = sync.push_once(rows)
    assert counters["pushed"] == 1
    assert counters["dead_lettered"] == 0
    status = json.loads((tmp_path / LAST_STATUS_REL).read_text())
    assert status["cycle"] == "push"
    assert status["pushed"] == 1


def test_push_retries_and_dead_letters_on_persistent_failure(tmp_path: Path) -> None:
    rows = [{"seq": 1, "path": "a", "frontmatter": {"sync_class": "shareable"}, "chain_anchor": "h1"}]
    sync = MemorySync(
        store=tmp_path,
        lumi_url="https://lumi.test",
        lumi_token="t",
        retry=RetryPolicy(base_secs=0.0, max_attempts=2),  # no real sleep
    )
    with patch("cyberos.core.memory_sync._http_post_json",
               side_effect=SyncTransportError("nope")), \
         patch("cyberos.core.memory_sync.time.sleep", lambda _x: None):
        counters = sync.push_once(rows)
    assert counters["pushed"] == 0
    assert counters["dead_lettered"] == 1
    dead = (tmp_path / DEAD_LETTER_REL).read_text().strip().split("\n")
    assert json.loads(dead[0])["reason"] == "push_exhausted_retries"


def test_pull_once_admits_and_writes_cursor(tmp_path: Path) -> None:
    sync = MemorySync(store=tmp_path, lumi_url="https://lumi.test", lumi_token="t")
    resp = {
        "rows": [
            {"seq": 11, "sync_class": "shareable", "chain_anchor": "h11"},
            {"seq": 12, "sync_class": "private",   "chain_anchor": "h12"},  # rejected
            {"seq": 13, "sync_class": "team",      "chain_anchor": "h13"},
        ]
    }
    with patch("cyberos.core.memory_sync._http_get_json", return_value=resp):
        counters = sync.pull_once()
    assert counters["fetched"] == 3
    assert counters["admitted"] == 2
    assert counters["rejected"] == 1
    assert counters["new_cursor"] == 13
    assert _read_pull_cursor(tmp_path) == 13


def test_pull_once_records_network_error(tmp_path: Path) -> None:
    sync = MemorySync(store=tmp_path, lumi_url="https://lumi.test", lumi_token="t")
    with patch("cyberos.core.memory_sync._http_get_json",
               side_effect=SyncTransportError("unreachable")):
        counters = sync.pull_once()
    assert counters["fetched"] == 0
    assert counters["admitted"] == 0
    assert "error" in counters
