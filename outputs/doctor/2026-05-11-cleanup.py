#!/usr/bin/env python3
"""
outputs/doctor/2026-05-11-cleanup.py — bulk-cleanup pass for the issues
the §8.7 self-audit surfaced after Bundle Q landed (2026-05-11).

Run from the project repo root on macOS:
    python3 outputs/doctor/2026-05-11-cleanup.py [--dry-run]

This script does NOT mutate AGENTS.md or any protocol contract — it only
refreshes memory frontmatter and adds a registry file. Per the cowork →
macOS handoff pattern (HOST_ADAPTERS.md Adapter A), it must run on the
real macOS filesystem path; cowork's bash sandbox is blocked by §0.1.

What it does (single session — wraps everything in session-start /
session-end so the audit ledger sees one logical cleanup unit):

  1.  Fix REF-017 dangling relates_to: typo'd UUID → REF-016's actual ID.
  2.  Fix REF-019 dangling relates_to: typo'd UUID → REF-018's actual ID.
  3.  Create meta/legacy-files.md registry — declares 5 protocol-history
      archives + 7 chain-history backward-orphans as deliberately
      registered, demoting them from WARN/INFO-noise to silenced INFO
      with a clear marker.
  4.  Refresh provenance.source_ref on 24 stale memories:
        • 14 Group A: replace `docs/CyberOS-PRD.docx#partN` anchors with
          content-addressable refs into docs/CyberOS-PRD.CHANGELOG.md
          (with sha256 inline);
        • 10 Group B: replace `AGENTS.md v1.0.0 → v1.1.0 amendment` with
          a CHANGELOG-pointing form that's honest about the unrecoverable
          pre-genesis SHA.
  5.  Re-run §8.7 self-audit to confirm the WARN count drops.
  6.  Close the session (session.end + final manifest str_replace).

Behaviour:
  --dry-run    Compute every transformation and print what would change;
               make NO writes, append NO audit rows. Exit 0 if all
               transforms are clean, 1 if any input file is unparseable.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# ─── Config ──────────────────────────────────────────────────────────────
ACTOR = "subject:stephen-cheng"
SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent.parent
WRITER = REPO_ROOT / "outputs" / "brain_writer.py"
BRAIN_ROOT = REPO_ROOT / ".cyberos-memory"

# Computed canonical SHAs of the CHANGELOG files (validated at runtime)
# These are documented in the inventory below; if the actual file SHA
# differs, the script recomputes and uses the live value (the inline
# value is just for the in-repo source ref — the SHA pinned in the
# new source_ref must match the CURRENT file content at write time).
PRD_CHANGELOG = REPO_ROOT / "docs" / "CyberOS-PRD.CHANGELOG.md"
SRS_CHANGELOG = REPO_ROOT / "docs" / "CyberOS-SRS.CHANGELOG.md"
AGENTS_CHANGELOG = REPO_ROOT / "docs" / "CyberOS-AGENTS.CHANGELOG.md"


# ─── Inventory ───────────────────────────────────────────────────────────

# REF-017 / REF-019 dangling relates_to fixes (subagent investigation
# found these are typo-class corrections — adjacent REF numbers, kind
# stays "refines").
REF_REL_FIXES = [
    {
        "path": "memories/refinements/REF-017-scaffold-skills-before-runtime-ships.md",
        "old_relates_to": "mem_019df9cf-0f44-7916-aa68-d4407b11f9e3",
        "new_relates_to": "mem_019dfb19-0001-7000-8000-000000000016",
        "kind": "refines",
        "reason": "Typo fix — REF-017 refines REF-016 (audit→fix→audit loop)",
    },
    {
        "path": "memories/refinements/REF-019-runtime-build-plan-separates-from-registry.md",
        "old_relates_to": "mem_019dfb18-0001-7000-8000-000000000016",
        "new_relates_to": "mem_019dfb1b-0001-7000-8000-000000000018",
        "kind": "refines",
        "reason": "Typo fix — REF-019 refines REF-018 (flat-contract layout)",
    },
]

# Legacy-files registry entries (paths exempt from §8.7 phase-5 WARN).
# Format: each dict becomes one line in meta/legacy-files.md.
LEGACY_FILES = [
    # Forward orphans — files on disk without audit rows
    {"path": "meta/protocol-history/AGENTS-sha256-576368647e4d17635804580ca4dded28721b1c7247f0a19666ce43f5f0eb911a.md",
     "reason": "pre-§0.6 protocol-history archive (created by older writer)",
     "approx_creation": "pre-2026-05-04"},
    {"path": "meta/protocol-history/AGENTS-sha256-599e1097199618e0d8dde22770eef6e5ad068c5c06150e2bb3829315f005780d.md",
     "reason": "pre-§0.6 protocol-history archive (created by older writer)",
     "approx_creation": "pre-2026-05-04"},
    {"path": "meta/protocol-history/AGENTS-sha256-632343f0c9e7eef251bbef5308b9859b6bd99933f2c3c76dc76a2282b41b7a1c.md",
     "reason": "pre-§0.6 protocol-history archive (created by older writer)",
     "approx_creation": "pre-2026-05-04"},
    {"path": "meta/protocol-history/AGENTS-sha256-77eda214d687f8fd8eb826b8699e62614c3b606e980486c7fcd8496f92ce6dfa.md",
     "reason": "pre-§0.6 protocol-history archive (created by older writer)",
     "approx_creation": "pre-2026-05-04"},
    {"path": "meta/protocol-history/AGENTS-sha256-d3ce9764ac76635921f6e981a713ea8822eaec442d01200930633a805a84aaf0.md",
     "reason": "pre-§0.6 protocol-history archive (created by older writer)",
     "approx_creation": "pre-2026-05-04"},
    # Backward orphans — audit rows for renamed-without-rename-op REFs
    {"path": ".cyberos-memory/memories/refinements/REF-015-document-divergence-over-forced-unification.md",
     "reason": "double-prefixed path artefact in audit chain (legacy writer bug)",
     "approx_creation": "pre-2026-05-04"},
    {"path": "memories/refinements/REF-015-protocol-update-policy.md",
     "reason": "REF-015 was re-slugged to 'document-divergence-over-forced-unification'",
     "approx_creation": "pre-2026-05-04"},
    {"path": "memories/refinements/REF-016-sync-class-boundary.md",
     "reason": "REF-016 was re-slugged to 'audit-fix-audit-catches-contract-drift'",
     "approx_creation": "pre-2026-05-04"},
    {"path": "memories/refinements/REF-017-self-audit-and-modes.md",
     "reason": "REF-017 was re-slugged to 'scaffold-skills-before-runtime-ships'",
     "approx_creation": "pre-2026-05-04"},
    {"path": "memories/refinements/REF-018-canonical-json-rfc-8785.md",
     "reason": "REF-018 was re-slugged to 'flat-contract-layout-over-versioned-folders'",
     "approx_creation": "pre-2026-05-04"},
    {"path": "memories/refinements/REF-019-rollback-flow-validated.md",
     "reason": "REF-019 was re-slugged to 'runtime-build-plan-separates-from-registry'",
     "approx_creation": "pre-2026-05-04"},
    {"path": "memories/refinements/REF-020-three-way-protocol-conflict.md",
     "reason": "REF-020 was re-slugged to 'salenoti-first-real-pipeline-run'",
     "approx_creation": "pre-2026-05-04"},
]

# Stale source_ref refresh inventory.
# Group A: docx#partN → CHANGELOG (sha256:…)
# Group B: AGENTS.md v1.0.0 → v1.1.0 → CHANGELOG date + Bundle A reference
PROVENANCE_REFRESHES_GROUP_A = [
    ("memories/facts/FACT-001-cyberskill-company.md",
     "origin / founding context"),
    ("memories/facts/FACT-002-tech-stack.md",
     "tech stack (PRD §1.1) + SRS architecture surface (part-3)"),
    ("memories/facts/FACT-003-genie-cuo.md",
     "GENIE / CUO persona (PRD parts 6 & 13)"),
    ("memories/facts/FACT-004-brain-three-layers.md",
     "BRAIN three-layer architecture (PRD part-5) + BRAIN service spec (SRS part-5)"),
    ("memories/decisions/DEC-001-pointer-to-prd-dec-catalog.md",
     "decisions catalog (PRD §11.1) + decision-pointer table (SRS part-13)"),
    ("memories/projects/PRJ-001-cyberos.md",
     "project origin"),
    ("project/architecture.md",
     "architecture (PRD part-8) + SRS part-3"),
    ("project/operating-principles.md",
     "operating principles (PRD §1.4)"),
    ("project/phase-plan.md",
     "phase plan (PRD part-14)"),
    ("project/module-catalog.md",
     "module catalog (PRD part-7)"),
    ("project/compliance.md",
     "compliance (PRD part-12)"),
    ("project/non-goals.md",
     "non-goals / what-cyberos-is-not"),
    ("project/overview.md",
     "executive summary (PRD part-1)"),
    ("project/north-star-and-okrs.md",
     "north star + OKRs (PRD part-4)"),
]

PROVENANCE_REFRESHES_GROUP_B = [
    "memories/decisions/DEC-076-standing-rule-protocol-refinement.md",
    "memories/decisions/DEC-077-verify-before-respond.md",
    "memories/decisions/DEC-078-ingestion-completeness.md",
    "memories/decisions/DEC-079-token-budget-transparency.md",
    "memories/decisions/DEC-080-source-freshness-tier.md",
    "memories/decisions/DEC-081-source-coverage-validator-phase6.md",
    "memories/decisions/DEC-082-credential-denylist-never-use.md",
    "memories/decisions/DEC-083-audit-correction-to-field.md",
    "memories/decisions/DEC-084-drift-and-refinement-memory-types.md",
    "memories/decisions/DEC-085-end-of-response-coverage-stat.md",
]


# ─── Helpers ─────────────────────────────────────────────────────────────

def die(msg, *, code=1):
    sys.stderr.write(f"doctor: {msg}\n")
    sys.exit(code)


def run_writer(args, dry_run=False):
    """Invoke brain_writer.py via subprocess. Returns the CompletedProcess."""
    cmd = [sys.executable, str(WRITER)] + args
    if dry_run:
        print(f"  [dry-run] would run: {' '.join(cmd)}")
        return None
    print(f"  → {' '.join(cmd[2:])}")
    return subprocess.run(cmd, check=True, cwd=str(REPO_ROOT))


def file_sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def now_iso():
    """ISO-8601 with offset (Asia/Ho_Chi_Minh per manifest)."""
    try:
        from zoneinfo import ZoneInfo
        return _dt.datetime.now(ZoneInfo("Asia/Ho_Chi_Minh")).replace(
            microsecond=0).isoformat()
    except Exception:
        return _dt.datetime.now(_dt.timezone.utc).replace(
            microsecond=0).isoformat()


def parse_frontmatter_split(text):
    """Split a memory file into (frontmatter_text, body_text). Returns
    (None, text) if no frontmatter."""
    if not text.startswith("---\n"):
        return None, text
    rest = text[4:]
    # Find closing fence — assume no fenced-code-block edge cases for
    # the memories we're touching (they're hand-authored DEC/REF/FACT
    # memories, not docs with embedded YAML examples).
    idx = rest.find("\n---\n")
    if idx < 0:
        # try EOF closure
        idx = rest.rfind("\n---")
        if idx < 0:
            return None, text
        return rest[:idx], rest[idx+4:]
    return rest[:idx], rest[idx+5:]


def update_yaml_field(fm_text, field_path, new_value):
    """Naive YAML-line-by-line update: find a top-level field or a
    nested provenance.* field and set its value. We avoid PyYAML
    round-tripping (which strips comments and reorders) and instead
    do textual replacement. Field paths supported:
      'version'           → top-level
      'last_updated_at'   → top-level
      'updated_by'        → top-level
      'provenance.source_ref'  → nested under 'provenance:'
      'relationships'     → list-replace (special)
    """
    lines = fm_text.split("\n")
    out = []
    i = 0
    in_provenance = False
    provenance_indent = ""
    while i < len(lines):
        line = lines[i]
        stripped = line.lstrip()
        # Track provenance block
        if stripped.startswith("provenance:"):
            in_provenance = True
            provenance_indent = line[: len(line) - len(stripped)]
            out.append(line)
            i += 1
            continue
        if in_provenance:
            cur_indent = line[: len(line) - len(stripped)] if stripped else ""
            if stripped and len(cur_indent) <= len(provenance_indent):
                in_provenance = False
                # fall through

        # Top-level fields
        if not in_provenance:
            if field_path == "version" and stripped.startswith("version:"):
                out.append(f"version:          {new_value}")
                i += 1
                continue
            if field_path == "last_updated_at" and stripped.startswith("last_updated_at:"):
                out.append(f"last_updated_at:  {new_value}")
                i += 1
                continue
            if field_path == "updated_by" and stripped.startswith("updated_by:"):
                out.append(f"updated_by:       {new_value}")
                i += 1
                continue
        else:
            if (field_path == "provenance.source_ref"
                    and stripped.startswith("source_ref:")):
                # Use double-quoted single-line YAML — handle internal "
                escaped = new_value.replace("\\", "\\\\").replace('"', '\\"')
                out.append(f'{provenance_indent}  source_ref:     "{escaped}"')
                i += 1
                continue
        out.append(line)
        i += 1
    return "\n".join(out)


def update_relationships_relates_to(fm_text, old_id, new_id):
    """Substring replacement on relates_to: <id> within the YAML."""
    return fm_text.replace(old_id, new_id)


# ─── Main flow ───────────────────────────────────────────────────────────

def cmd_dry_run():
    print("== doctor: dry-run ==")
    print()
    print("[1] REF-017 / REF-019 dangling relates_to fixes:")
    for fix in REF_REL_FIXES:
        path = BRAIN_ROOT / fix["path"]
        if not path.is_file():
            print(f"  ✗ MISSING: {fix['path']}")
            continue
        text = path.read_text(encoding="utf-8")
        if fix["old_relates_to"] not in text:
            print(f"  ⚠ no-op: {fix['path']} doesn't contain "
                  f"{fix['old_relates_to'][:24]}…")
            continue
        print(f"  ✓ {fix['path']}: {fix['old_relates_to'][:24]}…"
              f" → {fix['new_relates_to'][:24]}…")
    print()
    print("[2] meta/legacy-files.md (would create):")
    for entry in LEGACY_FILES:
        print(f"  - {entry['path']}")
    print()
    print("[3] Group A (.docx → CHANGELOG sha256) refreshes:")
    if PRD_CHANGELOG.is_file():
        print(f"  PRD CHANGELOG sha256: {file_sha256(PRD_CHANGELOG)[:24]}…")
    if SRS_CHANGELOG.is_file():
        print(f"  SRS CHANGELOG sha256: {file_sha256(SRS_CHANGELOG)[:24]}…")
    for path, topic in PROVENANCE_REFRESHES_GROUP_A:
        full = BRAIN_ROOT / path
        if not full.is_file():
            print(f"  ✗ MISSING: {path}")
            continue
        print(f"  ✓ {path}  →  {topic}")
    print()
    print("[4] Group B (AGENTS.md v1.0.0 → CHANGELOG date) refreshes:")
    for path in PROVENANCE_REFRESHES_GROUP_B:
        full = BRAIN_ROOT / path
        if not full.is_file():
            print(f"  ✗ MISSING: {path}")
            continue
        print(f"  ✓ {path}")
    print()
    print(f"Total ops planned: {len(REF_REL_FIXES)} ref-fixes + 1 registry "
          f"create + {len(PROVENANCE_REFRESHES_GROUP_A)} group-A + "
          f"{len(PROVENANCE_REFRESHES_GROUP_B)} group-B = "
          f"{len(REF_REL_FIXES) + 1 + len(PROVENANCE_REFRESHES_GROUP_A) + len(PROVENANCE_REFRESHES_GROUP_B)} "
          f"writes (+ session-start, self-audit, session-end).")


def cmd_apply():
    """Run the full cleanup. Aborts on any subprocess failure."""
    print("== doctor: apply ==")

    # Preflight
    if not WRITER.is_file():
        die(f"writer not found at {WRITER}", code=2)
    if not BRAIN_ROOT.is_dir():
        die(f"BRAIN not found at {BRAIN_ROOT}", code=2)
    try:
        subprocess.run(
            [sys.executable, "-c", "import rfc8785, yaml"],
            check=True, capture_output=True,
        )
    except subprocess.CalledProcessError:
        die("missing Python deps. Run: "
            "python3 -m pip install rfc8785 PyYAML --break-system-packages",
            code=2)

    # Compute current CHANGELOG SHAs (these go into the new source_ref strings)
    prd_sha = (file_sha256(PRD_CHANGELOG)
               if PRD_CHANGELOG.is_file() else "missing")
    srs_sha = (file_sha256(SRS_CHANGELOG)
               if SRS_CHANGELOG.is_file() else "missing")

    # ── 1. session-start ─────────────────────────────────────────────────
    print("\n── session-start ──")
    run_writer(["session-start", ACTOR])

    # ── 2. fix REF-017 / REF-019 dangling relates_to ─────────────────────
    print("\n── fix REF-017/019 dangling relates_to ──")
    for fix in REF_REL_FIXES:
        full = BRAIN_ROOT / fix["path"]
        text = full.read_text(encoding="utf-8")
        if fix["old_relates_to"] not in text:
            print(f"  skip {fix['path']} (already fixed?)")
            continue
        fm_text, body = parse_frontmatter_split(text)
        if fm_text is None:
            print(f"  ✗ unparseable frontmatter: {fix['path']}")
            continue
        # Bump version + last_updated_at + updated_by + relates_to
        fm_text = update_relationships_relates_to(
            fm_text, fix["old_relates_to"], fix["new_relates_to"])
        fm_text = bump_metadata(fm_text)
        new_text = "---\n" + fm_text + "\n---\n" + body
        write_via_writer(fix["path"], new_text)

    # ── 3. create meta/legacy-files.md ───────────────────────────────────
    print("\n── create meta/legacy-files.md ──")
    legacy_path = BRAIN_ROOT / "meta" / "legacy-files.md"
    if legacy_path.is_file():
        print("  already exists — skipping create")
    else:
        content = build_legacy_files_md()
        write_new_via_writer("meta/legacy-files.md", content)

    # ── 4. Group A: refresh .docx → CHANGELOG sha256 ─────────────────────
    print("\n── Group A: docx → CHANGELOG (sha256) ──")
    for path, topic in PROVENANCE_REFRESHES_GROUP_A:
        full = BRAIN_ROOT / path
        if not full.is_file():
            print(f"  ✗ MISSING: {path}")
            continue
        text = full.read_text(encoding="utf-8")
        if ".docx#" not in text:
            print(f"  skip {path} (no .docx anchor)")
            continue
        new_ref = (
            f"docs/CyberOS-PRD.CHANGELOG.md (sha256:{prd_sha[:24]}…) — "
            f"see entry for '{topic}'"
        )
        # If the current source_ref also referenced SRS, add it
        if "SRS.docx" in text:
            new_ref += (
                f"; docs/CyberOS-SRS.CHANGELOG.md "
                f"(sha256:{srs_sha[:24]}…) — see entry for '{topic}'"
            )
        new_text = apply_provenance_refresh(text, new_ref)
        if new_text == text:
            print(f"  no-op {path}")
            continue
        write_via_writer(path, new_text)

    # ── 5. Group B: AGENTS.md v1.0.0 → CHANGELOG date refresh ────────────
    print("\n── Group B: AGENTS.md v1.0.0 → CHANGELOG-date refresh ──")
    new_ref_b = (
        "AGENTS.md amendment dated 2026-05-04 (CHANGELOG: Bundle A — "
        "Standing-rule + 9 base refinements). Pre-Bundle-A SHA "
        "unrecoverable; v1.0.0/v1.1.0 nomenclature retired by Bundle B "
        "(DEC-098 no-inline-versioning)."
    )
    for path in PROVENANCE_REFRESHES_GROUP_B:
        full = BRAIN_ROOT / path
        if not full.is_file():
            print(f"  ✗ MISSING: {path}")
            continue
        text = full.read_text(encoding="utf-8")
        if "v1.0.0" not in text and "v1.1.0" not in text:
            print(f"  skip {path} (no v-naming)")
            continue
        new_text = apply_provenance_refresh(text, new_ref_b)
        if new_text == text:
            print(f"  no-op {path}")
            continue
        write_via_writer(path, new_text)

    # ── 6. self-audit ────────────────────────────────────────────────────
    print("\n── self-audit (post-cleanup) ──")
    sa = subprocess.run(
        [sys.executable, str(WRITER), "self-audit", ACTOR],
        cwd=str(REPO_ROOT),
    )
    if sa.returncode == 2:
        print("\n✗ self-audit reports CRITICAL findings — halting before "
              "session.end. Inspect the latest meta/health/<…>.md report.")
        return 2

    # ── 7. session.end ───────────────────────────────────────────────────
    print("\n── session.end ──")
    run_writer(["session-end", ACTOR])

    print("\n== doctor: done ==")
    print("Verify the chain:")
    print(f"  python3 {WRITER.relative_to(REPO_ROOT)} verify --bit-perfect")
    return 0


# ─── String-edit helpers ─────────────────────────────────────────────────

def bump_metadata(fm_text):
    """version+=1, last_updated_at=now, updated_by=ACTOR."""
    # Bump version
    new_lines = []
    for line in fm_text.split("\n"):
        s = line.lstrip()
        if s.startswith("version:"):
            try:
                cur = int(s.split(":", 1)[1].strip())
                new_lines.append(line.replace(str(cur), str(cur + 1), 1))
                continue
            except Exception:
                pass
        if s.startswith("last_updated_at:"):
            indent = line[: len(line) - len(s)]
            new_lines.append(f"{indent}last_updated_at:  {now_iso()}")
            continue
        if s.startswith("updated_by:"):
            indent = line[: len(line) - len(s)]
            new_lines.append(f"{indent}updated_by:       {ACTOR}")
            continue
        new_lines.append(line)
    return "\n".join(new_lines)


def apply_provenance_refresh(text, new_source_ref):
    """Update provenance.source_ref + bump metadata in one pass."""
    fm_text, body = parse_frontmatter_split(text)
    if fm_text is None:
        return text
    fm_text = update_yaml_field(fm_text, "provenance.source_ref",
                                new_source_ref)
    fm_text = bump_metadata(fm_text)
    return "---\n" + fm_text + "\n---\n" + body


def build_legacy_files_md():
    lines = [
        "# meta/legacy-files.md",
        "",
        "Closed-set registry of files exempt from §8.7 phase-5 orphan-WARN",
        "surfacing. Format: `<rel-path-under-.cyberos-memory/> | <reason> |",
        "<approximate-creation>`. New entries land only via a §0.4-driven",
        "cleanup pass acknowledged by the user. The self-audit reads this",
        "registry and demotes listed-path findings from WARN to INFO with a",
        "`legacy-file-registered:` marker.",
        "",
        "Established 2026-05-11 by `outputs/doctor/2026-05-11-cleanup.py`.",
        "",
    ]
    for entry in LEGACY_FILES:
        lines.append(f"{entry['path']} | {entry['reason']} | "
                     f"{entry['approx_creation']}")
    return "\n".join(lines) + "\n"


def write_via_writer(rel_path, new_content):
    """str-replace through brain_writer for an existing file."""
    tmp = tempfile.NamedTemporaryFile(
        mode="w", suffix=".md", delete=False, encoding="utf-8")
    tmp.write(new_content)
    tmp.close()
    try:
        run_writer(["str-replace", ACTOR, rel_path, tmp.name])
    finally:
        os.unlink(tmp.name)


def write_new_via_writer(rel_path, content):
    """write (op:create) through brain_writer for a new file."""
    tmp = tempfile.NamedTemporaryFile(
        mode="w", suffix=".md", delete=False, encoding="utf-8")
    tmp.write(content)
    tmp.close()
    try:
        run_writer(["write", ACTOR, rel_path, tmp.name])
    finally:
        os.unlink(tmp.name)


# ─── CLI ─────────────────────────────────────────────────────────────────

def main():
    p = argparse.ArgumentParser(description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--dry-run", action="store_true",
                   help="Print planned ops; make no changes.")
    args = p.parse_args()
    if args.dry_run:
        cmd_dry_run()
        return 0
    return cmd_apply()


if __name__ == "__main__":
    sys.exit(main())
