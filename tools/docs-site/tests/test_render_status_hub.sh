#!/usr/bin/env bash
# test_render_status_hub.sh - TASK-DOCS-006 §5 (t01-t06) carried forward to the single-page
# hub of TASK-IMP-074: Roadmap | Backlog | Changelog are no longer three tabs but three lenses
# (board | table | releases) over one corpus, plus a task drawer that carries the full spec.
# t07-t09 cover what the merge added: changelog->task binding, lazy spec chunks, no-JS truth.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
R="$repo/tools/docs-site/render-status-hub.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

mkfix() {
  d="$1"
  mkdir -p "$d/docs/tasks/aa/TASK-AA-001-first" "$d/docs/tasks/bb/TASK-BB-001-third" \
           "$d/.git/refs/heads" "$d/modules/templates/html" "$d/modules/templates/cds"
  cp "$repo/modules/templates/html/status-hub.html" "$repo/modules/templates/html/status-app.js" "$d/modules/templates/html/"
  cp "$repo/modules/templates/cds/tokens.css" "$repo/modules/templates/cds/status.css" "$d/modules/templates/cds/"
  echo "ref: refs/heads/main" > "$d/.git/HEAD"; echo "abcdef1234567890" > "$d/.git/refs/heads/main"
  printf -- '---\nid: TASK-AA-001\ntitle: First\nmodule: aa\npriority: MUST\nstatus: done\nclass: product\nphase: P0\nowner: Ada\neffort_hours: 3\nshipped: 2026-07-01\ndepends_on: []\nblocks: [TASK-BB-001]\nsubtasks:\n  - "wire it"\n  - "prove it"\n---\n## §1 — Description\n\nFirst task body paragraph.\n\n## §4 — Acceptance criteria\n\n- one\n' > "$d/docs/tasks/aa/TASK-AA-001-first/spec.md"
  printf -- '---\nid: TASK-BB-001\ntitle: Third\nmodule: bb\npriority: SHOULD\nstatus: draft\nclass: improvement\nphase: P1\ndepends_on: [TASK-AA-001]\n---\n## §1 — Description\n\nThird task body paragraph.\n' > "$d/docs/tasks/bb/TASK-BB-001-third/spec.md"
  printf '# CL\n\n## [2.0.0] - 2026-07-01\n\nAdded\n- TASK-AA-001 first thing landed\n' > "$d/CHANGELOG.md"
  echo "2.0.0" > "$d/VERSION"
}

