#!/usr/bin/env bash
# test_benchmark_gates.sh — TASK-IMP-140: the six benchmark-gate checkers no sibling task
# owns (G3 enum cross-check, G4 headline counts, G5 payload reference walker, G6 vendored-
# gate smoke, G13 stuck-WIP report, G16 reinstall idempotency), plus the doc/register/
# changelog meta-asserts. Gate definitions: docs/verification/benchmark-gates.md (the
# published home; TASK-IMP-140 spec embeds the same definitions).
#
#   t01_doc_complete_and_consistent   (AC1) benchmark-gates.md carries sixteen ### G
#        sections, each with the seven fields, a 16-row status table, and severities/tiers
#        matching the spec's embedded definitions.
#   t02_checkers_fail_on_violations   (AC2) each of the six checkers fails (or, for the
#        report-only G13, SPEAKS) on a constructed violation fixture — six negatives.
#   t03_green_at_head_reportonly_declared (AC3) the six checkers run against the repo;
#        report-only gates print their report block; the doc's status table declares
#        exactly the report-only set.
#   t04_g13_reports_never_mutates     (AC4) a backdated in-flight fixture is LISTED and
#        every fixture spec is byte-identical after the run (detection, never mutation).
#   t05_risk_rows_complete            (AC5) the risk register carries exactly seven R-EXT
#        rows, each with all seven fields non-empty and at least one G-reference.
#   t06_brain_record_fixture          (AC6) DEFERRED behind TASK-MEMORY-303 (the live
#        store is FROZEN_RECOVERABLE; spec §1.6 forbids recording below READY). Until the
#        store repair lands, this asserts the ready-to-run recording deliverable exists in
#        the task folder; the fixture-store demonstration ships with the §1.6 execution.
#   t07_changelog_four_deliverables   (AC7) CHANGELOG's top entry names doc, suite,
#        register rows, and the BRAIN record.
#
# CHECKERS ARE WRITTEN AGAINST THE POST-HARDENING EXPECTED STATE (batch/8): unified
# 12-value enum, fail-closed gates, vendored-CAF ROOT fix, route-back ceiling 3. Mid-wave
# a checker may fail on a surface a sibling has not landed yet; every failure message
# names the exact file + expectation so the final sequential pass can act on it.
#
# Heavy checkers (G5/G6/G16) build a scratch payload with tools/install/build.sh into
# $TMP (never dist/) and install into scratch repos. Set CYBEROS_BENCHMARK_SKIP_HEAVY=1
# to defer them (loud "defer" lines, not failures) — used for mid-wave dry-runs where the
# payload sources are being edited concurrently.
#
# Registration: run_all.sh's scripts/tests/test_*.sh glob (the glob IS the registration).
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../.." && pwd)"

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
case "$TMP/" in "$repo"/*) echo "FATAL scratch $TMP is under the repo $repo"; exit 1 ;; esac

PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

DOC="$repo/docs/verification/benchmark-gates.md"
SPEC="$repo/docs/tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection/spec.md"
REGISTER="$repo/docs/reference/risk-register.md"
TASKDIR="$repo/docs/tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection"

_sha() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"; else shasum -a 256 "$@"; fi; }

# ISO-8601 (or YYYY-MM-DD prefix) -> epoch seconds; GNU date first, BSD date fallback.
to_epoch() {
  local d="$1"
  date -d "$d" +%s 2>/dev/null && return 0
  date -j -f '%Y-%m-%d' "$(printf '%s' "$d" | cut -c1-10)" +%s 2>/dev/null && return 0
  echo ""
}

# ══════════════════════════ G3 — status-enum single source ══════════════════════════════
# Five surfaces, one 12-value enum. Parenthesized qualifiers are stripped BEFORE token
# extraction, which cleanly drops `(type: bug only)`, `(requires duplicate_of, FM-113)`
# and `(per modules/...)` — no hand-kept exception list.
_g03_tokens_sorted() { tr ' ' '\n' <<<"$1" | sed '/^$/d' | LC_ALL=C sort -u; }

t_g03() { # $1 = root (defaults to repo)
  local root="${1:-$repo}" bad=0
  local sr="$root/modules/skill/contracts/task/STATUS-REFERENCE.md"
  local rubric="$root/modules/skill/task-audit/RUBRIC.md"
  local lint="$root/tools/install/docs-tools/task-lint.mjs"
  local hub="$root/tools/docs-site/render-status-hub.mjs"
  local tpl="$root/tools/install/templates/BACKLOG.md"
  local f
  for f in "$sr" "$rubric" "$lint" "$hub" "$tpl"; do
    [ -f "$f" ] || { echo "    g03: surface missing: $f" >&2; return 1; }
  done

  # canonical: the §1.1 + §1.2 table rows of STATUS-REFERENCE ("| <n> | `status` |")
  local canonical
  canonical="$(grep -E '^\| [0-9]+ \| `[a-z_]+`' "$sr" | sed -E 's/^\| [0-9]+ \| `([a-z_]+)`.*/\1/' | LC_ALL=C sort -u)"
  local n; n="$(wc -l <<<"$canonical" | tr -d ' ')"
  [ "$n" = "12" ] || { echo "    g03: $sr §1 tables yield $n statuses, expected the canonical 12" >&2; bad=1; }

  _cmp() { # $1 surface-name  $2 extracted-set
    if [ "$2" != "$canonical" ]; then
      echo "    g03: enum fork at $1" >&2
      echo "      missing there: $(comm -23 <(echo "$canonical") <(echo "$2") | tr '\n' ' ')" >&2
      echo "      extra there:   $(comm -13 <(echo "$canonical") <(echo "$2") | tr '\n' ' ')" >&2
      bad=1
    fi
  }

  # RUBRIC FM-104 row: cut everything before 'one of:' (drops the id + field-name
  # columns, whose `status` token is not an enum value), strip parenthesized qualifiers,
  # then take backticked [a-z_]+ tokens
  local fm104
  fm104="$(grep -F '| `FM-104` |' "$rubric" | sed 's/.*one of://; s/([^)]*)//g' | grep -oE '`[a-z_]+`' | tr -d '`' | LC_ALL=C sort -u)"
  _cmp "modules/skill/task-audit/RUBRIC.md FM-104" "$fm104"

  # the two mjs STATUSES arrays (may span lines)
  local s
  for f in "$lint" "$hub"; do
    s="$(sed -n '/const STATUSES = \[/,/\]/p' "$f" | grep -oE "'[a-z_]+'|\"[a-z_]+\"" | tr -d "'\"" | LC_ALL=C sort -u)"
    _cmp "${f#$root/} STATUSES" "$s"
  done

  # vendored BACKLOG template: the Lifecycle line's chain + Off-ramps vocabulary
  local line l1 l2 tset
  line="$(grep -m1 '^Lifecycle:' "$tpl" | sed 's/([^)]*)//g')"
  [ -n "$line" ] || { echo "    g03: $tpl has no 'Lifecycle:' line to parse" >&2; return 1; }
  l1="$(sed -E 's/^Lifecycle:[[:space:]]*//; s/\..*$//' <<<"$line" | grep -oE '[a-z_]+' || true)"
  l2="$(sed -E 's/^.*Off-ramps:[[:space:]]*//; s/\.[[:space:]]*See.*$//' <<<"$line" | grep -oE '[a-z_]+' || true)"
  tset="$(printf '%s\n%s\n' "$l1" "$l2" | sed '/^$/d' | LC_ALL=C sort -u)"
  _cmp "tools/install/templates/BACKLOG.md lifecycle+off-ramps" "$tset"

  return "$bad"
}

