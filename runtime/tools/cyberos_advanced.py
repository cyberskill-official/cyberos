#!/usr/bin/env python3
"""
cyberos_advanced.py — Stage 8 future-state scaffolds (Batch 20).

Five forward-looking capabilities, scaffolded as working primitives so
the operator can exercise them today without committing to the full
build:

  S8.1 fr-council <FR-id>          — apply council mode (4 voices) at the
                                      FR layer; reuses cyberos_council
                                      heuristic context + voice prompts
  S8.2 auto-decompose <task-id>    — emit a runtime_spec for a task:
                                      bash commands + file edits + validate
                                      steps so an autonomous agent can run
                                      it without humans
  S8.3 client-chain run            — solo profile is internal-default; this
                                      is the inverse: client-visible chain
                                      with locked persona separation,
                                      consent gates, and EU AI Act high-risk
                                      classification by default
  S8.4 continuous-replan           — nightly: re-evaluates current backlog
                                      against new drift candidates +
                                      rejected items, proposes next sprint
  S8.5 skill-marketplace           — read/write a manifest at
                                      ~/.cyberos/skill-marketplace.json
                                      listing community skills to install

All five are scaffolds: they produce structured outputs the operator
reviews before any real action. None of them touches the BRAIN without
explicit confirmation.
"""
from __future__ import annotations
import argparse
import json
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


# ---- S8.1 — FR-layer council ---------------------------------------------

def cmd_fr_council(args):
    """Apply council mode to an FR. Reuses cyberos_council's voice templates."""
    brain_root = find_brain()
    # Find the FR
    fr_path = None
    for d in (brain_root / "planning", brain_root / ".cyberos-memory" / "memories" / "projects"):
        if not d.exists():
            continue
        for md in d.rglob(f"{args.fr_id}-*.md"):
            fr_path = md
            break
        if fr_path:
            break
    if not fr_path:
        print(f"  no FR matching {args.fr_id!r}", file=sys.stderr); return 2

    text = fr_path.read_text(encoding="utf-8")
    out_dir = brain_root / "outputs" / "council"
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / f"{args.fr_id}-council.md"

    body = "\n".join([
        f"# FR-council — {args.fr_id}",
        f"",
        f"**Source**: `{fr_path.relative_to(brain_root)}`",
        f"**Generated**: {datetime.now(ICT).isoformat(timespec='seconds')}",
        f"",
        f"## Voices",
        f"",
        f"### Architect",
        f"_Prompt:_ Evaluate this FR's tasks for structural fit. Are dependencies acyclic? Are sizes right-sized? Does the task graph parallelise well?",
        f"_Council finding:_ _(paste Claude's response here, or feed cyberos council --run-now)_",
        f"",
        f"### Skeptic",
        f"_Prompt:_ What worst-case failure mode does this FR enable? What's a missing edge case?",
        f"_Council finding:_ _(...)_",
        f"",
        f"### Pragmatist",
        f"_Prompt:_ What's the 80/20 cut? Which tasks could move to a follow-up FR without losing core value?",
        f"_Council finding:_ _(...)_",
        f"",
        f"### Critic",
        f"_Prompt:_ Audit each task's `description` for ≥200 chars + concrete acceptance test. Any task that fails: name + fix.",
        f"_Council finding:_ _(...)_",
        f"",
        f"## Synthesis",
        f"",
        f"_Operator fills:_ Verdict (ACCEPT / ACCEPT-WITH-MODS / REJECT) + decision rationale.",
        f"",
        f"---",
        f"",
        f"## FR body (for reference)",
        f"",
        text,
    ])
    out_path.write_text(body, encoding="utf-8")
    print(f"  ✓ FR-council staged: {out_path.relative_to(brain_root)}")
    print(f"  Feed each voice prompt to a fresh Claude conversation, or run --run-now (TODO)")
    return 0


# ---- S8.2 — Auto-decompose into runtime_spec -----------------------------

