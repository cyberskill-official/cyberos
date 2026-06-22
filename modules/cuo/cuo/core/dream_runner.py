"""dream_runner - FR-CUO-204 operator entrypoint that runs the dream loop *safely*.

`dream_loop.run_dream_cycle` is pure orchestration with injected dependencies. This module is the thin,
safety-enforcing layer an operator actually invokes. It resolves the enablement mode, picks the apply
binding accordingly, runs exactly one cycle, records a durable audit trail, and prints a report. It never
schedules itself and never runs git, commit, push, or deploy.

The whole point is graduated, locked-down enablement. The apply binding is chosen by mode, and auto-apply
is only ever possible when ALL of these hold (any one missing => dry-run, which changes nothing):

  1. the envelope is enabled and the kill switch (CYBEROS_DREAM_KILL) is unset;
  2. the configured mode is `auto`;
  3. the operator passed the explicit runtime opt-in (`allow_auto_apply=True` / `--allow-auto-apply`);
  4. the working tree is on a dedicated dream branch (its name contains "dream").

In `propose` mode (the shipped default) the apply binding is a dry run that NEVER calls the real applier,
so the loop runs every gate and records what it would do, but cannot change a single file. Stephen reviews
the audit trail, then moves to `auto` deliberately.

All dependencies are injectable so the safety logic is unit-tested with fakes; `main()` builds the real
defaults. The default proposer is intentionally empty - wiring the FR-CUO-201 refinement proposer (and, for
auto, the real FR-CUO-202 applier) is a deliberate operator step, fed through this same gated runner.
"""

from __future__ import annotations

import argparse
import json
import time
from dataclasses import dataclass
from pathlib import Path
from types import SimpleNamespace
from typing import Callable, Iterable, Optional

from cuo.core.dream_loop import DreamReport, run_dream_cycle
from cuo.core.evolution_envelope import EvolutionEnvelope

# Default config location, relative to this file: modules/cuo/config/dream.yaml.
_DEFAULT_CONFIG = Path(__file__).resolve().parents[2] / "config" / "dream.yaml"


@dataclass
class RunResult:
    """What the runner did: the resolved mode, whether apply was even possible, and the loop report."""

    mode: str  # "off" | "propose" | "auto"
    auto_apply_armed: bool  # True only if a real auto-apply could have happened this run
    report: DreamReport
    notes: list  # human-readable notes (e.g. why auto was downgraded to dry-run)


def _dry_run_apply(_prop: object) -> object:
    """Apply binding for every non-auto run: record-only. NEVER applies; never calls the real applier.

    Returns a `QUEUED` outcome so the loop logs the proposal for human review instead of applying it.
    """
    return SimpleNamespace(outcome="QUEUED")


def _looks_like_dream_branch(branch: Optional[str]) -> bool:
    return bool(branch) and "dream" in branch.lower()


def detect_branch(repo_root: Optional[Path] = None) -> Optional[str]:
    """Best-effort current git branch from .git/HEAD (read-only). None if undetectable/detached."""
    root = repo_root or _find_repo_root(Path(__file__).resolve())
    if root is None:
        return None
    head = root / ".git" / "HEAD"
    try:
        text = head.read_text(encoding="utf-8").strip()
    except OSError:
        return None
    if text.startswith("ref:"):
        return text.split("/", 2)[-1] if "/" in text else None
    return None  # detached HEAD


def _find_repo_root(start: Path) -> Optional[Path]:
    for p in [start, *start.parents]:
        if (p / ".git").exists():
            return p
    return None


def choose_apply_fn(
    mode: str,
    *,
    allow_auto_apply: bool,
    branch: Optional[str],
    real_apply_fn: Optional[Callable[[object], object]],
) -> tuple[Callable[[object], object], bool, list]:
    """Pick the apply binding under the auto-apply locks. Returns (apply_fn, auto_armed, notes).

    Auto-apply is selected ONLY when mode is auto AND the operator opted in AND a real applier was provided
    AND we are on a dream branch. Any miss yields the dry-run binding (which cannot change files).
    """
    notes: list = []
    if mode != "auto":
        return _dry_run_apply, False, notes
    if not allow_auto_apply:
        notes.append("mode=auto but --allow-auto-apply not passed; running dry (records only)")
        return _dry_run_apply, False, notes
    if real_apply_fn is None:
        notes.append("mode=auto but no real applier bound; running dry (records only)")
        return _dry_run_apply, False, notes
    if not _looks_like_dream_branch(branch):
        notes.append(
            f"mode=auto but not on a dream branch (branch={branch!r}); refusing auto-apply, running dry"
        )
        return _dry_run_apply, False, notes
    notes.append("auto-apply armed: enabled + mode=auto + opt-in + dream branch all satisfied")
    return real_apply_fn, True, notes