# ══════════════════════════ G4 — headline-count truth ═══════════════════════════════════
t_g04() { # $1 = root (defaults to repo)
  local root="${1:-$repo}" bad=0
  local m w t d dir
  m="$(ls -d "$root"/modules/*/ 2>/dev/null | wc -l | tr -d ' ')"
  w="$(ls "$root"/modules/*/*/workflows/*.md 2>/dev/null | wc -l | tr -d ' ')"
  t="$(ls "$root"/docs/tasks/*/TASK-*/spec.md 2>/dev/null | wc -l | tr -d ' ')"
  d=0
  for dir in "$root"/docs/tasks/*/; do
    ls "$dir"TASK-*/spec.md >/dev/null 2>&1 && d=$((d+1))
  done
  grep -q "$m federated modules" "$root/README.md" \
    || { echo "    g04: README.md module claim stale — tree measures $m federated modules (fix: update the headline line)" >&2; bad=1; }
  grep -q "$w CUO workflows" "$root/README.md" \
    || { echo "    g04: README.md workflow claim stale — tree measures $w CUO workflows (fix: update the headline line)" >&2; bad=1; }
  grep -q "$t tasks" "$root/README.md" \
    || { echo "    g04: README.md task claim stale — tree measures $t tasks (fix: update the headline line)" >&2; bad=1; }
  grep -q "$t task specs across $d domains" "$root/docs/README.md" \
    || { echo "    g04: docs/README.md claim stale — tree measures $t task specs across $d domains (fix: the tasks/ row in the directory map)" >&2; bad=1; }
  return "$bad"
}

