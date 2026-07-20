---
task_id: TASK-IMP-121
audited: 2026-07-18 (audit 2, of the FIRST rewrite). NOT an audit of HEAD's spec.md - see §0.
verdict: FAIL
score: 6/10
score_history: "4/10 -> 6/10. The fix that followed is UNAUDITED - there is no third audit."
issues_open: "NEW-001..006 are CLAIMED CLOSED by the third author (§3). No auditor has checked that claim."
issues_resolved: "audit 2 resolved ISS-001, ISS-003, ISS-004, ISS-005 (DISSOLVED by the frame replacement, not patched) and ISS-006 (all six citations fixed). ISS-002's shape RECURRED as NEW-003."
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: "task-lint clean at every round. At HEAD (the unaudited third rewrite): 10 clauses / 10 ACs / 9h."
auditor: "audit 1: independent subagent + author verification of the two load-bearing claims against source. audit 2: independent subagent; the orchestrator separately verified NEW-001 against source."
current_state: >
  HEAD's spec.md is the THIRD author's rewrite (commit 1f8143cf). It claims all six of audit 2's
  findings closed. THAT CLAIM IS UNAUDITED. No independent auditor has read HEAD's spec. This file
  therefore records NO VERDICT AND NO SCORE for HEAD - the 6/10 above is audit 2's verdict on the
  draft that PRECEDED the fix, and applying it to HEAD would be wrong in both directions. §3 records
  what the third author claims and what corroborates it; it does not convert that into a verdict.
reconstruction_notice: >
  §2 and §3 were RECONSTRUCTED on 2026-07-18 from commit messages (15894b1e, 1f8143cf). No
  contemporaneous audit file exists for either - they were never persisted. §1 (audit 1, 4/10) is
  the only section written at the time it was performed. Per-section provenance is stated at the
  head of every section.
citation_hazard: >
  Audit 2's findings (§2) cite the clause and AC numbers of the FIRST REWRITE - a 9-clause / 9-AC
  document. HEAD's spec.md has 10 clauses and 10 ACs and they DO NOT LINE UP. "§1.5" in NEW-002
  is not HEAD's §1.5. Do not resolve §2's citations against HEAD. See §0.3.
---

# TASK-IMP-121 - audit record

## §0 - Provenance, and why this file had to be reconstructed

**This page was rebuilt on 2026-07-18. Read this section before trusting any finding id on it.**

The 6/10 audit was written into a COMMIT MESSAGE and never into this file, which sat at the 4/10 version (`6a146869`) while two rewrites happened against findings no one could open. The defect is recorded as **NEW5-007** on TASK-IMP-122's audit page. This file is the remedy for 121's half of it.

### §0.1 - What kind of section you are reading

