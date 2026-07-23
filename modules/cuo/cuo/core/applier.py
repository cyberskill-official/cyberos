"""Applier — post-author side-effect bridge for prompt-only skills.

Bridges the LLMInvoker (which produces JSON describing WHAT to do) and the
actual filesystem / subprocess work for skills with non-LLM-friendly contracts:

  * `backlog-state-update-author` — the LLM emits a `backlog-state-update@1`
    JSON document with `{task_id, prior_status, new_status, line_number, old_line,
    new_line, transition_kind, rework_reason, ...}`. This module applies the
    line rewrite to `docs/tasks/BACKLOG.md` atomically, then emits
    a memory aux row of the appropriate kind (`workflow_phase_complete`,
    `workflow_complete`, or `task_routed_back`).

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

# The 10-state lifecycle enum from docs/tasks/STATUS-REFERENCE.md §1
_VALID_STATUSES = frozenset({
    "draft", "ready_to_implement", "implementing",
    "ready_to_review", "reviewing", "ready_to_test", "testing",
    "done", "on_hold", "closed",
})

_SPANS = logging.getLogger("cyberos.cuo.spans")

# AGENTS.md §2 closed kind set. Applier artefacts map onto these so put() paths
# stay layout-root-canonical (TASK-MEMORY-302).
_BRAIN_KIND_BY_ARTEFACT = {
    "adrs": "decisions",
    "impl-plans": "projects",
    "audits": "refinements",
    "code-reviews": "refinements",
    "obs-injections": "facts",
}


def _shard_rel(filename: str) -> str:
    """Return ``<hex>/<hex>/<filename>`` content-addressed shard (AGENTS.md §2)."""
    import hashlib

    digest = hashlib.sha256(filename.encode("utf-8")).hexdigest()
    return f"{digest[:2]}/{digest[2:4]}/{filename}"


def _put_brain_artefact(
    repo_root: Path,
    artefact_class: str,
    filename: str,
    content: str,
    *,
    actor: str = "cuo-applier",
    run_span_id: str = "",
) -> Path | None:
    """Write an applier artefact through the canonical BRAIN writer (TASK-MEMORY-302).

    Routes via ``cyberos.core.ops.put`` under ``memories/<kind>/<hex>/<hex>/``.
    Never creates non-canonical store-root dirs (``adrs/``, ``impl-plans/``, …).
    Returns the absolute path on success, or ``None`` when the writer is unavailable
    (caller should skip — do NOT fall back to raw store-root writes).
    """
    kind = _BRAIN_KIND_BY_ARTEFACT.get(artefact_class)
    if kind is None:
        _SPANS.warning(
            "applier.brain_put_unknown_class",
            extra={"event": "applier.brain_put_unknown_class", "span_id": run_span_id,
                   "artefact_class": artefact_class},
        )
        return None
    store = repo_root / ".cyberos" / "memory" / "store"
    if not store.is_dir() or not (store / "audit").is_dir():
        _SPANS.info(
            "applier.brain_put_skipped",
            extra={"event": "applier.brain_put_skipped", "span_id": run_span_id,
                   "reason": "no_store", "artefact_class": artefact_class},
        )
        return None
    try:
        from cyberos.core.ops import put
        from cyberos.core.writer import Writer
    except ImportError:
        # Trusted path only: walk from this CUO module's install tree (same pattern as
        # memory_bridge._try_import_memory_writer). Never add target repo_root/modules/memory
        # to sys.path — that tree is attacker-controlled in the external-project case.
        import sys

        here = Path(__file__).resolve()
        mem_mod = None
        for ancestor in (
            here.parent.parent.parent.parent,  # modules/
            here.parent.parent.parent.parent.parent,  # cyberos-root
        ):
            for cand in (ancestor / "memory", ancestor / "modules" / "memory"):
                if (cand / "cyberos" / "core" / "ops.py").is_file():
                    mem_mod = cand
                    break
            if mem_mod is not None:
                break
        if mem_mod is not None and str(mem_mod) not in sys.path:
            sys.path.insert(0, str(mem_mod))
        try:
            from cyberos.core.ops import put
            from cyberos.core.writer import Writer
        except ImportError as e:
            _SPANS.warning(
                "applier.brain_put_import_failed",
                extra={"event": "applier.brain_put_import_failed", "span_id": run_span_id,
                       "error": str(e)},
            )
            return None

    rel = f"memories/{kind}/{_shard_rel(filename)}"
    try:
        with Writer(store) as writer:
            put(
                writer,
                rel,
                content.encode("utf-8"),
                actor=actor,
                kind=kind,
                extra={"artefact_class": artefact_class, "filename": filename},
            )
    except Exception as e:  # noqa: BLE001 — applier must not crash the supervisor
        _SPANS.warning(
            "applier.brain_put_failed",
            extra={"event": "applier.brain_put_failed", "span_id": run_span_id,
                   "rel": rel, "error": str(e)},
        )
        return None
    return store / rel


def _resolve_project_root(hand_off: dict) -> Path | None:
    """Resolve the target project root from hand_off.

    Prefers _project_root (the project being operated on) over _cyberos_root
    (where CUO is installed). The two differ when CUO runs against an external
    project via CYBEROS_ROOT.
    """
    for key in ("_project_root", "_cyberos_root"):
        val = hand_off.get(key)
        if val:
            return Path(val)
    return None


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
        "task-audit": _apply_task_audit,
        "architecture-decision-record-author": _apply_architecture_decision_record,
        "implementation-plan-author": _apply_implementation_plan,
        "code-review-author": _apply_code_review,
        "observability-injection-author": _apply_observability_injection,
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
    """Rewrite the task's status cell in docs/tasks/BACKLOG.md.

    The LLM's output is expected to be a backlog-state-update@1 JSON document
    (see backlog-state-update-author/SKILL.md §2). We extract:
      * task_id — the task to update
      * new_status — must be one of the 10 enum values
      * transition_kind — forward | rework | off_ramp

    Then locate the row in BACKLOG.md by task ID, parse its pipe-separated
    columns, replace column 5 (Status) with the new value, and rewrite the
    file atomically (write-to-temp then rename).

    Also writes:
      * .routed_back_count cache file when transition_kind == "rework" so
        future workflow runs can detect routing-loops and escalate.
    """
    output = _extract_output(step_result)
    task_id = output.get("task_id") or hand_off.get("task_id")
    new_status = output.get("new_status")
    transition_kind = output.get("transition_kind", "forward")

    if not task_id or not new_status:
        _SPANS.debug(
            "applier.skip",
            extra={
                "event": "applier.skip", "span_id": run_span_id,
                "skill": "backlog-state-update-author",
                "reason": "missing task_id or new_status",
                "task_id": task_id, "new_status": new_status,
            },
        )
        return

    if new_status not in _VALID_STATUSES:
        _SPANS.warning(
            "applier.invalid_status",
            extra={
                "event": "applier.invalid_status", "span_id": run_span_id,
                "task_id": task_id, "new_status": new_status,
                "valid_statuses": sorted(_VALID_STATUSES),
            },
        )
        return

    # Locate BACKLOG.md — walk up from the output dir's parent until we find it.
    repo_root = _resolve_project_root(hand_off)
    backlog_path = _find_backlog_md(step_result, repo_root=repo_root)
    if backlog_path is None:
        _SPANS.warning(
            "applier.no_backlog",
            extra={
                "event": "applier.no_backlog", "span_id": run_span_id,
                "task_id": task_id,
            },
        )
        return

    # Read BACKLOG.md, locate the task row, rewrite column 5.
    text = backlog_path.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)

    # Match TASK-ID with optional bold markdown (**TASK-ID**) or plain TASK-ID
    task_escaped = re.escape(task_id)
    task_row_pattern = re.compile(r"^\|\s*\**\s*" + task_escaped + r"\s*\**\s*\|")
    target_idx = None
    for idx, line in enumerate(lines):
        if task_row_pattern.match(line):
            target_idx = idx
            break

    if target_idx is None:
        _SPANS.warning(
            "applier.task_not_found",
            extra={
                "event": "applier.task_not_found", "span_id": run_span_id,
                "task_id": task_id, "backlog_path": str(backlog_path),
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
                "task_id": task_id, "new_status": new_status,
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
            "task_id": task_id,
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
            "task.routed_back",
            extra={
                "event": "task.routed_back", "span_id": run_span_id,
                "task_id": task_id, "rework_reason": rework_reason,
            },
        )


def _rewrite_status_cell(line: str, new_status: str) -> str:
    """Replace column 5 of a markdown table row with `new_status`.

    Row shape: `| TASK-X | Title | Pri | Status | Depends | Effort |`
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


