#!/usr/bin/env python3
"""
cyberos_add.py — interactive memory creation wizard.

Aspect 1.2 + 4.1 + 12.4 of the Layer-1 improvement catalog.

Reads templates from `.cyberos-memory/meta/templates/<TYPE>.md`, asks for
required fields via interactive prompts, fills variables (UUID7, timestamps,
subject ID, next NNN), stages the result, then invokes brain_writer.write.

Usage:
    cyberos add DEC           # interactive DEC wizard
    cyberos add REF           # interactive REF wizard
    cyberos add FACT          # interactive FACT wizard
    cyberos add PERSON        # interactive PERSON wizard
    cyberos add PROJECT       # interactive PROJECT wizard
    cyberos add PREFERENCE    # interactive PREFERENCE wizard
    cyberos add DRIFT         # auto-generated normally; manual for testing

    cyberos add DEC --slug pricing-tiers --classification operational \
        --authority human-edited --tags pricing,starter,enterprise

Variables filled automatically (no prompt):
    ${UUID7}, ${TS_NOW}, ${SUBJECT_ID} (from $CYBEROS_SUBJECT_ID or git config),
    ${NEXT_NNN} (max existing NNN + 1, monotonic per bucket),
    ${SLUG_TITLE} (slug with hyphens → spaces, capitalised)

Variables prompted (interactive) or accepted via CLI flag:
    ${SLUG}, ${CLASSIFICATION}, ${AUTHORITY}, ${TAGS}, ${PROV_SOURCE},
    ${PROV_SOURCE_REF}, ${SYNC_CLASS}, ${FRESHNESS_TIER}
"""
from __future__ import annotations
import argparse
import os
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))

# Bucket map
BUCKET = {
    "DEC": ("memories/decisions", "DEC", "memories/decisions"),
    "REF": ("memories/refinements", "REF", "memories/refinements"),
    "FACT": ("memories/facts", "FACT", "memories/facts"),
    "PERSON": ("memories/people", "PERSON", "memories/people"),
    "PROJECT": ("memories/projects", "PROJECT", "memories/projects"),
    "PREFERENCE": ("memories/preferences", "PREF", "memories/preferences"),
    "PREF": ("memories/preferences", "PREF", "memories/preferences"),
    "DRIFT": ("memories/drift", "DRIFT", "memories/drift"),
}

def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")

def next_nnn(brain_root: Path, bucket_path: str) -> str:
    d = brain_root / ".cyberos-memory" / bucket_path
    max_n = 0
    if d.exists():
        for f in d.glob("*.md"):
            m = re.match(r"^[A-Z]+-(\d+)", f.name)
            if m:
                n = int(m.group(1))
                if n > max_n:
                    max_n = n
    return f"{max_n + 1:03d}"

def uuid7_helper(brain_root: Path):
    """Import new_uuid7 from brain_writer."""
    sys.path.insert(0, str(brain_root / "outputs"))
    try:
        from brain_writer import new_uuid7
        return new_uuid7
    except ImportError:
        # Fallback: time-based prefix + secure random
        import secrets, time
        def fallback(prefix):
            ts_ms = int(time.time() * 1000) & ((1 << 48) - 1)
            ra = secrets.randbits(12)
            rb = secrets.randbits(62)
            n = (ts_ms << 80) | (0x7 << 76) | (ra << 64) | (0b10 << 62) | rb
            h = f"{n:032x}"
            return f"{prefix}_{h[:8]}-{h[8:12]}-{h[12:16]}-{h[16:20]}-{h[20:32]}"
        return fallback

