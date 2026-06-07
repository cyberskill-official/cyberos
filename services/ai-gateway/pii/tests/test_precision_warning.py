"""FR-AI-013 precision report: warning-only, never a merge gate."""

from __future__ import annotations

from collections import defaultdict
import json
import os
from pathlib import Path

import yaml

os.environ["CYBEROS_PII_PATTERN_ONLY_NLP"] = "1"

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "fixtures" / "vn_pii_200_samples.yaml"
MANIFEST = ROOT / "fixtures" / "fixture_manifest.yaml"
WARN_OUT = ROOT / f"precision_warning_report_{os.getenv('GITHUB_SHA', 'local')}.json"
PRECISION_DELTA_WARN = 0.05


def test_precision_reported_no_gate():
    """AC #6: precision is emitted for trend monitoring but does not fail the PR."""
    from pattern_nlp import create_pattern_analyzer
    from recognizers import VN_RECOGNIZERS
    from recognizers.confidence import CONFIDENCE_HIGH

    analyzer = create_pattern_analyzer()
    for recognizer in VN_RECOGNIZERS:
        analyzer.registry.add_recognizer(recognizer)

    manifest = yaml.safe_load(MANIFEST.read_text(encoding="utf-8"))
    samples = yaml.safe_load(FIXTURE.read_text(encoding="utf-8"))["samples"]
    baselines = manifest["precision_baselines_prior_quarter"]

    true_positive = defaultdict(int)
    false_positive = defaultdict(int)

    for sample in samples:
        expected = set(sample["expected_entities"])
        results = analyzer.analyze(
            text=sample["text"],
            language="vi",
            entities=list(baselines),
        )
        actual_high_confidence = {
            result.entity_type for result in results if result.score >= CONFIDENCE_HIGH
        }
        for entity in actual_high_confidence:
            if entity in expected:
                true_positive[entity] += 1
            else:
                false_positive[entity] += 1

    precision_per_type = {}
    warnings = []
    for entity, baseline in baselines.items():
        denominator = true_positive[entity] + false_positive[entity]
        precision = true_positive[entity] / denominator if denominator else 1.0
        precision_per_type[entity] = precision
        delta = precision - baseline
        if delta < -PRECISION_DELTA_WARN:
            warnings.append(
                f"{entity}: precision={precision:.4f} baseline={baseline:.4f} "
                f"delta={delta:+.4f}"
            )

    WARN_OUT.write_text(
        json.dumps(
            {
                "fixture_version": manifest["fixture_version"],
                "precision_per_type": precision_per_type,
                "warnings": warnings,
            },
            indent=2,
            sort_keys=True,
        ),
        encoding="utf-8",
    )

    for warning in warnings:
        print(f"::warning:: precision-regression: {warning}")

    assert True
