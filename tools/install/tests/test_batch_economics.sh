#!/usr/bin/env bash
# test_batch_economics.sh - TASK-IMP-114. The per-batch economics row: one arm per AC it cites.
#
# What these arms are careful about: a row that TRANSCRIBES its numbers out of the ledger would
# pass a weaker test just as happily as one that DERIVES them (§1.3), and a renderer that never
# printed tokens at all would satisfy "tokens omitted" (§1.2) vacuously. So t01 mutates the
# members' frontmatter and leaves the ledger byte-identical, and t02 proves the same ledger with
# a tokens key DOES render one. Each assertion fails when its clause is violated, or it is
# decoration.
#
# Node stdlib only, no model, no network.
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; root="$(cd "$here/../../.." && pwd)"
R="$root/tools/docs-site/render-status-hub.mjs"
W="$root/modules/cuo/chief-technology-officer/workflows/ship-tasks.md"
PASS=0; FAIL=0
ok(){ PASS=$((PASS+1)); echo "  ok   $1"; }
no(){ FAIL=$((FAIL+1)); echo "  FAIL $1: ${2:-}"; }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# a scratch repo the renderer accepts: two member tasks, templates, CHANGELOG, VERSION.
# member <dir> <id> <module> <status> <routed_back_count>
member(){
  local d="$1" id="$2" mod="$3" st="$4" rb="$5"
  mkdir -p "$d/docs/tasks/$mod/$id-x"
  printf -- '---\nid: %s\ntitle: %s\nmodule: %s\npriority: p1\nstatus: %s\ntype: improvement\nrouted_back_count: %s\n---\n## 1. Description\n\nBody.\n' \
    "$id" "$id" "$mod" "$st" "$rb" > "$d/docs/tasks/$mod/$id-x/spec.md"
}
mkrepo(){
  local d="$1"; mkdir -p "$d/modules/templates/html" "$d/modules/templates/cds" "$d/docs/batches"
  cp "$root/modules/templates/html/status-hub.html" "$root/modules/templates/html/status-app.js" "$d/modules/templates/html/"
  cp "$root/modules/templates/cds/tokens.css" "$root/modules/templates/cds/status.css" "$d/modules/templates/cds/"
  printf '# CL\n\n## [2.0.0] - 2026-07-01\n\nAdded\n- a thing\n' > "$d/CHANGELOG.md"
  echo "2.0.0" > "$d/VERSION"
}
# the rendered page with every <script> (payload included) stripped: the clause says the row
# RENDERS, and a string inside a JSON island nothing reads is not a rendered row (§1.7's lesson).
visible(){
  node -e 'const fs=require("fs");let h=fs.readFileSync(process.argv[1],"utf8");
    h=h.replace(/<script[\s\S]*?<\/script>/g,"");process.stdout.write(h)' "$1"
}
# the <tr> of a named batch, out of the visible markup. Extracted with a lazy match in node:
# POSIX ERE has no `?` quantifier, so `grep -oE '<tr>...</tr>'` runs greedily to the LAST </tr>
# on the page and every equality assertion below would compare against half the document.
row_of(){
  node -e 'const fs=require("fs");let h=fs.readFileSync(process.argv[1],"utf8");
    h=h.replace(/<script[\s\S]*?<\/script>/g,"");
    const m=h.match(new RegExp("<tr><td class=\"code\">"+process.argv[2].replace(/[/\-]/g,"\\$&")+"</td>[\\s\\S]*?</tr>"));
    process.stdout.write(m?m[0]:"")' "$1" "$2"
}

