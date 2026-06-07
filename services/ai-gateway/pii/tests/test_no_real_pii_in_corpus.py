"""FR-AI-013 no-real-PII corpus guard."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from check_no_real_pii import check  # noqa: E402

FIXTURE = ROOT / "fixtures" / "vn_pii_200_samples.yaml"


def test_current_corpus_has_no_known_internal_matches(tmp_path):
    known_table = tmp_path / "known_patterns.txt"
    known_table.write_text("# empty test table\n", encoding="utf-8")

    assert check(FIXTURE, known_table) == []


def test_known_internal_match_is_rejected(tmp_path):
    corpus_text = FIXTURE.read_text(encoding="utf-8")
    match = re.search(r"\b\d{12}\b", corpus_text)
    assert match, "fixture should contain a synthetic CCCD-like sequence"

    known_table = tmp_path / "known_patterns.txt"
    known_table.write_text(match.group(0) + "\n", encoding="utf-8")

    failures = check(FIXTURE, known_table)
    assert failures
    assert "real-CCCD-like sequence" in failures[0]
