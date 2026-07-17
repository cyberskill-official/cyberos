# phase: corpus measurement (2026-07-18)

Evidence file for TASK-IMP-121/122/123. Written because three specs shipped **four false counts**
derived from greps of the `improvement` module reported as corpus-wide facts. Every number below
is script output, cross-validated against an independent adversarial audit that reached the same
totals by a different method. **Specs must cite this file, not a recollected grep.**

Measured at HEAD `e2504cf3`. Reproduce: `node docs/tasks/_audits/measure-phase.mjs "$(pwd)"`.

## Totals

| metric | value |
|---|---|
| specs total | **551** |
| carry `phase` | **531** |
| missing `phase` | **20** |
| `done` total | **183** |
| `done` carrying `phase` | **176** |
| `done` missing `phase` | **7** |
| distinct `phase` values | **31** |
| distinct vocabularies | **5** |

Cross-validation: an independent auditor reached 551 / 531 / 20 / 183 / 176 / 31 / 5 by separate
greps. Both methods agree on every headline. (Bucket counts differ slightly - the auditor's
per-vocabulary numbers sum to 528, this classifier's sum to 531; this file's are used because they
reconcile to the total.)

## The five vocabularies (the spec claimed TWO)

| vocabulary | values | specs | examples |
|---|---|---|---|
| P-number | 6 | 330 | `P0` .. `P5` |
| Wave-numeric | 12 | 76 | `Wave 1 - see and survive`, `Wave 6 - go-live (Track B: mobile shells)` |
| Phase-prose | 5 | 66 | `Phase 0 - safety rails` .. `Phase 4 - product depth, ecosystem, scale` |
| release-gate | 3 | 38 | `pre-1.0.0 release`, **`pre-1.0.0 hardening`**, `post-1.0.0` |
| Wave-lettered | 5 | 21 | `Wave A - version coupling` .. `Wave E` |

**The module split asserted by TASK-IMP-123 is false.** The `improvement` module carries THREE
vocabularies, and release-gate is the MINORITY within it:

```
improvement specs = 123   with_phase = 119
its vocabularies  = {"Wave-numeric":76, "Wave-lettered":5, "release-gate":38}
```

So "improvement tasks use pre-1.0.0/post-1.0.0" is wrong even about `improvement`: 81 of its 119
phase-carrying specs use Wave prose. `pre-1.0.0 hardening` is a third release-gate value the spec
did not know existed. And `render-module-changelog.mjs:39` hardcodes `ten: {phase:'P4'}` while
TEN's tasks carry P2/P3/P4 - the "matches the module map" inference fails on its own exemplar.

## The 20 specs missing `phase` (the spec claimed FOUR)

```
TASK-AI-105       draft              TASK-CUO-202      done
TASK-APP-001      implementing       TASK-CUO-203      done
TASK-CHAT-102     draft              TASK-CUO-204      done
TASK-CHAT-103     draft              TASK-CUO-301      done
TASK-CHAT-104     draft              TASK-DOCS-002     done
TASK-CHAT-105     draft              TASK-IMP-117      ready_to_implement
TASK-CHAT-106     draft              TASK-IMP-118      ready_to_implement
TASK-CUO-200      done               TASK-IMP-119      ready_to_implement
TASK-CUO-201      done               TASK-IMP-120      ready_to_implement
TASK-MEMORY-261   draft              TASK-MEMORY-302   draft
```
IMP-117..120 are four of twenty, across seven modules. The other 16 were invisible to a grep
scoped at `docs/tasks/improvement/`.

## Who reads `phase`: SIX frontmatter-derived readers (spec claimed 2; audit found 5)

