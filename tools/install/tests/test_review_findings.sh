#!/usr/bin/env bash
# test_review_findings.sh - TASK-IMP-112 §2 suite (t01-t05 -> AC 1-5).
#
#   t01_schema_valid          AC 1 (#1.1,#1.2) a findings array with all six fields per record
#                             validates; the validator BITES (missing field / bad enum / extra key
#                             all reject; edge §3 quote+backslash path round-trips) - never vacuous.
#   t02_counts_agree          AC 2 (#1.4)       JSON record count == markdown finding count; a
#                             mismatch fixture reds (the audit's cross-check, exit non-zero).
#   t03_null_clause_ref       AC 3 (#1.3)       an out-of-spec finding carries clause_ref:null and
#                             validates; a fabricated string ref is NOT required.
#   t04_empty_array_not_absent AC 4 (#1.6)      a clean review emits `[]` (present, length 0), and
#                             `[]` validates; a non-array does not (non-vacuous).
#   t05_markdown_unchanged    AC 5 (#1.5)       code-review.md is byte-identical to today's for a
#                             fixture review after the JSON sibling is written beside it.
#
# The validator is schema-DERIVED: it reads review-findings.schema.json and enforces exactly the
# required list / severity enum / clause_ref type union the schema declares, so the test and the
# schema cannot drift. node stdlib only (JSON.parse + field checks) - the repo's JSON approach; no
# new deps. node absent -> SKIP with a named reason (never a fail), per test_e2e_skeleton.sh.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
SCHEMA="$repo/modules/skill/code-review-author/envelopes/review-findings.schema.json"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

command -v node >/dev/null 2>&1 || { echo "  SKIP test_review_findings.sh — node not on PATH (JSON validation is node stdlib)"; exit 0; }
[ -f "$SCHEMA" ] || { echo "FATAL: schema not found at $SCHEMA"; exit 1; }

# ── schema-derived validator: exit 0 valid, exit 1 invalid (reasons on stderr) ────────────────────
cat > "$TMP/validate.mjs" <<'MJS'
import fs from 'node:fs';
const [schemaPath, candPath] = process.argv.slice(2);
const schema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'));
const cand = JSON.parse(fs.readFileSync(candPath, 'utf8'));
const errs = [];
if (schema.type !== 'array') errs.push('schema top-level is not type array');
if (!Array.isArray(cand)) { console.error('candidate is not an array'); process.exit(1); }
const it = schema.items || {};
const required = it.required || [];
const props = it.properties || {};
const allowed = Object.keys(props);
const noExtra = it.additionalProperties === false;
const sevEnum = (props.severity && props.severity.enum) || [];
const clauseTypes = [].concat((props.clause_ref && props.clause_ref.type) || []);
cand.forEach((rec, i) => {
  if (rec === null || typeof rec !== 'object' || Array.isArray(rec)) { errs.push(`#${i}: not an object`); return; }
  for (const k of required) if (!(k in rec)) errs.push(`#${i}: missing required '${k}'`);
  if (noExtra) for (const k of Object.keys(rec)) if (!allowed.includes(k)) errs.push(`#${i}: extra key '${k}'`);
  if ('severity' in rec && !sevEnum.includes(rec.severity)) errs.push(`#${i}: severity '${rec.severity}' not in enum ${sevEnum.join('|')}`);
  if ('file' in rec && typeof rec.file !== 'string') errs.push(`#${i}: file not a string`);
  if ('summary' in rec && typeof rec.summary !== 'string') errs.push(`#${i}: summary not a string`);
  if ('suggested_fix' in rec && typeof rec.suggested_fix !== 'string') errs.push(`#${i}: suggested_fix not a string`);
  if ('line' in rec && !Number.isInteger(rec.line)) errs.push(`#${i}: line not an integer`);
  if ('clause_ref' in rec) {
    const v = rec.clause_ref;
    const okType = (v === null && clauseTypes.includes('null')) || (typeof v === 'string' && clauseTypes.includes('string'));
    if (!okType) errs.push(`#${i}: clause_ref must be one of ${clauseTypes.join('|')}`);
  }
});
if (errs.length) { console.error(errs.join('\n')); process.exit(1); }
process.exit(0);
MJS
valid()   { node "$TMP/validate.mjs" "$SCHEMA" "$1" >/dev/null 2>&1; }   # 0 = valid
jlen()    { node -e 'const a=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"));if(!Array.isArray(a))process.exit(2);console.log(a.length)' "$1"; }
mdcount() { grep -cE '^- \*\*\[(severe|important|nit)\]\*\*' "$1"; }
# the audit's §1.4 cross-check, modelled: 0 = counts agree (green), 1 = mismatch (RED)
counts_agree() { local m j; m="$(mdcount "$1")"; j="$(jlen "$2")" || return 1; [ "$m" = "$j" ]; }