# ══════════════════════════ G5 — payload reference walker ═══════════════════════════════
# Structural exclusions — every entry commented (allowlist-with-reasons; G5 edge case):
#   AGENT-ENTRY(.md)  written by install.sh (heredoc, step 5), never a payload file
#                     (refs may omit the .md suffix — match both forms)
#   status-site/      docs-site output tree, generated at install / docs-build time
#   status.html       top-level status hub pointer written by install, not vendored
#   memory/store      the runtime BRAIN store — created by the writer, not vendored
#   gates.env         generated by install.sh autodetect at install time
#   config.yaml       scaffolded once per repo by install.sh, operator-owned after
#   sessions/         §18 transcript ledger — runtime state under the store
#   docs/tasks        consumer-repo corpus paths, not payload deliverables
_G5_EXCLUDE='^\.cyberos/(AGENT-ENTRY(\.md)?|status-site(/|$)|status\.html|memory/store|gates\.env|config\.yaml|memory/sessions|sessions/|docs/tasks)'

t_g05() { # $1 = payload dir (built)
  local pay="$1" bad=0 f ref rel
  [ -d "$pay" ] || { echo "    g05: payload dir missing: $pay" >&2; return 1; }
  local misses="$TMP/g05.misses"; : > "$misses"
  while IFS= read -r f; do
    # skip exempted lines, extract .cyberos/<path> refs, trim trailing punctuation
    while IFS= read -r ref; do
      ref="$(sed -E 's/[]).,;:`"'"'"'[]*$//' <<<"$ref")"
      case "$ref" in *'*'*|*'<'*|*'{'*|*'$'*) continue ;; esac   # glob/placeholder examples
      grep -qE "$_G5_EXCLUDE" <<<"$ref" && continue
      rel="${ref#.cyberos/}"
      [ "$rel" = ".cyberos" ] || [ -z "$rel" ] && continue
      [ -e "$pay/$rel" ] || echo "$ref  (referenced by ${f#$pay/})" >> "$misses"
    done < <(grep -v 'benchmark-gates:exempt' "$f" 2>/dev/null | grep -oE '\.cyberos/[A-Za-z0-9_./-]+' | LC_ALL=C sort -u)
  done < <(find "$pay" -type f \( -name '*.md' -o -name '*.sh' -o -name '*.mjs' -o -name '*.yaml' -o -name '*.yml' -o -name '*.json' \))
  if [ -s "$misses" ]; then
    echo "    g05: vendored files reference payload paths that do not exist (fix the vendor list in tools/install/build.sh, the referencing doc, or add a 'benchmark-gates:exempt' marker with a reason):" >&2
    LC_ALL=C sort -u "$misses" | sed 's/^/      /' >&2
    bad=1
  fi
  return "$bad"
}

# ══════════════════════════ G6 — vendored-gate executability ════════════════════════════
t_g06() { # $1 = installed scratch repo
  local ins="$1" bad=0 rc out
  local gate="$ins/.cyberos/cuo/gates/caf/caf_gate.sh"
  [ -f "$gate" ] || { echo "    g06: vendored caf gate missing at $gate (tools/install/build.sh caf vendor block)" >&2; return 1; }
  # the seeded CAF_CMD must name a script that exists in the install
  local seeded
  seeded="$(sed -n 's/^CAF_CMD="\(.*\)"$/\1/p' "$ins/.cyberos/gates.env" | head -1)"
  [ -n "$seeded" ] || { echo "    g06: gates.env seeds no CAF_CMD (expected: bash .cyberos/cuo/gates/caf/caf_gate.sh .) — tools/install/install.sh step 3" >&2; bad=1; }
  case "$seeded" in
    *caf_gate.sh*) : ;;
    *) echo "    g06: seeded CAF_CMD '$seeded' does not invoke the vendored caf_gate.sh" >&2; bad=1 ;;
  esac

  # no profile at the scratch root -> the SEMANTIC fail-closed exit (1), never structural
  out="$( (cd "$ins" && bash .cyberos/cuo/gates/caf/caf_gate.sh . ) 2>&1 )"; rc=$?
  if [ "$rc" -eq 127 ] || [ "$rc" -eq 2 ] || grep -qE 'not vendored|command not found|No such file' <<<"$out"; then
    echo "    g06: caf_gate.sh '.' structural failure in a consumer-shaped install (exit $rc — the vendored-CAF ROOT-resolution class):" >&2
    sed 's/^/      /' <<<"$out" | head -5 >&2
    bad=1
  elif [ "$rc" -ne 1 ] || ! grep -q 'FAIL-CLOSED' <<<"$out"; then
    echo "    g06: caf_gate.sh '.' without a profile expected the semantic FAIL-CLOSED exit 1, got exit $rc:" >&2
    sed 's/^/      /' <<<"$out" | head -5 >&2
    bad=1
  fi

  # minimal CAF_ENABLED=true fixture: a root audit-profile.yaml -> the CLEAN exit (0)
  printf '# g06 smoke fixture\nconfig:\n  RUN_COMMANDS: true\n' > "$ins/audit-profile.yaml"
  out="$( (cd "$ins" && bash .cyberos/cuo/gates/caf/caf_gate.sh . ) 2>&1 )"; rc=$?
  rm -f "$ins/audit-profile.yaml"
  if [ "$rc" -ne 0 ] || ! grep -q 'CLEAN' <<<"$out"; then
    echo "    g06: caf_gate.sh '.' with a trivial root audit-profile.yaml (RUN_COMMANDS: true) expected CLEAN exit 0, got exit $rc:" >&2
    sed 's/^/      /' <<<"$out" | head -8 >&2
    bad=1
  fi
  return "$bad"
}

