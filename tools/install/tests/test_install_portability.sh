#!/usr/bin/env bash
# test_install_portability.sh - TASK-IMP-137 (t01-t07 -> AC 1-7): the MCP HTTP mode binds
# loopback by default and enforces the optional bearer token; bootstrap verifies checksums
# through the shasum fallback (verifies, never skips); the payload engines floor is one
# value; the GitHub Action channel really installs; the vendor step is stage-then-swap
# with no reader-visible gap; CHANGELOG names all five changes.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"; [ -n "${SRV_PID:-}" ] && kill "$SRV_PID" 2>/dev/null' EXIT
PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   $1"; }
fail() { FAIL=$((FAIL+1)); echo "  FAIL $1: $2"; }

echo "building scratch payload..."
bash "$repo/tools/install/build.sh" "$TMP/payload" >/dev/null 2>&1 || { echo FATAL build; exit 1; }
MCP="$TMP/payload/mcp/cyberos-mcp.mjs"
# Scan every versioned ## […] section — top entry moves with each cut.
top_entry() { awk '/^## \[/{p=1} p' "$repo/CHANGELOG.md"; }

# a non-loopback address of this host, when one exists (harness-conditional per AC 1)
second_ip() {
  ipconfig getifaddr en0 2>/dev/null && return 0
  hostname -I 2>/dev/null | awk '{print $1}' | grep -E '^[0-9]+\.' && return 0
  return 1
}
wait_http() { # $1 = url; poll until it answers (or 5s)
  local i=0
  while [ $i -lt 50 ]; do curl -s --max-time 1 "$1" >/dev/null 2>&1 && return 0; sleep 0.1; i=$((i+1)); done
  return 1
}
stop_srv() { [ -n "${SRV_PID:-}" ] && { kill "$SRV_PID" 2>/dev/null; wait "$SRV_PID" 2>/dev/null; SRV_PID=""; }; }

t01_loopback_default() {                                               # AC 1 (#1.1)
  command -v node >/dev/null 2>&1 || { fail t01 "node required"; return; }
  local port=$((21000 + RANDOM % 20000)) ip; ip="$(second_ip || true)"
  node "$MCP" --http "$port" 2>"$TMP/t01.err" & SRV_PID=$!
  wait_http "http://127.0.0.1:$port/healthz" || { fail t01 "server never answered on loopback"; stop_srv; return; }
  # behavior, not configuration-echo (audit ISS-005): the OS-reported bound address...
  if command -v lsof >/dev/null 2>&1; then
    local bound; bound="$(lsof -nP -a -iTCP:"$port" -sTCP:LISTEN 2>/dev/null || true)"
    grep -q "127\.0\.0\.1:$port" <<<"$bound" || { fail t01 "OS-reported bind is not loopback: $bound"; stop_srv; return; }
    grep -Eq "\*:$port|0\.0\.0\.0:$port" <<<"$bound" && { fail t01 "default bind reported wide: $bound"; stop_srv; return; }
  fi
  # ...AND a refused connect from a secondary address where the harness has one
  if [ -n "$ip" ]; then
    curl -s --max-time 2 "http://$ip:$port/healthz" >/dev/null 2>&1 \
      && { fail t01 "default bind ACCEPTED a non-loopback connect via $ip"; stop_srv; return; }
  fi
  stop_srv
  # --host 0.0.0.0 binds wide, and tokenless wide startup warns naming the exposure
  node "$MCP" --http "$port" --host 0.0.0.0 2>"$TMP/t01w.err" & SRV_PID=$!
  wait_http "http://127.0.0.1:$port/healthz" || { fail t01 "wide server never answered"; stop_srv; return; }
  grep -q "WARNING" "$TMP/t01w.err" || { fail t01 "no exposure warning on tokenless wide bind: $(cat "$TMP/t01w.err")"; stop_srv; return; }
  if [ -n "$ip" ]; then
    curl -s --max-time 2 "http://$ip:$port/healthz" >/dev/null 2>&1 \
      || { fail t01 "--host 0.0.0.0 refused a non-loopback connect via $ip"; stop_srv; return; }
  fi
  stop_srv
  ok t01_loopback_default
}

