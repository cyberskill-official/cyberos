#!/usr/bin/env python3
"""Generate cross-agent handoff packets for long CyberOS goals.

The packet is intentionally written under target/cuo-workflow/handoffs/ so it
is local operational state, not part of any FR implementation commit.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Iterable


REPO_ROOT = Path(__file__).resolve().parents[1]
BACKLOG = REPO_ROOT / "docs/feature-requests/BACKLOG.md"
DEFAULT_OUT = REPO_ROOT / "target/cuo-workflow/handoffs"
CLAIM_DIR = REPO_ROOT / "target/cuo-workflow/agent-session"
CLAIM_FILE = CLAIM_DIR / "CLAIM.json"


class HandoffError(RuntimeError):
    """Raised when a handoff command would create unsafe agent state."""


@dataclass(frozen=True)
class BacklogRow:
    fr_id: str
    title: str
    priority: str
    status: str
    depends_on: list[str]
    effort: str
    eligible: bool


@dataclass(frozen=True)
class ValidationResult:
    ok: bool
    errors: list[str]
    warnings: list[str]


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
        "handoff_to": args.handoff_to,
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
        "claim_file": str(CLAIM_FILE.relative_to(REPO_ROOT)),
        "resume_command": (
            "python3 scripts/agent_handoff.py resume "
            f"--agent {args.handoff_to or '<next-agent>'} "
            f"--packet {packet_dir.relative_to(REPO_ROOT)}"
        ),
        "claim_command": (
            "python3 scripts/agent_handoff.py claim "
            f"--agent {args.handoff_to or '<next-agent>'} "
            f"--packet {packet_dir.relative_to(REPO_ROOT)}"
        ),
        "handoff_command": (
            "python3 scripts/agent_handoff.py create "
            "--reason usage-limit --agent <current-agent> --active-fr <FR-ID> "
            "--handoff-to <next-agent> --note \"phase/tests/blockers\""
        ),
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
    if args.release_claim:
        release_claim(args.agent, force=args.force, reason=f"handoff:{args.reason}")
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
    handoff_to = state["handoff_to"] or "<next-agent>"
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
- Claim file: `{state["claim_file"]}`

## Commands

Next agent:

```bash
{state["resume_command"]}
```

Create a fresh packet before handing off again:

```bash
{state["handoff_command"]}
```

## Notes

{notes}

## Rules For The Next Agent

1. Read `STATE.json`, `git-status.txt`, `diff-stat.txt`, and `ready-queue.txt` before editing.
2. Run `python3 scripts/agent_handoff.py claim --agent {handoff_to} --packet {state["packet_dir"]}` before editing.
3. Do not stage unrelated dirty files. In this run, `AGENTS.md`, `services/audit-profile.yaml`, and `services/docs/` may be unrelated local state unless the packet notes say otherwise.
4. If `active_fr` is set and the worktree is dirty, finish or route back that FR before selecting another FR.
5. Keep the per-FR commit rule: exactly one shipped FR per commit, excluding `.cyberos-memory/` and `target/`.
6. Before handing off again, run `python3 scripts/agent_handoff.py create --reason <reason> --active-fr <FR-ID> --note "..."`.
7. When a handoff is complete, run `python3 scripts/agent_handoff.py release --agent {handoff_to}`.
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
2. Run `python3 scripts/agent_handoff.py claim --agent <your-agent-name> --packet {state["packet_dir"]}`.
3. Run `git status --short --branch`.
4. Continue `modules/cuo/chief-technology-officer/workflows/ship-feature-requests.md`.
5. Preserve the one-commit-per-done-FR rule and exclude `.cyberos-memory/` and `target/` artifacts.
6. If usage limits approach again, generate a fresh handoff packet with `python3 scripts/agent_handoff.py create`.
"""


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def isoformat(value: datetime) -> str:
    return value.isoformat()