# ══════════════════════════ G13 — stuck-WIP detector (report-only) ══════════════════════
t_g13() { # $1 = corpus root (defaults to repo)
  local root="${1:-$repo}"
  local threshold="${CYBEROS_G13_THRESHOLD_DAYS:-30}"
  local now scanned=0 stale=0 spec status created epoch gitep age id
  now="$(date +%s)"
  # git last-touch dates are trusted only with real history (a shallow CI checkout makes
  # every file look freshly touched, which would silence the detector).
  local use_git=0
  if git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
    [ "$(git -C "$root" rev-list --count HEAD 2>/dev/null || echo 1)" -gt 1 ] && use_git=1
  fi
  local found_any=0
  for spec in "$root"/docs/tasks/*/TASK-*/spec.md; do
    [ -e "$spec" ] || continue
    found_any=1
    status="$(sed -n 's/^status:[[:space:]]*//p' "$spec" | head -1 | tr -d '"' | tr -d "'")"
    case "$status" in implementing|ready_to_review|reviewing|ready_to_test|testing) ;; *) continue ;; esac
    scanned=$((scanned+1))
    created="$(sed -n 's/^created_at:[[:space:]]*//p' "$spec" | head -1 | tr -d '"' | tr -d "'")"
    epoch="$(to_epoch "$created" | head -1)"
    if [ "$use_git" -eq 1 ]; then
      # "last recorded transition" = the newest commit where the status VALUE changed
      # (added status: lines differ from removed ones), --follow so a tree rename does
      # not restart every clock, -G so only status-touching commits are patch-scanned.
      # A corpus-wide prose polish or a pure rename must not make a stuck task look
      # fresh; a file appearing WITH a status (its creation) is its first transition.
      gitep="$(git -C "$root" log --follow -G'^status:' --format='COMMIT %ct' -p -- "${spec#$root/}" 2>/dev/null | awk '
        /^COMMIT /{if (!done && seen && a != r) {print ct; done=1; exit}; ct=$2; a=""; r=""; seen=1; next}
        /^\+status:/{a=a substr($0,2) ";"}
        /^-status:/{r=r substr($0,2) ";"}
        END{if (!done && seen && a != r) print ct}' || true)"
      [ -n "$gitep" ] && [ -n "$epoch" ] && [ "$gitep" -gt "$epoch" ] && epoch="$gitep"
      [ -n "$gitep" ] && [ -z "$epoch" ] && epoch="$gitep"
    fi
    [ -n "$epoch" ] || { echo "  g13 REPORT unparseable created_at '$created' on ${spec#$root/} — cannot age it"; continue; }
    age=$(( (now - epoch) / 86400 ))
    if [ "$age" -gt "$threshold" ]; then
      stale=$((stale+1))
      id="$(basename "$(dirname "$spec")")"; id="${id%%-[a-z]*}"
      # detection automated, decision human (G13 tier): this line is the whole output
      # contract — it changes no status and writes no file (spec §1.4).
      echo "  g13 REPORT stale-WIP: ${id} status=${status} age=${age}d (> ${threshold}d) — operator triage: resume / route_back / on_hold (${spec#$root/})"
    fi
  done
  [ "$found_any" -eq 1 ] || { echo "    g13: detector found NO task specs under $root/docs/tasks — a silent detector is the gate failure" >&2; return 1; }
  echo "  g13: scanned $scanned in-flight spec(s), $stale stale (threshold ${threshold}d) — report-only, no status was changed"
  return 0
}

# ══════════════════════════ G16 — reinstall idempotency ═════════════════════════════════
# Diff-exclusion list — every entry commented (G16 edge case: the list may not silently
# grow into "modulo everything"):
#   gates\.env\.bak\.[0-9]+   timestamped backup churn — install.sh removes old backups and
#                             re-creates one per regeneration (install.sh steps 3/3b)
_G16_EXCLUDE='/gates\.env\.bak\.[0-9]+$'

_g16_tree_fp() { # $1 = .cyberos dir -> per-file sha listing on stdout
  ( cd "$1" && find . -type f | grep -vE "$_G16_EXCLUDE" | LC_ALL=C sort | while IFS= read -r f; do _sha "$f"; done )
}

