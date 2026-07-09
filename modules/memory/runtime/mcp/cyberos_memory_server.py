#!/usr/bin/env python3
"""
cyberos_memory_server.py — read-only MCP server for the .cyberos/memory/store memory.

Aspect 12.7 of the Layer-1 improvement catalog.

Minimal viable MCP server speaking line-delimited JSON-RPC 2.0 over stdio.
Exposes 4 read-only tools to MCP clients (Claude Code, Cursor, etc.):

  - memory_search    keyword search across slug/tags/scope/body
  - memory_show      list memories with metadata + optional filters
  - memory_get       fetch a single memory by memory_id or relative path
  - memory_stats     bucket counts + sync-class breakdown

NO WRITES. NO ledger mutation. NO audit row emission. If the caller
asks for a write, the tool returns a JSON error pointing at
`memory_writer.py` as the canonical write path.

This server respects:
  - §0.1 real-filesystem-only (resolves memory via .cyberos/memory/store walk)
  - §17 sync_class filtering (default: omit local-only from search hits
    unless the caller explicitly passes include_local_only=true)
  - tombstone filter (default: hide tombstoned; explicit opt-in)

Wire it into a client (Claude Code example) via .claude/mcp-config.json:

    {
      "mcpServers": {
        "cyberos-memory": {
          "command": "python3",
          "args": ["/abs/path/to/runtime/mcp/cyberos_memory_server.py"],
          "env": {"CYBEROS_MEMORY_ROOT": "/abs/path/to/cyberos"}
        }
      }
    }

If CYBEROS_MEMORY_ROOT is unset, the server walks up from CWD to find
the first `.cyberos/memory/store/` directory (§0.1 convention).

Run standalone for testing:
    echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.0"}}}' \
      | python3 runtime/mcp/cyberos_memory_server.py
"""
from __future__ import annotations

import json
import os
import re
import sys
import traceback
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))
SERVER_NAME = "cyberos-memory"
SERVER_VERSION = "0.1.0"
PROTOCOL_VERSION = "2024-11-05"


# ---------------------------------------------------------------------------
# memory access (read-only)
# ---------------------------------------------------------------------------

def resolve_memory_root() -> Path:
    env = os.environ.get("CYBEROS_MEMORY_ROOT")
    if env:
        p = Path(env)
        if (p / ".cyberos/memory/store").is_dir():
            return p
    cur = Path.cwd().resolve()
    while cur != cur.parent:
        if (cur / ".cyberos/memory/store").is_dir():
            return cur
        cur = cur.parent
    raise RuntimeError("no .cyberos/memory/store/ found; set CYBEROS_MEMORY_ROOT")


def parse_frontmatter(text: str) -> tuple[dict, str]:
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
        fm = {}
        for line in fm_text.splitlines():
            m = re.match(r"^([a-z_]+):\s*(.+?)\s*$", line)
            if m:
                fm[m.group(1)] = m.group(2)
        return fm, body


def iter_memories(memory_root: Path):
    memory = memory_root / ".cyberos/memory/store"
    for md in sorted(memory.rglob("*.md")):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(memory).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/")):
            continue
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            continue
        fm, body = parse_frontmatter(text)
        yield rel, fm, body, text


def apply_default_filters(fm: dict, include_local_only: bool, include_tombstoned: bool) -> bool:
    if fm.get("tombstoned") and not include_tombstoned:
        return False
    sync = fm.get("sync_class", "")
    if sync == "local-only" and not include_local_only:
        return False
    return True


# ---------------------------------------------------------------------------
# Tools
# ---------------------------------------------------------------------------

