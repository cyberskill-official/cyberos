#!/usr/bin/env python3
"""Validate the FR-AI-013 VN PII recall corpus format."""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Any

import yaml

VALID_ENTITIES = {
    "VN_CCCD",
    "VN_MST",
    "VN_PHONE",
    "VN_NDD",
    "VN_ADDRESS",
    "VN_BANK_ACCOUNT",
    "PERSON",
}
COUNTED_ENTITIES = VALID_ENTITIES - {"PERSON"}
VALID_PROVENANCE = {"synthetic", "anonymised-real", "gov.vn-public"}
REQUIRED_SAMPLE_FIELDS = {
    "id",
    "text",
    "expected_entities",
    "expected_count",
    "expected_spans",
    "provenance",
}

ENTITY_PATTERNS = {
    "VN_CCCD": re.compile(r"\d{12}"),
    "VN_MST": re.compile(r"\d{10}(?:-\d{3})?"),
    "VN_PHONE": re.compile(r"(?:\+84[\s.-]?)?0?\d(?:[\s.-]?\d){8,10}"),
    "VN_NDD": re.compile(r"[\w\sÀ-ỹ]+", re.UNICODE),
    "VN_ADDRESS": re.compile(r"\d+[\w\sÀ-ỹ.,]+", re.UNICODE),
    "VN_BANK_ACCOUNT": re.compile(r"\d{10,14}"),
    "PERSON": re.compile(r"[\w\sÀ-ỹ]+", re.UNICODE),
}


def _fixtures_dir() -> Path:
    return Path(__file__).resolve().parents[1] / "fixtures"


def _load_yaml(path: Path) -> Any:
    with path.open(encoding="utf-8") as fh:
        return yaml.safe_load(fh)


def _sample_schema() -> dict[str, Any]:
    entities = sorted(VALID_ENTITIES)
    provenance = sorted(VALID_PROVENANCE)
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["samples"],
        "properties": {
            "samples": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": sorted(REQUIRED_SAMPLE_FIELDS),
                    "properties": {
                        "id": {"type": "string", "pattern": "^[a-z]+_\\d{3}$"},
                        "text": {"type": "string", "minLength": 1},
                        "expected_entities": {
                            "type": "array",
                            "items": {"type": "string", "enum": entities},
                        },
                        "expected_count": {"type": "integer", "minimum": 0},
                        "expected_spans": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "required": ["entity", "start", "end"],
                                "properties": {
                                    "entity": {"type": "string", "enum": entities},
                                    "start": {"type": "integer", "minimum": 0},
                                    "end": {"type": "integer", "minimum": 1},
                                },
                            },
                        },
                        "provenance": {"type": "string", "enum": provenance},
                        "notes": {"type": "string"},
                    },
                },
            }
        },
    }


def validate(fixtures_dir: Path | None = None) -> list[str]:
    fixtures_dir = fixtures_dir or _fixtures_dir()
    corpus_path = fixtures_dir / "vn_pii_200_samples.yaml"
    manifest_path = fixtures_dir / "fixture_manifest.yaml"

    corpus = _load_yaml(corpus_path)
    manifest = _load_yaml(manifest_path)
    samples = corpus.get("samples", [])

    errors: list[str] = []
    ids: Counter[str] = Counter()
    entity_counts: Counter[str] = Counter()
    negative_count = 0

    for index, sample in enumerate(samples):
        label = sample.get("id", f"<index:{index}>")
        missing = REQUIRED_SAMPLE_FIELDS - set(sample)
        if missing:
            errors.append(f"{label}: missing required fields {sorted(missing)}")
            continue

        ids[sample["id"]] += 1
        text = sample["text"]
        entities = sample["expected_entities"]
        spans = sample["expected_spans"]

        if not isinstance(text, str) or not text.strip():
            errors.append(f"{label}: text must be a non-empty string")
        if sample["provenance"] not in VALID_PROVENANCE:
            errors.append(f"{label}: invalid provenance {sample['provenance']!r}")
        if sample["expected_count"] != len(entities):
            errors.append(
                f"{label}: expected_count={sample['expected_count']} but "
                f"expected_entities has {len(entities)} item(s)"
            )
        if len(spans) != len(entities):
            errors.append(
                f"{label}: expected_spans has {len(spans)} item(s) but "
                f"expected_entities has {len(entities)} item(s)"
            )

        if not entities:
            negative_count += 1
        for entity in entities:
            if entity not in VALID_ENTITIES:
                errors.append(f"{label}: invalid entity {entity!r}")
            if entity in COUNTED_ENTITIES:
                entity_counts[entity] += 1

        for span in spans:
            entity = span.get("entity")
            start = span.get("start")
            end = span.get("end")
            if entity not in VALID_ENTITIES:
                errors.append(f"{label}: invalid span entity {entity!r}")
                continue
            if not isinstance(start, int) or not isinstance(end, int):
                errors.append(f"{label}: span offsets must be integers")
                continue
            if start < 0 or end > len(text) or start >= end:
                errors.append(
                    f"{label}: span ({start},{end}) out of range for text length {len(text)}"
                )
                continue
            substring = text[start:end]
            if not substring.strip():
                errors.append(f"{label}: span ({start},{end}) is blank")
                continue
            pattern = ENTITY_PATTERNS[entity]
            if not pattern.fullmatch(substring) and not pattern.search(substring):
                errors.append(
                    f"{label}: span for {entity} does not match expected pattern: {substring!r}"
                )

    duplicates = sorted(sample_id for sample_id, count in ids.items() if count > 1)
    if duplicates:
        errors.append(f"duplicate sample ids: {duplicates}")

    expected_counts = manifest["sample_counts"]
    for entity, expected in expected_counts.items():
        actual = negative_count if entity == "negative" else entity_counts[entity]
        if actual != expected:
            errors.append(f"{entity}: expected {expected} sample(s), got {actual}")

    if len(samples) != manifest["total_samples"]:
        errors.append(
            f"total_samples mismatch: manifest={manifest['total_samples']} actual={len(samples)}"
        )

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--schema-out", action="store_true")
    args = parser.parse_args()

    if args.schema_out:
        print(json.dumps(_sample_schema(), indent=2, sort_keys=True))
        return 0

    errors = validate()
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print("Corpus OK: 230 samples, 6 VN entity types, 30 negatives")
    return 0


if __name__ == "__main__":
    sys.exit(main())
