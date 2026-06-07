"""FR-AI-013 runtime budget guard for the recall gate."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import time

import yaml

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures" / "fixture_manifest.yaml"


def test_recall_gate_completes_within_budget():
    """AC #4: the recall gate completes inside manifest runtime_budget_seconds."""
    budget = yaml.safe_load(MANIFEST.read_text(encoding="utf-8"))[
        "runtime_budget_seconds"
    ]
    env = {**os.environ, "CYBEROS_PII_PATTERN_ONLY_NLP": "1"}

    started_at = time.monotonic()
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "pytest",
            "-q",
            "tests/test_recall_gate.py::test_recall_per_recognizer_and_aggregate",
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=budget + 30,
        env=env,
    )
    elapsed = time.monotonic() - started_at

    assert result.returncode == 0, (
        "recall gate subprocess failed\n"
        f"stdout:\n{result.stdout}\n"
        f"stderr:\n{result.stderr}"
    )
    assert elapsed < budget, (
        f"recall gate took {elapsed:.1f}s, budget is {budget}s\n"
        f"stdout:\n{result.stdout}\n"
        f"stderr:\n{result.stderr}"
    )
