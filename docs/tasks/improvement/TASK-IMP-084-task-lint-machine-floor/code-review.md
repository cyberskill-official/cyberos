# TASK-IMP-084 — code review packet

Files under review: new `tools/install/docs-tools/task-lint.mjs` (the lint) and `tools/install/tests/test_task_lint.sh` (the gating suite), modified `modules/skill/task-audit/SKILL.md` (+4 lines, §1 #1.8 wiring) and `tools/install/build.sh` (+2 lines, vendor copy — disclosed below). Suite state at review: test_task_lint 8/8, 0 failed (~2 s including payload build + scratch install). Other dirt in the same working tree (`tools/docs-site/*`, `tools/install/install.sh`, `tools/install/uninstall.sh`, `tools/install/tests/test_install_hygiene.sh`, `scripts/tests/test_render_stamp.sh`) belongs to batch siblings TASK-IMP-082/083 and is covered by their own packets.

## §1 clause → proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | one or more spec paths or directories (dirs recurse to `*/spec.md`); node stdlib alone | `t01_cli_and_determinism` — single file, dir recursion finding nested green+red spec.md, no-args usage exit 2. Stdlib: the import block is `node:fs` + `node:path` only (task-lint.mjs:31-32), no child_process/net/eval; `t07_payload_and_install` runs the VENDORED copy standalone inside a scratch repo where no node_modules exists |
| 1.2 | FM family mechanical: FM-001 fences+strict-subset parse, FM-002 snake_case, FM-003 dups, FM-004 template, FM-101..111 field rules (FM-109 rejecting `unacceptable`), FM-112 marker, FM-113 duplicate_of iff duplicate + resolution, FM-114 severity iff bug | `t02_fm_family` — six red fixtures, each asserted to yield EXACTLY one error with its rule_id: anchor `&keep` → FM-001 naming the mutated line (grep -n cross-check), `p9` → FM-105, own-line `# UNREVIEWED` → FM-112, severity on a chore → FM-114, 80-code-point title → FM-101, bare `tester` → FM-102, template deleted → FM-004 `template_ambiguous` with the exactly-one-error assert proving the per-file STOP. Enums/regexes sit verbatim at task-lint.mjs:36-45 (12-value FM-104 set incl. `cannot_reproduce`/`duplicate`); FM-109's Article-5 reject, FM-111's quoted-ness check, and FM-113's docs/tasks/** name resolution are code-pinned (`checkFrontmatterFields`) and were exercised by the recorded red smoke (10 distinct FM rule_ids on one hostile fixture) |
| 1.3 | SEC mechanical: SEC-001..007 required H2s, SEC-008 non-empty, SEC-009 warning severity | `t03_sec_family` — Summary section deleted → exactly SEC-001 naming Summary; `Something.` deleted → exactly SEC-008 naming Problem. SEC-009 emits at warning severity (`checkSections`; recorded smoke shows `warning SEC-009 ... H2 to H4` while exit stays driven by errors only) |
| 1.4 | COND triggers: 001/002 on client_visible true, 003 with ordered H3s on limited/high, 004 with three labeled bullets on ai_authorship != none | `t04_cond_family` — `assisted` with no disclosure → exactly COND-004; `client_visible: true` with a Sales/CS Summary appended (so COND-002 stays green) and no Customer Quotes → exactly COND-001, proving trigger isolation. COND-003's in-order H3 walk and COND-001's `<untrusted_content` interior requirement are code-pinned in `checkConditionalSections` |
| 1.5 | TRACE structural halves: TRACE-001 clause cited via `§1 #N`/`#1.N`/traces_to (deferred-slice exempt), TRACE-002 test:/verify: presence, TRACE-003 backticked test paths in new_files or on disk; semantics stay with the model | `t05_trace_family` — uncited `- 1.2 ... MUST` → exactly TRACE-001 naming 1.2; AC stripped of its entry → exactly TRACE-002; `no/such/dir/test_x.sh::t01` → exactly TRACE-003 naming the path; the green verify: AC exits 0 (the justified-ops form passes the structural half by design); a test: path listed in new_files but absent on disk exits 0 (the will-be-authored branch); zero clauses → `info TRACE-001`, exit 0 |
| 1.6 | findings `SEVERITY rule_id file:line message`, bytewise sorted; `--json` same findings; byte-identical runs | `t01_cli_and_determinism` — two text runs and two `--json` runs over the same mixed dir are cmp-identical, and the JSON parses (`node -e JSON.parse`). Sorting is a plain code-unit sort over the formatted lines; JSON emits the identical findings in the identical order (`main`, task-lint.mjs) |
| 1.7 | exit 0 with no error findings, 2 otherwise; unreadable or non-task@1 → `template_ambiguous` at error severity, never a guess | `t01_cli_and_determinism` — green file exit 0 with EMPTY output, red exit 2, missing path → exit 2 with `template_ambiguous`; `t02_fm_family` FM-004 arm covers the absent-template detection; the info-severity note in t05's zero-clause arm proves non-errors never flip the exit |
| 1.8 | SKILL.md gains one normative loop step: lint runs FIRST when present (both install shapes named), seeding mechanical findings; model audits judgment families only | modules/skill/task-audit/SKILL.md §3 "Machine floor first (TASK-IMP-084)" — two paragraphs, names both paths, "MUST run it FIRST", seeds FM/SEC/COND/structural-TRACE from rule_id output, reserves QA/SAFE/TRACE-semantics/XCHAIN/STALE for the model, and states the floor-not-replacement rule with the 10/10 verdict. Gated by `t08_skill_wiring_present` in all three copies (modules/, payload cuo/skills/, payload plugin/skills/) |
| 1.9 | gating suite at `tools/install/tests/test_task_lint.sh`: batch specs pass; one fixture per family yields exactly its rule_id + exit 2; payload carries the tool; scratch install lays it into `.cyberos/docs-tools/` | the suite itself — `t06_green_corpus` (three specs exit 0, zero findings), t02/t03/t04/t05 (per-family exact-rule_id fixtures via `expect_one_rule`), `t07_payload_and_install` (build → payload `docs-tools/task-lint.mjs` non-empty AND cmp-identical to source → guarded install into a scratch git repo → `.cyberos/docs-tools/task-lint.mjs` present → the installed copy actually runs). Discovered by scripts/tests/run_all.sh:43's `tools/install/tests/test_*.sh` glob with zero wiring |

## Acceptance criteria

AC 1 `t01_cli_and_determinism` ok · AC 2 `t02_fm_family` ok · AC 3 `t03_sec_family` ok · AC 4 `t04_cond_family` ok · AC 5 `t05_trace_family` ok · AC 6 `t06_green_corpus` ok (TASK-IMP-082/083/084 specs: exit 0, zero findings — no genuine spec defects surfaced) · AC 7 `t07_payload_and_install` ok · AC 8 `t08_skill_wiring_present` ok. Suite 8/8.

## Diff size

Two new files: `tools/install/docs-tools/task-lint.mjs` (588 lines, self-contained ESM, node stdlib only) and `tools/install/tests/test_task_lint.sh` (289 lines, executable). Two modified files, +6/−0 total: `modules/skill/task-audit/SKILL.md` +4 (the §3 machine-floor passage) and `tools/install/build.sh` +2 (guarded vendor copy in the docs-tools block). No dependency added anywhere. `dist/` untouched here — rebuild, version-sync and full suite before commit are the batch parent's step per payload-sync doctrine.

## build.sh modification (disclosure)

The spec's `modified_files` lists only SKILL.md, and its source_pages row reads "build.sh:165-171 (docs-tools vendors what exists — a new .mjs lands in the payload automatically)". That block in fact copies NAMED files under `[ -f ] &&` guards — nothing globs the docs-tools source dir — so without a change the payload could never carry the lint and §1 #1.9 / AC 7 would be unimplementable. Changed minimally in the block's own idiom: one comment line + one guarded copy (build.sh:173-174). t07 pins both halves (payload presence, byte-parity with the source) so the vendoring can no longer silently regress. The sibling `install.sh:61` dir-copy needed no change and was not touched.

## Verdict

| Check | State |
|---|---|
| §1 clauses 1.1–1.9 | each proven above by a named test or pinned line |
| Guardrail metric (per-family fixture → exact rule_id + exit 2, every run) | pass (`expect_one_rule` asserts count==1 AND id) |
| Primary metric (mechanical families machine-executed; batch specs no false errors) | pass (t06: 3/3 exit 0, zero findings) |
| Determinism contract (byte-identical, no clock/env) | pass (t01, text + json) |
| build.sh ripple | disclosed; 2 lines, vendoring idiom preserved, gated by t07 |
| Invariants (§5: floor-not-replace, stdlib-only, payload doctrine, HITL) | intact |

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
