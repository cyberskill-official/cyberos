"""Self-evolution tracking — answers "is the harness fine-tuned *enough* yet?".

The harness "evolves" when adopting a real repo surfaces a gap that forces a
change to the harness itself: a new gate, a new recipe, a bug fix, a new gotcha.
Each such change is an **evolution event**. The harness is mature / ready when
adopting *new* repos stops producing evolution events — an empirical convergence
signal, not a gut feeling.

This module keeps one JSONL line per adoption run and reports a verdict:

    READY              recent adoptions needed no harness change (converged)
    STILL_EVOLVING     new repos are still teaching the harness things
    INSUFFICIENT_DATA  too few adoptions to judge

`harness_changed` is derived automatically by comparing the harness version of a
run to the previous run, so the curve is recorded with no extra bookkeeping; a
run can also carry explicit `categories` (e.g. ``recipe:release-safety``) when a
human/agent folds a lesson back.
"""

from __future__ import annotations

import json
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path


@dataclass
class Run:
    """One adoption / eval run of the harness against a repo."""

    ts: float
    repo: str
    harness_version: str
    outcome: str = "unknown"  # green | red | unknown
    harness_changed: bool = False
    categories: list[str] = field(default_factory=list)
    note: str = ""

    def is_evolution(self) -> bool:
        """True if the harness changed since the previous run, or this run
        explicitly recorded a fold-back category."""
        return bool(self.harness_changed or self.categories)


def read_runs(log_path) -> list[Run]:
    """Load all runs from a JSONL ledger (missing file -> empty).

    Degrades gracefully: a corrupt/partial line (e.g. a crash mid-write) or a line
    from a newer schema (extra keys) is skipped, never crashing the whole read —
    the ledger is append-only audit data, one bad line must not poison it.
    """
    p = Path(log_path)
    if not p.exists():
        return []
    fields = set(Run.__dataclass_fields__)
    runs: list[Run] = []
    for line in p.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            d = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(d, dict) or "repo" not in d or "harness_version" not in d:
            continue
        kwargs = {k: v for k, v in d.items() if k in fields}  # tolerate forward-compat keys
        kwargs.setdefault("ts", 0.0)
        try:
            runs.append(Run(**kwargs))
        except (TypeError, ValueError):
            continue
    return runs


def record_run(
    log_path,
    *,
    repo: str,
    harness_version: str,
    outcome: str = "unknown",
    categories=None,
    note: str = "",
    now: float | None = None,
) -> Run:
    """Append a run to the ledger. ``harness_changed`` is derived by comparing
    ``harness_version`` to the most recent prior run."""
    runs = read_runs(log_path)
    prev = runs[-1] if runs else None
    changed = prev is not None and prev.harness_version != harness_version
    run = Run(
        ts=time.time() if now is None else now,
        repo=repo,
        harness_version=harness_version,
        outcome=outcome,
        harness_changed=bool(changed),
        categories=list(categories or []),
        note=note,
    )
    p = Path(log_path)
    p.parent.mkdir(parents=True, exist_ok=True)
    with p.open("a", encoding="utf-8") as f:
        f.write(json.dumps(asdict(run)) + "\n")
    return run


@dataclass
class MaturityReport:
    """A convergence read-out over the evolution ledger."""

    adoptions: int
    distinct_repos: int
    evolution_events: int
    no_evolution_streak: int
    recent_rate: float
    window: int
    verdict: str
    rationale: str
    categories: dict[str, int]
    last_evolution_repo: str | None
    last_evolution_ts: float | None

    def to_dict(self) -> dict:
        return asdict(self)

    def summary(self) -> str:
        lines = [
            "auto-work-harness — maturity / self-evolution",
            f"  adoptions recorded : {self.adoptions} across {self.distinct_repos} repo(s)",
            f"  evolution events   : {self.evolution_events}",
            f"  clean streak       : {self.no_evolution_streak} consecutive no-change adoption(s)",
            f"  recent rate        : {self.recent_rate:.0%} over last {self.window}",
            f"  VERDICT            : {self.verdict}",
            f"    -> {self.rationale}",
        ]
        if self.categories:
            cats = ", ".join(f"{k}x{v}" for k, v in self.categories.items())
            lines.append(f"  evolution by type  : {cats}")
        return "\n".join(lines)


def summarize(
    log_path,
    *,
    window: int = 5,
    ready_streak: int = 3,
    max_rate: float = 0.2,
) -> MaturityReport:
    """Compute the convergence verdict.

    READY requires both (a) at least ``ready_streak`` trailing adoptions with no
    evolution event, and (b) a recent evolution rate at/below ``max_rate`` over
    the last ``window`` runs. Belt-and-suspenders: a long-ago streak alone isn't
    enough if the recent window is still churning.
    """
    window = max(1, window)  # a 0/negative window would slice the whole list via [-0:]
    runs = read_runs(log_path)
    n = len(runs)
    distinct = len({r.repo for r in runs})
    evo = [r for r in runs if r.is_evolution()]

    streak = 0
    for r in reversed(runs):
        if r.is_evolution():
            break
        streak += 1

    recent = runs[-window:]
    recent_evo = sum(1 for r in recent if r.is_evolution())
    rate = (recent_evo / len(recent)) if recent else 0.0

    cats: dict[str, int] = {}
    for r in evo:
        for c in r.categories:
            cats[c] = cats.get(c, 0) + 1

    last = evo[-1] if evo else None

    if n < ready_streak:
        verdict = "INSUFFICIENT_DATA"
        rationale = (
            f"only {n} adoption(s) recorded; need at least {ready_streak} "
            "to judge convergence. Adopt more repos and re-check."
        )
    elif streak >= ready_streak and rate <= max_rate:
        verdict = "READY"
        rationale = (
            f"{streak} consecutive adoption(s) needed no harness change and the "
            f"recent evolution rate ({rate:.0%} over last {len(recent)}) is "
            f"at/below the {max_rate:.0%} threshold — it has converged."
        )
    else:
        need = max(0, ready_streak - streak)
        verdict = "STILL_EVOLVING"
        rationale = (
            f"recent evolution rate {rate:.0%} over last {len(recent)}, clean "
            f"streak {streak}; need {need} more clean adoption(s) and rate "
            f"<= {max_rate:.0%} to call it READY."
        )

    return MaturityReport(
        adoptions=n,
        distinct_repos=distinct,
        evolution_events=len(evo),
        no_evolution_streak=streak,
        recent_rate=rate,
        window=len(recent),
        verdict=verdict,
        rationale=rationale,
        categories=dict(sorted(cats.items(), key=lambda kv: -kv[1])),
        last_evolution_repo=last.repo if last else None,
        last_evolution_ts=last.ts if last else None,
    )
