"""
Schema-drift regression test.

The committed ``memory.schema.json`` is generated from the
msgspec Struct definitions in :mod:`cyberos.core` by
``tools/cyberos_generate_schema.py``. If someone edits a Struct
(adds a field, narrows a type, etc.) without regenerating the schema,
the committed file silently goes stale and consumers validate against
the wrong contract.

This test runs the generator with ``--check`` and asserts the committed
file matches what the current Struct definitions produce. Catches the
"I changed a Struct but forgot to regenerate" class of bug in CI.

To fix a drift failure::

    cd memory && python tools/cyberos_generate_schema.py \\
        --out docs/memory.schema.json
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

# Tests live at memory/tests/test_schema_drift.py; parent.parent = memory/.
_MEMORY = Path(__file__).resolve().parent.parent
_GENERATOR = _MEMORY / "tools" / "cyberos_generate_schema.py"
_COMMITTED = _MEMORY / "docs" / "memory.schema.json"


def test_committed_schema_matches_generator_output() -> None:
    """`cyberos_generate_schema --check` must exit 0 against the committed file."""
    if not _COMMITTED.is_file():
        pytest.skip(f"committed schema not present at {_COMMITTED}")
    result = subprocess.run(
        [
            sys.executable, str(_GENERATOR),
            "--check",
            "--out", str(_COMMITTED),
        ],
        cwd=str(_MEMORY),
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, (
        "memory.schema.json is out of date vs cyberos.core Structs.\n"
        f"stderr: {result.stderr}\n"
        "Regenerate with:\n"
        "  cd memory && python tools/cyberos_generate_schema.py "
        "--out docs/memory.schema.json"
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


def test_schema_audit_record_op_is_permissive_string() -> None:
    """The op field is free-form text — historical rows from earlier protocol
    generations may carry op names not in any closed enum."""
    import json
    if not _COMMITTED.is_file():
        pytest.skip("committed schema not present")
    schema = json.loads(_COMMITTED.read_text(encoding="utf-8"))
    audit = schema["definitions"]["AuditRecord"]
    op_field = audit["properties"]["op"]
    assert op_field.get("type") == "string"
    assert "enum" not in op_field, (
        "op field must remain free-form: historical binlog rows may carry "
        "op names from earlier protocol generations."
    )
    assert op_field.get("minLength", 0) >= 1