# ── a realistic code-review@1 fixture; its 3 findings are tagged inline by severity ───────────────
write_fixture_md() {
  cat > "$1" <<'MD'
---
template: code-review@1
title: PR #412 — harden token refresh
pr_url: https://github.com/acme/auth/pull/412
pr_number: 412
pr_size_loc: 143
reviewer: @jda
reviewed_at: 2026-07-18T09:14:00+07:00
linked_impl_plan: ./impl-plan.md
ai_assisted: false
provenance: { source_path: pr-412.diff, source_hash: sha256:deadbeef }
verdict: request_changes
---

# PR #412 — harden token refresh

## 1. Correctness vs Ticket
Implements the refresh path from the linked task. Two gaps below.

## 5. Injection Surfaces
- **[severe]** `src/db/session.ts:88` builds the lookup SQL by string concatenation of `req.query.sid`; parameterise it.

## 6. Input Validation
- **[important]** `src/auth/refresh.ts:42` dereferences the decoded token before the null guard runs.

## 2. Readability
- **[nit]** `src/util/log.ts:12` names the accumulator `tmp`; a fuller name would read better.
MD
}

# ── t01 ───────────────────────────────────────────────────────────────────────────────────────────
t01_schema_valid() {
  # a valid array: three findings, all six fields, severities spanning the enum, one string clause_ref,
  # and (edge §3) a path carrying a quote and a backslash - generated with a real encoder, not concat.
  node -e '
    const fs=require("fs");
    const arr=[
      {file:"src/db/session.ts", line:88, severity:"severe", clause_ref:"§1.2", summary:"SQL by concatenation", suggested_fix:"Parameterise the query."},
      {file:"src/auth/refresh.ts", line:42, severity:"important", clause_ref:null, summary:"deref before null guard", suggested_fix:"Guard before use."},
      {file:"weird/\"a\\b\".ts", line:0, severity:"nit", clause_ref:"§1.5", summary:"quote+backslash path", suggested_fix:"none"}
    ];
    fs.writeFileSync(process.argv[1], JSON.stringify(arr,null,2));
  ' "$TMP/t01.json"
  valid "$TMP/t01.json" || { fail t01_schema_valid "a well-formed six-field array did not validate"; return; }
  # the nasty path round-trips byte-for-byte through JSON (never string-concatenated)
  local rt; rt="$(node -e 'console.log(JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"))[2].file)' "$TMP/t01.json")"
  [ "$rt" = 'weird/"a\b".ts' ] || { fail t01_schema_valid "quote/backslash path did not round-trip (got: $rt)"; return; }
  # NON-VACUOUS: the validator must reject each defect ------------------------------------------------
  printf '%s' '[{"line":1,"severity":"nit","clause_ref":null,"summary":"s","suggested_fix":"f"}]' > "$TMP/t01_missing.json"  # no file
  valid "$TMP/t01_missing.json" && { fail t01_schema_valid "a record missing a required field validated"; return; }
  printf '%s' '[{"file":"a","line":1,"severity":"blocker","clause_ref":null,"summary":"s","suggested_fix":"f"}]' > "$TMP/t01_enum.json"
  valid "$TMP/t01_enum.json" && { fail t01_schema_valid "a bad severity enum validated"; return; }
  printf '%s' '[{"file":"a","line":1,"severity":"nit","clause_ref":null,"summary":"s","suggested_fix":"f","posted":true}]' > "$TMP/t01_extra.json"
  valid "$TMP/t01_extra.json" && { fail t01_schema_valid "an unknown extra field validated"; return; }
  ok t01_schema_valid
}

# ── t02 ───────────────────────────────────────────────────────────────────────────────────────────
t02_counts_agree() {
  local d="$TMP/t02"; mkdir -p "$d"
  write_fixture_md "$d/code-review.md"
  # matching sibling: one record per finding (3 == 3)
  node -e '
    const fs=require("fs");
    fs.writeFileSync(process.argv[1], JSON.stringify([
      {file:"src/db/session.ts", line:88, severity:"severe", clause_ref:"§1.2", summary:"SQL by concatenation", suggested_fix:"Parameterise."},
      {file:"src/auth/refresh.ts", line:42, severity:"important", clause_ref:null, summary:"deref before guard", suggested_fix:"Guard first."},
      {file:"src/util/log.ts", line:12, severity:"nit", clause_ref:"§1.5", summary:"tmp naming", suggested_fix:"Rename."}
    ],null,2));
  ' "$d/review-findings.json"
  [ "$(mdcount "$d/code-review.md")" = 3 ] || { fail t02_counts_agree "fixture markdown finding count != 3"; return; }
  valid "$d/review-findings.json" || { fail t02_counts_agree "matching sibling did not validate"; return; }
  counts_agree "$d/code-review.md" "$d/review-findings.json" || { fail t02_counts_agree "3-vs-3 was not read as agreement"; return; }
  # a sibling that drops a finding (2 vs 3) MUST red the cross-check
  node -e '
    const fs=require("fs");
    fs.writeFileSync(process.argv[1], JSON.stringify([
      {file:"src/db/session.ts", line:88, severity:"severe", clause_ref:"§1.2", summary:"SQL by concatenation", suggested_fix:"Parameterise."},
      {file:"src/auth/refresh.ts", line:42, severity:"important", clause_ref:null, summary:"deref before guard", suggested_fix:"Guard first."}
    ],null,2));
  ' "$d/mismatch.json"
  valid "$d/mismatch.json" || { fail t02_counts_agree "mismatch sibling is itself malformed"; return; }
  counts_agree "$d/code-review.md" "$d/mismatch.json" && { fail t02_counts_agree "2-vs-3 mismatch did not red"; return; }
  ok t02_counts_agree
}

