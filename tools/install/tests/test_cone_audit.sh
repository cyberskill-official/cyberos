#!/usr/bin/env bash
# test_cone_audit.sh - TASK-IMP-119 §1 suite (t01-t07 -> clauses 1.1-1.7 / AC1-AC6; t08 -> AC7;
# t09 -> the build.sh vendor line so dist carries the tool).
#
# cone-audit REPORTS a task's writes that escape its declared cone; it never writes, refuses, or
# flips. Every clause is proven against a scratch git repo that bends exactly one thing, plus one
# test (t08) that runs against THIS repo's real 2026-07-17 batch (TASK-IMP-110 + TASK-IMP-114) -
# "a tool that cannot find the case that motivated it is decoration."
#
#   t01 -> clause 1.1 / AC1   an actual write outside the cone is NAMED as an escape; a task that
#                             wrote nothing (base==HEAD) is zero escapes, exit 0
#   t02 -> clause 1.2 / AC2   containment == batch-select's, proven on a SHARED table: cone-audit's
#                             exported `within` and batch-select's own primitive (batch-select:93,
#                             grep-pinned) agree on every row; end-to-end nested-vs-prefix binding
#   t03 -> clause 1.3 / AC3   an UNDECLARED cone escapes EVERY write (never zero)
#   t04 -> clause 1.4 / AC4   `(none)` is filtered from the cone, never treated as a path
#   t05 -> clause 1.5 / AC5   reports & exits 0 WITH escapes (never non-zero); writes nothing
#                             (worktree byte-identical; the spec is untouched)
#   t06 -> clause 1.6 / AC6   deterministic: two runs byte-identical; no wall clock in the artefact
#   t07 -> clause 1.7 / AC6   guard: spec symlinked out of the corpus / unreadable spec / not-a-repo
#                             / missing base are REFUSED and NAMED, never zero escapes
#   t08 -> AC7                the real 2026-07-17 batch: cone-audit(110) ∩ cone-audit(114) names
#                             EXACTLY the 3 files that escaped BOTH cones - no more, no fewer
#   t09 -> vendor line        build.sh vendors cone-audit.mjs byte-identical; the payload copy runs
#
# An external run_all.sh discovers this file via its tools/install/tests/test_*.sh glob.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
CA="$repo/tools/install/docs-tools/cone-audit.mjs"
BS="$repo/tools/install/docs-tools/batch-select.mjs"
BUILD="$repo/tools/install/build.sh"
TMP="$(mktemp -d)"; trap 'chmod -R u+rwX "$TMP" 2>/dev/null; rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

# JSON field helpers (node, stdlib) - read cone-audit --json from a file.
jq_field() { node -e 'const d=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"));const v=d[process.argv[2]];process.stdout.write(Array.isArray(v)?v.join("\n"):String(v))' "$1" "$2"; }

# A scratch git repo with one committed base state; echoes the base SHA on fd 1.
git_init() { ( cd "$1" && git init -qb main . && git config user.email t@t && git config user.name t ) >/dev/null 2>&1; }

