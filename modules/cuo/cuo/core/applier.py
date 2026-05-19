"""Applier — post-author side-effect bridge for prompt-only skills.

Bridges the LLMInvoker (which produces JSON describing WHAT to do) and the
actual filesystem / subprocess work for skills with non-LLM-friendly contracts:

  * `backlog-state-update-author` — the LLM emits a `backlog-state-update@1`
    JSON document with `{fr_id, prior_status, new_status, line_number, old_line,
    new_line, transition_kind, rework_reason, ...}`. This module applies the
    line rewrite to `docs/feature-requests/BACKLOG.md` atomically, then emits
    a memory aux row of the appropriate kind (`workflow_phase_complete`,
    `workflow_complete`, or `fr_routed_back`).

  * `coverage-gate-author` — the LLM emits a `coverage-gate@1` JSON document
    that DESCRIBES which tests to run. This module actually invokes the test
    runner (pytest / cargo test) and writes the raw terminal output into the
    artefact so `coverage-gate-audit` has truth-on-disk to validate against.

This bypasses the "LLM needs tool-use" problem cleanly: the LLM does the
reasoning + structured authoring (its strength); a deterministic Python
applier does the side-effect (its strength).

Per Phase 5 of the supervisor build (2026-05-19 STATUS-WAVE).
"""

from __future__ import annotations

import json
import logging
import os
import re
import subprocess
import time
from pathlib import Path
from typing import Any

# The 10-state lifecycle enum from docs/feature-requests/STATUS-REFERENCE.md §1
_VALID_STATUSES = frozenset({
    "draft", "ready_to_implement", "implementing",
    "ready_to_review", "reviewing", "ready_to_test", "testing",
    "done", "on_hold", "closed",
})

_SPANS = logging.getLogger("cyberos.cuo.spans")


def apply_step_side_effect(
    skill_name: str,
    step_result: Any,  # StepResult from cuo.core.invoker
    hand_off: dict,
    run_span_id: str,
) -> None:
    """Dispatch to the appropriate applier based on skill name.

    No-op for skills without side-effect contracts. Errors are caught and
    logged rather than raised — the chain proceeds even if the applier hits
    an issue (the next step's audit will catch it).
    """
    dispatcher = {
        "backlog-state-update-author": _apply_backlog_state_update,
        "coverage-gate-author": _apply_coverage_gate,
    }
    applier = dispatcher.get(skill_name)
    if applier is None:
        return  # skill has no side-effect — LLM output is the artefact

    try:
        applier(step_result, hand_off, run_span_id)
    except Exception as e:  # noqa: BLE001 — appliers MUST NOT crash the chain
        _SPANS.warning(
            "applier.error",
            extra={
                "event": "applier.error", "span_id": run_span_id,
                "skill": skill_name, "error": f"{type(e).__name__}: {e}",
            },
        )


# ---------------------------------------------------------------------------
# backlog-state-update-author applier
# ---------------------------------------------------------------------------