t01_deck_true() {                                                      # AC 1 - the deck is generated truth
  mkfix "$TMP/a"
  node "$R" "$TMP/a" "$TMP/a/out" >/dev/null 2>&1
  h="$TMP/a/out/reference/status.html"
  grep -q 'Overall progress · 2 tasks · 2 modules' "$h" \
    && grep -q 'VERSION <span class="code">2.0.0</span>' "$h" \
    && grep -Eq 'built from <span class="code">fp-[0-9a-f]{12}</span>' "$h" \
    && grep -Eq 'data-bucket="done"[^>]*><b>1</b>' "$h" \
    && grep -q 'seg-done" style="width:50.0%"' "$h" \
    && ok t01 || fail t01 "deck counts / stamp wrong"
}
t02_one_page_three_lenses() {                                          # AC 2 - the merge itself
  h="$TMP/a/out/reference/status.html"
  ! grep -q 'role="tabpanel"' "$h" \
    && [ "$(grep -co 'class="ln" role="tab"' "$h")" -eq 3 ] \
    && grep -q 'data-lens="board"' "$h" && grep -q 'data-lens="table"' "$h" && grep -q 'data-lens="timeline"' "$h" \
    && grep -q 'roadmap: "board", backlog: "table", changelog: "timeline"' "$h" \
    && ok t02 || fail t02 "lenses / legacy hash mapping"
}
t03_facets_and_search() {                                              # AC 3 - one filter set, every lens
  h="$TMP/a/out/reference/status.html"
  for id in f-m f-s f-p f-c f-ph f-g; do grep -q "id=\"$id\"" "$h" || { fail t03 "facet $id missing"; return; }; done
  grep -q 'id="q"' "$h" && grep -q '"p":"MUST"' "$h" && grep -q '"c":"improvement"' "$h" \
    && grep -q '"ph":"P0"' "$h" && grep -q '"st":\["wire it","prove it"\]' "$h" \
    && ok t03 || fail t03 "search box or corpus fields missing"
}
t04_supersession() {                                                   # AC 4 - bookmarks survive
  grep -q 'url=status.html#roadmap' "$TMP/a/out/reference/roadmap.html" \
    && bash "$repo/tools/docs-site/tests/test_render_roadmap.sh" >/dev/null 2>&1 \
    && ok t04 || fail t04 "stub or legacy suite red"
}
t05_deterministic() {                                                  # AC 5 - same input, same bytes
  node "$R" "$TMP/a" "$TMP/a/out2" >/dev/null 2>&1
  cmp -s "$TMP/a/out/reference/status.html" "$TMP/a/out2/reference/status.html" \
    && cmp -s "$TMP/a/out/reference/data/task/TASK-AA-001.js" "$TMP/a/out2/reference/data/task/TASK-AA-001.js" \
    && ok t05 || fail t05 "nondeterministic"
}
t06_task_page_links() {                                                  # AC 6 - the drawer links the task page
  mkfix "$TMP/d"
  mkdir -p "$TMP/d/out/tasks/aa/TASK-AA-001-first"; touch "$TMP/d/out/tasks/aa/TASK-AA-001-first/index.html"
  node "$R" "$TMP/d" "$TMP/d/out" >/dev/null 2>&1
  grep -q '"pg":"../tasks/aa/TASK-AA-001-first/index.html"' "$TMP/d/out/reference/status.html" \
    && ok t06 || fail t06 "task page link absent from the corpus"
}
t07_changelog_binds_tasks() {                                            # the changelog references tasks, not prose
  h="$TMP/a/out/reference/status.html"
  grep -q '"cited":\["TASK-AA-001"\]' "$h" \
    && grep -q 'data-task=\\"TASK-AA-001\\"' "$h" \
    && grep -q '"bound":\["TASK-AA-001"\]' "$h" \
    && ok t07 || fail t07 "release -> task binding missing"
}
t08_spec_chunks() {                                                    # full spec, lazily
  c="$TMP/a/out/reference/data/task/TASK-AA-001.js"
  [ -f "$c" ] && grep -q 'window.CS_SPEC\["TASK-AA-001"\]' "$c" \
    && grep -q 'First task body paragraph' "$c" \
    && grep -q '"sp":1' "$TMP/a/out/reference/status.html" || { fail t08 "chunk missing"; return; }
  CYBEROS_STATUS_SPECS=0 node "$R" "$TMP/a" "$TMP/a/out3" >/dev/null 2>&1
  [ ! -d "$TMP/a/out3/reference/data" ] && ! grep -q '"sp":1' "$TMP/a/out3/reference/status.html" \
    && ok t08 || fail t08 "CYBEROS_STATUS_SPECS=0 still emitted chunks"
}
t09_nojs_and_honest_failures() {                                       # degrade, and fail loudly
  h="$TMP/a/out/reference/status.html"
  grep -q '<noscript>' "$h" && grep -q 'TASK-BB-001' "$h" || { fail t09 "no-JS fallback missing"; return; }
  mkfix "$TMP/b"
  mkdir -p "$TMP/b/docs/tasks/aa/TASK-AA-009-broken"
  printf -- '---\nid: TASK-AA-009\nno closing fence\n' > "$TMP/b/docs/tasks/aa/TASK-AA-009-broken/spec.md"
  out="$(node "$R" "$TMP/b" "$TMP/b/out" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] && grep -q "TASK-AA-009" <<<"$out" && ok t09 || fail t09 "broken frontmatter did not fail loudly (rc=$rc)"
}
t10_token_clean() {                                                    # colour literals stay in the token layer
  h="$TMP/a/out/reference/status.html"
  hexes="$(sed -e '/:root {/,/^}/d' -e '/^\[data-theme="dark"\] {/,/^}/d' "$h" \
    | grep -oE '#[0-9a-fA-F]{3,6}\b' | wc -l | tr -d ' ')"
  [ "$hexes" -eq 0 ] && grep -q -- '--cs-color-brand-umber' "$h" \
    && ok t10 || fail t10 "$hexes raw hex outside the token blocks"
}


