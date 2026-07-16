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

E2 - AC 5 ops check (run_all glob discovery): scripts/tests/run_all.sh:43 is
`for t in scripts/tests/test_*.sh tools/docs-site/tests/test_*.sh tools/install/tests/test_*.sh; do`
and `tools/install/tests/test_*.sh` expands to include
`tools/install/tests/test_memory_append.sh` (8th entry of the tools/install glob at
check time) - the suite is discovered with zero wiring.

E3 - vendor line: tools/install/build.sh:178-179 (guarded copy, sibling idiom):
```
  # memory-append: doc-driven appender for the BRAIN audit chain (TASK-IMP-093)
  [ -f "$here/docs-tools/memory-append.mjs" ] && cp "$here/docs-tools/memory-append.mjs" "$out/docs-tools/"
```
t04 gates payload presence, byte-parity with the source, --help of the vendored copy,
and a live append+verify lifecycle against a scratch store.

E4 - cross-implementation spot check (recorded during implementation, not part of the
suite): an independent python walk of a tool-written store (struct '>IIQQ' frame parse,
json sorted-keys compact recompute, raw-byte prev_chain concat) reported every chain
link OK and HEAD == tip seq; the crc32c implementation matches the Castagnoli test
vector (crc32c("123456789") == 0xE3069283).
