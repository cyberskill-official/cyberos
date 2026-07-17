#!/usr/bin/env bash
# test_memory_append.sh - memory-append.mjs, the doc-driven BRAIN chain appender
# (TASK-IMP-093).
#
#   t01  fresh store: three appends bootstrap (HEAD=0, null-root prev_chain,
#        canonical dirs) then advance HEAD by exactly 3; rows are chained
#        (prev_chain[n] == chain[n-1], verified independently of the tool);
#        a stale two-phase tmp seeded mid-sequence is cleaned and never
#        corrupts HEAD or the chain.
#   t02  verify passes clean on an intact store; ONE flipped payload byte makes
#        it exit non-zero NAMING the tampered ordinal; a held §4.2 lease makes
#        a concurrent append fail fast with nothing written; an expired lease
#        is reaped loudly and the append proceeds.
#   t03  kinds outside the closed four-kind set are refused BEFORE any write -
#        the store stays byte-identical and a fresh path gains no files; the
#        non-JSON and non-object payload arms are refused the same way.
#   t04  the assembled payload carries the tool byte-identically and the
#        vendored copy runs (--help exits 0 and documents both subcommands).
#
# Origin: IMPROVEMENT_HANDOFF.md IMP-05 - ship-tasks phases declare memory rows
# but a doc-driven run had no writer; the sachviet run parked payloads in a
# tracked _audits file (chain-shaped data with no chain). This suite gates the
# writer that closes that hole.
#
# Usage: bash test_memory_append.sh [t01 t02 ...]   (no args = all scenarios)
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
MA="$repo/tools/install/docs-tools/memory-append.mjs"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }
only="$*"
want() { [ -z "$only" ] && return 0; case " $only " in *" $1 "*) return 0;; *) return 1;; esac; }

NOW="2026-07-17T00:00:00Z"   # injectable clock: CYBEROS_NOW pins ts_ns
ma() { CYBEROS_NOW="$NOW" CYBEROS_ACTOR="suite" node "$MA" "$@" > "$TMP/out" 2> "$TMP/err"; }
head_val() { node -e 'process.stdout.write(String(require("node:fs").readFileSync(process.argv[1]).readBigUInt64LE(0)))' "$1/HEAD"; }
# content fingerprint of every regular file in a store (bytes only, order pinned)
store_sum() { (cd "$1" && find . -type f | LC_ALL=C sort | xargs sha256sum) ; }

emit_payloads() { # $1 = dir
  mkdir -p "$1"
  printf '{"task_id":"TASK-T-001","phase":"implementing","artefacts":["context-map.md"]}\n' > "$1/p1.json"
  printf '{"task_id":"TASK-T-001","phase":"reviewing","note":"ti\xE1\xBA\xBFng Vi\xE1\xBB\x87t \xE2\x9C\x93"}\n' > "$1/p2.json"
  printf '{"task":"TASK-T-001","summary":"all gates green"}\n' > "$1/p3.json"
}

