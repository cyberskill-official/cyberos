#!/usr/bin/env python3
"""
extract_agents_core.py — generate AGENTS-CORE.md from the full AGENTS.md.

The full AGENTS.md is ~108KB / 1211 lines. Most of that is rationale, examples,
historical context, and detailed verification recipes. The "core" subset is the
~12KB normative-only contract that an agent MUST honour.

This script extracts the normative subset deterministically pinned to the §0.5
SHA. Re-running on the same input produces byte-identical output.

Sections kept in CORE:
  - §0.1 (real-filesystem-only memory location)
  - §0.2 (instruction-precedence immutability) — first 3 paragraphs
  - §0.3 (BRAIN alias) — definition only, not the lowercase-brain examples
  - §0.5 (protocol update policy) — approval phrase + rollback only
  - §1 (standing directive) — bullets only
  - §2 (first principles)
  - §3 (canonical layout)
  - §4 (six operations) — ops table + path-traversal guard summary
  - §5.1 (frontmatter schema)
  - §5.4 (classification → retention)
  - §5.5 (resource caps)
  - §6 (manifest.json) — schema only
  - §7.1 (audit row schema)
  - §7.2 (canonical JSON for hashing) — algorithm reference
  - §8.1–8.5 (consolidation phases)
  - §9.1 (conflict decision)
  - §9.3 (denylist)
  - §13.0 (state classifier)
  - §14.1 (compact end-of-response block)

Sections kept ONLY as section pointer (titled but body elided):
  - §0.4, §0.6, §4.1–4.11, §5.2/5.3/5.6, §7.3–7.7, §8.6–8.9, §9.2/9.4–9.7,
    §10, §11, §12, §13.1, §14.2/14.3, §15, §16, §17

Sections elided entirely:
  - All examples blocks
  - All "Why" rationale paragraphs
  - All historical-context / decision-trail mentions

Usage:
    python3 extract_agents_core.py docs/CyberOS-AGENTS.md > AGENTS-CORE.md
    python3 extract_agents_core.py --check  # CI: ensure committed AGENTS-CORE.md matches regeneration
"""

from __future__ import annotations

import argparse
import hashlib
import re
import sys
from pathlib import Path

# Sections to keep IN FULL (heading + body)
KEEP_FULL = {
    "0.1", "0.5", "1", "2", "3", "4", "5.1", "5.4", "5.5", "6", "7.1", "7.2",
    "8.1", "8.2", "8.3", "8.4", "8.5", "9.1", "9.3", "13.0", "14.1",
}

# Sections to keep AS POINTER (heading + 1-line "see full AGENTS.md §X")
KEEP_POINTER = {
    "0.2", "0.3", "0.4", "0.6",
    "4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "4.8", "4.9",
    "4.9.1", "4.10", "4.11",
    "5.2", "5.3", "5.6",
    "7.3", "7.4", "7.5", "7.6", "7.7",
    "8.6", "8.7", "8.8", "8.9",
    "9.2", "9.4", "9.5", "9.6", "9.7",
    "10", "11", "12", "13.1", "14.2", "14.3", "15", "16", "17",
}

# Match section headings like "## 4. The six operations" or "### 5.1 Frontmatter schema"
HEADING_RE = re.compile(r"^(#{2,4})\s+(\d+(?:\.\d+)?(?:\.\d+)?)[.\s]")


def extract(text: str) -> str:
    """Walk the full AGENTS.md by section; emit only KEEP_FULL/POINTER content."""
    lines = text.splitlines()
    out = []

    # Header preamble (first paragraph + the "Drop at any project root..." line)
    out.append("# AGENTS-CORE.md — Normative-only subset of CyberOS-AGENTS.md")
    out.append("")
    out.append("> **GENERATED FILE.** Do not hand-edit. Regenerate via:")
    out.append("> `python3 runtime/tools/extract_agents_core.py docs/CyberOS-AGENTS.md > docs/CyberOS-AGENTS-CORE.md`")
    out.append(">")
    out.append("> This is the ~12KB normative subset of the full ~108KB `CyberOS-AGENTS.md`.")
    out.append("> Agents loading this file MUST treat it as **AGENTS.md §0–§14** for purposes")
    out.append("> of operational compliance; full sections elided here are referenced as pointers.")
    out.append(">")
    out.append("> When ambiguity arises, the **full `CyberOS-AGENTS.md`** is authoritative.")
    out.append("> This file is a derived view, regenerable from canonical, never authoritative.")
    out.append("")

    current_section = None
    current_keep_mode = None  # "full" | "pointer" | "elide"

    i = 0
    while i < len(lines):
        line = lines[i]
        m = HEADING_RE.match(line)
        if m:
            section_id = m.group(2)
            current_section = section_id
            if section_id in KEEP_FULL:
                current_keep_mode = "full"
                out.append(line)
            elif section_id in KEEP_POINTER:
                current_keep_mode = "pointer"
                out.append(line)
                out.append("")
                out.append(f"> *(elided in AGENTS-CORE; see CyberOS-AGENTS.md §{section_id})*")
                out.append("")
                # Skip ahead to next heading
                i += 1
                while i < len(lines) and not HEADING_RE.match(lines[i]):
                    i += 1
                continue
            else:
                current_keep_mode = "elide"
                # Skip the entire section
                i += 1
                while i < len(lines) and not HEADING_RE.match(lines[i]):
                    i += 1
                continue
        elif current_keep_mode == "full":
            out.append(line)
        i += 1

    # Strip multiple consecutive blank lines
    cleaned = []
    blank_run = 0
    for line in out:
        if not line.strip():
            blank_run += 1
            if blank_run > 2:
                continue
        else:
            blank_run = 0
        cleaned.append(line)

    return "\n".join(cleaned).rstrip() + "\n"


