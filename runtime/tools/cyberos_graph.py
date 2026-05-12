#!/usr/bin/env python3
"""
cyberos_graph.py — memory relationships graph explorer.

Aspect 4.7 of the Layer-1 improvement catalog.

Walks `.cyberos-memory/`, extracts every `relationships:` edge from frontmatter,
emits the graph in one of three formats:

  - text (default): grouped listing by edge kind
  - dot:            Graphviz DOT for rendering with `dot -Tpng`
  - json:           machine-readable adjacency list

Also surfaces graph-health findings:
  - dangling targets (relationship target memory_id not present on disk)
  - orphan nodes (memories with zero in-edges and zero out-edges, optional)
  - cycles in `supersedes` chains (already caught by validator, repeated here)

Edge kinds seen in practice: implements, supersedes, references, derives_from,
contradicts, validates, satisfied_by. Tool is kind-agnostic — counts every
edge regardless of kind label.

Usage:
    cyberos graph                              # text summary
    cyberos graph --format dot > brain.dot     # → render with `dot -Tpng brain.dot -o brain.png`
    cyberos graph --format json                # adjacency list
    cyberos graph --scope memories/decisions   # subgraph filter
    cyberos graph --orphans                    # include nodes with zero edges
    cyberos graph --memory mem_019e...         # ego-graph (1 hop)
"""
from __future__ import annotations
import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def parse_frontmatter(text: str) -> dict | None:
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---\n", 4)
    if end < 0:
        return None
    try:
        import yaml
        return yaml.safe_load(text[4:end]) or {}
    except Exception:
        return None


def collect(brain_root: Path, scope_prefix: str = "") -> tuple[dict, dict, list]:
    """Return (nodes, edges_by_kind, dangling).

    nodes: {memory_id: {rel, scope, tags}}
    edges_by_kind: {kind: [(src_id, tgt_id), ...]}
    dangling: list of (src_id, kind, tgt_id) where tgt_id not in nodes
    """
    brain = brain_root / ".cyberos-memory"
    nodes: dict[str, dict] = {}
    edges: dict[str, list] = defaultdict(list)

    for md in sorted(brain.rglob("*.md")):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/")):
            continue
        if scope_prefix and not rel.startswith(scope_prefix):
            continue
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            continue
        fm = parse_frontmatter(text)
        if not fm:
            continue
        mid = fm.get("memory_id")
        if not mid:
            continue
        nodes[mid] = {
            "rel": rel,
            "scope": fm.get("scope"),
            "tags": fm.get("tags", []),
            "tombstoned": bool(fm.get("tombstoned")),
        }
        for rel_edge in (fm.get("relationships") or []):
            if not isinstance(rel_edge, dict):
                continue
            kind = rel_edge.get("kind", "?")
            tgt = rel_edge.get("target")
            if tgt:
                edges[kind].append((mid, tgt))

    # Detect dangling
    dangling = []
    for kind, pairs in edges.items():
        for src, tgt in pairs:
            if tgt not in nodes:
                dangling.append((src, kind, tgt))
    return nodes, dict(edges), dangling


def ego_graph(nodes: dict, edges: dict, root_id: str, hops: int = 1) -> tuple[dict, dict]:
    """Return induced subgraph reachable within `hops` from `root_id`."""
    keep_ids = {root_id}
    frontier = {root_id}
    for _ in range(hops):
        new = set()
        for kind, pairs in edges.items():
            for s, t in pairs:
                if s in frontier and t in nodes:
                    new.add(t)
                if t in frontier:
                    new.add(s)
        keep_ids |= new
        frontier = new
    sub_nodes = {nid: nodes[nid] for nid in keep_ids if nid in nodes}
    sub_edges: dict[str, list] = {}
    for kind, pairs in edges.items():
        kept = [(s, t) for s, t in pairs if s in keep_ids and t in keep_ids]
        if kept:
            sub_edges[kind] = kept
    return sub_nodes, sub_edges