t11_draft_staleness_report() {                                         # TASK-IMP-108 AC 6 - §1.7
  # §1.7: "The status page MUST RENDER a staleness report."
  #
  # This arm previously asserted `grep -q '"draft_staleness"'` - that the string appeared in the
  # HTML. It did, inside a JSON blob no code read: status-app.js has zero references to the key.
  # So the clause said RENDER, the test said PRESENT-IN-PAYLOAD, and 108 shipped `done` with §1.7
  # unsatisfied and this arm green. A test that cites a clause and asserts something weaker than
  # the clause is not evidence. (External review 2026-07-17.)
  #
  # It now asserts the RENDERED report: real markup, outside the payload, readable with JS off.
  local h="$TMP/a/out/reference/status.html"
  [ -s "$h" ] || { fail t11 "no page rendered at $h"; return; }

  # 1. the report is VISIBLE MARKUP, not payload. Strip the data island first, then look.
  local visible; visible="$(node -e '
    const fs=require("fs"); let h=fs.readFileSync(process.argv[1],"utf8");
    h=h.replace(/<script[^>]*id="cs-data"[\s\S]*?<\/script>/g,"");   // the JSON island
    h=h.replace(/<script[\s\S]*?<\/script>/g,"");                    // and every other script
    process.stdout.write(h);' "$h")"
  grep -q "Drafts awaiting triage" <<<"$visible" || { fail t11 "no rendered staleness section outside the payload - §1.7 says RENDER"; return; }

  # 2. grouped BY REASON with an AGE, as §1.7 words it - a heading alone is not a report.
  grep -qE "<th>reason</th>" <<<"$visible" || { fail t11 "rendered report has no reason column"; return; }
  grep -qE "<th>oldest</th>"  <<<"$visible" || { fail t11 "rendered report has no age - §1.7 requires age"; return; }
  grep -qE "<td>[a-z_]+</td><td>[0-9]+</td>" <<<"$visible" || { fail t11 "no reason/count row rendered"; return; }

  # 3. absent draft_reason MUST render as `unknown`. The page may not invent a reason it was not
  #    told - inventing one for the untriaged drafts is the `# UNREVIEWED` mistake in new clothes.
  grep -q "<td>unknown</td>" <<<"$visible" || { fail t11 "absent draft_reason did not render as unknown"; return; }

  # 4. §1.7: it MUST NOT change any status. The report is derived; assert it stayed a report.
  grep -q "A report, not an action" <<<"$visible" || { fail t11 "the report does not state that it changes nothing"; return; }

  # 5. the payload stays well-formed too - the client may render it later, and the sums must hold.
  node -e '
    const fs=require("fs"), h=fs.readFileSync(process.argv[1],"utf8");
    const m=h.match(/"draft_staleness":(\{.*?\]\})/); if(!m){console.error("no payload");process.exit(1);}
    const d=JSON.parse(m[1]);
    if(typeof d.total!=="number"){console.error("no total");process.exit(1);}
    if(!Array.isArray(d.by_reason)){console.error("no by_reason");process.exit(1);}
    for(const r of d.by_reason) if(!("reason" in r)||!("count" in r)||!("oldest" in r)){console.error("row missing a field");process.exit(1);}
    const sum=d.by_reason.reduce((a,r)=>a+r.count,0);
    if(sum!==d.total){console.error(`by_reason sums to ${sum}, total says ${d.total}`);process.exit(1);}
  ' "$h" || { fail t11 "draft_staleness payload malformed"; return; }

  ok t11_draft_staleness_report
}


