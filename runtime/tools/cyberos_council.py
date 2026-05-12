#!/usr/bin/env python3
"""
cyberos_council.py — opt-in council-mode synthesis for ambiguous REFs.

Aspect 3.3 of the Layer-1 improvement catalog.

Council mode runs 4 voices over a draft REF:
  - Architect    (structural fit with existing protocol)
  - Skeptic      (what breaks; failure modes; perverse incentives)
  - Pragmatist   (implementation cost; minimal viable variant)
  - Critic       (voice-standard + AGENTS.md alignment)

Each voice produces a section. The tool also performs deterministic
heuristic checks:
  - Cross-reference with company/locked-decisions.md (LOCK conflicts)
  - GLOSSARY (FACT-014) term overlap
  - Related REFs (same tag / same § anchor)
  - Recent rejected.md entries (prior art against this idea)

Output is staged at .cyberos-memory/cache/council/REF-NNN-council.md and is NOT
written to the BRAIN — it is a working artefact for the human author
to use when finalising the REF.

The 4 voice sections in the output are templated prompts; the operator
either fills them by hand or pipes them to Claude (or another LLM) and
pastes the responses back. Council mode itself does NOT call any LLM.

Usage:
    cyberos council REF-042
    cyberos council memories/refinements/REF-042-brain-writer-recompute-memory-count.md

Design note: keep council mode opt-in. It is not the default code-path
for every REF; only REFs flagged ambiguous by the author should pay
the API cost (~4× regular review).
"""
from __future__ import annotations
import argparse
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def parse_frontmatter(text: str) -> tuple[dict, str]:
    """Return (frontmatter_dict, body) from a memory file. Frontmatter parsed by yaml if available."""
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}, text
    fm_text = text[4:end]
    body = text[end + 5:]
    try:
        import yaml
        return yaml.safe_load(fm_text) or {}, body
    except Exception:
        # Fallback: line-by-line tag extraction
        fm = {}
        for line in fm_text.splitlines():
            m = re.match(r"^([a-z_]+):\s*(.+?)\s*$", line)
            if m:
                fm[m.group(1)] = m.group(2)
        return fm, body


def resolve_ref(brain_root: Path, ref_arg: str) -> Path:
    """Accept either REF-NNN shorthand or full path."""
    p = Path(ref_arg)
    if p.is_file():
        return p
    # Try as REF-NNN
    m = re.match(r"^REF-(\d{3})", ref_arg)
    if m:
        nnn = m.group(1)
        candidates = list((brain_root / ".cyberos-memory" / "memories" / "refinements").glob(f"REF-{nnn}-*.md"))
        if len(candidates) == 1:
            return candidates[0]
        if len(candidates) > 1:
            raise SystemExit(f"ambiguous REF-{nnn}: {[c.name for c in candidates]}")
    raise SystemExit(f"could not resolve {ref_arg!r}; pass full path or REF-NNN")


def load_locked_decisions(brain_root: Path) -> list[str]:
    p = brain_root / ".cyberos-memory" / "company" / "locked-decisions.md"
    if not p.exists():
        return []
    text = p.read_text(encoding="utf-8")
    # Each LOCK is a line beginning with "LOCK-NNN"
    return re.findall(r"^LOCK-\d{3}.*$", text, flags=re.MULTILINE)


def load_glossary(brain_root: Path) -> dict[str, str]:
    """Read FACT-014 GLOSSARY entries — return {term: short-definition}."""
    p = brain_root / ".cyberos-memory" / "memories" / "facts" / "FACT-014-glossary.md"
    if not p.exists():
        # Try fallback location
        for cand in (brain_root / ".cyberos-memory" / "memories" / "facts").glob("FACT-*glossary*.md"):
            p = cand
            break
    if not p.exists() or not p.is_file():
        return {}
    text = p.read_text(encoding="utf-8")
    terms = {}
    # Format A: "## Term\ndefinition"
    for m in re.finditer(r"^##\s+([A-Za-z][\w\s/-]*?)\s*\n([^\n#].+?)(?=\n##|\n---|\Z)", text, flags=re.MULTILINE | re.DOTALL):
        term = m.group(1).strip().lower()
        defn = re.sub(r"\s+", " ", m.group(2)).strip()
        terms[term] = defn[:200]
    # Format B: "**term** — definition" lines (em-dash, en-dash, or hyphen-space-hyphen)
    for m in re.finditer(r"^\*\*([A-Za-z][\w\s/-]*)\*\*\s*[—–-]\s*(.+?)$", text, flags=re.MULTILINE):
        term = m.group(1).strip().lower()
        defn = m.group(2).strip()
        terms.setdefault(term, defn[:200])
    return terms


