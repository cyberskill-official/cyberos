"""
Schema-drift regression test.

The committed ``docs/memory/memory.schema.json`` is generated from the
msgspec Struct definitions in :mod:`cyberos.core` by
``runtime/tools/cyberos_generate_schema.py``. If someone edits a Struct
(adds a field, narrows a type, etc.) without regenerating the schema,
the committed file silently goes stale and consumers validate against
the wrong contract.

This test runs the generator with ``--check`` and asserts the committed
file matches what the current Struct definitions produce. Catches the
"I changed a Struct but forgot to regenerate" class of bug in CI.

To fix a drift failure::

    python -m runtime.tools.cyberos_generate_schema \\
        --out docs/memory/memory.schema.json
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

_REPO = Path(__file__).resolve().parent.parent
_COMMITTED = _REPO / "docs" / "memory" / "memory.schema.json"


def test_committed_schema_matches_generator_output() -> None:
    """`cyberos_generate_schema --check` must exit 0 against the committed file."""
    if not _COMMITTED.is_file():
        pytest.skip(f"committed schema not present at {_COMMITTED}")
    result = subprocess.run(
        [
            sys.executable, "-m", "runtime.tools.cyberos_generate_schema",
            "--check",
            "--out", str(_COMMITTED),
        ],
        cwd=str(_REPO),
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, (
        "memory.schema.json is out of date vs cyberos.core Structs.\n"
        f"stderr: {result.stderr}\n"
        "Regenerate with:\n"
        "  python -m runtime.tools.cyberos_generate_schema "
        "--out docs/memory/memory.schema.json"
    )


def test_schema_has_required_definitions() -> None:
    """Sanity check: the schema actually contains the expected definitions."""
    import json
    if not _COMMITTED.is_file():
        pytest.skip("committed schema not present")
    schema = json.loads(_COMMITTED.read_text(encoding="utf-8"))
    defs = schema.get("definitions", {})
    for name in ("MemoryPath", "Sha256Hex", "Sha256Prefixed",
                 "AuditRecord", "Frontmatter", "Manifest", "Envelope"):
        assert name in defs, f"missing definition: {name}"


def test_schema_audit_record_has_op_enum() -> None:
    """The op enum on AuditRecord is the contract surface — pin it explicitly."""
    import json
    if not _COMMITTED.is_file():
        pytest.skip("committed schema not present")
    schema = json.loads(_COMMITTED.read_text(encoding="utf-8"))
    audit = schema["definitions"]["AuditRecord"]
    op_enum = audit["properties"]["op"]["enum"]
    # Required for v1 + v2 compat
    for op in ("view", "create", "str_replace", "insert", "delete", "rename"):
        assert op in op_enum, f"v1 op missing from enum: {op}"
    # Session boundary rows
    for op in ("session.start", "session.end"):
        assert op in op_enum, f"session boundary op missing: {op}"
