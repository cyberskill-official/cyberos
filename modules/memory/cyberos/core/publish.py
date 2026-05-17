"""
cyberos.core.publish — read-only static-site export (PROPOSAL.md P12).

Produces a SINGLE self-contained HTML file that you can airdrop to your
phone, host on GitHub Pages, drop in iCloud Drive — anywhere.  Opens
offline (no network needed once it's on the device), is fully searchable
client-side, and renders well on a 375-pixel-wide iPhone viewport.

Design constraints:

* one file (``brain.html``), no companion JS/CSS bundles;
* mobile-first layout (single column under 720px, two-column above);
* the JSON payload is the deterministic source of truth — we just inline
  it via a `<script type="application/json" id="brain-data">` tag and
  let a tiny vanilla-JS app on the page filter, search, and render;
* zero external requests — fonts inherit from system stack, no CDN, no
  embedded analytics, no service worker, no remote API;
* the file is reproducible: two invocations on the same store with the
  same ``--include`` flags produce a byte-identical output (modulo
  ``generated_at_ns``; opt-out via ``--no-timestamp``).

Privacy posture: publish READS but never WRITES the chain. The output
contains memory bodies verbatim — the operator can scope the export down
via ``--kinds`` (allowlist) or ``--exclude-kinds`` (blocklist) before
sharing. There is no "send" step; the file is yours to hand-off.
"""

from __future__ import annotations

import base64
import datetime as _dt
import hashlib
import json
import re
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Iterable

from cyberos.core.frontmatter import looks_like_yaml, parse, parse_legacy_yaml


@dataclass
class MemorySummary:
    """One memory, projected into a JSON-serialisable record."""
    id: str
    kind: str
    path: str
    actor: str
    ts_ns: int
    tags: list[str] = field(default_factory=list)
    body: str = ""
    body_sha256: str = ""


@dataclass
class PublishManifest:
    """Top-level data structure embedded in the HTML page."""
    store_fingerprint: str
    generated_at_ns: int
    counts_by_kind: dict[str, int] = field(default_factory=dict)
    counts_by_actor: dict[str, int] = field(default_factory=dict)
    memories: list[MemorySummary] = field(default_factory=list)


# ---------------------------------------------------------------------------
# scan
# ---------------------------------------------------------------------------


_MEMORY_ROOTS = ("memories", "company", "module", "member", "client", "project", "persona")


def _walk_memory_files(store: Path) -> Iterable[Path]:
    """Yield every memory .md file under canonical roots, sorted."""
    paths: list[Path] = []
    for root in _MEMORY_ROOTS:
        base = store / root
        if not base.is_dir():
            continue
        for p in base.rglob("*.md"):
            if p.is_file():
                paths.append(p)
    paths.sort()
    yield from paths


def _read_one(path: Path) -> tuple[dict, bytes] | None:
    """Best-effort parse — returns (frontmatter-as-dict, body-bytes) or None."""
    try:
        raw = path.read_bytes()
    except OSError:
        return None
    try:
        if looks_like_yaml(raw):
            fm, body = parse_legacy_yaml(raw)
        else:
            fm, body = parse(raw)
        # msgspec.Struct → plain dict for easy JSON serialisation
        import msgspec
        fm_dict = msgspec.to_builtins(fm)
        return fm_dict, body
    except Exception:  # noqa: BLE001 — silently skip unparseable bodies
        return None


def collect(
    store: Path,
    *,
    kinds: list[str] | None = None,
    exclude_kinds: list[str] | None = None,
    max_body_chars: int = 200_000,
) -> PublishManifest:
    """Scan the store and return everything that should appear in the site.

    Filters:

    * ``kinds`` — if set, only include memories whose kind is in the list;
    * ``exclude_kinds`` — drop memories whose kind is in this list;
    * ``max_body_chars`` — cap per-body size (very long bodies are
      truncated with a trailing ``…[truncated]`` marker so the inline JSON
      doesn't balloon).

    Returns a :class:`PublishManifest` ready to be rendered by :func:`render_html`.
    """
    fingerprint = hashlib.sha256(str(store.resolve()).encode("utf-8")).hexdigest()[:16]
    counts_kind: dict[str, int] = {}
    counts_actor: dict[str, int] = {}
    memories: list[MemorySummary] = []

    for path in _walk_memory_files(store):
        rel = path.relative_to(store).as_posix()
        parsed = _read_one(path)
        if parsed is None:
            continue
        fm, body = parsed

        kind = fm.get("kind", "unknown")
        if kinds and kind not in kinds:
            continue
        if exclude_kinds and kind in exclude_kinds:
            continue

        body_text = body.decode("utf-8", errors="replace")
        truncated = False
        if len(body_text) > max_body_chars:
            body_text = body_text[:max_body_chars] + "\n\n…[truncated]"
            truncated = True

        memories.append(MemorySummary(
            id=str(fm.get("id", path.stem)),
            kind=kind,
            path=rel,
            actor=str(fm.get("actor", "unknown")),
            ts_ns=int(fm.get("ts_ns", 0)),
            tags=list(fm.get("tags", []) or []),
            body=body_text,
            body_sha256=hashlib.sha256(body).hexdigest(),
        ))
        counts_kind[kind] = counts_kind.get(kind, 0) + 1
        actor = str(fm.get("actor", "unknown"))
        counts_actor[actor] = counts_actor.get(actor, 0) + 1

    # Sort memories by ts_ns DESC so newest appear first in default render
    memories.sort(key=lambda m: (-m.ts_ns, m.path))

    return PublishManifest(
        store_fingerprint=fingerprint,
        generated_at_ns=time.time_ns(),
        counts_by_kind=dict(sorted(counts_kind.items())),
        counts_by_actor=dict(sorted(counts_actor.items())),
        memories=memories,
    )