def find_related_refs(brain_root: Path, tags: list[str], exclude: Path) -> list[tuple[str, str]]:
    """Return list of (filename, first_tag_match) for REFs that share tags."""
    refs_dir = brain_root / ".cyberos-memory" / "memories" / "refinements"
    if not refs_dir.exists():
        return []
    out = []
    tag_set = set(tags) - {"refinement", "tier-1", "tier-2", "tier-3"}  # too generic
    for ref in sorted(refs_dir.glob("REF-*.md")):
        if ref.resolve() == exclude.resolve():
            continue
        try:
            fm, _ = parse_frontmatter(ref.read_text(encoding="utf-8"))
        except Exception:
            continue
        ref_tags = fm.get("tags", []) or []
        if isinstance(ref_tags, str):
            ref_tags = [t.strip() for t in ref_tags.strip("[]").split(",")]
        shared = tag_set & set(ref_tags)
        if shared:
            out.append((ref.name, ", ".join(sorted(shared))))
    return out[:10]


def find_recent_rejected(brain_root: Path) -> list[str]:
    """List rejected.md entries from the last 90 days."""
    rej_dir = brain_root / ".cyberos-memory" / "rejected"
    if not rej_dir.exists():
        return []
    cutoff = datetime.now(ICT) - timedelta(days=90)
    out = []
    for r in sorted(rej_dir.glob("**/*.md")):
        try:
            mtime = datetime.fromtimestamp(r.stat().st_mtime, tz=ICT)
            if mtime >= cutoff:
                out.append(r.relative_to(brain_root).as_posix())
        except Exception:
            continue
    return out[:5]


VOICE_TEMPLATES = {
    "architect": """
### Voice: Architect

**Prompt to feed Claude (or another LLM):**

```
You are reviewing a CyberOS BRAIN refinement (REF-{nnn}). Your role is
the **Architect**. Evaluate this REF strictly through the lens of
structural fit with the existing AGENTS.md protocol.

Read the REF body below. Answer:
1. Which AGENTS.md section does this modify (§-number)? Is the change
   confined to one section or does it ripple through multiple?
2. Are the field/schema additions backward-compatible? Would a pre-REF
   manifest fail to validate post-REF?
3. Does this introduce new invariants? List them. Are existing
   invariants still satisfied?
4. What is the smallest set of code changes that delivers the
   capability described? (Architect-minimal, not pragmatist-minimal.)
5. Are there protocol-amendment bundles already in flight that would
   conflict? (Check AGENTS.CHANGELOG.md.)

REF body:
---
{body}
---

Respond in ≤300 words. No em dashes. No AI-vocab. Plain English.
```

**Council finding:** _(paste Claude's response here, or your own
analysis if you're filling this in by hand)_

""",

    "skeptic": """
### Voice: Skeptic

**Prompt to feed Claude (or another LLM):**

```
You are reviewing a CyberOS BRAIN refinement (REF-{nnn}). Your role is
the **Skeptic**. Find what breaks.

Read the REF body below. Answer:
1. What is the worst case this REF enables? (Adversarial user, buggy
   tool, accidental data loss, malicious co-author.)
2. What perverse incentive does this create for future authors? (Does
   it reward gaming a metric, hiding mistakes, skipping reconciliation?)
3. What happens at scale? (N=1,000 memories. N=100,000. Multiple
   subjects writing concurrently.)
4. What edge case is the author NOT thinking about? (Power-loss mid
   write; clock skew; symlinks; UTF-8 normalisation; ACL changes.)
5. What is the rollback path if this REF turns out wrong 3 months
   later? Is the change reversible?

REF body:
---
{body}
---

Respond in ≤300 words. Be specific. Name the failure mode. No em
dashes. No AI-vocab.
```

**Council finding:** _(paste Claude's response here)_

""",

    "pragmatist": """
### Voice: Pragmatist

**Prompt to feed Claude (or another LLM):**

```
You are reviewing a CyberOS BRAIN refinement (REF-{nnn}). Your role is
the **Pragmatist**. Find the smallest possible variant.

Read the REF body below. Answer:
1. What is the 80/20 cut? Could 80% of the value ship in 20% of the
   effort by trimming a specific scope?
2. Which steps in "Implementation steps" can be deferred to a follow-up
   REF without losing the core?
3. Could this be a runtime tool / hook instead of a protocol change?
   (Tools are cheaper to revert than protocol amendments.)
4. Estimate implementation effort: <1h, <4h, <1d, <1wk, >1wk. Show
   your reasoning.
5. Is there a 3-line fix that gets the user unblocked TODAY while the
   full REF lands later?

REF body:
---
{body}
---

Respond in ≤300 words. Concrete. No em dashes. No AI-vocab.
```

**Council finding:** _(paste Claude's response here)_

""",

    "critic": """
### Voice: Critic

**Prompt to feed Claude (or another LLM):**

```
You are reviewing a CyberOS BRAIN refinement (REF-{nnn}). Your role is
the **Critic**. Audit the writing itself against the gstack /codex
voice standard and AGENTS.md §0.4 (propose-adopt-record cycle).

Read the REF body below. Answer:
1. Does the REF cite a TRIGGER concretely (a real incident, not a
   hypothetical)?
2. Does it state TIER (1/2/3) and justify it?
3. Does the implementation-steps section give exact code or exact
   file/section edits? (Vague language → bounce.)
4. Does the language follow the gstack /codex voice? (No em dashes;
   no AI-vocab like "leverage", "robust", "ensure", "comprehensive",
   "seamless", "delve", "navigate", "tapestry"; no marketing.)
5. List every voice-standard violation with line number.

REF body:
---
{body}
---

Respond in ≤300 words. Be picky.
```

**Council finding:** _(paste Claude's response here)_

""",
}


