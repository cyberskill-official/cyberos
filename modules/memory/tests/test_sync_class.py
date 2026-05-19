"""Tests for FR-MEMORY-106 sync_class enforcement helpers."""

from __future__ import annotations

import pytest

from cyberos.core.sync_class import (
    SYNC_CLASS_DEFAULT,
    SYNC_CLASS_ENUM,
    SyncClassError,
    assert_enum_value,
    classify,
    filter_shareable,
)


def test_default_is_private() -> None:
    assert SYNC_CLASS_DEFAULT == "private"


def test_enum_closed_set() -> None:
    assert SYNC_CLASS_ENUM == frozenset({"private", "shareable", "team"})


def test_classify_none_returns_default() -> None:
    assert classify(None) == SYNC_CLASS_DEFAULT


def test_classify_empty_returns_default() -> None:
    assert classify({}) == SYNC_CLASS_DEFAULT


def test_classify_explicit_values() -> None:
    assert classify({"sync_class": "private"}) == "private"
    assert classify({"sync_class": "shareable"}) == "shareable"
    assert classify({"sync_class": "team"}) == "team"


def test_classify_rejects_unknown_string() -> None:
    with pytest.raises(SyncClassError):
        classify({"sync_class": "public"})


def test_classify_rejects_non_string() -> None:
    with pytest.raises(SyncClassError):
        classify({"sync_class": 42})


def test_filter_shareable_drops_private() -> None:
    rows = [
        {"path": "p", "frontmatter": {"sync_class": "private"}},
        {"path": "s", "frontmatter": {"sync_class": "shareable"}},
    ]
    assert [r["path"] for r in filter_shareable(rows)] == ["s"]


def test_filter_shareable_includes_team() -> None:
    rows = [
        {"path": "t", "frontmatter": {"sync_class": "team"}},
    ]
    assert [r["path"] for r in filter_shareable(rows)] == ["t"]


def test_filter_shareable_drops_no_frontmatter() -> None:
    rows = [
        {"path": "x"},
        {"path": "y", "frontmatter": "not-a-mapping"},
        {"path": "z", "frontmatter": {"sync_class": "shareable"}},
    ]
    assert [r["path"] for r in filter_shareable(rows)] == ["z"]


def test_assert_enum_value_includes_path_in_error() -> None:
    with pytest.raises(SyncClassError) as exc:
        assert_enum_value("memories/foo.md", {"sync_class": "leaky"})
    assert "memories/foo.md" in str(exc.value)


def test_self_test_passes() -> None:
    from cyberos.core.sync_class import _test_self
    _test_self()
