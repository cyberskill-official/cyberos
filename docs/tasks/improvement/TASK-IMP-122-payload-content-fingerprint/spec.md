---
id: TASK-IMP-122
title: rules_sha must be recomputed, not recalled
template: task@1
type: improvement
module: improvement
status: draft
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-18T04:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-074, TASK-IMP-104, TASK-IMP-082]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-18
memory_chain_hash: null
effort_hours: 16
service: tools/install
new_files:
  - tools/install/lib/rules-cone.sh
  - tools/install/tests/test_rules_sha_recompute.sh
modified_files:
  - tools/install/build.sh
  - tools/install/install.sh
  - tools/install/version.sh
  - tools/install/lib/update-check.sh
  - tools/install/audit-fleet.sh
source_pages:
  - "docs/tasks/_audits/2026-07-18-phase-corpus-measurement.md ADDENDUM - the operator's comparable-set decision: the cone is the VENDORED set, not the SHIPPED set. Every count below was re-measured at HEAD by this author on 2026-07-18, not copied from that file."
  - "tools/install/build.sh:353 - `_rsha()` defined INLINE; :354-355 - rules_sha computed at BUILD over $out, cone = `find cuo plugin mcp cli memory`; :358 - `cat > \"$out/manifest.yaml\"` writes the file holding the value (:357 is BLANK - the earlier :357 citation was wrong)"
  - "tools/install/version.sh:59-63 `_rs()` greps ^rules_sha:; :64-65 reads BOTH manifests; :66-68 CYBEROS_PAYLOAD is a FALLBACK only; :89-93 verdict=rules_drift"
  - "tools/install/lib/update-check.sh:8-14 `_cyberos_rules_sha()` greps a manifest; :18 mode soft by default; :82-97 the drift computation; :84 CYBEROS_PAYLOAD takes PRECEDENCE over self_root (:86); :101-105 the RULE DRIFT emit"
  - "tools/install/audit-fleet.sh:13 `_rs()` definition; :14 CYBEROS_EXPECT_RULES_SHA (token-only mode); :16-17 default mode resolves a REAL payload tree at $_self/dist/cyberos; :19 the fail-open 'rule-drift check DISABLED' warning (NOT a token reader); :36-37 the installed-side token read and compare"
  - "tools/install/lib/update-check.sh:99 writes $cy/.update-check-cache - but NOT on every run: three early returns precede it (:19 mode off, :28 no VERSION, :39 the 12h throttle)"
  - "tools/install/check-version-sync.sh:56 reads rules_sha for 64-hex SHAPE only (:57-58), never compares an installed side; `grep -c '\\.cyberos'` on it = 0"
  - "measured 2026-07-18: `grep -nE 'shasum|sha256sum'` across version.sh + lib/update-check.sh + audit-fleet.sh returns EMPTY - no component recomputes anything"
  - "measured 2026-07-18: build.sh is NEVER VENDORED - absent from both dist/cyberos/ and .cyberos/; `_rsha()` is defined in exactly ONE place in-tree (build.sh:353), which the two installed comparators cannot reach"
  - "measured 2026-07-18: docs-tools/ and lib/ EXIST in dist/cyberos/ AND in .cyberos/ and are ABSENT from the :354 cone; cli/ IS in the cone and is ABSENT from .cyberos/"
  - "measured 2026-07-18: dist/cyberos/memory/store/ does not exist (0 files); .cyberos/memory/store/ holds 5 files. The 3-vs-8 figure is memory/'s TREE total (payload 3, installed 8), not memory/store/'s"
  - "measured 2026-07-18: four cone combinations over dist/cyberos vs .cyberos - today `66bb0459`/`ae756045` MISMATCH; prune cli only `1f05a84f`/`ae756045` MISMATCH; prune memory/store only `66bb0459`/`1f05a84f` MISMATCH; prune both `1f05a84f`/`1f05a84f` MATCH"
  - "measured 2026-07-18: the corrected cone of §1.4 (cuo plugin mcp lib docs-tools + memory minus store + 6 root scripts) yields `102dc507` on BOTH sides over 1525 files per side - a clean install is current, with every coned dir and root script byte-identical under `diff -rq`"
  - "tools/install/install.sh:185-198 (the vendor block) and :430-433 (the memory loop) - TWO places, not three; :484 heredoc-generates AGENT-ENTRY.md; :282-290 generates gates.env; :320-335 scaffolds config.yaml; :105 .install.lock; :199 gates.env.bak.*"
  - "tools/install/lib/version-compare.sh - the in-tree PRECEDENT for a shared vendored definition (TASK-IMP-104: 'ONE comparator, sourced'); vendored to .cyberos/lib/ and sourced by update-check.sh:62 and version.sh:78-79"
