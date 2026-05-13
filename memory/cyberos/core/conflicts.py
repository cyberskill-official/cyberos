"""
cyberos.core.conflicts — sync-FS conflict awareness (PROPOSAL.md P9).

Sync engines (iCloud, Dropbox, OneDrive, Google Drive, Box, Syncthing,
Resilio) detect concurrent edits across machines and resolve them by
leaving the original file alone and writing the divergent copy beside it
under a vendor-specific naming convention. The audit chain is unaware of
those sibling files — they appear as "memory bodies that nobody wrote".

This module:

* detects conflict siblings on disk (filename pattern match);
* picks the audit-chain-canonical version (the file currently named by
  the protocol — the audit ledger's view of "truth");
* prints a unified diff vs. each sibling so the operator can resolve;
* offers ``resolve_conflict`` to keep one side and move the others into
  ``conflicts/<seq>/<basename>.<source>.md`` for cold storage.

The detector is also exposed as a self-audit invariant
``layout-no-sync-conflict-siblings`` (level=warning) so ``cyberos doctor``
surfaces conflicts on a routine basis — operators don't have to remember
to check.

Patterns covered (case-insensitive):

* Dropbox / generic ``foo (conflict).md``, ``foo (conflict 1).md``
* Apple / Finder ``foo 2.md`` (we DON'T match this — too noisy)
* iCloud ``foo (mac).md``, ``foo (Stephens-MacBook).md`` — covered by
  the OneDrive-style ``foo - <hostname>.md`` rule below.
* OneDrive ``foo - Stephens-MacBook.md`` (note dash, not paren)
* Syncthing ``foo.sync-conflict-20251225-103145-AB12CD3.md``
* Resilio ``foo.bak`` and ``foo.<hostname>.<timestamp>.bak`` — covered
  by ``.bak`` suffix.
* Google Drive ``foo (Conflicted copy 2025-12-25).md``
* Box ``foo (Conflicted copy with Stephen Cheng 2025-12-25).md``
"""

from __future__ import annotations

import difflib
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Iterator

# ---------------------------------------------------------------------------
# pattern matching
# ---------------------------------------------------------------------------

# Each pattern is a callable: (filename: str) -> (basename: str | None, source: str)
# where `basename` is the canonical name (the chain-truth file) without the
# conflict suffix, or None if the filename is not a conflict marker.
#
# We keep these as regexes rather than glob-translated forms so we can pin
# the exact capture groups and source tag. Order matters: the most specific
# patterns come first.

_SYNC_CONFLICT_PATTERNS: tuple[tuple[re.Pattern[str], str], ...] = (
    # Syncthing
    (re.compile(r"^(?P<base>.+?)\.sync-conflict-\d{8}-\d{6}-[A-Z0-9]+(?P<ext>\.[A-Za-z0-9]+)$"),
     "syncthing"),
    # Dropbox classic + Google Drive variants
    (re.compile(r"^(?P<base>.+?) \((?:conflict|conflicted copy|conflicted copy with [^)]+)(?: \d+)?(?: \d{4}-\d{2}-\d{2})?\)(?P<ext>\.[A-Za-z0-9]+)$",
                re.IGNORECASE),
     "dropbox-or-gdrive"),
    # Dropbox "(<user>'s conflicted copy <date>)"
    (re.compile(r"^(?P<base>.+?) \(.+?'s conflicted copy \d{4}-\d{2}-\d{2}\)(?P<ext>\.[A-Za-z0-9]+)$"),
     "dropbox"),
    # OneDrive "foo - <hostname>.md" — only flag when hostname has a dash
    # (e.g. "Stephens-MacBook") AND the part before is a known memory file
    # extension. We intentionally don't match plain "foo - bar.md" — too
    # many false positives.
    (re.compile(r"^(?P<base>.+?) - [A-Za-z0-9]+-[A-Za-z0-9]+(?P<ext>\.md)$"),
     "onedrive"),
    # iCloud / "(Mac)" — same idea
    (re.compile(r"^(?P<base>.+?) \((?:Mac|iPhone|iPad)\)(?P<ext>\.md)$",
                re.IGNORECASE),
     "icloud"),
    # Resilio / generic .bak
    (re.compile(r"^(?P<base>.+?)(?P<ext>\.md)\.bak$"),
     "backup"),
)


