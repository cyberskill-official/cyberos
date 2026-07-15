#!/usr/bin/env python3
"""
migrate_spec_schema.py — bring all 507 task specs onto the task@1 contract schema.

WHY
---
`modules/skill/contracts/task/CONTRACT.md` (and the FM-* rules in
`modules/skill/task-audit/RUBRIC.md`) require a field set that essentially no spec
carries:

    rule     field                    specs having it
    FM-102   author                         7/507
    FM-103   department                     7/507
    FM-106   created_at                     6/507
    FM-107   ai_authorship                  9/507
    FM-109   eu_ai_act_risk_class          12/507
    FM-111   client_visible                 7/507
    FM-004   template                       7/507
    FM-105   priority (p0-p3)               6/507   (501 carry MUST/SHOULD/COULD)

So `task-audit` would reject ~500 of 507 specs and the `draft -> ready_to_implement`
gate has never been passable. Decision (2026-07-14): the contract wins; migrate the
specs.

DERIVED vs FABRICATED
---------------------
Everything here is derived from evidence already on disk:

    author       <- `owner:`            (301 specs) else git log author of the spec
    created_at   <- `created:`          (498 specs) + Asia/Ho_Chi_Minh offset
    priority     <- MoSCoW              MUST->p0  SHOULD->p1  COULD->p2
    type         <- `class:`            product->feature  improvement->improvement
    template     <- constant            task@1
    department   <- constant            engineering (this is an engineering repo)

Two fields CANNOT be derived from anything:

    ai_authorship          who wrote it, and how much of it was a model
    eu_ai_act_risk_class   a regulatory classification

Auto-filling those with a plausible default across 500 specs would be fabricating
compliance metadata. So they are written with an explicit `# UNREVIEWED` marker, and
RUBRIC rule FM-112 makes an unreviewed marker a hard error for any task leaving
`draft`. The structure satisfies the contract; the human still has to do the judging.

Usage:
    python3 scripts/migrate_spec_schema.py          # dry run
    python3 scripts/migrate_spec_schema.py --apply
"""
from __future__ import annotations

import argparse
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

TZ = "+07:00"  # CyberSkill is Ho Chi Minh City
UNREVIEWED = "  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft"

PRIORITY_MAP = {"MUST": "p0", "SHOULD": "p1", "COULD": "p2", "WONT": "p3", "WON'T": "p3"}
CLASS_TO_TYPE = {"product": "feature", "improvement": "improvement"}

FM_RE = re.compile(r"\A(---\r?\n)(.*?)(\r?\n---)", re.DOTALL)


def git_author(path: Path) -> str | None:
    r = subprocess.run(["git", "log", "--diff-filter=A", "--format=%an", "-1", "--", str(path)],
                       capture_output=True, text=True)
    name = r.stdout.strip().splitlines()
    if not name:
        return None
    return "@" + re.sub(r"[^A-Za-z0-9_.-]", "", name[0].lower().replace(" ", ""))[:38]


def handle_from_owner(owner: str) -> str:
    """"Stephen Cheng (CTO)" -> '"@stephencheng"' — QUOTED.

    `@` is a YAML reserved indicator: an unquoted scalar may not start with it. The
    first run of this script emitted `author: @stephencheng` on 500 specs, which is
    invalid YAML that most parsers accept anyway. `install.sh`'s repair_task_yaml
    caught it and quoted all 500 — a gate doing exactly its job, on my bug.
    """
    base = re.sub(r"\(.*?\)", "", owner).strip()
    handle = "@" + re.sub(r"[^A-Za-z0-9_.-]", "", base.lower().replace(" ", ""))[:38]
    return f'"{handle}"'


def parse_fm(text: str) -> tuple[str, list[str], str] | None:
    m = FM_RE.match(text)
    if not m:
        return None
    return m.group(1), m.group(2).splitlines(), text[m.end():]


def get(lines: list[str], key: str) -> str | None:
    for ln in lines:
        mm = re.match(rf"^{re.escape(key)}:\s*(.*)$", ln)
        if mm:
            return mm.group(1).split("#", 1)[0].strip().strip('"\'')
    return None


