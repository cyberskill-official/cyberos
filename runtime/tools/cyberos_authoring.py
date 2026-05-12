#!/usr/bin/env python3
"""
cyberos_authoring.py — Stage 3 authoring quality amplifiers for skill output.

Shared library used by cyberos_chain.py and individual skill runtimes.

Functions:

  llm_draft_body(prompt, model)            — S3.1 — Claude call (anthropic SDK)
                                              with graceful fallback when SDK / key missing
  voice_gate(text)                         — S3.2 — run cyberos_voice on a draft;
                                              return findings (no em dashes, no AI vocab)
  attribute_claims(body, source_text)      — S3.3 — for every claim in body, mark it
                                              human-confirmed if it appears in source_text,
                                              llm-explicit otherwise
  diff_artefact(old_path, new_text)        — S3.4 — unified diff between prior and new
                                              versions of an FR/spec
  interview_questions(persona, mode)       — S3.5 — load per-persona interview templates
                                              from interview-templates/<persona>.md
"""
from __future__ import annotations
import difflib
import os
import re
import sys
from pathlib import Path

# Persona-specific interview question banks. Loaded from disk if available,
# otherwise from the embedded defaults below.

PERSONA_QUESTIONS = {
    "cpo": [
        ("target_sprint", "Which sprint? (current / next / unsequenced)", "unsequenced"),
        ("ai_token_budget", "AI-agent token budget per FR (default 30000)?", "30000"),
        ("risk_tier_ceiling", "Highest acceptable EU AI Act tier? (minimal/limited/high_risk)", "limited"),
    ],
    "cto": [
        ("target_release", "Target release? (this-sprint / next-sprint / next-quarter)", "next-sprint"),
        ("breaking_changes_ok", "Are breaking API changes acceptable? (yes/no)", "no"),
        ("rollback_window", "Required rollback window after release? (hours)", "24"),
    ],
    "cseco": [
        ("threat_model", "Have you done a threat model? (yes/no/will-do)", "no"),
        ("data_classification", "Highest data classification touched? (public/internal/confidential/regulated)", "internal"),
        ("auth_pattern", "Auth pattern? (oauth/api-key/jwt/none)", "oauth"),
    ],
    "clo": [
        ("jurisdictions", "Jurisdictions in scope? (comma-separated; e.g. EU,US,VN)", "VN"),
        ("data_subjects", "Are EU data subjects involved? (yes/no/likely)", "no"),
        ("ai_decision_making", "Does this make automated decisions about people? (yes/no/likely)", "no"),
    ],
    "founder": [  # the catch-all "Stephen wears all hats" persona
        ("scope", "Single-paragraph scope description?", ""),
        ("budget", "Calendar budget? (days)", "5"),
        ("risk_appetite", "Risk appetite? (conservative/balanced/aggressive)", "balanced"),
    ],
}


# ---- S3.1 — LLM draft helper ---------------------------------------------

def llm_draft_body(prompt: str, model: str = "claude-sonnet-4-6", max_tokens: int = 2000) -> str:
    """Return Claude's response, or a clearly-marked stub if SDK / key absent."""
    try:
        import anthropic  # type: ignore
    except ImportError:
        return "_(anthropic SDK not installed; pass --with-llm only when `pip install anthropic` is available)_"
    if not os.environ.get("ANTHROPIC_API_KEY"):
        return "_(ANTHROPIC_API_KEY not set; pass --with-llm only when the key is exported)_"
    try:
        client = anthropic.Anthropic()
        msg = client.messages.create(
            model=model,
            max_tokens=max_tokens,
            messages=[{"role": "user", "content": prompt}],
        )
        return "\n".join(b.text for b in msg.content if hasattr(b, "text"))
    except Exception as e:
        return f"_(LLM call failed: {e})_"


# ---- S3.2 — voice gate -----------------------------------------------------

# Mirrors voice_check.py rules (no em dashes, no AI vocabulary).
EM_DASHES = ["—", "–"]
AI_VOCAB = ["leverage", "robust", "ensure", "comprehensive", "seamless", "delve",
            "navigate", "tapestry", "in today's", "elevate", "unleash",
            "in the realm of", "embark on", "facilitate", "utilize", "synergy"]


def voice_gate(text: str) -> list[dict]:
    """Return list of findings. Empty list = passed."""
    findings = []
    for em in EM_DASHES:
        if em in text:
            count = text.count(em)
            findings.append({"code": "em-dash", "count": count,
                             "message": f"{count} em/en dash(es) — replace with comma or sentence break"})
    lower = text.lower()
    for word in AI_VOCAB:
        if re.search(rf"\b{re.escape(word)}\b", lower):
            findings.append({"code": "ai-vocab", "word": word,
                             "message": f"AI vocabulary {word!r} — rewrite plainly"})
    return findings


# ---- S3.3 — source-tier auto-attribution -----------------------------------