# ---------------------------------------------------------------------------
# render
# ---------------------------------------------------------------------------


_HTML_TEMPLATE = """\
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>BRAIN — {fingerprint}</title>
<meta name="generator" content="cyberos publish v1">
<style>
:root {{
  --bg: #fbfbfa;
  --fg: #1f1f1f;
  --muted: #6c6c6c;
  --accent: #2563eb;
  --card-bg: #ffffff;
  --border: #e5e5e3;
  --code-bg: #f1f1ef;
}}
@media (prefers-color-scheme: dark) {{
  :root {{
    --bg: #1a1a1a;
    --fg: #ededed;
    --muted: #9c9c9c;
    --accent: #60a5fa;
    --card-bg: #242424;
    --border: #333;
    --code-bg: #2c2c2c;
  }}
}}
* {{ box-sizing: border-box; }}
html, body {{ margin: 0; padding: 0; background: var(--bg); color: var(--fg);
  font: 16px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, system-ui, sans-serif; }}
header {{ padding: 16px 20px; border-bottom: 1px solid var(--border); position: sticky; top: 0; background: var(--bg); z-index: 10; }}
h1 {{ font-size: 18px; margin: 0 0 8px 0; }}
.meta {{ font-size: 12px; color: var(--muted); }}
.controls {{ margin-top: 12px; display: flex; gap: 8px; flex-wrap: wrap; }}
.controls input, .controls select {{
  font: inherit; padding: 8px 10px; border: 1px solid var(--border);
  border-radius: 6px; background: var(--card-bg); color: var(--fg);
}}
.controls input[type="search"] {{ flex: 1 1 200px; min-width: 120px; }}
main {{ padding: 16px 20px 40px; max-width: 880px; margin: 0 auto; }}
.empty {{ color: var(--muted); padding: 24px 0; }}
.card {{ background: var(--card-bg); border: 1px solid var(--border);
  border-radius: 8px; padding: 14px 16px; margin: 12px 0;
  box-shadow: 0 1px 0 rgba(0,0,0,0.02); }}
.card h2 {{ margin: 0 0 4px 0; font-size: 15px; }}
.card .row {{ font-size: 12px; color: var(--muted); margin-bottom: 8px; }}
.card .tag {{ display: inline-block; padding: 1px 8px; margin-right: 4px;
  border: 1px solid var(--border); border-radius: 999px; font-size: 11px; color: var(--muted); }}
.card pre {{ background: var(--code-bg); padding: 10px; border-radius: 6px;
  overflow-x: auto; font-size: 13px; white-space: pre-wrap; word-wrap: break-word; }}
.card .body {{ font-size: 14px; white-space: pre-wrap; word-wrap: break-word; }}
.summary {{ font-size: 12px; color: var(--muted); margin-bottom: 12px; }}
@media (min-width: 720px) {{
  main {{ padding: 24px; }}
}}
</style>
</head>
<body>
<header>
  <h1>BRAIN</h1>
  <div class="meta">store {fingerprint} · {n_memories} memories · generated {generated_at}</div>
  <div class="controls">
    <input type="search" id="q" placeholder="Search bodies, ids, tags…">
    <select id="kind"><option value="">all kinds</option></select>
    <select id="actor"><option value="">all actors</option></select>
  </div>
</header>
<main>
  <div class="summary" id="summary"></div>
  <div id="list"></div>
</main>
<script type="application/json" id="brain-data">{payload}</script>
<script>
(function() {{
  var dataNode = document.getElementById('brain-data');
  var data;
  try {{ data = JSON.parse(dataNode.textContent); }}
  catch(e) {{ document.getElementById('list').innerHTML = '<p class="empty">Failed to parse embedded BRAIN data: ' + e.message + '</p>'; return; }}

  var q = document.getElementById('q');
  var kindSel = document.getElementById('kind');
  var actorSel = document.getElementById('actor');
  var list = document.getElementById('list');
  var summary = document.getElementById('summary');

  // Populate kind / actor selects
  Object.keys(data.counts_by_kind).sort().forEach(function(k) {{
    var opt = document.createElement('option');
    opt.value = k; opt.textContent = k + ' (' + data.counts_by_kind[k] + ')';
    kindSel.appendChild(opt);
  }});
  Object.keys(data.counts_by_actor).sort().forEach(function(a) {{
    var opt = document.createElement('option');
    opt.value = a; opt.textContent = a + ' (' + data.counts_by_actor[a] + ')';
    actorSel.appendChild(opt);
  }});

  function fmtDate(ns) {{
    if (!ns) return 'unknown';
    var ms = Math.floor(ns / 1e6);
    return new Date(ms).toISOString().replace('T', ' ').replace(/\\..*/, ' UTC');
  }}

  function escape(s) {{
    return String(s).replace(/[&<>"']/g, function(c) {{
      return {{ '&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;' }}[c];
    }});
  }}

  function render() {{
    var needle = q.value.trim().toLowerCase();
    var kind = kindSel.value;
    var actor = actorSel.value;
    var matches = data.memories.filter(function(m) {{
      if (kind && m.kind !== kind) return false;
      if (actor && m.actor !== actor) return false;
      if (!needle) return true;
      if (m.id.toLowerCase().indexOf(needle) !== -1) return true;
      if (m.path.toLowerCase().indexOf(needle) !== -1) return true;
      if (m.body.toLowerCase().indexOf(needle) !== -1) return true;
      for (var i = 0; i < m.tags.length; i++) {{
        if (m.tags[i].toLowerCase().indexOf(needle) !== -1) return true;
      }}
      return false;
    }});

    summary.textContent = matches.length + ' / ' + data.memories.length + ' memories';
    if (matches.length === 0) {{
      list.innerHTML = '<p class="empty">No matches.</p>'; return;
    }}
    var html = '';
    for (var i = 0; i < matches.length; i++) {{
      var m = matches[i];
      html += '<article class="card">'
        + '<h2>' + escape(m.id) + ' <small style="color:var(--muted);font-weight:normal">· ' + escape(m.kind) + '</small></h2>'
        + '<div class="row">' + escape(m.path) + ' · ' + escape(m.actor) + ' · ' + fmtDate(m.ts_ns) + '</div>'
        + (m.tags.length ? '<div>' + m.tags.map(function(t) {{ return '<span class="tag">' + escape(t) + '</span>'; }}).join('') + '</div>' : '')
        + '<div class="body">' + escape(m.body) + '</div>'
      + '</article>';
    }}
    list.innerHTML = html;
  }}

  q.addEventListener('input', render);
  kindSel.addEventListener('change', render);
  actorSel.addEventListener('change', render);
  render();
}})();
</script>
</body>
</html>
"""