def parse_timestamp(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def relative(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def latest_packet_dir(out_dir: Path = DEFAULT_OUT) -> Path:
    latest = out_dir / "LATEST"
    if not latest.is_file():
        raise HandoffError(f"no latest handoff pointer at {relative(latest)}")
    raw = latest.read_text(encoding="utf-8").strip()
    if not raw:
        raise HandoffError(f"empty latest handoff pointer at {relative(latest)}")
    path = Path(raw)
    return path if path.is_absolute() else REPO_ROOT / path


def resolve_packet_dir(packet: str | None, out_dir: Path = DEFAULT_OUT) -> Path:
    if packet in {None, "", "latest"}:
        path = latest_packet_dir(out_dir)
    else:
        path = Path(packet)
        if not path.is_absolute():
            path = REPO_ROOT / path
    if path.is_file() and path.name == "STATE.json":
        path = path.parent
    return path.resolve()


def load_packet_state(packet_dir: Path) -> dict:
    state_path = packet_dir / "STATE.json"
    if not state_path.is_file():
        raise HandoffError(f"missing handoff state at {relative(state_path)}")
    with state_path.open(encoding="utf-8") as f:
        state = json.load(f)
    if state.get("schema") != "cyberos.agent-handoff@1":
        raise HandoffError(f"unsupported handoff schema in {relative(state_path)}")
    return state


def load_claim() -> dict | None:
    if not CLAIM_FILE.is_file():
        return None
    with CLAIM_FILE.open(encoding="utf-8") as f:
        return json.load(f)


def claim_is_expired(claim: dict, now: datetime | None = None) -> bool:
    expires_at = parse_timestamp(claim.get("expires_at"))
    if not expires_at:
        return False
    return (now or utc_now()) >= expires_at


def write_claim(
    *,
    agent: str,
    active_fr: str | None,
    packet_dir: Path | None,
    ttl_hours: float,
    force: bool,
    reason: str,
    notes: list[str],
) -> Path:
    existing = load_claim()
    if existing and not force and not claim_is_expired(existing):
        existing_agent = existing.get("agent")
        if existing_agent != agent:
            raise HandoffError(
                "active agent claim exists for "
                f"{existing_agent}; release it or rerun with --force"
            )

    state = load_packet_state(packet_dir) if packet_dir else {}
    inferred_fr = active_fr or state.get("active_fr")
    if not inferred_fr and state.get("recommended_next_fr"):
        inferred_fr = state["recommended_next_fr"].get("fr_id")

    now = utc_now()
    claim = {
        "schema": "cyberos.agent-claim@1",
        "agent": agent,
        "active_fr": inferred_fr,
        "packet_dir": relative(packet_dir) if packet_dir else None,
        "branch": run_git(["branch", "--show-current"]),
        "head": run_git(["rev-parse", "HEAD"]),
        "claimed_at": isoformat(now),
        "updated_at": isoformat(now),
        "expires_at": isoformat(now + timedelta(hours=ttl_hours)),
        "reason": reason,
        "notes": notes,
    }
    CLAIM_DIR.mkdir(parents=True, exist_ok=True)
    CLAIM_FILE.write_text(json.dumps(claim, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return CLAIM_FILE


def release_claim(agent: str, *, force: bool, reason: str) -> bool:
    claim = load_claim()
    if not claim:
        return False
    if not force and claim.get("agent") != agent:
        raise HandoffError(
            f"claim belongs to {claim.get('agent')}; rerun with --force to release it"
        )
    CLAIM_DIR.mkdir(parents=True, exist_ok=True)
    released = dict(claim)
    released["released_at"] = isoformat(utc_now())
    released["release_reason"] = reason
    archive_name = utc_now().strftime("RELEASED-%Y%m%dT%H%M%SZ.json")
    (CLAIM_DIR / archive_name).write_text(
        json.dumps(released, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    CLAIM_FILE.unlink()
    return True


def validate_packet_dir(packet_dir: Path, *, strict: bool = False) -> ValidationResult:
    errors: list[str] = []
    warnings: list[str] = []
    required = [
        "HANDOFF.md",
        "RESUME_PROMPT.md",
        "STATE.json",
        "git-status.txt",
        "diff-stat.txt",
        "staged-diff-stat.txt",
        "recent-commits.txt",
        "ready-queue.txt",
    ]
    for name in required:
        if not (packet_dir / name).is_file():
            errors.append(f"missing {relative(packet_dir / name)}")
    try:
        state = load_packet_state(packet_dir)
    except HandoffError as exc:
        errors.append(str(exc))
        state = {}

    current_branch = run_git(["branch", "--show-current"])
    current_head = run_git(["rev-parse", "HEAD"])
    if state:
        if state.get("branch") and state["branch"] != current_branch:
            errors.append(
                f"branch mismatch: packet={state['branch']} current={current_branch}"
            )
        if state.get("head") and state["head"] != current_head:
            message = f"HEAD differs: packet={state['head']} current={current_head}"
            if strict:
                errors.append(message)
            else:
                warnings.append(message)
        current_dirty = bool(status_entries()["all"])
        if bool(state.get("dirty")) != current_dirty:
            message = (
                f"dirty-state differs: packet={bool(state.get('dirty'))} "
                f"current={current_dirty}"
            )
            if strict:
                errors.append(message)
            else:
                warnings.append(message)

    claim = load_claim()
    if claim and claim_is_expired(claim):
        warnings.append(f"agent claim is expired for {claim.get('agent')}")
    elif claim:
        warnings.append(f"active agent claim exists for {claim.get('agent')}")

    return ValidationResult(ok=not errors, errors=errors, warnings=warnings)


def add_create_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--reason", default="manual-handoff")
    parser.add_argument("--agent", default=os.environ.get("CYBEROS_AGENT", "codex"))
    parser.add_argument("--handoff-to", default=None)
    parser.add_argument("--active-fr", default=None)
    parser.add_argument(
        "--next-fr",
        default=None,
        help="recommended FR for the next agent when first eligible rows are intentionally routed back",
    )
    parser.add_argument("--note", action="append", default=[])
    parser.add_argument("--out-dir", type=Path, default=DEFAULT_OUT)
    parser.add_argument(
        "--release-claim",
        action="store_true",
        help="release this agent's current claim after writing the packet",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="force claim release when used with --release-claim",
    )


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    argv = list(sys.argv[1:] if argv is None else argv)
    if not argv or argv[0].startswith("-"):
        parser = argparse.ArgumentParser(description=__doc__)
        add_create_arguments(parser)
        parser.set_defaults(command="create")
        return parser.parse_args(argv)

    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)

    create = subcommands.add_parser("create", help="write a handoff packet")
    add_create_arguments(create)

    claim = subcommands.add_parser("claim", help="claim the local FR-drain session")
    claim.add_argument("--agent", required=True)
    claim.add_argument("--active-fr", default=None)
    claim.add_argument("--packet", default="latest")
    claim.add_argument("--reason", default="resume")
    claim.add_argument("--note", action="append", default=[])
    claim.add_argument("--ttl-hours", type=float, default=24.0)
    claim.add_argument("--force", action="store_true")

    release = subcommands.add_parser("release", help="release the local agent claim")
    release.add_argument("--agent", required=True)
    release.add_argument("--reason", default="handoff-complete")
    release.add_argument("--force", action="store_true")

    resume = subcommands.add_parser("resume", help="validate packet, claim it, and print resume prompt")
    resume.add_argument("--agent", required=True)
    resume.add_argument("--packet", default="latest")
    resume.add_argument("--ttl-hours", type=float, default=24.0)
    resume.add_argument("--force", action="store_true")
    resume.add_argument("--no-claim", action="store_true")
    resume.add_argument("--strict", action="store_true")

    validate = subcommands.add_parser("validate", help="validate a handoff packet")
    validate.add_argument("--packet", default="latest")
    validate.add_argument("--strict", action="store_true")

    subcommands.add_parser("status", help="print latest packet and active claim")
    return parser.parse_args(argv)


def main() -> int:
    args = parse_args()
    try:
        if args.command == "create":
            args.out_dir = args.out_dir.resolve()
            packet = write_packet(args)
            print(packet.relative_to(REPO_ROOT))
            return 0
        if args.command == "claim":
            packet = resolve_packet_dir(args.packet)
            path = write_claim(
                agent=args.agent,
                active_fr=args.active_fr,
                packet_dir=packet,
                ttl_hours=args.ttl_hours,
                force=args.force,
                reason=args.reason,
                notes=args.note,
            )
            print(relative(path))
            return 0
        if args.command == "release":
            released = release_claim(args.agent, force=args.force, reason=args.reason)
            print("released" if released else "no active claim")
            return 0
        if args.command == "resume":
            packet = resolve_packet_dir(args.packet)
            result = validate_packet_dir(packet, strict=args.strict)
            for warning in result.warnings:
                print(f"warning: {warning}", file=sys.stderr)
            if not result.ok:
                for error in result.errors:
                    print(f"error: {error}", file=sys.stderr)
                return 1
            if not args.no_claim:
                write_claim(
                    agent=args.agent,
                    active_fr=None,
                    packet_dir=packet,
                    ttl_hours=args.ttl_hours,
                    force=args.force,
                    reason="resume",
                    notes=[],
                )
            print((packet / "RESUME_PROMPT.md").read_text(encoding="utf-8"))
            return 0
        if args.command == "validate":
            packet = resolve_packet_dir(args.packet)
            result = validate_packet_dir(packet, strict=args.strict)
            for warning in result.warnings:
                print(f"warning: {warning}", file=sys.stderr)
            for error in result.errors:
                print(f"error: {error}", file=sys.stderr)
            if result.ok:
                print(f"OK {relative(packet)}")
            return 0 if result.ok else 1
        if args.command == "status":
            try:
                latest = latest_packet_dir()
                print(f"latest_packet: {relative(latest)}")
            except HandoffError as exc:
                print(f"latest_packet: none ({exc})")
            claim = load_claim()
            if claim:
                expiry = "expired" if claim_is_expired(claim) else "active"
                print(
                    "claim: "
                    f"{expiry} agent={claim.get('agent')} active_fr={claim.get('active_fr')} "
                    f"packet={claim.get('packet_dir')}"
                )
            else:
                print("claim: none")
            return 0
        raise HandoffError(f"unknown command {args.command}")
    except HandoffError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
