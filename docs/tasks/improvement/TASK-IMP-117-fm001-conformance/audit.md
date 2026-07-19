---
audited_file: docs/tasks/improvement/TASK-IMP-117-fm001-conformance/spec.md
audited_file_sha256: 25898022e5d4a865
audited_body_sha256_prefix: 1467a30b176cb219
rubric: audit_rubric@2.0
audited_at: 2026-07-18T16:00:00+07:00
auditor: claude-opus-4.8
verdict: pass
score: 10/10
machine_floor: task-lint 0 errors, 1 info (TRACE-001)
---

# Audit - TASK-IMP-117 (re-audit after nested-map amendment, 2026-07-18)

Re-audit of the spec after it was amended to honestly cover FM-001's SECOND structural class
(nested maps) alongside the original trailing-comment class. The prior audit (2026-07-17, body
`547a9a53e34784f1`) judged the trailing-comment-only spec; this one judges the amended spec (body
`1467a30b176cb219`). The change of `audited_body_sha256_prefix` is expected: the spec's normative
half changed when clauses 1.8/1.9, AC7, and edge rows 16-21 were added.

Machine floor ran first per TASK-IMP-084. Command and result (re-derived, not recalled):

```
$ node tools/install/docs-tools/task-lint.mjs docs/tasks/improvement/TASK-IMP-117-fm001-conformance/spec.md
info TRACE-001 .../spec.md:1 no numbered '- 1.N' clauses under '## 1. Description' ...
$ echo $?            # 0 → zero error-severity findings
```

0 errors, 1 TRACE-001 **info**. The info is by design and unchanged from the prior audit: the clause
block is `## 1. Clauses` with `1.N` numbering rather than `## 1. Description` with `- 1.N` bullets, so
the structural lint hands clause traceability to this model audit, which follows.

## What I verified vs. reconstructed

Marked honestly per the standing session finding (authors do not check what they originate).

**Verified by re-run this session (command behind every number):**
- FM-001 corpus count is **4005** at HEAD `8dd0ca2f`: `node task-lint.mjs --json docs/tasks` parsed
  and filtered on `rule_id === "FM-001"` → 4005 findings across 141 files; message split = **4004**
  "indented line outside a block list" + **1** "trailing comment after value".
- **140** specs carry `^build_envelope:` (`grep -rl` → 140); they are exactly the 140 indented-line
  files. The 1 trailing-comment residual is `TASK-SKILL-104:63` (the apostrophe edge).
- The nested-map flatten, collision union, done-spec body-hold, and apostrophe fix were demonstrated
  EMPIRICALLY on scratch git repos before this audit: a 6-child `build_envelope` → 6 flat top-level
  keys with byte-identical values, FM-001 11→0, body hash held, idempotent; a `new_files` collision →
  order-preserving union (shared item deduped, nothing unique dropped), FM-001 6→0, FM-003 stays 0;
  the `broker's ... #4` plain scalar → ` #4` moved own-line, FM-001 1→0, while `label: 'issue # 42
  stays'` (value begins with a quote) stayed data.
- A `--check --json` dry run over `docs/tasks/*/*/spec.md` reports **141 would-migrate (140 flatten +
  1 comment-only), 411 clean, 0 refused**, and no file flattens more than one nested map — so the
  amended §1.8/§1.9 remit matches the real residual exactly.
- Binding disjointness: the 140 `build_envelope` dirs and the dirs carrying an
  `audited_body_sha256_prefix` binding are DISJOINT (`comm -12` of the two sorted dir sets → empty);
  `TASK-SKILL-104` is not audit-bound. So the migrator is a byte-for-byte no-op on every bound spec.

**Reconstructed (cited from the investigation / commit history, NOT re-run by me):**
- The pre-migration trailing-comment counts (2104 → 1) and the "497 specs migrated by `4c02b556`"
  figure come from `docs/tasks/_audits/2026-07-18-fm001-nested-map-fork.md`, which backs them with
  commands; I did not reconstruct the pre-`4c02b556` corpus state.
- The origin-era "501 of 544" and "149 done" figures in §Problem / §AI Authorship Disclosure are the
  task's originating measurements (trailing-comment class); I preserved them as history and did not
  re-run them. The current two-class reality (4005 across 552 specs) is what the amended sections and
  AC7 state and what I re-derived.

## Findings

ISS-001 (accepted, TRACE-001 info): as above — heading shape, not a traceability gap. Every clause
1.1-1.9 cites a named test; every AC1-AC7 cites a clause or a test.

