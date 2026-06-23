"""promptfoo python-assertion bridge -> harness multi-seed runner.

promptfoo calls ``get_assert(output, context)`` and expects a dict
``{pass: bool, score: float, reason: str}``. We ignore the model ``output`` and
instead run the named golden task across several seeds through the harness, so
pass@k math lives in one tested place. Optional path — the primary gate is
``make eval``.
"""

from __future__ import annotations

import sys
from pathlib import Path

# repo root = parents[3] (promptfoo/ -> stage1_measurement/ -> harness/ -> root)
_ROOT = Path(__file__).resolve().parents[3]
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from harness.stage1_measurement.goldenset import load_tasks  # noqa: E402
from harness.stage1_measurement.runner import evaluate  # noqa: E402

_TASKS_FILE = _ROOT / "harness" / "goldenset" / "tasks" / "example_tasks.yaml"
_SEEDS = list(range(5))


def get_assert(output, context):  # noqa: ARG001 - promptfoo contract
    task_id = (context or {}).get("vars", {}).get("task_id")
    tasks = [t for t in load_tasks(_TASKS_FILE) if t.id == task_id]
    if not tasks:
        # "pass" below is promptfoo's pass/fail verdict field, not a credential (B105 FP)
        return {"pass": False, "score": 0.0, "reason": f"unknown task_id {task_id!r}"}  # nosec B105
    report = evaluate(tasks, _SEEDS, base_dir=str(_ROOT))
    tr = report.tasks[0]
    return {
        "pass": tr.c > 0,
        "score": tr.pass_at_1,
        "reason": f"{task_id}: pass@1={tr.pass_at_1:.2f}, pass^{len(_SEEDS)}={tr.pass_hat(len(_SEEDS)):.2f}",
    }
