# TASK-CUO-302 — implementation evidence

Implementer: batch/8-audit-hardening worker (gates + install/portability lane), 2026-07-23.
Status frontmatter untouched per shared-tree rules; HITL gates remain the operator's.

## What changed and why

| File | Change | Traces to |
|---|---|---|
| `tools/install/gates/run-gates.sh` | All-empty floor now exits **3** with an actionable RED message (names `.cyberos/config.yaml` `gates.*`, re-running install, and `CYBEROS_ALLOW_EMPTY_GATES` by exact name). `CYBEROS_ALLOW_EMPTY_GATES=1` (the literal `1` only; `true`/`yes`/`0` behave as unset) prints `GATES: EMPTY-ACKNOWLEDGED` — never `GATES: GREEN` — and exits 0. Check evaluates only after the config.yaml layer parsed (malformed config keeps its distinct exit 2). Header comment documents the 0/1/2/3 exit-code contract. | #1.1 #1.2 #1.3 |
| `tools/install/install.sh` | Autodetect gains the monorepo fallback tier for TEST: ordered, closed list — `scripts/tests/run_all.sh` (`SRC_TEST="fallback:run_all"`), then `Makefile` with a `test:` target (`SRC_TEST="fallback:make"`). Placed BEFORE the Makefile stack block so `run_all.sh` outranks `make test` when both exist (the contractual order). Existence probes only; nothing is executed at install time; `ECOSYSTEM` is not polluted (a fallback is not a detected ecosystem). | #1.4 |
| `tools/install/install.sh` | Generated `gates.env` header no longer says "edit freely": it states machine-owned + regenerated-on-every-install + durable overrides belong in `.cyberos/config.yaml`. | #1.5 |
| `tools/install/README.md` | Documents the fail-closed floor, exit-code semantics, the escape hatch (literal-1 rule), and the fallback tier. | #1.2 #1.4 |
| `CHANGELOG.md` | New top `## [Unreleased]` section marks RED-on-empty as **breaking** and names `CYBEROS_ALLOW_EMPTY_GATES=1` as the migration path. | #1.6 |
| `tools/install/tests/test_fail_closed_gates.sh` (new) | t01–t06, one per AC, including the literal-1 negative cases, the sentinel non-execution fixtures, and the run_all-beats-Makefile precedence fixture. | AC 1–6 |
| `tools/install/tests/test_gate_autodetect.sh` (updated) | t07 rewritten: the old AC asserted an advisory "floor only" line above a green exit — superseded by this task (now asserts exit 3 + `EMPTY FLOOR` + config.yaml named). Suite also stubs the doctor-gate import probe (see carve-out below) so it keeps testing autodetect semantics only, deterministically on machines with the memory package pip-installed. | guardrail |

## Verbatim test output

```
$ bash tools/install/tests/test_fail_closed_gates.sh
building scratch payload...
test_fail_closed_gates.sh (TASK-CUO-302)
  ok   t01_empty_floor_exits_red
  ok   t02_red_message_actionable
  ok   t03_ack_line_distinct
  ok   t04_monorepo_fallback_seeds_test_cmd
  ok   t05_header_machine_owned
  ok   t06_changelog_breaking_entry
----
pass=6 fail=0

$ bash tools/install/tests/test_gate_autodetect.sh
  ok t01..t08  (pass=8 fail=0)
```

Scratch-install demo (fresh /tmp repo, canonical rebuilt payload):

```
$ bash .cyberos/cuo/gates/run-gates.sh ; echo rc=$?
...SKIP build/lint/test/coverage... PASS doctor ...
GATES: RED - EMPTY FLOOR: zero gate commands are configured, so nothing was verified and this run cannot be green.
  Fix durably: set gates.build / gates.lint / gates.test / gates.coverage in .cyberos/config.yaml,
  or re-run the install (bash .cyberos/install.sh) so autodetect can seed commands from your repo.
  Genuinely nothing to run (docs-only repo)? Acknowledge it per run: CYBEROS_ALLOW_EMPTY_GATES=1
rc=3

$ CYBEROS_ALLOW_EMPTY_GATES=1 bash .cyberos/cuo/gates/run-gates.sh ; echo rc=$?
GATES: EMPTY-ACKNOWLEDGED - the floor ran nothing (build/lint/test/coverage all empty); CYBEROS_ALLOW_EMPTY_GATES=1 accepted that for THIS run only.
rc=0        # output contains no "GATES: GREEN"

$ mkdir -p scripts/tests && printf '...' > scripts/tests/run_all.sh && bash <payload>/install.sh .
$ grep -E '^(TEST_CMD|SRC_TEST)' .cyberos/gates.env
TEST_CMD="bash scripts/tests/run_all.sh"
SRC_TEST="fallback:run_all"
$ bash .cyberos/cuo/gates/run-gates.sh
gate test: bash scripts/tests/run_all.sh (source: autodetect:fallback:run_all)
PASS  test
GATES: GREEN (machine gates only).   # rc=0
```