# --- TASK-IMP-114: the batch economics row ---------------------------------------------
# NOTE ON THE NAMES. TASK-IMP-114's AC 3 / AC 5 cite `t09_economics_row` and
# `t10_economics_row_deterministic`, and the citations are what TRACE-004 greps, so the names
# stand as written. The t09/t10 prefixes were already taken by t09_nojs_and_honest_failures and
# t10_token_clean when the spec was authored - the function names are distinct, the numbering is
# not. Reported as a spec defect rather than silently renumbered: a renamed arm is an AC that
# cites a test which does not exist.
#
# Its own fixture, not mkfix's: t01's counts and t05's cmp are asserted against $TMP/a, and a
# suite arm that perturbs a sibling arm's fixture is a flake waiting for a bad day.
mkbatchfix() {
  local d="$1"; mkfix "$d"; mkdir -p "$d/docs/batches"
  # AA is done (2 route-backs), BB is not (1) -> shipped 1 of 2, route-backs 3
  sed -i.bak 's/^shipped: 2026-07-01$/shipped: 2026-07-01\nrouted_back_count: 2/' "$d/docs/tasks/aa/TASK-AA-001-first/spec.md"
  sed -i.bak 's/^status: draft$/status: draft\nrouted_back_count: 1/' "$d/docs/tasks/bb/TASK-BB-001-third/spec.md"
  printf -- '---\nbatch: batch/1-closed\nmembers: [TASK-AA-001, TASK-BB-001]\nstarted: 2026-07-17T14:00:00Z\nended: 2026-07-17T15:30:00Z\nroute_backs: 3\ngate_reasks: 3\n---\n# closed batch\n' \
    > "$d/docs/batches/batch-1-closed.md"
  # the batch this run lost three of four agents from: started, never ended
  printf -- '---\nbatch: batch/2-cut\nmembers: [TASK-AA-001]\nstarted: 2026-07-17T16:00:00Z\nroute_backs: 0\ngate_reasks: 0\n---\n# cut mid-flight\n' \
    > "$d/docs/batches/batch-2-cut.md"
}
vis() {                                                                # the page minus every script
  node -e 'const fs=require("fs");let h=fs.readFileSync(process.argv[1],"utf8");
    h=h.replace(/<script[\s\S]*?<\/script>/g,"");process.stdout.write(h)' "$1"
}

t09_economics_row() {                                                  # TASK-IMP-114 AC 3 - §1.4
  # §1.4: "The row MUST render on the status page." RENDER, not "appear in the payload" - the
  # distinction §1.7 shipped `done` without, so the payload is stripped before anything is asserted.
  mkbatchfix "$TMP/e"
  node "$R" "$TMP/e" "$TMP/e/out" >/dev/null 2>&1 || { fail t09_economics_row "render failed"; return; }
  local v; v="$(vis "$TMP/e/out/reference/status.html")"
  grep -q "Batch economics" <<<"$v"      || { fail t09_economics_row "no rendered economics panel outside the payload - §1.4 says RENDER"; return; }
  # §1.1 - every non-optional field is a column, and the batch's own row carries the derived values
  for c in "<th>batch</th>" "<th>tasks</th>" "<th>shipped</th>" "<th>route-backs</th>" "<th>gate re-asks</th>" "<th>wall time</th>"; do
    grep -q "$c" <<<"$v" || { fail t09_economics_row "column $c missing - §1.1 requires it"; return; }
  done
  local r; r="$(node -e 'const fs=require("fs");let h=fs.readFileSync(process.argv[1],"utf8");
    h=h.replace(/<script[\s\S]*?<\/script>/g,"");
    const m=h.match(/<tr><td class="code">batch\/1-closed<\/td>[\s\S]*?<\/tr>/);
    process.stdout.write(m?m[0]:"")' "$TMP/e/out/reference/status.html")"
  [ "$r" = '<tr><td class="code">batch/1-closed</td><td>2</td><td>1</td><td>3</td><td>3</td><td>1h 30m</td></tr>' ] \
    || { fail t09_economics_row "the rendered row is not the derived one: $r"; return; }
  # §1.5 - it MUST NOT gate. Assert it stayed a measurement, on the page, where the reader is.
  grep -q "Measured, not enforced" <<<"$v" || { fail t09_economics_row "the panel does not state that it enforces nothing"; return; }
  ok t09_economics_row
}

