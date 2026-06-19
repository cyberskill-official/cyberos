#!/usr/bin/env python3
"""Compute the FR topological build order from depends_on frontmatter.

Kahn-layered: layer 0 = FRs whose dependencies are all already placed (or external),
each later layer depends only on earlier ones. Detects cycles and missing dependencies,
rolls up per module, and checks the deploy roadmap order against the real DAG.

  awh_build_order.py            # human summary
  awh_build_order.py --json     # machine-readable layers
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

FR_ROOT = Path("docs/feature-requests")
ROADMAP = ["MEMORY", "SKILL", "CUO", "AUTH", "CHAT", "PROJ"]


def load():
    mod, deps, status = {}, {}, {}
    for p in FR_ROOT.rglob("FR-*.md"):
        if p.name.endswith(".audit.md"):
            continue
        t = p.read_text(encoding="utf-8")[:4000]
        mid = re.search(r"^id:\s*(FR-[A-Z]+-\d+)", t, re.M)
        if not mid:
            continue
        fid = mid.group(1)
        mm = re.search(r"^module:\s*([A-Za-z]+)", t, re.M)
        mod[fid] = (mm.group(1).upper() if mm else fid.split("-")[1])
        st = re.search(r"^status:\s*(\S+)", t, re.M)
        status[fid] = st.group(1) if st else "?"
        dep = re.search(r"^depends_on:\s*\[([^\]]*)\]", t, re.M)
        deps[fid] = re.findall(r"FR-[A-Z]+-\d+", dep.group(1)) if dep else []
    return mod, deps, status


def layer(mod, deps):
    nodes = set(mod)
    indeps = {f: [d for d in deps[f] if d in nodes] for f in nodes}
    missing = {f: [d for d in deps[f] if d not in nodes] for f in nodes}
    missing = {f: v for f, v in missing.items() if v}
    remaining = set(nodes)
    layers = []
    while remaining:
        ready = sorted(f for f in remaining if all(d not in remaining for d in indeps[f]))
        if not ready:
            break  # cycle
        layers.append(ready)
        remaining -= set(ready)
    return layers, sorted(remaining), missing


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    mod, deps, status = load()
    layers, cyclic, missing = layer(mod, deps)
    layer_of = {f: i for i, lay in enumerate(layers) for f in lay}

    if args.json:
        print(json.dumps({
            "n_frs": len(mod), "n_layers": len(layers),
            "layers": [[{"fr": f, "module": mod[f]} for f in lay] for lay in layers],
            "cyclic": cyclic,
            "missing_deps": missing,
        }, indent=2))
        return 0

    print(f"FRs: {len(mod)} | topological layers: {len(layers)} | cyclic: {len(cyclic)}")
    print()
    for i, lay in enumerate(layers):
        by_mod = {}
        for f in lay:
            by_mod.setdefault(mod[f], 0)
            by_mod[mod[f]] += 1
        roll = " ".join(f"{m}:{n}" for m, n in sorted(by_mod.items(), key=lambda x: -x[1]))
        print(f"  layer {i:2d}: {len(lay):3d} FRs   {roll}")
    if cyclic:
        print(f"\nCYCLE among {len(cyclic)} FRs (could not be layered): {cyclic[:10]}{' ...' if len(cyclic)>10 else ''}")

    print("\nroadmap-module layer span (min..max build layer of each module's FRs):")
    for m in ROADMAP:
        ls = [layer_of[f] for f in mod if mod[f] == m and f in layer_of]
        if ls:
            print(f"  {m:7s} layers {min(ls)}..{max(ls)}  ({len(ls)} FRs)")

    print("\nroadmap-order violations (FR depends on an FR built in a later roadmap wave):")
    rank = {m: i for i, m in enumerate(ROADMAP)}
    v = 0
    for f in sorted(mod):
        if mod[f] not in rank:
            continue
        for d in deps[f]:
            dm = mod.get(d)
            if dm in rank and rank[dm] > rank[mod[f]]:
                print(f"  {f} [{mod[f]}, {status[f]}] -> {d} [{dm}]")
                v += 1
    print(f"  total: {v}")

    miss_in_roadmap = {f: v for f, v in missing.items() if mod.get(f) in rank}
    if miss_in_roadmap:
        print(f"\nroadmap FRs with missing (unknown) dependencies: {len(miss_in_roadmap)}")
        for f, v in list(miss_in_roadmap.items())[:8]:
            print(f"  {f} -> {v}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