# ── t01: an actual write outside the cone is named; wrote-nothing is zero escapes ─────────────────
t01_escape_is_named() {
  local d="$TMP/t01"; mkdir -p "$d/docs/tasks/x/TASK-T-001" "$d/src" "$d/tools"
  cat > "$d/docs/tasks/x/TASK-T-001/spec.md" <<'SPEC'
---
id: TASK-T-001
title: t
template: task@1
status: implementing
new_files:
  - src/a.ts
modified_files:
  - README.md
---

# TASK-T-001
SPEC
  echo "base" > "$d/README.md"
  git_init "$d"; ( cd "$d" && git add -A && git commit -qm base ) >/dev/null 2>&1
  local BASE; BASE="$(cd "$d" && git rev-parse HEAD)"
  # the implementing writes: one inside (src/a.ts), one inside (README.md), one ESCAPE (tools/evil.sh)
  echo "a" > "$d/src/a.ts"; echo "changed" > "$d/README.md"; echo "x" > "$d/tools/evil.sh"
  ( cd "$d" && git add -A && git commit -qm impl ) >/dev/null 2>&1
  node "$CA" TASK-T-001 --base "$BASE" --repo "$d" --json > "$TMP/t01.json" 2>"$TMP/t01.err"; local rc=$?
  [ "$rc" -eq 0 ] || { fail t01 "exit $rc (want 0): $(cat "$TMP/t01.err")"; return; }
  local esc; esc="$(jq_field "$TMP/t01.json" escapes)"
  [ "$esc" = "tools/evil.sh" ] || { fail t01 "escapes='$esc' (want exactly tools/evil.sh)"; return; }
  # the inside writes are NOT escapes
  grep -q 'src/a.ts' <<<"$esc" && { fail t01 "a declared new_file was reported as an escape"; return; }
  grep -q 'README.md' <<<"$esc" && { fail t01 "a declared modified_file was reported as an escape"; return; }
  # edge #1/#2: base == HEAD -> empty diff -> zero escapes, exit 0
  node "$CA" TASK-T-001 --base HEAD --repo "$d" --json > "$TMP/t01b.json" 2>/dev/null; rc=$?
  { [ "$rc" -eq 0 ] && [ "$(jq_field "$TMP/t01b.json" escape_count)" = "0" ]; } \
    || { fail t01 "wrote-nothing was not zero escapes / exit 0 (rc=$rc)"; return; }
  ok t01_escape_is_named
}