source_decisions:
  - "2026-07-18 Stephen: PLAN gate - author as its own task, distinct from TASK-IMP-104 (ordering)."
  - "2026-07-18 audit FAIL 4/10 -> rewrite. The first draft claimed 'nothing compares content', which is false: TASK-IMP-074 ships rules_sha. REPAIR it; do not add a second comparator (TASK-IMP-104 §1.2)."
  - "2026-07-18 operator: the cone is a MAINTAINED LIST declared once in a shared file and reconciled against install.sh by a build check - NOT derived by static analysis, NOT a per-file manifest."
  - "2026-07-18 operator: the report promises only what a tree digest delivers - name paths where a payload TREE is reachable, say plainly where only a token is."
  - "2026-07-18 author (this rewrite, NEW3-004): the cone's element grammar is `dir:` / `file:` / `prune:`, and memory is coned as `dir:memory` + `prune:memory/store/`. Rationale in Alternatives."
  - "2026-07-18 author (this rewrite, NEW3-005): `_rsha()` moves into the same shared vendored file as the cone. Rationale in Alternatives."
  - "2026-07-18 author (this rewrite): the six vendored root scripts are IN the cone per the ADDENDUM, superseding rewrite 3's §1.3 class (d) which excluded them. Measured: including them yields no false drift."
---

# TASK-IMP-122: rules_sha must be recomputed, not recalled

## Summary

CyberOS already ships a rule-content fingerprint. `rules_sha` (TASK-IMP-074) is computed at build
time over the payload and written into `manifest.yaml`. Three components compare it against an
installed machine, and every one of them answers the installed side by grepping the stored token
out of `.cyberos/manifest.yaml` rather than by re-hashing the installed files. Two tokens from one
build always agree, whatever happens to those files afterward. So the check answers "did these
manifests come from the same build?" and is read as "do these bytes still match?".

## Problem

Measured on this repo at HEAD, 2026-07-18. Every number below was re-measured by this author.

**Defect 1 - the fingerprint is recalled, not recomputed.** `build.sh:354` computes `rules_sha`
over the assembled output. `install.sh:188` vendors the manifest containing it. Then:

| component | what it does |
|---|---|
| `version.sh:59-63, :89-93` | `_rs()` greps `^rules_sha:` from each manifest; `verdict="rules_drift"` on mismatch |
| `lib/update-check.sh:8-14, :82-97` | `_cyberos_rules_sha()` greps a manifest; `:101-105` emits `RULE DRIFT ... (same version, different rules)` |
| `audit-fleet.sh:13, :36-37` | `_rs()` greps `$cy/manifest.yaml`; compares each fleet repo's token to an expected one |

`grep -nE 'shasum|sha256sum'` across all three returns **empty**. Nothing recomputes.
(`check-version-sync.sh` reads `rules_sha` too, at `:56`, but only asserts it is 64-hex
(`:57-58`); it is not a drift comparator and is out of scope: `grep -c '\.cyberos'` on it returns
**0** - it compares payload stamps to the root `VERSION` and has no installed side at all.)
`update-check.sh:76` states the correct thesis - *"VERSION is a promise; rules_sha is the
evidence"* - and then greps a token that is a promise for exactly the same reason VERSION is: it
was written once, at build, by the thing being checked.

The sharpest form of this is measurable today. Run as `bash .cyberos/version.sh`, `$here` resolves
to `$CY`, so `:64` reads `$root/.cyberos/manifest.yaml` and `:65` reads `$here/manifest.yaml` -
**the same file**. `:66-68` cannot rescue it, because `CYBEROS_PAYLOAD` is consulted only when
`$here/manifest.yaml` yields nothing, and it always yields something on an installed machine. So
`:89`'s `[ "$inst_sha" != "$pay_sha" ]` is unreachable: that invocation compares a file to itself
and **can never report `rules_drift`**. Verified by running it - both sides print `66bb0459...`,
`verdict=up_to_date`.

The consequence is not theoretical. On 2026-07-18 this repo's own `.cyberos` differed from `dist/`
in six vendored artefacts while `VERSION` read `1.0.0` and `rules_sha` matched on both sides. The
installed `batch-select.mjs` was the pre-PR#53 build carrying the undeclared-cone bug, and returned
a different batch than source. Every check reported current.