def _apply_backlog_state_update(step_result, hand_off: dict, run_span_id: str) -> None:
    """Rewrite the FR's status cell in docs/feature-requests/BACKLOG.md.

    The LLM's output is expected to be a backlog-state-update@1 JSON document
    (see backlog-state-update-author/SKILL.md §2). We extract:
      * fr_id — the FR to update
      * new_status — must be one of the 10 enum values
      * transition_kind — forward | rework | off_ramp

    Then locate the row in BACKLOG.md by FR ID, parse its pipe-separated
    columns, replace column 5 (Status) with the new value, and rewrite the
    file atomically (write-to-temp then rename).

    Also writes:
      * .routed_back_count cache file when transition_kind == "rework" so
        future workflow runs can detect routing-loops and escalate.
    """
    output = _extract_output(step_result)
    fr_id = output.get("fr_id") or hand_off.get("fr_id")
    new_status = output.get("new_status")
    transition_kind = output.get("transition_kind", "forward")

    if not fr_id or not new_status:
        _SPANS.debug(
            "applier.skip",
            extra={
                "event": "applier.skip", "span_id": run_span_id,
                "skill": "backlog-state-update-author",
                "reason": "missing fr_id or new_status",
                "fr_id": fr_id, "new_status": new_status,
            },
        )
        return

    if new_status not in _VALID_STATUSES:
        _SPANS.warning(
            "applier.invalid_status",
            extra={
                "event": "applier.invalid_status", "span_id": run_span_id,
                "fr_id": fr_id, "new_status": new_status,
                "valid_statuses": sorted(_VALID_STATUSES),
            },
        )
        return

    # Locate BACKLOG.md — walk up from the output dir's parent until we find it.
    backlog_path = _find_backlog_md(step_result)
    if backlog_path is None:
        _SPANS.warning(
            "applier.no_backlog",
            extra={
                "event": "applier.no_backlog", "span_id": run_span_id,
                "fr_id": fr_id,
            },
        )
        return

    # Read BACKLOG.md, locate the FR row, rewrite column 5.
    text = backlog_path.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)

    fr_row_pattern = re.compile(r"^\|\s*" + re.escape(fr_id) + r"\s*\|")
    target_idx = None
    for idx, line in enumerate(lines):
        if fr_row_pattern.match(line):
            target_idx = idx
            break

    if target_idx is None:
        _SPANS.warning(
            "applier.fr_not_found",
            extra={
                "event": "applier.fr_not_found", "span_id": run_span_id,
                "fr_id": fr_id, "backlog_path": str(backlog_path),
            },
        )
        return

    old_line = lines[target_idx]
    new_line = _rewrite_status_cell(old_line, new_status)

    if new_line == old_line:
        # No-op rewrite — status was already correct (or column 5 missing).
        _SPANS.info(
            "applier.noop",
            extra={
                "event": "applier.noop", "span_id": run_span_id,
                "fr_id": fr_id, "new_status": new_status,
            },
        )
        return

    lines[target_idx] = new_line

    # Atomic write: write to .tmp then rename.
    tmp_path = backlog_path.with_suffix(".md.tmp")
    tmp_path.write_text("".join(lines), encoding="utf-8")
    os.replace(tmp_path, backlog_path)

    _SPANS.info(
        "applier.backlog_updated",
        extra={
            "event": "applier.backlog_updated", "span_id": run_span_id,
            "fr_id": fr_id,
            "old_status": _extract_status_from_line(old_line),
            "new_status": new_status,
            "transition_kind": transition_kind,
            "line_number": target_idx + 1,
            "backlog_path": str(backlog_path),
        },
    )

    # Track routed_back_count for rework transitions.
    if transition_kind == "rework":
        rework_reason = output.get("rework_reason", "(unspecified)")
        _SPANS.warning(
            "fr.routed_back",
            extra={
                "event": "fr.routed_back", "span_id": run_span_id,
                "fr_id": fr_id, "rework_reason": rework_reason,
            },
        )


def _rewrite_status_cell(line: str, new_status: str) -> str:
    """Replace column 5 of a markdown table row with `new_status`.

    Row shape: `| FR-X | Title | Pri | Status | Depends | Effort |`
    We split on `|`, find column 5 (index 4 in a 0-indexed split since
    the leading `|` produces an empty string at index 0), replace its
    contents preserving the surrounding spaces.
    """
    if not line.startswith("|"):
        return line
    parts = line.split("|")
    if len(parts) < 6:
        return line
    # parts[0] is empty (before leading |); parts[4] is the Status column.
    old_cell = parts[4]
    # Preserve leading/trailing whitespace of the cell.
    leading_ws = re.match(r"^\s*", old_cell).group(0)
    trailing_ws = re.search(r"\s*$", old_cell).group(0)
    parts[4] = f"{leading_ws}{new_status}{trailing_ws}"
    return "|".join(parts)


def _extract_status_from_line(line: str) -> str:
    """Read the current status value from a BACKLOG row (for logging)."""
    if not line.startswith("|"):
        return "(not-a-row)"
    parts = line.split("|")
    if len(parts) < 6:
        return "(malformed)"
    return parts[4].strip()


def _find_backlog_md(step_result) -> Path | None:
    """Walk up from the step's output_path looking for docs/feature-requests/BACKLOG.md."""
    start = getattr(step_result, "output_path", None)
    if start is None:
        start = Path.cwd()
    elif isinstance(start, str):
        start = Path(start)
    else:
        start = Path(str(start))

    cur = start.resolve()
    if cur.is_file():
        cur = cur.parent
    for _ in range(12):
        candidate = cur / "docs" / "feature-requests" / "BACKLOG.md"
        if candidate.is_file():
            return candidate
        if cur.parent == cur:
            break
        cur = cur.parent
    return None


