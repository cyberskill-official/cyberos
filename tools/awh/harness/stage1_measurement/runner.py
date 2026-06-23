"""Multi-seed eval runner: execute a golden set, score pass@k/pass^k, gate on regression.

Implements the KTH "On Randomness in Agentic Evals" discipline:
  * estimate pass@1 from MULTIPLE independent seeds (never a single run),
  * report optimistic pass@k AND pessimistic pass^k,
  * gate PRs on *regression vs a baseline*, not an absolute threshold.

The ``runner`` callable is injectable (defaults to subprocess.run) so the engine
is unit-testable without spawning processes.
"""

from __future__ import annotations

import contextlib
import os
import signal
import subprocess
import time
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path

from harness.common.stats import pass_at_k, pass_hat_k, wilson_interval
from harness.stage1_measurement.goldenset import GoldenTask

__all__ = [
    "TaskResult",
    "EvalReport",
    "GateResult",
    "run_task",
    "evaluate",
    "gate",
]


@dataclass
class TaskResult:
    task_id: str
    weight: float
    seeds: list[int]
    passes: list[bool]

    @property
    def n(self) -> int:
        return len(self.passes)

    @property
    def c(self) -> int:
        return sum(self.passes)

    @property
    def pass_at_1(self) -> float:
        return self.c / self.n if self.n else 0.0

    def pass_at(self, k: int) -> float:
        return pass_at_k(self.n, self.c, min(k, self.n))

    def pass_hat(self, k: int) -> float:
        return pass_hat_k(self.n, self.c, min(k, self.n))

    def wilson(self) -> tuple[float, float]:
        iv = wilson_interval(self.c, self.n)
        return (iv.low, iv.high)

    def to_dict(self, k: int) -> dict:
        lo, hi = self.wilson()
        return {
            "task_id": self.task_id,
            "weight": self.weight,
            "n": self.n,
            "c": self.c,
            "pass_at_1": round(self.pass_at_1, 6),
            f"pass_at_{k}": round(self.pass_at(k), 6),
            f"pass_hat_{k}": round(self.pass_hat(k), 6),
            "wilson95": [round(lo, 6), round(hi, 6)],
        }


@dataclass
class EvalReport:
    tasks: list[TaskResult]
    seeds: list[int]
    k: int
    label: str = ""
    created: float = field(default_factory=time.time)

    @property
    def macro_pass_at_1(self) -> float:
        return sum(t.pass_at_1 for t in self.tasks) / len(self.tasks) if self.tasks else 0.0

    @property
    def weighted_pass_at_1(self) -> float:
        wsum = sum(t.weight for t in self.tasks)
        if wsum == 0:
            return 0.0
        return sum(t.weight * t.pass_at_1 for t in self.tasks) / wsum

    @property
    def mean_pass_hat_k(self) -> float:
        return sum(t.pass_hat(self.k) for t in self.tasks) / len(self.tasks) if self.tasks else 0.0

    @property
    def fully_consistent(self) -> int:
        return sum(1 for t in self.tasks if t.c == t.n and t.n > 0)

    def to_dict(self) -> dict:
        return {
            "label": self.label,
            "created": self.created,
            "seeds": self.seeds,
            "k": self.k,
            "aggregate": {
                "n_tasks": len(self.tasks),
                "macro_pass_at_1": round(self.macro_pass_at_1, 6),
                "weighted_pass_at_1": round(self.weighted_pass_at_1, 6),
                f"mean_pass_hat_{self.k}": round(self.mean_pass_hat_k, 6),
                "fully_consistent_tasks": self.fully_consistent,
            },
            "tasks": [t.to_dict(self.k) for t in self.tasks],
        }

    def summary(self) -> str:
        return (
            f"[{self.label or 'eval'}] {len(self.tasks)} tasks x {len(self.seeds)} seeds | "
            f"weighted pass@1={self.weighted_pass_at_1:.1%} | "
            f"mean pass^{self.k}={self.mean_pass_hat_k:.1%} | "
            f"fully-consistent {self.fully_consistent}/{len(self.tasks)}"
        )

    @staticmethod
    def from_dict(data: dict) -> EvalReport:
        k = int(data["k"])
        tasks = []
        for td in data["tasks"]:
            n, c = int(td["n"]), int(td["c"])
            tasks.append(
                TaskResult(
                    task_id=td["task_id"],
                    weight=float(td.get("weight", 1.0)),
                    seeds=list(range(n)),
                    passes=[True] * c + [False] * (n - c),
                )
            )
        return EvalReport(
            tasks=tasks, seeds=list(data.get("seeds", [])), k=k, label=data.get("label", "")
        )