**Defect 2 - the cone is the wrong set.** `build.sh:354` hashes `find cuo plugin mcp cli memory`.
Measured against the installed tree:

```
cone today (build.sh:354)  : cuo plugin mcp cli memory
vendored set (install.sh)  : cuo plugin mcp memory docs-tools lib + 6 root scripts
  in cone, NEVER installed : cli            -> guarantees false drift
  installed, NOT in cone   : docs-tools lib -> the blind spot (4 of the 6 measured drifts)
  never in cone at all     : the 6 root scripts (:190-195)
```

Four of the six artefacts that actually drifted (`batch-select.mjs`, `render-status-hub.mjs`,
`verify-goals.mjs`, `workflow-improve.mjs`) live in `docs-tools/`. So even a recomputing check
would miss them. And `cli/` is in the cone and is never installed, so a recomputing check with
today's cone would report drift on a **perfect** install. The two defects are independent and both
must close, or the fix either misses the evidence that motivated it or cries wolf on every machine.

**Why REPAIR and not a second mechanism.** TASK-IMP-104 §1.2 forbids install carrying a second
implementation, and 104 explicitly rejected content hashing for ITS question: *"Compare a manifest
hash rather than a version. Rejected: overkill for a monotonic version line, and it answers
'different' rather than 'older', which is not the question."* 104 guards ORDER and is correct as
shipped; "different" is precisely this task's question. `rules_sha` is the right mechanism, in the
right place, doing the wrong thing. This task repairs it.

## Proposed Solution

Split the two sides of the comparison and treat them differently, because they are not symmetric.

**The installed side is ALWAYS recomputed** - hashed from the files on disk at comparison time,
never read from `.cyberos/manifest.yaml`. That is the whole of Defect 1.

**The reference side is a token that `build.sh` wrote over a payload tree.** A stored token is
trustworthy *as a digest of the payload*, because build computed it from that payload's bytes; it
is untrustworthy *as a digest of the install*, because nothing recomputed it after the copy. Where
a payload tree is also reachable, the check additionally diffs that tree to name the differing
paths; where only a token is reachable, it reports drift without naming and says so. This makes
`bash .cyberos/version.sh` useful for the first time: recomputing the installed tree against the
token its own manifest carries catches post-install mutation, and `manifest.yaml`'s exclusion from
the cone is what keeps that from being circular.

**Fix the cone to the vendored set** - what `install.sh` installs, not what the payload ships -
declared once in a shared file that `build.sh` and every comparator read, with a build check that
reconciles it against a real install in both directions. Move `_rsha()` into that same shared file,
because `build.sh` is never vendored and the two installed comparators cannot reach it. Report the
differing paths where a tree allows it. Keep each component's existing exit contract.

## Alternatives Considered

- **Add a per-file manifest beside `rules_sha`.** Rejected: a second comparator for the same
  question, which TASK-IMP-104 §1.2 forbids by name, and the first draft of this spec proposed it
  only because it had not found `rules_sha`.
- **Bump VERSION on every payload change.** Rejected: conflates release identity with build
  identity, forces a bump for a comment fix, and still cannot see a vendor step that drops a file.
- **Compare mtimes.** Rejected: not content; survives no copy or checkout faithfully; would have
  reported the 2026-07-18 drift as fine.
- **Re-vendor on every `.cyberos` use.** Rejected: install is not free, and a guard that silently
  repairs drift teaches nobody that the channel leaked.
- **Widen the cone only, leaving the token stored.** Rejected: it fixes which files are covered and
  leaves the check comparing two copies of one build-time answer. Defect 1 survives untouched.
- **Derive the cone by static analysis of `install.sh`.** Rejected by the operator, and the code
  says why: the copies are conditionally guarded (`:187-198`), one iterates a loop variable
  (`:431-432`), and one branch is env-gated (`memory/` only when `CYBEROS_NO_MEMORY != 1`). No
  static read of that file yields the set.
