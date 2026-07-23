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

To fix a drift failure (regenerate BOTH tracked copies — they must stay
byte-identical per TASK-MEMORY-303 §1.1)::

    cd modules/memory && \\
        python tools/cyberos_generate_schema.py --out memory.schema.json && \\
        python tools/cyberos_generate_schema.py --out cyberos/data/memory.schema.json

A missing committed schema is a FAIL, not a skip: this module exists to
catch exactly that class of breakage, and a conformance test that can
skip on its trigger condition is not a conformance test
(TASK-MEMORY-303 §1.2).
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import pytest

# Tests live at modules/memory/tests/test_schema_drift.py; parent.parent
# = modules/memory/ — the module root, where the committed copy lives.
_MEMORY = Path(__file__).resolve().parent.parent
_GENERATOR = _MEMORY / "tools" / "cyberos_generate_schema.py"
_COMMITTED = _MEMORY / "memory.schema.json"

_REGEN_HINT = (
    "Regenerate with:\n"
    "  cd modules/memory && "
    "python tools/cyberos_generate_schema.py --out memory.schema.json && "
    "python tools/cyberos_generate_schema.py --out cyberos/data/memory.schema.json"
)


def _require_committed() -> None:
    """FAIL (never skip) when the committed schema is absent (§1.2)."""
    if not _COMMITTED.is_file():
        pytest.fail(
            f"committed schema missing at {_COMMITTED} — the contract file "
            f"was deleted or moved without updating this test. {_REGEN_HINT}"
        )


def test_committed_schema_matches_generator_output() -> None:
    """`cyberos_generate_schema --check` must exit 0 against the committed file."""
    _require_committed()
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
        f"{_REGEN_HINT}"
    )


def test_schema_has_required_definitions() -> None:
    """Sanity check: the schema actually contains the expected definitions.

    Includes the three StoreAcl definitions §14.4.7 makes normative
    (TASK-MEMORY-303 §1.1) — their absence was the original schema fork.
    """
    _require_committed()
    schema = json.loads(_COMMITTED.read_text(encoding="utf-8"))
    defs = schema.get("definitions", {})
    for name in ("MemoryPath", "Sha256Hex", "Sha256Prefixed",
                 "AuditRecord", "Frontmatter", "Manifest", "Envelope",
                 "StoreAcl", "StoreAclEntry", "StoreAclMode"):
        assert name in defs, f"missing definition: {name}"


def test_schema_audit_record_op_is_permissive_string() -> None:
    """The op field is free-form text — historical rows from earlier protocol
    generations may carry op names not in any closed enum."""
    _require_committed()
    schema = json.loads(_COMMITTED.read_text(encoding="utf-8"))
    audit = schema["definitions"]["AuditRecord"]
    op_field = audit["properties"]["op"]
    assert op_field.get("type") == "string"
    assert "enum" not in op_field, (
        "op field must remain free-form: historical binlog rows may carry "
        "op names from earlier protocol generations."
    )
    assert op_field.get("minLength", 0) >= 1