| file | what it does | frontmatter-derived? |
|---|---|---|
| `tools/docs-site/data-extract.mjs:141` | `phase: fm.phase \|\| ''` -> `data/tasks.json` | YES - the source |
| `tools/docs-site/nfr-extract.mjs:161` | `phase: fm.phase \|\| ''` -> `data/nfrs.json` | YES - the source |
| `tools/docs-site/render-task-catalog.mjs:43,54` | `data-phase` attr + badge | YES |
| `tools/docs-site/render-status-hub.mjs:319,548,550,559` | `new Set(...)`, filter `sel('ph','phase')`, group-by, `<th>phase</th>` | YES - not a badge |
| `tools/docs-site/render-nfr-catalog.mjs:53,59,89,217-282` | phase chips + `filterPhase` | YES |
| **`modules/templates/html/status-app.js:32-34,77,141,164,255`** | **filter key `ph`, search index, sortable column, `<td>`, detail row** | **YES - MISSED BY BOTH the spec AND the independent audit; it is outside `tools/docs-site/`** |

False positives (the word `phase`, not a reader of task frontmatter):
- `tools/docs-site/render-module-changelog.mjs:17-39` - `MODULE_META` hardcodes a **module** phase
  (`ai: {phase:'P0'}`). Never read from a task. A **third distinct meaning** of the word in-tree.
- `tools/install/tests/test_memory_append.sh:8,20` - prose ("two-phase tmp", "ship-tasks phases").
- `tools/install/docs-tools/coverage-scope.mjs:224` - writes `phase: testing` into coverage-gate
  artefacts. A **fourth** meaning; out of reach of task-lint (which recurses to `*/spec.md` only).

## Consequences for the specs

1. **TASK-IMP-123 §1.3** ("the two live vocabularies reconciled") is normative against a false
   denominator. Five vocabularies / 31 values / 531 specs. `effort_hours: 4` is sized against two.
2. **TASK-IMP-123 §1.5** ("removed from every renderer that reads it") is scoped against 2 of 6.
   `status-app.js` is the file from HANDOFF §15.1's defect - the one that "has zero references to
   the key". It has five.
3. **TASK-IMP-123's cone** declares 4 files and rewrites 531 + 6 renderers. This is TASK-IMP-117's
   recorded mistake, quoted in `batch-select.mjs:75-82`, the file the task modifies.
4. **The word `phase` has four meanings in-tree**: task release-gate/wave, module rollout wave
   (MODULE_META), coverage-gate lifecycle stage, and the client-side filter key `ph`. Any removal
   or enum must name which one it means.

## Method note

The failure this file exists to prevent: a grep scoped to one module, reported as a corpus fact,
with no denominator stated. Four counts were wrong in one spec (2 vs 5 vocabularies, 4 vs 20
missing, 183 vs 176 done-with-phase, 2 vs 6 readers). Every one would have passed task-lint, and
did. The floor cannot see a false measurement; only re-measurement can.

---

# ADDENDUM: the comparable-set decision (TASK-IMP-122 NEW-001)

The rewrite of TASK-IMP-122 failed at 6/10 on a design defect: it conflated the PAYLOAD cone with
the INSTALLED cone. Measured here so the next rewrite has the answer rather than an assumption.

## Measured: depth-1 dirs, payload vs installed

| dir | in payload | installed | verdict |
|---|---|---|---|
| `cuo` | yes | yes | COMPARABLE |
| `plugin` | yes | yes | COMPARABLE |
| `mcp` | yes | yes | COMPARABLE |
| `memory` | yes | yes | COMPARABLE (minus `memory/store/`, install-generated) |
| `docs-tools` | yes | yes | **COMPARABLE - and NOT in today's cone** |
| `lib` | yes | yes | **COMPARABLE - and NOT in today's cone** |
| `ci` | yes | **no** | distribution-only |
| `cli` | yes | **no** | distribution-only - **and IS in today's cone** |
| `template` | yes | **no** | distribution-only |

```
cone today (build.sh:354)  : find cuo plugin mcp cli memory
comparable (vendored set)  : cuo docs-tools lib mcp memory plugin
  in cone, never installed : cli          -> the cause of the false drift
  installed, not in cone   : docs-tools lib -> the blind spot (4 of the 6 measured drifts)
```

## The decision