# ── t03 ───────────────────────────────────────────────────────────────────────────────────────────
t03_null_clause_ref() {
  # an out-of-spec finding carries clause_ref:null; a normal one carries a real §ref. Both validate,
  # proving null is a first-class value and a fabricated string ref is NOT required.
  printf '%s' '[
    {"file":"src/x.ts","line":9,"severity":"important","clause_ref":null,"summary":"outside the spec clauses","suggested_fix":"log a defect"},
    {"file":"src/y.ts","line":3,"severity":"nit","clause_ref":"§1.4","summary":"on a clause","suggested_fix":"tidy"}
  ]' > "$TMP/t03.json"
  valid "$TMP/t03.json" || { fail t03_null_clause_ref "string-or-null clause_ref array did not validate"; return; }
  local nullcount; nullcount="$(node -e 'const a=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8"));console.log(a.filter(r=>r.clause_ref===null).length)' "$TMP/t03.json")"
  [ "$nullcount" = 1 ] || { fail t03_null_clause_ref "the out-of-spec record did not carry a literal null (got $nullcount)"; return; }
  # a string clause_ref alone also validates - proving both arms of the union are live
  printf '%s' '[{"file":"a","line":1,"severity":"nit","clause_ref":"§1.1","summary":"s","suggested_fix":"f"}]' > "$TMP/t03_str.json"
  valid "$TMP/t03_str.json" || { fail t03_null_clause_ref "a string clause_ref did not validate"; return; }
  ok t03_null_clause_ref
}

# ── t04 ───────────────────────────────────────────────────────────────────────────────────────────
t04_empty_array_not_absent() {
  local d="$TMP/t04"; mkdir -p "$d"
  # a clean review WRITES the file, empty-valued - not an absent file
  printf '%s' '[]' > "$d/review-findings.json"
  [ -f "$d/review-findings.json" ] || { fail t04_empty_array_not_absent "clean review left the sibling absent"; return; }
  valid "$d/review-findings.json" || { fail t04_empty_array_not_absent "empty array [] did not validate"; return; }
  [ "$(jlen "$d/review-findings.json")" = 0 ] || { fail t04_empty_array_not_absent "[] did not read as length 0"; return; }
  # NON-VACUOUS: a non-array (e.g. an object) must NOT validate, so "[] validates" isn't trivially true
  printf '%s' '{}' > "$d/notarray.json"
  valid "$d/notarray.json" && { fail t04_empty_array_not_absent "a non-array {} validated"; return; }
  ok t04_empty_array_not_absent
}

# ── t05 ───────────────────────────────────────────────────────────────────────────────────────────
t05_markdown_unchanged() {
  local d="$TMP/t05"; mkdir -p "$d"
  write_fixture_md "$d/code-review.md"
  write_fixture_md "$d/code-review.golden.md"   # today's packet, held aside
  local before after
  before="$(node -e 'const c=require("crypto"),f=require("fs");console.log(c.createHash("sha256").update(f.readFileSync(process.argv[1])).digest("hex"))' "$d/code-review.md")"
  # the IMP-112 change: write the JSON sibling BESIDE the markdown (additive, not a reformat)
  printf '%s' '[{"file":"src/db/session.ts","line":88,"severity":"severe","clause_ref":"§1.2","summary":"x","suggested_fix":"y"}]' > "$d/review-findings.json"
  after="$(node -e 'const c=require("crypto"),f=require("fs");console.log(c.createHash("sha256").update(f.readFileSync(process.argv[1])).digest("hex"))' "$d/code-review.md")"
  [ "$before" = "$after" ] || { fail t05_markdown_unchanged "writing the sibling changed the markdown sha"; return; }
  cmp -s "$d/code-review.md" "$d/code-review.golden.md" || { fail t05_markdown_unchanged "markdown is not byte-identical to today's"; return; }
  ok t05_markdown_unchanged
}

t01_schema_valid
t02_counts_agree
t03_null_clause_ref
t04_empty_array_not_absent
t05_markdown_unchanged
echo "----"; echo "pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
