---
task_id: TASK-IMP-122
audited: 2026-07-18 (audit 6, of rewrite 5). Rounds 4-6 RECONSTRUCTED FROM COMMIT MESSAGES - see §0.
verdict: FAIL
score: 8/10
score_history: "4/10 -> 6/10 -> 6/10 -> 6/10 -> 8/10 -> 8/10"
issues_closed: "round 6: NEW4-001, NEW4-002, NEW4-003, NEW4-004 - all four verified closed by the round-6 auditor. NEW4-005/006 were acted on by rewrite 5; their provenance is unverifiable (§3)."
issues_open: "NEW5-001, NEW5-002, NEW5-003, NEW5-004, NEW5-006 (four clause edits, not a rewrite). NEW5-007 is the persistence defect this file answers. NEW5-005: NO SURVIVING RECORD - see §5.4."
weak_acs: "0/15 at rewrite 5 (was 3/15 at rewrite 4, 4/12 at rewrite 3)"
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: "task-lint clean at every round measured. At rewrite 5: 15 clauses / 15 ACs / 20h."
auditor: "round 6: independent subagent; re-derived every originated number and found ZERO false - the first such round in six."
reconstruction_notice: >
  Rounds 4, 5 and 6 were RECONSTRUCTED on 2026-07-18 from commit messages (7309cd80, 15894b1e,
  1f8143cf). No contemporaneous audit file exists for any of them - they were never persisted.
  Round 3 (§1) is the only section on this page that was written at the time it was performed.
  Per-section provenance is stated at the head of every section. Do not read a reconstructed
  section as an auditor's own words: it is the orchestrator's summary of an audit, and the audit
  itself is gone.
STOP_SIGNAL_round3: >
  The author has failed to raise this score across three rewrites. The failure mode is now
  legible and is recorded in §2. A fourth rewrite by the same author, patching the findings
  below, will likely reproduce it. Read §2 before attempting one.
STOP_SIGNAL_status: >
  SUPERSEDED 2026-07-18, by rewrite 4 (commit 6370548307e8, audited at 7309cd80). The signal is
  PRESERVED above because it was a real judgement honestly made on the evidence then available,
  and it was ACTED ON - it is why rewrite 4 was delegated to a fresh author at all. It is
  superseded rather than deleted because its central prediction was tested and did not hold: it
  predicted that a rewrite PATCHING THE NAMED FINDINGS would reproduce the pattern. The rewrite
  that followed was written by a DIFFERENT author who rewrote the whole document rather than
  patching it, and it scored 8/10 - breaking the flat line on the first round the original author
  did not hold the pen. The signal's condition ("a fourth rewrite by the same author") was
  therefore never actually run, so it was not falsified so much as routed around. What IS
  falsified is the frontmatter line it sat beside: see FLAT_LINE_status.
FLAT_LINE_status: >
  Round 3's score_history read "4/10 -> 6/10 -> 6/10 -> 6/10 (FLAT for four rounds)". That was
  TRUE ON 2026-07-18 WHEN WRITTEN and is preserved verbatim in §1's own header block below. It is
  FALSE NOW: rounds 5 and 6 scored 8/10, so the line is 4 -> 6 -> 6 -> 6 -> 8 -> 8 and has not
  been flat since round 4. Corrected in this file's score_history; preserved in §1 as the record
  of what was true when it was written.
---

# TASK-IMP-122 - audit record

## §0 - Provenance, and why this file had to be reconstructed

**This page was rebuilt on 2026-07-18. Read this section before trusting any finding id on it.**

The audits for rounds 4, 5 and 6 were written into COMMIT MESSAGES and never into this file. The
findings were then handed author-to-author through prompts. Three authors cited, closed and argued
against findings they could not open. That defect was found independently by two agents, recorded as
**NEW5-007**, and is quoted in §5.5. This file is the remedy: every id cited anywhere in the task now
resolves to a section here, and every section states where it came from.

**What that means for a reader.** There are two kinds of section on this page and they are not
interchangeable:

| kind | sections | what you are reading |
|---|---|---|
| **written at the time** | §1 (round 3) | the auditor's own audit file, committed at `b37b795a`, preserved verbatim |
| **reconstructed from commit** | §2, §3, §4, §5 | the orchestrator's *summary* of an audit. The audit's own text does not exist. |

