#!/usr/bin/env python3
"""
cyberos_serve.py — local web dashboard.

Batch 12 (Tier B) of post-catalog improvements.

Single-file stdlib HTTP server. No external dependencies. Renders:

  /             — operator dashboard (HEALTHY / BOTTLENECK / CHANGED / WHAT NOW)
  /memories     — list with filters
  /memory/<id>  — single memory view (frontmatter + body)
  /audit        — recent audit rows
  /stats.json   — machine-readable stats

Read-only by design. To mutate, use `cyberos add` / `cyberos doctor`.

Usage:
    cyberos serve --port 8080
    open http://localhost:8080/
"""
from __future__ import annotations
import argparse
import html
import json
import re
import sys
from datetime import datetime, timedelta, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path
from urllib.parse import urlparse, parse_qs

ICT = timezone(timedelta(hours=7))


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def parse_frontmatter(text: str) -> tuple[dict, str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}, text
    try:
        import yaml
        return yaml.safe_load(text[4:end]) or {}, text[end + 5:]
    except Exception:
        return {}, text[end + 5:]


def page(title: str, body: str) -> str:
    return f"""<!doctype html>
<html><head><meta charset=utf-8><title>{html.escape(title)}</title>
<style>
body{{font:14px/1.5 system-ui,-apple-system,sans-serif;max-width:1100px;margin:0 auto;padding:1rem;color:#222}}
header{{border-bottom:1px solid #ddd;padding-bottom:.5rem;margin-bottom:1rem}}
nav a{{margin-right:1rem;color:#06c}}
table{{border-collapse:collapse;width:100%}}
th,td{{text-align:left;padding:.4rem .6rem;border-bottom:1px solid #eee;font-size:13px}}
th{{background:#fafafa}}
.ok{{color:#067a30}}.warn{{color:#b76b00}}.crit{{color:#b00020}}
pre{{background:#f5f5f5;padding:.6rem;border-radius:4px;overflow-x:auto;font-size:12px}}
code{{font-family:'SF Mono',Menlo,monospace}}
.kv{{display:grid;grid-template-columns:160px 1fr;gap:.3rem;font-size:13px}}
.kv b{{color:#666;font-weight:500}}
.muted{{color:#888;font-size:12px}}
</style></head><body>
<header><strong>CyberOS BRAIN</strong>
<nav><a href=/>dashboard</a><a href=/memories>memories</a><a href=/audit>audit</a><a href=/stats.json>stats.json</a></nav>
</header>
{body}
</body></html>"""


def render_dashboard(brain_root):
    brain = brain_root / ".cyberos-memory"
    try:
        m = json.loads((brain / "manifest.json").read_text())
    except Exception:
        m = {}
    n = sum(1 for p in brain.rglob("*.md") if p.is_file() and not p.name.startswith("."))
    now = datetime.now(ICT)
    cutoff = now - timedelta(hours=24)
    ops = {}
    sessions = 0
    if (brain / "audit").exists():
        for ledger in (brain / "audit").glob("*.jsonl"):
            for line in ledger.read_text(encoding="utf-8", errors="ignore").splitlines():
                if not line.strip():
                    continue
                try:
                    r = json.loads(line)
                    ts = datetime.fromisoformat(r.get("ts", ""))
                    if ts >= cutoff:
                        ops[r.get("op", "?")] = ops.get(r.get("op", "?"), 0) + 1
                        if r.get("op") == "session.start":
                            sessions += 1
                except Exception:
                    continue
    drift = sum(1 for _ in (brain / "memories" / "drift").glob("*.md")) if (brain / "memories" / "drift").exists() else 0
    rows = "".join(f"<tr><td>{html.escape(op)}</td><td>{n}</td></tr>" for op, n in sorted(ops.items(), key=lambda x: -x[1]))
    body = f"""
<h2>Operator dashboard</h2>
<div class=kv>
  <b>Project</b><span>{html.escape(str(m.get("project", {}).get("name", "?")))}</span>
  <b>Memories on disk</b><span>{n}</span>
  <b>Audit chain head</b><span><code>{html.escape(m.get("audit_chain_head", "")[:48])}…</code></span>
  <b>Last 24h</b><span>{sum(ops.values())} ops, {sessions} session(s)</span>
  <b>Drift candidates</b><span>{drift}</span>
</div>
<h3>Last 24h ops</h3>
<table><tr><th>op</th><th>count</th></tr>{rows}</table>
"""
    return page("dashboard", body)


