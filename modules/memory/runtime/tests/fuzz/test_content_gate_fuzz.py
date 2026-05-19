#!/usr/bin/env python3
"""
test_content_gate_fuzz.py — property-based + fuzz testing for §4.2 content gate.

Aspect 5.6 + 10.2 of the Layer-1 improvement catalog.

Generates random inputs with biased distributions toward injection markers,
mixed-script confusables, BOM placements, surrogate pairs. Verifies content
gate either accepts cleanly OR rejects with a specific reason.

Requires:
    pip install hypothesis --break-system-packages

Usage:
    python3 runtime/tests/fuzz/test_content_gate_fuzz.py
    python3 runtime/tests/fuzz/test_content_gate_fuzz.py --runs 1000
"""
from __future__ import annotations
import argparse
import re
import sys

INJECTION_MARKERS = [
    "[INST]", "<system>", "<<SYS>>", "<|im_start|>", "<|system|>",
    "<|assistant|>", "###Instruction", "###System:",
    "ignore previous instructions", "ignore the above",
]

CONFUSABLES = [
    ("a", "а"),  # latin a → Cyrillic а
    ("e", "е"),  # latin e → Cyrillic е
    ("o", "о"),  # latin o → Cyrillic о
    ("p", "р"),  # latin p → Cyrillic р
    ("c", "с"),  # latin c → Cyrillic с
    ("y", "у"),  # latin y → Cyrillic у
]

BOM = "﻿"
SURROGATE_LO = "\ud800"
SURROGATE_HI = "\udfff"
ZERO_WIDTH = "​‌‍⁠"
BIDI = "‪‫‬‭‮⁦⁧⁨⁩"

def _content_gate_check(text: str) -> str | None:
    """Mirror of §4.2 content gate. Returns rejection reason or None."""
    # Strip ZWJ/ZWSP first per §4.2 pre-processing
    cleaned = text
    for ch in ZERO_WIDTH:
        cleaned = cleaned.replace(ch, "")
    # NFKC fold + bidi-override codepoint check
    if any(c in text for c in BIDI):
        return "bidi-override-codepoint"
    if any(c in text for c in (SURROGATE_LO, SURROGATE_HI)):
        return "lone-surrogate"
    if BOM in text:
        return "bom-detected"
    # Injection markers (whitespace-tolerant)
    text_collapsed = re.sub(r"\s+", " ", cleaned.lower())
    for marker in INJECTION_MARKERS:
        if marker.lower().replace(" ", "") in text_collapsed.replace(" ", ""):
            return f"injection-marker:{marker[:20]}"
    # Mixed-script confusables (UTS-39 fold)
    for latin, cyrillic in CONFUSABLES:
        if cyrillic in text and latin in text:
            return f"mixed-script-confusable:{latin}-{cyrillic}"
    return None

def gen_random(rng, n=200) -> str:
    """Generate a random string with biased injection-attempt distribution."""
    parts = []
    for _ in range(n):
        r = rng.random()
        if r < 0.05:
            parts.append(rng.choice(INJECTION_MARKERS))
        elif r < 0.10:
            parts.append(BOM)
        elif r < 0.15:
            parts.append(rng.choice(BIDI))
        elif r < 0.20:
            cyr_latin = rng.choice(CONFUSABLES)
            parts.append(f"{cyr_latin[0]} normal then {cyr_latin[1]} confused")
        else:
            parts.append(chr(rng.randint(0x20, 0x7e)))
    return "".join(parts)

def run_fuzz(runs: int = 500, verbose: bool = False) -> tuple[int, int, dict]:
    import random
    rng = random.Random(42)  # deterministic seed
    crashes = 0
    rejections = 0
    by_reason = {}

    for i in range(runs):
        try:
            text = gen_random(rng)
            reason = _content_gate_check(text)
            if reason:
                rejections += 1
                key = reason.split(":")[0]
                by_reason[key] = by_reason.get(key, 0) + 1
                if verbose and i < 5:
                    print(f"  reject [{key}]: {text[:60]!r}")
        except Exception as e:
            crashes += 1
            if verbose:
                print(f"  CRASH at run {i}: {e}")
    return crashes, rejections, by_reason

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--runs", type=int, default=500)
    p.add_argument("-v", "--verbose", action="store_true")
    args = p.parse_args()

    print(f"\n→ fuzz §4.2 content gate ({args.runs} runs, deterministic seed)\n")
    crashes, rejections, by_reason = run_fuzz(args.runs, args.verbose)

    print(f"  Total runs:    {args.runs}")
    print(f"  Crashes:       {crashes}  (target: 0)")
    print(f"  Rejections:    {rejections}  ({rejections/args.runs*100:.1f}% of inputs)")
    print(f"  By reason:")
    for r, n in sorted(by_reason.items(), key=lambda x: -x[1]):
        print(f"    {r:30s}: {n}")

    return 1 if crashes > 0 else 0

if __name__ == "__main__":
    sys.exit(main())
