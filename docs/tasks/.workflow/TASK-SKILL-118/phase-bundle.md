# TASK-SKILL-118 phase bundle

## repo-context-map (step 1)
Gap inventory via the new checker itself (completeness authority per §6): 86 missing files across
7 pairs + 2 spike acceptance READMEs. The four reference pairs (task, implementation-plan,
architecture-decision-record) pass clean - parity defined by pointing at them, per §2. Conventions
copied from task-author/-audit: PIPELINE = chain-binding tables, AUDIT_LOOP = canonical-loop
binding (no duplication), REPORT_FORMAT = frontmatter + findings-by-rule-id, envelopes = json-schema-lite.
Scope note (newest wins, recorded): checker SCOPE includes debugging-cycle (vendored thin by
TASK-SKILL-116) and architectural-spike (TASK-SKILL-117) - 8 pairs raised, not 6; TASK-CUO-209 AC 5's
"scope list grows" clause anticipated exactly this.

## edge-case matrix (step 5) -> covering check
NULL: pair dir absent in trimmed payload -> checker skips (presence is chain-coverage's job) - build on
  /tmp/payload-118 full profile + t07 reduced fixture upstream. BOUNDS: rubric header must sit in head-5
  (t03). MALFORMED: unreadable skills dir -> exit 2 (checker guard). RACE: none (read-only checker).
SECURITY: none new (docs + read-only bash; no injection surface - checker takes one dir arg, no eval).
DEGRADATION: file-class policy drift -> arrays at top of checker ARE the policy, changeable only via
  this task (§10 #3); t05 proves a deleted RUBRIC.md exits 10 by name.

## implementation (steps 6-14)
86 files generated: per author PIPELINE/INVARIANTS/envelopes(2)/FAILURE_MODES/acceptance-README; per
audit RUBRIC/AUDIT_LOOP/REPORT_FORMAT/envelopes(2)/acceptance-README; spike pair acceptance READMEs.
Rubrics encode task §1 #3 gates verbatim with prose->rule tables (AC 2): RCM-/ECM-/MCT-/OBS-/BSU-/COV-/DBG-
prefixes. BSU rubric versioned backlog_state_update_rubric@2.0 (NOT @1.0): TASK-CUO-205 landed first and
already bumped the artefact to @2 with BSU-INS-001..005 in SKILL.md prose - this task migrates them to the
file form its §7 anticipated ("rules land in SKILL.md prose and migrate"). Deviation from §1 #2's "@1.0"
literal recorded here per newest-wins doctrine.
SKILL.md changes: one additive "Contract files (TASK-SKILL-118)" section per skill, appended at EOF -
descriptions, artefact sections, trigger tests untouched (t04 proves zero removed lines; t06 proves
TRIGGER_TESTS.md byte-identical). check-pair-parity.sh + test_pair_parity.sh (t01-t06) + build.sh hookup
after chain-coverage (§1 #6 ordering).

## observability (step 15)
Checker output greppable: `PARITY <skill>: missing <file>` / `parity OK: N author dirs scanned, scope K
pairs`; exit 0|10|2 contract in header comment. Build log now carries both chain + parity lines per build.

## code review vs §1 (steps 16-18)
#1 author classes x6(+dbg) PASS (t01); #2 audit classes PASS (t01); #3 prose gates as rule IDs PASS (t02);
#4 named constants + TASK-CUO-207 hook PASS (t03); #5 artefact shapes untouched PASS (t04 additive-only);
#6 checker + build.sh propagation PASS (t05 + live build); #7 trigger contracts byte-stable PASS (t06).
Secret scan: none. Injection: checker quotes all paths, no eval. Backwards compat: @1 artefacts unchanged;
BSU @2 transition window intact.

## coverage gate (steps 21-29)
test_pair_parity.sh 6/6 (one per AC). Full regression: 6/6 cyberos-init suites (incl. new suite),
ship_manifest 8/8, full-profile build green with parity gate live (52 skills, plugin 1.09MB < 2MB).
ECM rows each name their covering check inline above. tests_failed=0; files_below_90pct n/a (bash+docs task,
no python touched); ecm_rows_uncovered=[].

## ship-manifest dogfood (TASK-CUO-206 in production)
docs/tasks/.workflow/TASK-SKILL-118.ship.json live: initialized at step 1 with task_sha256 +
workflow_version 2.4.0, updated after implementation chunks (steps 1/5/8 recorded, current_step=16),
confirmed gitignored via git check-ignore. First real manifest-tracked ship run.