def load_persona_defaults(brain_root: Path, persona_name: str = None) -> dict:
    """Read persona_defaults frontmatter from .cyberos-memory/persona/<name>.md.

    Aspect 12.6 — persona-defined defaults. Returns empty dict if persona file
    missing / unparseable / has no persona_defaults block.

    persona_name resolution order:
      1. explicit arg
      2. $CYBEROS_PERSONA env var
      3. None → no defaults
    """
    if not persona_name:
        persona_name = os.environ.get("CYBEROS_PERSONA", "").strip()
    if not persona_name:
        return {}
    p = brain_root / ".cyberos-memory" / "persona" / f"{persona_name}.md"
    if not p.exists():
        return {}
    try:
        text = p.read_text(encoding="utf-8")
        if not text.startswith("---\n"):
            return {}
        end = text.find("\n---\n", 4)
        if end < 0:
            return {}
        import yaml
        fm = yaml.safe_load(text[4:end]) or {}
        return fm.get("persona_defaults", {}) or {}
    except Exception:
        return {}


def detect_subject(brain_root: Path) -> str:
    env = os.environ.get("CYBEROS_SUBJECT_ID")
    if env:
        return f"subject:{env}" if not env.startswith("subject:") else env
    try:
        name = subprocess.check_output(["git", "config", "user.name"],
                                        cwd=brain_root, stderr=subprocess.DEVNULL,
                                        text=True).strip()
        slug = re.sub(r"[^a-z0-9-]+", "-", name.lower()).strip("-")
        return f"subject:{slug}"
    except Exception:
        return "subject:unknown"

def load_glossary_terms(brain_root: Path) -> dict[str, str]:
    """Read FACT-014 GLOSSARY entries — return {term: short-definition}.

    Supports two formats:
      A) `## Term` followed by definition paragraph
      B) `**term** — definition` inline lines
    """
    fact_dir = brain_root / ".cyberos-memory" / "memories" / "facts"
    p = fact_dir / "FACT-014-glossary.md"
    if not p.exists():
        for cand in fact_dir.glob("FACT-*glossary*.md"):
            p = cand
            break
    if not p.exists() or not p.is_file():
        return {}
    text = p.read_text(encoding="utf-8")
    terms = {}
    for m in re.finditer(r"^##\s+([A-Za-z][\w\s/-]*?)\s*\n([^\n#].+?)(?=\n##|\n---|\Z)",
                         text, flags=re.MULTILINE | re.DOTALL):
        term = m.group(1).strip().lower()
        defn = re.sub(r"\s+", " ", m.group(2)).strip()
        terms[term] = defn[:200]
    for m in re.finditer(r"^\*\*([A-Za-z][\w\s/-]*)\*\*\s*[—–-]\s*(.+?)$",
                         text, flags=re.MULTILINE):
        term = m.group(1).strip().lower()
        defn = m.group(2).strip()
        terms.setdefault(term, defn[:200])
    return terms


def suggest_tags(text: str, glossary: dict[str, str], existing: list[str], max_new: int = 5) -> list[str]:
    """Suggest kebab-case tags from GLOSSARY terms appearing in text.

    Heuristic:
      - Lowercase the haystack
      - Match each GLOSSARY term as whole-word
      - Convert term to kebab-case (`drift candidate` → `drift-candidate`)
      - Drop generic terms (too noisy) and any tag already present
      - Cap at `max_new`
    """
    GENERIC = {"brain", "memory", "field", "section", "file", "system", "tag"}
    hay = text.lower()
    out = []
    for term in sorted(glossary, key=lambda t: -len(t)):  # longer matches first
        if term in GENERIC:
            continue
        # Whole-word boundary check
        if not re.search(r"\b" + re.escape(term) + r"\b", hay):
            continue
        kebab = re.sub(r"\s+", "-", term).strip("-")
        if kebab in existing or kebab in out:
            continue
        out.append(kebab)
        if len(out) >= max_new:
            break
    return out


def ask(prompt, default=None, choices=None, validator=None, non_interactive=False):
    if non_interactive:
        if default is not None:
            return default
        print(f"non-interactive + no default: {prompt}", file=sys.stderr)
        sys.exit(2)
    d_str = f" [{default}]" if default else ""
    c_str = f" ({'/'.join(choices)})" if choices else ""
    while True:
        r = input(f"  ? {prompt}{c_str}{d_str}: ").strip()
        if not r and default is not None:
            r = default
        if choices and r not in choices:
            print(f"    must be one of: {choices}")
            continue
        if validator and not validator(r):
            print(f"    invalid; try again")
            continue
        return r