t01_fresh_store_three_appends() {
  local d="$TMP/t01"; emit_payloads "$d"; local s="$d/store"
  # 1st append bootstraps: HEAD=0 then advances to 1; canonical dirs exist
  ma append "$s" workflow_phase_complete "$d/p1.json" \
    || { fail t01 "first append failed: $(cat "$TMP/err")"; return; }
  grep -q "bootstrapped fresh store" "$TMP/err" || { fail t01 "no bootstrap note on stderr"; return; }
  for dir in audit memories meta conflicts exports index; do
    [ -d "$s/$dir" ] || { fail t01 "bootstrap did not create $dir/"; return; }
  done
  [ "$(head_val "$s")" = "1" ] || { fail t01 "HEAD after append 1 is $(head_val "$s")"; return; }
  # seed stale two-phase tmp litter from an "interrupted" append (both patterns)
  printf 'GARBAGE-not-a-frame' > "$s/audit/current.binlog.tmp.deadbeef"
  printf 'GARBAGE' > "$s/HEAD.tmp"
  # 2nd + 3rd append (one via stdin '-') must clean the litter and stay consistent
  ma append "$s" task_routed_back "$d/p2.json" \
    || { fail t01 "append 2 over stale tmp failed: $(cat "$TMP/err")"; return; }
  grep -q "cleaned 2 stale tmp file(s)" "$TMP/err" || { fail t01 "stale tmp not cleaned/announced: $(cat "$TMP/err")"; return; }
  [ ! -e "$s/audit/current.binlog.tmp.deadbeef" ] || { fail t01 "stale binlog tmp survived"; return; }
  [ ! -e "$s/HEAD.tmp" ] || { fail t01 "stale HEAD.tmp survived"; return; }
  CYBEROS_NOW="$NOW" CYBEROS_ACTOR="suite" node "$MA" --json append "$s" workflow_complete - < "$d/p3.json" > "$TMP/out" 2> "$TMP/err" \
    || { fail t01 "append 3 via stdin failed: $(cat "$TMP/err")"; return; }
  node -e 'const j=JSON.parse(require("node:fs").readFileSync(process.argv[1],"utf8")); if (j.ok!==true||j.seq!==3||!/^[0-9a-f]{64}$/.test(j.chain)) process.exit(1)' "$TMP/out" \
    || { fail t01 "--json envelope wrong: $(cat "$TMP/out")"; return; }
  # HEAD advanced by exactly 3
  [ "$(head_val "$s")" = "3" ] || { fail t01 "HEAD is $(head_val "$s"), want 3"; return; }
  # rows chained: independent frame walk (not the tool) proves prev_chain linkage
  node -e '
    const fs=require("node:fs"),crypto=require("node:crypto");
    const b=fs.readFileSync(process.argv[1]); let off=0,prev="0".repeat(64),n=0;
    while(off<b.length){
      const len=b.readUInt32BE(off), seq=b.readBigUInt64BE(off+8);
      const rec=JSON.parse(b.subarray(off+24,off+24+len).toString("utf8"));
      if(rec.prev_chain!==prev){console.error(`row ${seq}: prev_chain broken`);process.exit(1)}
      if(Number(seq)!==n+1){console.error(`row seq ${seq} != ${n+1}`);process.exit(1)}
      prev=rec.chain; n++; off+=24+len;
    }
    if(n!==3){console.error(`walked ${n} rows, want 3`);process.exit(1)}
    const kinds=["workflow_phase_complete","task_routed_back","workflow_complete"];
  ' "$s/audit/current.binlog" || { fail t01 "independent chain walk failed"; return; }
  # and the tool's own verify agrees
  ma verify "$s" || { fail t01 "verify after 3 appends failed: $(cat "$TMP/err")"; return; }
  grep -q "3 row(s), HEAD=3" "$TMP/out" || { fail t01 "verify summary wrong: $(cat "$TMP/out")"; return; }
  # no tmp litter left behind by the appends themselves
  [ "$(find "$s" -name '*.tmp.*' | wc -l)" -eq 0 ] || { fail t01 "append left tmp litter"; return; }
  ok t01
}

