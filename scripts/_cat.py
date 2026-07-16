#!/usr/bin/env python3
"""Categorise every remaining `init` so the decision is on evidence, not assertion."""
import re, subprocess, collections
from pathlib import Path

files = subprocess.run(["git", "ls-files"], capture_output=True, text=True).stdout.split()
TOK = re.compile(r"(?<![A-Za-z])init(?![A-Za-z])", re.I)
SKIP = ("node_modules/", "playground/", "apps/console/web/assets/", "package-lock")

BUCKETS = [
    ("OUR install verb (init.sh / cyberos init / /init / re-init)",
     re.compile(r"init\.sh|cyberos init|(?<![a-z])/init\b|re-init|cyberos-init")),
    ("BRAIN store init  (python -m cyberos init, _auto_init_if_needed, auto-init)",
     re.compile(r"_auto_init_if_needed|auto-init|_cmd_init|store init|memory store|"
                r"cyberos init|store_rel|initialise the BRAIN|init a fresh store", re.I)),
    ("load-or-init an audit report  (English: create if absent)",
     re.compile(r"load-or-init")),
    ("Python/Rust/JS language or library init",
     re.compile(r"__init__|get_or_init|OnceCell|_try_init|\.init\(\)|::init|lazy init|"
                r"module init|obs_sdk|red::init|initialize\(")),
    ("other tools' init  (git / terraform / npm / cargo / postgres / docker)",
     re.compile(r"git init|terraform init|npm init|cargo init|postgres-init|initdb|sql/init")),
    ("Capacitor / mobile one-time init",
     re.compile(r"capacitor|mobile one-time|one-time init", re.I)),
    ("WebGPU / graphics init",
     re.compile(r"WebGPU|GPU init|webgl", re.I)),
    ("a subcommand of another CLI  (cyberos-ten ... init)",
     re.compile(r"cyberos-ten|holdco-flip")),
    ("test fixture / helper  (_init_store, commit -qm init)",
     re.compile(r"_init_store|commit -qm init|init_redis|initial")),
]

counts = collections.Counter()
examples = collections.defaultdict(list)
other = []

for f in files:
    if any(s in f for s in SKIP):
        continue
    p = Path(f)
    if not p.is_file():
        continue
    try:
        body = p.read_text(encoding="utf-8")
    except Exception:
        continue
    for i, l in enumerate(body.splitlines(), 1):
        if not TOK.search(l):
            continue
        for name, rx in BUCKETS:
            if rx.search(l):
                counts[name] += 1
                if len(examples[name]) < 2:
                    examples[name].append(f"{f}:{i}  {l.strip()[:74]}")
                break
        else:
            counts["UNCLASSIFIED"] += 1
            other.append(f"{f}:{i}  {l.strip()[:74]}")

total = sum(counts.values())
print(f"{total} `init` hits\n")
for name, _ in BUCKETS:
    n = counts.get(name, 0)
    if not n:
        continue
    print(f"  {n:4d}  {name}")
    for e in examples[name]:
        print(f"          {e}")
print(f"\n  {counts.get('UNCLASSIFIED', 0):4d}  UNCLASSIFIED")
for e in other[:12]:
    print(f"          {e}")
