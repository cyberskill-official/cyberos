"""FR-AI-012 §5 — Recall-floor test.

AC #13: per-type AND aggregate recall floors at ≥ 99%.
Used by FR-AI-013 recall-floor CI gate.
"""

from collections import defaultdict
from pathlib import Path

import pytest
import yaml

from presidio_analyzer import AnalyzerEngine

from recognizers import VN_RECOGNIZERS


@pytest.fixture
def analyzer():
    a = AnalyzerEngine()
    for rec in VN_RECOGNIZERS:
        a.registry.add_recognizer(rec)
    return a


def test_recall_at_least_99_percent_per_type(analyzer):
    """Per-type AND aggregate recall floors at ≥ 99%."""
    fixture_path = Path(__file__).parent / "fixtures" / "vn_pii_200_samples.yaml"
    samples = yaml.safe_load(fixture_path.read_text())

    correct_by_type = defaultdict(int)
    total_by_type = defaultdict(int)

    for sample in samples["samples"]:
        text = sample["text"]
        for expected in sample.get("expected_entities", []):
            total_by_type[expected] += 1
            results = analyzer.analyze(
                text=text, language="vi", entities=[expected]
            )
            if any(r.entity_type == expected for r in results):
                correct_by_type[expected] += 1

    failures = []
    for entity_type, total in total_by_type.items():
        if total == 0:
            continue
        recall = correct_by_type[entity_type] / total
        if recall < 0.99:
            failures.append(
                f"{entity_type}: recall={recall:.4f} "
                f"({correct_by_type[entity_type]}/{total})"
            )
    assert not failures, "Per-type recall floor violated:\n" + "\n".join(failures)

    # Aggregate floor too.
    total = sum(total_by_type.values())
    correct = sum(correct_by_type.values())
    aggregate = correct / total if total else 1.0
    assert aggregate >= 0.99, f"aggregate recall {aggregate:.4f} below 0.99 floor"
