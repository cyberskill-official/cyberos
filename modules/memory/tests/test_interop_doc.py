"""
INTEROP.md + CHANGELOG conformance (TASK-MEMORY-303 §1.3 / §1.8).

AGENTS.md §14.1 binds every non-ledger consumer to a ≤ 6,000-character
``INTEROP.md``. Until TASK-MEMORY-303 no such file existed anywhere —
every cross-agent consumer was bound to a document nobody could read.
These tests pin its existence, the normative length bound, and the five
mandated content anchors, plus the §1.8 CHANGELOG record.
"""

from __future__ import annotations

from pathlib import Path

_MEMORY = Path(__file__).resolve().parent.parent          # modules/memory/
_INTEROP = _MEMORY / "INTEROP.md"
_CHANGELOG = _MEMORY / "CHANGELOG.md"
_BUILD_SH = _MEMORY.parent.parent / "tools" / "install" / "build.sh"

# §14.1's normative cap. The bound is pinned here so a future edit that
# exceeds it fails CI rather than silently violating the protocol the
# document exists to describe.
_MAX_CHARS = 6000

# The five mandated content anchors (spec §1.3). Each tuple is
# (label, list of substrings — ALL must appear).
_ANCHORS: tuple[tuple[str, list[str]], ...] = (
    ("read paths", ["## 2. Read paths", "memories/<kind>/"]),
    (
        "no-write rule for audit/, HEAD, .lock",
        ["MUST NOT write `audit/`, `HEAD`, or `.lock`"],
    ),
    (
        "canonical-writer routing",
        ["Canonical-writer routing", "cyberos.core.ops.put"],
    ),
    (
        "STORE.yaml ACL honor-for-writes (§14.4.6)",
        ["STORE.yaml", "§14.4.6", "honour the ACL for writes"],
    ),
    (
        "sync_class export semantics (§14.3)",
        ["sync_class", "§14.3", "`shareable`", "`private`"],
    ),
)


def test_interop_present_bounded_vendored() -> None:
    """AC 3 — INTEROP.md exists, ≤ 6,000 chars, carries the five anchors,
    and build.sh vendors it into the payload beside the schema.

    The vendoring leg is asserted at the source (build.sh referencing
    `modules/memory/INTEROP.md`); executing a full scratch payload build
    is the final verification pass's job. build.sh itself belongs to the
    install workstream — this test only reads it.
    """
    assert _INTEROP.is_file(), (
        f"INTEROP.md missing at {_INTEROP} — AGENTS.md §14.1 binds "
        "non-ledger consumers to this document"
    )
    text = _INTEROP.read_text(encoding="utf-8")
    assert len(text) <= _MAX_CHARS, (
        f"INTEROP.md is {len(text)} chars; §14.1 caps it at {_MAX_CHARS}. "
        "Trim it — the bound is normative."
    )
    for label, needles in _ANCHORS:
        for needle in needles:
            assert needle in text, (
                f"INTEROP.md missing mandated content anchor {label!r} "
                f"(expected substring {needle!r})"
            )
    build_text = _BUILD_SH.read_text(encoding="utf-8")
    assert "modules/memory/INTEROP.md" in build_text, (
        "tools/install/build.sh no longer vendors modules/memory/INTEROP.md "
        "into the payload — §1.3 requires it to ship beside the schema"
    )


def test_changelog_records_hardening() -> None:
    """AC 8 — the CHANGELOG's top entry names all four deliverable groups."""
    text = _CHANGELOG.read_text(encoding="utf-8")
    # "Top entry" = everything between the first two `## ` headings.
    parts = text.split("\n## ")
    assert len(parts) >= 3, "CHANGELOG has fewer than two entries"
    top_entry = parts[1]
    for group_needle in (
        "Schema unification",
        "INTEROP.md",
        "Walker + doctor additions",
        "Store repair",
    ):
        assert group_needle in top_entry, (
            f"CHANGELOG top entry does not name deliverable group "
            f"{group_needle!r}"
        )