t10_economics_row_deterministic() {                                    # TASK-IMP-114 AC 5 - §1.6
  # §1.6: "a re-render of an unchanged corpus stays byte-identical, economics row included."
  #
  # cmp alone would NOT catch a clock: two renders 40 ms apart with minute granularity agree.
  # So the arm pins the two things a wall clock would actually break - the OPEN batch reads
  # `incomplete` instead of a duration to now, and the closed one reads exactly the distance
  # between its two committed instants - and then diffs the bytes on top.
  local v; v="$(vis "$TMP/e/out/reference/status.html")"
  grep -q "<td>incomplete</td>" <<<"$v" \
    || { fail t10_economics_row_deterministic "a batch with no end did not read incomplete - §1.6 / the mid-flight edge case"; return; }
  local cut; cut="$(node -e 'const fs=require("fs");let h=fs.readFileSync(process.argv[1],"utf8");
    h=h.replace(/<script[\s\S]*?<\/script>/g,"");
    const m=h.match(/<tr><td class="code">batch\/2-cut<\/td>[\s\S]*?<\/tr>/);
    process.stdout.write(m?m[0]:"")' "$TMP/e/out/reference/status.html")"
  grep -qE '<td>[0-9]+(m|h [0-9]+m)</td>' <<<"$cut" \
    && { fail t10_economics_row_deterministic "an unfinished batch was given a duration: $cut"; return; }

  # byte-for-byte, twice, same corpus
  node "$R" "$TMP/e" "$TMP/e/out2" >/dev/null 2>&1
  cmp -s "$TMP/e/out/reference/status.html" "$TMP/e/out2/reference/status.html" \
    || { fail t10_economics_row_deterministic "re-render of an unchanged corpus differs"; return; }

  # and the fp- stamp must MOVE when a ledger moves: a stamp that ignores an input reports the
  # same fingerprint for two different pages, which is TASK-IMP-082's guarantee inverted.
  local fp1 fp2
  fp1="$(grep -oE 'built from <span class="code">fp-[0-9a-f]{12}' "$TMP/e/out/reference/status.html")"
  sed -i.bak 's/^gate_reasks: 3$/gate_reasks: 9/' "$TMP/e/docs/batches/batch-1-closed.md"
  node "$R" "$TMP/e" "$TMP/e/out3" >/dev/null 2>&1
  fp2="$(grep -oE 'built from <span class="code">fp-[0-9a-f]{12}' "$TMP/e/out3/reference/status.html")"
  [ -n "$fp1" ] && [ "$fp1" != "$fp2" ] \
    || { fail t10_economics_row_deterministic "the fp- stamp does not cover the batch ledgers ($fp1)"; return; }
  ok t10_economics_row_deterministic
}

t01_deck_true; t02_one_page_three_lenses; t03_facets_and_search; t04_supersession
t05_deterministic; t06_task_page_links; t07_changelog_binds_tasks; t08_spec_chunks
t09_nojs_and_honest_failures; t10_token_clean; t11_draft_staleness_report
t09_economics_row; t10_economics_row_deterministic
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