def _subst(template: str, *, seed: int, task: GoldenTask, task_dir: Path) -> str:
    return (
        template.replace("{seed}", str(seed))
        .replace("{task_id}", task.id)
        .replace("{task_dir}", str(task_dir))
    )


def _kill_tree(proc: subprocess.Popen) -> None:
    """Kill the child's whole process group (so daemon grandchildren die too).

    With ``start_new_session=True`` the child is its own group leader, so its PGID
    equals its PID — use that directly (robust even if the leader already exited and
    only a backgrounded grandchild remains)."""
    try:
        if hasattr(os, "killpg"):
            os.killpg(proc.pid, signal.SIGKILL)
        else:  # pragma: no cover - non-POSIX
            proc.kill()
    except (ProcessLookupError, PermissionError, OSError):
        with contextlib.suppress(Exception):
            proc.kill()


def _safe_run(
    cmd,
    *,
    shell: bool = False,
    cwd=None,
    env=None,
    capture_output: bool = False,
    text: bool = True,
    timeout: float | None = None,
):
    """A subprocess.run-compatible runner that does NOT hang on stuck children.

    Two hardening choices over plain subprocess.run:
      * the child runs in its OWN process group (start_new_session) and on timeout
        the WHOLE group is SIGKILLed — a watcher/daemon grandchild that keeps the
        output pipe open can no longer hang the eval past ``timeout``;
      * stdin is /dev/null, so a command that prompts for input gets EOF instead of
        blocking forever.
    Returns a CompletedProcess; raises TimeoutExpired on timeout (mapped to a fail).
    """
    stdout = subprocess.PIPE if capture_output else None
    stderr = subprocess.PIPE if capture_output else None
    kwargs = {
        "shell": shell,
        "cwd": cwd,
        "env": env,
        "stdout": stdout,
        "stderr": stderr,
        "stdin": subprocess.DEVNULL,
        "text": text,
    }
    if hasattr(os, "setsid"):
        kwargs["start_new_session"] = True
    proc = subprocess.Popen(cmd, **kwargs)
    try:
        out, err = proc.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        _kill_tree(proc)
        with contextlib.suppress(subprocess.TimeoutExpired):
            proc.communicate(timeout=5)
        raise
    return subprocess.CompletedProcess(cmd, proc.returncode, out, err)


def run_task(
    task: GoldenTask,
    seed: int,
    base_dir: Path,
    runner=_safe_run,
) -> bool:
    """Run one task at one seed; return True iff it passed.

    Pass criterion: if ``check`` is set, the agent step ``cmd`` runs first (work)
    and pass = (check exit 0) — end-state evaluation. If no ``check``, pass =
    (cmd exit 0). Any timeout or exception counts as a failure (never a pass).
    """
    workdir = task.resolved_workdir(base_dir)
    env = {**os.environ, "AWH_SEED": str(seed)}

    def _run(template: str) -> int:
        cmd = _subst(template, seed=seed, task=task, task_dir=workdir)
        try:
            proc = runner(
                cmd,
                shell=True,  # nosec B604 - runs the operator's golden-set task command
                cwd=str(workdir) if workdir.exists() else None,
                env=env,
                capture_output=True,
                text=True,
                timeout=task.timeout_sec,
            )
            return proc.returncode
        except subprocess.TimeoutExpired:
            return 124
        except OSError:
            return 127

    work_rc = _run(task.cmd)
    if task.check is None:
        return work_rc == 0
    # End-state scoring: the check is the source of truth.
    return _run(task.check) == 0


