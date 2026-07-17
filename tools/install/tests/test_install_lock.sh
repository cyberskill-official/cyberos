#!/usr/bin/env bash
# TASK-IMP-103 - install concurrency lock. One arm per AC.
# No model, no network. Drives install.sh's lock helpers directly against a scratch .cyberos.
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; root="$(cd "$here/../../.." && pwd)"
pass=0; fail=0
ok(){ printf '  ok   %s\n' "$1"; pass=$((pass+1)); }
no(){ printf '  FAIL %s\n     %s\n' "$1" "${2:-}"; fail=$((fail+1)); }
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# Extract the lock block into a harness so the arms exercise the REAL code, not a copy.
mk_harness(){
  local d="$1"; mkdir -p "$d/.cyberos"
  { echo '#!/usr/bin/env bash'; echo 'set -uo pipefail'; echo "CY=\"$d/.cyberos\""
    sed -n '/^CY_LOCK=/,/^_cy_lock_acquire$/p' "$root/tools/install/install.sh" | sed '$d'
    echo '"$@"'
  } > "$d/h.sh"; chmod +x "$d/h.sh"
}

# --- AC 1 (#1.1,#1.2,#1.5): concurrent install refuses, names both pids, holder untouched
t01_concurrent_refuses(){
  local d="$TMP/t01"; mk_harness "$d"
  mkdir -p "$d/.cyberos/.install.lock"
  printf 'pid=%s\nstarted_at=x\nhost=%s\n' "$$" "$(hostname 2>/dev/null || echo unknown)" > "$d/.cyberos/.install.lock/owner"
  local out rc; out="$("$d/h.sh" _cy_lock_acquire 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] || { no t01_concurrent_refuses "expected non-zero, got $rc"; return; }
  grep -q "another install holds the lock" <<<"$out" || { no t01_concurrent_refuses "no contention wording: $out"; return; }
  grep -q "pid $$" <<<"$out" || { no t01_concurrent_refuses "holder pid not named: $out"; return; }
  [ -f "$d/.cyberos/.install.lock/owner" ] || { no t01_concurrent_refuses "holder's lock was released by the refusal path (violates 1.5)"; return; }
  ok t01_concurrent_refuses
}
# --- AC 2 (#1.3): stale + dead pid -> broken with a warning naming pid and age
t02_stale_broken_with_warning(){
  local d="$TMP/t02"; mk_harness "$d"
  mkdir -p "$d/.cyberos/.install.lock"
  printf 'pid=999999\nstarted_at=x\nhost=%s\n' "$(hostname 2>/dev/null || echo unknown)" > "$d/.cyberos/.install.lock/owner"
  touch -t 200001010000 "$d/.cyberos/.install.lock" 2>/dev/null
  local out rc; out="$(CYBEROS_LOCK_STALE_SECS=1 "$d/h.sh" _cy_lock_acquire 2>&1)"; rc=$?
  [ "$rc" -eq 0 ] || { no t02_stale_broken_with_warning "expected acquire, rc=$rc: $out"; return; }
  grep -q "breaking stale lock" <<<"$out" || { no t02_stale_broken_with_warning "no break warning: $out"; return; }
  grep -q "999999" <<<"$out" || { no t02_stale_broken_with_warning "pid not named: $out"; return; }
  grep -qE "age [0-9]+s" <<<"$out" || { no t02_stale_broken_with_warning "age not named: $out"; return; }
  ok t02_stale_broken_with_warning
}
# --- AC 3 (#1.4): fresh lock + dead pid -> REFUSES (age gate, not pid alone)
t03_fresh_dead_pid_refuses(){
  local d="$TMP/t03"; mk_harness "$d"
  mkdir -p "$d/.cyberos/.install.lock"
  printf 'pid=999999\nstarted_at=x\nhost=%s\n' "$(hostname 2>/dev/null || echo unknown)" > "$d/.cyberos/.install.lock/owner"
  local out rc; out="$(CYBEROS_LOCK_STALE_SECS=900 "$d/h.sh" _cy_lock_acquire 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] || { no t03_fresh_dead_pid_refuses "broke a fresh lock on a dead pid - violates 1.4"; return; }
  grep -q "another install holds the lock" <<<"$out" || { no t03_fresh_dead_pid_refuses "wrong wording: $out"; return; }
  ok t03_fresh_dead_pid_refuses
}
# --- AC 4 (#1.5): trap releases on signal; next install acquires cleanly
t04_trap_releases_on_signal(){
  local d="$TMP/t04"; mk_harness "$d"
  { echo '#!/usr/bin/env bash'; echo 'set -uo pipefail'; echo "CY=\"$d/.cyberos\""
    sed -n '/^CY_LOCK=/,/^_cy_lock_acquire$/p' "$root/tools/install/install.sh" | sed '$d'
    echo '_cy_lock_acquire'; echo 'sleep 1'
  } > "$d/hold.sh"
  chmod +x "$d/hold.sh"; "$d/hold.sh" & local hp=$!
  local i=0; while [ ! -d "$d/.cyberos/.install.lock" ] && [ $i -lt 50 ]; do sleep 0.1; i=$((i+1)); done
  [ -d "$d/.cyberos/.install.lock" ] || { no t04_trap_releases_on_signal "holder never acquired"; kill $hp 2>/dev/null; return; }
  # bash defers a trap until the foreground child exits, so the release lands when the
  # holder actually dies - which is the invariant 1.5 states. We assert the OUTCOME (no lock
  # survives a dead install), not the latency of bash's signal delivery.
  kill -TERM $hp 2>/dev/null; wait $hp 2>/dev/null
  i=0; while [ -d "$d/.cyberos/.install.lock" ] && [ $i -lt 50 ]; do sleep 0.1; i=$((i+1)); done
  [ -d "$d/.cyberos/.install.lock" ] && { no t04_trap_releases_on_signal "lock survived SIGTERM - trap did not fire"; return; }
  "$d/h.sh" _cy_lock_acquire >/dev/null 2>&1 || { no t04_trap_releases_on_signal "next install could not acquire"; return; }
  ok t04_trap_releases_on_signal
}
# --- AC 5 (#1.6): uninstall refuses under a LIVE install; removes a stale lock
t05_uninstall_lock_ownership(){
  local d="$TMP/t05"; mkdir -p "$d/.cyberos/.install.lock"
  printf 'pid=%s\nstarted_at=x\nhost=%s\n' "$$" "$(hostname 2>/dev/null || echo unknown)" > "$d/.cyberos/.install.lock/owner"
  local blk; blk="$(sed -n '/^_ul="\$CY\/.install.lock"$/,/^fi$/p' "$root/tools/install/uninstall.sh")"
  [ -n "$blk" ] || { no t05_uninstall_lock_ownership "ownership block not found in uninstall.sh"; return; }
  local out rc
  out="$(CY="$d/.cyberos" bash -c "set -uo pipefail; $blk" 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] || { no t05_uninstall_lock_ownership "uninstall did not refuse under a live install (rc=$rc)"; return; }
  grep -q "Refusing to remove the machine" <<<"$out" || { no t05_uninstall_lock_ownership "wrong refusal: $out"; return; }
  printf 'pid=999999\nstarted_at=x\nhost=%s\n' "$(hostname 2>/dev/null || echo unknown)" > "$d/.cyberos/.install.lock/owner"
  out="$(CY="$d/.cyberos" bash -c "set -uo pipefail; $blk" 2>&1)"; rc=$?
  [ "$rc" -eq 0 ] || { no t05_uninstall_lock_ownership "refused on a stale lock (rc=$rc): $out"; return; }
  grep -q "removing stale install lock" <<<"$out" || { no t05_uninstall_lock_ownership "stale removal not named: $out"; return; }
  ok t05_uninstall_lock_ownership
}
# --- edge (§3): mkdir failure that is NOT contention must say so
t06_non_contention_failure_named(){
  local d="$TMP/t06"; mk_harness "$d"
  : > "$d/.cyberos/.install.lock"     # a FILE where the lock dir goes: mkdir fails, not contention
  local out rc; out="$("$d/h.sh" _cy_lock_acquire 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] || { no t06_non_contention_failure_named "expected failure"; return; }
  grep -q "NOT contention" <<<"$out" || { no t06_non_contention_failure_named "conflated with contention: $out"; return; }
  ok t06_non_contention_failure_named
}
# --- edge (§3): a foreign host's pid is unknowable -> treated as ALIVE until the threshold
t07_foreign_host_pid_is_alive(){
  local d="$TMP/t07"; mk_harness "$d"
  mkdir -p "$d/.cyberos/.install.lock"
  # dead pid, but owned by another host: liveness is undecidable, so it must NOT be broken
  printf 'pid=999999\nstarted_at=x\nhost=some-other-machine\n' > "$d/.cyberos/.install.lock/owner"
  # FRESH (mtime = now) + a live threshold: liveness is unverifiable, so it must defer, not break
  local out rc; out="$(CYBEROS_LOCK_STALE_SECS=900 "$d/h.sh" _cy_lock_acquire 2>&1)"; rc=$?
  [ "$rc" -ne 0 ] || { no t07_foreign_host_pid_is_alive "broke a foreign-host lock on an unverifiable pid"; return; }
  grep -q "on some-other-machine" <<<"$out" || { no t07_foreign_host_pid_is_alive "holder host not named: $out"; return; }
  # ...but once the threshold expires, it IS broken - the threshold is the only honest signal
  # there, and deferring forever would wedge the lock permanently on a shared mount.
  touch -t 200001010000 "$d/.cyberos/.install.lock" 2>/dev/null
  out="$(CYBEROS_LOCK_STALE_SECS=1 "$d/h.sh" _cy_lock_acquire 2>&1)"; rc=$?
  [ "$rc" -eq 0 ] || { no t07_foreign_host_pid_is_alive "stale foreign lock never expires (rc=$rc)"; return; }
  ok t07_foreign_host_pid_is_alive
}
echo "test_install_lock.sh (TASK-IMP-103)"
# t08 - the stale-break handoff. Greptile P1, 2026-07-17: `_cy_lock_held` records that we ONCE
# owned the lock, never that we STILL do. A holds it and hangs; B breaks the stale lock (§1.3) and
# mkdirs a new one at the same path; A wakes and exits. A held-only release deletes B's lock and
# reopens the unguarded vendor window - the exact window the lock exists to close.
t08_stale_break_handoff_is_not_released(){
  local d="$TMP/t08"; mkdir -p "$d/.cyberos/.install.lock"
  # the lock now at this path belongs to B, not to us
  printf 'pid=999999\nstarted_at=2099-01-01T00:00:00Z\nhost=other\n' > "$d/.cyberos/.install.lock/owner"
  local blk; blk="$(sed -n '/^CY_LOCK=/,/^_cy_lock_acquire$/p' "$root/tools/install/install.sh" | sed '$d')"
  # A's state: it held the lock and stamped it with ITS bytes, which are no longer on disk.
  CY="$d/.cyberos" bash -c "set -uo pipefail
CY=\"$d/.cyberos\"
$blk
_cy_lock_held=1
_cy_lock_stamp=\"pid=1
started_at=1970-01-01T00:00:00Z
host=A\"
_cy_lock_release" >/dev/null 2>&1
  if [ -d "$d/.cyberos/.install.lock" ]; then ok t08_stale_break_handoff_is_not_released
  else no t08_stale_break_handoff_is_not_released "A's trap deleted a lock it no longer owned"; fi
}

