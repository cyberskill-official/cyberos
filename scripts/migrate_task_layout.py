#!/usr/bin/env python3
"""TASK-DOCS-004: migrate docs/tasks to folder-per-FR layout.

<module>/FR-STEM.md        -> <module>/FR-STEM/spec.md
<module>/FR-STEM.audit.md  -> <module>/FR-STEM/audit.md
FR-STEM.md (root-level)    -> <module>/FR-STEM/spec.md   (module = frontmatter `module:`,
                              else the FR id segment FR-<SEG>-..., else `misc`)

History-preserving (plain renames; git detects), idempotent (re-run = no-op),
per-module summary. assets/ folders are created on demand by authors, never here.
Also rewrites one level of relative citations inside moved specs and repo-wide
live references to the old flat paths (.workflow/ and _audits/ untouched).
"""
import os, re, sys
from pathlib import Path

import subprocess
def _default_root():
    try:
        return Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True,
                                   text=True, check=True).stdout.strip())
    except Exception:
        return Path.cwd()
# portable (FR 1.0.0 kit): --root <dir> targets any repo; default = git toplevel of cwd, falling
# back to this script's repo when invoked in-tree.
if "--root" in sys.argv:
    ROOT = Path(sys.argv[sys.argv.index("--root") + 1]).resolve()
elif (Path(__file__).resolve().parent.parent / "docs" / "tasks").is_dir() and _default_root() == Path(__file__).resolve().parent.parent:
    ROOT = Path(__file__).resolve().parent.parent
else:
    ROOT = _default_root()
FR = ROOT / "docs" / "tasks"

FM_FENCE = re.compile(r"\A---\n(.*?\n)---\n", re.S)

def _sanitize_mod(s):
    s = re.sub(r"[^a-z0-9-]+", "-", s.strip().lower()).strip("-")
    return s or "misc"

def _module_for(f):
    """Module for a ROOT-level flat FR: frontmatter `module:` > FR id segment > misc."""
    try:
        text = f.read_text()
    except (UnicodeDecodeError, OSError):
        text = ""
    m = FM_FENCE.match(text)
    if m:
        mm = re.search(r"^module:\s*(.+?)\s*$", m.group(1), re.M)
        if mm:
            v = re.sub(r"\s+#.*$", "", mm.group(1)).strip().strip("\"'")
            if v:
                return _sanitize_mod(v)
    mid = re.match(r"FR-([A-Za-z0-9]+)-", f.name)
    return _sanitize_mod(mid.group(1)) if mid else "misc"

def migrate():
    moved, summary, renames = 0, {}, []
    # 1) root-level flat FRs (docs/tasks/FR-*.md) - relocate into a module folder
    #    so the folder-per-FR rule AND the status hub (which scans <module>/FR-*/spec.md) see them.
    for f in sorted(FR.glob("FR-*.md")):
        if not f.is_file():
            continue
        is_audit = f.name.endswith(".audit.md")
        stem = f.name[:-len(".audit.md")] if is_audit else f.stem
        mod = _module_for(f)
        target = FR / mod / stem / ("audit.md" if is_audit else "spec.md")
        if target.exists():
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        os.rename(f, target)
        renames.append((f.name, f"{mod}/{stem}/{'audit.md' if is_audit else 'spec.md'}"))
        moved += 1; summary[mod] = summary.get(mod, 0) + 1
    # 2) module-level flat FRs (<module>/FR-*.md)
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
        if n: summary[mod_dir.name] = summary.get(mod_dir.name, 0) + n
    return moved, summary, renames

REF_MD = re.compile(r"(docs/tasks/)([a-z][a-z0-9-]*/)(FR-[A-Za-z0-9-]+?)(\.audit\.md|\.md)")
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

def sweep(renames=()):
    changed = 0
    denied.clear()
    # root-level renames rewrite by exact string (repo-relative form everywhere; bare
    # filenames only inside files that sit directly in the FR root, e.g. BACKLOG.md).
    exact = [(f"docs/tasks/{old}", f"docs/tasks/{new}") for old, new in renames]
    # live trees only (same doctrine as TASK-SKILL-119: archives/changelogs excluded)
    roots = ["docs/tasks", "modules", "tools", "scripts", ".github", "AGENTS.md", "CLAUDE.md"]
    skip = ("/.workflow/", "/_audits/", "/_archive/", "CHANGELOG.md", "appendices.md", "runners/README.md")
    for r in roots:
        p = ROOT / r
        files = [p] if p.is_file() else [f for f in p.rglob("*") if f.is_file() and f.suffix in (".md", ".sh", ".py", ".mjs", ".yml", ".yaml", ".json")]
        for f in files:
            rp = str(f)
            if any(s in rp for s in skip): continue
            try: s = f.read_text()
            except (UnicodeDecodeError, OSError): continue
            t = s
            for old, new in exact:
                if old in t: t = t.replace(old, new)
            t = REF_MD.sub(new_ref, t)
            if "/docs/tasks/" not in rp:
                pass
            else:
                if f.parent == FR:
                    for old, new in renames:
                        if old in t: t = t.replace(old, new)
                t = REL_SIB.sub(new_rel, t)
                t = SAME_DIR.sub(new_same, t)
            if t != s:
                try:
                    f.write_text(t); changed += 1
                except PermissionError:
                    denied.append(str(f.relative_to(ROOT)))
    return changed

if __name__ == "__main__":
    moved, summary, renames = migrate()
    swept = sweep(renames) if moved or "--sweep" in sys.argv else 0
    if moved == 0:
        print("fr-layout: nothing to do (already folder-per-FR)")
    else:
        mods = ", ".join(f"{k}:{v}" for k, v in sorted(summary.items()))
        print(f"fr-layout: moved {moved} files across {len(summary)} modules ({mods}); {swept} files re-referenced")
    if denied:
        print(f"fr-layout: WARN {len(denied)} write-protected files skipped (fix perms + re-run --sweep):")
        for d in denied: print(f"  {d}")