t_g16() { # $1 = payload dir
  local pay="$1" bad=0
  local d="$TMP/g16"; mkdir -p "$d"
  ( cd "$d" && git init -q . && git config user.email g16@test && git config user.name g16 )
  # a PRE-SET operator override: the C1 wipe class is exactly this file degrading silently
  mkdir -p "$d/.cyberos"
  printf '# operator override (g16 fixture)\ngates:\n  test: "echo g16-config-survived"\n' > "$d/.cyberos/config.yaml"
  local cfg_before; cfg_before="$(_sha "$d/.cyberos/config.yaml" | awk '{print $1}')"

  bash "$pay/install.sh" "$d" >"$TMP/g16.install1.log" 2>&1 \
    || { echo "    g16: first install exited nonzero — $(tail -2 "$TMP/g16.install1.log" | head -1)" >&2; return 1; }
  local fp1="$TMP/g16.fp1"; _g16_tree_fp "$d/.cyberos" > "$fp1"

  # reader poll during install #2: the vendored spine file must never be absent (the
  # partial-vendor window TASK-IMP-137 closes with its staged swap).
  local absent=0
  bash "$pay/install.sh" "$d" >"$TMP/g16.install2.log" 2>&1 &
  local ipid=$!
  while kill -0 "$ipid" 2>/dev/null; do
    [ -e "$d/.cyberos/cuo/ship-tasks.md" ] || absent=1
    sleep 0.05
  done
  wait "$ipid" || { echo "    g16: second install exited nonzero — $(tail -2 "$TMP/g16.install2.log" | head -1)" >&2; return 1; }
  local fp2="$TMP/g16.fp2"; _g16_tree_fp "$d/.cyberos" > "$fp2"

  if ! diff -u "$fp1" "$fp2" > "$TMP/g16.diff" 2>&1; then
    echo "    g16: install -> reinstall diverged (.cyberos tree, exclusions applied) — divergent files:" >&2
    grep -E '^[+-][^+-]' "$TMP/g16.diff" | sed 's/^/      /' | head -12 >&2
    bad=1
  fi
  if [ "$(_sha "$d/.cyberos/config.yaml" | awk '{print $1}')" != "$cfg_before" ]; then
    echo "    g16: pre-set .cyberos/config.yaml did NOT survive reinstall byte-identical — the C1 config-wipe class (tools/install/install.sh must never clobber it)" >&2
    bad=1
  fi
  if [ "$absent" -eq 1 ]; then
    echo "    g16: reader-visible vendored-tree absence during reinstall (.cyberos/cuo/ship-tasks.md vanished mid-loop) — the partial-vendor window TASK-IMP-137's atomic swap closes" >&2
    bad=1
  fi
  return "$bad"
}

# ═════════════════════════════ AC meta-tests ════════════════════════════════════════════

t01_doc_complete_and_consistent() {
  local all=1 n
  [ -f "$DOC" ] || { fail t01_doc_complete_and_consistent "$DOC missing"; return; }
  local count; count="$(grep -cE '^### G[0-9]+ — ' "$DOC")"
  [ "$count" = "16" ] || { fail t01_doc_complete_and_consistent "doc carries $count '### G' sections, expected 16"; all=0; }
  local field section
  for n in $(seq 1 16); do
    section="$(awk -v s="^### G$n — " -v e="^### G[0-9]+ — " '$0 ~ s {f=1} f && $0 ~ e && $0 !~ s {exit} f' "$DOC")"
    [ -n "$section" ] || { fail t01_doc_complete_and_consistent "doc section G$n missing (header '### G$n — ...')"; all=0; continue; }
    for field in '**Purpose:**' '**Pass/fail:**' '**Severity:**' '**Tier:**' '**Test method:**' '**Checked files:**' '**Owner:**'; do
      grep -qF "$field" <<<"$section" || { fail t01_doc_complete_and_consistent "doc section G$n lacks the $field field"; all=0; }
    done
    grep -qE "^\| G$n \|" "$DOC" || { fail t01_doc_complete_and_consistent "status table lacks a row for G$n"; all=0; }
    # severity/tier must match the spec's embedded definition (same gates, same
    # severities, same tiers — the doc is their published home, not a fork)
    if [ -f "$SPEC" ]; then
      local shead ssev stier dsev dtier
      shead="$(grep -m1 -E "^### G$n - " "$SPEC")"
      ssev="$(grep -oE 'severity: [a-z]+' <<<"$shead" | awk '{print $2}')"
      stier="$(grep -oE 'tier: [a-z+]+' <<<"$shead" | awk '{print $2}')"
      dsev="$(grep -m1 -F '**Severity:**' <<<"$section" | sed -E 's/.*\*\*Severity:\*\* ([a-z]+).*/\1/')"
      dtier="$(grep -m1 -F '**Tier:**' <<<"$section" | sed -E 's/.*\*\*Tier:\*\* ([a-z+]+).*/\1/')"
      [ "$ssev" = "$dsev" ] || { fail t01_doc_complete_and_consistent "G$n severity forked: spec says '$ssev', doc says '$dsev'"; all=0; }
      [ "$stier" = "$dtier" ] || { fail t01_doc_complete_and_consistent "G$n tier forked: spec says '$stier', doc says '$dtier'"; all=0; }
    fi
  done
  [ "$all" -eq 1 ] && ok t01_doc_complete_and_consistent
}