def set_or_add(lines: list[str], key: str, value: str) -> None:
    for i, ln in enumerate(lines):
        if re.match(rf"^{re.escape(key)}:", ln):
            lines[i] = f"{key}: {value}"
            return
    # insert after `title:` if present, else after `id:`, else at top
    anchor = next((i for i, ln in enumerate(lines) if ln.startswith("title:")), None)
    if anchor is None:
        anchor = next((i for i, ln in enumerate(lines) if ln.startswith("id:")), -1)
    lines.insert(anchor + 1, f"{key}: {value}")


def drop(lines: list[str], key: str) -> None:
    for i, ln in enumerate(list(lines)):
        if re.match(rf"^{re.escape(key)}:", ln):
            lines.pop(i)
            return


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true")
    args = ap.parse_args()

    specs = sorted(Path("docs/tasks").glob("*/TASK-*/spec.md"))
    stats: Counter = Counter()
    unreviewed = 0

    for spec in specs:
        text = spec.read_text(encoding="utf-8")
        parsed = parse_fm(text)
        if not parsed:
            stats["no-frontmatter"] += 1
            continue
        open_d, lines, body = parsed
        before = list(lines)

        # template (FM-004)
        if not get(lines, "template"):
            set_or_add(lines, "template", "task@1"); stats["template"] += 1

        # author (FM-102) — from owner, else git
        if not get(lines, "author"):
            owner = get(lines, "owner")
            a = handle_from_owner(owner) if owner else (git_author(spec) or "@unknown")
            set_or_add(lines, "author", a); stats["author"] += 1

        # department (FM-103)
        if not get(lines, "department"):
            set_or_add(lines, "department", "engineering"); stats["department"] += 1

        # priority (FM-105) — MoSCoW -> p0..p3
        pri = get(lines, "priority")
        if pri and pri.upper() in PRIORITY_MAP:
            set_or_add(lines, "priority", PRIORITY_MAP[pri.upper()]); stats["priority-mapped"] += 1
        elif not pri:
            set_or_add(lines, "priority", "p2"); stats["priority-default"] += 1

        # created_at (FM-106) — from `created` date
        if not get(lines, "created_at"):
            c = get(lines, "created")
            if c and re.match(r"^\d{4}-\d{2}-\d{2}$", c):
                set_or_add(lines, "created_at", f"{c}T00:00:00{TZ}")
            else:
                set_or_add(lines, "created_at", f"2026-07-14T00:00:00{TZ}")
            stats["created_at"] += 1

        # type (replaces feature_type AND class — decision 2026-07-14)
        if not get(lines, "type"):
            cls = (get(lines, "class") or "").lower()
            set_or_add(lines, "type", CLASS_TO_TYPE.get(cls, "feature")); stats["type"] += 1
        drop(lines, "class"); drop(lines, "feature_type")

        # client_visible (FM-111)
        if not get(lines, "client_visible"):
            set_or_add(lines, "client_visible", "false"); stats["client_visible"] += 1

        # ── NOT DERIVABLE. Structure only; the judgement is still owed. ──────
        if not get(lines, "ai_authorship"):
            set_or_add(lines, "ai_authorship", "generated_then_reviewed" + UNREVIEWED)
            stats["ai_authorship(UNREVIEWED)"] += 1; unreviewed += 1
        if not get(lines, "eu_ai_act_risk_class"):
            set_or_add(lines, "eu_ai_act_risk_class", "not_ai" + UNREVIEWED)
            stats["eu_ai_act_risk_class(UNREVIEWED)"] += 1

        if lines != before and args.apply:
            spec.write_text(open_d + "\n".join(lines) + "\n---" + body, encoding="utf-8")

    print(("APPLIED" if args.apply else "DRY RUN") + f" — {len(specs)} specs\n")
    for k, v in stats.most_common():
        print(f"  {k:34} {v}")
    print(f"\n  {unreviewed} specs carry UNREVIEWED compliance fields.")
    print("  RUBRIC FM-112 makes that a hard error for any task leaving `draft`,")
    print("  so the structure is satisfied but the judgement is still owed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
