"""TASK-MEMORY-106 — sync_class enforcement.

The sync_class frontmatter field controls whether a memory crosses the
personal-memory ↔ Lumi sync boundary:

  * ``private`` — never leaves this device. The memory-sync daemon MUST
    filter it out of every outbound push. Reads stay local. This is the
    default for new memories.

  * ``shareable`` — replicates to Cloud memory. Anyone with sync access to
    that memory can read it.

  * ``team`` (P3+) — visible to a named team scope; routed through Lumi.

This module provides:

  * :func:`classify` — pure function: given a memory's frontmatter dict,
    return the canonical sync_class value (defaulting to ``private``).
  * :func:`filter_shareable` — used by the future memory-sync daemon to
    drop private rows from a push payload.
  * :func:`assert_enum_value` — doctor invariant: every shipped memory
    must carry a sync_class value from the closed enum.
"""

from __future__ import annotations

from typing import Iterable, Mapping, MutableMapping

SYNC_CLASS_DEFAULT: str = "private"

# Closed enum — additions require an ADR + matching update to the
# personal-memory sync daemon (TASK-MEMORY-103) and the Cloud-memory admit policy.
SYNC_CLASS_ENUM: frozenset[str] = frozenset({"private", "shareable", "team"})


class SyncClassError(ValueError):
    """A memory carried an unknown or malformed sync_class value."""


def classify(frontmatter: Mapping[str, object] | None) -> str:
    """Resolve a memory's effective sync_class.

    * Returns ``"private"`` when the frontmatter is ``None`` or missing the
      ``sync_class`` key (data-minimisation default).
    * Returns the explicit value otherwise.
    * Raises :class:`SyncClassError` on unknown values — the doctor invariant
      below catches these at the per-memory level, but callers in the sync
      path should also reject them at write time.
    """
    if frontmatter is None:
        return SYNC_CLASS_DEFAULT
    v = frontmatter.get("sync_class", SYNC_CLASS_DEFAULT)
    if v is None:
        return SYNC_CLASS_DEFAULT
    if not isinstance(v, str):
        raise SyncClassError(f"sync_class must be a string, got {type(v).__name__}")
    if v not in SYNC_CLASS_ENUM:
        raise SyncClassError(
            f"sync_class {v!r} not in closed enum {sorted(SYNC_CLASS_ENUM)}"
        )
    return v


def filter_shareable(
    rows: Iterable[Mapping[str, object]],
    *,
    frontmatter_key: str = "frontmatter",
) -> list[Mapping[str, object]]:
    """Drop every row whose sync_class is not ``shareable`` (or ``team`` for P3+).

    Used by the memory-sync daemon (TASK-MEMORY-103) to enforce DEC-070's
    "Layer 1 is the source of truth; what crosses the device boundary is
    operator-chosen" invariant.
    """
    out: list[Mapping[str, object]] = []
    for row in rows:
        fm = row.get(frontmatter_key)
        if not isinstance(fm, Mapping):
            # No frontmatter ⇒ defaults to private ⇒ don't push.
            continue
        try:
            cls = classify(fm)
        except SyncClassError:
            # Unknown sync_class ⇒ refuse to push; let doctor surface it.
            continue
        if cls in {"shareable", "team"}:
            out.append(row)
    return out


def assert_enum_value(
    path: str,
    frontmatter: Mapping[str, object] | None,
) -> None:
    """Doctor-invariant hook.

    Raises :class:`SyncClassError` if the memory's sync_class is malformed.
    Pure validator — no I/O. Called by ``cyberos doctor`` once per memory.
    """
    try:
        classify(frontmatter)
    except SyncClassError as e:
        raise SyncClassError(f"{path}: {e}") from e


# ---------------------------------------------------------------------------
# Tests — keep the closed-enum invariant honest.
# ---------------------------------------------------------------------------

def _test_self() -> None:
    """Module self-test invoked from pytest via test_sync_class.py."""
    assert classify(None) == SYNC_CLASS_DEFAULT
    assert classify({}) == SYNC_CLASS_DEFAULT
    assert classify({"sync_class": "private"}) == "private"
    assert classify({"sync_class": "shareable"}) == "shareable"
    assert classify({"sync_class": "team"}) == "team"
    try:
        classify({"sync_class": "public"})
    except SyncClassError:
        pass
    else:
        raise AssertionError("expected SyncClassError for 'public'")

    rows = [
        {"path": "a", "frontmatter": {"sync_class": "private"}},
        {"path": "b", "frontmatter": {"sync_class": "shareable"}},
        {"path": "c", "frontmatter": {"sync_class": "team"}},
        {"path": "d", "frontmatter": {}},                       # default → private → dropped
        {"path": "e"},                                          # no frontmatter → dropped
    ]
    out = filter_shareable(rows)
    assert [r["path"] for r in out] == ["b", "c"], out
