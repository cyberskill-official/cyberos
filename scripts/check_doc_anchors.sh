#!/usr/bin/env bash
# check_doc_anchors.sh - TASK-SKILL-119 §1 #3. Scans modules/skill/**/*.md + modules/cuo/**/*.md
# for repo-relative markdown links and inline `path#anchor` citations; verifies the target file
# exists and, when an anchor is given, that it resolves to a heading (GitHub slug rules).
# exit 0 clean | exit 10 with `DEAD <citing-file>:<line> -> <target>` lines | exit 2 unusable.
# --list prints every reference with status, always exit 0. External http(s) URLs are skipped.
set -uo pipefail
repo="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
mode="check"; [ "${1:-}" = "--list" ] && mode="list"
python3 - "$repo" "$mode" <<'PY'
import os, re, sys, unicodedata
repo, mode = sys.argv[1], sys.argv[2]
EXE = os.path.join(repo, "scripts", "doc-anchor-exemptions.txt")
exempt = []
if os.path.isfile(EXE):
    for line in open(EXE):
        line = line.split("#")[0].strip()
        if line: exempt.append(line)
used = set()
# extraction grammar (deterministic): markdown links `](target)` + backticked inline citations
MD_LINK = re.compile(r"\]\(([^)\s]+)\)")
INLINE  = re.compile(r"`((?:modules|docs|tools|scripts)/[A-Za-z0-9_./-]+\.md(?:#[A-Za-z0-9_§().%-]+)?)`")

def slug(h):
    h = re.sub(r"[*`_]", "", h.strip().lower())
    h = unicodedata.normalize("NFKD", h)
    h = re.sub(r"[^\w\s§().-]", "", h, flags=re.UNICODE)
    return re.sub(r"\s+", "-", h).strip("-")

def planned_files(src):
    """new_files/modified_files entries of a task spec (planned deliverables are valid citations)."""
    out = set()
    try:
        txt = open(src, encoding="utf-8", errors="replace").read()
        m = re.match(r"\A---\n(.*?)\n---\n", txt, re.S)
        if m:
            cur = None
            for line in m.group(1).split("\n"):
                if re.match(r"^(new_files|modified_files):", line): cur = True; continue
                if cur and re.match(r"^\s+-\s+", line): out.add(line.split("-", 1)[1].strip().strip('"'))
                elif not line.startswith(" "): cur = None
    except OSError: pass
    return out

CORPUS_PLANNED = None
def corpus_planned(repo):
    global CORPUS_PLANNED
    if CORPUS_PLANNED is None:
        CORPUS_PLANNED = set()
        import glob as g
        for spec in g.glob(os.path.join(repo, "docs", "tasks", "*", "TASK-*", "spec.md")):
            CORPUS_PLANNED |= planned_files(spec)
    return CORPUS_PLANNED

def fr_status(src):
    try:
        head = open(src, encoding="utf-8", errors="replace").read(2000)
        m = re.search(r"^status:\s*([a-z_]+)", head, re.M)
        return m.group(1) if m else ""
    except OSError: return ""

def headings(path):
    hs = set()
    for line in open(path, encoding="utf-8", errors="replace"):
        m = re.match(r"#{1,6}\s+(.*)", line)
        if m:
            hs.add(slug(m.group(1)))
    return hs

dead, listed = [], []
planned_cache = {}
roots = [os.path.join(repo, "modules", "skill"), os.path.join(repo, "modules", "cuo"), os.path.join(repo, "docs", "tasks")]
for root in roots:
    for dp, _, files in os.walk(root):
        for f in files:
            if not f.endswith(".md"): continue
            if "tasks" in dp and (os.sep + "." in dp or "_audits" in dp or "_archive" in dp): continue
            src = os.path.join(dp, f)
            rel_src = os.path.relpath(src, repo)
            for ln, line in enumerate(open(src, encoding="utf-8", errors="replace"), 1):
                targets = MD_LINK.findall(line) + INLINE.findall(line)
                for t in targets:
                    if t.startswith(("http://", "https://", "mailto:")): continue
                    if "<" in t or "{" in t: continue  # scaffold placeholders by design (TASK-SKILL-115)
                    if re.search(r"YYYY|XX|\bN\.N\b", t): continue  # date/number placeholder patterns
                    if "tasks" in rel_src and f == "spec.md":
                        base = t.split("#")[0]
                        if base in planned_cache.setdefault(src, planned_files(src)) or base in corpus_planned(repo):
                            listed.append(f"planned     {rel_src}:{ln} -> {t}"); continue
                    if any(rel_src == e or rel_src.startswith(e.rstrip("/") + "/") for e in exempt):
                        used.update(e for e in exempt if rel_src == e or rel_src.startswith(e.rstrip("/") + "/"))
                        continue
                    path, _, anchor = t.partition("#")
                    if not path:  # same-file anchor
                        path = rel_src
                    if not (path.endswith(".md") or "/" in path): continue
                    # resolve: repo-root-relative for known top dirs, else citing-file-relative
                    if path.startswith(("modules/", "docs/", "tools/", "scripts/", ".cyberos/")):
                        target = os.path.join(repo, path)
                    else:
                        target = os.path.normpath(os.path.join(dp, path))
                    status = "ok"
                    if not os.path.isfile(target):
                        if os.path.isdir(target): status = "ok(dir)"
                        else: status = "dead-file"
                    elif anchor and target.endswith(".md"):
                        want = slug(anchor.replace("%20", " "))
                        if want not in headings(target):
                            status = "dead-anchor"
                    if status.startswith("dead"):
                        if "tasks" in rel_src and f == "spec.md" and fr_status(src) in ("done", "closed", "on_hold"):
                            print(f"WARN historical spec ref: {rel_src}:{ln} -> {t}", file=sys.stderr)
                            listed.append(f"hist-stale  {rel_src}:{ln} -> {t}"); continue
                        dead.append(f"DEAD {rel_src}:{ln} -> {t}")
                    listed.append(f"{status:11s} {rel_src}:{ln} -> {t}")
for e in exempt:
    if e not in used:
        print(f"WARN unused exemption: {e}", file=sys.stderr)
if mode == "list":
    print("\n".join(listed)); print(f"total={len(listed)} dead={len(dead)}"); sys.exit(0)
if dead:
    print("\n".join(dead)); sys.exit(10)
print(f"anchors OK: {len(listed)} references resolved across modules/skill + modules/cuo")
PY
exit $?