| kind | sections | what you are reading |
|---|---|---|
| **written at the time** | §1 (audit 1, 4/10) | the auditor's own audit file, committed at `6a146869`, preserved verbatim |
| **reconstructed from commit** | §2 (audit 2, 6/10) | the orchestrator's *summary* of an audit. The audit's own text does not exist. |
| **not an audit at all** | §3 (the third author's fix) | the **author's** closure claims. **Nobody audited this.** |

### §0.2 - The rounds, mapped

| section | what it is | source | persisted at the time? | verdict |
|---|---|---|---|---|
| §1 | audit 1, of the first draft | `6a146869` (this file) | **YES** | FAIL 4/10 |
| §2 | audit 2, of the first rewrite | `15894b1e` message | no | FAIL 6/10 |
| §3 | the third author's fix - **the current spec.md** | `1f8143cf` message | no | **NONE. UNAUDITED.** |

**All six ids audit 2 raised - NEW-001 through NEW-006 - resolve to §2.** Neither `spec.md` cites a finding id by name (verified: `grep -oE 'NEW-[0-9]{3}|ISS-[0-9]{3}'` on both specs returns nothing for 121), so no citation in the task depends on this page. `source_decisions` refers to the rounds by score - *"2026-07-18 audit FAIL 4/10"* (§1) and *"2026-07-18 audit FAIL 6/10"* (§2) - and both now resolve.

### §0.3 - A citation hazard a reader will otherwise walk into

**Audit 2's clause and AC numbers do not resolve against HEAD's `spec.md`.**

Audit 2 examined the **first rewrite**: 9 clauses, 9 ACs, `effort 5 -> 8` (per `7309cd80`). HEAD is the **third author's** whole-document rewrite: 10 clauses, 10 ACs, `effort 8 -> 9`. The renumbering between them was never mapped, and §2's citations are preserved **as the auditor wrote them** rather than silently re-pointed - re-pointing them would be a reconstruction of a mapping nobody recorded.

Concretely: NEW-002 is about *"§1.5's six channel dirs"* and *"AC 5"*. In HEAD, the six-channel container rule is **§1.6** and its test is **AC 6**; HEAD's **§1.5** is the operator-edited-`.mcp.json` rule and its **AC 5** is that rule's test. **Reading NEW-002 against HEAD's §1.5/AC 5 will mislead you.** The same hazard applies to NEW-001 (§1.6/AC 6/AC 7 as numbered in the first rewrite; the byte-exact strip rule is §1.7 in HEAD) and NEW-004 (§1.3 vs §1.8).

---

# §1 - AUDIT 1: audit of the first draft - FAIL 4/10

> **Provenance: WRITTEN AT THE TIME.** This is the audit file as committed at `6a146869`
> (2026-07-18). It is the only section of this page that is an auditor's own artefact. Preserved
> verbatim; nothing below this line in §1 has been edited.
>
> **Status: superseded as a verdict, and its findings are resolved** - audit 2 (§2) records
> ISS-001/003/004/005 as **DISSOLVED** by the frame replacement rather than patched, and all six of
> ISS-006's citations as fixed. **ISS-002's shape RECURRED** on a new input as NEW-003 (§2.4).

Audit 1's own header block, as written:

```yaml
task_id: TASK-IMP-121
audited: 2026-07-18
verdict: FAIL
score: 4/10
issues_open: 6
issues_resolved: 0
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084. The floor passing is why this
  audit matters: every finding below is a JUDGMENT defect the mechanical check cannot see.
auditor: independent subagent (had not seen the author's reasoning) + author verification of the
  two load-bearing claims against source
```

## §1.1 - Verdict summary

FAIL at 4/10. The four underlying defects are REAL and reproduced by the harness. The spec diagnoses two of them from a false premise, carries the TASK-IMP-118 defect class in its own AC 1, and cites six line numbers that do not resolve. Not a patch: a rewrite against the true mechanism.

## §1.2 - Findings (ALL OPEN)

### ISS-001 (CRITICAL) - the premise ".cyberos is removed" is false in the default path
§1.1/§1.3 are written against "the removed `.cyberos/`". VERIFIED against source: `uninstall.sh:151` runs `rm -rf "$CY"` and `:155-158` then runs `mkdir -p "$root/.cyberos/memory"` + `mv "$KEEP_BRAIN_STASH" "$root/.cyberos/memory/store"`. **`.cyberos/` survives** with BRAIN inside unless `CYBEROS_UNINSTALL_KEEP_BRAIN=0`. The harness's own fresh-git run listed `./.cyberos/memory/store/*` among the leftovers - the author had the evidence and wrote the clause against the opposite state.

### ISS-002 (CRITICAL) - AC 1 asserts a different predicate than §1.1 (the TASK-IMP-118 defect)
§1.1 verb: a symlink "may remain [only if its] `readlink` target [does not resolve] inside the removed `.cyberos/`". AC 1 asserts: "zero unresolvable targets". These diverge BOTH ways:
- false negative: with BRAIN restored (ISS-001), a symlink into a SURVIVING `.cyberos/` path resolves fine -> passes AC 1 -> violates §1.1.
- false positive: an operator's own broken symlink pointing OUTSIDE `.cyberos/` is unresolvable -> fails AC 1 -> but §1.2 requires it be KEPT. AC 1 and AC 2 contradict on that input. The author wrote both the clause and its test, and the test is weaker. This is precisely §15.2.

### ISS-003 (CRITICAL) - §1.3/§1.4 are undecidable; the enabling mechanism is rejected in-spec
Both condition on "if install created it". Nothing on disk records that: `install.sh` tracks creation in a shell variable that dies with the process; the only ownership marker (`.cyberos-owned`) is written ONLY into skill copy dirs, never onto `.gitignore` or `.mcp.json`. The spec's Proposed Solution permits only "readlink target inside `.cyberos/`, or our marker" - `readlink` is meaningless on a regular file and no marker exists - while Alternatives REJECTS "Track install's creations in a manifest" as scope creep. The spec forbids the only mechanism that would make its own clauses decidable.

### ISS-004 (CRITICAL) - §1.4 contradicts §1.7 and the zero-casualties guardrail
An operator who ran `touch .gitignore` before install is byte-indistinguishable from install's `: > "$gi"`. After block-strip the file is empty; §1.4 says it MUST NOT survive -> uninstall deletes an operator's file -> §1.7 ("No path present before install may be missing") violated. §1.7 admits no exception and §3 does not cover the case. Same hazard for `.mcp.json`: install points operators at `.cyberos/mcp/README.md` for hand-registration, so an operator-authored `.mcp.json` is a supported path.

### ISS-005 (MAJOR) - section 6 already ACCEPTS the dangling links; the spec frames it as oversight
VERIFIED: `uninstall.sh:162` reads `# 6. skill symlinks into .cyberos (dangling) - leave dirs; operator cleans`. The dangling is KNOWN and DELIBERATE in source. The spec's Problem section says four channels "are never looked at" - true of the code path, false of the intent. This is a recorded decision to OVERTURN with reasons, not a gap to fill. The operator approved overturning it (2026-07-18) on the readlink-proves-ownership argument, which stands - but the spec must argue against section 6 explicitly rather than not notice it.

### ISS-006 (MAJOR) - six citations do not resolve
- `:70` cited as "(block strip)" and as the proven newline leak -> `:70` is `echo "  stripped cyberos block from pre-commit"`. The sed is `:68`. This is the spec's single most load-bearing citation and it points at the echo, not the mechanism.
- `:81` cited as the .gitignore strip -> `:81` is the echo; the strip is `:80`.
- `:114` cited BOTH as the ship-tasks exemption AND as the readlink test it exempts from. One line cannot be both. The `[ "$_sc" != "ship-tasks" ]` guard is `:112`.
- `:111` cited as an exemption whose rationale is "avoid clobbering operator files" -> `:111` is a comment; it records no such rationale.
- `:105-107` cited as a prune precedent -> it is the "kept unmarked skill dir" echo branch. The actual prune is `:118-119` and has NO install-created check.
- `:92-93` resolves but is a comment naming 3 paths, cited to cover 7.

## §1.3 - Verified accurate (credit)
- `uninstall.sh` never mentions mcp/codex/grok/commandcode/opencode - grep returns zero. The five-channel and dead-registration findings are REAL.
- The +1-newline-per-cycle diagnosis is mechanically correct and fixable from `uninstall.sh` alone: install appends `\n# >>> cyberos-status-hook v2`, and `:68`'s sed range starts AT the marker, so the blank separator above it survives.
- 121's reading of TASK-IMP-106 AC 3 is accurate; `depends_on: [TASK-IMP-106]` correctly serialises.
- TRACE-003 passes: `test_install_hygiene.sh` exists and is declared in modified_files.

## §1.4 - Required before re-audit
Rewrite §1.1/§1.3 against the surviving-.cyberos reality; re-derive AC 1 to assert §1.1's actual predicate; resolve ISS-003 by either scoping the creation-manifest IN or narrowing the clauses to what readlink/marker can decide; reconcile §1.4 with §1.7; argue against section 6 explicitly; fix all six citations. Residual noted: after 121 re-points t22, TASK-IMP-106 §1.5 has no test left that verifies it, and a test named `t22_uninstall_behavior_unchanged` whose baseline moved asserts the opposite of its name.

---

# §2 - AUDIT 2: audit of the first rewrite - FAIL 6/10

> **Provenance: RECONSTRUCTED FROM COMMIT `15894b1e` (2026-07-18); no contemporaneous audit file
> exists.** What follows is the orchestrator's summary of an independent audit. The auditor's own
> text is gone. NEW-001 carries a note that the orchestrator verified it against source
> independently (*"I verified this myself, it is exact"*); the other five carry no such note.
>
> **The subject was the FIRST rewrite** (9 clauses / 9 ACs, `7309cd80`), written by a fresh author
> who had written none of the failing draft. **It is not HEAD.** See §0.3 before resolving any
> clause number below.

## §2.1 - Verdict summary

**FAIL 6/10** (from 4/10). **The frame-replacement does real work:** ISS-001/003/004/005 are **DISSOLVED not patched**, all six inherited citations fixed, every inherited number reproduced exactly. **It fails on what the author ORIGINATED.**

## §2.2 - NEW-001 (CRITICAL) - the impossibility proof is FALSE

> The orchestrator's note records this as independently verified: *"I verified this myself, it is
> exact."*

§3 claims an operator hook with no trailing newline *"cannot be restored byte-exact - the information is destroyed at append time and no uninstall-side rule inverts it"*. **FALSE.** The information survives as the presence/absence of the blank line above the marker, and install ALWAYS prepends exactly one `\n`, so the byte before the marker is always ours. Measured:

```
line-oriented strip (the spec's):  6B -> 7B  DIFFERS
byte-oriented rule (awk, no perl): 6B -> 6B  BYTE-EXACT   <- inverts it
same rule, newline-terminated:     7B -> 7B  BYTE-EXACT
```

**A LEVEL CONFUSION:** the author reasons in LINES, and in line-space the no-newline case has no blank line to consume - so they generalised *"my line-oriented rule cannot"* to *"no rule can"*.

**§1.6 is NARROWED on that false proof**, which excuses a live §1.8 violation (an operator's hook permanently mutated +1 byte) and scopes AC 6/7 so the suite can never catch it.

## §2.3 - NEW-002 (MAJOR) - AC 5 is UNSATISFIABLE

§1.5 requires each of §1.1's **six** channel dirs to survive and forbids removing a container for emptiness. **`uninstall.sh:118` `rmdir`s `.agents/skills` - the FIRST of the six - for exactly that.** Verified. §3 calls it *"mild tension"* and re-scopes §1.5 to five **IN PROSE ONLY**; the clause and the AC still say six.

**Aggravating:** an operator's pre-existing EMPTY `.agents/skills` is deleted - a §1.8 violation **AC 8's four cases miss**.

## §2.4 - NEW-003 (MAJOR) - AC 1 and AC 2 contradict: ISS-002's exact shape on a new input

§1.1's chained-form pattern **widens the hazard from the 3 names install writes to EVERY entry in the dir**, destroying the ours-by-construction justification while keeping the pattern. §3's *"carried forward unchanged"* is false.

> ISS-002 (§1.2) is the same defect class - clause and test diverge, author wrote both - recurring
> after being dissolved once.

## §2.5 - NEW-004 (MAJOR) - §1.3 vs §1.8: ISS-004's shape relocated

From container-DELETION to content-MUTATION. **The §1.3/§1.4 split fixed the drafting, not the substance:** an operator's hand-registered `.mcp.json` byte-identical to `:685`'s form triggers §1.3, which then **mutates a path present before install**.

## §2.6 - NEW-005 (MINOR) - a SEVENTH mis-citation, ORIGINATED

`:684` is an if-guard; the summary is `:689` - **under a disclosure reading "every line number was re-verified"**. The six inherited citations were all genuinely fixed; the seventh is the author's own.

## §2.7 - NEW-006 (MINOR) - the receipt rejection's conclusion stands but one universal is false

Under `CYBEROS_COPY_SKILLS=1` the copy **is** decidable from a receipt and from nothing else - which is §3's own conceded gap. The conclusion (reject the receipt) survives; the universal supporting it does not.

## §2.8 - THE PATTERN, ACROSS SEVEN AUDITS AND THREE AUTHORS

Recorded by the orchestrator alongside this audit, because 121 is the round that made it legible:

```
round 3 (me): patched what the audit NAMED, did not re-read     -> 4 findings survived verbatim
round 4:      re-read everything, false NUMBER of their own      -> 1525
121 rewrite:  fixed every citation, false REASONING of their own -> the impossibility proof
```

> One rule underneath all three: **AUTHORS DO NOT CHECK WHAT THEY ORIGINATE.** An inherited claim
> carries a provenance that invites scrutiny; an originated one arrives already believed. It is
> TASK-IMP-118's defect class one level up - there, an author's test asserts something weaker than
> their own clause; here, an author's evidence never gets tested at all. Both are the same failure:
> **the author is the wrong reader of their own work.**

This became **TASK-IMP-124**. Note that 124 **rejected** rotate-the-author using the evidence offered FOR it: both fresh-author rounds shipped an originated falsehood. *"Rotation changes who INHERITS; it does not make anyone check what they ORIGINATE."*

---

# §3 - THE THIRD AUTHOR'S FIX: CLAIMED CLOSED, **UNAUDITED**

> **Provenance: RECONSTRUCTED FROM COMMIT `1f8143cf` (2026-07-18); no contemporaneous file exists.**
>
> **THIS IS NOT AN AUDIT AND THIS SECTION RECORDS NO VERDICT.** It is the third author's account of
> closing audit 2's six findings, relayed through the orchestrator's commit message. **HEAD's
> `spec.md` is this rewrite. No independent auditor has read it.**

## §3.1 - The status, stated exactly

**The six findings NEW-001..006 are CLAIMED CLOSED. The fix is UNAUDITED.**

- **No third audit exists.** Not in this file, not in any commit message, not anywhere.
- **The closure claims are the author's own** - which is precisely the category audit 2's own §2.8 identifies as the least-checked thing in any document. The rule the round established applies to the round's own output.
- **`score_history` therefore stops at 6/10.** The 6/10 is audit 2's verdict on the draft that PRECEDED this fix. It is **not** HEAD's score, and this file does not assign HEAD one. A reader who needs a verdict on HEAD must commission an audit; there is nothing here to read one off.

**Two things partially corroborate the claims, and neither is an audit:**
1. The orchestrator reports reproducing NEW-001's disproof independently: *"line-strip 6B->7B DIFFERS, byte-strip 6B->6B EXACT, controls 7B/9B exact."* That is one finding's evidence re-measured, not a review of the document.
2. The machine floor passes: `task-lint` clean, 10 clauses / 10 ACs / 9h. **Audit 1's own header says why that is worth little here:** *"The floor passing is why this audit matters: every finding below is a JUDGMENT defect the mechanical check cannot see."*

## §3.2 - What the third author claims, per `1f8143cf`

**All six findings closed.** Specifically recorded:

- **NEW-001.** Reproduced the disproof independently and **retracted §3's impossibility**. Then found **the impossibility IS REAL - on the OTHER FILE**: `install.sh:733-746`'s awk collapses `*.log\n` (6B), `*.log\n\n\n` (8B) and `*.log` (5B) **all to 6B**. Three distinct pre-install states, one post state. **A true pigeonhole, install-side, correctly scoped out.**
- It **tested its own new rule and found its limit** - operator content BELOW our block with no trailing newline: **want 17B, got 18B** - and **recorded it rather than papering it**. (This is the §2.8 rule being applied by an author to their own work, which is the first instance of it in the session.)
- **§1.9's carve-out was FORCED, not chosen:** §1.9 contradicted **§1.1**, not only §1.4 - an operator's pre-existing symlink whose target names our machine is a path present before install that §1.1 removes. **The 6/10 audit found the §1.4 collision and MISSED the §1.1 one.**
- **effort 8 -> 9.**

## §3.3 - OPEN: needs an operator verdict, not an auditor's

**Bringing `uninstall.sh:118-119` in scope** - keeping six channel dirs rather than scoping §1.6 to five - **is the author's decision and is NOT covered by the recorded 2026-07-18 PLAN gate.** Flagged by the author in the spec's own AI Authorship Disclosure. This is a scope question for the operator and an audit cannot settle it.

## §3.4 - Also filed against this round

`1f8143cf` records a finding about **TASK-IMP-118**, raised while filing TASK-IMP-124 and belonging to neither 121 nor 122: **TASK-IMP-118 declares `modified_files: tools/install/docs-tools/templates/ task-audit/RUBRIC.md` - WHICH DOES NOT EXIST** (verified). TRACE-003 covers test paths only, so an originated citation about the author's own cone is checked by nobody. Recorded here only so it is not lost with the commit message; **it needs its own task and is not 121's.**

## §3.5 - Required before this task can be trusted

1. **Audit HEAD.** The six closures are unverified. This is the whole of what is missing.
2. **Get the operator's verdict on §3.3** - `:118-119`'s scope is outside the PLAN gate.
3. Note for whoever audits: **§2's clause numbers do not resolve against HEAD** (§0.3). The mapping from the 9-clause draft to HEAD's 10 was never recorded and is not reconstructed here.

Not promoted; no BACKLOG row.
