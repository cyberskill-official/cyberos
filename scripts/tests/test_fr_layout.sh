#!/usr/bin/env bash
# test_fr_layout.sh - FR-DOCS-004 §5 suite (t01-t06 -> AC 1-6).
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../.." && pwd)"
FRD="$repo/docs/feature-requests"
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

t01_no_flat_files() {                                                  # AC 1
  n="$(find "$FRD" -maxdepth 2 -name 'FR-*.md' ! -path '*_audits*' ! -path '*_archive*' | wc -l | tr -d ' ')"
  [ "$n" -eq 0 ] && ok t01 || fail t01 "$n flat FR files remain"
}
t02_folder_count_matches() {                                           # AC 1
  folders="$(find "$FRD" -maxdepth 2 -type d -name 'FR-*' ! -path '*/.*/*' | wc -l | tr -d ' ')"
  specs="$(find "$FRD" -maxdepth 3 -name spec.md | wc -l | tr -d ' ')"
  [ "$folders" -eq "$specs" ] && [ "$specs" -gt 400 ] && ok t02 || fail t02 "folders=$folders specs=$specs"
}
t03_idempotent_rerun() {                                               # AC 2
  out="$(python3 "$repo/scripts/migrate_fr_layout.py")"
  grep -q "nothing to do" <<<"$out" && ok t03 || fail t03 "$out"
}
t04_regen_loud_and_reconciled() {                                      # AC 3+4
  err="$(python3 "$repo/scripts/migrate_improvement_to_fr.py" --backlog 2>&1 >/dev/null)"
  if grep -q "unparseable" <<<"$err"; then fail t04 "regen still skipping: $err"; return; fi
  bl="$(python3 "$repo/scripts/migrate_improvement_to_fr.py" --backlog 2>/dev/null | grep -o '[0-9]* FRs' | grep -o '[0-9]*')"
  rm="$(node "$repo/tools/docs-site/render-roadmap.mjs" "$repo" "$(mktemp -d)" 2>/dev/null | sed -n 's/^roadmap: \([0-9]*\) FRs.*/\1/p')"
  [ "$bl" = "$rm" ] && ok t04 || fail t04 "backlog=$bl roadmap=$rm"
}
t05_repairs_minimal() {                                                # AC 5
  python3 - "$FRD" <<'PY' && ok t05 || fail t05 "corpus not strict-yaml clean"
import sys, yaml, re
from pathlib import Path
for f in Path(sys.argv[1]).glob("*/FR-*/spec.md"):
    m = re.match(r"\A---\n(.*?)\n---\n", f.read_text(), re.S)
    if not m: sys.exit(1)
    yaml.safe_load(m.group(1))
PY
}
t06_anchors_green() {                                                  # AC 6
  bash "$repo/scripts/check_doc_anchors.sh" >/dev/null 2>&1 && ok t06 || fail t06 "doc anchors exit != 0"
}

t01_no_flat_files; t02_folder_count_matches; t03_idempotent_rerun
t04_regen_loud_and_reconciled; t05_repairs_minimal; t06_anchors_green
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
