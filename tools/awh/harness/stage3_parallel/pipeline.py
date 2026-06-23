"""planner -> parallel workers (isolated worktrees) -> verifier -> serialized merge.

The only multi-agent shape with strong *coding* evidence (Augment Intent;
Anthropic orchestrator-worker). The invariants this module enforces:

  * Workers run in PARALLEL but each in its OWN git worktree — isolated writes,
    no shared context to fragment (Cognition's objection is about shared-context
    writes; hard filesystem isolation sidesteps it).
  * A SEPARATE verifier reviews each worker's artifact against the spec before
    anything merges ("never validate your own code in the same context window").
  * Merges are SERIALIZED through the integration branch by the orchestrator —
    never agent-to-agent — so a conflict is caught and aborted, not propagated.
  * Cross-worker handoff is via filesystem artifacts + lightweight references, not
    a game-of-telephone of full traces.

Workers and the verifier are injected callables, so the orchestration is unit
tested with stubs against a real temporary repository (and, in one test, the real
Stage-2 Verifier).
"""

from __future__ import annotations

import contextlib
from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field

from harness.stage3_parallel.worktree import Worktree, WorktreeError, WorktreeManager

__all__ = ["WorkerResult", "PipelineTask", "TaskOutcome", "PipelineReport", "Pipeline", "plan"]


@dataclass
class WorkerResult:
    """What a worker returns after operating inside its worktree.

    The worker (the coding agent) is expected to COMMIT its changes on the
    worktree branch and return the artifact (e.g. the diff) for the verifier.
    """

    ok: bool
    artifact: str = ""
    summary: str = ""
    spec: str = ""


# A worker does work in a worktree and returns a WorkerResult.
Worker = Callable[[Worktree], WorkerResult]


@dataclass
class PipelineTask:
    id: str
    worker: Worker
    spec: str = ""


@dataclass
class TaskOutcome:
    task_id: str
    status: str  # merged | verify_failed | worker_failed | conflict | error
    reason: str = ""
    verifier_score: float | None = None
    branch: str = ""

    @property
    def merged(self) -> bool:
        return self.status == "merged"


@dataclass
class PipelineReport:
    outcomes: list[TaskOutcome] = field(default_factory=list)

    @property
    def merged_ids(self) -> list[str]:
        return [o.task_id for o in self.outcomes if o.merged]

    @property
    def blocked_ids(self) -> list[str]:
        return [o.task_id for o in self.outcomes if not o.merged]

    def summary(self) -> str:
        merged = len(self.merged_ids)
        return (
            f"pipeline: {merged}/{len(self.outcomes)} merged; "
            f"blocked: {', '.join(self.blocked_ids) or 'none'}"
        )


def plan(tasks: list[PipelineTask]) -> list[PipelineTask]:
    """Validate that tasks are independent (unique ids). Decomposition by
    domain/feature boundary is the caller's (or an LLM planner's) job; this just
    refuses an obviously-unsafe plan."""
    seen: set[str] = set()
    for t in tasks:
        if t.id in seen:
            raise ValueError(f"duplicate task id in plan: {t.id!r}")
        seen.add(t.id)
    return tasks


def _verify_fn(verifier):
    """Normalize a verifier (Verifier instance or callable) to (artifact, spec) -> result."""
    if hasattr(verifier, "verify"):
        return verifier.verify
    return verifier


class Pipeline:
    def __init__(
        self,
        manager: WorktreeManager,
        verifier,
        integration_branch: str = "main",
        max_parallel: int = 4,
        cleanup_merged: bool = True,
        keep_blocked: bool = True,
    ):
        self.manager = manager
        self.verify = _verify_fn(verifier)
        self.integration_branch = integration_branch
        self.max_parallel = max(1, max_parallel)
        self.cleanup_merged = cleanup_merged
        self.keep_blocked = keep_blocked

    @staticmethod
    def _run_worker(
        item: tuple[PipelineTask, Worktree | None, str | None],
    ) -> tuple[PipelineTask, Worktree | None, WorkerResult | str]:
        task, wt, err = item
        if wt is None:
            return task, None, err or "worktree error"
        try:
            return task, wt, task.worker(wt)
        except Exception as e:  # a worker crash must not sink the whole run
            return task, wt, f"worker raised: {e}"

    def run(self, tasks: list[PipelineTask]) -> PipelineReport:
        plan(tasks)
        # Fail fast BEFORE spawning workers if the integration branch is missing —
        # otherwise every worker runs and then the first merge's `checkout` crashes.
        if tasks and not self.manager._branch_exists(self.integration_branch):
            raise WorktreeError(
                f"integration branch {self.integration_branch!r} does not exist — "
                "create it before running the pipeline"
            )
        report = PipelineReport()

        # Phase 0 — SERIAL worktree creation. `git worktree add` mutates shared .git
        # state and is not safe to run concurrently; isolation is set up here, then
        # the heavy worker bodies run in parallel against their own directories.
        prepared: list[tuple[PipelineTask, Worktree | None, str | None]] = []
        for task in tasks:
            try:
                prepared.append((task, self.manager.create(task.id), None))
            except WorktreeError as e:
                prepared.append((task, None, f"worktree error: {e}"))

        # Phase 1 — PARALLEL, isolated worker execution.
        with ThreadPoolExecutor(max_workers=self.max_parallel) as pool:
            spawned = list(pool.map(self._run_worker, prepared))

        # Phase 2 — SERIALIZED verify + merge, in task order (the integration layer).
        for task, wt, result in spawned:
            if wt is None or isinstance(result, str):
                report.outcomes.append(
                    TaskOutcome(
                        task.id,
                        "error",
                        reason=str(result),
                        branch=self.manager.branch_for(task.id),
                    )
                )
                continue
            branch = wt.branch
            if not result.ok:
                report.outcomes.append(
                    TaskOutcome(
                        task.id,
                        "worker_failed",
                        reason=result.summary or "worker returned ok=False",
                        branch=branch,
                    )
                )
                self._maybe_keep(task.id)
                continue

            vr = self.verify(result.artifact, result.spec or task.spec)
            passed = bool(getattr(vr, "passed", vr))
            score = getattr(vr, "overall_score", None)
            if not passed:
                report.outcomes.append(
                    TaskOutcome(
                        task.id,
                        "verify_failed",
                        reason=getattr(vr, "summary", "")
                        or getattr(vr, "error", "verifier rejected"),
                        verifier_score=score,
                        branch=branch,
                    )
                )
                self._maybe_keep(task.id)
                continue

            merge = self.manager.merge(task.id, into=self.integration_branch)
            if merge.ok:
                report.outcomes.append(
                    TaskOutcome(task.id, "merged", verifier_score=score, branch=branch)
                )
                if self.cleanup_merged:
                    # a failed cleanup must not abort the remaining serialized merges
                    with contextlib.suppress(WorktreeError):
                        self.manager.remove(task.id, force=True, delete_branch=True)
            else:
                report.outcomes.append(
                    TaskOutcome(
                        task.id,
                        "conflict",
                        reason=merge.message,
                        verifier_score=score,
                        branch=branch,
                    )
                )
                self._maybe_keep(task.id)
        return report

    def _maybe_keep(self, task_id: str) -> None:
        if not self.keep_blocked:
            with contextlib.suppress(WorktreeError):
                self.manager.remove(task_id, force=True)
