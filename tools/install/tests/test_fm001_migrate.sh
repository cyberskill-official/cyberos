#!/usr/bin/env bash
# test_fm001_migrate.sh - TASK-IMP-117 §1 suite (t01-t08 -> AC1-AC6).
#
# The tool rewrites specs, so every clause is proven against a scratch git repo that bends exactly
# one thing. Each test is LOAD-BEARING: it fails if the clause it guards is violated (proven by
# breaking the tool and watching the matching test go red - recorded in the ship gate log).
#
#   t01 -> clause 1.1 / AC1,AC2  the vendored TASK-TEMPLATE.md lints with zero FM-001
#   t02 -> clause 1.2 / AC3      trailing comments (scalar + list-item + inline-list) move own-line,
#                                the file goes FM-001-clean, values are byte-preserved; --check + --json
#   t03 -> clause 1.3 / AC3      a '#' inside a quoted value, or with no leading whitespace, is NOT a
#                                comment - those lines are byte-identical after a run
#   t04 -> clause 1.4 / AC5      the body (below the 2nd '---') is byte-identical; a conformant spec is
#                                a no-op, so an audit-bound spec's normative-half hash is preserved
#   t05 -> clause 1.5 / AC3      idempotent (second run byte-identical); no-comment no-op; CRLF round-trip
#   t06 -> clause 1.6 / AC4      a whole scratch CORPUS goes FM-001-clean (capability proof; the real
#                                501-spec repo run is deferred to the operator's review gate - preview)
#   t07 -> clause 1.7 / AC3      guard: untracked / escaping / no-frontmatter / not-a-repo are REFUSED
#   t08 -> AC6                   build.sh vendors the migrator byte-identical and the payload copy runs
#   t09 -> clause 1.8 / AC7      nested-map flatten: children hoisted to top-level keys with values +
#                                order preserved; whole scratch corpus (build_envelope + done +
#                                collision) goes FM-001-clean; collision -> order-preserving union
#                                (exact dup deduped, nothing dropped, no FM-003); done spec migrated +
#                                body held; scalar conflict HALTS and names the file; idempotent
#   t10 -> clause 1.9 / AC3      a mid-value apostrophe in a PLAIN scalar is literal, so `broker's ...
#                                #4` detects ` #4` as a comment and moves it own-line; a value that
#                                BEGINS with a quote keeps its '#' as data (1.3 preserved by 1.9)
#
# run_all.sh discovers this file via its tools/install/tests/test_*.sh glob.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
MIG="$repo/tools/install/docs-tools/fm001-migrate.mjs"
LINT="$repo/tools/install/docs-tools/task-lint.mjs"
TMPL="$repo/tools/install/templates/TASK-TEMPLATE.md"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