- **NEW3-004, the element grammar: `dir:` / `file:` / `prune:`, with memory as `dir:memory` +
  `prune:memory/store/`.** Chosen over the file-granular reading (`file:memory/AGENTS.md` and its
  two siblings). Both hash identically today - each resolves to the same 3 files per side - so the
  tie breaks on what happens when they diverge. Under `file:` × 3, a fourth file added to the
  payload's `memory/` that `:431-432`'s hardcoded loop does not copy is invisible to every check,
  forever. Under `dir:memory`, the cone resolves to 4 while the install has 3, and §1.6's second
  direction fails the build - the divergence surfaces at build time, on the machine of the person
  who caused it, instead of becoming permanent drift on every install that no re-install can clear.
  The file-granular reading also makes `prune:memory/store/` dead text for §1.4, since nothing then
  enumerates it - which was the audit's charge; under the chosen grammar it is load-bearing in both
  §1.4 and §1.6. Three kinds and not two, because `install.sh` has exactly three copy shapes:
  `cp -R` on a directory, `cp` on a named file, and install-generated content nested inside a coned
  directory.
- **NEW3-005, `_rsha()`: move it into the shared cone file rather than leave it inline at
  `build.sh:353`.** `build.sh` is never vendored - measured absent from both `dist/cyberos/` and
  `.cyberos/` - so `.cyberos/version.sh` and `.cyberos/lib/update-check.sh` cannot reach it, and
  "use `build.sh`'s own `_rsha()`" was unsatisfiable for two of the three comparators. The
  alternative, letting each comparator define its own, is the exact duplication §1.2 forbids for
  the cone and would let the two platform branches drift apart silently. The shared file goes in
  `lib/`, which `install.sh:197` already vendors wholesale, following `lib/version-compare.sh` -
  TASK-IMP-104's "ONE comparator, sourced", already resolved two ways by `version.sh:78` and
  `update-check.sh:62`. The file lands inside the cone, so it fingerprints itself; it contains no
  digest, so nothing is circular.
- **NEW3-007, effort: 6 -> 16 hours.** 6 was carried unchanged across four revisions and was sized
  for "widen a `find` and add a re-hash". The task now also creates a shared vendored library,
  rewires four callers onto it, adds a reconciler that performs a real install, and edits
  `install.sh` to vendor the library. Breakdown: shared lib 2h; `build.sh` rewire 1h; reconciler
  3h; `version.sh` 2h; `update-check.sh` 2h; `audit-fleet.sh` 2h; `install.sh` 0.5h; suite 5h.

## Success Metrics

- Primary: a vendored file mutated by one byte after install, with both VERSIONs and both stored
  tokens equal, is reported as drift and named. Baseline: reported `up_to_date` - reproduced
  synthetically 2026-07-18; the original six-artefact instance was repaired by `e2504cf3` and is
  no longer live evidence.
- Guardrail: a freshly installed, unmodified machine reports current from every component.
  Measured target: the corrected cone yields `102dc507` over both `dist/cyberos` and `.cyberos`
  (1525 files per side). Baseline: today's cone yields `66bb0459` vs `ae756045` on that same
  byte-identical pair - a recomputing check with today's cone would report drift on a perfect
  install.
- Guardrail: every path `install.sh` VENDORS is either inside the cone or inside a named exclusion
  class, enforced at build. (Not "every directory the payload ships": that formulation is retracted
  by the ADDENDUM and is what forced `ci/`, `cli/` and `template/` into the cone and guaranteed
  self-drift.)

## Scope

In scope: the cone and `_rsha()` at `build.sh:353-355`, relocated to a new shared vendored library;
recomputation and path-naming in `version.sh`, `lib/update-check.sh`, `audit-fleet.sh`; the build
reconciler; `install.sh` vendoring the new library; a suite. `check-version-sync.sh` is deliberately
NOT in scope: it has no installed side (`grep -c '\.cyberos'` = 0) and only shape-checks the token.

### Out of scope / Non-Goals

- TASK-IMP-104's ordering guard. Correct as shipped; untouched.
- Install-generated and operator-owned paths (`gates.env`, `config.yaml`, `.update-check-cache`,
  `AGENT-ENTRY.md`, `memory/store/`). Normatively excluded by §1.5, not merely noted.
- Auto-repairing drift. Reporting is the deliverable; `install.sh` already re-vendors.
- Tamper-evidence. §1.14 makes the prohibition normative rather than a hope in prose.
- Build-side conditional copies that silently drop a file from the payload (`build.sh:198`). See
  §3 - a recorded, uncovered gap, not a promise this task can keep.

## Dependencies

None. TASK-IMP-104's guard runs before the lock and is untouched, and the new shared library sits
beside `lib/version-compare.sh` without altering it.

## AI Authorship Disclosure