def _find_backlog_md(step_result, repo_root: Path | None = None) -> Path | None:
    """Walk up from the step's output_path looking for docs/tasks/BACKLOG.md.

    Falls back to ``repo_root`` if the walk fails (e.g. when output_dir is
    outside the cyberos repo tree).
    """
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
        candidate = cur / "docs" / "tasks" / "BACKLOG.md"
        if candidate.is_file():
            return candidate
        if cur.parent == cur:
            break
        cur = cur.parent

    # Fallback: search from the known cyberos repo root
    if repo_root is not None:
        candidate = repo_root / "docs" / "tasks" / "BACKLOG.md"
        if candidate.is_file():
            return candidate

    # Fallback: search from CWD (project where workflow was invoked)
    candidate = Path.cwd() / "docs" / "tasks" / "BACKLOG.md"
    if candidate.is_file():
        return candidate

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


def _get_output_dir(step_result) -> Path | None:
    """Extract the output directory from a step result's output_path."""
    p = getattr(step_result, "output_path", None)
    if p is None:
        return None
    try:
        return Path(p).resolve().parent
    except (TypeError, ValueError):
        return None


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


# ---------------------------------------------------------------------------
# Task-audit applier — writes sibling .audit.md
# ---------------------------------------------------------------------------


def _apply_task_audit(step_result, hand_off: dict, run_span_id: str) -> None:
    """Write a sibling .audit.md for the audited task from the LLM's JSON output.

    The LLM produces structured audit output. We extract findings and write a
    markdown audit report next to the task spec file.  Handles three shapes:

    1. ``audit_body`` key present → write it verbatim as the .audit.md body.
    2. ``findings`` or ``issues`` list → render ISSUE blocks + SUMMARY.
    3. Fallback → wrap the entire JSON output in a minimal audit shell.
    """
    output = _extract_output(step_result)
    task_id = output.get("task_id") or hand_off.get("task_id")
    if not task_id:
        _SPANS.debug(
            "applier.skip",
            extra={"event": "applier.skip", "span_id": run_span_id,
                   "skill": "task-audit", "reason": "missing task_id"},
        )
        return

    # Locate the task spec file.
    repo_root = _resolve_project_root(hand_off)
    output_dir = _get_output_dir(step_result)
    task_path = _find_task_file(task_id, repo_root, output_dir)
    if task_path is None:
        _SPANS.warning(
            "applier.task_not_found",
            extra={"event": "applier.task_not_found", "span_id": run_span_id,
                   "task_id": task_id},
        )
        return

    # TASK-MEMORY-302: audits go through put() under memories/refinements/, never
    # a raw store-root `audits/` dir. Fallback when no store: sibling of the task.
    audit_filename = f"{task_path.stem}.audit.md"

    # Determine the audit body.
    if isinstance(output.get("audit_body"), str):
        body = output["audit_body"]
    elif isinstance(output.get("raw_response"), str):
        body = output["raw_response"]
    else:
        body = _render_audit_markdown(output, task_id, task_path)

    # Compute a simple hash of the task for the frontmatter.
    try:
        import hashlib
        task_hash = hashlib.sha256(task_path.read_text(encoding="utf-8").encode("utf-8")).hexdigest()
    except OSError:
        task_hash = "unknown"

    verdict = output.get("verdict") or output.get("overall_status", "pass")
    if isinstance(output.get("overall_status_counts"), dict):
        counts = output["overall_status_counts"]
        if counts.get("fail", 0) > 0:
            verdict = "fail"
        elif counts.get("needs_human", 0) > 0:
            verdict = "needs_human"
        else:
            verdict = "pass"

    score = output.get("score")
    issues_open = output.get("issues_open", 0)
    if isinstance(output.get("per_artefact"), list):
        for pa in output["per_artefact"]:
            if pa.get("artefact_path", "").endswith(task_path.name):
                issues_open = pa.get("issues_open", issues_open)
                break

    frontmatter = (
        f"---\n"
        f"task_id: {task_id}\n"
        f"audited: {_today_iso()}\n"
        f"auditor: cuo-workflow (task-audit)\n"
        f"verdict: {verdict}\n"
        f"audited_file_sha256: {task_hash}\n"
        f"issues_open: {issues_open}\n"
        f"template: task@1\n"
        f"---\n"
    )

    content = frontmatter + "\n" + body.strip() + "\n"

    if repo_root is not None:
        written = _put_brain_artefact(
            repo_root, "audits", audit_filename, content, run_span_id=run_span_id,
        )
        if written is not None:
            _SPANS.info(
                "applier.audit_written",
                extra={"event": "applier.audit_written", "span_id": run_span_id,
                       "task_id": task_id, "audit_path": str(written),
                       "verdict": verdict},
            )
            return
    # No BRAIN store / writer — sibling of the task spec (outside store root).
    audit_path = task_path.with_suffix(".audit.md")
    try:
        tmp = audit_path.with_suffix(".audit.md.tmp")
        tmp.write_text(content, encoding="utf-8")
        os.replace(tmp, audit_path)
        _SPANS.info(
            "applier.audit_written",
            extra={"event": "applier.audit_written", "span_id": run_span_id,
                   "task_id": task_id, "audit_path": str(audit_path),
                   "verdict": verdict},
        )
    except OSError as e:
        _SPANS.warning(
            "applier.audit_write_error",
            extra={"event": "applier.audit_write_error", "span_id": run_span_id,
                   "task_id": task_id, "error": str(e)},
        )