# FM-001 finding count for one spec file (0 = clean).
fm001() { node "$LINT" --json "$1" 2>/dev/null | node -e 'const d=JSON.parse(require("fs").readFileSync(0,"utf8"));process.stdout.write(String(d.filter(x=>x.rule_id==="FM-001").length))'; }
# FM-003 (duplicate frontmatter key) count for one spec file — a naive nested-map hoist would trip it.
fm003() { node "$LINT" --json "$1" 2>/dev/null | node -e 'const d=JSON.parse(require("fs").readFileSync(0,"utf8"));process.stdout.write(String(d.filter(x=>x.rule_id==="FM-003").length))'; }
# Parsed frontmatter (own-line comments dropped, trailing comments stripped, sorted) - for the
# field-preservation compare. NOT quote-aware, so fixtures that use it carry no quoted '#' in values.
fmparse() { node -e '
  const fs=require("fs");const L=fs.readFileSync(process.argv[1],"utf8").split("\n");
  if(L[0]!=="---"){process.exit(0)} let end=-1;for(let i=1;i<L.length;i++){if(L[i]==="---"){end=i;break}}
  const out=[];for(let i=1;i<end;i++){const l=L[i];if(l.trim()===""||/^\s*#/.test(l))continue;
    out.push(l.replace(/\s+#.*$/,"").replace(/\s+$/,""));} console.log(out.sort().join("\n"))' "$1"; }

# A dirty spec: trailing comments on a scalar, two list items, and an inline list. NO quoted '#'.
dirty_spec() { cat > "$1" <<'SPEC'
---
id: TASK-DEMO-001  # module-scoped, e.g. TASK-AUTH-001
title: A plain title
template: task@1
type: feature      # feature | bug | improvement | chore
status: draft
priority: p2       # p0 | p1 | p2 | p3
author: "@me"
department: engineering
created_at: 2026-05-15T00:00:00+07:00
ai_authorship: none
eu_ai_act_risk_class: not_ai
client_visible: false
new_files:
  - src/a.ts       # add module a
  - src/b.ts       # add module b
related_tasks: [TASK-DEMO-002, TASK-DEMO-003]   # siblings
depends_on: []
routed_back_count: 0
---

# TASK-DEMO-001: a plain title

## 1. Description (normative)

- 1.1 The system SHALL do X.
SPEC
}
# git repo around one spec at docs/tasks/demo/<dir>/spec.md; commits it (tracked at HEAD).
scratch_repo() { local d="$1"; mkdir -p "$d/docs/tasks/demo/D"; dirty_spec "$d/docs/tasks/demo/D/spec.md"
  ( cd "$d" && git init -qb main . && git config user.email t@t && git config user.name t && git add -A && git commit -qm init ) >/dev/null 2>&1; }

# ── t01: the vendored template is FM-001-clean (AC1, AC2) ─────────────────────
t01_template_is_clean() {
  local n; n="$(fm001 "$TMPL")"
  [ "$n" = "0" ] || { fail t01_template_is_clean "template has $n FM-001 finding(s) - a spec born from it is not clean"; return; }
  # and the migrator agrees it needs nothing (no-op / already own-line)
  node "$MIG" --check --repo "$repo" "tools/install/templates/TASK-TEMPLATE.md" >/dev/null 2>&1 \
    || { fail t01_template_is_clean "migrator --check flags the template (it should be born clean)"; return; }
  ok t01_template_is_clean
}

# ── t02: trailing comments move own-line; values preserved; --check; --json ───
t02_migrator_moves_trailing_comments() {
  local d="$TMP/t02"; scratch_repo "$d"; local s="$d/docs/tasks/demo/D/spec.md"
  [ "$(fm001 "$s")" -gt 0 ] || { fail t02 "fixture is not dirty to begin with"; return; }
  local before; before="$(fmparse "$s")"
  local out; out="$(node "$MIG" --repo "$d" docs/tasks/demo/D/spec.md 2>&1)"; local rc=$?
  [ "$rc" -eq 0 ] || { fail t02 "migrate exited $rc: $out"; return; }
  [ "$(fm001 "$s")" = "0" ] || { fail t02 "file still has FM-001 after migration"; return; }
  # values byte-preserved (only comment POSITIONS changed)
  [ "$(fmparse "$s")" = "$before" ] || { fail t02 "a frontmatter value changed (not just comment position)"; return; }
  # the comment is now OWN-LINE above its field, and the trailing form is gone
  grep -qxF '# p0 | p1 | p2 | p3' "$s" && grep -qxF 'priority: p2' "$s" \
    || { fail t02 "scalar comment did not move to its own line above the field"; return; }
  grep -qE '^priority: p2[[:space:]]+#' "$s" && { fail t02 "a trailing comment survived on the value line"; return; }
  grep -qxF '  # add module a' "$s" && grep -qxF '  - src/a.ts' "$s" \
    || { fail t02 "list-item comment did not move above the item with indent preserved"; return; }
  grep -qxF 'related_tasks: [TASK-DEMO-002, TASK-DEMO-003]' "$s" \
    || { fail t02 "inline-list value not preserved after its trailing comment moved"; return; }
  # --check writes nothing and signals exit 2 on a still-dirty file
  local e="$TMP/t02b"; scratch_repo "$e"; local es="$e/docs/tasks/demo/D/spec.md"
  local pre; pre="$(sha256sum "$es")"
  node "$MIG" --check --repo "$e" docs/tasks/demo/D/spec.md >/dev/null 2>&1; [ "$?" -eq 2 ] \
    || { fail t02 "--check on a dirty file did not exit 2"; return; }
  [ "$(sha256sum "$es")" = "$pre" ] || { fail t02 "--check WROTE to the file (must be report-only)"; return; }
  # --json is valid and reports the migration
  local e2="$TMP/t02c"; scratch_repo "$e2"
  node "$MIG" --json --repo "$e2" docs/tasks/demo/D/spec.md 2>/dev/null \
    | node -e 'const j=JSON.parse(require("fs").readFileSync(0,"utf8"));process.exit(j.migrated===1&&j.tool==="fm001-migrate@1"?0:1)' \
    || { fail t02 "--json did not report migrated=1"; return; }
  ok t02_migrator_moves_trailing_comments
}

# ── t03: a '#' inside a quoted value or with no leading ws is not a comment ───
t03_hash_inside_value_is_not_a_comment() {
  local d="$TMP/t03"; mkdir -p "$d/docs/tasks/demo/D"; local s="$d/docs/tasks/demo/D/spec.md"
  cat > "$s" <<'SPEC'
---
id: TASK-DEMO-001  # real trailing comment
title: "Fix the # parsing bug"
label: 'issue # 42 stays'
url_field: https://x/y.html#anchor
template: task@1
status: draft
---

# body
SPEC
  ( cd "$d" && git init -qb main . && git config user.email t@t && git config user.name t && git add -A && git commit -qm init ) >/dev/null 2>&1
  node "$MIG" --repo "$d" docs/tasks/demo/D/spec.md >/dev/null 2>&1
  grep -qx 'title: "Fix the # parsing bug"' "$s" || { fail t03 "a '#' inside a double-quoted value was treated as a comment"; return; }
  grep -qx "label: 'issue # 42 stays'" "$s"      || { fail t03 "a '#' inside a single-quoted value was treated as a comment"; return; }
  grep -qx 'url_field: https://x/y.html#anchor' "$s" || { fail t03 "a '#' with no leading whitespace (a#b) was treated as a comment"; return; }
  # the ONE genuine trailing comment still moved (the tool is not just inert)
  grep -qx 'id: TASK-DEMO-001' "$s" && grep -qx '# real trailing comment' "$s" \
    || { fail t03 "the one real trailing comment was not migrated"; return; }
  ok t03_hash_inside_value_is_not_a_comment
}

# ── t04: body byte-identical; conformant spec is a no-op (binding preserved) ──
t04_body_is_untouched_and_body_hash_holds() {
  local d="$TMP/t04"; scratch_repo "$d"; local s="$d/docs/tasks/demo/D/spec.md"
  # body = everything from the 2nd '---' onward; capture before + after
  body_bytes() { awk 'c>=2{print} /^---$/{c++}' "$1" | sha256sum; }
  local bb; bb="$(body_bytes "$s")"
  node "$MIG" --repo "$d" docs/tasks/demo/D/spec.md >/dev/null 2>&1
  [ "$(fm001 "$s")" = "0" ] || { fail t04 "migration did not clean the file"; return; }
  [ "$(body_bytes "$s")" = "$bb" ] || { fail t04 "the body changed - the migrator touched bytes below the frontmatter"; return; }
  # normative-half (audited_body_sha256_prefix, task-audit §12) of an ALREADY-CLEAN spec is
  # unchanged across a run: a no-op on conformant input is what preserves audit bindings.
  norm_half() { node -e '
    const fs=require("fs"),c=require("crypto");const L=fs.readFileSync(process.argv[1],"utf8").split("\n");
    if(L[0].trim()!=="---"){console.log("x");process.exit(0)} const e=L.findIndex((l,i)=>i>0&&l.trim()==="---");
    const LF=["status","shipped","routed_back_count","memory_chain_hash"];
    const keep=L.slice(1,e).filter(l=>{const m=l.match(/^([A-Za-z0-9_]+):/);return !(m&&LF.includes(m[1]))});
    console.log(c.createHash("sha256").update(Buffer.from([...keep,...L.slice(e+1)].join("\n"))).digest("hex").slice(0,16))' "$1"; }
  local c2="$TMP/t04b"; scratch_repo "$c2"; local cs="$c2/docs/tasks/demo/D/spec.md"
  node "$MIG" --repo "$c2" docs/tasks/demo/D/spec.md >/dev/null 2>&1   # first run: now conformant
  local h1; h1="$(norm_half "$cs")"
  node "$MIG" --repo "$c2" docs/tasks/demo/D/spec.md >/dev/null 2>&1   # no-op on the clean spec
  [ "$(norm_half "$cs")" = "$h1" ] || { fail t04 "a no-op run changed the normative half (an audit binding would break)"; return; }
  ok t04_body_is_untouched_and_body_hash_holds
}

# ── t05: idempotent; no-comment no-op; CRLF round-trip ───────────────────────
t05_idempotent() {
  local d="$TMP/t05"; scratch_repo "$d"; local s="$d/docs/tasks/demo/D/spec.md"
  node "$MIG" --repo "$d" docs/tasks/demo/D/spec.md >/dev/null 2>&1
  cp "$s" "$TMP/t05.once"
  node "$MIG" --repo "$d" docs/tasks/demo/D/spec.md >/dev/null 2>&1
  cmp -s "$TMP/t05.once" "$s" || { fail t05 "second run was not byte-identical (not idempotent)"; return; }
  # edge #1: a spec with NO frontmatter comments is a byte-identical no-op
  local e="$TMP/t05b"; mkdir -p "$e/docs/tasks/demo/D"; local es="$e/docs/tasks/demo/D/spec.md"
  printf -- '---\nid: TASK-DEMO-001\ntitle: no comments here\ntemplate: task@1\nstatus: draft\n---\n\n# body\n' > "$es"
  ( cd "$e" && git init -qb main . && git config user.email t@t && git config user.name t && git add -A && git commit -qm init ) >/dev/null 2>&1
  cp "$es" "$TMP/t05.nc"
  node "$MIG" --repo "$e" docs/tasks/demo/D/spec.md >/dev/null 2>&1
  cmp -s "$TMP/t05.nc" "$es" || { fail t05 "a comment-free spec was modified (should be a no-op)"; return; }
  # edge #9: CRLF is round-tripped, never normalized; and the comment still moves + clears FM-001
  local f="$TMP/t05c"; mkdir -p "$f/docs/tasks/demo/D"; local fs="$f/docs/tasks/demo/D/spec.md"
  printf -- '---\r\nid: TASK-DEMO-001  # c\r\ntitle: crlf\r\ntemplate: task@1\r\nstatus: draft\r\n---\r\n\r\n# body\r\n' > "$fs"
  ( cd "$f" && git init -qb main . && git config user.email t@t && git config user.name t && git add -A && git commit -qm init ) >/dev/null 2>&1
  node "$MIG" --repo "$f" docs/tasks/demo/D/spec.md >/dev/null 2>&1
  grep -qU $'\r' "$fs" || { fail t05 "CRLF line endings were normalized away"; return; }
  [ "$(fm001 "$fs")" = "0" ] || { fail t05 "CRLF file not cleaned"; return; }
  node "$MIG" --repo "$f" docs/tasks/demo/D/spec.md >/dev/null 2>&1; cp "$fs" "$TMP/t05.crlf"
  node "$MIG" --repo "$f" docs/tasks/demo/D/spec.md >/dev/null 2>&1
  cmp -s "$TMP/t05.crlf" "$fs" || { fail t05 "CRLF file not idempotent"; return; }
  ok t05_idempotent
}

# ── t06: a whole scratch corpus goes FM-001-clean (capability; real 501 deferred) ──
t06_corpus_is_fm001_clean() {
  local d="$TMP/t06"; mkdir -p "$d/docs/tasks/demo"
  # three specs spanning all sub-kinds: scalar-only, block-list, inline-list+block
  mkdir -p "$d/docs/tasks/demo/A" "$d/docs/tasks/demo/B" "$d/docs/tasks/demo/C"
  dirty_spec "$d/docs/tasks/demo/A/spec.md"
  printf -- '---\nid: TASK-DEMO-002\ntitle: b\ntemplate: task@1\nstatus: draft\nmodified_files:\n  - x.ts  # x\n  - y.ts  # y\n---\n\n# b\n' > "$d/docs/tasks/demo/B/spec.md"
  printf -- '---\nid: TASK-DEMO-003\ntitle: c\ntemplate: task@1\nstatus: draft\nblocks: [T1, T2]  # z\n---\n\n# c\n' > "$d/docs/tasks/demo/C/spec.md"
  ( cd "$d" && git init -qb main . && git config user.email t@t && git config user.name t && git add -A && git commit -qm init ) >/dev/null 2>&1
  local before=0; for s in "$d"/docs/tasks/demo/*/spec.md; do before=$((before + $(fm001 "$s"))); done
  [ "$before" -gt 0 ] || { fail t06 "scratch corpus was already clean - nothing proven"; return; }
  # cd so the glob expands against the scratch repo, not the caller's cwd (the real consumer runs
  # `node .cyberos/docs-tools/fm001-migrate.mjs docs/tasks/*/*/spec.md` from its own repo root).
  ( cd "$d" && node "$MIG" --repo "$d" docs/tasks/demo/*/spec.md >/dev/null 2>&1 )
  local after=0; for s in "$d"/docs/tasks/demo/*/spec.md; do after=$((after + $(fm001 "$s"))); done
  [ "$after" = "0" ] || { fail t06 "corpus still has $after FM-001 finding(s) after migration"; return; }
  ok t06_corpus_is_fm001_clean
}

# ── t07: the guard refuses untracked / escaping / no-frontmatter / not-a-repo ─
t07_guard_refuses_untracked_and_escaping_paths() {
  local d="$TMP/t07"; scratch_repo "$d"
  # (a) untracked spec on disk -> REFUSED, unchanged
  mkdir -p "$d/docs/tasks/demo/U"; local u="$d/docs/tasks/demo/U/spec.md"; dirty_spec "$u"
  local upre; upre="$(sha256sum "$u")"
  local out; out="$(node "$MIG" --repo "$d" docs/tasks/demo/U/spec.md 2>&1)"; local rc=$?
  { [ "$rc" -eq 2 ] && grep -q "not tracked at HEAD" <<<"$out"; } || { fail t07 "untracked spec was not refused (rc=$rc): $out"; return; }
  [ "$(sha256sum "$u")" = "$upre" ] || { fail t07 "an untracked spec was MIGRATED"; return; }
  # (b) path escaping the repo root -> REFUSED, the outside file untouched
  local outside="$TMP/outside.md"; dirty_spec "$outside"; local opre; opre="$(sha256sum "$outside")"
  out="$(node "$MIG" --repo "$d" ../outside.md 2>&1)"; rc=$?
  { [ "$rc" -eq 2 ] && grep -q "escapes the repo root" <<<"$out"; } || { fail t07 "an escaping path was not refused (rc=$rc): $out"; return; }
  [ "$(sha256sum "$outside")" = "$opre" ] || { fail t07 "a path outside the root was MIGRATED"; return; }
  # (c) tracked file with NO frontmatter -> REFUSED (edge #2)
  mkdir -p "$d/docs/tasks/demo/N"; printf 'no frontmatter here\n' > "$d/docs/tasks/demo/N/spec.md"
  ( cd "$d" && git add -A && git commit -qm "no-fm" ) >/dev/null 2>&1
  out="$(node "$MIG" --repo "$d" docs/tasks/demo/N/spec.md 2>&1)"; rc=$?
  { [ "$rc" -eq 2 ] && grep -q "no frontmatter block" <<<"$out"; } || { fail t07 "a no-frontmatter file was not refused (rc=$rc): $out"; return; }
  # (d) not a git repo -> refuse the whole run and say so (edge #14)
  local ng="$TMP/t07ng"; mkdir -p "$ng/docs/tasks/demo/D"; dirty_spec "$ng/docs/tasks/demo/D/spec.md"; local ngpre; ngpre="$(sha256sum "$ng/docs/tasks/demo/D/spec.md")"
  out="$(node "$MIG" --repo "$ng" docs/tasks/demo/D/spec.md 2>&1)"; rc=$?
  { [ "$rc" -eq 2 ] && grep -q "not a git repository" <<<"$out"; } || { fail t07 "a non-repo run was not refused (rc=$rc): $out"; return; }
  [ "$(sha256sum "$ng/docs/tasks/demo/D/spec.md")" = "$ngpre" ] || { fail t07 "a spec under a non-repo dir was MIGRATED"; return; }
  ok t07_guard_refuses_untracked_and_escaping_paths
}

# ── t08: build.sh vendors the migrator byte-identical and it runs ────────────
t08_payload_carries_it() {
  bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { fail t08 "build.sh failed"; return; }
  local p="$TMP/payload/docs-tools/fm001-migrate.mjs"
  [ -s "$p" ] || { fail t08 "fm001-migrate.mjs not vendored into the payload"; return; }
  cmp -s "$p" "$MIG" || { fail t08 "payload copy differs from source"; return; }
  # the payload copy actually migrates a scratch spec
  local d="$TMP/t08"; scratch_repo "$d"
  node "$p" --repo "$d" docs/tasks/demo/D/spec.md >/dev/null 2>&1
  [ "$(fm001 "$d/docs/tasks/demo/D/spec.md")" = "0" ] || { fail t08 "the vendored copy does not migrate"; return; }
  ok t08_payload_carries_it
}

# ── t09: nested-map flatten; corpus clean; collision union; done-spec; conflict halts ─────────
t09_flattens_nested_map() {
  local d="$TMP/t09"
  mkdir -p "$d/docs/tasks/demo/BE" "$d/docs/tasks/demo/DONE" "$d/docs/tasks/demo/COL"
  # a build_envelope nested map with 6 map children: 2 scalars + 4 block lists (one item carries a ':')
  cat > "$d/docs/tasks/demo/BE/spec.md" <<'SPEC'
---
id: TASK-DEMO-010
title: nested map
template: task@1
status: draft
build_envelope:
  language: rust 1.81
  service: cyberos/services/ai/
  new_files:
    - services/ai/a.rs
    - services/ai/b.rs
  modified_files:
    - services/ai/c.rs
  allowed_tools:
    - file_read: services/ai/**
  disallowed_tools:
    - non-CTO creds write
effort_hours: 12
---

# body stays put

- normative clause
SPEC
  # a done spec carrying build_envelope (edge #18: migrated, body held)
  cat > "$d/docs/tasks/demo/DONE/spec.md" <<'SPEC'
---
id: TASK-DEMO-011
title: done envelope
template: task@1
status: done
build_envelope:
  language: markdown
  service: modules/x/
  new_files:
    - modules/x/a.md
---

# done body
SPEC
  # collision: top-level new_files + build_envelope.new_files sharing one exact item (edge #17)
  cat > "$d/docs/tasks/demo/COL/spec.md" <<'SPEC'
---
id: TASK-DEMO-012
title: collision
template: task@1
status: draft
new_files:
  - modules/y/SHARED.md
build_envelope:
  language: python
  new_files:
    - modules/y/a.py
    - modules/y/SHARED.md
---

# body
SPEC
  ( cd "$d" && git init -qb main . && git config user.email t@t && git config user.name t && git add -A && git commit -qm init ) >/dev/null 2>&1
  local be="$d/docs/tasks/demo/BE/spec.md"
  [ "$(fm001 "$be")" -gt 0 ] || { fail t09 "build_envelope fixture not dirty to begin with"; return; }
  # body (everything from the 2nd '---') hash before, to prove flatten never touches it (edge #18)
  local be_body; be_body="$(awk 'c>=2{print} /^---$/{c++}' "$be" | sha256sum)"
  # migrate the whole scratch corpus in one invocation
  ( cd "$d" && node "$MIG" --repo "$d" docs/tasks/demo/*/spec.md >/dev/null 2>&1 )
  # (b) whole corpus FM-001-clean, INCLUDING build_envelope + done + collision (AC7 capability)
  local after=0; for s in "$d"/docs/tasks/demo/*/spec.md; do after=$((after + $(fm001 "$s"))); done
  [ "$after" = "0" ] || { fail t09 "scratch corpus still has $after FM-001 after flatten"; return; }
  # (c) build_envelope key GONE; all 6 children hoisted to top-level with byte-identical values + order
  grep -q '^build_envelope:' "$be" && { fail t09 "build_envelope parent key survived the flatten"; return; }
  grep -qxF 'language: rust 1.81' "$be" && grep -qxF 'service: cyberos/services/ai/' "$be" \
    || { fail t09 "scalar children not hoisted to top-level with their values"; return; }
  grep -qxF 'new_files:' "$be" && grep -qxF '  - services/ai/a.rs' "$be" && grep -qxF '  - services/ai/b.rs' "$be" \
    || { fail t09 "block-list child new_files not hoisted as a top-level block list (indent/order)"; return; }
  grep -qxF 'modified_files:' "$be" && grep -qxF '  - services/ai/c.rs' "$be" \
    || { fail t09 "modified_files not hoisted"; return; }
  grep -qxF 'allowed_tools:' "$be" && grep -qxF '  - file_read: services/ai/**' "$be" \
    || { fail t09 "allowed_tools (value carrying a ':') not hoisted intact"; return; }
  grep -qxF 'disallowed_tools:' "$be" && grep -qxF '  - non-CTO creds write' "$be" \
    || { fail t09 "disallowed_tools not hoisted"; return; }
  # (d) body byte-identical — the mechanism that holds audited_body_sha256_prefix on a bound spec
  [ "$(awk 'c>=2{print} /^---$/{c++}' "$be" | sha256sum)" = "$be_body" ] \
    || { fail t09 "flatten changed the body (bytes below the frontmatter)"; return; }
  # (e) the done spec was migrated too
  [ "$(fm001 "$d/docs/tasks/demo/DONE/spec.md")" = "0" ] || { fail t09 "done spec not cleaned"; return; }
  # (f) collision reconciled: no FM-003 duplicate key; union keeps SHARED.md ONCE and a.py — nothing dropped
  local col="$d/docs/tasks/demo/COL/spec.md"
  [ "$(fm003 "$col")" = "0" ] || { fail t09 "flatten produced a duplicate key (FM-003) on collision"; return; }
  local nshared; nshared="$(grep -cxF '  - modules/y/SHARED.md' "$col")"
  [ "$nshared" = "1" ] || { fail t09 "SHARED.md appears $nshared times after union (want 1: exact dup deduped, not dropped)"; return; }
  grep -qxF '  - modules/y/a.py' "$col" || { fail t09 "union dropped the unique item a.py"; return; }
  # (g) idempotent across BOTH passes (edge #21): a second run is byte-identical
  cp "$be" "$TMP/t09.once"
  node "$MIG" --repo "$d" docs/tasks/demo/BE/spec.md >/dev/null 2>&1
  cmp -s "$TMP/t09.once" "$be" || { fail t09 "second run not byte-identical (flatten not idempotent)"; return; }
  # (h) a genuine scalar conflict HALTS, names the key, migrates nothing (edge #19)
  mkdir -p "$d/docs/tasks/demo/CONF"; local conf="$d/docs/tasks/demo/CONF/spec.md"
  cat > "$conf" <<'SPEC'
---
id: TASK-DEMO-013
title: scalar conflict
template: task@1
status: draft
service: top-level-value
build_envelope:
  service: envelope-value
---

# body
SPEC
  ( cd "$d" && git add -A && git commit -qm conf ) >/dev/null 2>&1
  local cpre; cpre="$(sha256sum "$conf")"
  local out; out="$(node "$MIG" --repo "$d" docs/tasks/demo/CONF/spec.md 2>&1)"; local rc=$?
  { [ "$rc" -eq 2 ] && grep -q "collides on scalar key 'service'" <<<"$out"; } \
    || { fail t09 "scalar conflict not refused + named (rc=$rc): $out"; return; }
  [ "$(sha256sum "$conf")" = "$cpre" ] || { fail t09 "a scalar-conflict spec was MIGRATED (must halt, touch nothing)"; return; }
  ok t09_flattens_nested_map
}

# ── t10: a mid-value apostrophe is literal; ` #` after it is a comment; quoted '#' stays data ─
t10_apostrophe_then_hash_is_a_comment() {
  local d="$TMP/t10"; mkdir -p "$d/docs/tasks/demo/AP"; local s="$d/docs/tasks/demo/AP/spec.md"
  cat > "$s" <<'SPEC'
---
id: TASK-DEMO-020
title: apostrophe
template: task@1
status: draft
disallowed_tools:
  - allow subprocess to inherit broker's descriptors (per note #4 - seal stdio only)
  - dispatch without checking allowed_tools (per DEC-191)
label: 'issue # 42 stays'
title2: "Fix the # bug"
---

# body
SPEC
  ( cd "$d" && git init -qb main . && git config user.email t@t && git config user.name t && git add -A && git commit -qm init ) >/dev/null 2>&1
  # task-lint flags the ' #4' as a trailing comment; the OLD quote model missed it (broker's apostrophe)
  [ "$(fm001 "$s")" -gt 0 ] || { fail t10 "apostrophe fixture not dirty (task-lint should flag the ' #4' trailing comment)"; return; }
  node "$MIG" --repo "$d" docs/tasks/demo/AP/spec.md >/dev/null 2>&1
  [ "$(fm001 "$s")" = "0" ] || { fail t10 "apostrophe line not cleaned — ' #4' was not detected as a comment"; return; }
  # the ' #4 ...' moved to its own line above, indent preserved; the value line keeps everything left of it
  grep -qxF '  #4 - seal stdio only)' "$s" || { fail t10 "the ' #4' comment did not move to its own line"; return; }
  grep -qxF "  - allow subprocess to inherit broker's descriptors (per note" "$s" \
    || { fail t10 "the value line (with the literal mid-value apostrophe) was not preserved"; return; }
  # a value that BEGINS with a quote keeps its '#' as data — 1.3's protection preserved by 1.9
  grep -qxF "label: 'issue # 42 stays'" "$s" || { fail t10 "single-quoted value with '#' was altered"; return; }
  grep -qxF 'title2: "Fix the # bug"' "$s" || { fail t10 "double-quoted value with '#' was altered"; return; }
  ok t10_apostrophe_then_hash_is_a_comment
}

echo "fm001-migrate suite (TASK-IMP-117):"
t01_template_is_clean
t02_migrator_moves_trailing_comments
t03_hash_inside_value_is_not_a_comment
t04_body_is_untouched_and_body_hash_holds
t05_idempotent
t06_corpus_is_fm001_clean
t07_guard_refuses_untracked_and_escaping_paths
t08_payload_carries_it
t09_flattens_nested_map
t10_apostrophe_then_hash_is_a_comment
echo "test_fm001_migrate: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