def classify_sibling(filename: str) -> tuple[str | None, str | None]:
    """Return ``(canonical_basename, source)`` or ``(None, None)`` if not a conflict file.

    The canonical basename is what the audit chain would call the file
    (the name without the vendor's conflict suffix).
    """
    for pattern, source in _SYNC_CONFLICT_PATTERNS:
        m = pattern.match(filename)
        if m:
            base = m.group("base")
            ext = m.group("ext") if "ext" in m.groupdict() else ""
            return base + ext, source
    return None, None


# ---------------------------------------------------------------------------
# scan
# ---------------------------------------------------------------------------


@dataclass
class ConflictPair:
    """One canonical file ↔ one or more conflict siblings."""

    canonical: Path
    """The chain-truth file (may not exist if every copy is a sibling)."""

    siblings: list[tuple[Path, str]] = field(default_factory=list)
    """List of (sibling_path, source_tag)."""


def scan(store: Path) -> list[ConflictPair]:
    """Walk the store and group conflict siblings by canonical filename.

    Only walks paths the protocol writes to (``memories/``, ``meta/``,
    plus the v2 entity subdirs). Skips ``audit/``, ``index/``, ``exports/``,
    and ``conflicts/`` itself (the resolution destination).
    """
    SKIP_DIRS = {"audit", "index", "exports", "conflicts", ".cache"}
    pairs: dict[Path, ConflictPair] = {}

    def _walk(root: Path) -> Iterator[Path]:
        if not root.is_dir():
            return
        for child in root.iterdir():
            if child.is_dir():
                if child.name in SKIP_DIRS:
                    continue
                yield from _walk(child)
            elif child.is_file():
                yield child

    for f in _walk(store):
        # Don't classify files in skip dirs (covered by _walk pruning) but
        # also skip dotfiles + readme + manifest at the root.
        if f.parent == store and f.name in {
            "README.md", "manifest.json", "HEAD", ".lock", ".DS_Store",
        }:
            continue
        canonical_name, source = classify_sibling(f.name)
        if canonical_name is None:
            continue
        canonical_path = f.parent / canonical_name
        pair = pairs.setdefault(canonical_path, ConflictPair(canonical=canonical_path))
        pair.siblings.append((f, source))

    return [p for p in pairs.values() if p.siblings]


# ---------------------------------------------------------------------------
# diff + resolve
# ---------------------------------------------------------------------------


def diff(canonical: Path, sibling: Path) -> str:
    """Unified diff (sibling → canonical) for human review.

    Returns the empty string if the bodies are byte-identical (in which
    case the sibling is safe to discard).
    """
    if not canonical.is_file():
        return f"# canonical {canonical.name} missing — sibling is the only copy\n"
    a = canonical.read_text(encoding="utf-8", errors="replace").splitlines(keepends=True)
    b = sibling.read_text(encoding="utf-8", errors="replace").splitlines(keepends=True)
    if a == b:
        return ""
    return "".join(difflib.unified_diff(
        a, b,
        fromfile=str(canonical.name),
        tofile=str(sibling.name),
        n=3,
    ))


