#!/usr/bin/env python3
"""
migrate_consumer_fr_to_task.py — one-shot fr->task migration for a CONSUMER repo.

Why this exists (and why it is NOT in the installer):
  The fr->task rename was applied to the cyberos repo itself by scripts/migrate_fr_to_task.py,
  a codemod whose repo_root() and every move go through git, and whose PATH_RENAMES are
  cyberos-internal paths. Consumer repos got nothing: install.sh has no docs/feature-requests
  -> docs/tasks step at all, so installing 1.0.0 over a pre-rename repo would create an empty
  docs/tasks/ and strand every existing spec in the old directory. audit-fleet would then
  report the repo GREEN, because it compares page-count to disk-count and 0 == 0.

  Stephen's call (2026-07-16): this is a one-time migration with no future use, so it does not
  belong in the installer. Apply it directly, once, to every repo.

Differences from the cyberos codemod, all forced by consumer reality:
  - NO git. sachviet and ssl have no .git at all (ssl has three nested repos under it), so
    every move is os.rename and the safety net is a filesystem backup, not `git checkout`.
  - Only the two path renames a consumer actually has (docs/feature-requests, FR-* dirs).
  - Order matters: subtask ids (FR-<NNN>-T-<MM>) MUST be rewritten before plain FR-<NNN>,
    or the plain rule eats the prefix and the -T- suffix survives as garbage.

Usage:
    python3 scripts/migrate_consumer_fr_to_task.py <repo> [<repo> ...]        # dry run
    python3 scripts/migrate_consumer_fr_to_task.py --apply <repo> [<repo> ...]
    python3 scripts/migrate_consumer_fr_to_task.py --verify <repo> [...]      # assert clean

Safe to re-run: idempotent. A repo already on docs/tasks/ with no FR- residue is a no-op.
"""
from __future__ import annotations

import argparse
import os
import re
import shutil
import sys
import time
from pathlib import Path

# ---------------------------------------------------------------- content rules
# Ordered. Most specific first — see the module docstring on subtask ids.
# Mirrors docs/tasks/RENAME-EPOCH.md.
# The id rules. rename_path() applies ONLY these (a filename is an id, never prose).
ID_RULES: list[tuple[str, str, str]] = [
    # Subtask first: -T- must become -S- before the generic prefix rule consumes the FR-.
    ("id:subtask",      r"(?<![A-Za-z0-9_-])FR-(\d+)-T-(\d+)", r"TASK-\1-S-\2"),
    # Then the generic prefix. RENAME-EPOCH documents only FR-<MOD>-<NNN> and FR-<NNN>,
    # which is cyberos's own vocabulary — the fleet is wilder. A scan of every real id in
    # the 23 repos (74 distinct shapes) found: multi-segment modules (FR-DD-EDU-001),
    # letter+digit (FR-A01..FR-F05), no-number ids (FR-API-READY-CAST-CLI), module words
    # (FR-CLICK, FR-WEB), and template placeholders (FR-ID, FR-TEMPLATE, FR-XXX). Matching
    # shapes one at a time is how dom-defender's 13 specs and strategem's 1 were missed and
    # still rendered 0-on-page. So: replace the PREFIX, keep whatever follows.
    #
    # The lookbehind is load-bearing. NFR-AFFIL-001 is a Non-Functional Requirement — a
    # DIFFERENT artefact that must never become NTASK-. `\b` does NOT protect it (there is
    # no word boundary between N and F), and BSD grep's \b is unreliable anyway, which is
    # exactly how an earlier shell scan "found" 300 phantom hits inside NFR- ids.
    ("id:prefix",       r"(?<![A-Za-z0-9_-])FR-(?=[A-Z0-9])",  "TASK-"),
]

RULES: list[tuple[str, str, str]] = [
    *ID_RULES,
    # -- commands / skills (must precede the generic vocabulary rules) -------
    ("cmd:ship",        r"\bship-feature-requests\b",       "ship-tasks"),
    ("cmd:create",      r"\bcreate-feature-requests\b",     "create-tasks"),
    # -- paths ---------------------------------------------------------------
    ("path:docs-dir",   r"docs/feature-requests\b",         "docs/tasks"),
    ("path:status-data", r"docs/status/data/fr\b",          "docs/status/data/task"),
    # -- vocabulary ----------------------------------------------------------
    ("vocab:kebab-pl",  r"\bfeature-requests\b",            "tasks"),
    ("vocab:kebab",     r"\bfeature-request\b",             "task"),
    ("vocab:snake-pl",  r"\bfeature_requests\b",            "tasks"),
    ("vocab:snake",     r"\bfeature_request\b",             "task"),
    ("vocab:prose-pl",  r"\bfeature requests\b",            "tasks"),
    ("vocab:prose",     r"\bfeature request\b",             "task"),
    ("vocab:Prose-pl",  r"\bFeature Requests\b",            "Tasks"),
    ("vocab:Prose",     r"\bFeature Request\b",             "Task"),
    # -- the bare abbreviation, last: it is the loosest and would otherwise
    #    swallow the id rules above. Word-boundary guarded so "FROM"/"FRAME"
    #    are untouched.
    ("abbr:plural",     r"(?<![A-Za-z0-9_-])FRs(?![A-Za-z0-9_-])",  "tasks"),
    ("abbr:singular",   r"(?<![A-Za-z0-9_-])FR(?![A-Za-z0-9_-])",   "task"),
]