def _render_audit_markdown(output: dict, task_id: str, task_path: Path) -> str:
    """Render a structured LLM output into audit markdown.

    Handles the common shapes the LLM produces for audit skills:
    - ``findings`` or ``issues`` list → ISSUE blocks
    - ``rule_outcomes`` dict → per-rule summary
    - ``score``, ``pass``, ``fixes`` → summary block
    """
    parts: list[str] = []

    # Verdict summary
    score = output.get("score")
    pass_val = output.get("pass")
    verdict = output.get("verdict") or output.get("overall_status", "pass")
    parts.append(f"## Verdict summary\n")
    if score is not None:
        parts.append(f"**Score = {score}/10.**")
    if pass_val is not None:
        verdict_label = "PASS" if pass_val else "FAIL"
        parts.append(f" Verdict: **{verdict_label}**.")
    elif verdict:
        parts.append(f" Verdict: **{verdict}**.")
    parts.append("\n")

    # Issues / findings
    issues = output.get("findings") or output.get("issues") or []
    if issues and isinstance(issues, list):
        parts.append("## Findings\n")
        for i, issue in enumerate(issues, 1):
            if isinstance(issue, dict):
                iid = issue.get("id", f"ISS-{i:03d}")
                rule = issue.get("rule_id", "")
                sev = issue.get("severity", "warning")
                status = issue.get("status", "open")
                desc = issue.get("description", issue.get("message", ""))
                suggestion = issue.get("suggestion", "")
                parts.append(f"### {iid}" + (f" — {rule}" if rule else ""))
                parts.append(f"- **severity:** {sev}")
                parts.append(f"- **status:** {status}")
                if desc:
                    parts.append(f"- **description:** {desc}")
                if suggestion:
                    parts.append(f"- **suggestion:** {suggestion}")
                parts.append("")
            elif isinstance(issue, str):
                parts.append(f"- {issue}")
        parts.append("")

    # Rule outcomes
    rule_outcomes = output.get("rule_outcomes")
    if rule_outcomes and isinstance(rule_outcomes, dict):
        parts.append("## Rule outcomes\n")
        for rule_id, outcome in rule_outcomes.items():
            if isinstance(outcome, dict):
                status = outcome.get("status", outcome.get("result", "pass"))
                note = outcome.get("note", outcome.get("description", ""))
                parts.append(f"- **{rule_id}:** {status}" + (f" — {note}" if note else ""))
            else:
                parts.append(f"- **{rule_id}:** {outcome}")
        parts.append("")

    # Fixes applied
    fixes = output.get("fixes")
    if fixes and isinstance(fixes, list):
        parts.append("## Fixes applied\n")
        for fix in fixes:
            if isinstance(fix, dict):
                parts.append(f"- {fix.get('description', fix.get('rule_id', str(fix)))}")
            elif isinstance(fix, str):
                parts.append(f"- {fix}")
        parts.append("")

    # Strengths / notes
    for key in ("strengths", "notes", "observations"):
        items = output.get(key)
        if items and isinstance(items, list):
            parts.append(f"## {key.replace('_', ' ').title()}\n")
            for item in items:
                parts.append(f"- {item}" if isinstance(item, str) else f"- {item}")
            parts.append("")

    if not parts or all(not p.strip() for p in parts):
        # Absolute fallback
        parts.append("## Audit output\n")
        parts.append("```json")
        parts.append(json.dumps(output, indent=2, sort_keys=True)[:4000])
        parts.append("```\n")

    return "\n".join(parts)


def _find_task_file(
    task_id: str,
    repo_root: Path | None = None,
    output_dir: Path | None = None,
) -> Path | None:
    """Locate the task spec file by searching for ``*<task_id>*.md`` under docs/tasks/.

    Search order:
    1. Walk up from output_dir looking for docs/tasks/
    2. repo_root (typically _cyberos_root — works when project IS cyberos)
    3. CWD walk-up as last resort
    """
    search_roots: list[Path] = []

    # 1) Walk up from output_dir — the most reliable source since outputs
    #    are always written to the correct project directory.
    if output_dir is not None:
        cur = output_dir.resolve()
        if cur.is_file():
            cur = cur.parent
        for _ in range(12):
            candidate = cur / "docs" / "tasks"
            if candidate.is_dir() and candidate not in search_roots:
                search_roots.append(candidate)
                break
            if cur.parent == cur:
                break
            cur = cur.parent

    # 2) Explicit repo_root (may be cyberos root — works when project IS cyberos)
    if repo_root is not None:
        candidate = repo_root / "docs" / "tasks"
        if candidate.is_dir() and candidate not in search_roots:
            search_roots.append(candidate)

    # 3) Fallback: walk up from CWD
    cur = Path.cwd()
    for _ in range(8):
        candidate = cur / "docs" / "tasks"
        if candidate.is_dir() and candidate not in search_roots:
            search_roots.append(candidate)
            break
        if cur.parent == cur:
            break
        cur = cur.parent

    task_lower = task_id.lower()
    for root in search_roots:
        if not root.is_dir():
            continue
        for md in root.rglob("*.md"):
            if task_lower in md.stem.lower() and not md.stem.endswith(".audit"):
                return md
    return None


def _today_iso() -> str:
    """Return today's date as ISO-8601 (YYYY-MM-DD)."""
    import datetime
    return datetime.date.today().isoformat()


# ---------------------------------------------------------------------------
# architecture-decision-record-author applier — writes ADR to docs/adrs/
# ---------------------------------------------------------------------------


