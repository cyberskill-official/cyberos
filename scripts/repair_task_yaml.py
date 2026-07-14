#!/usr/bin/env python3
"""Portable FR frontmatter repairer (TASK-DOCS-004 lineage, vendored for target repos).

Iteratively quotes exactly the line strict-YAML trips on (minimal formatting-only edits,
never value semantics). Usage: repair_fr_yaml.py [--root <dir>] [--glob '*/FR-*/spec.md']
Default glob covers BOTH layouts (flat FR-*.md and folder-per-FR spec.md).
"""
import re, sys
from pathlib import Path
try:
    import yaml
except ImportError:
    print("repair_fr_yaml: needs pyyaml (pip install pyyaml)", file=sys.stderr); sys.exit(2)

root = Path(sys.argv[sys.argv.index("--root") + 1]).resolve() if "--root" in sys.argv else Path.cwd()
FR = root / "docs" / "tasks"

def err_line(e):
    m = getattr(e, "problem_mark", None) or getattr(e, "context_mark", None)
    return m.line if m else None

def quote_line(line):
    if line.strip().startswith("- ") and not line.strip().startswith('- "'):
        indent = line[:len(line) - len(line.lstrip())]
        rest = line.strip()[2:]
        return f'{indent}- "{rest.replace(chr(92), chr(92)*2).replace(chr(34), chr(92)+chr(34))}"'
    # broken double-quoted LIST item (e.g. - "Bypass for "system" admin") - re-escape the inner
    # text and re-wrap; only reached when strict YAML already failed at/near this line.
    if line.strip().startswith('- "') and line.strip().endswith('"') and len(line.strip()) > 4:
        indent = line[:len(line) - len(line.lstrip())]
        inner = line.strip()[3:-1]
        return f'{indent}- "{inner.replace(chr(92), chr(92)*2).replace(chr(34), chr(92)+chr(34))}"'
    m = re.match(r"^(\s*[A-Za-z_][A-Za-z0-9_]*):\s+(.*[^\s].*)$", line)
    if m and not m.group(2).startswith(('"', "'", "[", "{", "|", ">")):
        v = m.group(2).replace("\\", "\\\\").replace('"', '\\"')
        return f'{m.group(1)}: "{v}"'
    # broken double-quoted scalar (e.g. title: ""Verify us" block: ...") - the author quoted the
    # value but left inner quotes unescaped. Re-escape the inner text and re-wrap; only reached
    # when strict YAML already failed on this line, so a valid quoted scalar is never touched.
    if m and m.group(2).startswith('"') and m.group(2).endswith('"') and len(m.group(2)) > 1:
        inner = m.group(2)[1:-1]
        v = inner.replace("\\", "\\\\").replace('"', '\\"')
        return f'{m.group(1)}: "{v}"'
    return None

repaired, failed = [], []
targets = (list(FR.glob("FR-*.md"))                                     # root-level flat (pre-migration)
           + list(FR.glob("*/FR-*.md")) + list(FR.glob("*/FR-*/spec.md")) + list(FR.glob("*/FR-*/audit.md")))
for f in sorted(targets):
    if "_audits" in f.parts or f.name.endswith(".audit.md") and False: pass
    text = f.read_text()
    m = re.match(r"\A---\n(.*?\n)---\n", text, re.S)
    if not m: continue
    fm = m.group(1)
    try:
        yaml.safe_load(fm); continue
    except yaml.YAMLError:
        pass
    lines = fm.replace("\t", "  ").split("\n")
    changed, ok = 0, False
    for _ in range(80):
        try:
            yaml.safe_load("\n".join(lines)); ok = True; break
        except yaml.YAMLError as e:
            ln = err_line(e); fixed = False
            for cand in ([ln, ln - 1, ln + 1] if ln is not None else []):
                if cand is not None and 0 <= cand < len(lines):
                    q = quote_line(lines[cand])
                    if q and q != lines[cand]:
                        lines[cand] = q; changed += 1; fixed = True; break
            if not fixed: break
    if ok:
        f.write_text(text[:m.start(1)] + "\n".join(lines) + text[m.end(1):])
        repaired.append((str(f), changed))
    else:
        failed.append(str(f))

print(f"repair_fr_yaml: repaired {len(repaired)} file(s) ({sum(c for _, c in repaired)} lines quoted); {len(failed)} need manual attention")
for p in failed: print(f"  MANUAL: {p}")
sys.exit(0 if not failed else 1)