# t09 - the fail-safe direction. An unreadable or absent owner file is NOT provably ours, so
# release leaves it. A leaked lock self-heals at the stale threshold; a wrongly-deleted one
# corrupts a live install. Tidiness loses to safety here, deliberately.
t09_unverifiable_owner_is_left_alone(){
  local d="$TMP/t09"; mkdir -p "$d/.cyberos/.install.lock"   # no owner file at all
  local blk; blk="$(sed -n '/^CY_LOCK=/,/^_cy_lock_acquire$/p' "$root/tools/install/install.sh" | sed '$d')"
  CY="$d/.cyberos" bash -c "set -uo pipefail
CY=\"$d/.cyberos\"
$blk
_cy_lock_held=1
_cy_lock_stamp=\"pid=1\"
_cy_lock_release" >/dev/null 2>&1
  if [ -d "$d/.cyberos/.install.lock" ]; then ok t09_unverifiable_owner_is_left_alone
  else no t09_unverifiable_owner_is_left_alone "released a lock whose ownership could not be read"; fi
}

# t10 - and the release MUST still work when it IS ours, or the fix has simply broken the lock.
t10_own_lock_is_released(){
  local d="$TMP/t10"; mkdir -p "$d/.cyberos/.install.lock"
  local stamp; stamp="$(printf 'pid=1\nstarted_at=x\nhost=A\n')"
  printf '%s\n' "$stamp" > "$d/.cyberos/.install.lock/owner"
  local blk; blk="$(sed -n '/^CY_LOCK=/,/^_cy_lock_acquire$/p' "$root/tools/install/install.sh" | sed '$d')"
  CY="$d/.cyberos" bash -c "set -uo pipefail
CY=\"$d/.cyberos\"
$blk
_cy_lock_held=1
_cy_lock_stamp=\"$stamp\"
_cy_lock_release" >/dev/null 2>&1
  if [ ! -d "$d/.cyberos/.install.lock" ]; then ok t10_own_lock_is_released
  else no t10_own_lock_is_released "refused to release a lock that IS ours - the lock now leaks forever"; fi
}

t01_concurrent_refuses; t02_stale_broken_with_warning; t03_fresh_dead_pid_refuses
t04_trap_releases_on_signal; t05_uninstall_lock_ownership; t06_non_contention_failure_named
t07_foreign_host_pid_is_alive; t08_stale_break_handoff_is_not_released; t09_unverifiable_owner_is_left_alone; t10_own_lock_is_released
echo "  ---"; echo "  $pass passed, $fail failed"
[ "$fail" -eq 0 ]