ISS-002 (accepted, honest correction — AC5's "40"): AC5 cites "40 audit-bound specs". The current
measured count is **19** (`grep -rl '^audited_body_sha256_prefix:' docs/tasks/` → 19 audit.md files,
IMP-102..120). The "40" is the spec's origin-era figure; I did not edit AC5 (amending it is outside
this task's nested-map remit and would touch a class-1 clause), but I record the true count here. The
INVARIANT AC5 asserts — no binding changes across the migration — holds regardless of the count and
is now DOUBLY assured: §1.4 (body untouched) AND disjointness (bound specs carry neither class). Only
IMP-117's OWN binding changes, and that is this re-audit, not the migration.

ISS-003 (resolved — the honest-remit gap the amendment closes): the pre-amendment spec framed FM-001
as only the 501 trailing-comment findings and its §Out-of-scope was SILENT on nested maps, so AC4's
"FM-001 = 0 corpus-wide" was UNREACHABLE by the AC1-AC6 migrator (which never modelled nested maps).
Resolved: §Summary/§Problem now name both classes; §1.8 adds the general nested-map flatten; §1.9
fixes the apostrophe quote model; AC7 binds the 140-spec flatten to a real-corpus FM-001==0 check;
§Out-of-scope now explicitly REJECTS relaxing FM-001 (route b) and takes route (a). No existing
clause or AC was weakened — 1.3's quoted-value protection is explicitly preserved by 1.9.

ISS-004 (accepted — §1.8 reconcile is specified, not hand-waved): the collision path names its exact
policy (order-preserving union for lists, dedupe-equal for scalars, HALT-and-name for a genuine
scalar/kind conflict) and binds it to edge rows 17 (union) and 19 (halt), both cited to t09. The two
real collision files (PLUGIN-003, TEN-002) are `new_files` list-vs-list unions; the corpus has zero
scalar collisions (`top-level language/service/modified_files/allowed_tools/disallowed_tools` = 0
across the 140), so the HALT path is a guard, not a corpus need — correctly specified anyway.

ISS-005 (accepted — body-binding mechanism): §1.4 (body never touched) is the mechanism that lets a
`done` `build_envelope` spec be flattened without moving its normative-half hash (edge row 18). Only
14 of the 140 are `done` and none is audit-bound, so this is belt-and-suspenders; the clause is
correct and the property was demonstrated live on the scratch done-spec case.

ISS-006 (accepted — idempotence across BOTH passes): edge row 21 binds "flatten output re-run =
byte-identical no-op" to t05+t09. Demonstrated on scratch: a second run after flatten is `cmp`-clean.
Idempotence now spans the flatten pass (hoisted keys re-parse as plain block lists / scalars) and the
comment pass (own-line comments are skipped by the `^\s*#` guard).

## Rubric families

- **FM:** clean (machine floor: 0 errors). FM-001/002/003/004 pass; all per-field FM-1xx present and
  in-enum. The amendment touched only the body, so the frontmatter verdict is unchanged.
- **SEC:** all seven required H2s present and non-empty (Summary, Problem, Proposed Solution,
  Alternatives Considered, Success Metrics, Scope, Dependencies). One H1, no level jumps (SEC-009).
- **COND:** `ai_authorship: generated_then_reviewed` → COND-004 satisfied by the three-bullet
  disclosure (Tools used / Scope / Human review). `client_visible: false`, `eu_ai_act_risk_class:
  not_ai` → COND-001/002/003 not triggered.
- **QA:** metrics carry baseline+target+source (FM-001 4005→0, 140 specs, all re-derived — QA-004/007
  clean); ≥2 distinct Alternatives (QA-005); §Scope has `### Out of scope / Non-Goals` with multiple
  bullets (QA-006). Edge-case matrix is 21 rows spanning NULL/EMPTY, BOUNDS, MALFORMED, CONCURRENT,
  SECURITY, DEGRADATION, and the new NESTED MAP category — well above the 8-row floor.
- **SAFE:** no `<untrusted_content>` blocks and no injection markers; the spec quotes tool paths and
  guard names, no foreign bytes.
- **TRACE (semantic, this gate's job):** every §1 clause carries a BCP-14 verb and a cited test, and
  the cited test's specified assertion discharges the verb — 1.8 (flatten/hoist/preserve/reconcile/
  HALT) → t09 asserts flatten-to-top-level, byte-equal values, union-nothing-dropped, and refuse-on-
  conflict; 1.9 (detect ` #` as comment and move own-line, keep quoted `#` as data) → t10 asserts the
  moved comment, the preserved value line, and the untouched quoted value. TRACE-004 (each cited test
  actually PASSES) and TRACE-006's live pass/fail are the coverage gate's job at `testing → done`;
  this spec-correctness gate confirms the citations and the verb→evidence match, both of which hold.

## Verdict

**pass - 10/10.** The amendment makes the spec honest about what the migrator must do: FM-001 has two
structural classes, and the spec now specifies and tests BOTH. It adds no rule and invents no guard —
§1.8 hoists to a shape a `done` sibling already uses and §1.9 aligns the migrator's quote model with
task-lint's. AC4's corpus-wide FM-001==0 is now genuinely reachable, and AC7 binds it to a re-derived
real-corpus check. No existing clause or AC was weakened.