def render_html(manifest: PublishManifest, *, deterministic: bool = False) -> str:
    """Materialise the manifest as a complete HTML document.

    When ``deterministic=True``, ``manifest.generated_at_ns`` is zeroed
    out in both the JSON payload and the human header, so two consecutive
    invocations over the same store produce byte-identical output.
    """
    payload_obj = asdict(manifest)
    if deterministic:
        payload_obj["generated_at_ns"] = 0

    payload = json.dumps(payload_obj, sort_keys=True, separators=(",", ":"))
    # Escape "</" to prevent the inline JSON from accidentally closing
    # the <script> tag with a literal "</script>" in some memory body.
    payload = payload.replace("</", "<\\/")

    generated_at = (
        "—"
        if deterministic
        else _dt.datetime.fromtimestamp(
            manifest.generated_at_ns / 1e9, tz=_dt.timezone.utc,
        ).strftime("%Y-%m-%d %H:%M:%S UTC")
    )

    return _HTML_TEMPLATE.format(
        fingerprint=manifest.store_fingerprint,
        n_memories=len(manifest.memories),
        generated_at=generated_at,
        payload=payload,
    )


def publish_to_file(
    store: Path,
    out_path: Path,
    *,
    kinds: list[str] | None = None,
    exclude_kinds: list[str] | None = None,
    max_body_chars: int = 200_000,
    deterministic: bool = False,
) -> dict:
    """End-to-end: scan → render → write. Returns a summary dict."""
    manifest = collect(
        store, kinds=kinds, exclude_kinds=exclude_kinds,
        max_body_chars=max_body_chars,
    )
    html = render_html(manifest, deterministic=deterministic)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(html, encoding="utf-8")
    return {
        "out_path": str(out_path),
        "bytes": len(html.encode("utf-8")),
        "n_memories": len(manifest.memories),
        "counts_by_kind": manifest.counts_by_kind,
        "counts_by_actor": manifest.counts_by_actor,
        "deterministic": deterministic,
        "sha256": hashlib.sha256(html.encode("utf-8")).hexdigest(),
    }


__all__ = [
    "MemorySummary",
    "PublishManifest",
    "collect",
    "render_html",
    "publish_to_file",
]