t02_checkers_fail_on_violations() {
  local all=1 d out

  # g03 negative: a five-surface fixture whose task-lint copy lost 'duplicate'
  d="$TMP/g03root"
  mkdir -p "$d/modules/skill/contracts/task" "$d/modules/skill/task-audit" \
           "$d/tools/install/docs-tools" "$d/tools/docs-site" "$d/tools/install/templates"
  cp "$repo/modules/skill/contracts/task/STATUS-REFERENCE.md" "$d/modules/skill/contracts/task/"
  cp "$repo/modules/skill/task-audit/RUBRIC.md" "$d/modules/skill/task-audit/"
  sed 's/, "duplicate"\]/]/' "$repo/tools/install/docs-tools/task-lint.mjs" > "$d/tools/install/docs-tools/task-lint.mjs"
  cp "$repo/tools/docs-site/render-status-hub.mjs" "$d/tools/docs-site/"
  cp "$repo/tools/install/templates/BACKLOG.md" "$d/tools/install/templates/"
  if t_g03 "$d" >/dev/null 2>&1; then
    fail t02_checkers_fail_on_violations "g03 passed a fixture whose task-lint.mjs lost the 'duplicate' status"; all=0
  fi

  # g04 negative: a tiny tree whose README claims wrong counts
  d="$TMP/g04root"
  mkdir -p "$d/modules/a/b/workflows" "$d/docs/tasks/dom/TASK-X-001-x" "$d/docs"
  printf '# w\n' > "$d/modules/a/b/workflows/w.md"
  printf -- '---\nstatus: draft\ncreated_at: 2026-07-01T00:00:00Z\n---\n' > "$d/docs/tasks/dom/TASK-X-001-x/spec.md"
  printf '9 federated modules, 9 CUO workflows, 9 tasks.\n' > "$d/README.md"
  printf '9 task specs across 9 domains\n' > "$d/docs/README.md"
  if t_g04 "$d" >/dev/null 2>&1; then
    fail t02_checkers_fail_on_violations "g04 passed a fixture whose README counts disagree with the tree"; all=0
  fi

  # g05 negative: a payload referencing a path it does not deliver
  d="$TMP/g05pay"
  mkdir -p "$d/cuo"
  printf 'run node .cyberos/docs-tools/ghost-tool.mjs then stop\n' > "$d/cuo/doc.md"
  if t_g05 "$d" >/dev/null 2>&1; then
    fail t02_checkers_fail_on_violations "g05 passed a payload referencing .cyberos/docs-tools/ghost-tool.mjs which it does not deliver"; all=0
  fi

  # g06 negative: an install with the seeded CAF_CMD but no vendored gate script
  d="$TMP/g06ins"
  mkdir -p "$d/.cyberos"
  printf 'CAF_CMD="bash .cyberos/cuo/gates/caf/caf_gate.sh ."\n' > "$d/.cyberos/gates.env"
  if t_g06 "$d" >/dev/null 2>&1; then
    fail t02_checkers_fail_on_violations "g06 passed an install whose vendored caf_gate.sh is missing"; all=0
  fi

  # g13 negative-of-silence: a backdated in-flight fixture MUST be listed (the report-only
  # detector's failure mode is silence, not a nonzero exit)
  d="$TMP/g13root"
  mkdir -p "$d/docs/tasks/dom/TASK-STALE-001-fixture"
  printf -- '---\nstatus: implementing\ncreated_at: 2026-01-01T00:00:00Z\n---\n# stale fixture\n' > "$d/docs/tasks/dom/TASK-STALE-001-fixture/spec.md"
  out="$(t_g13 "$d" 2>&1)"
  grep -q 'TASK-STALE-001' <<<"$out" \
    || { fail t02_checkers_fail_on_violations "g13 stayed SILENT on a backdated implementing fixture (created_at 2026-01-01): $out"; all=0; }

  # g16 negative: the tree-fingerprint assert must see a divergent tree, and the config
  # sha compare must see a degraded override (the install mechanics run in the real t_g16)
  mkdir -p "$TMP/g16a" "$TMP/g16b"
  printf 'same\n' > "$TMP/g16a/f"; printf 'same\n' > "$TMP/g16b/f"; printf 'extra\n' > "$TMP/g16b/g"
  if [ "$(_g16_tree_fp "$TMP/g16a")" = "$(_g16_tree_fp "$TMP/g16b")" ]; then
    fail t02_checkers_fail_on_violations "g16 tree fingerprint treated divergent trees as identical"; all=0
  fi
  printf 'gates:\n  test: "real"\n' > "$TMP/g16a/config.yaml"; printf 'gates:\n  test: "wiped"\n' > "$TMP/g16b/config.yaml"
  if [ "$(_sha "$TMP/g16a/config.yaml" | awk '{print $1}')" = "$(_sha "$TMP/g16b/config.yaml" | awk '{print $1}')" ]; then
    fail t02_checkers_fail_on_violations "g16 config compare treated a degraded override as identical"; all=0
  fi

  [ "$all" -eq 1 ] && ok t02_checkers_fail_on_violations
}