def render_memories(brain_root, query: dict):
    brain = brain_root / ".cyberos-memory"
    scope = query.get("scope", [""])[0]
    tag = query.get("tag", [""])[0]
    rows = []
    for md in sorted(brain.rglob("*.md")):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/")):
            continue
        if scope and not rel.startswith(scope):
            continue
        try:
            fm, _ = parse_frontmatter(md.read_text())
        except Exception:
            continue
        tags = fm.get("tags") or []
        if tag and tag not in tags:
            continue
        mid = fm.get("memory_id", "—")
        rows.append(f"<tr><td><a href=/memory/{html.escape(mid)}>{html.escape(rel)}</a></td>"
                    f"<td>{html.escape(str(fm.get('classification', '?')))}</td>"
                    f"<td>{html.escape(str(fm.get('sync_class', '?')))}</td>"
                    f"<td><span class=muted>{', '.join(html.escape(str(t)) for t in tags[:6])}</span></td></tr>")
    body = f"""
<h2>Memories ({len(rows)})</h2>
<form><label>scope <input name=scope value="{html.escape(scope)}"></label>
<label>tag <input name=tag value="{html.escape(tag)}"></label>
<button>filter</button></form>
<table><tr><th>path</th><th>class</th><th>sync</th><th>tags</th></tr>{''.join(rows)}</table>
"""
    return page("memories", body)


def render_memory(brain_root, mid):
    brain = brain_root / ".cyberos-memory"
    for md in brain.rglob("*.md"):
        try:
            text = md.read_text()
            if f"memory_id: {mid}" not in text:
                continue
            fm, body_text = parse_frontmatter(text)
            return page(mid, f"<h2>{html.escape(mid)}</h2>"
                             f"<p class=muted>{html.escape(md.relative_to(brain).as_posix())}</p>"
                             f"<pre>{html.escape(text)}</pre>")
        except Exception:
            continue
    return page("not found", "<p>memory not found</p>")


def render_audit(brain_root):
    brain = brain_root / ".cyberos-memory"
    rows = []
    if (brain / "audit").exists():
        for ledger in sorted((brain / "audit").glob("*.jsonl")):
            for line in ledger.read_text(encoding="utf-8", errors="ignore").splitlines()[-50:]:
                if not line.strip():
                    continue
                try:
                    r = json.loads(line)
                    rows.append(f"<tr><td>{html.escape(r.get('ts',''))}</td>"
                                f"<td>{html.escape(r.get('op','?'))}</td>"
                                f"<td>{html.escape(r.get('actor','?'))}</td>"
                                f"<td><code>{html.escape((r.get('path') or r.get('memory_id') or '')[:80])}</code></td></tr>")
                except Exception:
                    continue
    body = f"<h2>Recent audit rows</h2><table><tr><th>ts</th><th>op</th><th>actor</th><th>target</th></tr>{''.join(rows[-50:])}</table>"
    return page("audit", body)


def render_stats(brain_root):
    brain = brain_root / ".cyberos-memory"
    try:
        m = json.loads((brain / "manifest.json").read_text())
    except Exception:
        m = {}
    n = sum(1 for p in brain.rglob("*.md") if p.is_file() and not p.name.startswith("."))
    return json.dumps({
        "memory_count": n,
        "audit_chain_head": m.get("audit_chain_head", ""),
        "project": m.get("project", {}),
        "last_updated_at": m.get("last_updated_at", ""),
    }, indent=2)


def make_handler(brain_root):
    class H(BaseHTTPRequestHandler):
        def log_message(self, fmt, *a):
            pass  # quiet
        def do_GET(self):
            u = urlparse(self.path)
            try:
                if u.path == "/":
                    body, ct = render_dashboard(brain_root), "text/html; charset=utf-8"
                elif u.path == "/memories":
                    body, ct = render_memories(brain_root, parse_qs(u.query)), "text/html; charset=utf-8"
                elif u.path.startswith("/memory/"):
                    body, ct = render_memory(brain_root, u.path.split("/", 2)[2]), "text/html; charset=utf-8"
                elif u.path == "/audit":
                    body, ct = render_audit(brain_root), "text/html; charset=utf-8"
                elif u.path == "/stats.json":
                    body, ct = render_stats(brain_root), "application/json"
                else:
                    self.send_response(404); self.end_headers(); self.wfile.write(b"not found"); return
                data = body.encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", ct)
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
            except Exception as e:
                self.send_response(500); self.end_headers(); self.wfile.write(str(e).encode())
    return H


def main():
    p = argparse.ArgumentParser(description="local web dashboard for the BRAIN (Batch 12 / Tier B)")
    p.add_argument("--port", type=int, default=8080)
    p.add_argument("--host", default="127.0.0.1")
    args = p.parse_args()
    brain_root = find_brain()
    handler = make_handler(brain_root)
    server = HTTPServer((args.host, args.port), handler)
    print(f"  serving BRAIN at http://{args.host}:{args.port}/   (Ctrl-C to stop)")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.shutdown()
    return 0


if __name__ == "__main__":
    sys.exit(main())