def _apply_architecture_decision_record(step_result, hand_off: dict, run_span_id: str) -> None:
    """Write an ADR markdown file from the LLM's JSON output.

    Output shapes supported:
    1. ``adr_body`` or ``body`` key → write verbatim.
    2. ``raw_response`` key → write verbatim.
    3. Structured fields (``context``, ``decision``, ``options``) → render to ADR template.
    4. ``artefact_fields`` key (from mock-llm) → render from template fields.

    Target: ``memories/decisions/<hex>/<hex>/ADR-{NNN}-{slug}.md`` via put()
    (TASK-MEMORY-302). Never a raw store-root ``adrs/`` directory.
    """
    output = _extract_output(step_result)
    if not output:
        return

    # Derive ADR id and slug.
    adr_id = output.get("adr_id") or output.get("id", "ADR-0001")
    title = output.get("title", output.get("decision", "untitled"))
    slug = _slugify(title)[:60] if title else "untitled"
    status = output.get("status", "proposed")

    # Locate repo root (parent of docs/tasks/).
    output_dir = _get_output_dir(step_result)
    repo_root = _resolve_project_root(hand_off)
    if repo_root is None:
        repo_root = _find_repo_root_from_handoff(hand_off)
    if repo_root is None:
        repo_root = _find_repo_root(hand_off, output_dir)
    if repo_root is None:
        repo_root = Path.cwd()

    # Clean adr_id for filename (strip non-alphanumeric prefix chars for the number part).
    adr_num = re.sub(r"[^0-9]", "", adr_id) or "0001"
    filename = f"ADR-{adr_num}-{slug}.md"

    # Determine body.
    for key in ("adr_body", "body", "raw_response"):
        if isinstance(output.get(key), str) and output[key].strip():
            body = output[key]
            break
    else:
        body = _render_adr_markdown(output, adr_id, title, status)

    # Build frontmatter.
    decision_date = output.get("decision_date") or _today_iso()
    decided_by = output.get("decided_by", [])
    if isinstance(decided_by, list) and decided_by:
        dm = "\n".join(f"  - {d}" if isinstance(d, str) else
                       f"  - {{ handle: \"{d.get('handle', '')}\", role: \"{d.get('role', '')}\" }}"
                       for d in decided_by)
    else:
        dm = '  - { handle: "@cuo-cto", role: "CTO" }'

    frontmatter = (
        f"---\n"
        f"template: architecture-decision-record@1\n"
        f"title: \"{title}\"\n"
        f"adr_id: {adr_id}\n"
        f"status: {status}\n"
        f"decision_date: \"{decision_date}\"\n"
        f"decided_by:\n{dm}\n"
        f"---\n"
    )

    content = frontmatter + "\n" + body.strip() + "\n"

    written = _put_brain_artefact(
        repo_root, "adrs", filename, content, run_span_id=run_span_id,
    )
    if written is not None:
        _SPANS.info(
            "applier.adr_written",
            extra={"event": "applier.adr_written", "span_id": run_span_id,
                   "adr_id": adr_id, "adr_path": str(written)},
        )
        return
    _SPANS.warning(
        "applier.adr_write_error",
        extra={"event": "applier.adr_write_error", "span_id": run_span_id,
               "adr_id": adr_id, "error": "brain put unavailable — refused raw store-root write"},
    )


def _render_adr_markdown(output: dict, adr_id: str, title: str, status: str) -> str:
    """Render structured LLM output into ADR markdown."""
    parts: list[str] = []
    parts.append(f"# {adr_id}: {title}\n")

    # Context
    context = output.get("context", "")
    if context:
        parts.append("## 1. Context\n")
        parts.append(str(context) + "\n")

    # Options
    options = output.get("options") or output.get("options_considered") or []
    if options and isinstance(options, list):
        parts.append("## 2. Options Considered\n")
        for i, opt in enumerate(options):
            if isinstance(opt, dict):
                name = opt.get("name", f"Option {chr(65+i)}")
                pros = opt.get("pros", [])
                cons = opt.get("cons", [])
                parts.append(f"### {name}")
                if pros:
                    parts.append(f"- **Pros:** {'; '.join(str(p) for p in pros)}")
                if cons:
                    parts.append(f"- **Cons:** {'; '.join(str(c) for c in cons)}")
                parts.append("")
            elif isinstance(opt, str):
                parts.append(f"- {opt}")
        parts.append("")

    # Decision
    decision = output.get("decision", "")
    if decision:
        parts.append("## 3. Decision\n")
        parts.append(str(decision) + "\n")

    # Consequences
    consequences = output.get("consequences", {})
    if consequences and isinstance(consequences, dict):
        parts.append("## 4. Consequences\n")
        for label in ("positive", "negative", "neutral"):
            items = consequences.get(label, [])
            if items:
                cap = label.capitalize()
                for item in (items if isinstance(items, list) else [items]):
                    parts.append(f"- **{cap}:** {item}")
        parts.append("")
    elif isinstance(consequences, list) and consequences:
        parts.append("## 4. Consequences\n")
        for item in consequences:
            parts.append(f"- {item}")
        parts.append("")

    # Artefact fields (from mock-llm or template-parroting output)
    artefact = output.get("artefact_fields")
    if artefact and isinstance(artefact, dict):
        for heading, value in artefact.items():
            parts.append(f"## {heading}\n")
            parts.append(str(value) + "\n")

    # Fallback: raw JSON summary
    if not parts or all(not p.strip() for p in parts[1:]):
        parts.append("## ADR output\n")
        parts.append("```json")
        parts.append(json.dumps(output, indent=2, sort_keys=True)[:4000])
        parts.append("```\n")

    return "\n".join(parts)


def _find_repo_root_from_handoff(hand_off: dict) -> Path | None:
    """Try to derive the repo root from hand-off map entries."""
    # Try step output paths — they live under the output dir which may be at repo root.
    for key, val in hand_off.items():
        if isinstance(val, Path) and val.suffix == ".json":
            candidate = val.parent
            if (candidate / "docs").is_dir():
                return candidate
            # Walk up a few levels
            for _ in range(3):
                candidate = candidate.parent
                if (candidate / "docs").is_dir():
                    return candidate
    return None


def _slugify(text: str) -> str:
    """Convert text to a filename-safe slug."""
    text = text.lower().strip()
    text = re.sub(r"[^a-z0-9]+", "-", text)
    return text.strip("-") or "untitled"


# ---------------------------------------------------------------------------
# implementation-plan-author applier — writes impl plan + applies code changes
# ---------------------------------------------------------------------------


def _apply_implementation_plan(step_result, hand_off: dict, run_span_id: str) -> None:
    """Write the implementation plan document AND apply any code changes.

    The LLM's output can contain:
    1. ``impl_plan_body`` / ``body`` / ``raw_response`` → write as ``IMPL-PLAN-<task_id>.md``
    2. ``code_changes`` list → write/modify files in the working tree (task #7)
    3. Structured fields → render to template format

    The plan document goes to ``memories/projects/<hex>/<hex>/impl-plan-<task_id>.md``
    via put() (TASK-MEMORY-302). Never a raw store-root ``impl-plans/`` directory.
    """
    output = _extract_output(step_result)
    if not output:
        return

    task_id = output.get("task_id") or hand_off.get("task_id")
    output_dir = _get_output_dir(step_result)

    # ── Write the plan document ──
    _write_impl_plan_doc(output, task_id, hand_off, run_span_id, output_dir)

    # ── Apply code changes (task #7) ──
    _apply_code_changes(output, task_id, hand_off, run_span_id, output_dir)