SYNTHESIS_TEMPLATE = """## Synthesis (author fills after collecting voices)

After all 4 voices have weighed in, the author writes the synthesis:

**Verdict:** [ACCEPT as written / ACCEPT with modifications / DEFER / REJECT]

**Modifications:** _(if ACCEPT-with-mods, list every change to the REF
body in diff form)_

**Why this verdict:** _(1-2 paragraphs)_

**Cross-references to council voices:**
- Architect raised: ...
- Skeptic raised: ...
- Pragmatist raised: ...
- Critic raised: ...

**Next action:** _(specific action: e.g. "amend REF body, re-run
`cyberos verify`, then `cyberos eval REF-{nnn}`")_

---

_This file is a working artefact at .cyberos-memory/cache/council/REF-{nnn}-council.md.
It is NOT a memory in the BRAIN. Once synthesis is complete and the
REF body is updated, this file can be archived or deleted._
"""


def run_voices_live(prompts: dict[str, str], model: str = "claude-sonnet-4-6") -> dict[str, str]:
    """Batch 11 (Tier A) — call Claude for each voice. Returns {voice: response_text}.

    Requires `pip install anthropic` and ANTHROPIC_API_KEY env var. Falls
    back to "(no anthropic SDK; paste manually)" stubs when either is
    missing so the operator surface degrades cleanly.
    """
    try:
        import anthropic  # type: ignore
    except ImportError:
        return {v: "_(anthropic SDK not installed; paste prompt into a fresh Claude conversation by hand)_" for v in prompts}
    import os as _os
    if not _os.environ.get("ANTHROPIC_API_KEY"):
        return {v: "_(ANTHROPIC_API_KEY not set; paste prompt by hand)_" for v in prompts}
    client = anthropic.Anthropic()
    out = {}
    for voice, prompt in prompts.items():
        try:
            msg = client.messages.create(
                model=model,
                max_tokens=600,
                messages=[{"role": "user", "content": prompt}],
            )
            out[voice] = "\n".join(b.text for b in msg.content if hasattr(b, "text"))
        except Exception as e:
            out[voice] = f"_(API error: {e})_"
    return out


