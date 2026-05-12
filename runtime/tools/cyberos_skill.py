#!/usr/bin/env python3
"""
cyberos_skill.py — skill registry loader.

Aspect 12.5 of the Layer-1 improvement catalog. SCAFFOLD-LEVEL.

Loads `runtime/tools/skills/registry.json`, exposes 3 verbs:

    cyberos skill list                    # one-line summary per skill
    cyberos skill describe <name>         # full record including §-rules + verb
    cyberos skill chain <name1> <name2>   # describe a hypothetical chain

The "chain" verb does NOT yet execute composed pipelines — that work
belongs in the full skill-orchestration system. For now it surfaces the
dependency graph and warns if invariants conflict (e.g. a non-mutating
skill feeding into a mutating one without a verify step in between).
"""
from __future__ import annotations
import argparse
import json
import sys
from pathlib import Path


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def load_registry(brain_root: Path) -> dict:
    p = brain_root / "runtime" / "tools" / "skills" / "registry.json"
    if not p.exists():
        raise SystemExit(f"no skill registry at {p}")
    return json.loads(p.read_text(encoding="utf-8"))


def discover_docs_skills(brain_root: Path) -> list[dict]:
    """S4.1 — auto-discover skills under docs/skills/cuo/ by reading their SKILL.md frontmatter."""
    out = []
    skills_dir = brain_root / "docs" / "skills"
    if not skills_dir.exists():
        return out
    for skill_md in skills_dir.rglob("SKILL.md"):
        try:
            text = skill_md.read_text(encoding="utf-8")
        except Exception:
            continue
        if not text.startswith("---\n"):
            continue
        end = text.find("\n---\n", 4)
        if end < 0:
            continue
        try:
            import yaml
            fm = yaml.safe_load(text[4:end]) or {}
        except Exception:
            continue
        name = fm.get("name")
        if not name:
            continue
        rel = skill_md.parent.relative_to(brain_root).as_posix()
        out.append({
            "name": name,
            "tool": rel + "/SKILL.md",
            "umbrella_alias": None,
            "verb": "author" if "author" in name else ("audit" if "audit" in name else "process"),
            "mutates_brain": False,  # skills declare scope; runtime decides
            "persona": fm.get("persona", "?"),
            "owner_role": fm.get("owner_role", "?"),
            "version": fm.get("skill_version", "?"),
            "source": "docs/skills",
        })
    return out


def cmd_list(args):
    brain_root = find_brain()
    reg = load_registry(brain_root)
    skills = reg.get("skills", [])
    # S4.1 — augment with discovered chain skills
    chain_skills = discover_docs_skills(brain_root)
    print(f"\n  {len(skills)} operator-tool skill(s) + {len(chain_skills)} chain skill(s):\n")
    print(f"  {'name':22s}  {'verb':14s}  {'mutates':10s}  origin")
    for s in sorted(skills, key=lambda x: x["name"]):
        m = s.get("mutates_brain", False)
        m_label = ("yes" if m is True else ("partial" if isinstance(m, str) else "no"))
        print(f"  {s['name']:22s}  {s.get('verb', '?'):14s}  {m_label:10s}  registry")
    for s in sorted(chain_skills, key=lambda x: x["name"]):
        print(f"  {s['name']:22s}  {s.get('verb', '?'):14s}  {'no':10s}  docs/skills ({s['persona']}/{s['owner_role']})")
    return 0


def cmd_describe(args):
    brain_root = find_brain()
    reg = load_registry(brain_root)
    matches = [s for s in reg["skills"] if s["name"] == args.name]
    if not matches:
        print(f"  no skill named {args.name!r}", file=sys.stderr)
        return 2
    s = matches[0]
    print()
    for key in ("name", "tool", "umbrella_alias", "verb", "invocation_modes",
                "depends_on", "sections", "mutates_brain", "sev_0_ops"):
        if key not in s:
            continue
        v = s[key]
        if isinstance(v, list):
            v = ", ".join(str(x) for x in v) or "—"
        print(f"  {key:18s}  {v}")
    return 0


def cmd_chain(args):
    brain_root = find_brain()
    reg = load_registry(brain_root)
    by_name = {s["name"]: s for s in reg["skills"]}
    chain = args.names
    missing = [n for n in chain if n not in by_name]
    if missing:
        print(f"  unknown skill(s): {missing}", file=sys.stderr)
        return 2
    print()
    print(f"  Chain analysis:")
    last_mutator = None
    warnings = []
    for n in chain:
        s = by_name[n]
        mutates = s.get("mutates_brain", False)
        symbol = "△" if mutates is True else ("◐" if isinstance(mutates, str) else "·")
        deps = ", ".join(s.get("depends_on") or []) or "—"
        print(f"    {symbol} {n:18s}  verb={s.get('verb','?'):16s}  depends_on={deps}")
        if mutates is True and last_mutator and "verify" not in chain[chain.index(last_mutator)+1:chain.index(n)+1]:
            warnings.append(f"chain mutates twice without a verify between {last_mutator} and {n}")
        if mutates is True:
            last_mutator = n
    if warnings:
        print()
        print(f"  ⚠ {len(warnings)} chain warning(s):")
        for w in warnings:
            print(f"    {w}")
        return 1
    print(f"\n  ✓ chain looks safe (no double-mutate without intermediate verify)")
    return 0


def main():
    p = argparse.ArgumentParser(description="skill registry loader (Aspect 12.5)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("list").set_defaults(func=cmd_list)
    pd = sub.add_parser("describe")
    pd.add_argument("name")
    pd.set_defaults(func=cmd_describe)
    pc = sub.add_parser("chain")
    pc.add_argument("names", nargs="+", help="ordered skill names")
    pc.set_defaults(func=cmd_chain)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
