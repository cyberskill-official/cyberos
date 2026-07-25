#!/usr/bin/env bash
set -euo pipefail
payload="${1:-}"; out="${2:-}"
[ -n "$payload" ] && [ -n "$out" ] || { echo "usage: emit-payload-sbom.sh <payload-dir> <out.cdx.json>" >&2; exit 2; }
[ -d "$payload" ] || { echo "emit-payload-sbom: payload dir missing: $payload" >&2; exit 2; }
if command -v syft >/dev/null 2>&1; then
  syft "dir:$payload" -o cyclonedx-json="$out"; echo "emit-payload-sbom: wrote $out (syft)"; exit 0
fi
python3 - "$payload" "$out" <<'PY'
import hashlib, json, os, sys, time
payload, out = sys.argv[1], sys.argv[2]
components = []
for root, dirs, files in os.walk(payload):
    dirs[:] = [d for d in dirs if d != ".git"]
    for name in files:
        path = os.path.join(root, name)
        rel = os.path.relpath(path, payload).replace(os.sep, "/")
        h = hashlib.sha256()
        with open(path, "rb") as f:
            for chunk in iter(lambda: f.read(1024*1024), b""): h.update(chunk)
        components.append({"type":"file","name":rel,"hashes":[{"alg":"SHA-256","content":h.hexdigest()}]})
if not components:
    print("emit-payload-sbom: payload has no files", file=sys.stderr); sys.exit(2)
ver = "unknown"
vp = os.path.join(payload, "VERSION")
if os.path.isfile(vp): ver = open(vp, encoding="utf-8").read().strip() or "unknown"
doc = {"bomFormat":"CycloneDX","specVersion":"1.5","version":1,
  "metadata":{"timestamp":time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    "component":{"type":"application","name":"cyberos-payload","version":ver},
    "tools":[{"vendor":"CyberOS","name":"emit-payload-sbom.sh","version":"1"}]},
  "components":components}
os.makedirs(os.path.dirname(out) or ".", exist_ok=True)
with open(out,"w",encoding="utf-8") as f: json.dump(doc,f,indent=2,sort_keys=True); f.write("\n")
print(f"emit-payload-sbom: wrote {out} ({len(components)} components, hermetic)")
PY