t03_green_at_head_reportonly_declared() {
  local all=1 heavy_ok=1

  t_g03 || { fail t03_green_at_head_reportonly_declared "G3 enum cross-check failed at HEAD (surfaces above; expected the unified 12-value enum)"; all=0; }
  t_g04 || { fail t03_green_at_head_reportonly_declared "G4 headline counts failed at HEAD (measured numbers above; a one-line doc edit restores truth)"; all=0; }

  if [ "${CYBEROS_BENCHMARK_SKIP_HEAVY:-0}" = "1" ]; then
    echo "  defer g05/g06/g16 — CYBEROS_BENCHMARK_SKIP_HEAVY=1 (mid-wave dry-run; the final pass runs this suite without the flag)"
    heavy_ok=0
  elif ! command -v git >/dev/null 2>&1 || ! command -v node >/dev/null 2>&1 || ! command -v zip >/dev/null 2>&1; then
    echo "  defer g05/g06/g16 — git/node/zip required to build + install the scratch payload"
    heavy_ok=0
  fi

  if [ "$heavy_ok" -eq 1 ]; then
    local PAY="$TMP/payload"
    if bash "$repo/tools/install/build.sh" "$PAY" >"$TMP/build.log" 2>&1; then
      t_g05 "$PAY" || { fail t03_green_at_head_reportonly_declared "G5 payload reference walker failed (missing paths above)"; all=0; }
      local INS="$TMP/g06repo"
      mkdir -p "$INS"; ( cd "$INS" && git init -q . && git config user.email g6@test && git config user.name g6 )
      if bash "$PAY/install.sh" "$INS" >"$TMP/g06.install.log" 2>&1; then
        t_g06 "$INS" || { fail t03_green_at_head_reportonly_declared "G6 vendored-gate smoke failed (structural failure above)"; all=0; }
      else
        fail t03_green_at_head_reportonly_declared "G6 precondition: install.sh failed on a scratch repo — $(tail -2 "$TMP/g06.install.log" | head -1)"; all=0
      fi
      t_g16 "$PAY" || { fail t03_green_at_head_reportonly_declared "G16 reinstall idempotency failed (divergence above)"; all=0; }
    else
      fail t03_green_at_head_reportonly_declared "G5/G6/G16 precondition: tools/install/build.sh failed on a scratch dir — $(tail -2 "$TMP/build.log" | head -1)"; all=0
    fi
  fi

  # G13 is report-only BY TIER: it must run and speak, and must never fail the suite on
  # findings. Its output IS the report block AC 3 requires.
  t_g13 || { fail t03_green_at_head_reportonly_declared "G13 detector errored (a silent/broken detector is the gate failure)"; all=0; }

  # the doc's status table must declare the report-only set: exactly G13 of the six here
  grep -E '^\| G13 \|' "$DOC" | grep -q 'report-only' \
    || { fail t03_green_at_head_reportonly_declared "doc status table does not declare G13 report-only (docs/verification/benchmark-gates.md)"; all=0; }
  local g
  for g in 3 4 5 6 16; do
    grep -E "^\| G$g \|" "$DOC" | grep -q 'live' \
      || { fail t03_green_at_head_reportonly_declared "doc status table row G$g should read 'live' (docs/verification/benchmark-gates.md)"; all=0; }
  done
  [ "$all" -eq 1 ] && ok t03_green_at_head_reportonly_declared
}

t04_g13_reports_never_mutates() {
  local all=1 d="$TMP/g13mut"
  mkdir -p "$d/docs/tasks/dom/TASK-STALE-002-fixture" "$d/docs/tasks/dom/TASK-FRESH-001-fixture"
  printf -- '---\nstatus: implementing\ncreated_at: 2026-01-01T00:00:00Z\n---\n# stale\n' > "$d/docs/tasks/dom/TASK-STALE-002-fixture/spec.md"
  printf -- '---\nstatus: done\ncreated_at: 2026-01-01T00:00:00Z\n---\n# done (never listed)\n' > "$d/docs/tasks/dom/TASK-FRESH-001-fixture/spec.md"
  local before after out
  before="$(cd "$d" && find . -type f | LC_ALL=C sort | while IFS= read -r f; do _sha "$f"; done | _sha | awk '{print $1}')"
  out="$(t_g13 "$d" 2>&1)"
  after="$(cd "$d" && find . -type f | LC_ALL=C sort | while IFS= read -r f; do _sha "$f"; done | _sha | awk '{print $1}')"
  grep -q 'TASK-STALE-002' <<<"$out" || { fail t04_g13_reports_never_mutates "backdated implementing fixture not listed: $out"; all=0; }
  grep -q 'TASK-FRESH-001' <<<"$out" && { fail t04_g13_reports_never_mutates "terminal-status fixture (done) was listed — the in-flight filter is broken"; all=0; }
  [ "$before" = "$after" ] || { fail t04_g13_reports_never_mutates "fixture corpus NOT byte-identical after the detector ran — G13 must never mutate (spec §1.4)"; all=0; }
  [ "$all" -eq 1 ] && ok t04_g13_reports_never_mutates
}