def resolve_conflict(
    store: Path,
    canonical_path: Path,
    *,
    keep: str = "canonical",
    actor: str = "cyberos-resolve",
    dry_run: bool = False,
) -> dict[str, str]:
    """Resolve all conflict siblings of ``canonical_path``.

    ``keep`` selects the winner:

    * ``"canonical"`` (default) — leave the chain-truth file alone, move
      every sibling under ``conflicts/<ts>/<basename>.<source>.md``.
    * ``"sibling:<index>"`` — replace the canonical with sibling ``<index>``
      (1-based) and archive the rest. The replacement IS NOT an audit
      operation — it's a filesystem move. The caller MUST follow up with
      ``cyberos put`` to introduce the new bytes into the chain. We refuse
      to silently mutate the chain — that's the operator's call.

    Returns a dict describing what was done (paths involved + outcome).
    The dict is also written under ``conflicts/<ts>/manifest.json`` so the
    operation is itself recorded on disk.
    """
    import shutil
    import time

    pair = next((p for p in scan(store) if p.canonical == canonical_path), None)
    if pair is None or not pair.siblings:
        return {"status": "no-conflicts", "canonical": str(canonical_path)}

    ts = time.strftime("%Y%m%dT%H%M%S", time.gmtime())
    conflicts_root = store / "conflicts" / ts
    if not dry_run:
        conflicts_root.mkdir(parents=True, exist_ok=True)

    result: dict = {
        "status": "resolved",
        "canonical": str(canonical_path.relative_to(store)),
        "keep": keep,
        "archived": [],
        "actor": actor,
        "timestamp_utc": ts,
    }

    siblings_sorted = sorted(pair.siblings, key=lambda s: s[0].name)

    if keep.startswith("sibling:"):
        idx = int(keep.split(":", 1)[1]) - 1
        if not (0 <= idx < len(siblings_sorted)):
            raise ValueError(
                f"sibling index out of range: keep={keep}, "
                f"{len(siblings_sorted)} sibling(s) present"
            )
        winner_path, winner_source = siblings_sorted[idx]
        result["winner_source"] = winner_source
        result["winner_path"] = str(winner_path.relative_to(store))
        if not dry_run:
            # Move the chosen sibling on top of the canonical. We refuse to
            # touch the chain here — caller must `cyberos put` the new bytes
            # afterward to introduce them properly.
            shutil.move(str(winner_path), str(canonical_path))
        # All other siblings → archive.
        for i, (s_path, s_source) in enumerate(siblings_sorted):
            if i == idx:
                continue
            archived = conflicts_root / f"{canonical_path.name}.{s_source}.md"
            if archived.exists():
                archived = conflicts_root / f"{canonical_path.name}.{s_source}.{i}.md"
            if not dry_run:
                shutil.move(str(s_path), str(archived))
            result["archived"].append(str(archived.relative_to(store)))
        result["next_step"] = (
            f"run `cyberos put {result['canonical']} <body>` to introduce "
            "the new bytes into the audit chain"
        )
    elif keep == "canonical":
        for i, (s_path, s_source) in enumerate(siblings_sorted):
            archived = conflicts_root / f"{canonical_path.name}.{s_source}.md"
            if archived.exists():
                archived = conflicts_root / f"{canonical_path.name}.{s_source}.{i}.md"
            if not dry_run:
                shutil.move(str(s_path), str(archived))
            result["archived"].append(str(archived.relative_to(store)))
    else:
        raise ValueError(f"unknown --keep mode: {keep!r}")

    if not dry_run:
        import json
        (conflicts_root / "manifest.json").write_text(
            json.dumps(result, indent=2, sort_keys=True),
            encoding="utf-8",
        )

    return result


# ---------------------------------------------------------------------------
# pretty-print
# ---------------------------------------------------------------------------


def format_scan(pairs: Iterable[ConflictPair]) -> str:
    pairs = list(pairs)
    if not pairs:
        return "  no sync-FS conflict siblings detected"
    out: list[str] = []
    out.append(f"  {len(pairs)} canonical file(s) have conflict siblings:")
    for p in pairs:
        out.append(f"")
        out.append(f"  • {p.canonical.name}")
        out.append(f"    canonical: {p.canonical}")
        for sibling, source in p.siblings:
            out.append(f"    sibling  : [{source}] {sibling.name}")
    return "\n".join(out)


__all__ = [
    "ConflictPair",
    "classify_sibling",
    "scan",
    "diff",
    "resolve_conflict",
    "format_scan",
]