t02_bearer_token_enforced() {                                          # AC 2 (#1.2)
  command -v node >/dev/null 2>&1 || { fail t02 "node required"; return; }
  local port=$((21000 + RANDOM % 20000)) code body
  CYBEROS_MCP_TOKEN="t02-throwaway-token" node "$MCP" --http "$port" 2>"$TMP/t02.err" & SRV_PID=$!
  wait_http "http://127.0.0.1:$port/healthz" || { fail t02 "server never answered"; stop_srv; return; }
  code="$(curl -s -o "$TMP/t02.body" -w '%{http_code}' -X POST -d '{"jsonrpc":"2.0","id":1,"method":"ping"}' "http://127.0.0.1:$port/mcp")"
  [ "$code" = "401" ] || { fail t02 "tokenless POST got $code (want 401)"; stop_srv; return; }
  grep -q "t02-throwaway-token" "$TMP/t02.body" && { fail t02 "401 body leaks the token"; stop_srv; return; }
  code="$(curl -s -o /dev/null -w '%{http_code}' -X POST -H 'Authorization: Bearer wrong' -d '{"jsonrpc":"2.0","id":1,"method":"ping"}' "http://127.0.0.1:$port/mcp")"
  [ "$code" = "401" ] || { fail t02 "wrong token got $code (want 401)"; stop_srv; return; }
  body="$(curl -s -X POST -H 'Authorization: Bearer t02-throwaway-token' -d '{"jsonrpc":"2.0","id":7,"method":"ping"}' "http://127.0.0.1:$port/mcp")"
  grep -q '"result"' <<<"$body" || { fail t02 "correct token did not succeed: $body"; stop_srv; return; }
  code="$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:$port/healthz")"
  [ "$code" = "200" ] || { fail t02 "tokenless /healthz got $code (want 200 - probes stay open)"; stop_srv; return; }
  stop_srv
  ok t02_bearer_token_enforced
}

t03_shasum_fallback_verifies() {                                       # AC 3 (#1.3)
  command -v shasum >/dev/null 2>&1 || { fail t03 "host has no shasum - the fallback cannot be exercised here"; return; }
  # a minimal payload whose install.sh writes a marker: bootstrap's own logic is under
  # test (download via file://, verify, extract, delegate), not the full install
  local bsrc="$TMP/bsrc" bweb="$TMP/bweb"
  mkdir -p "$bsrc" "$bweb"
  printf '#!/usr/bin/env bash\ntouch "$1/.bootstrap-installed"\n' > "$bsrc/install.sh"
  tar -czf "$bweb/cyberos-payload.tar.gz" -C "$bsrc" .
  ( cd "$bweb" && shasum -a 256 cyberos-payload.tar.gz > SHA256SUMS )
  # masked PATH: every tool bootstrap + its children need, EXCEPT sha256sum
  local mask="$TMP/mask"; mkdir -p "$mask"
  local t p
  for t in bash sh env mktemp curl grep shasum dirname basename mkdir tar find head mv rm cat sed date sleep touch; do
    p="$(command -v "$t" 2>/dev/null || true)"; [ -n "$p" ] && ln -s "$p" "$mask/$t"
  done
  [ -e "$mask/sha256sum" ] && { fail t03 "mask dir leaked sha256sum"; return; }
  # good archive verifies and installs
  local tgt="$TMP/btgt"; mkdir -p "$tgt"
  local out rc
  out="$(env PATH="$mask" CYBEROS_PAYLOAD_URL="file://$bweb/cyberos-payload.tar.gz" bash "$repo/tools/install/bootstrap.sh" "$tgt" 2>&1)"; rc=$?
  [ "$rc" -eq 0 ] && [ -f "$tgt/.bootstrap-installed" ] || { fail t03 "shasum-only good-archive run failed rc=$rc: $out"; return; }
  # corrupted archive FAILS - the fallback verifies, it does not skip (audit ISS-001)
  local bweb2="$TMP/bweb2" tgt2="$TMP/btgt2"; mkdir -p "$tgt2"
  cp -R "$bweb" "$bweb2"; printf 'corruption' >> "$bweb2/cyberos-payload.tar.gz"
  out="$(env PATH="$mask" CYBEROS_PAYLOAD_URL="file://$bweb2/cyberos-payload.tar.gz" bash "$repo/tools/install/bootstrap.sh" "$tgt2" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] || { fail t03 "corrupted archive PASSED under the fallback - it skipped, not verified"; return; }
  grep -q "checksum mismatch" <<<"$out" || { fail t03 "corruption not named: $out"; return; }
  [ ! -f "$tgt2/.bootstrap-installed" ] || { fail t03 "corrupted payload was INSTALLED"; return; }
  # neither tool: abort naming both, before anything is fetched
  local mask2="$TMP/mask2"; mkdir -p "$mask2"
  for t in bash sh env mktemp curl grep dirname basename mkdir tar find head mv rm cat sed date sleep touch; do
    p="$(command -v "$t" 2>/dev/null || true)"; [ -n "$p" ] && ln -s "$p" "$mask2/$t"
  done
  local tgt3="$TMP/btgt3"; mkdir -p "$tgt3"
  out="$(env PATH="$mask2" CYBEROS_PAYLOAD_URL="file://$bweb/cyberos-payload.tar.gz" bash "$repo/tools/install/bootstrap.sh" "$tgt3" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] || { fail t03 "no-verifier run did not abort"; return; }
  grep -q "sha256sum" <<<"$out" && grep -q "shasum" <<<"$out" || { fail t03 "abort does not name both tools: $out"; return; }
  ok t03_shasum_fallback_verifies
}