- **Tools used:** Claude (Opus 4.8) running the CyberOS task-author skill inside Cowork.
- **Scope:** rewritten in full 2026-07-18 after four independent audits (4/10, 6/10, 6/10, 6/10)
  found that the prior author patched only what each audit named. This is a whole-document rewrite,
  not a patch. Every numeric and line-number claim was re-measured against source at HEAD by this
  author during this rewrite - including those inherited from the evidence file, four of which were
  wrong (`build.sh:357`, `memory/store/` 3-vs-8, "three separate places", `audit-fleet.sh:19`).
  Claims that could not be measured are marked as gaps in §3 rather than asserted.
- **Human review:** scope approved at the 2026-07-18 PLAN gate; both HITL gates are recorded human
  verdicts. The comparable-set, maintained-list and report-promise decisions are the operator's,
  recorded in `source_decisions`; the grammar, `_rsha()` and effort decisions are this author's,
  with rationale in Alternatives.

## 1. Description (normative)

- 1.1 Every component that reports rule drift between an installed machine and a payload (`version.sh`, `lib/update-check.sh`, `audit-fleet.sh`) MUST derive the INSTALLED side by RECOMPUTING the fingerprint over the installed tree at comparison time. No component may derive the installed side by reading a stored token out of `.cyberos/manifest.yaml`. The REFERENCE side MAY be a stored token, because `build.sh` computed it over a payload tree: a token is a faithful digest of the payload and never of the install.
- 1.2 The cone AND `_rsha()` MUST each be declared exactly ONCE, in a single shared file (`lib/rules-cone.sh`) that `build.sh`, `version.sh`, `lib/update-check.sh` and `audit-fleet.sh` all read by sourcing, and which `install.sh` MUST vendor so the two installed comparators can reach it. No component may carry its own copy of the list or its own definition of the digest function. The list is MAINTAINED, not derived: no static read of `install.sh` yields the vendored set, because its copies are conditionally guarded (`:187-198`), one iterates a loop variable (`:431-432`), and `memory/` is env-gated on `CYBEROS_NO_MEMORY`.
- 1.3 Every entry in the shared file MUST carry exactly one of three kinds: `dir:<path>` (every file beneath it, recursively), `file:<path>` (exactly that path), or `prune:<path>` (every file beneath it is REMOVED from the resolved set). An entry with no kind, or with an unrecognised kind, MUST fail the build rather than be silently skipped.
- 1.4 The cone MUST be exactly the vendored set: `dir:cuo`, `dir:plugin`, `dir:mcp`, `dir:lib`, `dir:docs-tools`, `dir:memory` (`:185-198`, `:430-433`), the six root scripts as `file:` entries - `install.sh uninstall.sh version.sh status.sh help.sh check-latest.sh` (`:190-195`) - and `prune:memory/store/`. The root scripts ARE in the cone, per the ADDENDUM: they are vendored, and they are byte-identical after a faithful install (measured). "They change on every payload edit" is true of the entire cone and is not a reason to exclude anything.
- 1.5 The cone MUST exclude three classes, each declared BY NAME alongside it: (a) install-generated or operator-owned paths that appear under `.cyberos/` and are never copied from the payload - `memory/store/` (`:430`, `:441-447`), `gates.env` (`:282-290`), `config.yaml` (`:320-335`), `.update-check-cache` (`update-check.sh:99`), `AGENT-ENTRY.md` (heredoc-generated at `:484`), `gates.env.bak.*` (`:199`), `.install.lock` (`:105`); (b) `manifest.yaml`, which IS copied (`:188`) but MUST be excluded because it CONTAINS the value - `build.sh:354` computes `rules_sha` and `:358` writes the file holding it, so covering it would make the value depend on itself; (c) `VERSION`, which IS copied (`:189`) but is TASK-IMP-104's axis, not this one. Classes (b) and (c) MUST be excluded by NAME rather than by omission, so that §1.6 can distinguish a deliberate exclusion from an unclassified path.
- 1.6 A build check MUST reconcile the declared list against what `install.sh` actually installs, in BOTH directions, and MUST fail the build on either. Direction 1: a path present under `$CY` after an install that is in neither the cone nor an exclusion class MUST fail the build. Direction 2: a path the cone resolves against the payload that is ABSENT from `$CY` after an install MUST fail the build - this is the direction that catches `cli`, which is in today's cone and is never vendored, is the measured cause of the `66bb0459` vs `ae756045` false drift, and which nothing else catches structurally.
- 1.7 The reconciler MUST obtain `install.sh`'s vendored set by EXECUTING a real install of the freshly built payload into a temporary root and enumerating the resulting `$CY`, never by parsing `install.sh`. This is what §1.2 means by reconciling a maintained list against reality: running the loop at `:431-432` resolves its three names without the reconciler hardcoding them, and running the guards at `:187-198` resolves them without interpreting them. The reconciler MUST run with `CYBEROS_NO_MEMORY` unset (the default), MUST enumerate `$CY` ONLY and not the wider temp repo that `install.sh` also writes to (`:611`, `:636`, `:666`), and MUST NOT be written so that it derives its expected set from the same list it validates.
- 1.8 A recomputed installed fingerprint differing from the reference MUST be reported as drift, INCLUDING when both VERSION strings and both stored manifest tokens are equal.
- 1.9 Path-naming MUST be decided per INVOCATION, never per component, because reachability of a payload TREE is a property of how a tool is called. Where a payload tree IS reachable, the report MUST name every differing path. Where only a token is reachable, the report MUST state that it cannot name paths, rather than implying a complete report. The arms, measured: `bash <payload>/version.sh <repo>` has a tree (`$here` is the payload); `bash .cyberos/version.sh` has NONE (`$here == $CY`, and `CYBEROS_PAYLOAD` at `:44`/`:66` is a fallback that a present installed manifest pre-empts); `lib/update-check.sh` has a tree when `CYBEROS_PAYLOAD` is set (`:84` takes PRECEDENCE over `self_root`) or when sourced from a payload's `lib/` (`:86`), and has none when sourced from `.cyberos/lib/` with `CYBEROS_PAYLOAD` unset (`self_root == $CY`); `audit-fleet.sh` has a tree in its DEFAULT mode (`:16-17` resolve `$_self/dist/cyberos`) and only a token under `CYBEROS_EXPECT_RULES_SHA` (`:14`, which short-circuits `:15`'s resolution entirely).
- 1.10 An installed tree byte-identical to the payload ACROSS THE CONE MUST be reported as current by every component. "Byte-identical across the cone" is the whole test: paths outside the cone MUST NOT affect the verdict - neither `ci/`, `cli/` and `template/`, which ship and are never installed, nor `memory/store/`, which installs and never ships.
- 1.11 Recomputation MUST use the shared `_rsha()` (`sha256sum` on Linux, `shasum -a 256` on macOS) and its `LC_ALL=C` sort, so that a byte-identical tree yields a byte-identical fingerprint on both platforms and on both sides of every comparison.
- 1.12 A comparison that cannot be computed (the reference manifest absent or unreadable, the installed tree unreadable, or the shared cone file unreadable) MUST yield the verdict `unknown`. No component may yield a current verdict on a comparison it could not perform.
- 1.13 Exit contracts: `lib/update-check.sh` remains soft-by-default (`:18`) and MUST NOT change its exit semantics in any mode it binds. `audit-fleet.sh` MUST NOT retain its present fail-open behaviour (`:19`, "rule-drift check DISABLED"); with no expected token it MUST yield `unknown` per §1.12 rather than warning and silently passing.
- 1.14 No component's output may describe the fingerprint as detecting tampering, since anything able to rewrite a vendored file can rewrite the manifest beside it.
- 1.15 A comparison MUST NOT modify the installed machine. `lib/update-check.sh` alone MAY continue writing its existing `$cy/.update-check-cache` (`:99`, reached only when none of the three early returns at `:19`, `:28` and `:39` fires); no other component may write anything.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - for EACH of the three comparators, mutating one byte of a vendored file while leaving BOTH stored manifest tokens untouched is RECOMPUTED to a differing digest and reported as drift; the test MUST exercise all three by name and MUST FAIL if any one still answers the installed side from the stored token - test: `tools/install/tests/test_rules_sha_recompute.sh::t01_installed_side_recomputed_not_recalled`
- [ ] AC 2 (traces_to: #1.2) - `lib/rules-cone.sh` is the ONLY in-tree definition of both the cone and `_rsha()`, is sourced by `build.sh`, `version.sh`, `lib/update-check.sh` and `audit-fleet.sh`, and is present at `.cyberos/lib/rules-cone.sh` after an install; the test MUST FAIL if any caller carries a second copy of either, MUST FAIL against today's `build.sh:354` which inlines `find cuo plugin mcp cli memory`, and MUST FAIL if `install.sh` does not vendor it - test: `tools/install/tests/test_rules_sha_recompute.sh::t02_one_shared_vendored_declaration`
- [ ] AC 3 (traces_to: #1.3) - a cone entry with no kind prefix, and separately one with an unrecognised kind, each FAIL the build rather than being skipped; and each of the three kinds resolves as specified - `dir:` pulls in a nested file, `file:` pulls in exactly one path and not its siblings, `prune:` removes a file beneath a coned dir - test: `tools/install/tests/test_rules_sha_recompute.sh::t03_element_grammar_dir_file_prune`
- [ ] AC 4 (traces_to: #1.4) - the cone resolves to exactly the vendored set: mutating one file under EACH of `cuo plugin mcp lib docs-tools memory` and EACH of the six root scripts yields drift (12 arms, each asserted separately); the test MUST FAIL on today's cone, which omits `lib`, `docs-tools` and all six root scripts, and MUST FAIL if `cli` is present in the cone - test: `tools/install/tests/test_rules_sha_recompute.sh::t04_cone_is_exactly_the_vendored_set`
- [ ] AC 5 (traces_to: #1.5) - each exclusion class yields NO drift when its member changes, tested by a member that requires an ACTIVE prune and not only by trivial members: class (a) via `memory/store/` (adding a file INSIDE the coned `dir:memory`) AND via `gates.env`, `config.yaml`, `.update-check-cache` and `AGENT-ENTRY.md`; class (b) via mutating `manifest.yaml`'s own `rules_sha:` line; class (c) via `VERSION`; and the test MUST FAIL if class (b) or (c) is excluded by omission rather than by a named entry - test: `tools/install/tests/test_rules_sha_recompute.sh::t05_exclusions_named_and_pruned`
- [ ] AC 6 (traces_to: #1.6) - BOTH directions fail the build, each asserted separately: direction 1, adding a `cp` of a NEW path into `install.sh` that is in neither cone nor exclusions fails the build naming that path; direction 2, a cone entry the install does not produce fails the build, exercised with the REAL `cli` fixture (add `dir:cli` to the cone, build, assert non-zero exit naming `cli`), and the test MUST FAIL if the build passes with `cli` coned - test: `tools/install/tests/test_rules_sha_recompute.sh::t06_reconciler_fails_both_directions`
- [ ] AC 7 (traces_to: #1.7) - the reconciler obtains its expected set by EXECUTING install.sh into a temp root: the test asserts a temp `$CY` was created and enumerated, asserts the `:431-432` memory loop resolved to its three files with no memory filename hardcoded in the reconciler, and MUST FAIL if the reconciler is rewritten to parse `install.sh` textually, if it derives its expected set from the list it validates, or if it enumerates any path outside `$CY` - test: `tools/install/tests/test_rules_sha_recompute.sh::t07_reconciler_runs_install_does_not_parse_it`
- [ ] AC 8 (traces_to: #1.8) - drift is reported with both VERSION strings AND both stored tokens equal - the exact 2026-07-18 shape; the test MUST FAIL if any component reports current - test: `tools/install/tests/test_rules_sha_recompute.sh::t08_equal_tokens_still_drift`
- [ ] AC 9 (traces_to: #1.9) - per INVOCATION and not per component, all seven measured arms asserted separately: `bash <payload>/version.sh <repo>` names all THREE mutated files and MUST FAIL if only the first is named; `bash .cyberos/version.sh` states it cannot name paths AND still reports the drift AND MUST FAIL if setting `CYBEROS_PAYLOAD` changes either (it is a fallback, pre-empted at `:66`); `update-check.sh` with `CYBEROS_PAYLOAD` set names paths, sourced from a payload's `lib/` names paths, and sourced from `.cyberos/lib/` without it states it cannot; `audit-fleet.sh` on its DEFAULT names them, and under `CYBEROS_EXPECT_RULES_SHA` states it cannot - test: `tools/install/tests/test_rules_sha_recompute.sh::t09_naming_follows_invocation_not_component`
- [ ] AC 10 (traces_to: #1.10) - a freshly installed unmodified machine is reported CURRENT by every component, asserting the measured `102dc507` on both sides over 1525 files per side; the test MUST FAIL on today's cone, which yields `66bb0459` over the payload vs `ae756045` over the install on that byte-identical pair - caused INDEPENDENTLY by `cli/` (coned, never installed) AND by `memory/store/` (install-generated inside a coned dir, 0 payload files vs 5 installed), and the AC MUST exercise both by asserting all four measured combinations: today `66bb0459`/`ae756045`, prune-cli-only `1f05a84f`/`ae756045`, prune-store-only `66bb0459`/`1f05a84f`, prune-both `1f05a84f`/`1f05a84f` - test: `tools/install/tests/test_rules_sha_recompute.sh::t10_clean_install_is_current`
- [ ] AC 11 (traces_to: #1.11) - every comparator RESOLVES `_rsha()` from the shared file and carries no second definition (identity of implementation, not merely a matching digest), the recomputed fingerprint EQUALS `build.sh`'s for the same tree, and a fixture forcing the `shasum -a 256` branch yields the same digest as one forcing `sha256sum`; the test MUST FAIL if either platform branch diverges or if any comparator defines its own `_rsha` - test: `tools/install/tests/test_rules_sha_recompute.sh::t11_shared_rsha_identity_and_cross_platform`
- [ ] AC 12 (traces_to: #1.12) - with the reference manifest removed, separately with it unreadable, separately with the installed tree unreadable, and separately with `lib/rules-cone.sh` unreadable, every component emits the verdict `unknown` and none emits a current verdict; the assertion is on the emitted VERDICT (§1.12's verb) - test: `tools/install/tests/test_rules_sha_recompute.sh::t12_uncomputable_is_unknown_not_current`
- [ ] AC 13 (traces_to: #1.13) - `update-check.sh` keeps its exit semantics in EVERY mode it binds (`soft` returns 0 on drift, `strict` returns non-zero, `always` ignores the throttle, `0`/`off`/`false` returns 0 immediately) - a regression guardrail, stated as such; and `audit-fleet.sh` with no expected token emits `unknown` and does NOT pass, which MUST FAIL against today's `:19` warning-and-continue - test: `tools/install/tests/test_rules_sha_recompute.sh::t13_exit_contracts`
- [ ] AC 14 (traces_to: #1.14) - no component's output matches tamper/integrity/authenticity wording on a drift run, an `unknown` run, OR a current run - test: `tools/install/tests/test_rules_sha_recompute.sh::t14_no_tamper_claims`
- [ ] AC 15 (traces_to: #1.15) - the installed tree is byte-identical before and after a comparison from EACH component; `.update-check-cache` is written by `lib/update-check.sh` ONLY, and the test MUST FAIL if `version.sh` or `audit-fleet.sh` writes it or anything else - test: `tools/install/tests/test_rules_sha_recompute.sh::t15_check_is_read_only`

## 3. Edge cases

- Payload NEWER than installed with VERSION differing: TASK-IMP-104's ordering guard owns the refusal; this check reports drift and does not duplicate it.
- Operator hand-edited a vendored file: reported as drift and named. The report says what differs, never who differed it (§1.14 covers the wording).
- A vendored file that is legitimately empty: hashed like any other. Empty is a content.
- First install, no installed tree: §1.12's `unknown`, not drift and not current.
- **A BUILD-side conditional copy that silently drops a file from the payload** (`build.sh:198`'s `[ -f "$here/docs-tools/workflow-improve.mjs" ] && cp ...` - the measured `workflow-improve.mjs` case): the file never reaches the payload, so it is absent from the payload fingerprint AND from the install, and both sides agree. **Nothing in this task catches that.** §1.6's reconciler cannot either: it resolves the cone against the payload, and a file missing from the payload is missing from that resolution too. Recorded as an uncovered gap, not handed to a clause that cannot honour it. It is the sibling defect and wants its own task.
- **A payload file under a coned dir that `install.sh` does not vendor** is caught at BUILD by §1.6's direction 2, not at comparison time - deliberately. The build fails for the person who caused the divergence, instead of every installed machine reporting a drift that no re-install can clear. This is the `dir:memory` grammar decision doing its work.
- `audit-fleet.sh` with a reachable payload rather than a bare token: it MAY name paths per §1.9's first arm; the clause requires the disclaimer only when no tree is reachable.
- `CYBEROS_NO_MEMORY=1` installs: `memory/` is absent from `$CY` by design, so §1.6's direction 2 would fail the build if the reconciler ran under it. §1.7 pins the reconciler to the default environment; the env-gated variant is a documented install mode, not a cone violation, and this task does not extend the reconciler to cover it. A machine installed with `CYBEROS_NO_MEMORY=1` will report drift against a memory-carrying payload, correctly and unhelpfully - out of scope here.
- macOS vs Linux: §1.11 pins the shared `_rsha()` and `LC_ALL=C`. This is the `stat -c/-f` class of defect TASK-IMP-103 already paid for once.
- Security-class: the fingerprint is a staleness signal, not an integrity guarantee. §1.14 makes that normative and AC 14 tests it across all three verdicts.
