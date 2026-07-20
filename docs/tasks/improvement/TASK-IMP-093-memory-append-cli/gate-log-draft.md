# TASK-IMP-093 gate-log evidence (implementing -> ready_to_review)

E1 - gating suite (AC 1-4), full run, verbatim:
```
$ bash tools/install/tests/test_memory_append.sh
  ok   t01
  ok   t02
  ok   t03
  ok   t04
test_memory_append: pass=4 fail=0
```

E2 - AC 5 ops check (run_all glob discovery): scripts/tests/run_all.sh:43 is `for t in scripts/tests/test_*.sh tools/docs-site/tests/test_*.sh tools/install/tests/test_*.sh; do` and `tools/install/tests/test_*.sh` expands to include `tools/install/tests/test_memory_append.sh` (8th entry of the tools/install glob at check time) - the suite is discovered with zero wiring.

E3 - vendor line: tools/install/build.sh:178-179 (guarded copy, sibling idiom):
```
  # memory-append: doc-driven appender for the BRAIN audit chain (TASK-IMP-093)
  [ -f "$here/docs-tools/memory-append.mjs" ] && cp "$here/docs-tools/memory-append.mjs" "$out/docs-tools/"
```
t04 gates payload presence, byte-parity with the source, --help of the vendored copy, and a live append+verify lifecycle against a scratch store.

E4 - cross-implementation spot check (recorded during implementation, not part of the suite): an independent python walk of a tool-written store (struct '>IIQQ' frame parse, json sorted-keys compact recompute, raw-byte prev_chain concat) reported every chain link OK and HEAD == tip seq; the crc32c implementation matches the Castagnoli test vector (crc32c("123456789") == 0xE3069283).

## PR-review addendum (2026-07-17, Devin Review x2)

F1 (defect, fixed): the §4.2 lease compared a stored MONOTONIC expiry against a fresh monotonic read - valid within one boot, but a .lock left behind across a host reboot carries an expiry far beyond the reset clock and would wedge appends until cleared by hand. acquireLease now runs two stale-detectors before the expiry comparison: an implausible-horizon guard (expiry more than one TTL ahead = another boot epoch) and a same-host pid-liveness probe (kill(pid,0); ESRCH = orphan). t02 gained both arms (the old 9e18 held-lock fixture was itself boot-skew-shaped and became the skew arm; the held-lock fixture is now a realistic live foreign lease inside one TTL). Suite 4/4.

F2 (info, no change): the chain-field blanking's reliance on sorted-key canonical form + JSON quote escaping was reviewed and affirmed sound ("a malicious/exotic payload cannot forge an earlier match; append/verify are self-consistent"). Recorded, nothing to fix.