def cmd_auto_decompose(args):
    """Emit a runtime_spec for a task: structured agent-runnable steps."""
    brain_root = find_brain()
    # Parse task id: FR-NNN-T-MM
    m = re.match(r"^(FR-\d+)-T-(\d+)$", args.task_id)
    if not m:
        print(f"  task_id must be FR-NNN-T-MM; got {args.task_id!r}", file=sys.stderr); return 2
    fr_id = m.group(1)
    # Find FR
    fr_path = None
    for d in (brain_root / "planning", brain_root / ".cyberos-memory" / "memories" / "projects"):
        if not d.exists():
            continue
        for md in d.rglob(f"{fr_id}-*.md"):
            fr_path = md; break
        if fr_path:
            break
    if not fr_path:
        print(f"  no FR matching {fr_id!r}", file=sys.stderr); return 2

    # Extract tasks via yaml
    try:
        import yaml
    except ImportError:
        print(f"  pyyaml required", file=sys.stderr); return 3
    text = fr_path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        print(f"  FR lacks frontmatter", file=sys.stderr); return 2
    end = text.find("\n---\n", 4)
    fm = yaml.safe_load(text[4:end])
    tasks = fm.get("tasks") or []
    task = next((t for t in tasks if t.get("id") == args.task_id), None)
    if not task:
        print(f"  no task {args.task_id} in {fr_id}", file=sys.stderr); return 2

    # Emit runtime_spec
    runtime_spec = {
        "task_id": args.task_id,
        "agent_profile": task.get("agent_profile", "claude-sonnet-4-6"),
        "steps": [
            {"step": 1, "kind": "read",  "target": "the task description + preconditions",
             "purpose": "context"},
            {"step": 2, "kind": "explore", "target": str(fr_path.relative_to(brain_root)),
             "purpose": "load the surrounding FR for cross-references"},
            {"step": 3, "kind": "act", "target": "produce each deliverable",
             "purpose": "perform the work — one tool call per deliverable",
             "deliverables": task.get("deliverables") or []},
            {"step": 4, "kind": "verify", "target": task.get("acceptance_test", {}).get("shell") or task.get("acceptance_test", {}).get("assertion"),
             "purpose": "run the acceptance test and report rc"},
            {"step": 5, "kind": "report", "target": "task status (done / blocked / needs-human)",
             "purpose": "emit a structured report"},
        ],
        "budget": {"max_tokens": task.get("estimated_tokens", 10000)},
        "abort_conditions": [
            "ENOSPC during write",
            "acceptance test rc > 0 after 3 attempts",
            "any tool call requires escalated perms",
        ],
    }
    out = brain_root / "outputs" / "runtime-specs" / f"{args.task_id}.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(runtime_spec, indent=2) + "\n")
    print(f"  ✓ runtime_spec for {args.task_id}: {out.relative_to(brain_root)}")
    return 0


# ---- S8.3 — Client-facing chain stub -------------------------------------

def cmd_client_chain(args):
    """Run the FULL chain (CPO + CTO separation) for client-visible work."""
    brain_root = find_brain()
    # This is the inverse of `solo` — force the `full` profile for client work
    print(f"\n  client-chain — invokes `cyberos chain run --profile full` with these locks:")
    print(f"   - client_visible: true (forced)")
    print(f"   - confidentiality: client_confidential (default; --regulated to override)")
    print(f"   - chain_profile: full (no solo / lean / standard allowed)")
    print(f"   - persona separation enforced (CPO + CTO + CSecO + CLO trails distinct)")
    print(f"\n  To run for real:")
    print(f"    cyberos chain run --pitch \"...\" --profile full")
    print(f"\n  Client-chain artefacts emitted at planning/<date>-CLIENT-<slug>/")
    return 0


# ---- S8.4 — Continuous re-planning ---------------------------------------

