#!/usr/bin/env bash
# Trigger one MCP tool through the gateway, without hand-writing JSON-RPC. Prints the
# tool's text result (or the error). Pairs with run-demo.sh.
#
#   bash call.sh <tool-name> ['<json-args>']
#
# Examples:
#   bash call.sh cyberos.demo.now
#   bash call.sh cyberos.demo.echo '{"message":"hello"}'
#
# Point at a different gateway with MCP_GATEWAY (default http://127.0.0.1:8090).

set -euo pipefail

GATEWAY="${MCP_GATEWAY:-http://127.0.0.1:8090}"
TOOL="${1:-}"
# Default to an empty JSON object. (Written out rather than ${2:-{}} - that form's brace
# matching appends a stray "}" to a supplied argument.)
if [ "$#" -ge 2 ] && [ -n "$2" ]; then ARGS="$2"; else ARGS='{}'; fi

if [ -z "$TOOL" ]; then
  echo "usage: bash call.sh <tool-name> ['<json-args>']" >&2
  echo "  e.g. bash call.sh cyberos.demo.echo '{\"message\":\"hi\"}'" >&2
  exit 2
fi

# Validate the arguments are JSON and build the JSON-RPC envelope (env-passed to avoid
# shell quoting pitfalls).
if ! printf '%s' "$ARGS" | python3 -c 'import sys,json; json.load(sys.stdin)' 2>/dev/null; then
  echo "arguments must be valid JSON, got: $ARGS" >&2
  exit 2
fi

payload="$(TOOL="$TOOL" ARGS="$ARGS" python3 -c '
import os, json
print(json.dumps({
    "jsonrpc": "2.0", "id": 1, "method": "tools/call",
    "params": {"name": os.environ["TOOL"], "arguments": json.loads(os.environ["ARGS"])},
}))')"

resp="$(curl -fsS -X POST "$GATEWAY/mcp" -H 'content-type: application/json' -d "$payload")" || {
  echo "could not reach the gateway at $GATEWAY (is run-demo.sh running?)" >&2
  exit 1
}

printf '%s' "$resp" | python3 -c '
import sys, json
r = json.load(sys.stdin)
if "error" in r:
    e = r["error"]
    print("error " + str(e.get("code")) + ": " + str(e.get("message")))
    data = e.get("data")
    if data:
        print(json.dumps(data, indent=2))
    sys.exit(1)
res = r.get("result", {})
blocks = res.get("content", [])
text = "\n".join(b.get("text", "") for b in blocks if b.get("type") == "text")
print(text if text else json.dumps(res, indent=2))
sys.exit(1 if res.get("isError") else 0)
'