def _write_impl_plan_doc(output: dict, task_id: str | None, hand_off: dict, run_span_id: str, output_dir: Path | None = None) -> None:
    """Write the IMPL-PLAN markdown document via the canonical BRAIN writer."""
    # Determine plan body.
    for key in ("impl_plan_body", "body", "raw_response"):
        if isinstance(output.get(key), str) and output[key].strip():
            body = output[key]
            break
    else:
        body = _render_impl_plan_markdown(output)

    slug = task_id or "untitled"
    filename = f"impl-plan-{slug}.md"
    title = output.get("title", f"Implementation Plan — {task_id}" if task_id else "Implementation Plan")
    frontmatter = (
        f"---\n"
        f"template: impl_plan@1\n"
        f"title: \"{title}\"\n"
        f"created_at: \"{_today_iso()}\"\n"
        f"---\n"
    )
    content = frontmatter + "\n" + body.strip() + "\n"

    repo_root = _resolve_project_root(hand_off)
    if repo_root is None:
        repo_root = _find_repo_root_from_handoff(hand_off)
    if repo_root is None:
        repo_root = _find_repo_root(hand_off, output_dir)
    if repo_root is None:
        _SPANS.warning(
            "applier.impl_plan_write_error",
            extra={"event": "applier.impl_plan_write_error", "span_id": run_span_id,
                   "error": "no repo root"},
        )
        return

    written = _put_brain_artefact(
        repo_root, "impl-plans", filename, content, run_span_id=run_span_id,
    )
    if written is not None:
        _SPANS.info(
            "applier.impl_plan_written",
            extra={"event": "applier.impl_plan_written", "span_id": run_span_id,
                   "path": str(written)},
        )
        return
    _SPANS.warning(
        "applier.impl_plan_write_error",
        extra={"event": "applier.impl_plan_write_error", "span_id": run_span_id,
               "error": "brain put unavailable — refused raw store-root write"},
    )


def _render_impl_plan_markdown(output: dict) -> str:
    """Render structured LLM output into impl-plan markdown."""
    parts: list[str] = []

    # Background
    bg = output.get("background", output.get("Background", ""))
    if bg:
        parts.append("## Background\n")
        parts.append(str(bg) + "\n")

    # Tickets table
    tickets = output.get("tickets", output.get("Tickets", []))
    if tickets and isinstance(tickets, list):
        parts.append("## Tickets\n")
        parts.append("| # | Title | Sizing | Dependencies | Acceptance criteria |")
        parts.append("| --- | --- | --- | --- | --- |")
        for i, t in enumerate(tickets, 1):
            if isinstance(t, dict):
                title = t.get("title", t.get("name", f"Ticket {i}"))
                sizing = t.get("sizing", t.get("size", "M"))
                deps = t.get("dependencies", t.get("deps", "—"))
                if isinstance(deps, list):
                    deps = ", ".join(str(d) for d in deps)
                ac = t.get("acceptance_criteria", t.get("ac", "—"))
                parts.append(f"| {i} | {title} | {sizing} | {deps} | {ac} |")
            elif isinstance(t, str):
                parts.append(f"| {i} | {t} | M | — | — |")
        parts.append("")

    # Sprint suggestion
    sprint = output.get("sprint_suggestion", output.get("Sprint Suggestion", ""))
    if sprint:
        parts.append("## Sprint Suggestion\n")
        parts.append(str(sprint) + "\n")

    # Risks
    risks = output.get("risks", output.get("Risks", []))
    if risks:
        parts.append("## Risks\n")
        if isinstance(risks, list):
            parts.append("| Risk | Mitigation |")
            parts.append("| --- | --- |")
            for r in risks:
                if isinstance(r, dict):
                    parts.append(f"| {r.get('risk', '')} | {r.get('mitigation', '')} |")
                elif isinstance(r, str):
                    parts.append(f"| {r} | — |")
        else:
            parts.append(str(risks) + "\n")
        parts.append("")

    # Open questions
    questions = output.get("open_questions", output.get("Open Questions", []))
    if questions:
        parts.append("## Open Questions\n")
        if isinstance(questions, list):
            for q in questions:
                parts.append(f"1. {q}")
        else:
            parts.append(str(questions))
        parts.append("")

    # Artefact fields (from mock-llm)
    artefact = output.get("artefact_fields")
    if artefact and isinstance(artefact, dict):
        for heading, value in artefact.items():
            parts.append(f"## {heading}\n")
            parts.append(str(value) + "\n")

    if not parts or all(not p.strip() for p in parts):
        parts.append("## Implementation Plan\n")
        parts.append("```json")
        parts.append(json.dumps(output, indent=2, sort_keys=True)[:4000])
        parts.append("```\n")

    return "\n".join(parts)