t04_engines_unified() {                                                # AC 4 (#1.4)
  grep -qF '"engines": { "node": ">=24 <25" }' "$TMP/payload/package.json" \
    || { fail t04 "payload engines: $(grep engines "$TMP/payload/package.json")"; return; }
  if command -v node >/dev/null 2>&1; then
    local e; e="$(node -p 'JSON.parse(require("fs").readFileSync(process.argv[1],"utf8")).engines.node' "$TMP/payload/package.json")"
    [ "$e" = ">=24 <25" ] || { fail t04 "parsed engines.node='$e' (want '>=24 <25')"; return; }
  fi
  ok t04_engines_unified
}

t05_ci_channel_real() {                                                # AC 5 (#1.5)
  local d="$TMP/ci-fix"; mkdir -p "$d"
  ( cd "$d" && git init -q . 2>/dev/null; CYBEROS_OFFLINE=1 CYBEROS_NO_MIGRATE=1 bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1 )
  [ -f "$d/.cyberos/ci/github-action/action.yml" ] || { fail t05 ".cyberos/ci/github-action/action.yml not installed"; return; }
  grep -qF 'uses: ./.cyberos/ci/github-action' "$repo/tools/install/README.md" \
    || { fail t05 "README lacks the installed-path uses: example"; return; }
  grep -qF 'dist/cyberos/ci' "$repo/tools/install/README.md" \
    && { fail t05 "README still claims a dist/ ci path works post-install"; return; }
  ok t05_ci_channel_real
}

t06_atomic_swap_no_reader_gap() {                                      # AC 6 (#1.6)
  local d="$TMP/atomic"; mkdir -p "$d"
  ( cd "$d" && git init -q . 2>/dev/null; CYBEROS_OFFLINE=1 CYBEROS_NO_MIGRATE=1 bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1 )
  [ -f "$d/.cyberos/cuo/ship-tasks.md" ] || { fail t06 "baseline install incomplete"; return; }
  # a tight reader polls the machine's entry point across 20 reinstalls
  ( cd "$d" && polls=0 miss=0
    while [ ! -f STOP ]; do
      [ -e .cyberos/cuo/ship-tasks.md ] || miss=$((miss+1))
      polls=$((polls+1))
    done
    echo "$polls $miss" > reader-result ) & local rp=$!
  local i=0
  while [ $i -lt 20 ]; do
    CYBEROS_OFFLINE=1 CYBEROS_NO_MIGRATE=1 bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
    i=$((i+1))
  done
  # a CHANGED payload takes the per-path rename-sync arm (identical reinstalls above take
  # the fingerprint short-circuit) - the reader must see zero absences on that arm too,
  # and adds/updates/prunes must all land
  echo "# upgrade marker t06" >> "$TMP/payload/cuo/ship-tasks.md"
  mkdir -p "$TMP/payload/cuo/t06-new-dir"; echo new > "$TMP/payload/cuo/t06-new-dir/inner.md"
  rm -f "$TMP/payload/cuo/STATUS-REFERENCE.md"
  CYBEROS_OFFLINE=1 CYBEROS_NO_MIGRATE=1 bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
  touch "$d/STOP"; wait "$rp" 2>/dev/null
  local polls miss; read -r polls miss < "$d/reader-result"
  [ "$polls" -gt 1000 ] || { fail t06 "reader barely ran ($polls polls) - result not meaningful"; return; }
  [ "$miss" -eq 0 ] || { fail t06 "reader observed $miss absences of ship-tasks.md across the reinstalls ($polls polls)"; return; }
  grep -q "upgrade marker t06" "$d/.cyberos/cuo/ship-tasks.md" || { fail t06 "changed file did not sync"; return; }
  [ -f "$d/.cyberos/cuo/t06-new-dir/inner.md" ] || { fail t06 "new payload dir/file did not sync"; return; }
  [ ! -e "$d/.cyberos/cuo/STATUS-REFERENCE.md" ] || { fail t06 "payload-dropped file was not pruned"; return; }
  # kill between stage and swap: the post-kill state is a completed staging dir beside an
  # UNTOUCHED old tree (stage precedes any touch of the live tree by construction)
  cp -R "$d/.cyberos/cuo" "$d/.cyberos/cuo.tmp.4242.1"
  mkdir -p "$d/.cyberos/plugin.old.4242.1"
  [ -f "$d/.cyberos/cuo/ship-tasks.md" ] || { fail t06 "old tree not intact beside the stray stage"; return; }
  CYBEROS_OFFLINE=1 CYBEROS_NO_MIGRATE=1 bash "$TMP/payload/install.sh" "$d" >/dev/null 2>&1
  ls -d "$d/.cyberos/"*.tmp.* >/dev/null 2>&1 && { fail t06 "stray staging dir survived the next install"; return; }
  ls -d "$d/.cyberos/"*.old.* >/dev/null 2>&1 && { fail t06 "stray trash dir survived the next install"; return; }
  [ -f "$d/.cyberos/cuo/ship-tasks.md" ] || { fail t06 "machine not healthy after stray cleanup"; return; }
  ok t06_atomic_swap_no_reader_gap
}