def main():
    p = argparse.ArgumentParser(description="Interactive memory creation wizard")
    p.add_argument("type", help="DEC | REF | FACT | PERSON | PROJECT | PREFERENCE")
    p.add_argument("--slug", help="lowercase-kebab slug")
    p.add_argument("--classification", choices=["personnel", "client", "operational", "public"])
    p.add_argument("--authority", choices=["human-edited", "human-confirmed", "llm-explicit", "llm-implicit"])
    p.add_argument("--tags", help="comma-separated kebab-case tags")
    p.add_argument("--sync-class", choices=["local-only", "publishable", "shared", "client-visible"])
    p.add_argument("--prov-source", choices=["chat", "doc", "code", "inference", "manual", "imported", "conflict_resolution"])
    p.add_argument("--prov-source-ref", help="opaque source reference")
    p.add_argument("--freshness-tier", type=int, help="source freshness tier (1=most authoritative)")
    p.add_argument("--non-interactive", action="store_true", help="fail if any input missing")
    p.add_argument("--dry-run", action="store_true", help="show resulting file, don't write")
    p.add_argument("--auto-tags", action="store_true",
                   help="suggest tags from FACT-014 GLOSSARY (opt-in; reviewed before write)")
    p.add_argument("--auto-tags-max", type=int, default=5,
                   help="cap auto-suggested tags (default 5)")
    p.add_argument("--persona", help="apply persona/<name>.md defaults (Aspect 12.6)")
    args = p.parse_args()

    type_key = args.type.upper()
    if type_key not in BUCKET:
        print(f"ERROR: unknown type {args.type}; valid: {list(BUCKET.keys())}", file=sys.stderr)
        return 2

    brain_root = find_brain()
    bucket_path, prefix, _ = BUCKET[type_key]
    template_path = brain_root / "outputs" / "templates" / f"{prefix if prefix != 'PREF' else 'PREFERENCE'}.md"
    if not template_path.exists():
        print(f"ERROR: template not found: {template_path}", file=sys.stderr)
        return 2

    # Aspect 12.6 — persona-defined defaults
    persona_defaults = load_persona_defaults(brain_root, args.persona)
    if persona_defaults:
        print(f"\n  Persona defaults loaded ({args.persona or os.environ.get('CYBEROS_PERSONA')}): "
              f"{', '.join(f'{k}={v}' for k, v in persona_defaults.items() if isinstance(v, (str, int, bool)))}")

    print(f"\n  Creating {type_key} memory in {bucket_path}/\n")

    # Gather inputs (persona defaults applied where CLI flag absent)
    pd_cls = persona_defaults.get("default_classification", "operational")
    pd_auth = persona_defaults.get("default_authority", "human-edited")
    pd_sync = persona_defaults.get("default_sync_class", "publishable" if type_key != "PERSON" else "local-only")

    slug = args.slug or ask("slug (lowercase-kebab)", validator=lambda x: bool(re.match(r"^[a-z][a-z0-9-]*$", x)), non_interactive=args.non_interactive)
    classification = args.classification or ask("classification", default=pd_cls, choices=["personnel", "client", "operational", "public"], non_interactive=args.non_interactive)
    authority = args.authority or ask("authority", default=pd_auth, choices=["human-edited", "human-confirmed", "llm-explicit", "llm-implicit"], non_interactive=args.non_interactive)
    tags_str = args.tags or ask("tags (comma-separated kebab-case)", default="", non_interactive=args.non_interactive)
    tags_list = [t.strip() for t in tags_str.split(",") if t.strip()] if tags_str else []
    sync_class = args.sync_class or ask("sync_class", default=pd_sync, choices=["local-only", "publishable", "shared", "client-visible"], non_interactive=args.non_interactive)
    prov_source = args.prov_source or ask("provenance source", default="chat", choices=["chat", "doc", "code", "inference", "manual", "imported", "conflict_resolution"], non_interactive=args.non_interactive)
    prov_source_ref = args.prov_source_ref or ask("provenance source_ref", default="cyberos add wizard", non_interactive=args.non_interactive)
    freshness_tier = args.freshness_tier or int(ask("source_freshness_tier (lower = more authoritative)", default="10", non_interactive=args.non_interactive))

    # Auto-supersedes detection (Batch 12 / Tier B)
    if not args.non_interactive:
        try:
            stem = slug.replace("-", " ")
            bucket_dir = brain_root / ".cyberos-memory" / bucket_path
            candidates = []
            if bucket_dir.exists():
                for ex in bucket_dir.glob("*.md"):
                    ex_stem = re.sub(r"^[A-Z]+-\d+-?", "", ex.stem).replace("-", " ")
                    # Simple substring match either direction
                    if ex_stem and (ex_stem in stem or stem in ex_stem):
                        if ex_stem != stem:  # skip self
                            candidates.append(ex.name)
            if candidates[:3]:
                print(f"\n  Auto-supersedes scan found {len(candidates)} similar memo(s) in {bucket_path}:")
                for c in candidates[:3]:
                    print(f"    - {c}")
                print(f"  (set `supersedes:` in your frontmatter manually if any of these should be replaced)")
        except Exception:
            pass

    # Auto-fill
    new_uuid7 = uuid7_helper(brain_root)
    uuid = new_uuid7("mem")
    ts_now = datetime.now(ICT).isoformat(timespec='seconds')
    subject = detect_subject(brain_root)
    nnn = next_nnn(brain_root, bucket_path)
    slug_title = slug.replace("-", " ").title()

    # Opt-in: GLOSSARY auto-tagging (Aspect 5.2)
    if args.auto_tags:
        glossary = load_glossary_terms(brain_root)
        haystack = " ".join([slug, slug_title, prov_source_ref])
        suggested = suggest_tags(haystack, glossary, tags_list, max_new=args.auto_tags_max)
        if suggested:
            print(f"  Auto-tag suggestions from GLOSSARY: {', '.join(suggested)}")
            if not args.non_interactive:
                accept = input(f"  Accept all? [Y/n/edit]: ").strip().lower() or "y"
                if accept == "n":
                    suggested = []
                elif accept in ("e", "edit"):
                    keep_str = input(f"    Keep which (comma-separated, blank = all): ").strip()
                    if keep_str:
                        wanted = {t.strip() for t in keep_str.split(",") if t.strip()}
                        suggested = [t for t in suggested if t in wanted]
            tags_list = tags_list + suggested
        else:
            print(f"  Auto-tag: no GLOSSARY hits in slug + title")

    # Load template
    text = template_path.read_text()

    # Substitute variables — handle ${VAR:default} and ${VAR}
    substitutions = {
        "UUID7": uuid.replace("mem_", ""),  # template prefixes mem_ separately
        "TS_NOW": ts_now,
        "SUBJECT_ID": subject,
        "NEXT_NNN": nnn,
        "SLUG_TITLE": slug_title,
        "SLUG": slug,
        "CLASSIFICATION": classification,
        "AUTHORITY": authority,
        "TAGS": ", ".join(f'"{t}"' if " " in t else t for t in tags_list),
        "SYNC_CLASS": sync_class,
        "PROV_SOURCE": prov_source,
        "PROV_SOURCE_REF": prov_source_ref,
        "FRESHNESS_TIER": str(freshness_tier),
        "CONFIDENCE": "1.0",
        "CONSENT_EVENT_ID": "null",
        "IMPLEMENTS_DEC_ID": "null",
        "IMPLEMENTS_DEC_NNN": "NNN",
        "SOURCE_PATH": prov_source_ref,
        "SOURCE_SHA": "null",
        "SOURCE_LINES": "null",
        "PROCESSED_LINES": "null",
        "INCIDENT_TS": ts_now,
        "TS_SIGNAL": ts_now,
        "TS_MISS": "TBD",
        "TS_DETECT": "TBD",
        "TS_FIX": ts_now,
        "ORIGINAL_MEMORY_ID": "TBD",
        "ORIGINAL_MEMORY_PATH": "TBD",
        "CONSOLIDATION_RUN_ID": "TBD",
        "AGENT_ID": "claude-sonnet-4.7",
        "SOURCE_SHA_BEFORE": "TBD",
        "SOURCE_SHA_AFTER": "TBD",
        "PROJECT_ID": "TBD",
        "START_DATE": ts_now[:10],
        "TARGET_DATE": "TBD",
        "OWNER_SUBJECT_ID": subject.replace("subject:", ""),
        "CLIENT_ID": "TBD",
        "CHAT_TURN_REF": "TBD",
        "DISPLAY_NAME": "TBD",
        "WORK_EMAIL": "TBD",
        "ROLE": "TBD",
        "TZ": "Asia/Ho_Chi_Minh",
        "LANGUAGES": "en",
        "SUBJECT_ID_TARGET": subject.replace("subject:", ""),
    }

    # Replace ${VAR:default} first
    def replace_with_default(m):
        var, default = m.group(1), m.group(2)
        return substitutions.get(var, default)
    text = re.sub(r"\$\{(\w+):([^}]*)\}", replace_with_default, text)
    # Then replace ${VAR}
    def replace_plain(m):
        var = m.group(1)
        return substitutions.get(var, m.group(0))  # leave unfilled if no value
    text = re.sub(r"\$\{(\w+)\}", replace_plain, text)

    # Fix mem_${UUID7} → mem_<actual>
    text = text.replace("mem_" + substitutions["UUID7"], uuid)

    # Convert tags array — template uses [${TAGS}], we generated comma-separated values
    text = text.replace(f"[{substitutions['TAGS']}]", f"[{substitutions['TAGS']}]")

    # Compute target path
    target_rel = f"{bucket_path}/{prefix}-{nnn}-{slug}.md"
    target_abs = brain_root / ".cyberos-memory" / target_rel
    if target_abs.exists():
        print(f"ERROR: target already exists: {target_rel}", file=sys.stderr)
        return 2

    # Stage
    staged = brain_root / "outputs" / "staged-memories" / f"cyberos-add-{prefix}-{nnn}-{slug}.md"
    staged.parent.mkdir(parents=True, exist_ok=True)
    staged.write_text(text)
    print(f"\n  Staged: {staged.relative_to(brain_root)}\n")

    if args.dry_run:
        print(f"  --dry-run: would write to {target_rel}")
        print(f"  Preview:\n{'-'*60}")
        for line in text.split("\n")[:20]:
            print(f"  {line}")
        print(f"  {'-'*60}")
        print(f"  Confirm by re-running without --dry-run, OR:")
        print(f"  python3 runtime/lib/brain_writer.py write {subject} {target_rel} {staged}")
        return 0

    # Confirm before write
    if not args.non_interactive:
        confirm = input(f"  Write to {target_rel}? [y/N]: ").strip().lower()
        if confirm != "y":
            print(f"  cancelled (staged file remains at {staged.relative_to(brain_root)})")
            return 0

    # brain_writer write
    bw = brain_root / "outputs" / "brain_writer.py"
    rc = subprocess.run(["python3", str(bw), "write", subject, target_rel, str(staged)]).returncode
    if rc == 0:
        print(f"\n  ✓ written: {target_rel}")
        # Cleanup staged
        try:
            staged.unlink()
        except Exception:
            pass
    return rc

if __name__ == "__main__":
    sys.exit(main())