t02_verify_and_tamper() {
  local d="$TMP/t02"; emit_payloads "$d"; local s="$d/store"
  ma append "$s" workflow_phase_complete "$d/p1.json" || { fail t02 "seed append 1 failed"; return; }
  ma append "$s" artefact_write "$d/p2.json" || { fail t02 "seed append 2 failed"; return; }
  # clean pass
  ma verify "$s"; local rc=$?
  { [ "$rc" -eq 0 ] && grep -q "verify: OK" "$TMP/out"; } || { fail t02 "clean verify rc=$rc: $(cat "$TMP/out")"; return; }
  # flip ONE payload byte in row 1 -> non-zero naming ordinal 1 (the FIRST bad row)
  cp "$s/audit/current.binlog" "$TMP/t02/binlog.bak"
  node -e '
    const fs=require("node:fs");const b=fs.readFileSync(process.argv[1]);
    b[24+5]^=0x01; fs.writeFileSync(process.argv[1],b);' "$s/audit/current.binlog"
  ma verify "$s"; rc=$?
  { [ "$rc" -eq 4 ] && grep -q "first bad ordinal 1" "$TMP/err"; } \
    || { fail t02 "tampered verify rc=$rc err=$(cat "$TMP/err")"; return; }
  grep -q "ordinal 2" "$TMP/err" && { fail t02 "verify named a later ordinal, not the first"; return; }
  cp "$TMP/t02/binlog.bak" "$s/audit/current.binlog"
  # a chain-field flip (payload still valid JSON) is also caught, naming ordinal 2
  node -e '
    const fs=require("node:fs");const b=fs.readFileSync(process.argv[1]);
    let off=0,idx=0; while(off<b.length){const len=b.readUInt32BE(off); idx++;
      if(idx===2){const p=b.subarray(off+24,off+24+len);const i=p.indexOf(Buffer.from("\"chain\":\""))+9;
        p[i]=p[i]===97?98:97; // keep it hex-ish, break the hash
        // recompute crc so ONLY the chain check can catch it
        const CRC=(()=>{const t=new Uint32Array(256);for(let n=0;n<256;n++){let c=n;for(let k=0;k<8;k++)c=c&1?(0x82f63b78^(c>>>1))>>>0:c>>>1;t[n]=c>>>0}return t})();
        let c=0xffffffff;for(const x of p)c=(CRC[(c^x)&0xff]^(c>>>8))>>>0;
        b.writeUInt32BE((c^0xffffffff)>>>0,off+4);break}
      off+=24+len}
    fs.writeFileSync(process.argv[1],b);' "$s/audit/current.binlog"
  ma verify "$s"; rc=$?
  { [ "$rc" -eq 4 ] && grep -q "first bad ordinal 2" "$TMP/err"; } \
    || { fail t02 "chain-flip verify rc=$rc err=$(cat "$TMP/err")"; return; }
  cp "$TMP/t02/binlog.bak" "$s/audit/current.binlog"
  # held lock -> second invocation fails fast, nothing written. The lease must be
  # REALISTIC (live foreign holder, expiry inside one TTL of the current monotonic
  # clock): the PR-review reboot guard reaps anything whose expiry sits beyond one
  # TTL horizon, so the old 9e18 fixture now belongs to the boot-skew arm below.
  node -e 'const now=process.hrtime.bigint(); process.stdout.write(JSON.stringify({pid:99999,host:"other",monotonic_ns:Number(now),expiry_ns:Number(now+8000000000n),version:1}))' > "$s/.lock"
  local before; before="$(sha256sum "$s/audit/current.binlog" "$s/HEAD" | sha256sum)"
  ma append "$s" workflow_complete "$d/p3.json"; rc=$?
  { [ "$rc" -eq 3 ] && grep -q "locked" "$TMP/err" && grep -q "failing fast" "$TMP/err"; } \
    || { fail t02 "held-lock append rc=$rc err=$(cat "$TMP/err")"; return; }
  [ "$(sha256sum "$s/audit/current.binlog" "$s/HEAD" | sha256sum)" = "$before" ] \
    || { fail t02 "held-lock append mutated the store"; return; }
  grep -q '"pid":99999' "$s/.lock" || { fail t02 "held lease was clobbered by the refused append"; return; }
  # boot-epoch skew (PR-review, Devin 2026-07-17): a pre-reboot .lock carries a monotonic
  # expiry far beyond the fresh clock - without the horizon guard this wedges appends
  # forever; with it, the lease is reaped as stale and the append proceeds.
  printf '{"pid":99999,"host":"other","monotonic_ns":1,"expiry_ns":9000000000000000000,"version":1}' > "$s/.lock"
  ma append "$s" workflow_complete "$d/p3.json"; rc=$?
  { [ "$rc" -eq 0 ] && grep -q "boot-epoch skew" "$TMP/err"; } \
    || { fail t02 "boot-skew lease append rc=$rc err=$(cat "$TMP/err")"; return; }
  # orphan lease (same host, holder pid dead, expiry still valid) -> reaped via the
  # kill(pid,0) liveness probe, append proceeds.
  dead_pid="$(bash -c 'echo $BASHPID')"   # that shell has exited; its pid is free/dead
  node -e "const now=process.hrtime.bigint(); const os=require('os'); process.stdout.write(JSON.stringify({pid:${dead_pid},host:os.hostname(),monotonic_ns:Number(now),expiry_ns:Number(now+8000000000n),version:1}))" > "$s/.lock"
  ma append "$s" workflow_complete "$d/p3.json"; rc=$?
  { [ "$rc" -eq 0 ] && grep -q "holder pid is gone" "$TMP/err"; } \
    || { fail t02 "orphan-lease append rc=$rc err=$(cat "$TMP/err")"; return; }
  # expired lease is reaped loudly and the append proceeds
  printf '{"pid":99999,"host":"other","monotonic_ns":1,"expiry_ns":1,"version":1}' > "$s/.lock"
  ma append "$s" workflow_complete "$d/p3.json"; rc=$?
  { [ "$rc" -eq 0 ] && grep -q "reaping stale lease" "$TMP/err"; } \
    || { fail t02 "expired-lease append rc=$rc err=$(cat "$TMP/err")"; return; }
  [ "$(head_val "$s")" = "5" ] || { fail t02 "HEAD after the three reap-appends is $(head_val "$s")"; return; }
  ma verify "$s" || { fail t02 "final verify failed: $(cat "$TMP/err")"; return; }
  ok t02
}