t07_changelog_five_changes() {                                         # AC 7 (#1.7)
  local top; top="$(top_entry)"
  grep -qi "breaking" <<<"$top"          || { fail t07 "no breaking language"; return; }
  grep -q "127\.0\.0\.1" <<<"$top"       || { fail t07 "loopback default not named"; return; }
  grep -q "CYBEROS_MCP_TOKEN" <<<"$top"  || { fail t07 "token auth not named"; return; }
  grep -q "shasum" <<<"$top"             || { fail t07 "shasum fallback not named"; return; }
  grep -qF ">=24 <25" <<<"$top"          || { fail t07 "engines floor not named"; return; }
  grep -qF ".cyberos/ci" <<<"$top"       || { fail t07 "ci channel not named"; return; }
  grep -qi "stage" <<<"$top"             || { fail t07 "stage-then-swap not named"; return; }
  ok t07_changelog_five_changes
}

t08_rollout_sums_exact_match() {                                        # rollout SHA256SUMS filename pin
  # The rollout chooser must match only a valid digest + exact payload name —
  # alternate filenames / non-hex digests must not be selected for -c verify.
  local dig matched
  dig="$(printf '%064d' 0 | tr '0' 'a')"
  matched="$(printf '%s\n' \
    "$dig  cyberos-payload.tar.gz" \
    "$dig *cyberos-payload.tar.gz" \
    "$dig  evil-payload.tar.gz" \
    "notadigest  cyberos-payload.tar.gz" \
    "$dig  cyberos-payloadXtar.gz" \
    | grep -cE '^[[:xdigit:]]{64}[[:space:]]+\*?cyberos-payload\.tar\.gz$' || true)"
  [ "$matched" = "2" ] || { fail t08 "exact-match filter accepted $matched rows (want 2 good forms)"; return; }
  # Pin the full production regex from rollout.sh (digest + optional '*' + exact
  # filename + EOL) — a prefix-only check would miss a dropped `$` or filename pin.
  grep -qF "grep -E '^[[:xdigit:]]{64}[[:space:]]+\\*?cyberos-payload\\.tar\\.gz$'" \
    "$repo/tools/install/rollout.sh" \
    || { fail t08 "rollout.sh lacks the full anchored exact-match checksum grep"; return; }
  ok t08_rollout_sums_exact_match
}

echo "test_install_portability.sh (TASK-IMP-137)"
t01_loopback_default; t02_bearer_token_enforced; t03_shasum_fallback_verifies
t04_engines_unified; t05_ci_channel_real; t06_atomic_swap_no_reader_gap; t07_changelog_five_changes
t08_rollout_sums_exact_match
echo "----"; echo "pass=$PASS fail=$FAIL"; [ "$FAIL" -eq 0 ]
