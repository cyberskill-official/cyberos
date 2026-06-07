#!/usr/bin/env python3
"""Reject corpus rows containing digit sequences from a known internal PII table."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

import yaml

CCCD_RE = re.compile(r"\b\d{12}\b")
MST_RE = re.compile(r"\b\d{10}(?:-\d{3})?\b")


def _load_known_patterns(path: Path) -> set[str]:
    values = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        value = line.strip()
        if value and not value.startswith("#"):
            values.add(value)
    return values


def check(corpus_path: Path, known_patterns_path: Path) -> list[str]:
    samples = yaml.safe_load(corpus_path.read_text(encoding="utf-8"))["samples"]
    known = _load_known_patterns(known_patterns_path)
    failures: list[str] = []

    for sample in samples:
        sample_id = sample.get("id", "<missing-id>")
        text = sample.get("text", "")
        for digit_run in CCCD_RE.findall(text):
            if digit_run in known:
                failures.append(
                    f"{sample_id}: real-CCCD-like sequence {digit_run!r} matches internal record"
                )
        for digit_run in MST_RE.findall(text):
            if digit_run in known:
                failures.append(
                    f"{sample_id}: real-MST-like sequence {digit_run!r} matches internal record"
                )

    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("corpus", type=Path)
    parser.add_argument("known_patterns", type=Path)
    args = parser.parse_args()

    failures = check(args.corpus, args.known_patterns)
    if failures:
        print("pre-commit hook: real PII detected in corpus:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        print(
            "\nUse scripts/regen_fixture.py to generate format-valid synthetic equivalents.",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
