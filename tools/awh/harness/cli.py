"""`awh` — command-line entry point for the auto-work-harness.

Subcommands map 1:1 to the stages so the toolkit is usable from a Makefile, a
CI job, or an AUTO_WORK gate script:

    awh lock <root>              Stage 0  make test/scoring files read-only
    awh firewall                 Stage 0  render the default-deny egress script
    awh power --baseline ...     Stage 1  seeds-for-power calculator
    awh eval <tasks>             Stage 1  multi-seed eval + regression gate
    awh mutate <file> --test-cmd Stage 2  mutation score for a test suite
    awh worktree <op> ...        Stage 3  manage per-task worktrees
    awh adopt <repo>             Meta     one-command Stage-0 scaffold for a repo
    awh maturity [report|log]    Meta     is the harness fine-tuned enough yet?
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from harness import __version__


def _cmd_lock(args) -> int:
    from harness.stage0_verification.readonly import lock_tests, write_policy_augmentation

    report = lock_tests(
        args.root,
        scoring_globs=tuple(args.scoring) if args.scoring else None,
        hidden=args.hidden,
    )
    print(report.summary())
    if args.write_policy and report.recommended_write_globs:
        p = write_policy_augmentation(args.root, report.recommended_write_globs)
        print(f"wrote deny_write_globs -> {p}")
    return 0 if not report.failed else 1


def _cmd_firewall(args) -> int:
    from harness.stage0_verification.egress import render_init_script

    script = render_init_script(args.domain or None)
    if args.out:
        Path(args.out).write_text(script)
        print(f"wrote {args.out}")
    else:
        print(script)
    return 0


def _cmd_power(args) -> int:
    from harness.common.stats import seeds_for_power

    res = seeds_for_power(args.baseline, args.mde, alpha=args.alpha, power=args.power)
    print(res.summary())
    return 0


def _cmd_eval(args) -> int:
    from harness.stage1_measurement.goldenset import load_tasks
    from harness.stage1_measurement.runner import EvalReport, evaluate, gate

    tasks = load_tasks(args.tasks)
    seeds = list(range(args.seeds))
    print(
        f"[awh] running {len(tasks)} task(s) x {len(seeds)} seed(s); "
        f"output is captured — progress below:",
        file=sys.stderr,
    )

    def _progress(i: int, total: int, task_id: str) -> None:
        print(f"[awh] ({i + 1}/{total}) {task_id} …", file=sys.stderr, flush=True)

    report = evaluate(
        tasks, seeds, base_dir=args.base_dir, label=args.label or "eval", progress=_progress
    )
    print(report.summary())
    if args.out:
        Path(args.out).write_text(json.dumps(report.to_dict(), indent=2))
        print(f"wrote {args.out}")
    if args.baseline:
        base = EvalReport.from_dict(json.loads(Path(args.baseline).read_text()))
        g = gate(report, base, max_regression=args.max_regression)
        print(g.message)
        return 0 if g.ok else 1
    return 0


def _cmd_mutate(args) -> int:
    from harness.stage2_structural.mutation import run_mutation

    report = run_mutation(args.target, args.test_cmd, workdir=args.workdir)
    print(report.summary())
    return 0 if report.score >= args.min_score else 1


def _cmd_worktree(args) -> int:
    from harness.stage3_parallel.worktree import WorktreeManager

    mgr = WorktreeManager(args.repo)
    if args.op == "create":
        wt = mgr.create(args.task_id, base_ref=args.base)
        print(f"created {wt.branch} at {wt.path}")
    elif args.op == "list":
        for w in mgr.list():
            print(f"{w.task_id}\t{w.branch}\t{w.path}")
    elif args.op == "remove":
        mgr.remove(args.task_id, force=args.force, delete_branch=args.delete_branch)
        print(f"removed {args.task_id}")
    elif args.op == "merge":
        res = mgr.merge(args.task_id, into=args.into)
        print(res.message)
        return 0 if res.ok else 1
    return 0


def _cmd_adopt(args) -> int:
    from harness.adopt import scaffold

    root = Path(__file__).resolve().parents[1]
    report = scaffold(args.repo, root, force=args.force)
    print(report.summary())
    print(report.next_steps())
    return 0


def _harness_version(root: Path) -> str:
    import subprocess

    try:
        out = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        if out.returncode == 0 and out.stdout.strip():
            return out.stdout.strip()
    except Exception:  # nosec B110 - git sha is best-effort; fall through to __version__
        pass
    return __version__


def _cmd_maturity(args) -> int:
    from harness import maturity

    root = Path(__file__).resolve().parents[1]
    log_path = Path(args.log) if args.log else root / "evolution-log.jsonl"

    if args.op == "log":
        if not args.repo:
            print("maturity log: --repo is required", file=sys.stderr)
            return 2
        version = args.version or _harness_version(root)
        run = maturity.record_run(
            log_path,
            repo=args.repo,
            harness_version=version,
            outcome=args.outcome,
            categories=args.category,
            note=args.note,
        )
        flag = "evolution" if run.is_evolution() else "no-change"
        print(f"recorded {args.repo} @ {version} [{flag}] -> {log_path}")
        return 0

    report = maturity.summarize(
        log_path,
        window=args.window,
        ready_streak=args.ready_streak,
        max_rate=args.max_rate,
    )
    if args.json:
        print(json.dumps(report.to_dict(), indent=2))
    else:
        print(report.summary())
    # `--gate` turns the report into a CI check (non-zero unless READY).
    if args.gate and report.verdict != "READY":
        return 1
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="awh", description="auto-work-harness CLI")
    p.add_argument("--version", action="version", version=f"awh {__version__}")
    sub = p.add_subparsers(dest="command", required=True)

    lk = sub.add_parser("lock", help="Stage 0: make test/scoring files read-only")
    lk.add_argument("root")
    lk.add_argument("--scoring", nargs="*", help="extra scoring-file globs")
    lk.add_argument("--hidden", action="store_true", help="seal scoring files unreadable")
    lk.add_argument("--write-policy", action="store_true", help="augment .awh/policy.json")
    lk.set_defaults(func=_cmd_lock)

    fw = sub.add_parser("firewall", help="Stage 0: render default-deny egress script")
    fw.add_argument("--domain", nargs="*", help="allowlisted domains (default: built-in set)")
    fw.add_argument("--out", help="write script to this path")
    fw.set_defaults(func=_cmd_firewall)

    pw = sub.add_parser("power", help="Stage 1: seeds needed to detect an effect")
    pw.add_argument("--baseline", type=float, required=True)
    pw.add_argument("--mde", type=float, required=True, help="minimum detectable effect")
    pw.add_argument("--alpha", type=float, default=0.05)
    pw.add_argument("--power", type=float, default=0.80)
    pw.set_defaults(func=_cmd_power)

    ev = sub.add_parser("eval", help="Stage 1: multi-seed eval + regression gate")
    ev.add_argument("tasks", help="golden-set YAML file or directory")
    ev.add_argument("--seeds", type=int, default=5)
    ev.add_argument("--base-dir", default=".")
    ev.add_argument("--label", default="")
    ev.add_argument("--out", help="write the JSON report here")
    ev.add_argument("--baseline", help="baseline report JSON to gate against")
    ev.add_argument("--max-regression", type=float, default=0.0)
    ev.set_defaults(func=_cmd_eval)

    mu = sub.add_parser("mutate", help="Stage 2: mutation score for a test suite")
    mu.add_argument("target", help="source file to mutate")
    mu.add_argument("--test-cmd", required=True, help="command that fails when a mutant is killed")
    mu.add_argument("--workdir", default=None)
    mu.add_argument("--min-score", type=float, default=1.0)
    mu.set_defaults(func=_cmd_mutate)

    wt = sub.add_parser("worktree", help="Stage 3: manage per-task worktrees")
    wt.add_argument("op", choices=["create", "list", "remove", "merge"])
    wt.add_argument("--repo", default=".")
    wt.add_argument("--task-id", default="")
    wt.add_argument("--base", default="HEAD")
    wt.add_argument("--into", default="main")
    wt.add_argument("--force", action="store_true")
    wt.add_argument("--delete-branch", action="store_true")
    wt.set_defaults(func=_cmd_worktree)

    ad = sub.add_parser(
        "adopt",
        help="Meta: one-command Stage-0 scaffold (install hooks + seed .awh/) for a repo",
    )
    ad.add_argument("repo", help="path to the target repo to harden")
    ad.add_argument(
        "--force", action="store_true", help="overwrite an existing .claude/settings.json"
    )
    ad.set_defaults(func=_cmd_adopt)

    ma = sub.add_parser(
        "maturity",
        help="report/track the harness's own fine-tuning convergence",
    )
    ma.add_argument(
        "op",
        nargs="?",
        choices=["report", "log"],
        default="report",
        help="report (default) the verdict, or log an adoption run",
    )
    ma.add_argument("--repo", help="(log) repo being adopted")
    ma.add_argument("--version", help="(log) harness version/sha; default: git short HEAD")
    ma.add_argument(
        "--outcome",
        choices=["green", "red", "unknown"],
        default="unknown",
    )
    ma.add_argument(
        "--category",
        action="append",
        default=[],
        help="(log) evolution category, e.g. recipe:release-safety (repeatable)",
    )
    ma.add_argument("--note", default="")
    ma.add_argument(
        "--log", default=None, help="ledger path (default: <harness>/evolution-log.jsonl)"
    )
    ma.add_argument("--window", type=int, default=5, help="(report) recent-rate window")
    ma.add_argument(
        "--ready-streak", type=int, default=3, help="(report) clean adoptions for READY"
    )
    ma.add_argument(
        "--max-rate", type=float, default=0.2, help="(report) max recent evolution rate for READY"
    )
    ma.add_argument("--gate", action="store_true", help="(report) exit non-zero unless READY")
    ma.add_argument("--json", action="store_true", help="(report) emit JSON")
    ma.set_defaults(func=_cmd_maturity)
    return p


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except Exception as e:  # noqa: BLE001 — turn known user errors into clean messages
        from harness.stage3_parallel.worktree import WorktreeError

        # ValueError covers GoldenSetError / MutationError / VerifierError (all subclasses)
        # plus arg-validation (bad --seeds/--mde/...); FileNotFoundError covers a missing
        # golden set/baseline. Anything else is a real bug -> re-raise with a traceback.
        if isinstance(e, WorktreeError | ValueError | FileNotFoundError):
            print(f"awh: error: {e}", file=sys.stderr)
            return 1
        raise


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