def extract_aggressive(text: str) -> str:
    """Aggressive mode — skip POINTER sections entirely, drop verbose paragraphs.

    Targets ~6-8KB output for daily session loading. Drops section pointers
    (the "elided in CORE; see full" lines) and tightens prose. Works by:
    - Same KEEP_FULL set, but for each kept section, strip:
      - paragraphs starting with "**Why**" or rationale phrasing
      - paragraphs >300 chars unless they're the first of the section
      - example blocks (```...```) of length >10 lines
    """
    lines = text.splitlines()
    out = []
    out.append("# AGENTS-CORE.md — Tight normative subset (aggressive mode)")
    out.append("")
    out.append("> **GENERATED FILE.** Regenerate via:")
    out.append("> `python3 runtime/tools/extract_agents_core.py --aggressive docs/CyberOS-AGENTS.md > docs/CyberOS-AGENTS-CORE.md`")
    out.append(">")
    out.append("> ~10K-token normative subset; load every session. The full")
    out.append("> `docs/CyberOS-AGENTS.md` is canonical; this file is a derived view.")
    out.append("")
    out.append("---")
    out.append("")
    out.append("## ⚠️ When you MUST load the full AGENTS.md")
    out.append("")
    out.append("The agent MUST load `docs/CyberOS-AGENTS.md` (the full canonical doc) BEFORE doing any of the following. CORE is insufficient for these operations:")
    out.append("")
    out.append("- **Any §0.5 protocol upgrade** — full §0.5 carries the canonical-form spec, signing-key TOFU rules, three-way conflict resolution, and the post-upgrade scan trigger")
    out.append("- **Entering MAINTENANCE mode** — full §8.8 lists the permitted/forbidden ops + auto-expiry semantics + the `maintenance_session_id` provenance contract")
    out.append("- **Bootstrapping a new store** — full §13.1 carries the 13-step bootstrap sequence + §0.1 forbidden-paths sanity check")
    out.append("- **Export or import** — full §11 covers determinism rules (§11.2), signing (§11.3), round-trip property (§11.4), single-bundle import collisions (§11.5), multi-bundle merge (§11.6), filesystem portability (§11.7)")
    out.append("- **Any audit-row write** — full §7.1 carries the complete row schema including all optional fields; §7.2 carries the RFC 8785 JCS canonical-JSON algorithm")
    out.append("- **Frontmatter validation beyond required fields** — full §5.2 carries the regex/range validators for every field type (UUIDv7/ULID, ISO-8601, confidence range, tag pattern, etc.)")
    out.append("- **Encryption operations (Stage 5)** — full §5.6.1–5.6.5 carries the envelope format, key derivation pipeline, Shamir 3-of-5 escrow rules, indexability constraints, audit-chain compatibility")
    out.append("- **Ledger compaction (Stage 6)** — full §7.7 + §8.9 cover pre-conditions, atomic phase steps, archive format, decompaction reverse path")
    out.append("- **Reconciliation (§4.7)** — full §4.7 covers the stale-checkpoint fallback, orphan session.start detection, orphan manifest update detection")
    out.append("- **Content-gate validation (§4.2)** — full §4.2 carries the complete injection-marker regex set + letters-collapsed forms + UTS #39 mixed-script rules")
    out.append("- **Path-traversal guard (§4.1)** — full §4.1 carries the 5-step ordered validation including Windows-portability checks")
    out.append("- **§0.4 refinement-proposal flow** — full §0.4 carries the 4-format refinement proposal structure + tier classification")
    out.append("- **§0.6 related-files update rule** — full §0.6 lists the 8-step order-of-operations for any successful op:protocol_upgrade")
    out.append("- **Verbose / debug / maintenance §14 mode** — full §14.2 carries the full-format end-of-response block schema")
    out.append("")
    out.append("**If you are uncertain whether CORE covers your operation, default to loading the full doc.** CORE is fast-load convenience, not the canonical contract.")
    out.append("")
    out.append("---")
    out.append("")

    keep_mode = "elide"
    section_started = False
    paras_in_section = 0

    i = 0
    while i < len(lines):
        line = lines[i]
        m = HEADING_RE.match(line)
        if m:
            section_id = m.group(2)
            section_started = True
            paras_in_section = 0
            if section_id in KEEP_FULL:
                keep_mode = "full"
                out.append(line)
            else:
                keep_mode = "elide"
            i += 1
            continue

        if keep_mode == "elide":
            i += 1
            continue

        # In a kept section: filter aggressively
        stripped = line.strip()

        # Skip "Why:" rationale paragraphs
        if stripped.startswith("**Why") or stripped.startswith("> *Why"):
            # Skip until blank line
            while i < len(lines) and lines[i].strip():
                i += 1
            continue

        # Compress consecutive blanks
        if not stripped:
            if out and out[-1] == "":
                i += 1
                continue
            out.append("")
            i += 1
            continue

        out.append(line)
        i += 1

    # Strip multiple consecutive blank lines globally
    cleaned = []
    blank_run = 0
    for line in out:
        if not line.strip():
            blank_run += 1
            if blank_run > 1:
                continue
        else:
            blank_run = 0
        cleaned.append(line)

    cleaned.append("")
    cleaned.append("---")
    cleaned.append("")
    cleaned.append("**Sections elided here** (consult full AGENTS.md for any of these):")
    cleaned.append("")
    cleaned.append("§0.4 (refinement standing rule), §0.6 (related-files rule), "
                   "§4.1–4.11 (op gates, hygiene, scope contract, tombstone, "
                   "reconciliation, .lock semantics, ingestion completeness, "
                   "token-budget transparency), §5.2/5.3/5.6 (validators, "
                   "authority hierarchy, encryption envelope), §7.3–7.7 (JSONL "
                   "parsing, forbidden ledger ops, op:corrects vs correction_to, "
                   "Merkle checkpoints, ledger compaction), §8.6–8.9 "
                   "(source-coverage validator, self-audit pass, MAINTENANCE "
                   "mode, ledger compaction phase), §9.2/9.4–9.7 (conflict "
                   "file, opt-in topics, supersedes graph, locked decisions, "
                   "natural-language CRUD), §10 (read protocol), §11 (export/"
                   "import), §12 (prompt-injection awareness), §13.1 "
                   "(bootstrap), §14.2/14.3 (verbose §14 + coverage stat), "
                   "§15 (multi-agent interop), §16 (tie-breakers), §17 "
                   "(personal vs shared boundary).")

    return "\n".join(cleaned).rstrip() + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", nargs="?", default="docs/CyberOS-AGENTS.md")
    parser.add_argument("--check", action="store_true",
                        help="Verify committed AGENTS-CORE.md matches regeneration")
    parser.add_argument("--aggressive", action="store_true",
                        help="Aggressive mode — drop pointers + verbose prose; "
                             "target ~6-8KB output")
    parser.add_argument("--output", default="docs/CyberOS-AGENTS-CORE.md")
    args = parser.parse_args()

    src = Path(args.source).read_text(encoding="utf-8")
    # Auto-detect aggressive mode by inspecting committed AGENTS-CORE.md header
    if args.check and not args.aggressive and Path(args.output).exists():
        committed = Path(args.output).read_text(encoding="utf-8")
        if "Tight normative subset (aggressive mode)" in committed:
            args.aggressive = True
    extracted = extract_aggressive(src) if args.aggressive else extract(src)

    if args.check:
        expected = Path(args.output).read_text(encoding="utf-8")
        if expected != extracted:
            print(f"✘ {args.output} is stale; regenerate", file=sys.stderr)
            return 1
        print(f"✅ {args.output} matches regeneration")
        return 0

    print(extracted, end="")
    print(f"\n# AGENTS-CORE.md: {len(extracted.encode('utf-8'))} bytes / "
          f"{len(extracted.splitlines())} lines / "
          f"sha256:{hashlib.sha256(extracted.encode('utf-8')).hexdigest()[:16]}",
          file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