def attribute_claims(body: str, source_text: str) -> dict:
    """Walk paragraphs of body; mark each as human-confirmed if its key tokens
    appear in source_text, else llm-explicit.

    Returns a dict {paragraph_index: authority} for the body.
    """
    out = {}
    source_lower = source_text.lower()
    for i, para in enumerate(body.split("\n\n")):
        para_clean = re.sub(r"[^a-z0-9 ]", " ", para.lower()).split()
        # Skip very short paragraphs
        if len(para_clean) < 5:
            out[i] = "skipped"
            continue
        # Pick 4 longest tokens as fingerprint
        keys = sorted(set(para_clean), key=len, reverse=True)[:4]
        keys = [k for k in keys if len(k) >= 4 and k not in {"that", "with", "this", "have", "from", "your", "their"}]
        if len(keys) < 3:
            out[i] = "llm-explicit"
            continue
        hits = sum(1 for k in keys if k in source_lower)
        if hits >= max(2, len(keys) // 2):
            out[i] = "human-confirmed"
        else:
            out[i] = "llm-explicit"
    return out


# ---- S3.4 — diff vs prior version -----------------------------------------

def diff_artefact(old_path: Path, new_text: str) -> str:
    """Return a unified diff string. Empty when files are identical or old absent."""
    if not old_path.exists():
        return f"--- (no prior version)\n+++ {old_path}\n+ <new file: {len(new_text)} bytes>"
    old_text = old_path.read_text(encoding="utf-8")
    if old_text == new_text:
        return ""
    diff_lines = difflib.unified_diff(
        old_text.splitlines(), new_text.splitlines(),
        fromfile=f"prior/{old_path.name}", tofile=f"new/{old_path.name}",
        lineterm="",
    )
    return "\n".join(diff_lines)


# ---- S3.5 — per-persona interview templates -------------------------------

def interview_questions(persona: str, mode: str = "standalone") -> list[tuple[str, str, str]]:
    """Load (key, question, default) triples for a persona.

    Looks first under .cyberos-memory/meta/interview-templates/<persona>.md
    for the operator to override; falls back to embedded defaults above.
    """
    persona = persona.lower().replace("cuo-", "")
    # Disk override
    try:
        from pathlib import Path as _P
        cur = _P.cwd().resolve()
        while cur != cur.parent:
            f = cur / ".cyberos-memory" / "meta" / "interview-templates" / f"{persona}.md"
            if f.exists():
                return _parse_interview_md(f.read_text())
            cur = cur.parent
    except Exception:
        pass
    return PERSONA_QUESTIONS.get(persona, PERSONA_QUESTIONS["founder"])


def _parse_interview_md(text: str) -> list[tuple[str, str, str]]:
    """Parse interview md: each ## section is a question.

    Format:
        ## key
        Question prose.
        > default: <default-value>
    """
    out = []
    blocks = re.split(r"^##\s+", text, flags=re.MULTILINE)
    for b in blocks[1:]:
        lines = b.splitlines()
        if not lines:
            continue
        key = lines[0].strip()
        question = []
        default = ""
        for ln in lines[1:]:
            if ln.startswith("> default:"):
                default = ln.replace("> default:", "").strip()
            else:
                question.append(ln)
        q_text = " ".join(question).strip()
        out.append((key, q_text, default))
    return out


# ---- Standalone CLI for testing -------------------------------------------

if __name__ == "__main__":
    import argparse
    p = argparse.ArgumentParser(description="Stage 3 authoring helpers")
    sub = p.add_subparsers(dest="cmd", required=True)

    pl = sub.add_parser("llm")
    pl.add_argument("prompt")
    pl.add_argument("--model", default="claude-sonnet-4-6")

    pv = sub.add_parser("voice")
    pv.add_argument("file", nargs="?")

    pa = sub.add_parser("attribute")
    pa.add_argument("body_file")
    pa.add_argument("source_file")

    pd = sub.add_parser("diff")
    pd.add_argument("old_file")
    pd.add_argument("new_file")

    pi = sub.add_parser("interview")
    pi.add_argument("persona")

    args = p.parse_args()
    if args.cmd == "llm":
        print(llm_draft_body(args.prompt, args.model))
    elif args.cmd == "voice":
        text = Path(args.file).read_text() if args.file else sys.stdin.read()
        for f in voice_gate(text):
            print(f"  {f}")
        if not voice_gate(text):
            print("  ✓ voice clean")
    elif args.cmd == "attribute":
        body = Path(args.body_file).read_text()
        source = Path(args.source_file).read_text()
        for i, auth in attribute_claims(body, source).items():
            print(f"  paragraph {i}: {auth}")
    elif args.cmd == "diff":
        old = Path(args.old_file)
        new = Path(args.new_file).read_text()
        diff = diff_artefact(old, new)
        print(diff or "  (identical)")
    elif args.cmd == "interview":
        for k, q, d in interview_questions(args.persona):
            print(f"  [{k}] {q} (default: {d!r})")