# DELIBERATELY NOT A RULE: fr_id / frId.
#   The cyberos codemod renames fr_id -> task_id, because there fr_id is CYBEROS's own
#   contract. In a consumer repo it is the CONSUMER's. tamagochi/src/media.ts builds
#       const body = { fr_id: input.frId, asset_url, caption, scheduled_for }
#   and POSTs it to https://social-publisher.local/<platform>/posts — an outbound wire
#   contract with a service that does not live in this repo and would not be renamed with
#   it. 8 repos / 311 files carry fr_id, including production code and tests.
#   The VALUE is a cyberos id and migrates (fr_id: "TASK-VIRAL-004"). The KEY is the
#   consumer's schema and does not. Renaming someone else's wire field to tidy our
#   vocabulary is not a rename, it is a breaking change.

# Only text we own. Never walk the vendored machine or the BRAIN: .cyberos/ is replaced
# wholesale by install.sh, and the BRAIN audit chain records each body's sha256 (AGENTS.md
# §5.3) — a byte-level rewrite there breaks the chain and flips the store to FROZEN.
SKIP_DIRS = {".git", ".cyberos", ".cyberos-install", "node_modules", "dist", "target",
             ".next", ".venv", "__pycache__", ".vercel-out", "build"}
TEXT_EXT = {".md", ".markdown", ".yaml", ".yml", ".json", ".js", ".mjs", ".cjs", ".ts",
            ".tsx", ".jsx", ".py", ".sh", ".rs", ".toml", ".txt", ".html", ".css"}


def is_text(p: Path) -> bool:
    return p.suffix.lower() in TEXT_EXT


def rewrite(s: str) -> tuple[str, dict[str, int]]:
    hits: dict[str, int] = {}
    for name, pat, rep in RULES:
        s, n = re.subn(pat, rep, s)
        if n:
            hits[name] = hits.get(name, 0) + n
    return s, hits


def walk(root: Path):
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for f in filenames:
            yield Path(dirpath) / f


def rename_path(name: str) -> str:
    """FR-WEB-002-slug -> TASK-WEB-002-slug (dirs AND files)."""
    for _, pat, rep in ID_RULES:
        name = re.sub(pat, rep, name)
    return name


def is_husk(d: Path) -> bool:
    """True only if d provably carries NO work: no specs, and a BACKLOG with no rows.

    install.sh creates docs/tasks/ without touching a pre-existing docs/feature-requests/,
    so a repo that has been installed-over has BOTH: the fresh scaffold and the old husk.
    Deleting the husk is right; deleting a real backlog is not. So prove it is empty.
    """
    if not d.is_dir():
        return False
    for f in d.rglob("*"):
        if f.is_dir():
            continue
        if f.name == "spec.md" or re.match(r"^FR-.*\.md$", f.name):
            return False
        if f.suffix == ".md" and f.name != "BACKLOG.md":
            return False
    bl = d / "BACKLOG.md"
    if bl.is_file():
        rows = [l for l in bl.read_text(encoding="utf-8", errors="ignore").splitlines()
                if l.lstrip().startswith("|")]
        if rows:
            return False
    return True