def tool_memory_search(args: dict) -> dict:
    """Keyword search across slug/tags/scope/body. Returns up to `limit` hits."""
    query = (args.get("query") or "").strip().lower()
    if not query:
        return {"error": "missing 'query'"}
    limit = int(args.get("limit") or 20)
    include_local = bool(args.get("include_local_only", False))
    include_tomb = bool(args.get("include_tombstoned", False))
    scope = (args.get("scope") or "").strip()

    memory_root = resolve_memory_root()
    hits = []
    q_words = [w for w in re.findall(r"\w+", query) if len(w) >= 2]
    for rel, fm, body, _ in iter_memories(memory_root):
        if scope and not rel.startswith(scope):
            continue
        if not apply_default_filters(fm, include_local, include_tomb):
            continue
        haystack = (rel + " " + " ".join(fm.get("tags", []) or []) + " " + body[:4000]).lower()
        score = sum(1 for w in q_words if w in haystack)
        if score == 0:
            continue
        hits.append({
            "score": score,
            "path": rel,
            "memory_id": fm.get("memory_id"),
            "scope": fm.get("scope"),
            "tags": fm.get("tags", []),
            "sync_class": fm.get("sync_class"),
            "classification": fm.get("classification"),
            "snippet": body[:240].replace("\n", " ")[:240] + ("…" if len(body) > 240 else ""),
        })
    hits.sort(key=lambda h: (-h["score"], h["path"]))
    return {"query": query, "hits": hits[:limit], "total_matched": len(hits)}


def tool_memory_show(args: dict) -> dict:
    """List memories with metadata + optional filters."""
    tag = (args.get("tag") or "").strip()
    scope = (args.get("scope") or "").strip()
    classification = (args.get("classification") or "").strip()
    limit = int(args.get("limit") or 50)
    include_local = bool(args.get("include_local_only", False))
    include_tomb = bool(args.get("include_tombstoned", False))

    memory_root = resolve_memory_root()
    rows = []
    for rel, fm, _, _ in iter_memories(memory_root):
        if scope and not rel.startswith(scope):
            continue
        if classification and fm.get("classification") != classification:
            continue
        if tag:
            tags = fm.get("tags") or []
            if isinstance(tags, str):
                tags = [t.strip() for t in tags.strip("[]").split(",")]
            if tag not in tags:
                continue
        if not apply_default_filters(fm, include_local, include_tomb):
            continue
        rows.append({
            "path": rel,
            "memory_id": fm.get("memory_id"),
            "scope": fm.get("scope"),
            "classification": fm.get("classification"),
            "authority": fm.get("authority"),
            "sync_class": fm.get("sync_class"),
            "version": fm.get("version"),
            "tags": fm.get("tags"),
            "tombstoned": bool(fm.get("tombstoned")),
        })
    return {"rows": rows[:limit], "total_matched": len(rows)}


def tool_memory_get(args: dict) -> dict:
    """Fetch a single memory by memory_id or relative path."""
    mid = (args.get("memory_id") or "").strip()
    path = (args.get("path") or "").strip()
    if not mid and not path:
        return {"error": "pass either 'memory_id' or 'path'"}

    memory_root = resolve_memory_root()
    for rel, fm, body, text in iter_memories(memory_root):
        if mid and fm.get("memory_id") == mid:
            return {"path": rel, "frontmatter": fm, "body": body, "content_size": len(text)}
        if path and rel == path:
            return {"path": rel, "frontmatter": fm, "body": body, "content_size": len(text)}
    return {"error": f"not found: memory_id={mid!r} path={path!r}"}


def tool_memory_stats(args: dict) -> dict:
    """Bucket counts + sync-class breakdown + tombstone count."""
    memory_root = resolve_memory_root()
    by_scope: dict[str, int] = {}
    by_sync: dict[str, int] = {}
    by_class: dict[str, int] = {}
    tomb = 0
    total = 0
    for _, fm, _, _ in iter_memories(memory_root):
        total += 1
        scope = (fm.get("scope") or "").split("/")[0] or "unknown"
        by_scope[scope] = by_scope.get(scope, 0) + 1
        sync = fm.get("sync_class") or "unknown"
        by_sync[sync] = by_sync.get(sync, 0) + 1
        cls = fm.get("classification") or "unknown"
        by_class[cls] = by_class.get(cls, 0) + 1
        if fm.get("tombstoned"):
            tomb += 1
    return {
        "total": total,
        "tombstoned": tomb,
        "by_scope": by_scope,
        "by_sync_class": by_sync,
        "by_classification": by_class,
        "memory_root": str(memory_root),
    }