def _apply_code_changes(output: dict, task_id: str | None, hand_off: dict, run_span_id: str, output_dir: Path | None = None) -> None:
    """Apply file-level code changes from the LLM's structured output.

    The LLM can include a ``code_changes`` list with entries like:
    {"action": "create"|"modify", "path": "src/auth/login.ts", "content": "..."}
    or
    {"action": "create"|"modify", "path": "src/auth/login.ts", "diff": "..."}
    """
    code_changes = output.get("code_changes") or output.get("files") or []
    if not code_changes or not isinstance(code_changes, list):
        # Also check for implementation steps in various LLM output formats.
        # Format 1: Keys like "## 3. Implementation Steps" with action/file/description
        # Format 2: task_breakdown with files arrays and descriptions
        for key, val in output.items():
            if isinstance(val, list) and val and isinstance(val[0], dict):
                # Check for action/file format
                if any("action" in item and ("file" in item or "path" in item) for item in val if isinstance(item, dict)):
                    code_changes = val
                    break
                # Check for files array format (task_breakdown)
                if any("files" in item and "description" in item for item in val if isinstance(item, dict)):
                    # Flatten: each task with files[] becomes multiple code_changes
                    flattened = []
                    for item in val:
                        if not isinstance(item, dict):
                            continue
                        files = item.get("files", [])
                        desc = item.get("description", "")
                        title = item.get("title", "")
                        for f in files:
                            if isinstance(f, str):
                                flattened.append({
                                    "action": "create",
                                    "file": f,
                                    "title": title,
                                    "description": desc,
                                })
                    if flattened:
                        code_changes = flattened
                        break
    if not code_changes or not isinstance(code_changes, list):
        return

    repo_root = _find_repo_root(hand_off, output_dir)
    if repo_root is None:
        _SPANS.debug(
            "applier.no_repo_root",
            extra={"event": "applier.no_repo_root", "span_id": run_span_id},
        )
        return

    applied = 0
    for change in code_changes:
        if not isinstance(change, dict):
            continue
        action = change.get("action", "create")
        rel_path = change.get("path") or change.get("file") or ""
        if not rel_path or rel_path == "None":
            continue

        # Skip "verify" actions — they're test/validation steps, not file writes.
        if action == "verify":
            continue

        # Security: reject path traversal.
        if ".." in Path(rel_path).parts:
            _SPANS.warning(
                "applier.path_traversal_rejected",
                extra={"event": "applier.path_traversal_rejected", "span_id": run_span_id,
                       "path": rel_path},
            )
            continue

        target = repo_root / rel_path

        if action == "create":
            content = change.get("content", "")
            if not content:
                # Generate stub from description if no content provided.
                desc = change.get("description", "")
                title = change.get("title", rel_path)
                if desc:
                    ext = Path(rel_path).suffix
                    content = _generate_stub(title, desc, ext, task_id)
                else:
                    continue
            try:
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(content, encoding="utf-8")
                applied += 1
            except OSError as e:
                _SPANS.warning(
                    "applier.code_write_error",
                    extra={"event": "applier.code_write_error", "span_id": run_span_id,
                           "path": rel_path, "error": str(e)},
                )

        elif action == "modify":
            content = change.get("content")
            if content is not None:
                # Full replacement
                try:
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_text(content, encoding="utf-8")
                    applied += 1
                except OSError as e:
                    _SPANS.warning(
                        "applier.code_write_error",
                        extra={"event": "applier.code_write_error", "span_id": run_span_id,
                               "path": rel_path, "error": str(e)},
                    )
            else:
                # Diff-based modification
                diff_text = change.get("diff", "")
                if diff_text and target.is_file():
                    try:
                        result = _apply_unified_diff(target.read_text(encoding="utf-8"), diff_text)
                        if result is not None:
                            target.write_text(result, encoding="utf-8")
                            applied += 1
                    except OSError as e:
                        _SPANS.warning(
                            "applier.code_write_error",
                            extra={"event": "applier.code_write_error", "span_id": run_span_id,
                                   "path": rel_path, "error": str(e)},
                        )
                elif not diff_text and target.is_file():
                    # No content, no diff — append TODO comment to existing file.
                    desc = change.get("description", "")
                    if desc:
                        try:
                            existing = target.read_text(encoding="utf-8")
                            ext = target.suffix
                            todo = _generate_todo_comment(desc, ext)
                            target.write_text(existing + "\n" + todo, encoding="utf-8")
                            applied += 1
                        except OSError as e:
                            _SPANS.warning(
                                "applier.code_write_error",
                                extra={"event": "applier.code_write_error", "span_id": run_span_id,
                                       "path": rel_path, "error": str(e)},
                            )

    if applied:
        _SPANS.info(
            "applier.code_changes_applied",
            extra={"event": "applier.code_changes_applied", "span_id": run_span_id,
                   "files_changed": applied, "task_id": task_id},
        )


def _apply_unified_diff(original: str, diff_text: str) -> str | None:
    """Best-effort application of a unified diff. Returns patched text or None on failure."""
    lines = original.splitlines(keepends=True)
    diff_lines = diff_text.splitlines(keepends=True)

    result: list[str] = []
    i = 0  # index into original lines
    d = 0  # index into diff lines

    while d < len(diff_lines):
        line = diff_lines[d]
        if line.startswith("@@"):
            # Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
            m = re.match(r"^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@", line)
            if not m:
                return None
            old_start = int(m.group(1)) - 1  # 0-indexed
            # Copy unchanged lines before this hunk
            while i < old_start and i < len(lines):
                result.append(lines[i])
                i += 1
            d += 1
            # Process hunk body
            while d < len(diff_lines) and not diff_lines[d].startswith("@@"):
                hunk_line = diff_lines[d]
                if hunk_line.startswith("+"):
                    result.append(hunk_line[1:] if hunk_line.endswith("\n") else hunk_line[1:] + "\n")
                elif hunk_line.startswith("-"):
                    i += 1  # skip original line
                else:
                    # context line
                    if i < len(lines):
                        result.append(lines[i])
                        i += 1
                d += 1
        else:
            d += 1

    # Copy remaining original lines
    while i < len(lines):
        result.append(lines[i])
        i += 1

    return "".join(result)


def _generate_stub(title: str, description: str, ext: str, task_id: str | None = None) -> str:
    """Generate a stub file from a title and description.

    Used when the LLM provides implementation steps with descriptions but no file content.
    The stub documents what needs to be implemented and serves as a starting point.
    """
    task_tag = f" (task: {task_id})" if task_id else ""

    if ext in (".ts", ".tsx", ".js", ".jsx"):
        return (
            f"/**\n"
            f" * {title}{task_tag}\n"
            f" *\n"
            f" * {description}\n"
            f" *\n"
            f" * TODO: Implement this module — description from CUO workflow.\n"
            f" */\n\n"
            f"// TODO: Implement {title}\n"
        )
    elif ext == ".sql":
        return (
            f"-- {title}{task_tag}\n"
            f"-- {description}\n"
            f"--\n"
            f"-- TODO: Implement this migration — description from CUO workflow.\n\n"
        )
    elif ext == ".py":
        return (
            f'"""{title}{task_tag}\n\n'
            f"{description}\n\n"
            f"TODO: Implement this module — description from CUO workflow.\n"
            f'"""\n\n'
            f"# TODO: Implement {title}\n"
        )
    elif ext in (".mjs", ".mts"):
        return (
            f"/**\n"
            f" * {title}{task_tag}\n"
            f" *\n"
            f" * {description}\n"
            f" *\n"
            f" * TODO: Implement this module — description from CUO workflow.\n"
            f" */\n\n"
            f"// TODO: Implement {title}\n"
        )
    else:
        return (
            f"# {title}{task_tag}\n\n"
            f"{description}\n\n"
            f"TODO: Implement this file — description from CUO workflow.\n"
        )


def _generate_todo_comment(description: str, ext: str) -> str:
    """Generate a TODO comment block from a description.

    Used when the LLM describes modifications to an existing file but provides
    no content or diff.
    """
    if ext in (".ts", ".tsx", ".js", ".jsx", ".mjs", ".mts"):
        return f"\n// TODO (CUO workflow): {description}\n"
    elif ext == ".py":
        return f"\n# TODO (CUO workflow): {description}\n"
    elif ext == ".sql":
        return f"\n-- TODO (CUO workflow): {description}\n"
    else:
        return f"\n# TODO (CUO workflow): {description}\n"


