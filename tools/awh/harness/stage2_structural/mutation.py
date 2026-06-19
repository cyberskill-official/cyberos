"""A small, dependency-free mutation tester.

Mutation testing scores *test-suite quality* by injecting mutants (off-by-one,
flipped comparison/boolean/arithmetic operators) and checking the suite kills
each one. It is the complement to property-based testing: PBT stops an agent
passing by memorizing test inputs; mutation testing stops an agent passing with
tautological/weak tests (``assert isinstance(result, int)``) that a coverage
number would call "tested".

``mutate_source`` is pure (AST in -> list of single-mutation variants out) and
exhaustively unit-tested. ``run_mutation`` applies each mutant to a target file,
runs the test command, and restores the original — the standard mutate-run-restore
loop (mutmut/cosmic-ray), here with zero third-party deps so it runs in any
sandbox. For large suites, prefer mutmut/cosmic-ray; this is the always-available
baseline and the tested reference.
"""

from __future__ import annotations

import ast
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

__all__ = ["Mutant", "MutationError", "MutationReport", "mutate_source", "run_mutation"]


class MutationError(ValueError):
    """The target file cannot be mutation-tested — a syntax error in the source, or
    a test command that does not even pass on the UNMUTATED source."""


# operator -> replacement (single, opinionated mutant per site)
_BINOP = {
    ast.Add: ast.Sub,
    ast.Sub: ast.Add,
    ast.Mult: ast.Div,
    ast.Div: ast.Mult,
    ast.Mod: ast.Mult,
}
_CMPOP = {
    ast.Lt: ast.GtE,
    ast.LtE: ast.Gt,
    ast.Gt: ast.LtE,
    ast.GtE: ast.Lt,
    ast.Eq: ast.NotEq,
    ast.NotEq: ast.Eq,
}
_BOOLOP = {ast.And: ast.Or, ast.Or: ast.And}


@dataclass(frozen=True)
class Mutant:
    index: int
    description: str
    source: str


class _Mutator(ast.NodeTransformer):
    """Applies the ``target``-th mutation; ``target=-1`` mutates nothing (count pass)."""

    def __init__(self, target: int):
        self.target = target
        self.i = 0
        self.desc: str | None = None

    def _site(self, label: str) -> bool:
        idx = self.i
        self.i += 1
        if idx == self.target:
            self.desc = label
            return True
        return False

    def visit_BinOp(self, node: ast.BinOp):
        self.generic_visit(node)
        repl = _BINOP.get(type(node.op))
        if repl and self._site(f"BinOp {type(node.op).__name__}->{repl.__name__}"):
            node.op = repl()
        return node

    def visit_Compare(self, node: ast.Compare):
        self.generic_visit(node)
        # One site PER operator, so a chained compare (lo < x < hi) is fully mutated
        # — mutating only ops[0] let the middle operator escape every test.
        for k, op in enumerate(node.ops):
            repl = _CMPOP.get(type(op))
            if repl and self._site(f"Compare[{k}] {type(op).__name__}->{repl.__name__}"):
                node.ops[k] = repl()
        return node

    def visit_BoolOp(self, node: ast.BoolOp):
        self.generic_visit(node)
        repl = _BOOLOP.get(type(node.op))
        if repl and self._site(f"BoolOp {type(node.op).__name__}->{repl.__name__}"):
            node.op = repl()
        return node

    def visit_Constant(self, node: ast.Constant):
        if isinstance(node.value, bool):
            if self._site(f"Constant {node.value}->{not node.value}"):
                return ast.copy_location(ast.Constant(value=not node.value), node)
        elif isinstance(node.value, int) and self._site(f"Constant {node.value}->{node.value + 1}"):
            return ast.copy_location(ast.Constant(value=node.value + 1), node)
        return node


def _count_sites(src: str) -> int:
    m = _Mutator(target=-1)
    m.visit(ast.parse(src))
    return m.i


def mutate_source(src: str) -> list[Mutant]:
    """Return one Mutant per mutation site (a single mutation each)."""
    total = _count_sites(src)
    mutants: list[Mutant] = []
    for i in range(total):
        mut = _Mutator(target=i)
        tree = mut.visit(ast.parse(src))
        ast.fix_missing_locations(tree)
        mutants.append(Mutant(index=i, description=mut.desc or "?", source=ast.unparse(tree)))
    return mutants


@dataclass
class MutationReport:
    total: int
    killed: int
    survived: list[Mutant] = field(default_factory=list)

    @property
    def score(self) -> float:
        # total==0 means there were no mutatable operations — test-suite quality is
        # UNMEASURED, not perfect. Return NaN so a `score >= min_score` gate fails
        # (and the summary says N/A) rather than silently passing a trivial file at 100%.
        return self.killed / self.total if self.total else float("nan")

    def summary(self) -> str:
        if self.total == 0:
            return "mutation score: N/A — no mutatable operations found (0 mutants)"
        s = f"mutation score {self.score:.0%} ({self.killed}/{self.total} killed)"
        if self.survived:
            s += f"; {len(self.survived)} survived (weak tests): " + ", ".join(
                m.description for m in self.survived[:5]
            )
        return s


def run_mutation(
    target_file: str | Path,
    test_cmd: str,
    *,
    workdir: str | Path | None = None,
    runner=subprocess.run,
    timeout: float = 60.0,
    require_baseline: bool = True,
) -> MutationReport:
    """Mutate ``target_file`` one site at a time, run ``test_cmd``, restore.

    A mutant is *killed* if ``test_cmd`` exits non-zero while it is in place. A
    surviving mutant means the suite did not detect that change — a gap in the
    tests. Returns a MutationReport; the original file is always restored.
    """
    target = Path(target_file)
    original = target.read_text(encoding="utf-8")
    try:
        mutants = mutate_source(original)
    except SyntaxError as e:
        raise MutationError(f"cannot mutate {target}: source has a syntax error: {e}") from e

    def _run() -> int:
        try:
            return runner(
                test_cmd,
                shell=True,  # nosec B604 - runs the operator's test command
                cwd=str(workdir) if workdir else None,
                capture_output=True,
                text=True,
                timeout=timeout,
            ).returncode
        except subprocess.TimeoutExpired:
            return 124  # a hang counts as "killed" (the change broke something)

    # Sanity baseline: the suite must PASS on the unmutated source, else every
    # mutant is spuriously "killed" and the score is meaningless (a misconfigured
    # test command, not strong tests). Catches the most common mutation footgun.
    if require_baseline and _run() != 0:
        raise MutationError(
            f"test command {test_cmd!r} fails on the UNMUTATED source — fix the "
            "suite/command before mutation testing (mutants would be spuriously killed)."
        )

    killed = 0
    survived: list[Mutant] = []
    try:
        for m in mutants:
            target.write_text(m.source, encoding="utf-8")
            if _run() != 0:
                killed += 1
            else:
                survived.append(m)
    finally:
        target.write_text(original, encoding="utf-8")
    return MutationReport(total=len(mutants), killed=killed, survived=survived)