def _empty_proposer() -> Iterable:
    """Default proposal source: none. Wiring the real FR-CUO-201 refinement proposer is an operator step."""
    return []


def _review_required_classifier(_prop: object) -> object:
    """Default classifier: hold everything for review (never auto-applicable). Safe placeholder until the
    real FR-CUO-202 `classify_proposal` is bound."""
    return SimpleNamespace(will_auto_apply=False, risk_class="minor")


@dataclass
class _RefinementProposal:
    """One open refinement proposal, adapted to the dream loop's contract: `.target` is the repo-relative
    SKILL.md the change would write (what the envelope gates on); the rest lets the classifier read it."""

    target: str
    proposal_path: Path
    skill_root: Path
    skill_name: str


def build_refinement_bindings(
    proposals_root, skill_root, repo_root=None
) -> tuple[Callable[[], Iterable], Callable[[object], object]]:
    """Bind the real FR-CUO-201 proposal feed and FR-CUO-202 classifier for propose mode.

    `propose_fn` enumerates OPEN refinement proposals under `proposals_root` and maps each to its target
    SKILL.md (`skill_root/<skill_name>/SKILL.md`, expressed repo-relative so the path envelope matches).
    `classify_fn` is the real `classify_proposal`. Both are read-only - they never move or apply a proposal,
    so they are safe to run in propose mode, which applies nothing regardless. Imports are lazy so the
    runner's safety core does not depend on the proposer modules loading.
    """
    proposals_root = Path(proposals_root)
    skill_root = Path(skill_root)
    repo_root = Path(repo_root) if repo_root else (_find_repo_root(skill_root) or skill_root)

    def propose_fn() -> Iterable:
        from cuo.core.proposal_applier import _extract_frontmatter
        from cuo.core.refinement_proposal import list_proposals

        try:
            open_proposals = list_proposals(proposals_root).get("open", [])
        except OSError:
            return []
        out: list = []
        for pf in open_proposals:
            try:
                fm = _extract_frontmatter(pf.read_text(encoding="utf-8"))
            except OSError:
                continue
            skill_name = str(fm.get("skill_name", "") or "")
            if skill_name:
                skill_md = skill_root / skill_name / "SKILL.md"
                try:
                    target = str(skill_md.resolve().relative_to(repo_root.resolve()))
                except ValueError:
                    target = str(skill_md)  # not under repo_root - the envelope will deny it
            else:
                target = "(unknown)"  # no skill_name - default-deny, recorded for a human
            out.append(
                _RefinementProposal(
                    target=target, proposal_path=pf, skill_root=skill_root, skill_name=skill_name
                )
            )
        return out

    def classify_fn(prop: object) -> object:
        from cuo.core.proposal_applier import classify_proposal

        return classify_proposal(prop.proposal_path, prop.skill_root)

    return propose_fn, classify_fn


def _jsonl_audit(path: Optional[Path]) -> Callable[[str, dict], None]:
    """An audit sink that appends one JSON object per row to `path` (and is a no-op if path is None)."""

    def emit(kind: str, body: dict) -> None:
        if path is None:
            return
        row = {"ts": time.time(), "kind": kind, **(body or {})}
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(row, ensure_ascii=False) + "\n")

    return emit


