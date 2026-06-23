#!/usr/bin/env bash
# run-evals.sh — run the full fixture suite and record the result.
#
# Usage:  ./evals/run-evals.sh            # human output
#         ./evals/run-evals.sh --record   # also update evals/baseline.json
#
# Exit code: 0 = all fixtures behave as declared; 1 = regression.

set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"

python3 "$HERE/validate.py" --all
STATUS=$?

if [[ "${1:-}" == "--record" && $STATUS -eq 0 ]]; then
  TMP="$(mktemp)"
  python3 "$HERE/validate.py" --all --json > "$TMP"
  FP="$( (shasum -a 256 "$ROOT/AUDIT.md" 2>/dev/null || sha256sum "$ROOT/AUDIT.md") | cut -d' ' -f1)"
  VERSION="$(head -1 "$ROOT/AUDIT.md" | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' || echo unknown)"
  # Version-drift guard (improve/BLINDSPOTS.md BS-11): refuse to pin a baseline
  # when the protocol's title version and package.json disagree.
  PKG_VERSION="v$(python3 -c "import json;print(json.load(open('$ROOT/../package.json'))['version'])" 2>/dev/null || echo unknown)"
  if [[ "$VERSION" != "$PKG_VERSION" ]]; then
    echo "VERSION DRIFT: AUDIT.md title says $VERSION but package.json says $PKG_VERSION — align before recording." >&2
    rm -f "$TMP"
    exit 1
  fi
  RESULTS="$TMP" FP="$FP" VERSION="$VERSION" OUT="$HERE/baseline.json" python3 <<'PY'
import json, os, datetime, pathlib
data = json.load(open(os.environ["RESULTS"]))
out = {
  "audit_md_version": os.environ["VERSION"],
  "audit_md_sha256": os.environ["FP"],
  "recorded_at": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
  "fixtures": data["fixtures"],
  "passed": data["passed"],
  "all_ok": data["all_ok"],
  "matrix": {r["fixture"]: {"expect": r["expect"], "ok": r["ok"], "codes": r["codes"]} for r in data["results"]},
}
p = pathlib.Path(os.environ["OUT"])
p.write_text(json.dumps(out, indent=2) + "\n")
print(f"baseline recorded → {p.name} ({out['passed']}/{out['fixtures']} OK at {out['audit_md_version']})")
PY
  rm -f "$TMP"
fi

exit $STATUS