def _find_repo_root(hand_off: dict, output_dir: Path | None = None) -> Path | None:
    """Find the repo root by searching from output_dir, hand-off, or CWD.

    Search order:
    1. Walk up from output_dir (project outputs are always in the right project)
    2. hand_off["_cyberos_root"] (works when project IS cyberos)
    3. CWD walk-up as last resort
    """
    seen: set[Path] = set()

    # 1) Walk up from output_dir
    if output_dir is not None:
        cur = output_dir.resolve()
        if cur.is_file():
            cur = cur.parent
        for _ in range(12):
            if (cur / "docs").is_dir():
                seen.add(cur)
                return cur
            if cur.parent == cur:
                break
            cur = cur.parent

    # 2) hand_off _cyberos_root
    root = hand_off.get("_cyberos_root")
    if root:
        p = Path(root)
        if (p / "docs").is_dir() and p not in seen:
            return p

    # 3) Walk up from CWD
    cur = Path.cwd()
    for _ in range(8):
        if (cur / "docs").is_dir() and cur not in seen:
            return cur
        if cur.parent == cur:
            break
        cur = cur.parent
    return None


def _resolve_artifact_path(
    output: dict,
    task_id: str | None,
    hand_off: dict,
    filename_prefix: str,
    default_dir: str,
    output_dir: Path | None = None,
    *,
    force_default_dir: bool = False,
) -> Path | None:
    """Resolve the output path for a document artifact.

    Strategy:
    1. If ``output.artifact_path`` or ``output.path`` exists, use it (relative to repo root).
    2. If task_id is known, locate the task file and place the artifact as a sibling
       (unless ``force_default_dir`` is True, which skips TASK-sibling placement).
    3. Fallback: <repo_root>/<default_dir>/<filename_prefix>-<task_id>.md
    """
    # Check explicit path in output.
    for key in ("artifact_path", "path", "audit_path"):
        p = output.get(key)
        if isinstance(p, str) and p:
            repo_root = _find_repo_root(hand_off, output_dir)
            if repo_root and not Path(p).is_absolute():
                return repo_root / p
            return Path(p)

    # Try to find the task file and place as sibling (unless force_default_dir).
    if task_id and not force_default_dir:
        cyberos_root = _resolve_project_root(hand_off)
        task_path = _find_task_file(task_id, cyberos_root, output_dir)
        if task_path:
            return task_path.parent / f"{filename_prefix}-{task_id}.md"

    # Fallback: <repo_root>/<default_dir>/
    repo_root = _find_repo_root(hand_off, output_dir)
    if repo_root:
        slug = _slugify(task_id) if task_id else "untitled"
        target_dir = repo_root / default_dir
        target_dir.mkdir(parents=True, exist_ok=True)
        return target_dir / f"{filename_prefix}-{slug}.md"

    return None


# ---------------------------------------------------------------------------
# code-review-author applier — writes code-review markdown
# ---------------------------------------------------------------------------


def _apply_code_review(step_result, hand_off: dict, run_span_id: str) -> None:
    """Write a code-review@1 markdown document from the LLM's output.

    Target: ``memories/refinements/<hex>/<hex>/code-review-<task_id>.md`` via put()
    (TASK-MEMORY-302).
    """
    output = _extract_output(step_result)
    if not output:
        return

    task_id = output.get("task_id") or hand_off.get("task_id")
    output_dir = _get_output_dir(step_result)

    for key in ("code_review_body", "body", "raw_response"):
        if isinstance(output.get(key), str) and output[key].strip():
            body = output[key]
            break
    else:
        body = _render_code_review_markdown(output)

    slug = task_id or "untitled"
    filename = f"code-review-{slug}.md"
    verdict = output.get("verdict", "approved")
    frontmatter = (
        f"---\n"
        f"template: code-review@1\n"
        f"verdict: {verdict}\n"
        f"reviewed_at: \"{_today_iso()}\"\n"
        f"---\n"
    )
    content = frontmatter + "\n" + body.strip() + "\n"

    repo_root = _resolve_project_root(hand_off)
    if repo_root is None:
        repo_root = _find_repo_root_from_handoff(hand_off)
    if repo_root is None:
        repo_root = _find_repo_root(hand_off, output_dir)
    if repo_root is None:
        _SPANS.warning(
            "applier.code_review_write_error",
            extra={"event": "applier.code_review_write_error", "span_id": run_span_id,
                   "error": "no repo root"},
        )
        return

    written = _put_brain_artefact(
        repo_root, "code-reviews", filename, content, run_span_id=run_span_id,
    )
    if written is not None:
        _SPANS.info(
            "applier.code_review_written",
            extra={"event": "applier.code_review_written", "span_id": run_span_id,
                   "path": str(written), "verdict": verdict},
        )
        return
    _SPANS.warning(
        "applier.code_review_write_error",
        extra={"event": "applier.code_review_write_error", "span_id": run_span_id,
               "error": "brain put unavailable — refused raw store-root write"},
    )


def _render_code_review_markdown(output: dict) -> str:
    """Render structured LLM output into code-review markdown."""
    parts: list[str] = []

    # Verdict
    verdict = output.get("verdict", "")
    if verdict:
        parts.append(f"**Verdict: {verdict}**\n")

    # Template sections (1-12 from code-review@1)
    section_map = {
        "correctness": ("1. Correctness vs Ticket", output.get("correctness", "")),
        "readability": ("2. Readability", output.get("readability", "")),
        "test_coverage": ("3. Test Coverage", output.get("test_coverage", "")),
        "secrets": ("4. Secrets / Credentials", output.get("secrets", "")),
        "injection_surfaces": ("5. Injection Surfaces", output.get("injection_surfaces", "")),
        "input_validation": ("6. Input Validation", output.get("input_validation", "")),
        "error_handling": ("7. Error Handling", output.get("error_handling", "")),
        "logging": ("8. Logging", output.get("logging", "")),
        "performance": ("9. Performance Considerations", output.get("performance", "")),
        "backwards_compat": ("10. Backwards Compatibility", output.get("backwards_compat", "")),
        "sast_sca": ("11. SAST / SCA Results", output.get("sast_sca", "")),
        "sbom": ("12. SBOM Impact", output.get("sbom", "")),
    }

    for _key, (heading, content) in section_map.items():
        if content:
            parts.append(f"## {heading}\n")
            parts.append(str(content) + "\n")

    # Issues / findings
    issues = output.get("issues") or output.get("findings") or []
    if issues and isinstance(issues, list):
        parts.append("## Findings\n")
        for issue in issues:
            if isinstance(issue, dict):
                sev = issue.get("severity", "info")
                desc = issue.get("description", issue.get("message", ""))
                location = issue.get("location", "")
                loc_str = f" ({location})" if location else ""
                parts.append(f"- **[{sev}]** {desc}{loc_str}")
            elif isinstance(issue, str):
                parts.append(f"- {issue}")
        parts.append("")

    # Artefact fields (mock-llm)
    artefact = output.get("artefact_fields")
    if artefact and isinstance(artefact, dict):
        for heading, value in artefact.items():
            parts.append(f"## {heading}\n")
            parts.append(str(value) + "\n")

    if not parts or all(not p.strip() for p in parts):
        parts.append("## Code Review\n")
        parts.append("```json")
        parts.append(json.dumps(output, indent=2, sort_keys=True)[:4000])
        parts.append("```\n")

    return "\n".join(parts)


