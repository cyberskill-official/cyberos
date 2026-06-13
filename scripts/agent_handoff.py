#!/usr/bin/env python3
"""Generate cross-agent handoff packets for long CyberOS goals.

The packet is intentionally written under target/cuo-workflow/handoffs/ so it
is local operational state, not part of any FR implementation commit.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable


REPO_ROOT = Path(__file__).resolve().parents[1]
BACKLOG = REPO_ROOT / "docs/feature-requests/BACKLOG.md"
DEFAULT_OUT = REPO_ROOT / "target/cuo-workflow/handoffs"


@dataclass(frozen=True)
class BacklogRow:
    fr_id: str
    title: str
    priority: str
    status: str
    depends_on: list[str]
    effort: str
    eligible: bool


def run_git(args: list[str]) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=REPO_ROOT,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    return result.stdout.rstrip("\n")


def git_lines(args: list[str]) -> list[str]:
    output = run_git(args)
    return output.splitlines() if output else []


def parse_backlog_rows(text: str) -> list[BacklogRow]:
    rows: list[tuple[str, str, str, str, list[str], str]] = []
    row_re = re.compile(
        r"^\|\s+\*\*(FR-[A-Z0-9-]+)\*\*\s+\|\s+(.+?)\s+\|\s+(.+?)\s+\|\s+(.+?)\s+\|\s+(.+?)\s+\|\s+(.+?)\s+\|$"
    )
    for line in text.splitlines():
        match = row_re.match(line)
        if not match:
            continue
        fr_id, title, priority, status, depends, effort = match.groups()
        deps = parse_depends(depends)
        rows.append((fr_id, title, priority, status, deps, effort))

    status_by_id = {fr_id: status for fr_id, _t, _p, status, _d, _e in rows}
    out: list[BacklogRow] = []
    for fr_id, title, priority, status, deps, effort in rows:
        eligible = status == "ready_to_implement" and all(
            status_by_id.get(dep) == "done" for dep in deps
        )
        out.append(
            BacklogRow(
                fr_id=fr_id,
                title=clean_cell(title),
                priority=clean_cell(priority),
                status=clean_cell(status),
                depends_on=deps,
                effort=clean_cell(effort),
                eligible=eligible,
            )
        )
    return out


def parse_depends(cell: str) -> list[str]:
    cleaned = clean_cell(cell)
    if cleaned in {"-", "—", "none", ""}:
        return []
    return re.findall(r"FR-[A-Z0-9-]+", cleaned)


def clean_cell(cell: str) -> str:
    return re.sub(r"\s+", " ", cell.replace("`", "").strip())


def first_eligible(rows: Iterable[BacklogRow]) -> BacklogRow | None:
    return next((row for row in rows if row.eligible), None)


def slugify(value: str) -> str:
    value = re.sub(r"[^A-Za-z0-9._-]+", "-", value.strip().lower())
    return value.strip("-") or "handoff"


def status_entries() -> dict[str, list[str]]:
    short = git_lines(["status", "--short"])
    return {
        "staged": [line for line in short if line[:2] != "??" and line[0] != " "],
        "unstaged": [line for line in short if line[:2] != "??" and line[1] != " "],
        "untracked": [line for line in short if line.startswith("??")],
        "all": short,
    }


def build_state(args: argparse.Namespace, rows: list[BacklogRow], packet_dir: Path) -> dict:
    eligible = [row for row in rows if row.eligible]
    next_row = first_eligible(rows)
    recommended = row_by_id(rows, args.next_fr) if args.next_fr else next_row
    status = status_entries()
    return {
        "schema": "cyberos.agent-handoff@1",
        "created_at": datetime.now(timezone.utc).isoformat(),
        "reason": args.reason,
        "agent": args.agent,
        "active_fr": args.active_fr,
        "next_eligible_fr": asdict(next_row) if next_row else None,
        "recommended_next_fr": asdict(recommended) if recommended else None,
        "eligible_count": len(eligible),
        "eligible_preview": [asdict(row) for row in eligible[:20]],
        "branch": run_git(["branch", "--show-current"]),
        "head": run_git(["rev-parse", "HEAD"]),
        "status": status,
        "dirty": bool(status["all"]),
        "notes": args.note,
        "packet_dir": str(packet_dir.relative_to(REPO_ROOT)),
    }


def row_by_id(rows: Iterable[BacklogRow], fr_id: str | None) -> BacklogRow | None:
    if not fr_id:
        return None
    return next((row for row in rows if row.fr_id == fr_id), None)


def write_packet(args: argparse.Namespace) -> Path:
    rows = parse_backlog_rows(BACKLOG.read_text(encoding="utf-8"))
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    subject = args.active_fr or (first_eligible(rows).fr_id if first_eligible(rows) else "drained")
    packet_dir = args.out_dir / f"{stamp}-{slugify(subject)}"
    packet_dir.mkdir(parents=True, exist_ok=False)

    state = build_state(args, rows, packet_dir)
    (packet_dir / "STATE.json").write_text(
        json.dumps(state, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (packet_dir / "git-status.txt").write_text(
        run_git(["status", "--short", "--branch"]) + "\n",
        encoding="utf-8",
    )
    (packet_dir / "diff-stat.txt").write_text(
        run_git(["diff", "--stat"]) + "\n",
        encoding="utf-8",
    )
    (packet_dir / "staged-diff-stat.txt").write_text(
        run_git(["diff", "--cached", "--stat"]) + "\n",
        encoding="utf-8",
    )
    (packet_dir / "recent-commits.txt").write_text(
        run_git(["log", "--oneline", "-12"]) + "\n",
        encoding="utf-8",
    )
    (packet_dir / "ready-queue.txt").write_text(render_ready_queue(rows), encoding="utf-8")
    (packet_dir / "HANDOFF.md").write_text(render_handoff(state), encoding="utf-8")
    (packet_dir / "RESUME_PROMPT.md").write_text(render_resume_prompt(state), encoding="utf-8")

    latest = args.out_dir / "LATEST"
    latest.write_text(str(packet_dir.relative_to(REPO_ROOT)) + "\n", encoding="utf-8")
    return packet_dir


def render_ready_queue(rows: list[BacklogRow]) -> str:
    lines = ["# Ready Queue", ""]
    for row in rows:
        if row.status != "ready_to_implement":
            continue
        marker = "eligible" if row.eligible else "blocked-by-deps"
        deps = ", ".join(row.depends_on) if row.depends_on else "-"
        lines.append(f"- {row.fr_id} [{marker}] {row.title} | deps: {deps}")
    return "\n".join(lines) + "\n"


def render_handoff(state: dict) -> str:
    next_fr = state["next_eligible_fr"]["fr_id"] if state["next_eligible_fr"] else "none"
    recommended = (
        state["recommended_next_fr"]["fr_id"] if state["recommended_next_fr"] else next_fr
    )
    active = state["active_fr"] or "none"
    notes = "\n".join(f"- {note}" for note in state["notes"]) or "- none"
    dirty = "yes" if state["dirty"] else "no"
    return f"""# Agent Handoff