def cmd_replan(args):
    """Nightly re-plan: drift candidates + rejected items → next-sprint proposal."""
    brain_root = find_brain()
    brain = brain_root / ".cyberos-memory"
    proposals = []

    drift_dir = brain / "memories" / "drift"
    rejected_dir = brain / "rejected"

    if drift_dir.exists():
        for d in drift_dir.glob("*.md"):
            text = d.read_text(encoding="utf-8", errors="ignore")
            if "## Resolution" not in text:
                first_line = (text.splitlines() or [""])[0][:80]
                proposals.append({"source": "drift", "path": str(d.relative_to(brain_root)),
                                  "summary": first_line})

    if rejected_dir.exists():
        for r in sorted(rejected_dir.glob("*.md"))[:5]:
            # Old rejected items might now be worth revisiting
            try:
                mtime = datetime.fromtimestamp(r.stat().st_mtime, tz=ICT)
                age_days = (datetime.now(ICT) - mtime).days
                if age_days >= 90:  # 3+ months old; worth re-checking
                    first_line = r.read_text(encoding="utf-8").splitlines()[0][:80]
                    proposals.append({"source": "rejected-stale", "path": str(r.relative_to(brain_root)),
                                      "age_days": age_days, "summary": first_line})
            except Exception:
                continue

    out = brain_root / "outputs" / "replan" / f"{datetime.now(ICT).strftime('%Y-%m-%d')}-proposal.md"
    out.parent.mkdir(parents=True, exist_ok=True)

    lines = [
        f"# Replan proposal — {datetime.now(ICT).isoformat(timespec='minutes')}",
        f"",
        f"## Drift candidates ({sum(1 for p in proposals if p['source']=='drift')})",
        f"",
    ]
    for p in proposals:
        if p["source"] == "drift":
            lines.append(f"- `{p['path']}` — {p['summary']}")
    lines.append("")
    lines.append(f"## Re-eligible rejected items (3+ months old, {sum(1 for p in proposals if p['source']=='rejected-stale')})")
    lines.append("")
    for p in proposals:
        if p["source"] == "rejected-stale":
            lines.append(f"- `{p['path']}` (age {p['age_days']}d) — {p['summary']}")
    lines.append("")
    lines.append(f"## Action")
    lines.append(f"")
    lines.append(f"Review each item. For drift candidates, decide: ship REF, defer, or reject.")
    lines.append(f"For re-eligible rejected items, decide: re-open or close-with-rationale.")

    out.write_text("\n".join(lines) + "\n")
    print(f"  ✓ replan proposal: {out.relative_to(brain_root)}")
    print(f"    drift: {sum(1 for p in proposals if p['source']=='drift')}, rejected-stale: {sum(1 for p in proposals if p['source']=='rejected-stale')}")
    return 0


# ---- S8.5 — Skill marketplace --------------------------------------------

def marketplace_path() -> Path:
    return Path.home() / ".cyberos" / "skill-marketplace.json"


def cmd_marketplace(args):
    """Read/write a community skill registry."""
    mp = marketplace_path()
    if args.action == "list":
        if not mp.exists():
            print("  marketplace empty (run `add` to register a skill)")
            return 0
        data = json.loads(mp.read_text())
        print(f"\n  {len(data.get('skills', []))} skill(s) registered:")
        for s in data["skills"]:
            print(f"    {s['name']:24s}  v{s.get('version', '?')}  by {s.get('author', '?')}  src={s.get('source', '?')}")
        return 0
    elif args.action == "add":
        mp.parent.mkdir(parents=True, exist_ok=True)
        data = json.loads(mp.read_text()) if mp.exists() else {"skills": []}
        entry = {
            "name": args.name,
            "version": args.version or "0.1.0",
            "author": args.author or "unknown",
            "source": args.source or "local",
            "registered_at": datetime.now(ICT).isoformat(timespec="seconds"),
        }
        # Replace if exists
        data["skills"] = [s for s in data["skills"] if s["name"] != args.name]
        data["skills"].append(entry)
        mp.write_text(json.dumps(data, indent=2) + "\n")
        print(f"  ✓ registered {args.name} v{entry['version']}")
        return 0
    elif args.action == "install":
        print(f"  ⚠ install is a scaffold: would clone {args.name} from its source URL into docs/skills/community/{args.name}/")
        print(f"  Today, do this manually: git clone <src> docs/skills/community/{args.name}")
        return 0


def main():
    p = argparse.ArgumentParser(description="Stage 8 advanced/future capabilities (Batch 20)")
    sub = p.add_subparsers(dest="cmd", required=True)
    pc = sub.add_parser("fr-council"); pc.add_argument("fr_id"); pc.set_defaults(func=cmd_fr_council)
    pd = sub.add_parser("auto-decompose"); pd.add_argument("task_id"); pd.set_defaults(func=cmd_auto_decompose)
    sub.add_parser("client-chain").set_defaults(func=cmd_client_chain)
    sub.add_parser("replan").set_defaults(func=cmd_replan)
    pm = sub.add_parser("marketplace")
    pm.add_argument("action", choices=["list", "add", "install"])
    pm.add_argument("name", nargs="?")
    pm.add_argument("--version")
    pm.add_argument("--author")
    pm.add_argument("--source")
    pm.set_defaults(func=cmd_marketplace)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