# ── t02: containment matches batch-select on a shared table ───────────────────────────────────────
t02_containment_matches_batch_select() {
  # batch-select's containment primitive is inlined in clash() as `x.startsWith(y + "/")` (plus the
  # `x === y` equality). If that operator form ever changes, cone-audit MUST be revisited - so pin it.
  grep -qF 'startsWith(y + "/")' "$BS" \
    || { fail t02 "batch-select.mjs no longer carries the 'startsWith(y + \"/\")' containment operator - the two tools may have drifted; re-derive cone-audit's within()"; return; }
  # The shared table + the agreement proof: cone-audit's exported within() and a byte-faithful copy
  # of batch-select's primitive must BOTH equal the expected result on every row.
  cat > "$TMP/t02probe.mjs" <<EOF
import { within, insideCone } from "file://$CA";
// batch-select.mjs:93 primitive, directional half (the "path at-or-under entry" side), verbatim:
const bs = (p, e) => p === e || p.startsWith(e + "/");
// [path, cone-entry, expected-inside]  (spec §3 rows 4,5,6 + the "+/" boundary cases)
const table = [
  ["a/b.ts", "a/b.ts", true],                    // equal (row 4)
  ["a/b/c.ts", "a", true],                       // nested under a service (row 5)
  ["a/b/c.ts", "a/b", true],                     // nested under a dir
  ["a", "a/b.ts", false],                        // write is the declared file's PARENT (row 6)
  ["a/b.ts", "a/c.ts", false],                   // sibling files
  ["ab/c.ts", "a", false],                       // shares prefix "a" but not the path boundary "a/"
  ["abc", "ab", false],                          // string prefix, not nested
  ["tools/install", "tools/install", true],      // service equals
  ["tools/install/x.sh", "tools/install", true], // file under a service
  ["tools/installer/x", "tools/install", false], // prefix-not-boundary on a service
];
let bad = 0;
for (const [p, e, exp] of table) {
  const wa = within(p, e), wb = bs(p, e);
  if (wa !== exp || wb !== exp) { bad++; console.error(\`MISMATCH within(\${p},\${e}) cone-audit=\${wa} batch-select=\${wb} expected=\${exp}\`); }
}
// insideCone over a multi-entry cone agrees with row-wise within()
if (insideCone("a/b/c", new Set(["z", "a"])) !== true) { bad++; console.error("insideCone nested miss"); }
if (insideCone("ab/c", new Set(["a"])) !== false) { bad++; console.error("insideCone prefix-not-boundary miss"); }
process.exit(bad === 0 ? 0 : 1);
EOF
  node "$TMP/t02probe.mjs" 2>"$TMP/t02.err" \
    || { fail t02 "containment disagreed with batch-select on the shared table: $(cat "$TMP/t02.err")"; return; }
  # end-to-end binding: the tool's REAL run uses the same containment (nested inside, prefix escapes)
  local d="$TMP/t02e"; mkdir -p "$d/docs/tasks/x/TASK-C-001" "$d/proj/deep" "$d/projendix"
  cat > "$d/docs/tasks/x/TASK-C-001/spec.md" <<'SPEC'
---
id: TASK-C-001
title: t
template: task@1
status: implementing
service: proj
---

# TASK-C-001
SPEC
  git_init "$d"; ( cd "$d" && git add -A && git commit -qm base ) >/dev/null 2>&1
  local BASE; BASE="$(cd "$d" && git rev-parse HEAD)"
  echo x > "$d/proj/deep/f.ts"; echo x > "$d/projendix/f.ts"
  ( cd "$d" && git add -A && git commit -qm impl ) >/dev/null 2>&1
  node "$CA" TASK-C-001 --base "$BASE" --repo "$d" --json > "$TMP/t02json" 2>/dev/null
  local esc; esc="$(jq_field "$TMP/t02json" escapes)"
  { grep -qxF 'projendix/f.ts' <<<"$esc" && ! grep -qxF 'proj/deep/f.ts' <<<"$esc"; } \
    || { fail t02 "end-to-end containment wrong: nested-under should be inside, prefix-not-boundary should escape (escapes='$esc')"; return; }
  ok t02_containment_matches_batch_select
}

# ── t03: an undeclared cone escapes every write (never zero) ──────────────────────────────────────
t03_undeclared_cone_escapes_everything() {
  local d="$TMP/t03"; mkdir -p "$d/docs/tasks/x/TASK-U-001" "$d/src"
  cat > "$d/docs/tasks/x/TASK-U-001/spec.md" <<'SPEC'
---
id: TASK-U-001
title: t
template: task@1
status: implementing
---

# TASK-U-001 (no new_files / modified_files / service - an UNKNOWN cone)
SPEC
  git_init "$d"; ( cd "$d" && git add -A && git commit -qm base ) >/dev/null 2>&1
  local BASE; BASE="$(cd "$d" && git rev-parse HEAD)"
  echo x > "$d/src/a.ts"; echo x > "$d/src/b.ts"
  ( cd "$d" && git add -A && git commit -qm impl ) >/dev/null 2>&1
  node "$CA" TASK-U-001 --base "$BASE" --repo "$d" --json > "$TMP/t03.json" 2>/dev/null || { fail t03 "run failed"; return; }
  local w e; w="$(jq_field "$TMP/t03.json" writes | grep -c .)"; e="$(jq_field "$TMP/t03.json" escape_count)"
  # every write is an escape - and there IS at least one (never the silent zero of an empty cone)
  { [ "$e" -gt 0 ] && [ "$e" = "$w" ]; } || { fail t03 "undeclared cone did not escape every write (writes=$w escapes=$e)"; return; }
  [ "$(jq_field "$TMP/t03.json" cone_declared)" = "false" ] || { fail t03 "cone_declared should be false"; return; }
  ok t03_undeclared_cone_escapes_everything
}

# ── t04: `(none)` is filtered from the cone, never a path ──────────────────────────────────────────
t04_placeholder_is_not_a_path() {
  local d="$TMP/t04"; mkdir -p "$d/docs/tasks/x/TASK-N-001" "$d/keep" "$d/other"
  cat > "$d/docs/tasks/x/TASK-N-001/spec.md" <<'SPEC'
---
id: TASK-N-001
title: t
template: task@1
status: implementing
new_files:
  - (none)
modified_files:
  - keep/a.ts
---

# TASK-N-001
SPEC
  git_init "$d"; ( cd "$d" && git add -A && git commit -qm base ) >/dev/null 2>&1
  local BASE; BASE="$(cd "$d" && git rev-parse HEAD)"
  echo x > "$d/keep/a.ts"; echo x > "$d/other/b.ts"
  ( cd "$d" && git add -A && git commit -qm impl ) >/dev/null 2>&1
  node "$CA" TASK-N-001 --base "$BASE" --repo "$d" --json > "$TMP/t04.json" 2>/dev/null || { fail t04 "run failed"; return; }
  # the literal (none) is NOT a cone entry ...
  jq_field "$TMP/t04.json" cone | grep -qxF '(none)' && { fail t04 "(none) survived into the cone as a path"; return; }
  # ... the real entry did survive, and covers keep/a.ts; other/b.ts escapes
  jq_field "$TMP/t04.json" cone | grep -qxF 'keep/a.ts' || { fail t04 "the real modified_files entry was dropped"; return; }
  [ "$(jq_field "$TMP/t04.json" escapes)" = "other/b.ts" ] || { fail t04 "escapes wrong (want only other/b.ts): $(jq_field "$TMP/t04.json" escapes)"; return; }
  ok t04_placeholder_is_not_a_path
}

# ── t05: reports & exits 0 with escapes; writes nothing ───────────────────────────────────────────
t05_reports_never_refuses() {
  local d="$TMP/t05"; mkdir -p "$d/docs/tasks/x/TASK-R-001" "$d/tools"
  cat > "$d/docs/tasks/x/TASK-R-001/spec.md" <<'SPEC'
---
id: TASK-R-001
title: t
template: task@1
status: implementing
new_files:
  - src/a.ts
---

# TASK-R-001
SPEC
  git_init "$d"; ( cd "$d" && git add -A && git commit -qm base ) >/dev/null 2>&1
  local BASE; BASE="$(cd "$d" && git rev-parse HEAD)"
  mkdir -p "$d/src"; echo x > "$d/src/a.ts"; echo x > "$d/tools/escape.sh"
  ( cd "$d" && git add -A && git commit -qm impl ) >/dev/null 2>&1
  # WITH an escape present, the tool still exits 0 (an escape is a finding, not a failure)
  local before after specpre specpost
  before="$(cd "$d" && git status --porcelain)"
  specpre="$(sha256sum "$d/docs/tasks/x/TASK-R-001/spec.md")"
  node "$CA" TASK-R-001 --base "$BASE" --repo "$d" > "$TMP/t05.out" 2>&1; local rc=$?
  [ "$rc" -eq 0 ] || { fail t05 "exited non-zero ($rc) WITH an escape - it must report, not fail"; return; }
  grep -q 'ESCAPE  tools/escape.sh' "$TMP/t05.out" || { fail t05 "the escape was not reported"; return; }
  # read-only: the worktree and the spec are byte-identical after the run
  after="$(cd "$d" && git status --porcelain)"
  [ "$before" = "$after" ] || { fail t05 "the tool changed the worktree (not read-only): [$after]"; return; }
  specpost="$(sha256sum "$d/docs/tasks/x/TASK-R-001/spec.md")"
  [ "$specpre" = "$specpost" ] || { fail t05 "the tool wrote to the spec (must never flip/refuse/write)"; return; }
  ok t05_reports_never_refuses
}

# ── t06: deterministic; no wall clock in the artefact ─────────────────────────────────────────────
t06_deterministic() {
  local d="$TMP/t06"; mkdir -p "$d/docs/tasks/x/TASK-D-001"
  cat > "$d/docs/tasks/x/TASK-D-001/spec.md" <<'SPEC'
---
id: TASK-D-001
title: t
template: task@1
status: implementing
service: mod/a
---

# TASK-D-001
SPEC
  git_init "$d"; ( cd "$d" && git add -A && git commit -qm base ) >/dev/null 2>&1
  local BASE; BASE="$(cd "$d" && git rev-parse HEAD)"
  mkdir -p "$d/mod/a" "$d/mod/b"; echo x > "$d/mod/a/f"; echo x > "$d/mod/b/f"
  ( cd "$d" && git add -A && git commit -qm impl ) >/dev/null 2>&1
  node "$CA" TASK-D-001 --base "$BASE" --repo "$d" --json > "$TMP/t06.1" 2>/dev/null
  node "$CA" TASK-D-001 --base "$BASE" --repo "$d" --json > "$TMP/t06.2" 2>/dev/null
  cmp -s "$TMP/t06.1" "$TMP/t06.2" || { fail t06 "two runs on identical state differ (not deterministic)"; return; }
  # no wall clock: the artefact carries no ISO-8601 timestamp / date
  grep -Eq '[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}|"(generated|built_at|timestamp|now)"' "$TMP/t06.1" \
    && { fail t06 "the artefact carries a wall-clock field (non-deterministic across days)"; return; }
  ok t06_deterministic
}

# ── t07: guard refuses & names (symlink-out / unreadable / not-a-repo / missing base) ─────────────
t07_guard_refuses_and_names() {
  # (a) spec symlinked OUT of the corpus -> REFUSED + named, never read as authority
  local d="$TMP/t07a"; mkdir -p "$d/docs/tasks/x/TASK-SYM-001" "$TMP/t07ext"
  cat > "$TMP/t07ext/spec.md" <<'SPEC'
---
id: TASK-SYM-001
title: outside
template: task@1
status: implementing
service: whatever
---

# outside the corpus
SPEC
  ln -s "$TMP/t07ext/spec.md" "$d/docs/tasks/x/TASK-SYM-001/spec.md"
  git_init "$d"; ( cd "$d" && git add -A && git commit -qm base ) >/dev/null 2>&1
  local out rc
  out="$(node "$CA" TASK-SYM-001 --base HEAD --repo "$d" 2>&1)"; rc=$?
  { [ "$rc" -eq 2 ] && grep -qiE 'outside the repo root|refused' <<<"$out" && grep -q 'TASK-SYM-001' <<<"$out"; } \
    || { fail t07 "(a) symlink-out spec not refused+named (rc=$rc): $out"; return; }
  # (b) unreadable spec -> REFUSED + named (unreadable is not "clean")
  local e="$TMP/t07b"; mkdir -p "$e/docs/tasks/x/TASK-UNR-001"
  cat > "$e/docs/tasks/x/TASK-UNR-001/spec.md" <<'SPEC'
---
id: TASK-UNR-001
title: t
template: task@1
status: implementing
service: x
---

# TASK-UNR-001
SPEC
  git_init "$e"; ( cd "$e" && git add -A && git commit -qm base ) >/dev/null 2>&1
  chmod 000 "$e/docs/tasks/x/TASK-UNR-001/spec.md"
  if [ -r "$e/docs/tasks/x/TASK-UNR-001/spec.md" ]; then
    echo "  SKIP t07(b) unreadable-spec: chmod 000 is still readable here (running as root?)"
  else
    out="$(node "$CA" TASK-UNR-001 --base HEAD --repo "$e" 2>&1)"; rc=$?
    chmod u+rwX "$e/docs/tasks/x/TASK-UNR-001/spec.md" 2>/dev/null
    { [ "$rc" -eq 2 ] && grep -qi 'unreadable\|refused' <<<"$out"; } \
      || { fail t07 "(b) unreadable spec not refused+named (rc=$rc): $out"; return; }
  fi
  # (c) not a git repo -> REFUSED, and it says so (never "zero escapes")
  local ng="$TMP/t07c"; mkdir -p "$ng/docs/tasks/x/TASK-NG-001"
  cat > "$ng/docs/tasks/x/TASK-NG-001/spec.md" <<'SPEC'
---
id: TASK-NG-001
title: t
template: task@1
status: implementing
service: x
---

# TASK-NG-001
SPEC
  out="$(node "$CA" TASK-NG-001 --base HEAD --repo "$ng" 2>&1)"; rc=$?
  { [ "$rc" -eq 2 ] && grep -qi 'not a git repository' <<<"$out"; } \
    || { fail t07 "(c) non-repo not refused (rc=$rc): $out"; return; }
  # (d) no --base and no entry-flip commit -> REFUSED demanding --base (never a guessed zero)
  local nb="$TMP/t07d"; mkdir -p "$nb/docs/tasks/x/TASK-NB-001"
  cat > "$nb/docs/tasks/x/TASK-NB-001/spec.md" <<'SPEC'
---
id: TASK-NB-001
title: t
template: task@1
status: implementing
service: x
---

# TASK-NB-001
SPEC
  git_init "$nb"; ( cd "$nb" && git add -A && git commit -qm base ) >/dev/null 2>&1
  out="$(node "$CA" TASK-NB-001 --repo "$nb" 2>&1)"; rc=$?
  { [ "$rc" -eq 2 ] && grep -qi 'pass --base' <<<"$out"; } \
    || { fail t07 "(d) missing base not refused with --base demand (rc=$rc): $out"; return; }
  ok t07_guard_refuses_and_names
}

# ── t08: the real 2026-07-17 batch names EXACTLY the 3 files that escaped BOTH cones (AC7) ─────────
t08_finds_the_motivating_case() {
  # base = the batch's commit range: parent-of-batch (f5db101b) .. the batch implementing commit
  # (2244b84c "batch: TASK-IMP-110 + TASK-IMP-114 (implementing)"). Using the range as --base audits
  # the historical batch with no checkout. Guarded: if this repo lacks that history (shallow clone /
  # consumer repo), SKIP rather than fail.
  local RANGE="f5db101b..2244b84c"
  if ! ( cd "$repo" && git cat-file -e 2244b84c^{commit} && git cat-file -e f5db101b^{commit} ) >/dev/null 2>&1; then
    echo "  SKIP t08 (AC7): this repo lacks the 2026-07-17 batch commits (f5db101b/2244b84c)"; return
  fi
  if [ ! -f "$repo/docs/tasks/improvement/TASK-IMP-110-outer-loop-skill-curation/spec.md" ] \
     || [ ! -f "$repo/docs/tasks/improvement/TASK-IMP-114-cost-cycle-telemetry/spec.md" ]; then
    echo "  SKIP t08 (AC7): TASK-IMP-110/114 specs not present"; return
  fi
  node "$CA" TASK-IMP-110 --base "$RANGE" --repo "$repo" --json > "$TMP/t08.110" 2>"$TMP/t08.110.err"; local r1=$?
  node "$CA" TASK-IMP-114 --base "$RANGE" --repo "$repo" --json > "$TMP/t08.114" 2>"$TMP/t08.114.err"; local r2=$?
  { [ "$r1" -eq 0 ] && [ "$r2" -eq 0 ]; } || { fail t08 "a run failed (110=$r1 114=$r2): $(cat "$TMP/t08.110.err" "$TMP/t08.114.err")"; return; }
  # files that escaped BOTH cones = the intersection of the two escape sets
  local both; both="$(node -e '
    const fs=require("fs");
    const a=new Set(JSON.parse(fs.readFileSync(process.argv[1],"utf8")).escapes);
    const b=JSON.parse(fs.readFileSync(process.argv[2],"utf8")).escapes;
    process.stdout.write(b.filter(x=>a.has(x)).sort().join("\n"));
  ' "$TMP/t08.110" "$TMP/t08.114")"
  local want; want="$(printf '%s\n' \
    'tools/docs-site/tests/test_render_status_hub.sh' \
    'tools/install/docs-tools/workflow-improve.mjs' \
    'tools/install/tests/test_full_sdp_payload.sh' | sort)"
  [ "$both" = "$want" ] \
    || { fail t08 "escaped-BOTH set is not exactly the 3 (AC7). got:[$both] want:[$want]"; return; }
  ok t08_finds_the_motivating_case
}

# ── t09: build.sh vendors cone-audit.mjs byte-identical and the payload copy runs ────────────────
t09_payload_carries_it() {
  bash "$BUILD" "$TMP/payload" >/dev/null 2>&1 || { fail t09 "build.sh failed"; return; }
  local p="$TMP/payload/docs-tools/cone-audit.mjs"
  [ -s "$p" ] || { fail t09 "cone-audit.mjs not vendored into the payload (dist would not carry it)"; return; }
  cmp -s "$p" "$CA" || { fail t09 "payload copy differs from source (a tool correct in tools/install but stale in dist is correct nowhere)"; return; }
  node "$p" --help >/dev/null 2>&1 || { fail t09 "the vendored copy does not run (--help)"; return; }
  ok t09_payload_carries_it
}

echo "cone-audit suite (TASK-IMP-119):"
t01_escape_is_named
t02_containment_matches_batch_select
t03_undeclared_cone_escapes_everything
t04_placeholder_is_not_a_path
t05_reports_never_refuses
t06_deterministic
t07_guard_refuses_and_names
t08_finds_the_motivating_case
t09_payload_carries_it
echo "test_cone_audit: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