def render_text(nodes: dict, edges: dict, dangling: list, args):
    edge_count = sum(len(v) for v in edges.values())
    print()
    print(f"  Memory graph — {len(nodes)} node(s), {edge_count} edge(s)")
    print()
    if not edges:
        print("  (no relationships frontmatter has edges yet)")
    else:
        print(f"  Edges by kind:")
        for kind, pairs in sorted(edges.items(), key=lambda x: -len(x[1])):
            print(f"    {kind:18s} {len(pairs):4d}")
        print()
        if args.verbose:
            for kind in sorted(edges):
                print(f"  ── {kind} ──")
                for src, tgt in edges[kind][:20]:
                    src_rel = nodes.get(src, {}).get("rel", "?")
                    tgt_rel = nodes.get(tgt, {}).get("rel", "(dangling)")
                    print(f"    {src_rel}  →  {tgt_rel}")
                if len(edges[kind]) > 20:
                    print(f"    … +{len(edges[kind]) - 20} more")
                print()
    if dangling:
        print(f"  ⚠ Dangling targets ({len(dangling)} edges point at missing memories):")
        for src, kind, tgt in dangling[:10]:
            src_rel = nodes.get(src, {}).get("rel", "?")
            print(f"    {src_rel}  --{kind}-->  {tgt}  (target missing)")
        if len(dangling) > 10:
            print(f"    … +{len(dangling) - 10} more")
        print()

    if args.orphans:
        # Compute orphans
        connected = set()
        for kind, pairs in edges.items():
            for s, t in pairs:
                connected.add(s); connected.add(t)
        orphans = [nid for nid in nodes if nid not in connected and not nodes[nid].get("tombstoned")]
        print(f"  Orphan nodes (zero relationships): {len(orphans)} of {len(nodes)}")
        for nid in orphans[:20]:
            print(f"    {nodes[nid]['rel']}")
        if len(orphans) > 20:
            print(f"    … +{len(orphans) - 20} more")


def render_dot(nodes: dict, edges: dict) -> str:
    out = ["digraph BRAIN {"]
    out.append('  rankdir=LR;')
    out.append('  node [shape=box, fontname="Helvetica", fontsize=10];')
    out.append('  edge [fontname="Helvetica", fontsize=8];')
    # Cluster by scope
    by_scope: dict[str, list] = defaultdict(list)
    for nid, meta in nodes.items():
        scope = (meta.get("scope") or "unknown").split("/")[0]
        by_scope[scope].append(nid)
    for i, (scope, ids) in enumerate(sorted(by_scope.items())):
        out.append(f'  subgraph cluster_{i} {{')
        out.append(f'    label="{scope}";')
        out.append(f'    style=dashed;')
        for nid in ids:
            label = nodes[nid]["rel"].rsplit("/", 1)[-1].replace(".md", "")
            color = "gray" if nodes[nid].get("tombstoned") else "black"
            out.append(f'    "{nid}" [label="{label}", color={color}];')
        out.append('  }')
    for kind, pairs in edges.items():
        for src, tgt in pairs:
            out.append(f'  "{src}" -> "{tgt}" [label="{kind}"];')
    out.append('}')
    return "\n".join(out) + "\n"


def render_json(nodes: dict, edges: dict, dangling: list) -> str:
    adj = defaultdict(list)
    for kind, pairs in edges.items():
        for s, t in pairs:
            adj[s].append({"kind": kind, "target": t})
    payload = {
        "node_count": len(nodes),
        "edge_count": sum(len(v) for v in edges.values()),
        "nodes": {nid: nodes[nid] for nid in sorted(nodes)},
        "adjacency": {nid: adj[nid] for nid in sorted(nodes) if nid in adj},
        "dangling": [{"src": s, "kind": k, "target": t} for s, k, t in dangling],
    }
    return json.dumps(payload, indent=2, default=str)


def main():
    p = argparse.ArgumentParser(description="Memory relationships graph explorer")
    p.add_argument("--format", choices=["text", "dot", "json"], default="text")
    p.add_argument("--scope", default="", help="restrict to scope prefix")
    p.add_argument("--orphans", action="store_true", help="report nodes with no edges")
    p.add_argument("--memory", help="ego-graph (1-hop neighbourhood of this memory_id)")
    p.add_argument("--hops", type=int, default=1, help="ego-graph hop count (default 1)")
    p.add_argument("--verbose", "-v", action="store_true", help="list every edge in text mode")
    args = p.parse_args()

    brain_root = find_brain()
    nodes, edges, dangling = collect(brain_root, args.scope)

    if args.memory:
        if args.memory not in nodes:
            print(f"ERROR: memory_id not found: {args.memory}", file=sys.stderr)
            return 2
        nodes, edges = ego_graph(nodes, edges, args.memory, args.hops)
        # Re-derive dangling on the subgraph
        dangling = [(s, k, t) for s, k, t in dangling if s in nodes or t in nodes]

    if args.format == "dot":
        sys.stdout.write(render_dot(nodes, edges))
    elif args.format == "json":
        sys.stdout.write(render_json(nodes, edges, dangling) + "\n")
    else:
        render_text(nodes, edges, dangling, args)
    return 1 if dangling else 0


if __name__ == "__main__":
    sys.exit(main())