def evaluate(
    tasks: list[GoldenTask],
    seeds: list[int],
    base_dir: str | os.PathLike[str] = ".",
    k: int | None = None,
    runner=_safe_run,
    label: str = "",
    progress: Callable[[int, int, str], None] | None = None,
) -> EvalReport:
    """Run every task across every seed and build an EvalReport.

    Args:
        tasks: golden tasks.
        seeds: independent seeds (>= 3-5 recommended; size with stats.seeds_for_power).
        base_dir: directory used to resolve each task's workdir.
        k: the k for pass@k / pass^k reporting (defaults to len(seeds)).
        runner: injectable subprocess.run-like callable (default isolates+kills on timeout).
        label: a name for this run (e.g. "baseline", "PR-1234").
        progress: optional callback(index, total, task_id) fired before each task —
            lets a CLI show liveness so a long run never looks frozen.
    """
    if not seeds:
        raise ValueError("need at least one seed")
    base = Path(base_dir)
    k = k or len(seeds)
    results: list[TaskResult] = []
    for i, task in enumerate(tasks):
        if progress:
            progress(i, len(tasks), task.id)
        passes = [run_task(task, s, base, runner=runner) for s in seeds]
        results.append(
            TaskResult(task_id=task.id, weight=task.weight, seeds=list(seeds), passes=passes)
        )
    return EvalReport(tasks=results, seeds=list(seeds), k=k, label=label)


@dataclass
class GateResult:
    ok: bool
    regressions: list[dict]
    aggregate_delta: float
    message: str

    def __bool__(self) -> bool:
        return self.ok


def gate(current: EvalReport, baseline: EvalReport, max_regression: float = 0.0) -> GateResult:
    """Block-on-regression gate: fail if any task (or the aggregate) dropped by more
    than ``max_regression`` vs the baseline.

    Set ``max_regression`` to your measured run-to-run noise floor (the KTH study
    saw >1.5 pts std dev even at temperature 0) so you don't block on noise — but
    never gate on an absolute score.
    """
    base_by_id = {t.task_id: t for t in baseline.tasks}
    regressions: list[dict] = []
    for t in current.tasks:
        b = base_by_id.get(t.task_id)
        if b is None:
            # A current task with no baseline counterpart is NOT silently trusted: the
            # golden set gained a task (e.g. a renamed or new held-out acceptance) without
            # a baseline refresh. Fail closed so the operator re-captures the baseline,
            # rather than skipping the new task (which would leave it un-gated).
            regressions.append(
                {
                    "task_id": t.task_id,
                    "baseline_pass_at_1": None,
                    "current_pass_at_1": round(t.pass_at_1, 6),
                    "drop": None,
                    "reason": "absent_from_baseline",
                }
            )
            continue
        drop = b.pass_at_1 - t.pass_at_1
        if drop > max_regression + 1e-9:
            regressions.append(
                {
                    "task_id": t.task_id,
                    "baseline_pass_at_1": round(b.pass_at_1, 6),
                    "current_pass_at_1": round(t.pass_at_1, 6),
                    "drop": round(drop, 6),
                }
            )
    agg_delta = current.weighted_pass_at_1 - baseline.weighted_pass_at_1
    agg_regressed = (-agg_delta) > max_regression + 1e-9
    ok = not regressions and not agg_regressed
    if ok:
        msg = (
            f"PASS: no task regressed > {max_regression:.1%}; "
            f"aggregate {agg_delta:+.1%} (base {baseline.weighted_pass_at_1:.1%} "
            f"-> {current.weighted_pass_at_1:.1%})."
        )
    else:
        msg = (
            f"FAIL: {len(regressions)} task regression(s); aggregate {agg_delta:+.1%}. "
            f"Block the merge (regression vs baseline, the KTH discipline)."
        )
    return GateResult(ok=ok, regressions=regressions, aggregate_delta=agg_delta, message=msg)