t05_risk_rows_complete() {
  local all=1 n section field
  [ -f "$REGISTER" ] || { fail t05_risk_rows_complete "$REGISTER missing"; return; }
  local count; count="$(grep -cE '^### R-EXT-[0-9]+' "$REGISTER")"
  [ "$count" = "7" ] || { fail t05_risk_rows_complete "register carries $count R-EXT rows, expected exactly the seven audit rows (spec §1.5)"; all=0; }
  for n in 01 02 03 04 05 06 07; do
    section="$(awk -v s="^### R-EXT-$n" '$0 ~ s {f=1; print; next} f && /^### / {exit} f' "$REGISTER")"
    [ -n "$section" ] || { fail t05_risk_rows_complete "R-EXT-$n missing from the register"; all=0; continue; }
    for field in '**Description:**' '**Cause:**' '**Impact:**' '**Detection:**' '**Prevention:**' '**Recovery:**' '**Automation tier:**'; do
      # non-empty = the field line carries content after its ':**'
      grep -F "$field" <<<"$section" | grep -qE ':\*\*[[:space:]]*[^[:space:]]' \
        || { fail t05_risk_rows_complete "R-EXT-$n lacks a non-empty $field field"; all=0; }
    done
    grep -qE 'G[0-9]+' <<<"$section" || { fail t05_risk_rows_complete "R-EXT-$n names no preventing G-gate"; all=0; }
  done
  [ "$all" -eq 1 ] && ok t05_risk_rows_complete
}

t06_brain_record_fixture() {
  # CARVE-OUT (recorded deviation): spec §1.6 + AC 6 demonstrate the BRAIN record on a
  # fixture store — but §1.6 is hard-deferred behind TASK-MEMORY-303's operator-gated
  # store repair (the live store is FROZEN_RECOVERABLE; recording below READY would
  # violate the protocol the audit measured, and the batch/8 partition forbids memory
  # writes from this task). Until 303 lands, the testable truth is: the ready-to-run
  # recording deliverable exists, is executable, and refuses a below-READY store.
  local all=1
  local checklist="$TASKDIR/brain-recording-checklist.md" script="$TASKDIR/brain-record.sh"
  [ -f "$checklist" ] || { fail t06_brain_record_fixture "$checklist missing — the deferred §1.6 step must stay executable-by-checklist"; all=0; }
  [ -f "$script" ] || { fail t06_brain_record_fixture "$script missing — the ready-to-run recording script for the final pass"; all=0; }
  if [ -f "$script" ]; then
    bash -n "$script" 2>/dev/null || { fail t06_brain_record_fixture "$script does not parse (bash -n)"; all=0; }
    grep -q 'doctor' "$script" || { fail t06_brain_record_fixture "$script never checks 'cyberos doctor' — §1.6 forbids recording below READY"; all=0; }
  fi
  echo "  defer t06 full fixture-store demonstration — BRAIN recording gated on TASK-MEMORY-303 (depends_on edge; store FROZEN_RECOVERABLE at authoring)"
  [ "$all" -eq 1 ] && ok "t06_brain_record_fixture (deliverable present; recording deferred per depends_on)"
}

t07_changelog_four_deliverables() {
  local all=1 top want
  top="$(awk '/^## \[/{n++} n==1{print} n==2{exit}' "$repo/CHANGELOG.md")"
  for want in 'benchmark-gates.md' 'test_benchmark_gates' 'R-EXT' 'BRAIN'; do
    grep -qF "$want" <<<"$top" \
      || { fail t07_changelog_four_deliverables "CHANGELOG.md top entry lacks '$want' — paste the prepared entry from docs/tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection/implementation-evidence.md"; all=0; }
  done
  [ "$all" -eq 1 ] && ok t07_changelog_four_deliverables
}

t01_doc_complete_and_consistent
t02_checkers_fail_on_violations
t03_green_at_head_reportonly_declared
t04_g13_reports_never_mutates
t05_risk_rows_complete
t06_brain_record_fixture
t07_changelog_four_deliverables

echo "benchmark-gates: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