def run_dream_safely(
    envelope: EvolutionEnvelope,
    *,
    propose_fn: Callable[[], Iterable] = _empty_proposer,
    classify_fn: Callable[[object], object] = _review_required_classifier,
    real_apply_fn: Optional[Callable[[object], object]] = None,
    idle_fn: Callable[[], bool] = lambda: True,
    audit_fn: Optional[Callable[[str, dict], None]] = None,
    now_fn: Callable[[], float] = time.monotonic,
    env: Optional[dict] = None,
    allow_auto_apply: bool = False,
    branch: Optional[str] = None,
    detect_branch_fn: Callable[[], Optional[str]] = detect_branch,
) -> RunResult:
    """Run one dream cycle under the mode ladder and the auto-apply locks. Applies nothing unless every
    lock is satisfied; otherwise records and reports only."""
    mode = envelope.effective_mode(env)
    if mode == "off":
        return RunResult(
            mode="off",
            auto_apply_armed=False,
            report=DreamReport(
                status="disabled",
                reason="dream loop off (mode=off, enabled=false, or kill switch set)",
            ),
            notes=[],
        )

    resolved_branch = branch if branch is not None else detect_branch_fn()
    apply_fn, auto_armed, notes = choose_apply_fn(
        mode,
        allow_auto_apply=allow_auto_apply,
        branch=resolved_branch,
        real_apply_fn=real_apply_fn,
    )

    report = run_dream_cycle(
        envelope,
        propose_fn=propose_fn,
        classify_fn=classify_fn,
        apply_fn=apply_fn,
        idle_fn=idle_fn,
        audit_fn=audit_fn,
        now_fn=now_fn,
        env=env,
    )
    if mode == "propose":
        report.notes.append("propose mode: nothing applied by design; halts above are recorded for review")
    return RunResult(mode=mode, auto_apply_armed=auto_armed, report=report, notes=notes)


def _print_result(result: RunResult) -> None:
    r = result.report
    print(f"[dream-runner] mode={result.mode} status={r.status} auto_apply_armed={result.auto_apply_armed}")
    for note in result.notes:
        print(f"[dream-runner] {note}")
    if r.status != "ran":
        print(f"[dream-runner] {r.reason}")
        return
    print(
        f"[dream-runner] seen={r.seen} applied={r.applied} halted_for_human={r.halted_hitl} "
        f"gate_failed={r.gate_failed}"
    )
    for action, target, reason in r.actions:
        print(f"[dream-runner]   {action}: {target} ({reason})")
    for note in r.notes:
        print(f"[dream-runner]   note: {note}")


def main(argv: Optional[list] = None) -> int:
    parser = argparse.ArgumentParser(description="Run one FR-CUO-204 dream cycle, safely.")
    parser.add_argument("--config", default=str(_DEFAULT_CONFIG), help="path to dream.yaml")
    parser.add_argument(
        "--mode",
        default=None,
        choices=["off", "propose", "auto"],
        help="override the configured mode for this run (does not change the file)",
    )
    parser.add_argument(
        "--allow-auto-apply",
        action="store_true",
        help="explicit opt-in required (with mode=auto + a dream branch) for any auto-apply",
    )
    parser.add_argument("--audit-log", default=None, help="append a JSONL audit row per action here")
    parser.add_argument(
        "--proposals-dir",
        default=None,
        help="root of refinement proposals (with an open/ subdir) to feed propose mode",
    )
    parser.add_argument(
        "--skill-root",
        default=None,
        help="root under which <skill_name>/SKILL.md live (required with --proposals-dir)",
    )
    args = parser.parse_args(argv)

    envelope = EvolutionEnvelope.load(Path(args.config))
    if args.mode is not None:
        envelope.mode = args.mode

    audit_path = Path(args.audit_log) if args.audit_log else None
    audit_fn = _jsonl_audit(audit_path)

    branch = detect_branch()
    if not _looks_like_dream_branch(branch):
        print(
            f"[dream-runner] note: current branch is {branch!r}; auto-apply requires a dream branch "
            "(name contains 'dream'). Propose mode is unaffected."
        )

    # Bind the real FR-CUO-201 proposer + FR-CUO-202 classifier when an operator points at a proposals
    # directory; otherwise the runner proposes nothing (safe default). The real applier (real_apply_fn) is
    # still NOT bound here - auto-apply wiring is the operator's separate, explicit step.
    propose_fn = _empty_proposer
    classify_fn = _review_required_classifier
    if args.proposals_dir and args.skill_root:
        propose_fn, classify_fn = build_refinement_bindings(args.proposals_dir, args.skill_root)
    elif args.proposals_dir or args.skill_root:
        parser.error("--proposals-dir and --skill-root must be given together")

    result = run_dream_safely(
        envelope,
        propose_fn=propose_fn,
        classify_fn=classify_fn,
        audit_fn=audit_fn,
        allow_auto_apply=args.allow_auto_apply,
        branch=branch,
    )
    _print_result(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
