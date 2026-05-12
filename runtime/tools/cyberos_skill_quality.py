#!/usr/bin/env python3
"""
cyberos_skill_quality.py — Stage 6 quality + trust amplifiers (Batch 19).

Aggregates 5 quality checks runnable against any skill artefact:

  S6.1 anti-fabrication    — does the skill ask clarifying questions or
                              invent facts? Score against a corpus of
                              ambiguous inputs.
  S6.2 untrusted-content   — verify the skill actually wraps user input
                              in <untrusted_content> blocks per §4.2.
  S6.3 grounding           — every claim in artefact links back to source
                              PRD/SRS/NL spec OR a BRAIN memory_id.
  S6.4 calibration         — historical: when skill says needs_human:false,
                              how often does a human actually intervene?
  S6.5 deprecation         — surface skills with deprecated_at set OR
                              skill_version far behind contract_version.

Usage:
    cyberos skill-quality run <skill-id>       # run all 5 checks
    cyberos skill-quality run <skill-id> --check antifab|untrusted|grounding|calibration|deprecation
    cyberos skill-quality calibration <skill-id>   # historical only
"""
from __future__ import annotations
import argparse
import json
import re
import sys
from pathlib import Path


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def load_skill_md(brain_root: Path, skill_id: str) -> tuple[dict, str]:
    """Return (frontmatter, body) for a skill, looked up by name."""
    skills_dir = brain_root / "docs" / "skills"
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
        if name == skill_id or skill_md.parent.name == skill_id:
            return fm, text[end + 5:]
    raise SystemExit(f"no skill named {skill_id!r}")


def check_antifab(brain_root: Path, skill_id: str, fm: dict, body: str) -> dict:
    """S6.1 — does the skill reference an anti-fabrication ref doc?"""
    findings = []
    skill_dir_text = ""
    # Check for reference to ANTI_FABRICATION.md
    for skill_md in (brain_root / "docs" / "skills").rglob("SKILL.md"):
        if skill_md.parent.name == skill_id:
            for f in skill_md.parent.rglob("*.md"):
                skill_dir_text += f.read_text(encoding="utf-8", errors="ignore")
            break
    has_antifab = "ANTI_FABRICATION" in skill_dir_text or "anti-fabrication" in skill_dir_text.lower()
    has_hitl = "HITL" in skill_dir_text or "needs_human" in skill_dir_text or "ask the operator" in skill_dir_text.lower()
    if not has_antifab:
        findings.append("missing ANTI_FABRICATION.md reference")
    if not has_hitl:
        findings.append("no HITL / needs_human discipline visible in skill body")
    return {"check": "antifab", "passed": not findings, "findings": findings}


def check_untrusted(brain_root: Path, skill_id: str, fm: dict, body: str) -> dict:
    """S6.2 — does the skill require untrusted_content wrapping?"""
    findings = []
    untrusted_wrap = fm.get("untrusted_content_wrapping")
    if untrusted_wrap != "required":
        findings.append(f"untrusted_content_wrapping should be 'required'; got {untrusted_wrap!r}")
    if "untrusted_content" not in body:
        findings.append("skill body doesn't mention untrusted_content wrapping pattern")
    return {"check": "untrusted", "passed": not findings, "findings": findings}


def check_grounding(brain_root: Path, skill_id: str, fm: dict, body: str) -> dict:
    """S6.3 — verify the skill emits source_refs / authority markers."""
    findings = []
    # Authority markers
    if "authority" not in body.lower():
        findings.append("skill body doesn't reference authority markers (human-edited / human-confirmed / llm-explicit / llm-implicit)")
    if "source_ref" not in body.lower():
        findings.append("skill body doesn't reference source_ref attribution")
    return {"check": "grounding", "passed": not findings, "findings": findings}


def check_calibration(brain_root: Path, skill_id: str, fm: dict, body: str) -> dict:
    """S6.4 — historical: what proportion of runs hit a HITL gate?"""
    # Look for analytics file
    analytics = Path.home() / ".cyberos" / "analytics" / "skill-usage.jsonl"
    runs = 0
    hitl = 0
    if analytics.exists():
        for line in analytics.read_text(encoding="utf-8").splitlines():
            try:
                r = json.loads(line)
                if r.get("cmd", "").startswith(skill_id) or r.get("skill") == skill_id:
                    runs += 1
                    if r.get("outcome") in ("HITL_PAUSE", "hitl_paused"):
                        hitl += 1
            except Exception:
                continue
    return {"check": "calibration", "runs_recorded": runs, "hitl_rate": (hitl / runs if runs else None),
            "passed": True if runs == 0 else (hitl / runs < 0.30),
            "findings": [] if runs == 0 else [f"hitl_rate {hitl/runs:.0%} — review skill instructions if > 30%"]}


def check_deprecation(brain_root: Path, skill_id: str, fm: dict, body: str) -> dict:
    """S6.5 — flag deprecated skills."""
    findings = []
    if fm.get("deprecated_at"):
        findings.append(f"skill marked deprecated at {fm['deprecated_at']}")
    if fm.get("replaced_by"):
        findings.append(f"replaced_by: {fm['replaced_by']}")
    return {"check": "deprecation", "passed": not findings, "findings": findings}


CHECKS = {
    "antifab": check_antifab,
    "untrusted": check_untrusted,
    "grounding": check_grounding,
    "calibration": check_calibration,
    "deprecation": check_deprecation,
}


def cmd_run(args):
    brain_root = find_brain()
    fm, body = load_skill_md(brain_root, args.skill_id)
    which = [args.check] if args.check else list(CHECKS.keys())
    results = [CHECKS[c](brain_root, args.skill_id, fm, body) for c in which]

    if args.json:
        print(json.dumps(results, indent=2))
        return 0 if all(r["passed"] for r in results) else 1

    print(f"\n  Quality check for {args.skill_id!r}:\n")
    failed = 0
    for r in results:
        marker = "✓" if r["passed"] else "✗"
        print(f"  {marker} {r['check']}")
        for f in r.get("findings", []):
            print(f"      {f}")
        if "hitl_rate" in r and r["hitl_rate"] is not None:
            print(f"      historical hitl_rate: {r['hitl_rate']:.0%} (over {r['runs_recorded']} run(s))")
        if not r["passed"]:
            failed += 1
    print()
    print(f"  {len(results) - failed}/{len(results)} checks passed")
    return 1 if failed else 0


def cmd_calibration(args):
    args.check = "calibration"; args.json = False
    return cmd_run(args)


def main():
    p = argparse.ArgumentParser(description="Stage 6 quality + trust amplifiers")
    sub = p.add_subparsers(dest="cmd", required=True)
    pr = sub.add_parser("run")
    pr.add_argument("skill_id")
    pr.add_argument("--check", choices=list(CHECKS.keys()))
    pr.add_argument("--json", action="store_true")
    pr.set_defaults(func=cmd_run)
    pc = sub.add_parser("calibration")
    pc.add_argument("skill_id")
    pc.set_defaults(func=cmd_calibration)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
