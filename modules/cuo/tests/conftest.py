"""Shared test fixtures for CUO tests."""

from __future__ import annotations

import json
import time
from pathlib import Path

from cuo.core.invoker import Invoker, StepResult


class FakeInvoker(Invoker):
    """Test-only invoker that produces deterministic synthetic output.

    Replaces the removed MockInvoker for test infrastructure.
    NOT for production use — tests import this directly.
    """

    def invoke(
        self,
        skill_name: str,
        inputs: dict,
        skill_root: Path,
        output_dir: Path,
        step_num: int,
        *,
        file_prefix: str = "",
    ) -> StepResult:
        t0 = time.monotonic_ns()
        result = StepResult(step=step_num, skill=skill_name, status="MOCKED")

        skill_dir = skill_root / skill_name
        if not (skill_dir / "SKILL.md").is_file():
            result.status = "FAILED"
            result.notes.append(f"skill not found at {skill_dir}/SKILL.md")
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result

        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / f"{file_prefix}step{step_num:02d}_{skill_name}.json"
        output = {
            "skill": skill_name,
            "step": step_num,
            "synthetic": True,
            "inputs": _stringify_inputs(inputs),
            "ts_ns": time.time_ns(),
        }
        try:
            output_path.write_text(json.dumps(output, indent=2, sort_keys=True), encoding="utf-8")
        except OSError as e:
            result.status = "FAILED"
            result.notes.append(f"could not write output: {e}")
            result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
            return result

        result.output = output
        result.output_path = output_path
        result.duration_ms = (time.monotonic_ns() - t0) // 1_000_000
        return result


def _stringify_inputs(inputs: dict) -> dict:
    out: dict = {}
    for k, v in inputs.items():
        if isinstance(v, Path):
            out[k] = str(v)
        elif isinstance(v, (list, tuple)):
            out[k] = [_stringify_one(x) for x in v]
        elif isinstance(v, set):
            out[k] = sorted([_stringify_one(x) for x in v])
        elif isinstance(v, dict):
            out[k] = _stringify_inputs(v)
        elif isinstance(v, (str, int, float, bool, type(None))):
            out[k] = v
        else:
            out[k] = repr(v)
    return out


def _stringify_one(v):
    if isinstance(v, Path):
        return str(v)
    if isinstance(v, dict):
        return _stringify_inputs(v)
    if isinstance(v, (str, int, float, bool, type(None))):
        return v
    return repr(v)