def main():
    p = argparse.ArgumentParser(description="Opt-in council-mode synthesis for ambiguous REFs")
    p.add_argument("ref", help="REF-NNN or full path to a refinement file")
    p.add_argument("--voices", default="architect,skeptic,pragmatist,critic",
                   help="comma-separated subset of voices (default: all 4)")
    p.add_argument("--print", action="store_true", help="print to stdout instead of writing file")
    p.add_argument("--run-now", action="store_true",
                   help="actually call Claude for each voice (requires anthropic SDK + ANTHROPIC_API_KEY)")
    p.add_argument("--model", default="claude-sonnet-4-6", help="model for --run-now")
    args = p.parse_args()

    brain_root = find_brain()
    ref_path = resolve_ref(brain_root, args.ref)

    text = ref_path.read_text(encoding="utf-8")
    fm, body = parse_frontmatter(text)

    nnn_match = re.search(r"REF-(\d{3})", ref_path.name)
    if not nnn_match:
        print(f"ERROR: {ref_path.name} does not match REF-NNN pattern", file=sys.stderr)
        return 2
    nnn = nnn_match.group(1)

    tags = fm.get("tags", []) or []
    if isinstance(tags, str):
        tags = [t.strip() for t in tags.strip("[]").split(",")]

    # Heuristic checks
    locked = load_locked_decisions(brain_root)
    glossary = load_glossary(brain_root)
    related = find_related_refs(brain_root, tags, ref_path)
    rejected_recent = find_recent_rejected(brain_root)

    # Cross-check body terms against GLOSSARY
    body_lower = body.lower()
    glossary_hits = [(term, defn) for term, defn in glossary.items() if term in body_lower]
    glossary_hits = glossary_hits[:8]

    # Cross-check body for LOCK conflicts (simple keyword scan)
    lock_hits = []
    for lock_line in locked:
        # extract first 60 chars as a fingerprint
        fp = lock_line[:60].lower()
        # very rough: if 4+ word-substring appears
        words = [w for w in re.findall(r"\b[a-z]{4,}\b", fp)]
        if len(words) >= 3 and all(w in body_lower for w in words[:3]):
            lock_hits.append(lock_line)
    lock_hits = lock_hits[:5]

    # Build output
    voices = [v.strip() for v in args.voices.split(",") if v.strip() in VOICE_TEMPLATES]
    if not voices:
        print(f"ERROR: no valid voices in --voices={args.voices}", file=sys.stderr)
        return 2

    ts = datetime.now(ICT).isoformat(timespec="seconds")
    out_lines = []
    out_lines.append(f"# Council session — REF-{nnn}")
    out_lines.append("")
    out_lines.append(f"**Subject REF:** `{ref_path.relative_to(brain_root)}`")
    out_lines.append(f"**Generated:** {ts}")
    out_lines.append(f"**Voices:** {', '.join(voices)}")
    out_lines.append("")
    out_lines.append("## Heuristic context (deterministic, no LLM)")
    out_lines.append("")
    out_lines.append(f"**Tags on this REF:** {', '.join(tags) if tags else '_(none)_'}")
    out_lines.append("")
    if glossary_hits:
        out_lines.append("**GLOSSARY terms used:**")
        for term, defn in glossary_hits:
            out_lines.append(f"- **{term}** — {defn}")
        out_lines.append("")
    else:
        out_lines.append("_(no GLOSSARY term overlap — REF may be introducing new vocabulary)_")
        out_lines.append("")
    if lock_hits:
        out_lines.append("**Possible LOCK conflicts (review manually):**")
        for lh in lock_hits:
            out_lines.append(f"- {lh}")
        out_lines.append("")
    else:
        out_lines.append("_(no obvious locked-decisions.md conflicts)_")
        out_lines.append("")
    if related:
        out_lines.append("**Related REFs (shared tags):**")
        for fname, shared in related:
            out_lines.append(f"- `{fname}` — shares: {shared}")
        out_lines.append("")
    if rejected_recent:
        out_lines.append("**Recent rejected/ entries (90d) — check for prior art:**")
        for r in rejected_recent:
            out_lines.append(f"- `{r}`")
        out_lines.append("")
    out_lines.append("---")
    out_lines.append("")
    out_lines.append("## Voice prompts (feed each to a fresh Claude conversation)")
    out_lines.append("")

    # Aspect-A5 — optionally call Claude for each voice
    voice_responses: dict[str, str] = {}
    if args.run_now:
        prompts = {v: VOICE_TEMPLATES[v].format(nnn=nnn, body=body.strip()) for v in voices}
        voice_responses = run_voices_live(prompts, model=args.model)
        print(f"  ran {len(voice_responses)} voice(s) live via {args.model}")

    for v in voices:
        section = VOICE_TEMPLATES[v].format(nnn=nnn, body=body.strip()).strip()
        if v in voice_responses:
            # Replace the "_(paste here)_" placeholder with the real response
            section = section.replace(
                "**Council finding:** _(paste Claude's response here, or your own\nanalysis if you're filling this in by hand)_",
                f"**Council finding (live, {args.model}):**\n\n{voice_responses[v]}",
            ).replace(
                "**Council finding:** _(paste Claude's response here)_",
                f"**Council finding (live, {args.model}):**\n\n{voice_responses[v]}",
            )
        out_lines.append(section)
        out_lines.append("")
        out_lines.append("---")
        out_lines.append("")

    out_lines.append(SYNTHESIS_TEMPLATE.format(nnn=nnn).strip())

    out_text = "\n".join(out_lines) + "\n"

    if args.print:
        sys.stdout.write(out_text)
        return 0

    out_dir = brain_root / "outputs" / "council"
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / f"REF-{nnn}-council.md"
    out_path.write_text(out_text, encoding="utf-8")
    rel = out_path.relative_to(brain_root)
    print(f"  ✓ council session staged: {rel}")
    print(f"  Voices: {', '.join(voices)}")
    print(f"  Heuristic context: glossary={len(glossary_hits)}, locks={len(lock_hits)}, related-refs={len(related)}, rejected={len(rejected_recent)}")
    print()
    print(f"  Next:")
    print(f"    1. Open {rel}")
    print(f"    2. Feed each Voice prompt to a fresh Claude conversation (4× API cost)")
    print(f"    3. Paste responses back into 'Council finding' sections")
    print(f"    4. Write the Synthesis section")
    print(f"    5. Amend the REF body if voices flagged issues")
    print(f"    6. Re-run: cyberos verify && cyberos eval REF-{nnn}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