A reconstructed section is weaker evidence than a written-at-the-time one, in a specific way: it is
one author's account of what an auditor said, written after the fact, with no artefact behind it to
check the account against. Where the commit message is the only source, the section says so.

**The rounds, mapped.** The naming is inherited from the commits and is not self-explanatory - audit
*n+1* audits rewrite *n* from round 4 on:

| section | round | what it is | source | persisted at the time? |
|---|---|---|---|---|
| §1 | round 3 | audit of rewrite 3 - FAIL 6/10 | `b37b795a` (this file) | **YES** |
| §2 | round 4 | audit of rewrite 4 - FAIL 8/10, NEW4-001..004 | `7309cd80` message | no |
| §3 | round 4? | NEW4-005 / NEW4-006 - **provenance unverifiable** | nothing | **no audit ever existed for these** |
| §4 | round 5 | rewrite 5's closure record - **the author's claims, not an audit** | `15894b1e` message | no |
| §5 | round 6 | audit 6, of rewrite 5 - FAIL 8/10, NEW5-001..004/006/007 | `1f8143cf` message | no |

**Ids cited by `spec.md` that resolve here:** NEW3-004 (§1.4), NEW3-005 (§1.4), NEW4-002 (§2.3),
NEW4-003 (§2.4), NEW4-005 (§3), NEW4-006 (§3).

**Ids that do NOT fully resolve, stated plainly:**
- **NEW4-005, NEW4-006** - cited in `spec.md` as *"audit rated LOW"*. **No audit raising them
  survives, and the round-4 audit that would have raised them enumerates NEW4-001..004 only.** The
  attribution cannot be checked. See §3 - this is recorded rather than reconstructed, deliberately.
- **NEW5-005** - **no record of this id exists anywhere**: not in any commit message, not in any
  file, not in either spec. See §5.4.

---

# §1 - ROUND 3: audit of rewrite 3 - FAIL 6/10

> **Provenance: WRITTEN AT THE TIME.** This is the audit file as committed at `b37b795a`
> (2026-07-18). It is the only section of this page that is an auditor's own artefact. Preserved
> verbatim; nothing below this line in §1 has been edited.
>
> **Two things in it are now out of date, and are preserved rather than corrected** (the corrections
> live in this file's frontmatter, and the reasoning in `FLAT_LINE_status` and
> `STOP_SIGNAL_status`):
> - *"FLAT for four rounds"* / *"flat for a fourth round"* - true when written, **false now**
>   (rounds 5-6 scored 8/10).
> - the **STOP_SIGNAL** - a real judgement, acted on, now **superseded**: a different author scored 8.

Round 3's own header block, as written:

```yaml
task_id: TASK-IMP-122
audited: 2026-07-18 (rewrite 3; supersedes the rewrite-2 audit)
verdict: FAIL
score: 6/10
score_history: "4/10 -> 6/10 -> 6/10 -> 6/10 (FLAT for four rounds)"
issues_closed: 4
issues_partially_closed: 3
issues_open: 4 (survived VERBATIM) + 8 new
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean, four rounds running. TRACE-001/002/003 pass every time.
auditor: independent subagent; diffed 2d478393..f8899d64 and measured all four cone combinations
STOP_SIGNAL: >
  The author has failed to raise this score across three rewrites. The failure mode is now
  legible and is recorded in §2. A fourth rewrite by the same author, patching the findings
  below, will likely reproduce it. Read §2 before attempting one.
```

## §1.1 - Verdict summary

FAIL at 6/10, flat for a fourth round. The operator's maintained-list decision IS honoured and
rewrite 2's two CRITICALs ARE genuinely closed. It fails because the diff proves the hypothesis
it was given: rewrite 3 edited ONLY §1.2, §1.3, §1.4, §1.6 and AC 2/3/4/6/7/10 - the exact block
the prior audit named. Everything outside that block survived VERBATIM, and two of the edits are
REGRESSIONS that deleted working normative text.

## §1.2 - THE FAILURE MODE (read this before rewriting)

The author patches what the audit NAMES and does not re-read the document. Consequences, all
measured this round:

1. **Four findings survived verbatim** because they sat outside the named block: NEW2-005 (the
   Proposed Solution still says "token"), NEW2-009 (Summary duplicates a proposition), NEW2-010
   (":99 on every run" - THIRD round unedited), NEW2-011 (AC 1's "BOTH stored manifest tokens").
2. **A fix DELETED the clause that catches the live bug** (NEW3-001, below).
3. **A repaired clause now contradicts two unrepaired sections** (NEW3-002, below).
4. **A false number was copied from the evidence file's parenthetical without re-measuring**
   (AC 7's memory counts), while the AI-authorship disclosure claims "every claim here was
   re-measured against source that day". It was not.

The pattern across four rounds: close the named finding, introduce the same defect class one
layer deeper. Rewrite 1 conflated payload/installed cones. Rewrite 2 cited a line range that
excluded the blind spot. Rewrite 3 deleted the direction that catches `cli`. Each is "the check
does not cover the thing it exists to check", relocated.

## §1.3 - Prior findings

CLOSED: NEW2-001 (cone now covers lib/docs-tools/memory; `:185-198` is BETTER than the audit's
`:184-198` - `:184` is `rm -rf`, not a vendor), NEW2-002 (all 18 vendored paths land in cone ∪
exclusions - the build no longer always-fails), NEW2-007, NEW2-008 (auditor measured all four
combinations: prune cli only -> 1f05a84f/ae756045 MISMATCH; prune store only -> 66bb0459/1f05a84f
MISMATCH; prune both -> MATCH. The independence claim is EXACTLY right).
PARTIALLY: NEW2-003, NEW2-004, NEW2-006.
NOT CLOSED (verbatim survivors): NEW2-005, NEW2-009, NEW2-010, NEW2-011.

## §1.4 - New findings

### NEW3-001 (CRITICAL, REGRESSION) - §1.4's second direction was DELETED; the check can no longer catch the live defect.
Author verified by diff:
  rewrite 2: "A path the cone covers that `install.sh` does not vendor MUST fail the build."
  rewrite 3: [deleted]
The prior audit called that direction a tautology - true ONLY while §1.2 defined cone == vendored
set. §1.2 no longer says that (it now says "a single explicit list"), so the tautology is gone and
the direction is LOAD-BEARING again. And today's live defect IS exactly that direction: `cli` is
IN the cone and NEVER vendored - the measured cause of 66bb0459 vs ae756045, named by §1.7, and
half of AC 7's own reasoning. Nothing in §1.3 forbids `cli` in the cone; §1.4 now fires only on
vendored-but-unclassified. The build check the rewrite was built around CANNOT FAIL on the defect
that motivated it.

### NEW3-002 (CRITICAL) - the Proposed Solution and Success Metrics still mandate the RETRACTED cone.
Author verified, unedited at :102-103 and :128:
  ":102 ... compare that against the payload's manifest token. Widen the cone"
  ":103 to every directory the payload ships."
  ":128 - Guardrail: every directory present in the payload is inside the cone, enforced at build."
The evidence file RETRACTS that formulation by name: "'the cone MUST cover every directory the
payload SHIPS' is wrong and is what forced ci/, cli/, template/ in and guaranteed self-drift".
§1.3 was repaired; the identical retracted wording survives in two other sections, where it now
contradicts §1.3, §1.7, AC 7 and the operator decision - and mandates precisely the cone AC 7
exists to fail on.

### NEW3-003 (MAJOR) - AC 3 tests class (a) by its trivial member.
`gates.env` sits OUTSIDE every coned dir - excluding it requires nothing. `memory/store/` sits
INSIDE a coned dir - excluding it requires an ACTIVE PRUNE, and is a measured co-equal cause of
the false drift. AC 3 tests the exclusion that cannot break and skips the only one that can.
6 of the 7 paths in class (a) untested.

### NEW3-004 (MAJOR) - the cone's element grammar is undefined; `memory` is the proof.
§1.2 mandates "a single explicit list" and never says what an entry IS. build.sh:354 is
dir-granular. §1.3 needs three kinds: dirs (`cuo`), files (the three under `memory/`), prunes
(`memory/store/`). "cover the vendored FILES under memory/" is file-granular - under which class
(a)'s `memory/store/` entry is DEAD TEXT. Listing `memory/store/` as an exclusion implies
dir-minus-prune - under which AC 3 never tests it. Either reading carries a defect.

### NEW3-005 (MODERATE) - §1.8 has §1.2's disease and did not get §1.2's cure.
§1.8 requires "build.sh's OWN `_rsha()`". `_rsha()` is defined INLINE at build.sh:353, and
**build.sh is NEVER VENDORED** (absent from dist/cyberos/). So `.cyberos/version.sh` and
`.cyberos/lib/update-check.sh` cannot reach it - §1.8 is unsatisfiable for two of three
comparators. It needs exactly what §1.2 gave the cone (one shared, vendored definition). AC 8
tests EQUALITY of digest, not IDENTITY of implementation - so AC 8 PERMITS the duplicated second
implementation that AC 2 forbids for the cone.

### NEW3-006 (MODERATE) - §1.6's invocation cut is incomplete; AC 6 dropped a capability.
update-check.sh:84 gives CYBEROS_PAYLOAD PRECEDENCE over self_root - so reachability there is
decided by the env var, not by where it was sourced. §1.6 hedges ("in its PRIMARY mode") and never
says what the non-primary mode owes. And AC 6 DROPPED update-check.sh entirely - rewrite 2's AC 6
tested it. Regression.

### NEW3-007 (MINOR) - §1.2's deliverable is in neither new_files nor modified_files.
The shared cone file appears nowhere; no such file exists in-tree; install.sh (which must vendor
it for the two installed comparators to read it) is absent from modified_files. effort_hours: 6
is UNCHANGED across all four revisions despite the prior audit's explicit "re-size effort_hours".

### NEW3-008 (LOW) - three citation errors the author inherited and did not re-measure.
- `build.sh:357` cited as the manifest write. Author verified: **:357 is a BLANK LINE**; the write
  is `cat > "$out/manifest.yaml" <<EOF` at **:358**. This is the load-bearing rationale for
  exclusion class (b).
- AC 7: "`memory/store/` 3 payload files vs 8 installed" is **FALSE**. Author measured: **0 and 5**.
  The 3-vs-8 is `memory/`'s tree total, misattributed. It also CONTRADICTS the spec's own §1.7
  ("installs and never ships"). Two clauses, one document, opposite counts.
- §1.2 says "three separate places" and cites TWO ranges (`:185-198` and `:432`).

## §1.5 - Clause-verb table: 4 of 12 weaker (identical count to rewrite 2)
The SET moved: AC 2/7/10 CLOSED; AC 4/6/8 NEWLY weaker; AC 3 weaker for a new reason. **Three of
the four weak ACs are ones the author rewrote this round.** The rewrite relocated the weakness.
AC 4 restates §1.4's prohibition with no fixture and no observable - the identical defect the
prior audit already flagged once ("the AC never says WHAT to mutate"), recurring at a new AC.

## §1.6 - Required before re-audit
1. RESTORE §1.4's second direction and AC 4's matching half. Rewrite 2 had it right.
2. Rewrite the Proposed Solution AND Success Metrics - never edited, still retracted wording.
3. Name the mechanism by which §1.4 reads install.sh's vendored set, given §1.2 says no static
   read yields it, and say how it resolves the `memory/$f` loop without hardcoding.
4. Give §1.2's list an element grammar (dir / file / prune); settle `memory`.
5. Give `_rsha()` §1.2's treatment; make AC 8 test identity, not equality.
6. AC 3 must test memory/store/. AC 6 must test update-check.sh + CYBEROS_PAYLOAD. AC 4 needs a fixture.
7. Fix the four verbatim survivors and the three citation errors.
8. Add the shared cone file to new_files, install.sh to modified_files, re-size effort_hours.

---

# §2 - ROUND 4: audit of rewrite 4 - FAIL 8/10

> **Provenance: RECONSTRUCTED FROM COMMIT `7309cd80` (2026-07-18); no contemporaneous audit file
> exists.** What follows is the orchestrator's summary of an independent audit. The auditor's own
> text is gone. The commit message enumerates **NEW4-001 through NEW4-004 and no others** - which is
> the fact §3 turns on.

## §2.1 - Verdict summary

**FAIL 8/10.** History 4 -> 6 -> 6 -> 6 -> 8. **The flat line broke on the first round the original
author did not hold the pen** - rewrite 4 was delegated to an author who had written none of the
prior drafts, on round 3's STOP_SIGNAL.

Credited by the round-4 auditor:
- Both CRITICALs closed with REAL fixtures (the `cli` fixture is live: 1 file in payload, 0 in `$CY`).
- Weaker-AC count **3/15, all minor** - the first round in five with **no STRUCTURAL weakness**
  (was 5/7, 3/10, 4/12, 4/12).
- All three novel claims verified independently. The reconciler survived every feasibility test the
  auditor could construct: 2s runtime, deterministic across all 11 `CYBEROS_*` branches, CI-safe,
  non-recursive, **zero side effects outside the temp root**. Direction 1 classifies a real `$CY`
  with ZERO unclassified paths.

> Note, added at reconstruction: that last credited property - "zero side effects outside the temp
> root" - is the one rewrite 5 turned back on this auditor. See §3 (NEW4-006) and §5.1.

## §2.2 - NEW4-001 (MAJOR) - AC 10 pairs a correct digest with the WRONG count, normatively

The orchestrator's note records this as independently verified: *"I verified this myself, it is
exact."*

```
1525 files -> 86cafee8   (cone WITHOUT memory + root scripts - the cone §1.4 REJECTS)
1534 files -> 102dc507   (the corrected cone §1.4 mandates)
```

The spec pairs **102dc507's digest with 1525's count**, and asserts the pair NORMATIVELY in **AC 10**
- the guardrail AC, the headline metric, the number the previous commit message brags is new. **A
test written faithfully to AC 10 asserts 1525 and FAILS ON A CORRECT IMPLEMENTATION.**

## §2.3 - NEW4-002 (MAJOR) - the grammar governs the cone and NOT the exclusion list

§1.3's grammar governs the cone and NOT the exclusion list - which §1.6 Direction 1 and AC 5 both
must read. **Nine root-level exclusions have no valid kind.** Either reading carries a defect.
**NEW3-004's exact shape, one layer deeper.**

*(Cited by `spec.md` source_decisions: rewrite 5 closed this with a fourth kind, `exempt:<glob>` -
see §4.2.)*

## §2.4 - NEW4-003 (MODERATE) - §1.9 enumerates seven arms and misses the eighth

§1.9 enumerates seven arms and misses the eighth - `version.sh [repo]`, where `$here != $CY` and
drift IS reported. **That arm holes Claim 3**: the reference is another install's manifest, not a
payload, so §1.1's licence to trust a stored token rests on a false premise there.

*(Cited by `spec.md` source_decisions: rewrite 5 added the eighth arm and QUALIFIED §1.1's premise
rather than deleting it - see §4.3.)*

## §2.5 - NEW4-004 (MINOR) - the effort breakdown does not sum to the field

The itemised breakdown added to close NEW3-007 **sums to 17.5**; the field says **16**.

## §2.6 - THE PATTERN, FIFTH ROUND RUNNING

A false number in a load-bearing metric. **But it MOVED.** Round 3's came from trusting the evidence
file; round 4's from trusting themselves. The auditor's diagnosis, quoted:

> *"They re-measured everything they INHERITED and did not re-measure what they ORIGINATED."*

Every inherited citation in rewrite 4 was genuinely re-measured and correct. The one number the
author invented was not. **That is a rule worth having: an author's own new numbers are the
least-checked thing in any document, because nothing flags them as needing a check.** (This became
TASK-IMP-124.)

---

# §3 - NEW4-005 and NEW4-006: FINDINGS WITH NO SURVIVING AUDIT

> **Provenance: UNVERIFIABLE. This section is deliberately NOT a reconstruction.**
>
> There is no audit text for these two findings, and this section does not invent one. What is
> recorded below is (a) the finding **as it was acted on**, which does survive, and (b) an explicit
> statement of what cannot be established. That is less satisfying than a plausible round-4 finding
> would be, and it is the more valuable artefact: a reader can tell the difference between what is
> known and what is not.

## §3.1 - What is verifiable

Exhaustive search of every commit message on every ref, every tracked file, the working tree and the
stash. `NEW4-005` and `NEW4-006` appear in exactly three places, **all of them downstream of the
audit that supposedly raised them**:

| where | what it says | is it the audit? |
|---|---|---|
| `15894b1e` message (rewrite 5's **author** notes) | *"NEW4-005/006 fixed though the audit rated them LOW. 006 rated higher on review..."* | **no** - the author's account of an audit |
| `1f8143cf` message (audit **6**'s notes) | *"NEW4-006: the author OUT-AUDITED the auditor..."* | **no** - a later auditor's remark about it |
| `spec.md` `source_decisions:67-68` | *"(rewrite 5, NEW4-005 - **audit rated LOW**, taken anyway)"* and the same for NEW4-006 | **no** - the spec citing the audit |

**And the audit that would have raised them does not contain them.** `7309cd80` - the round-4 audit,
reproduced in §2 - enumerates **NEW4-001, NEW4-002, NEW4-003, NEW4-004, and stops**. There is no
NEW4-005 and no NEW4-006 in it.

## §3.2 - What therefore CANNOT be checked

**`spec.md` attributes both findings to an audit that rated them LOW. That attribution cannot be
verified, and this file does not endorse it.**

The possibilities are not distinguishable from the surviving record:
1. an audit raised them at LOW and was delivered in a prompt and never persisted - the attribution
   is true and simply unprovable;
2. an audit raised them at some other severity and the LOW is misremembered;
3. no audit raised them - they are the author's own findings, later attributed to an audit.

**Nothing on disk or in history discriminates between these.** The one weak signal is that both
`15894b1e` and `1f8143cf` *dispute* the LOW rating (*"006 rated higher on review"*; *"the author
OUT-AUDITED the auditor"*) - which is consistent with a LOW having been given and argued against,
but is equally consistent with a LOW that was never given. It is not evidence.

This is exactly the failure NEW5-007 names (§5.5): **an audit that is not persisted cannot be
checked, and a finding id that is misremembered or invented is indistinguishable from a real one.**
NEW4-005 and NEW4-006 are the class's own worked example. They are preserved as ids so that
`spec.md`'s citations resolve to *this statement of the problem* rather than to nothing - which is
the honest destination for them.

## §3.3 - NEW4-005, as acted on

> Source: `15894b1e` (rewrite 5's author) and `spec.md` `source_decisions:67`. **The finding as
> RAISED does not survive; this is the finding as FIXED.**

The fix: **§1.4 is qualified to the DEFAULT install.** Unqualified, §1.4 is FALSE under
`CYBEROS_NO_MEMORY=1`, which §1.2 itself names - it would make the cone both "exactly the vendored
set" and a strict superset of it, on a documented install mode. §1.7 already carried the qualifier;
§1.4 did not. Cost: one clause.

**Severity: unknown.** `spec.md` says the audit rated it LOW and was overridden ("taken anyway").
Unverifiable per §3.2. On the substance the fix stands on its own reasoning and needs no audit to
justify it: the clause was false under a mode the document itself names.

## §3.4 - NEW4-006, as acted on

> Source: `15894b1e` (rewrite 5's author) and `1f8143cf` (audit 6). **The finding as RAISED does not
> survive; this is the finding as FIXED, plus a later auditor's remark on it.**

The fix: **§1.7 now pins the install ENVIRONMENT for ISOLATION, not only enumeration.**
*"Declining to look at a path does not prevent the write."* `install.sh:634-636` copies skills into
`$HOME/.claude/skills` and three sibling dirs when `CYBEROS_GLOBAL_SKILLS=1` - a write OUTSIDE the
temporary root, on the machine running the build, which no amount of careful enumeration undoes. A
`$HOME` canary was added to AC 7.

**The substance, which IS corroborated by a later audit.** Per `15894b1e` and confirmed by audit 6
(§5.1): the *"zero side effects outside the temp root"* property that the **round-4 auditor VERIFIED
and CREDITED** (§2.1) **was held by no clause** - it was true of the CI environment as it happened to
be, not of anything the spec required. Audit 6's own words: **"the author OUT-AUDITED the auditor."**

**Severity: unknown, and the LOW rating is doubly doubtful here.** `spec.md` says "audit rated LOW";
`15894b1e` says "006 rated higher on review"; audit 6 credits the author with beating the auditor on
it. All three of those are post-hoc accounts of a rating no artefact records. Unverifiable per §3.2.

---

# §4 - ROUND 5: rewrite 5's closure record

> **Provenance: RECONSTRUCTED FROM COMMIT `15894b1e` (2026-07-18); no contemporaneous file exists.**
>
> **This is not an audit.** It is the rewrite-5 **author's** record of closing round 4's findings.
> The independent check of these claims is audit 6 (§5), which re-derived them and confirmed them -
> so they are corroborated, but by §5 and not by anything in this section. Read the claims here as
> claims.

## §4.1 - NEW4-001 - CLOSED (author's claim; confirmed by audit 6 §5.1)
**1525 -> 1534 at both normative sites**, re-derived twice by different methods (whole-cone, and
component-wise 1525 + 3 + 6). **AC 10 now names the trap explicitly so it cannot silently return.**

## §4.2 - NEW4-002 - CLOSED (author's claim; confirmed by audit 6 §5.1)
Added a **FOURTH kind, `exempt:<glob>`**, for paths no `dir:`/`file:` reaches. **Both removing kinds
now carry an enforced invariant** - a `prune:` that removes NOTHING fails the build; an `exempt:`
that WOULD remove something fails the build (it is a `prune:` in disguise). *"That kills the
dead-text charge structurally instead of by promise."* Home settled: cone + exclusions + `_rsha()` in
ONE file, because *"alongside it is not a location and §1.6 cannot classify against a list it cannot
read."* Verified by the author: four kinds classify a real `$CY` with ZERO unclassified paths.

> **This closure is where NEW5-002 came from** (§5.2): the `prune:` invariant this fix introduced
> collides with the `prune:memory/store/` entry §1.5 mandates.

## §4.3 - NEW4-003 - CLOSED (author's claim; confirmed by audit 6 §5.1)
**Confirmed empirically, not reasoned** - `version.sh /tmp/otherrepo` prints `rules_drift` where the
no-arg control prints `up_to_date`. **Eighth arm added.** §1.1's premise **QUALIFIED not deleted**:
*"a token is a faithful digest of the BUILD that wrote it, never of a tree that merely carries it."*

## §4.4 - NEW4-004 - CLOSED (author's claim; confirmed by audit 6 §5.1)
The breakdown was right and **16 was wrong** - *"a round number written beside an itemisation nobody
added up."* **effort 16 -> 20**, rewrite-5 scope priced rather than absorbed.

## §4.5 - NEW4-005 / NEW4-006 - acted on; see §3
Fixed by rewrite 5. **Provenance unverifiable - do not read §4 as establishing that an audit raised
them.** §3 is the record.

## §4.6 - Weak ACs: 3/15 -> 0

## §4.7 - WHAT FIVE AUDITS MISSED: the disclosure was SELF-CERTIFYING

The author's own finding, against no audit's prompting, and the sharpest thing in the round:

> The **AI Authorship Disclosure was itself stale and SELF-CERTIFYING.** It claimed four audits (now
> five) and that *"every numeric claim was re-measured"* - **and rewrite 4's false 1525 shipped
> UNDERNEATH THAT SENTENCE.** The one clause meant to prevent the failure certified it instead.

Now scoped and checkable: **re-derived-and-CONFIRMED** vs **re-derived-and-CORRECTED** vs
**measured-and-ADDED**, and it states that **the audit's own figures were treated as claims to
verify**. (This partition became TASK-IMP-124's COND-004, with rewrite 5's disclosure cited as the
worked prototype.)

---

# §5 - ROUND 6: audit 6, of rewrite 5 - FAIL 8/10

> **Provenance: RECONSTRUCTED FROM COMMIT `1f8143cf` (2026-07-18); no contemporaneous audit file
> exists.** The orchestrator's summary of an independent audit. The auditor's own text is gone.

## §5.1 - Verdict summary

**FAIL 8/10.** Same score as round 4, **categorically smaller reasons**. Remaining work is **four
clause edits, not a rewrite.**

**THE FALSE-NUMBER ERA IS OVER.** Every originated number re-derived, **ZERO false - a first in six
rounds.** What the auditor re-derived and confirmed:
- `1534`/`102dc507` and `1525`/`86cafee8` confirmed **with counts**;
- **all four AC-10 combinations confirmed exactly**;
- **arm 8 run and confirmed**;
- the effort **breakdown sums to 20**;
- *"zero unclassified paths"* **independently reproduced**: `1546` real `$CY` paths = `1534` + 5
  store + 7 exempt-present, **0 unclassified**.

*"The disclosure is now checkable, not self-certifying, and the auditor re-derived every claim in its
CONFIRMED bucket and found them exact."*

**NEW4-006: the author OUT-AUDITED the auditor** - the *"zero side effects"* property the previous
auditor VERIFIED (§2.1) was held by no clause. See §3.4.

## §5.2 - The remaining findings

### NEW5-001 - AC 2 is BYTE-IDENTICAL to rewrite 4's while §1.2 widened to three items
The author **widened the clause to close their own NEW4-002 and never re-read the AC tracing to it.**
*"The pattern, in a new organ."*

### NEW5-002 - §1.3's prune invariant vs §1.5's mandated entry: the build fails unconditionally
`dist/cyberos/memory/store` **does not exist**, so resolved over the payload `prune:memory/store/`
removes **NOTHING** -> §1.3 says **fail the build** -> **the build fails unconditionally on the list
§1.5 mandates.** *"Extensional test, structural rationale; they diverge."*

> The collision is between two of rewrite 5's own fixes: the invariant from §4.2 and the entry from
> §1.5. Closing NEW4-002 opened this.

### NEW5-003 - AC 5 claims nine deletion arms; only 8 are producible
`.install.lock` is **absent**, so the ninth arm cannot be produced.

### NEW5-004 (INHERITED - missed by ALL FIVE prior audits) - AC 15 unsatisfiable
`version.sh:33-37` **forces `CYBEROS_UPDATE_CHECK=always`**, defeating the `:39` throttle -> `:99`
**writes every run**. Measured: cache `1784334942` -> `1784336844` after one `version.sh` run.

> This is the finding round 3's §1.2 logged as *"NEW2-010 (':99 on every run' - THIRD round
> unedited)"*, and which rewrite 4's commit message declared corrected as *":99 is guarded by three
> early returns, not 'every run'"*. **Audit 6 measured it and the original reading was right.** Five
> audits passed over it.

### NEW5-006 - `:199` is the REMOVAL site for `gates.env.bak.*`; creation is `:286-287`
A citation error in §1.5's exclusion list.

## §5.3 - NEW5-005 - **NO SURVIVING RECORD**

**There is no NEW5-005.** Not in `1f8143cf`, not in any other commit message on any ref, not in any
tracked or untracked file, not in either spec. The commit enumerates NEW5-001, NEW5-002, NEW5-003,
NEW5-004, NEW5-006 and NEW5-007 - **it skips 005 without comment.**

**No finding text is invented for it here.** Whether the id was allocated and dropped, allocated and
folded into another finding, or simply misnumbered in the commit message **cannot be determined from
the surviving record.** Nothing cites NEW5-005, so nothing is broken by its absence - it is recorded
because an unexplained gap in a finding sequence is exactly the thing a reader must be able to see.

## §5.4 - NEW5-007 - THE AUDITS WERE NEVER PERSISTED

> Found independently by two agents; verified. This is the finding this file exists to answer, and it
> is the orchestrator's own defect, recorded rather than quietly fixed. Quoted from `1f8143cf`:

> **THE DEFECT** (found independently by two agents; NEW5-007; verified):
> - `NEW4-001` -> in **0 tracked docs**. `NEW-002` -> in **0 tracked docs**.
> - `IMP-122/audit.md` is still the **ROUND-3 file**: verdict FAIL 6/10, "FLAT for four rounds", and
>   an obsolete STOP_SIGNAL, at `b37b795a`. `IMP-121/audit.md` is still the **4/10 file**.
> - `spec.md` cites *"audit rated LOW"* x2 for NEW4-005/006 - **findings that exist NOWHERE in the
>   repo.**
>
> *"I wrote every audit into COMMIT MESSAGES and then handed the findings to the next author through
> my prompt. Each author took my word for findings they could not open. **An audit that is not
> persisted cannot be checked; a finding id that is misremembered or invented is indistinguishable
> from a real one.** This is the exact pattern the whole session is about - an unverifiable claim
> accepted on authority - except it is one level above the three authors' and I was narrating the
> rule while committing it. Recorded here rather than quietly fixed. The audit files must be
> reconstructed from 7309cd80 / 15894b1e / this commit before any of these three specs is trusted."*

**Status: this file is that reconstruction** (2026-07-18). It does not close NEW5-007 - the
underlying process defect (audits authored into commit messages) is a workflow matter, and the
information destroyed by it (§3, §5.3) is not recoverable by writing this file. What it does is make
every id cited in `spec.md` resolve to a section, and make the unrecoverable parts legible as
unrecoverable.

## §5.5 - Required before re-audit (round 6's list)

1. **NEW5-001** - re-derive AC 2 against §1.2's widened three-item clause.
2. **NEW5-002** - reconcile §1.3's prune invariant with §1.5's mandated `prune:memory/store/`; the
   build currently fails unconditionally.
3. **NEW5-003** - AC 5's ninth deletion arm is not producible; `.install.lock` is absent.
4. **NEW5-004** - AC 15 is unsatisfiable while `version.sh:33-37` forces `always`.
5. **NEW5-006** - fix the `:199` / `:286-287` citation.

Four clause edits, not a rewrite. Not promoted; no BACKLOG row.
