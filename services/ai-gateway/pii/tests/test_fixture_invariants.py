"""FR-AI-013 fixture-format invariants."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from validate_corpus_format import validate  # noqa: E402


def test_fixture_format_validator_passes():
    """AC #13-#15: counts, fields, spans, ids, and provenance validate."""
    errors = validate(ROOT / "fixtures")
    assert not errors, "Fixture invariants violated:\n  " + "\n  ".join(errors)