# --- AC 1 (traces_to #1.1, #1.3) -------------------------------------------------------
# "a fixture batch yields a row with all non-optional fields derived from its artefacts"
t01_row_derived(){
  local d="$TMP/t01"; mkrepo "$d"
  member "$d" TASK-AA-001 aa done 2
  member "$d" TASK-BB-001 bb implementing 1
  printf -- '---\nbatch: batch/4-telemetry\nmembers: [TASK-AA-001, TASK-BB-001]\nstarted: 2026-07-17T14:00:00Z\nended: 2026-07-17T15:30:00Z\nroute_backs: 1\ngate_reasks: 3\n---\n# batch 4\n' \
    > "$d/docs/batches/batch-4-telemetry.md"
  node "$R" "$d" "$d/out" >/dev/null 2>&1 || { no t01_row_derived "render failed"; return; }
  local h="$d/out/reference/status.html" r
  r="$(row_of "$h" "batch/4-telemetry")"
  [ -n "$r" ] || { no t01_row_derived "no rendered row for batch/4-telemetry"; return; }

  # §1.1 - every non-optional field is on the row, with the value the artefacts imply:
  #   tasks 2 (members), shipped 1 (only AA is done), route-backs 1 (this batch's own),
  #   re-asks 3, wall 1h 30m (the distance between the two committed instants)
  [ "$r" = '<tr><td class="code">batch/4-telemetry</td><td>2</td><td>1</td><td>1</td><td>3</td><td>1h 30m</td></tr>' ] \
    || { no t01_row_derived "row is not the derived one: $r"; return; }

  # §1.3 - `shipped` is DERIVED from the members' frontmatter, not transcribed. Move only the
  # frontmatter; the ledger stays byte-identical. A row that copied the number out of the ledger
  # cannot notice this, and that is the point: frontmatter is the record of truth, and two homes
  # for one fact is one home too many.
  #
  # The same edit ALSO bumps BB's lifetime routed_back_count 1 -> 4, and the row's route-backs
  # MUST NOT move: that counter spans every batch the task ever sat in, while the row wants what
  # happened HERE, "which is where the cost fell". Summing the lifetime counter would charge this
  # batch for a route-back that happened in a later one - retroactively, months after it closed.
  local before; before="$(cksum < "$d/docs/batches/batch-4-telemetry.md")"
  sed -i.bak 's/^status: implementing$/status: done/;s/^routed_back_count: 1$/routed_back_count: 4/' \
    "$d/docs/tasks/bb/TASK-BB-001-x/spec.md"
  [ "$before" = "$(cksum < "$d/docs/batches/batch-4-telemetry.md")" ] || { no t01_row_derived "test mutated the ledger"; return; }
  node "$R" "$d" "$d/out2" >/dev/null 2>&1 || { no t01_row_derived "re-render failed"; return; }
  r="$(row_of "$d/out2/reference/status.html" "batch/4-telemetry")"
  [ "$r" = '<tr><td class="code">batch/4-telemetry</td><td>2</td><td>2</td><td>1</td><td>3</td><td>1h 30m</td></tr>' ] \
    || { no t01_row_derived "shipped did not follow the members' frontmatter, or route-backs drifted with a lifetime counter: $r"; return; }

  # §1.3 second half - no new writer on the phase path. The ledger is a batch-close transcription;
  # steps 1-31 are untouched and no skill_chain entry produces it.
  grep -q '^## 11d\. Batch economics' "$W"                        || { no t01_row_derived "ship-tasks has no §11d"; return; }
  grep -q 'At the batch close, ship-tasks MUST write .docs/batches/' "$W" || { no t01_row_derived "the ledger write is not a MUST at the batch close"; return; }
  grep -q 'This is not a new writer on the phase path' "$W"       || { no t01_row_derived "§1.3's phase-path promise is not on the record"; return; }
  grep -q 'ledger records ONLY what nothing else knows' "$W"      || { no t01_row_derived "the derive-don't-copy rule is missing"; return; }
  grep -q 'NOT a sum of .routed_back_count' "$W"                  || { no t01_row_derived "the lifetime-vs-this-batch distinction is not on the record"; return; }
  grep -iq 'batch.economics' <<<"$(sed -n '/^skill_chain:/,/^---$/p' "$W")" \
    && { no t01_row_derived "a batch-economics step was added to skill_chain - §1.3 forbids a new phase-path writer"; return; }
  ok t01_row_derived
}

