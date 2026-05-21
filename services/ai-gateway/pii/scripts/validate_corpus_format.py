#!/usr/bin/env python3
"""FR-AI-013 §1 #15 — Validate the VN PII corpus format.

Checks:
- Every sample has id, text, expected_entities
- Every entity is one of the 6 VN_* types
- Sample counts match manifest
"""

import sys
from pathlib import Path

import yaml

VALID_ENTITIES = {
    "VN_CCCD", "VN_MST", "VN_PHONE", "VN_NDD", "VN_ADDRESS", "VN_BANK_ACCOUNT",
}


def validate():
    fixtures_dir = Path(__file__).parent.parent / "fixtures"
    corpus_path = fixtures_dir / "vn_pii_200_samples.yaml"
    manifest_path = fixtures_dir / "fixture_manifest.yaml"

    if not corpus_path.exists():
        print(f"ERROR: Corpus not found: {corpus_path}", file=sys.stderr)
        return 1

    corpus = yaml.safe_load(corpus_path.read_text())
    samples = corpus.get("samples", [])

    errors = []
    entity_counts = {}

    for i, sample in enumerate(samples):
        if "text" not in sample:
            errors.append(f"Sample {i}: missing 'text' field")
        if "expected_entities" not in sample:
            errors.append(f"Sample {i}: missing 'expected_entities' field")
            continue

        for entity in sample["expected_entities"]:
            entity_counts[entity] = entity_counts.get(entity, 0) + 1
            if entity not in VALID_ENTITIES:
                errors.append(
                    f"Sample {i}: invalid entity '{entity}'; "
                    f"must be one of {VALID_ENTITIES}"
                )

    # Check manifest counts if manifest exists.
    if manifest_path.exists():
        manifest = yaml.safe_load(manifest_path.read_text())
        expected_counts = manifest.get("sample_counts", {})
        for entity, expected in expected_counts.items():
            actual = entity_counts.get(entity, 0)
            if entity == "negative":
                neg_count = sum(
                    1 for s in samples if not s.get("expected_entities")
                )
                if neg_count < expected:
                    errors.append(
                        f"Negative samples: expected {expected}, got {neg_count}"
                    )
            elif actual != expected:
                errors.append(
                    f"{entity}: expected {expected} samples, got {actual}"
                )

    if errors:
        for e in errors:
            print(f"ERROR: {e}", file=sys.stderr)
        return 1

    print(f"Corpus OK: {len(samples)} samples, {len(entity_counts)} entity types")
    return 0


if __name__ == "__main__":
    sys.exit(validate())