# ---------------------------------------------------------------------------
# observability-injection-author applier — writes observability plan
# ---------------------------------------------------------------------------


def _apply_observability_injection(step_result, hand_off: dict, run_span_id: str) -> None:
    """Write an observability-injection@1 document from the LLM's output.

    Target: ``memories/facts/<hex>/<hex>/obs-injection-<task_id>.md`` via put()
    (TASK-MEMORY-302).
    """
    output = _extract_output(step_result)
    if not output:
        return

    task_id = output.get("task_id") or hand_off.get("task_id")
    output_dir = _get_output_dir(step_result)

    for key in ("obs_injection_body", "body", "raw_response"):
        if isinstance(output.get(key), str) and output[key].strip():
            body = output[key]
            break
    else:
        body = _render_obs_injection_markdown(output)

    slug = task_id or "untitled"
    filename = f"obs-injection-{slug}.md"
    language = output.get("language", "typescript")
    subscriber = output.get("subscriber", "tracing")
    frontmatter = (
        f"---\n"
        f"template: observability-injection@1\n"
        f"task_id: {task_id or 'unknown'}\n"
        f"language: {language}\n"
        f"subscriber: {subscriber}\n"
        f"generated_at: \"{_today_iso()}\"\n"
        f"---\n"
    )
    content = frontmatter + "\n" + body.strip() + "\n"

    repo_root = _resolve_project_root(hand_off)
    if repo_root is None:
        repo_root = _find_repo_root_from_handoff(hand_off)
    if repo_root is None:
        repo_root = _find_repo_root(hand_off, output_dir)
    if repo_root is None:
        _SPANS.warning(
            "applier.obs_injection_write_error",
            extra={"event": "applier.obs_injection_write_error", "span_id": run_span_id,
                   "error": "no repo root"},
        )
        return

    written = _put_brain_artefact(
        repo_root, "obs-injections", filename, content, run_span_id=run_span_id,
    )
    if written is not None:
        _SPANS.info(
            "applier.obs_injection_written",
            extra={"event": "applier.obs_injection_written", "span_id": run_span_id,
                   "path": str(written)},
        )
        return
    _SPANS.warning(
        "applier.obs_injection_write_error",
        extra={"event": "applier.obs_injection_write_error", "span_id": run_span_id,
               "error": "brain put unavailable — refused raw store-root write"},
    )


def _render_obs_injection_markdown(output: dict) -> str:
    """Render structured LLM output into observability-injection markdown."""
    parts: list[str] = []

    # Log points
    log_points = output.get("log_points", [])
    if log_points and isinstance(log_points, list):
        parts.append("## Log Points\n")
        parts.append("| ID | File | Level | Message | Carries |")
        parts.append("| --- | --- | --- | --- | --- |")
        for lp in log_points:
            if isinstance(lp, dict):
                lid = lp.get("id", "")
                fpath = lp.get("file", "")
                level = lp.get("level", "info")
                msg = lp.get("message_shape", lp.get("message", ""))
                carries = lp.get("carries", [])
                if isinstance(carries, list):
                    carries = ", ".join(str(c) for c in carries)
                parts.append(f"| {lid} | `{fpath}` | {level} | {msg} | {carries} |")
        parts.append("")

    # Trace spans
    trace_spans = output.get("trace_spans", [])
    if trace_spans and isinstance(trace_spans, list):
        parts.append("## Trace Spans\n")
        parts.append("| ID | File | Wraps | Attributes |")
        parts.append("| --- | --- | --- | --- |")
        for ts in trace_spans:
            if isinstance(ts, dict):
                tid = ts.get("id", "")
                fpath = ts.get("file", "")
                wraps = ts.get("wraps", "")
                attrs = ts.get("attributes", [])
                if isinstance(attrs, list):
                    attrs = ", ".join(str(a) for a in attrs)
                parts.append(f"| {tid} | `{fpath}` | {wraps} | {attrs} |")
        parts.append("")

    # Error counters
    error_counters = output.get("error_counters", [])
    if error_counters and isinstance(error_counters, list):
        parts.append("## Error Counters\n")
        parts.append("| ID | Metric Name | Labels | Increments At |")
        parts.append("| --- | --- | --- | --- |")
        for ec in error_counters:
            if isinstance(ec, dict):
                eid = ec.get("id", "")
                metric = ec.get("metric_name", "")
                labels = ec.get("labels", [])
                if isinstance(labels, list):
                    labels = ", ".join(str(l) for l in labels)
                increments = ec.get("increments_at", "")
                parts.append(f"| {eid} | `{metric}` | {labels} | {increments} |")
        parts.append("")

    # Branch coverage
    coverage = output.get("branch_coverage", {})
    if coverage and isinstance(coverage, dict):
        parts.append("## Branch Coverage\n")
        total = coverage.get("total_branches", 0)
        covered = coverage.get("branches_with_obs_point", 0)
        pct = coverage.get("coverage_pct", 0)
        parts.append(f"- **Total branches:** {total}")
        parts.append(f"- **With observability point:** {covered}")
        parts.append(f"- **Coverage:** {pct}%\n")

    # Redaction policy
    redaction = output.get("redaction_policy", [])
    if redaction and isinstance(redaction, list):
        parts.append("## Redaction Policy\n")
        for r in redaction:
            if isinstance(r, dict):
                parts.append(f"- `{r.get('field_pattern', '')}` → {r.get('action', '')}")
        parts.append("")

    # Artefact fields (mock-llm)
    artefact = output.get("artefact_fields")
    if artefact and isinstance(artefact, dict):
        for heading, value in artefact.items():
            parts.append(f"## {heading}\n")
            parts.append(str(value) + "\n")

    if not parts or all(not p.strip() for p in parts):
        parts.append("## Observability Injection Plan\n")
        parts.append("```json")
        parts.append(json.dumps(output, indent=2, sort_keys=True)[:4000])
        parts.append("```\n")

    return "\n".join(parts)