# --- AC 2 (traces_to #1.2) -------------------------------------------------------------
# "a harness reporting no tokens yields a row with tokens omitted and all else present"
t02_tokens_optional(){
  local d="$TMP/t02"; mkrepo "$d"
  member "$d" TASK-AA-001 aa done 0
  # a harness that reports nothing: the key is simply absent
  printf -- '---\nbatch: batch/5-quiet\nmembers: [TASK-AA-001]\nstarted: 2026-07-17T09:00:00Z\nended: 2026-07-17T09:20:00Z\nroute_backs: 0\ngate_reasks: 1\n---\n# batch 5\n' \
    > "$d/docs/batches/batch-5-quiet.md"
  node "$R" "$d" "$d/out" >/dev/null 2>&1 || { no t02_tokens_optional "render failed"; return; }
  local v; v="$(visible "$d/out/reference/status.html")"

  # omitted, not zeroed: no tokens column exists at all when nothing reported one.
  grep -q '<th>tokens</th>' <<<"$v" && { no t02_tokens_optional "a tokens column rendered for a harness that reported none"; return; }
  # ...and the row DEGRADES rather than vanishing - every other field is present and true.
  local r; r="$(row_of "$d/out/reference/status.html" "batch/5-quiet")"
  [ "$r" = '<tr><td class="code">batch/5-quiet</td><td>1</td><td>1</td><td>0</td><td>1</td><td>20m</td></tr>' ] \
    || { no t02_tokens_optional "row missing or wrong without tokens: $r"; return; }

  # The other half of the clause, without which the arm is vacuous: a renderer that simply never
  # printed tokens would pass everything above. The SAME ledger plus a committed figure must
  # render it - so the omission is a property of the artefact, not of the code.
  local e="$TMP/t02b"; mkrepo "$e"; member "$e" TASK-AA-001 aa done 0
  printf -- '---\nbatch: batch/5-quiet\nmembers: [TASK-AA-001]\nstarted: 2026-07-17T09:00:00Z\nended: 2026-07-17T09:20:00Z\nroute_backs: 0\ngate_reasks: 1\ntokens: 412300\n---\n# batch 5\n' \
    > "$e/docs/batches/batch-5-quiet.md"
  node "$R" "$e" "$e/out" >/dev/null 2>&1 || { no t02_tokens_optional "render with tokens failed"; return; }
  local v2; v2="$(visible "$e/out/reference/status.html")"
  grep -q '<th>tokens</th>' <<<"$v2" || { no t02_tokens_optional "a reported token count did not render"; return; }
  grep -q '<td>412300</td>' <<<"$v2" || { no t02_tokens_optional "the reported figure is not on the row"; return; }

  # and a mixed corpus - one harness reporting, one not - names the silence rather than zeroing it
  member "$e" TASK-BB-001 bb done 0
  printf -- '---\nbatch: batch/6-mixed\nmembers: [TASK-BB-001]\nstarted: 2026-07-17T10:00:00Z\nended: 2026-07-17T10:05:00Z\nroute_backs: 0\ngate_reasks: 0\n---\n# batch 6\n' \
    > "$e/docs/batches/batch-6-mixed.md"
  node "$R" "$e" "$e/out3" >/dev/null 2>&1 || { no t02_tokens_optional "mixed render failed"; return; }
  local r6; r6="$(row_of "$e/out3/reference/status.html" "batch/6-mixed")"
  grep -q '<td>not reported</td>' <<<"$r6" || { no t02_tokens_optional "unreported tokens did not read as unreported: $r6"; return; }
  grep -qE '<td>0</td></tr>$' <<<"$r6" && { no t02_tokens_optional "unreported tokens rendered as 0 - §1.2 says omitted, not zeroed"; return; }

  # §1.2 generalised, and §11d says so in as many words: a counter the ledger never recorded reads
  # `unknown`, never `0`, for exactly the reason tokens do - a zero asserts a fact nobody measured.
  # The claim is in the doctrine AND on the rendered page, so it needs an arm; a normative sentence
  # with no test is the "deterministic by construction" mistake in different clothes.
  local f="$TMP/t02c"; mkrepo "$f"; member "$f" TASK-AA-001 aa done 0
  printf -- '---\nbatch: batch/7-silent\nmembers: [TASK-AA-001]\nstarted: 2026-07-17T11:00:00Z\nended: 2026-07-17T11:10:00Z\n---\n# nothing counted\n' \
    > "$f/docs/batches/batch-7-silent.md"
  node "$R" "$f" "$f/out" >/dev/null 2>&1 || { no t02_tokens_optional "render of an uncounted batch failed"; return; }
  local r7; r7="$(row_of "$f/out/reference/status.html" "batch/7-silent")"
  [ "$r7" = '<tr><td class="code">batch/7-silent</td><td>1</td><td>1</td><td>unknown</td><td>unknown</td><td>10m</td></tr>' ] \
    || { no t02_tokens_optional "an unrecorded route-back / re-ask count did not read unknown (a 0 would assert a fact nobody measured): $r7"; return; }

  # the rationale is on the record in the doctrine, not just in this file
  grep -q 'Tokens are OPTIONAL and MUST NOT be zeroed' "$W" || { no t02_tokens_optional "ship-tasks §11d does not forbid a zeroed token count"; return; }
  grep -q 'reads .unknown., never .0.' "$W"                 || { no t02_tokens_optional "§11d does not generalise the never-zero rule to the other counters"; return; }
  ok t02_tokens_optional
}

t01_row_derived
t02_tokens_optional
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