def migrate(repo: Path, apply: bool) -> dict:
    out = {"repo": str(repo), "moved_dir": False, "renamed": 0, "files": 0, "hits": {}}
    old, new = repo / "docs" / "feature-requests", repo / "docs" / "tasks"

    # 1. the directory move. Guarded so a re-run is a no-op, and so we never merge a
    #    half-migrated repo into an existing docs/tasks and silently lose one of them.
    if old.is_dir():
        if new.exists():
            if is_husk(old):
                # Proven empty: the pre-rename scaffold left behind by an install that
                # created docs/tasks/ beside it. Drop it; keep the live one.
                out["dropped_husk"] = True
                if apply:
                    shutil.rmtree(old)
            else:
                raise SystemExit(
                    f"REFUSING {repo}: BOTH docs/feature-requests and docs/tasks exist and "
                    f"the old one is NOT empty. Resolve by hand — merging could lose specs.")
        else:
            out["moved_dir"] = True
            if apply:
                os.rename(old, new)

    if not apply:
        # Dry run: count what WOULD happen, from the old location if it is still there.
        base = old if old.is_dir() else new
    else:
        base = new

    # 2. rename FR-* dirs and files, deepest-first so parents stay valid mid-walk.
    #    REPO-WIDE, not just under docs/tasks: three repos carry FR-named artefacts
    #    elsewhere (tamagochi/docs/marketing/FR-VIRAL-004-social-payload.json,
    #    design-system-audit-framework/docs/framework/core/FR-CORE-001-*.json,
    #    styx/archive/**/FR-024-*.md). Step 3 rewrites their CONTENTS, so leaving the
    #    filename is precisely the contents-renamed-directory-left bug this whole epoch
    #    keeps producing — and any code reference, already rewritten to TASK-*, would
    #    dangle. The filename encodes a cyberos id; it migrates with the id.
    for dirpath, dirnames, filenames in os.walk(repo, topdown=False):
        if any(part in SKIP_DIRS for part in Path(dirpath).parts):
            continue
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for n in filenames + dirnames:
            nn = rename_path(n)
            if nn != n:
                out["renamed"] += 1
                if apply:
                    os.rename(Path(dirpath) / n, Path(dirpath) / nn)

    # 3. content, across the whole repo (specs, BACKLOG, AGENTS.md, CLAUDE.md, READMEs...).
    for f in walk(repo):
        if not is_text(f):
            continue
        try:
            s = f.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        s2, hits = rewrite(s)
        if s2 != s:
            out["files"] += 1
            for k, v in hits.items():
                out["hits"][k] = out["hits"].get(k, 0) + v
            if apply:
                f.write_text(s2, encoding="utf-8")

    # 4. drop the stale generated status data. install.sh regenerates docs/status/ from the
    #    specs; leaving data/fr/ behind would strand a second, FR-named copy of the corpus.
    stale = repo / "docs" / "status" / "data" / "fr"
    if stale.is_dir():
        out["stale_status_data"] = True
        if apply:
            shutil.rmtree(stale)

    return out


def verify(repo: Path) -> list[str]:
    bad = []
    if (repo / "docs" / "feature-requests").exists():
        bad.append("docs/feature-requests still exists")
    if (repo / "docs" / "status" / "data" / "fr").exists():
        bad.append("docs/status/data/fr still exists")
    for f in walk(repo):
        if f.name != rename_path(f.name):
            bad.append(f"FR- in path: {f.relative_to(repo)}")
        if not is_text(f):
            continue
        try:
            s = f.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        for name, pat, _ in ID_RULES:
            m = re.search(pat, s)
            if m:
                bad.append(f"{name} residue in {f.relative_to(repo)}: {m.group(0)}")
                break
    return bad


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("repos", nargs="+")
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--verify", action="store_true")
    ap.add_argument("--backup-dir", default="/tmp/cyberos-migration-backups")
    a = ap.parse_args()

    rc = 0
    for r in a.repos:
        repo = Path(r).resolve()
        if not repo.is_dir():
            print(f"SKIP {r}: not a directory"); rc = 1; continue

        if a.verify:
            bad = verify(repo)
            print(f"{'CLEAN' if not bad else 'DIRTY'}  {repo.name}"
                  + ("" if not bad else f"  ({len(bad)} finding(s))"))
            for b in bad[:5]:
                print(f"    {b}")
            rc |= 1 if bad else 0
            continue

        if a.apply:
            # Filesystem backup, not git: two of these repos have no .git to fall back on.
            b = Path(a.backup_dir) / f"{repo.name}-{int(time.time())}"
            b.parent.mkdir(parents=True, exist_ok=True)
            shutil.copytree(repo / "docs", b / "docs", symlinks=True)
            print(f"  backup: {b}")

        res = migrate(repo, a.apply)
        tag = "APPLIED" if a.apply else "would"
        print(f"{tag:8s} {repo.name:34s} dir_move={res['moved_dir']!s:5s} "
              f"renamed={res['renamed']:<4d} files={res['files']:<4d} "
              f"ids={sum(v for k, v in res['hits'].items() if k.startswith('id:'))}")
    return rc


if __name__ == "__main__":
    sys.exit(main())