TOOLS = {
    "memory_search": {
        "fn": tool_memory_search,
        "description": "Keyword search across .cyberos/memory/store/ — matches slug, tags, scope, body. Returns top hits ranked by score. Default filters: tombstoned hidden, sync_class=local-only hidden.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "search keywords (space-separated)"},
                "limit": {"type": "integer", "default": 20},
                "scope": {"type": "string", "description": "filter to scope prefix (e.g. memories/decisions)"},
                "include_local_only": {"type": "boolean", "default": False},
                "include_tombstoned": {"type": "boolean", "default": False},
            },
            "required": ["query"],
        },
    },
    "memory_show": {
        "fn": tool_memory_show,
        "description": "List memories with metadata. Optional filters: tag, scope prefix, classification. Default filters: tombstoned hidden, sync_class=local-only hidden.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "tag": {"type": "string"},
                "scope": {"type": "string"},
                "classification": {"type": "string", "enum": ["personnel", "client", "operational", "public"]},
                "limit": {"type": "integer", "default": 50},
                "include_local_only": {"type": "boolean", "default": False},
                "include_tombstoned": {"type": "boolean", "default": False},
            },
        },
    },
    "memory_get": {
        "fn": tool_memory_get,
        "description": "Fetch a single memory by memory_id or relative path. Returns frontmatter (parsed dict) + body.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "memory_id": {"type": "string"},
                "path": {"type": "string", "description": "relative path inside .cyberos/memory/store/"},
            },
        },
    },
    "memory_stats": {
        "fn": tool_memory_stats,
        "description": "Bucket counts + sync-class breakdown + tombstone count for the current memory.",
        "inputSchema": {"type": "object", "properties": {}},
    },
}


# ---------------------------------------------------------------------------
# JSON-RPC handlers
# ---------------------------------------------------------------------------

def handle_initialize(params: dict) -> dict:
    return {
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": {},
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": SERVER_VERSION,
        },
    }


def handle_tools_list(params: dict) -> dict:
    return {
        "tools": [
            {
                "name": name,
                "description": meta["description"],
                "inputSchema": meta["inputSchema"],
            }
            for name, meta in TOOLS.items()
        ]
    }


def handle_tools_call(params: dict) -> dict:
    name = params.get("name")
    args = params.get("arguments") or {}
    if name not in TOOLS:
        raise ValueError(f"unknown tool: {name}")
    if name.startswith("memory_") and name not in ("memory_search", "memory_show", "memory_get", "memory_stats"):
        # Defensive: refuse anything that looks like a write
        return {"content": [{"type": "text", "text": json.dumps({"error": "this MCP server is read-only; use memory_writer.py for writes"})}], "isError": True}
    try:
        result = TOOLS[name]["fn"](args)
    except Exception as e:
        return {"content": [{"type": "text", "text": json.dumps({"error": str(e)})}], "isError": True}
    return {"content": [{"type": "text", "text": json.dumps(result, indent=2, default=str)}]}


METHODS = {
    "initialize": handle_initialize,
    "tools/list": handle_tools_list,
    "tools/call": handle_tools_call,
    "notifications/initialized": lambda p: None,  # client-sent notification, no response
}


def send(obj: dict):
    sys.stdout.write(json.dumps(obj, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def main():
    for raw in sys.stdin:
        raw = raw.strip()
        if not raw:
            continue
        try:
            msg = json.loads(raw)
        except json.JSONDecodeError as e:
            send({"jsonrpc": "2.0", "id": None, "error": {"code": -32700, "message": f"parse error: {e}"}})
            continue
        method = msg.get("method")
        params = msg.get("params") or {}
        msg_id = msg.get("id")
        if method not in METHODS:
            if msg_id is not None:
                send({"jsonrpc": "2.0", "id": msg_id, "error": {"code": -32601, "message": f"method not found: {method}"}})
            continue
        try:
            result = METHODS[method](params)
        except Exception as e:
            tb = traceback.format_exc()
            if msg_id is not None:
                send({"jsonrpc": "2.0", "id": msg_id, "error": {"code": -32603, "message": f"internal error: {e}", "data": tb}})
            continue
        # Skip response for notifications (no id)
        if msg_id is None:
            continue
        send({"jsonrpc": "2.0", "id": msg_id, "result": result})


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
