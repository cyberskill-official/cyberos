"""cyberos-cuo CLI — route natural-language requests to skills.

Subcommands:
    catalog            list every skill the router can dispatch to.
    route <query>      decide which skill to invoke; optionally invoke it
                       and record the decision in the BRAIN.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict
from pathlib import Path

from cuo.core.catalog import discover
from cuo.core.invoker import invoke
from cuo.core.memory_bridge import record_decision
from cuo.core.router import route


def _module_roots() -> tuple[Path, Path]:
    """Locate skill/skills/ and memory/ relative to this script.

    cuo/ sits next to skill/ and memory/. __main__.py is at
    cuo/cuo/__main__.py — the cyberos repo root is two parents above.
    """
    here = Path(__file__).resolve()
    repo = here.parents[2]
    return repo / "skill" / "skills", repo / "memory"


def cmd_catalog(_args: argparse.Namespace) -> int:
    skill_root, _ = _module_roots()
    entries = discover(skill_root)
    if not entries:
        print(f"(no skills found under {skill_root})", file=sys.stderr)
        return 1
    for e in entries:
        print(f"{e.name:<32}  {e.description[:80]}")
    return 0


def cmd_route(args: argparse.Namespace) -> int:
    skill_root, memory_root = _module_roots()
    catalog = discover(skill_root)
    decision = route(args.query, catalog)
    if decision is None:
        out = {"routed": False, "reason": "no skill matched with sufficient confidence"}
        print(json.dumps(out, ensure_ascii=False))
        return 1
    decision_dict = asdict(decision)
    out: dict = {"routed": True, "decision": decision_dict}
    if args.invoke:
        primary_input = decision.arguments.get("input")
        result = invoke(decision.skill_name, primary_input, skill_root)
        out["result"] = asdict(result)
        if args.record:
            recorded = record_decision(decision_dict, asdict(result), memory_root)
            out["recorded_at"] = str(recorded)
    print(json.dumps(out, indent=2, ensure_ascii=False))
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(prog="cyberos-cuo", description="CyberOS CUO routing CLI")
    sub = ap.add_subparsers(dest="cmd", required=True)

    sp = sub.add_parser("catalog", help="list available skills the router can dispatch to")
    sp.set_defaults(fn=cmd_catalog)

    sp = sub.add_parser("route", help="route a natural-language request to a skill")
    sp.add_argument("query")
    sp.add_argument("--invoke", action="store_true", help="actually call the chosen skill")
    sp.add_argument("--record", action="store_true",
                    help="record the decision in BRAIN (requires --invoke)")
    sp.set_defaults(fn=cmd_route)

    args = ap.parse_args()
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
