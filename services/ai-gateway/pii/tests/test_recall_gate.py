"""FR-AI-013 recall gate: manifest pin, per-type recall, aggregate recall."""

from __future__ import annotations

from collections import defaultdict
import json
import os
from pathlib import Path

import pytest
import yaml
from fastapi.testclient import TestClient

os.environ["CYBEROS_PII_PATTERN_ONLY_NLP"] = "1"

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "fixtures" / "vn_pii_200_samples.yaml"
MANIFEST = ROOT / "fixtures" / "fixture_manifest.yaml"
REPORT_OUT = ROOT / f"recall_gate_report_{os.getenv('GITHUB_SHA', 'local')}.json"

ALL_ENTITIES = [
    "VN_CCCD",
    "VN_MST",
    "VN_PHONE",
    "VN_NDD",
    "VN_ADDRESS",
    "VN_BANK_ACCOUNT",
]


@pytest.fixture(scope="module")
def manifest():
    return yaml.safe_load(MANIFEST.read_text(encoding="utf-8"))


@pytest.fixture(scope="module")
def samples():
    return yaml.safe_load(FIXTURE.read_text(encoding="utf-8"))["samples"]


@pytest.fixture(scope="module")
def analyzer():
    from pattern_nlp import create_pattern_analyzer
    from recognizers import VN_RECOGNIZERS

    engine = create_pattern_analyzer()
    for recognizer in VN_RECOGNIZERS:
        engine.registry.add_recognizer(recognizer)
    return engine


def test_recognizer_versions_match_manifest(manifest):
    """AC #9: fixture pins the exact recognizer versions under test."""
    from presidio_sidecar import app

    response = TestClient(app).get("/recognizers/version")
    assert response.status_code == 200, "version endpoint unreachable"

    live_versions = response.json()
    expected_versions = manifest["recognizer_versions"]
    mismatches = [
        f"{entity}: live={live_versions.get(entity)!r} manifest={expected!r}"
        for entity, expected in expected_versions.items()
        if live_versions.get(entity) != expected
    ]
    assert not mismatches, (
        "fixture_version_mismatch — recognizer versions diverged from manifest:\n  "
        + "\n  ".join(mismatches)
    )


def test_sample_counts_match_manifest(samples, manifest):
    """AC #13: exact per-type and negative sample counts are enforced."""
    counts_by_type = defaultdict(int)
    negative_count = 0

    for sample in samples:
        expected_entities = sample["expected_entities"]
        if not expected_entities:
            negative_count += 1
        for entity in expected_entities:
            counts_by_type[entity] += 1

    for entity, expected_count in manifest["sample_counts"].items():
        actual = negative_count if entity == "negative" else counts_by_type[entity]
        assert actual == expected_count, (
            f"{entity} sample count mismatch: got {actual}, expected {expected_count}"
        )
    assert len(samples) == manifest["total_samples"]


def test_recall_per_recognizer_and_aggregate(analyzer, samples, manifest):
    """AC #10 + AC #11: every recognizer and the aggregate clear the recall floor."""
    correct_by_type = defaultdict(int)
    total_by_type = defaultdict(int)
    missed_samples_by_type = defaultdict(list)

    for sample in samples:
        if not sample["expected_entities"]:
            continue
        results = analyzer.analyze(
            text=sample["text"],
            language="vi",
            entities=ALL_ENTITIES,
        )
        actual_entities = {result.entity_type for result in results}
        for expected in sample["expected_entities"]:
            total_by_type[expected] += 1
            if expected in actual_entities:
                correct_by_type[expected] += 1
            else:
                missed_samples_by_type[expected].append(sample["id"])

    floors = manifest["recall_floors"]
    failures = []
    recall_per_type = {}
    for entity in ALL_ENTITIES:
        total = total_by_type[entity]
        assert total > 0, f"{entity} has zero samples in fixture"
        recall = correct_by_type[entity] / total
        recall_per_type[entity] = recall
        if recall < floors.get(entity, 0.99):
            missed = missed_samples_by_type[entity]
            failures.append(
                f"{entity}: recall={recall:.4f} ({correct_by_type[entity]}/{total}) "
                f"below floor {floors.get(entity, 0.99)}; missed={missed}"
            )

    total = sum(total_by_type.values())
    correct = sum(correct_by_type.values())
    aggregate = correct / total if total else 1.0
    if aggregate < manifest["aggregate_recall_floor"]:
        failures.append(
            f"aggregate: recall={aggregate:.4f} below floor "
            f"{manifest['aggregate_recall_floor']}"
        )

    report = {
        "fixture_version": manifest["fixture_version"],
        "recognizer_versions": manifest["recognizer_versions"],
        "recall_per_type": recall_per_type,
        "aggregate_recall": aggregate,
        "missed_samples_by_type": {
            entity: missed_samples_by_type[entity] for entity in ALL_ENTITIES
        },
        "sample_counts": dict(total_by_type),
        "report_path": str(REPORT_OUT.name),
    }
    REPORT_OUT.write_text(json.dumps(report, indent=2, sort_keys=True), encoding="utf-8")

    assert not failures, "Recall floor violated:\n  " + "\n  ".join(failures)


def test_negative_samples_no_high_confidence_matches(analyzer, samples):
    """AC #5: negative samples produce no high-confidence VN recognizer matches."""
    from recognizers.confidence import CONFIDENCE_HIGH

    failures = []
    for sample in samples:
        if sample["expected_entities"]:
            continue
        results = analyzer.analyze(
            text=sample["text"],
            language="vi",
            entities=ALL_ENTITIES,
        )
        high_confidence = [result for result in results if result.score >= CONFIDENCE_HIGH]
        if high_confidence:
            failures.append(
                f"{sample['id']}: "
                + ", ".join(
                    f"{result.entity_type}@{result.start}-{result.end}"
                    for result in high_confidence
                )
            )

    assert not failures, (
        "Negative samples produced high-confidence false positives:\n  "
        + "\n  ".join(failures)
    )
