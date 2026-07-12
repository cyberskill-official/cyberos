#!/usr/bin/env python3
"""FR-DOCS-004: migrate docs/feature-requests to folder-per-FR layout.

<module>/FR-STEM.md        -> <module>/FR-STEM/spec.md
<module>/FR-STEM.audit.md  -> <module>/FR-STEM/audit.md

History-preserving (plain renames; git detects), idempotent (re-run = no-op),
per-module summary. assets/ folders are created on demand by authors, never here.
Also rewrites one level of relative citations inside moved specs and repo-wide
live references to the old flat paths (.workflow/ and _audits/ untouched).
"""
import os, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FR = ROOT / "docs" / "feature-requests"

def migrate():
    moved, summary = 0, {}
    for mod_dir in sorted(p for p in FR.iterdir() if p.is_dir() and not p.name.startswith(("_", "."))):
        n = 0
        for f in sorted(mod_dir.glob("FR-*.md")):
            stem = f.name[:-len(".audit.md")] if f.name.endswith(".audit.md") else f.stem
            target_dir = mod_dir / stem
            target = target_dir / ("audit.md" if f.name.endswith(".audit.md") else "spec.md")
            if target.exists():
                continue
            target_dir.mkdir(exist_ok=True)
            os.rename(f, target)
            moved += 1; n += 1
        if n: summary[mod_dir.name] = n
    return moved, summary

REF_MD = re.compile(r"(docs/feature-requests/)([a-z][a-z0-9-]*/)(FR-[A-Za-z0-9-]+?)(\.audit\.md|\.md)")
def new_ref(m):
    kind = "audit.md" if m.group(4) == ".audit.md" else "spec.md"
    return f"{m.group(1)}{m.group(2)}{m.group(3)}/{kind}"

denied = []
SAME_DIR = re.compile(r"(\]\(|\`)(FR-[A-Za-z0-9-]+?)(\.audit\.md|\.md)(\)|\`)")
def new_same(m):
    kind = "audit.md" if m.group(3) == ".audit.md" else "spec.md"
    return f"{m.group(1)}{m.group(2)}/{kind}{m.group(4)}"
REL_SIB = re.compile(r"(\]\()(\.\./)([a-z][a-z0-9-]*/)(FR-[A-Za-z0-9-]+?)(\.audit\.md|\.md)(\))")
def new_rel(m):  # inside a moved spec, ../<mod>/FR-x.md is now two levels up
    kind = "audit.md" if m.group(5) == ".audit.md" else "spec.md"
    return f"{m.group(1)}../../{m.group(3)}{m.group(4)}/{kind}{m.group(6)}"

def sweep():
    changed = 0
    denied.clear()
    # live trees only (same doctrine as FR-SKILL-119: archives/changelogs excluded)
    roots = ["docs/feature-requests", "modules", "tools", "scripts", ".github", "AGENTS.md", "CLAUDE.md"]
    skip = ("/.workflow/", "/_audits/", "/_archive/", "CHANGELOG.md", "appendices.md", "runners/README.md")
    for r in roots:
        p = ROOT / r
        files = [p] if p.is_file() else [f for f in p.rglob("*") if f.is_file() and f.suffix in (".md", ".sh", ".py", ".mjs", ".yml", ".yaml", ".json")]
        for f in files:
            rp = str(f)
            if any(s in rp for s in skip): continue
            try: s = f.read_text()
            except (UnicodeDecodeError, OSError): continue
            t = REF_MD.sub(new_ref, s)
            if "/docs/feature-requests/" not in rp:
                pass
            else:
                t = REL_SIB.sub(new_rel, t)
                t = SAME_DIR.sub(new_same, t)
            if t != s:
                try:
                    f.write_text(t); changed += 1
                except PermissionError:
                    denied.append(str(f.relative_to(ROOT)))
    return changed

if __name__ == "__main__":
    moved, summary = migrate()
    swept = sweep() if moved or "--sweep" in sys.argv else 0
    if moved == 0:
        print("fr-layout: nothing to do (already folder-per-FR)")
    else:
        mods = ", ".join(f"{k}:{v}" for k, v in sorted(summary.items()))
        print(f"fr-layout: moved {moved} files across {len(summary)} modules ({mods}); {swept} files re-referenced")
    if denied:
        print(f"fr-layout: WARN {len(denied)} write-protected files skipped (fix perms + re-run --sweep):")
        for d in denied: print(f"  {d}")
