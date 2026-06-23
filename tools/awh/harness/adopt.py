"""`awh adopt` — one-command Stage-0 scaffold for adopting the harness on a repo.

Folds PLAYBOOK steps 1–5 into a single **idempotent** call:

  1. render + install the Stage-0 Claude Code hooks (`.claude/settings.json`)
  2. seed `.awh/gate.sh`        (the evidence-gate command stub)
  3. seed `.awh/goldenset.yaml` (your real checks as deterministic tasks)
  4. seed `.awh/policy.json`    (lock tests read-only so they can't be gamed)

Baseline capture (`awh eval … --out`) and the maturity log (`awh maturity log`)
stay **manual** on purpose: the baseline needs the repo's native toolchain and a
known-green tree, so a scaffold step must not run it. `adopt` prints the exact
commands instead. Existing files are never clobbered (settings.json is written
beside as `settings.awh.json`); `--force` overwrites the settings only.
"""

from __future__ import annotations

import json
import stat
from dataclasses import dataclass, field
from pathlib import Path

_TEMPLATE_REL = "harness/stage0_verification/settings.template.json"

_GATE_STUB = """#!/usr/bin/env sh
# Stage-0 evidence gate — exit non-zero to BLOCK the agent from stopping.
# Replace the echo with this project's real test + lint + typecheck commands.
set -e
echo "[awh-gate] replace me with real checks (e.g. npm test && npm run lint)" >&2
exit 1
"""

_GOLDENSET_STUB = """---
# .awh/goldenset.yaml — your real checks as deterministic tasks.
# Deterministic => 1 seed is enough; every task needs a timeout_sec (bounds a hang).
tasks:
  - id: lint
    description: replace with your linter
    cmd: "echo REPLACE-WITH-YOUR-LINT && false"
    weight: 1.0
    timeout_sec: 240
  - id: test
    description: replace with your test command
    cmd: "echo REPLACE-WITH-YOUR-TESTS && false"
    weight: 3.0
    timeout_sec: 300
"""

# Starter deny-list: the usual test/scoring locations across JS/TS/Python/Go/Rust.
_POLICY_STUB = {
    "deny_write_globs": [
        "tests/**",
        "test/**",
        "**/__tests__/**",
        "**/*.test.*",
        "**/*.spec.*",
        "**/*_test.py",
        "**/*_test.go",
    ]
}


@dataclass
class AdoptReport:
    """What `scaffold()` created vs. left alone, plus a next-steps checklist."""

    repo: Path
    harness_root: Path
    settings_dest: str = ""
    is_git: bool = True
    created: list[str] = field(default_factory=list)
    skipped: list[str] = field(default_factory=list)

    def _rel(self, p: str | Path) -> str:
        try:
            return str(Path(p).relative_to(self.repo))
        except ValueError:
            return str(p)

    def summary(self) -> str:
        lines = [f"awh adopt — scaffolded Stage-0 gates into {self.repo}"]
        for c in self.created:
            lines.append(f"  + {self._rel(c)}")
        for s in self.skipped:
            lines.append(f"  · {self._rel(s)} (exists — left as-is)")
        if not self.is_git:
            lines.append("  ! not a git repo — worktree/release-safety stages assume git")
        return "\n".join(lines)

    def next_steps(self) -> str:
        repo = self.repo
        gate = self._rel(repo / ".awh/gate.sh")
        gs = self._rel(repo / ".awh/goldenset.yaml")
        return "\n".join(
            [
                "",
                "Next (these need your repo's real toolchain — run on the host):",
                f"  1. Edit {gate} to your real test+lint+typecheck (it currently fails on purpose).",
                f"  2. Edit {gs} — replace the stub tasks with your real checks.",
                "  3. Get the repo green, then capture the baseline ON THE NATIVE HOST:",
                f"       awh eval {gs} --seeds 1 --out .awh/eval-baseline.json",
                "     Eyeball it: every task should pass (a baseline captured red neuters the gate).",
                "  4. Commit  .claude/  .awh/  and record the adoption:",
                f"       awh maturity log --repo <org>/{repo.name} --outcome green",
            ]
        )


def _seed(path: Path, content: str, report: AdoptReport, *, make_exec: bool = False) -> None:
    """Write a starter file only if absent; record created vs skipped."""
    if path.exists():
        report.skipped.append(str(path))
        return
    path.write_text(content, encoding="utf-8")
    if make_exec:
        path.chmod(path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    report.created.append(str(path))


def scaffold(repo, harness_root, *, force: bool = False) -> AdoptReport:
    """Idempotently scaffold the Stage-0 gates + `.awh/` stubs into ``repo``.

    Raises ValueError if ``repo`` is not a directory, FileNotFoundError if the
    harness settings template is missing.
    """
    repo = Path(repo).resolve()
    if not repo.is_dir():
        raise ValueError(f"target repo is not a directory: {repo}")
    harness_root = Path(harness_root).resolve()
    template = harness_root / _TEMPLATE_REL
    if not template.exists():
        raise FileNotFoundError(f"harness settings template missing: {template}")

    report = AdoptReport(repo=repo, harness_root=harness_root, is_git=(repo / ".git").exists())

    # 1. Stage-0 hooks: render the absolute harness path into the template.
    claude_dir = repo / ".claude"
    claude_dir.mkdir(parents=True, exist_ok=True)
    rendered = template.read_text(encoding="utf-8").replace("__AWH_ROOT__", str(harness_root))
    dest = claude_dir / "settings.json"
    if dest.exists() and not force:
        # Never clobber a hand-written settings.json — drop ours beside it to merge.
        dest = claude_dir / "settings.awh.json"
    dest.write_text(rendered, encoding="utf-8")
    report.settings_dest = str(dest)
    report.created.append(str(dest))

    # 2–4. `.awh/` stubs — never clobber an existing one.
    awh = repo / ".awh"
    awh.mkdir(parents=True, exist_ok=True)
    _seed(awh / "gate.sh", _GATE_STUB, report, make_exec=True)
    _seed(awh / "goldenset.yaml", _GOLDENSET_STUB, report)
    _seed(awh / "policy.json", json.dumps(_POLICY_STUB, indent=2) + "\n", report)

    return report