**The cone is the VENDORED set, not the SHIPPED set.** The question the check answers is "is this
installed machine the machine this payload would install?" - so the cone is what install installs.
Derived from `install.sh:184-195`, not from `ls dist/cyberos/*/`.

    cone := cuo plugin mcp memory docs-tools lib
          + the vendored root scripts (install.sh uninstall.sh version.sh status.sh help.sh
            check-latest.sh - all `cp`'d at :190-195 and covered by NO cone today)
          - memory/store/          (install-generated; payload has 3 files, install has 8)
          - gates.env, config.yaml, .update-check-cache   (install-generated / operator-owned)
          - manifest.yaml          (MUST be excluded: it CONTAINS rules_sha, so hashing it is
                                    circular - the value would depend on itself)
          - VERSION                (TASK-IMP-104's axis, not this one)

So TASK-IMP-122's §1.3 ("the cone MUST cover every directory the payload SHIPS") is wrong and is
what forced `ci/`, `cli/`, `template/` in and guaranteed self-drift. The correct clause is "every
path install VENDORS". The build-time check derives its expected set from install.sh's copy steps,
not from a directory listing of the payload.

## Consequences for the next rewrite

- §1.2 "same cone as build.sh" survives only if build.sh's cone is FIXED first (drop `cli`, add
  `docs-tools`, `lib`, the root scripts). The build cone and the compare cone must be one list,
  defined once.
- §1.6 "byte-identical" and AC 7 "freshly installed" become the same proposition once the cone is
  the vendored set - which is what makes that prior finding closable rather than a wording fix.
- §1.5 "name every differing path" is still unsatisfiable against a tree hash (build.sh:355 does
  `| _rsha | cut`, discarding per-file digests) and audit-fleet.sh has only a bare token, no tree.
  Unresolved: either §1.5 drops to "report drift, and name paths WHERE A PAYLOAD TREE IS
  REACHABLE", or the design changes. Still an open operator decision.
- §1.1 must drop `check-version-sync.sh`: `grep -c '\.cyberos'` = 0; it compares payload stamps to
  the root VERSION and has no installed side at all.

---

# FINDING: one field, four questions - the status filter mixes incomparable values

Filed 2026-07-18 (operator decision). TASK-IMP-123 was DROPPED: `phase` is a live reporting
dimension that works, its scheduling harm is already fixed by `d19362ad` (release intent moved
into `priority`, the field that schedules), and removing it provably does NOT unblock the queue -
`TASK-IMP-106` excludes ten tasks today with `phase` playing no part. The serialisation is
service-cone subsumption (10 of 11 tasks touch `tools/install`), which is TASK-IMP-119's ground.

**What IS worth fixing, and it is not scheduling.** `phase` is read by six consumers - including
`render-status-hub.mjs` (filter facet + group-by + sortable column) and
`modules/templates/html/status-app.js` (filter key `ph`, search index, `<td>`, detail row). It is
how the corpus is sliced on the status page. But one dropdown now mixes five vocabularies:

    P0 | Wave 3 - widen the envelope | Phase 0 - safety rails | pre-1.0.0 release | Wave C

Those are not alternatives to one another. They are four different questions sharing one field
name: which module rollout wave · which programme wave · which programme phase · which release
gate. Grouping by `phase` therefore groups incomparable things, and filtering to `P0` silently
excludes every Wave/Phase/release-gate task rather than narrowing within a dimension.

Severity S3, reporting only. No scheduling impact - nothing reads it for scheduling, which is the
whole point. Scope if picked up: this is a corpus migration (531 specs, 31 values), so size it from
the measurement above, NOT from an estimate. The word `phase` also carries two further meanings
in-tree that any migration must not collide with: `MODULE_META.phase` (a module attribute
hardcoded in `render-module-changelog.mjs:17-39`, never read from a task) and
`coverage-scope.mjs:224`'s `phase: testing` in coverage-gate artefacts.

Not authored as a task. Recorded here so the next person has the measurement and the reasoning
rather than a 4-hour estimate derived from a false count.