## TASK-MEMORY-303 §1.6 carve-out (doctor gate) — implemented here

The memory worker owns TASK-MEMORY-303 but must not touch this lane's files, so the run-gates hook landed with this batch: when `.cyberos/memory/store/` exists AND `python3 -c "import cyberos.core"` succeeds (import probe, never a `$PATH` name — the TASK-IMP-130 lesson), `gate doctor "python3 -m cyberos doctor"` joins the run; doctor FAIL maps to the ordinary gate RED (exit 1); either precondition absent emits one SKIP provenance line and changes nothing. Doctor does NOT count toward the empty-floor check (additive, like caf/awh). `build.sh` also now vendors `memory.schema.json` from the canonical package-data copy `modules/memory/cyberos/data/memory.schema.json` (payload verified StoreAcl-bearing), and vendors/installs `modules/memory/INTEROP.md` when present (guarded `-f`, so the memory worker's new doc ships without them touching build/install).

```
$ bash tools/install/tests/test_doctor_gate.sh
  ok   t01_doctor_gate_three_states      # PASS doctor / FAIL+RED / SKIP no-store
  ok   t02_cli_absent_skips              # store present, CLI not importable -> SKIP
  ok   t03_real_doctor_when_available    # REAL doctor: 16/16 PASS on a fresh scaffolded store
pass=3 fail=0
```

## Deviations / notes for the reviewer

1. **Makefile-only provenance is `fallback:make`, not `make`.** The precedence edge case (run_all beats a `test:`-bearing Makefile) forces the fallback probe to run before the Makefile stack block, so the block's test claim is pre-empted on Makefile-only repos too. Command string and PASS/FAIL semantics are unchanged (`make test` either way); only the `SRC_TEST` provenance string moved. AC 4's own wording ("the Makefile probe seeds `make test`") reads as the fallback list's second arm, which is what now happens.
2. **Acknowledged-empty is exit 0 but deliberately not "GREEN"** (spec #1.3). The batch summary's phrase "GREEN with CYBEROS_ALLOW_EMPTY_GATES=1" is imprecise; the spec prevails.
3. **This repo's own gates remain GREEN-able**: `.cyberos/config.yaml` carries `gates.test: "bash scripts/tests/run_all.sh"` (floor non-empty, so no exit 3), and once installed, autodetect would also seed the same command via `fallback:run_all`. Not exercised end-to-end here because running the full suite is the final pass's job and the machine-local vendored gate script is deliberately not refreshed mid-wave (see open items).
4. **Open item — machine-local activation ordering.** The live store still carries stray `adrs/` + `impl-plans/` (repair is operator-gated inside TASK-MEMORY-303's flow), so refreshing this repo's installed `.cyberos/cuo/gates/run-gates.sh` now would turn sibling gate runs RED via the doctor gate — exactly the ordering TASK-MEMORY-303's edge case forbids (repair before gate wiring). After the store repair lands, run: `CYBEROS_SYNC_HOST_PLUGINS=0 bash tools/install/build.sh && CYBEROS_OFFLINE=1 bash dist/cyberos/install.sh .` — that refreshes the vendored gate script, regenerates `gates.env` with the new header + fallback-seeded TEST_CMD, and vendors `.cyberos/ci/`.
5. **CHANGELOG is a shared file this wave** (the memory worker's task also demands an entry); entries were written additively under a new `## [Unreleased]` section for the final pass to merge/rename at release stamping.