t03_bad_kind_refused() {
  local d="$TMP/t03"; emit_payloads "$d"; local s="$d/store"
  ma append "$s" workflow_phase_complete "$d/p1.json" || { fail t03 "seed append failed"; return; }
  store_sum "$s" > "$d/sum.before"
  # unknown kind -> exit 2, refusal names the closed set, store byte-untouched
  ma append "$s" episode_logged "$d/p1.json"; local rc=$?
  { [ "$rc" -eq 2 ] && grep -q "refused" "$TMP/err" && grep -q "workflow_phase_complete" "$TMP/err"; } \
    || { fail t03 "bad kind rc=$rc err=$(cat "$TMP/err")"; return; }
  store_sum "$s" > "$d/sum.after"
  cmp -s "$d/sum.before" "$d/sum.after" || { fail t03 "bad kind touched store bytes: $(diff "$d/sum.before" "$d/sum.after")"; return; }
  # bad kind against a FRESH path creates nothing (refusal precedes bootstrap too)
  ma append "$d/never" bogus "$d/p1.json"; rc=$?
  { [ "$rc" -eq 2 ] && [ ! -e "$d/never" ]; } || { fail t03 "bad kind bootstrapped a store (rc=$rc)"; return; }
  # non-JSON payload -> refused before write
  printf 'this is not json {' > "$d/bad.txt"
  ma append "$s" workflow_complete "$d/bad.txt"; rc=$?
  { [ "$rc" -eq 2 ] && grep -q "not JSON" "$TMP/err"; } || { fail t03 "non-JSON rc=$rc err=$(cat "$TMP/err")"; return; }
  store_sum "$s" > "$d/sum.after2"
  cmp -s "$d/sum.before" "$d/sum.after2" || { fail t03 "non-JSON payload touched store bytes"; return; }
  # JSON but not an object -> refused the same way
  printf '[1,2,3]' > "$d/arr.json"
  ma append "$s" workflow_complete "$d/arr.json"; rc=$?
  { [ "$rc" -eq 2 ] && grep -q "must be a JSON object" "$TMP/err"; } || { fail t03 "non-object rc=$rc"; return; }
  store_sum "$s" > "$d/sum.after3"
  cmp -s "$d/sum.before" "$d/sum.after3" || { fail t03 "non-object payload touched store bytes"; return; }
  # the store still verifies and HEAD never moved
  [ "$(head_val "$s")" = "1" ] || { fail t03 "HEAD moved to $(head_val "$s")"; return; }
  ma verify "$s" || { fail t03 "store no longer verifies: $(cat "$TMP/err")"; return; }
  ok t03
}

t04_payload_vendored() {
  bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { fail t04 "build.sh failed"; return; }
  [ -s "$TMP/payload/docs-tools/memory-append.mjs" ] || { fail t04 "payload docs-tools/memory-append.mjs missing or empty"; return; }
  cmp -s "$MA" "$TMP/payload/docs-tools/memory-append.mjs" \
    || { fail t04 "payload copy differs from tools/install/docs-tools/memory-append.mjs"; return; }
  node "$TMP/payload/docs-tools/memory-append.mjs" --help > "$TMP/out" 2>&1 \
    || { fail t04 "vendored copy --help failed"; return; }
  grep -q "append <store-root> <kind>" "$TMP/out" && grep -q "verify <store-root>" "$TMP/out" \
    || { fail t04 "--help does not document both subcommands"; return; }
  # the vendored copy actually appends + verifies against a scratch store
  local d="$TMP/t04"; emit_payloads "$d"
  CYBEROS_NOW="$NOW" node "$TMP/payload/docs-tools/memory-append.mjs" append "$d/store" artefact_write "$d/p1.json" >/dev/null 2>&1 \
    && CYBEROS_NOW="$NOW" node "$TMP/payload/docs-tools/memory-append.mjs" verify "$d/store" >/dev/null 2>&1 \
    || { fail t04 "vendored copy append/verify lifecycle failed"; return; }
  ok t04
}

want t01 && t01_fresh_store_three_appends
want t02 && t02_verify_and_tamper
want t03 && t03_bad_kind_refused
want t04 && t04_payload_vendored

echo "test_memory_append: pass=$PASS fail=$FAIL"
[ "$FAIL" -eq 0 ]