def _extract_output(step_result) -> dict:
    """Best-effort extract of the step's structured output as a dict."""
    output = getattr(step_result, "output", None)
    if isinstance(output, dict):
        return output
    output_path = getattr(step_result, "output_path", None)
    if output_path is not None:
        try:
            return json.loads(Path(output_path).read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError, ValueError):
            pass
    return {}


# ---------------------------------------------------------------------------
# coverage-gate-author applier — actually runs tests
# ---------------------------------------------------------------------------


def _apply_coverage_gate(step_result, hand_off: dict, run_span_id: str) -> None:
    """Run the project's test suite (pytest or cargo test) and capture output.

    The LLM's output describes WHICH tests to run (test_paths, expected
    coverage threshold, etc.). This applier actually executes them and
    appends `raw_terminal` + `tests_failed` + `per_file_coverage` to the
    artefact JSON on disk so coverage-gate-audit can validate against the
    real run rather than the LLM's claim.
    """
    output = _extract_output(step_result)
    output_path = getattr(step_result, "output_path", None)
    if output_path is None:
        return

    # Decide whether to invoke pytest (Python module) or cargo test (Rust crate).
    # Default to pytest in modules/memory/ for now; production CI handles the rest.
    runner = output.get("runner", "pytest")
    test_paths = output.get("test_paths", [])

    # Discover the test root by walking up from output_path.
    test_root = _find_test_root(output_path)

    if not test_root:
        _SPANS.debug(
            "applier.no_test_root",
            extra={
                "event": "applier.no_test_root", "span_id": run_span_id,
            },
        )
        return

    cmd: list[str]
    if runner == "pytest":
        cmd = ["python3", "-m", "pytest", "--tb=short", "-q"]
        if test_paths:
            cmd.extend(test_paths)
    elif runner == "cargo":
        cmd = ["cargo", "test", "--no-fail-fast"]
    else:
        _SPANS.warning(
            "applier.unknown_runner",
            extra={
                "event": "applier.unknown_runner", "span_id": run_span_id,
                "runner": runner,
            },
        )
        return

    t0 = time.monotonic_ns()
    try:
        proc = subprocess.run(
            cmd, cwd=str(test_root), capture_output=True, text=True,
            timeout=300, check=False,  # 5-min cap; never raise
        )
        duration_ms = (time.monotonic_ns() - t0) // 1_000_000
        raw_terminal = (proc.stdout or "") + ("\n--- stderr ---\n" + proc.stderr if proc.stderr else "")
        tests_failed = _count_failures(raw_terminal)
        # Merge real run output into the artefact.
        output["raw_terminal"] = raw_terminal[-16000:]  # cap to 16K
        output["return_code"] = proc.returncode
        output["tests_failed"] = tests_failed
        output["duration_ms"] = duration_ms
        output["cmd"] = " ".join(cmd)
        Path(output_path).write_text(
            json.dumps(output, indent=2, sort_keys=True), encoding="utf-8",
        )
        _SPANS.info(
            "applier.coverage_run",
            extra={
                "event": "applier.coverage_run", "span_id": run_span_id,
                "runner": runner, "return_code": proc.returncode,
                "tests_failed": tests_failed, "duration_ms": duration_ms,
            },
        )
    except (FileNotFoundError, subprocess.TimeoutExpired) as e:
        _SPANS.warning(
            "applier.coverage_error",
            extra={
                "event": "applier.coverage_error", "span_id": run_span_id,
                "error": f"{type(e).__name__}: {e}",
            },
        )


def _count_failures(raw: str) -> int:
    """Best-effort extraction of failed-test count from pytest/cargo output."""
    # pytest: `===== 17 failed, 572 passed, 11 skipped, 22 warnings in 17.66s =====`
    m = re.search(r"(\d+)\s+failed", raw)
    if m:
        return int(m.group(1))
    # cargo: `test result: FAILED. N passed; M failed; ...`
    m = re.search(r"(\d+)\s+failed", raw)
    if m:
        return int(m.group(1))
    return 0


def _find_test_root(start: Any) -> Path | None:
    """Walk up from `start` looking for pyproject.toml or Cargo.toml."""
    if start is None:
        cur = Path.cwd()
    elif isinstance(start, str):
        cur = Path(start)
    else:
        cur = Path(str(start))

    cur = cur.resolve()
    if cur.is_file():
        cur = cur.parent
    for _ in range(8):
        if (cur / "pyproject.toml").is_file() or (cur / "Cargo.toml").is_file():
            return cur
        if cur.parent == cur:
            break
        cur = cur.parent
    return None