Generated: {state["created_at"]}
Reason: {state["reason"]}
Agent: {state["agent"]}

## Resume State

- Branch: `{state["branch"]}`
- HEAD: `{state["head"]}`
- Active FR: `{active}`
- First dependency-eligible FR: `{next_fr}`
- Recommended resume FR: `{recommended}`
- Dirty worktree: `{dirty}`
- Packet: `{state["packet_dir"]}`

## Notes

{notes}

## Rules For The Next Agent

1. Read `STATE.json`, `git-status.txt`, `diff-stat.txt`, and `ready-queue.txt` before editing.
2. Do not stage unrelated dirty files. In this run, `AGENTS.md`, `services/audit-profile.yaml`, and `services/docs/` may be unrelated local state unless the packet notes say otherwise.
3. If `active_fr` is set and the worktree is dirty, finish or route back that FR before selecting another FR.
4. Keep the per-FR commit rule: exactly one shipped FR per commit, excluding `.cyberos-memory/` and `target/`.
5. Before handing off again, run `python3 scripts/agent_handoff.py --reason <reason> --active-fr <FR-ID> --note "..."`
"""


def render_resume_prompt(state: dict) -> str:
    next_fr = state["next_eligible_fr"]["fr_id"] if state["next_eligible_fr"] else "none"
    recommended = (
        state["recommended_next_fr"]["fr_id"] if state["recommended_next_fr"] else next_fr
    )
    active = state["active_fr"] or "none"
    return f"""Continue the CyberOS FR drain goal from this handoff packet.

Repo: {REPO_ROOT}
Branch: {state["branch"]}
HEAD: {state["head"]}
Handoff packet: {state["packet_dir"]}
Active FR: {active}
First dependency-eligible FR: {next_fr}
Recommended resume FR: {recommended}

First actions:
1. Read `{state["packet_dir"]}/HANDOFF.md`, `STATE.json`, `git-status.txt`, `diff-stat.txt`, and `ready-queue.txt`.
2. Run `git status --short --branch`.
3. Continue `modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md`.
4. Preserve the one-commit-per-done-FR rule and exclude `.cyberos-memory/` and `target/` artifacts.
5. If usage limits approach again, generate a fresh handoff packet with `python3 scripts/agent_handoff.py`.
"""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reason", default="manual-handoff")
    parser.add_argument("--agent", default="codex")
    parser.add_argument("--active-fr", default=None)
    parser.add_argument(
        "--next-fr",
        default=None,
        help="recommended FR for the next agent when first eligible rows are intentionally routed back",
    )
    parser.add_argument("--note", action="append", default=[])
    parser.add_argument("--out-dir", type=Path, default=DEFAULT_OUT)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    args.out_dir = args.out_dir.resolve()
    packet = write_packet(args)
    print(packet.relative_to(REPO_ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
