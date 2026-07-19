---
task_id: TASK-IMP-124
audited: 2026-07-18 (audit 1, of the draft at `1f8143cf`)
verdict: FAIL
score: 7/10
score_history: "7/10 (audit 1 - no prior audit of this task)"
issues_closed: n/a (first audit)
issues_open: "6 new: 2 MAJOR-a (design), 2 MAJOR-b (self-application), 2 MODERATE. Plus 4/10 weak ACs."
weak_acs: "4/10 (AC 4, AC 6, AC 8, AC 10)"
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: "task-lint CLEAN (zero findings). 10 clauses / 10 ACs / 13h. TRACE-001/002/003 pass."
auditor: >
  Independent subagent; wrote none of TASK-IMP-118/121/122/124. Re-derived every originated
  numeric claim from source, re-ran the awk disproof from scratch, and resolved every sha and
  file:line the spec cites. Method: the spec's own TRACE-007 turned on the spec.
headline: >
  ZERO false numbers. Every load-bearing derivation reproduces exactly: 1525/86cafee8,
  1534/102dc507, 1525+3+6=1534, the four awk measurements, effort 13=13, the ledger 5+2=7.
  This task passes its own NUMERIC class. It fails its own CITATION class, in four places,
  and the (b) half has a design tension its own §1.10 cannot survive.
STOP_SIGNAL: >
  Do NOT close these findings by patching the named lines. Six of the seven defects below are
  the SAME defect - 124 applied each rule it originates to the instance that motivated it and
  not to the class it names. Patching the four bare paths I name will leave the fifth. Re-read
  §1.2 against the whole document, not against my list. This is the failure mode of every
  round of 121 and 122 in this session (`TASK-IMP-122/audit.md` §2), and 124 is the task that
  generalises it.
---

## §1 - Verdict summary

FAIL at 7/10 - the highest first-round score in this session, and it fails anyway.

What the author got right is not small and I will not bury it. **Every originated numeric claim
in this document is TRUE and I reproduced each one from source.** Round 4 of TASK-IMP-122 shipped
a false number at 8/10 under a sentence certifying it had not; 124 shipped none. The awk disproof
is real - I wrote it from scratch without reading the author's and got the same four measurements
on the same gawk version. The effort breakdown sums to the field, which is the exact defect 122
carried for five rounds (NEW4-004). The dead path in `TASK-IMP-118/spec.md:22` is real. The
task-id reasoning is right. The seven-audit ledger is right. `git cat-file -e` resolves all six
shas. task-lint is clean.

It fails on four things:

1. **§1.6 and §1.10 cannot both be satisfied** (F-1). The rule forbids "an unscoped universal
   attestation"; §1.10 mandates it PASS rewrite 5's disclosure; rewrite 5's disclosure *contains*
   a universal attestation nearly identical to rewrite 4's. "Unscoped" is never defined. The rule
   either fails its own worked prototype - which the spec itself calls "mis-drafted" - or it
   passes the sentence it exists to kill.
2. **124 breaks its own §1.2 CITATION rule four times** (F-2), including in AC 10, its acceptance
   evidence. All four have already gone stale - three of them *while I was auditing*.
3. **§1.8's safeguard fires only on failure** (F-3). A GREEN COND-004 emits no message at all, so
   nothing names it "shape-only" in exactly the state where "green = disclosure verified" is the
   misreading. This is 118's warning landing, not being answered.
4. **§1.2's NUMERIC class has no revision requirement** (F-4), and the one numeric claim 124
   pinned to a moving "HEAD" is already false - and its drift is now *masked*.

The lint departure is **not principled, but it is also not harmful** (§5). The (c) rejection is
**sound** and I am not going to pretend otherwise (§6).

## §2 - PART A: every originated claim, re-derived

Method: I ran each derivation myself before reading the author's value. `HEAD` = `1f8143cf`.
The spec derives at `15894b1e`; where the two differ I report both.

### §2.1 - The cone claims (the heart of the task)

```
$ cd dist/cyberos
$ _rsha() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"; else shasum -a 256 "$@"; fi; }
$ find cuo plugin mcp lib docs-tools -type f | LC_ALL=C sort | wc -l
1525
$ find cuo plugin mcp lib docs-tools -type f | LC_ALL=C sort \
    | while IFS= read -r f; do _rsha "$f"; done | _rsha | cut -d' ' -f1
86cafee837d48c952535fb19072be686de9bb2d20c08bfc4c8788ef7ac893bde
$ mandated | wc -l                     # + memory minus memory/store/ + the six root scripts
1534
$ mandated | while IFS= read -r f; do _rsha "$f"; done | _rsha | cut -d' ' -f1
102dc507c4c5207eb93abcf3372dd2d3d88482d464fd04c8983636bc27593703
$ find memory -type f | grep -vc '^memory/store/'
3
$ ls install.sh uninstall.sh version.sh status.sh help.sh check-latest.sh | wc -l
6
```

| claim | spec | measured | verdict |
|---|---|---|---|
| rejected cone count | 1525 | **1525** | VERIFIED |
| rejected cone digest | `86cafee8` | **86cafee8**37d48c95… | VERIFIED |
| mandated cone count | 1534 | **1534** | VERIFIED |
| mandated cone digest | `102dc507` | **102dc507**c4c5207e… | VERIFIED |
| `memory` minus `memory/store/` | 3 | **3** | VERIFIED |
| root scripts present | 6 | **6** | VERIFIED |
| `1525 + 3 + 6 = 1534` | 1534 | **1534** | VERIFIED |
| `_rsha()` defined at `build.sh:353` | :353 | **:353** (`grep -n '_rsha() {'`) | VERIFIED |

All four cone values reproduce at HEAD as well as at `15894b1e`. The arithmetic composition and
the whole-cone measurement agree independently - the spec derived it twice by different methods
and both are right. **This is the claim class that broke round 4, and 124 is clean on it.**

### §2.2 - The anti-example citations (pinned)

```
$ git show 63705483:…/TASK-IMP-122…/spec.md | sed -n '280p'
- [ ] AC 10 … asserting the measured `102dc507` on both sides over 1525 files per side …
```
VERIFIED - the mandated cone's digest paired with the rejected cone's count, normatively, in the
guardrail AC. `280 - 242 = 38`: the spec's "thirty-eight lines above it" is exact.

```
$ git show 15894b1e:…/TASK-IMP-121…/spec.md | sed -n '168p'
…The information is destroyed at append time, so no uninstall-side rule can invert it -
measured today at 16 -> 17 bytes with the candidate strip.
$ … | sed -n '49p'
…is NOT recoverable for a hook with NO trailing newline (16 -> 17 bytes). See §3.
```
VERIFIED verbatim, both lines, both carrying the 16 -> 17.

The drift claim is VERIFIED and it is not rhetorical - I reproduced it:
`sed -n '168p'` in the working tree returns **AC 4**, an unrelated clause; `:178` carries the
RETRACTION, independently reaching the same four measurements. The pin was necessary and it holds.

`git cat-file -e` resolves all six: `63705483` `15894b1e` `f8899d64` `e2504cf3` `7309cd80`
`2d478393`. VERIFIED.

### §2.3 - The awk disproof (re-run from scratch)

I did not read the author's awk before writing mine. `install.sh:851-853` confirmed by
`cat -A`: `cat >> "$hk" <<'HOOK'` at :851, **blank line at :852**, marker at :853 - so the
append is `\n` + marker, exactly as claimed.

```
no trailing newline          orig=6B | line-> 7B DIFFERS     | byte-> 6B BYTE-EXACT
newline-terminated           orig=7B | line-> 7B BYTE-EXACT  | byte-> 7B BYTE-EXACT
three-newline-terminated     orig=9B | line-> 9B*            | byte-> 9B BYTE-EXACT
GNU Awk 5.1.0, API: 3.0
```
VERIFIED. The byte-oriented rule inverts the append on all three shapes; `cmp` is silent. The
impossibility claim at `15894b1e:…121…:168` is **FALSE**, and the spec's disproof of it is
correct. (*My crude line-oriented reconstruction diverges on the 9B control; that is my rule,
not the spec's, and the load-bearing 6B->7B DIFFERS reproduces exactly.)

**On the portability caveat (§3, `source_pages`): the author is RIGHT and the caveat is
correctly dispositioned.** `RS="\0"` is a gawk extension and is untested on BSD/macOS awk - but
the claim under disproof is a UNIVERSAL NEGATIVE ("*no* uninstall-side rule can invert it").
Disproving a universal negative requires **one** witness. A gawk-only witness is a rule, and it
inverts. Portability would matter to a *fix* for 121 and is immaterial to the *disproof* - and
§3 says precisely that, and explicitly declines to recommend it to 121. Marking it "not derived"
rather than asserting it is the correct call. **NOT a defect.**

### §2.4 - The remaining originated claims

| # | claim | my command | result | verdict |
|---|---|---|---|---|
| 1 | `grep -c TRACE-006 …/RUBRIC.md` = 0 | same | **0** | VERIFIED |
| 2 | COND-004 at `RUBRIC.md:62`, three bullets, nothing constrains Scope | read :62 | exact | VERIFIED |
| 3 | COND-004 implemented at `task-lint.mjs:447-457`, labels at `:452` | read | exact | VERIFIED |
| 4 | TRACE-001..005 at `:108-112`; TRACE-003 at `:110` = §5 test paths only | read | exact | VERIFIED |
| 5 | `118/spec.md:22` declares `…/templates/task-audit/RUBRIC.md` | `sed -n '22p'` | exact | VERIFIED |
| 6 | that path does not exist | `ls …/templates/` | No such file or directory | VERIFIED |
| 7 | `118:50-52`, `:134-136`, `:168-170` quotes | read all three | exact | VERIFIED |
| 8 | `backlog-mutate next-id` not implemented | ran it | `unknown command 'next-id'` | VERIFIED |
| 9 | 123 retired, dropped at `f8899d64` | `git log --diff-filter=D` | names `TASK-IMP-123…/spec.md` | VERIFIED |
| 10 | `TASK-IMP-105 §1.5` forbids reuse | read :96 | **see F-6** | PARTIAL |
| 11 | ledger: 5 + 2 = seven | audit.md + 2 commits | **7** at HEAD | VERIFIED |
| 12 | `15894b1e` says "SEVEN AUDITS AND THREE AUTHORS" | grep commit body | line 61, exact | VERIFIED |
| 13 | effort breakdown sums to 13 | `2+1.5+1+1.5+1+.5+3+2+.5` | **13.0** = field | VERIFIED |
| 14 | 183 `done` specs | `measure-phase.mjs` | `done_total = 183` | VERIFIED |
| 15 | "three authors" used nowhere normative | grep §1/§2 | absent | VERIFIED |
| 16 | phase-corpus re-run prints 550 | `measure-phase.mjs` | **551** | **FALSE at HEAD** - see F-4 |

**Fifteen of sixteen VERIFIED. One FALSE, and it was TRUE when written** (`git ls-tree -r
--name-only 15894b1e -- docs/tasks | grep -c '/spec\.md$'` → **550**). See F-4 for why that is
still a finding.

## §3 - PART D: where the pattern is in THIS document

It is in the same place every time, and it is not one line - it is a **rule applied to its
instance and not to its class**. 124 originated three rules. For each, the author fixed the case
that bit them and left the case that had not yet bitten:

| rule 124 originates | the instance that bit the author | the class the author left unchecked |
|---|---|---|
| §1.2 CITATION - pin the revision | `121/spec.md:168` went stale mid-draft → **pinned** | `122/audit.md:6`, `:12`, `121/audit.md:5`, **AC 10's own `121/spec.md:168`** → all bare, all now stale (F-2) |
| §1.2 NUMERIC - carry a re-running command | the cone counts → **carried, and correct** | the phase-corpus `550`, pinned to a moving "HEAD" → already false (F-4) |
| §1.6 - no unscoped universal attestation | rewrite 4's sentence → **named and forbidden** | rewrite 5's near-identical sentence, which §1.10 mandates PASS (F-1) |

The spec says it at :65: *"An originated claim arrives already believed."* The author derived
that sentence, published it, and then believed §1.2 was done because the one citation that broke
in front of them was fixed. **Nothing in this document flags a rule as needing to be applied to
itself** - which is precisely the gap 124 exists to close, at the layer 124 occupies.

The disclosure at :147 is the tell. It is scrupulous - it partitions correctly, it marks two
things as not-derived, it names the corrections. And it says: *"Every `file:line` above was
opened and read at HEAD `15894b1e`."* That is TRUE. It is also **the unscoped universal
attestation §1.6 forbids**, in the disclosure of the task that forbids it. Every file:line *was*
opened. Opening them is not what §1.2 requires - **pinning** them is, and four are not pinned.
The attestation certified the reading and not the rule. Rewrite 4's sentence certified the
measuring and not the origination. Same sentence, same failure, one layer deeper.

## §4 - New findings

### F-1 (MAJOR) - §1.6 and §1.10 cannot both hold; "unscoped" is undefined and does the work of the whole rule.

§1.6 requires COND-004 to *"forbid an unscoped universal attestation of the form 'every claim was
re-measured'"*. §1.10 requires the amended COND-004 to **PASS** rewrite 5's disclosure. I read
both sentences at their pinned revisions:

```
rewrite 4 (63705483:242-243) - the sentence §1.6 exists to kill:
  "Every numeric and line-number claim was re-measured against source at HEAD by this
   author during this rewrite - including those inherited from the evidence file, four
   of which were wrong (build.sh:357, memory/store/ 3-vs-8, …)."

rewrite 5 (15894b1e:289-291) - the disclosure §1.10 mandates PASS:
  "This disclosure is therefore scoped deliberately: every number in this document was
   re-derived from source at HEAD by this author during rewrite 5, including the ones
   this author invented and including the ones the round-4 audit supplied"
```

These are the same sentence. Both are universal ("Every numeric … claim" / "every number in this
document"). Both carry an "including …" qualifier that **emphasises** rather than restricts.
Rewrite 5's differs in exactly one respect: its qualifier names the ORIGINATED set as well as the
inherited one. Rewrite 5 asserts it is "scoped deliberately" - but asserting scope is not scope,
which is 124's own thesis.

So amended COND-004 must do one of two things, and both are fatal:
- **fail rewrite 5** → §1.10 is violated, and the spec's own words apply: *"one that fails its own
  worked prototype is mis-drafted"* (:161). The acceptance evidence is unsatisfiable.
- **pass rewrite 5** → the prohibition cannot be triggered by a universal attestation per se, so
  rewrite 4's disclosure **plus three labels** also passes. The sentence that certified the false
  1525 survives the rule written to kill it, wearing a partition.

Nothing in §1.6, §1.2, §1.7 or §3 defines "unscoped". AC 6 cannot rescue it: it tests only that
the prohibition **text is present** in the rubric ("the test MUST FAIL if … the prohibition is
absent"), never that the prohibition discriminates the two sentences. AC 10 is the only arm that
could - and it is MANUAL, so it hands a real auditor a term with no criterion.

The defensible reading exists and the spec does not state it: a universal attestation is "scoped"
**iff it is accompanied by an enumeration that makes it falsifiable** - which is what §1.7's two
findings actually test (a CONFIRMED value no derivation reproduces; a governed originated claim in
no set). Under that reading rewrite 5 passes, rewrite 4 fails, and the prohibition has teeth.
**Say it in §1.6.** As drafted the rule's central term is a mood.

Required: define "unscoped" operably in §1.6, and make AC 6 assert the definition discriminates
`63705483:242-243` from `15894b1e:289-291`. Those two strings are the rule's real test.

### F-2 (MAJOR) - 124 breaks its own §1.2 CITATION rule four times, including in its acceptance evidence.

§1.2: *"a bare path into a moving document does NOT discharge it."* §1.4: anti-examples must cite
*"the REVISION the claim shipped in rather than a path"*. AC 4 even makes it a test: *"MUST FAIL
if either … cites a bare path."*

124 cites four bare paths into documents it **knows** are moving (it says so at :87, :136, §3:179):

| site | citation | value as cited | at HEAD | in the worktree, now |
|---|---|---|---|---|
| `source_pages`:48 | `TASK-IMP-122/audit.md:6` | `4/10 -> 6/10 -> 6/10 -> 6/10` | same ✓ | `… -> 8/10 -> 8/10` **DRIFTED** |
| Problem:93 | `TASK-IMP-122/audit.md:12` | "task-lint clean, four rounds running" | same ✓ | "…At rewrite 5: 15 clauses / 15 ACs / 20h" **DRIFTED** |
| `source_pages`:48 | `TASK-IMP-121/audit.md:5` | `score: 4/10` | same ✓ | `score: 6/10` **DRIFTED** |
| **AC 10**:174 | `TASK-IMP-121/spec.md:168` | the impossibility proof | — | **AC 4**, unrelated **ALREADY DEAD** |

All four were true when written. **Three drifted during this audit** - `git status` shows both
audit.md files ` M` and their line 5/6/12 changed between my first read and my last. I did not
construct this; I tripped over it.

The fourth is the serious one. **§1.10 says `15894b1e:spec.md:168`. AC 10 says
`TASK-IMP-121/spec.md:168`.** The AC drops the revision its own clause mandates, on the citation
the whole §1.2 CITATION class was written because of. An implementer following AC 10 today
re-audits **AC 4 of the current 121 spec** and finds no universal negative there, because the
claim moved to `:178` and was retracted. The acceptance evidence points at the wrong text.

And the ledger consequence is already live: with the worktree's audit 6, `5 + 2 = seven` becomes
`6 + 2 = eight`. "Seven" is non-normative (§2.4 #15 confirms), so this is not a false number - but
§3:192's *"The audit COUNT (seven) is derived and holds"* is an originated claim whose derivation
rests on three bare paths that no longer resolve to the cited values.

Required: pin all four to `15894b1e` (or `1f8143cf`). AC 10 must carry the revision §1.10 carries.

### F-3 (MAJOR) - §1.8's shape-only safeguard is structurally unable to fire in the state it exists to protect.

§1.8: *"its finding message MUST name the check as shape-only"* - the mitigation for exactly the
hazard 118 §1.5 names, that a green floor is read as verification.

I read the implementation the spec cites (`task-lint.mjs:447-457`):

```js
const missing = ["Tools used:", "Scope:", "Human review:"].filter(…);
if (missing.length > 0) {
  finding(findings, "error", "COND-004", …);   // <- the ONLY message COND-004 ever emits
}
```

**COND-004 emits a message only when it FAILS.** A passing disclosure produces silence - I ran
task-lint on 124 itself and it printed nothing at all. So the sentence that names the check
"shape-only" appears **only to an author who is already being told they are wrong**, and never to
the reader of a green run. The misreading §1.8 guards against - *"a green lint is never read as a
verified disclosure"* (:112) - occurs precisely when no message exists.

AC 8 inherits the hole: *"its message contains 'shape'"* is assertable only on the failing arm.
Nothing in §1.8 or AC 8 constrains what a GREEN COND-004 communicates, because a green COND-004
communicates nothing, which is the problem.

This is not fatal - §1.7 forbids the auditor crediting the disclosure, and that is the real
defence. But §1.8's stated mitigation does not mitigate. Required: put the shape-only disclaimer
where a passing run can be read (the rule text in RUBRIC.md §4 and the SKILL.md instruction), and
drop the claim that a finding message carries it. Or have the check report shape-only status on
both arms.

### F-4 (MAJOR) - §1.2's NUMERIC class has no revision requirement; 124's own numeric example is already false, and its drift is now MASKED.

§1.2 requires the CITATION class to name a revision "where the cited document is itself in
flight". **It imposes no such requirement on the NUMERIC class** - only "a command, a script
invocation, or an arithmetic composition … that reproduces the value". Reproduces *when*? AC 2
confirms the asymmetry: it asserts *"the citation row names the revision requirement"* and asks
nothing of the numeric row.

The spec's own numeric derivation demonstrates the hole. `source_pages`:47 and Problem:91:
*"Re-running it at HEAD prints specs_total=550."*

```
$ node docs/tasks/_audits/measure-phase.mjs "$(pwd)"
specs_total             = 551          <- NOT 550
$ git ls-tree -r --name-only 15894b1e -- docs/tasks | grep -c '/spec\.md$'
550                                    <- true at the revision the spec derived at
$ git diff --name-only --diff-filter=A 15894b1e HEAD -- 'docs/tasks/*/*/spec.md'
docs/tasks/improvement/TASK-IMP-124-authors-do-not-check-what-they-originate/spec.md
```

**Filing this task is what broke its own derivation.** "At HEAD" meant `15894b1e` when written and
means `1f8143cf` now, and the corpus went 551 (`e2504cf3`) → 550 (`f8899d64` dropped IMP-123) →
551 (`1f8143cf` added IMP-124).

That is worse than a stale number, and §3 misses why. §3's edge case anticipates *"a derivation
that re-runs and reproduces a DIFFERENT value"* → STALE-001, derivation did its job. But here the
re-run reproduces **the SAME value as the stale table** - 551 = 551 - **for a different reason**.
Two errors cancelled. A stranger running the Reproduce line today gets a **false all-clear**: the
file's 551 counts a spec that no longer exists and misses one that does.

So the spec's worked precedent for its whole thesis - *"The number that carried its derivation was
checkable by a stranger in seconds"* (:91) - **is, at HEAD, not checkable by a stranger at all.**
The one property the phase-corpus file was held up for is the one it no longer has.

Required: give the NUMERIC class the revision requirement the CITATION class has (a count is a
claim about a tree at a revision), extend AC 2 to assert it, and add the masked-drift row to §3 -
"reproduces the stale value for a different reason" is a distinct and worse case than "reproduces
a different value".

### F-5 (MODERATE) - the "departure from TASK-IMP-118 §1.5" is not a departure, and the clause it departs from is not blanket.

Both halves of the author's argument verify, and I checked them:
- *"any TRACE-006-shaped lint would have PASSED 108 §1.7"* - **TRUE.** 118's Problem: the test was
  `grep -q '"draft_staleness"'` and the string genuinely is in the HTML, inside a JSON blob no
  code reads. A structural check passes it.
- *"the shape check FAILS rewrite 4's disclosure … and PASSES rewrite 5's"* - **TRUE, measured.**
  `git show 63705483:… | sed -n '238,250p' | grep -ci 'CONFIRMED\|CORRECTED\|ADDED'` → **0**.
  Rewrite 5 carries all three.

The claims are true. The **inference** is not, for two independently sufficient reasons.

**(a) There is no departure.** COND-004 is *already* in task-lint - the spec says so itself at
`source_pages`:43 and I confirmed it at `task-lint.mjs:447-457`. `:452` is literally
`["Tools used:", "Scope:", "Human review:"].filter(…)`. The entire "departure" is **adding three
strings to an existing array in an existing mechanical check**. 118 §1.5's text binds one rule:
*"TRACE-006 MUST be judgment-family and MUST NOT be added to task-lint."* It says nothing about
COND-004, a §4 structural rule that has been linted all along. 124 does not lint TRACE-007 -
Alternatives:111 explicitly refuses to. **118 §1.5 was never in the way.**

**(b) 118 §1.5 is not "blanket".** :54 calls it *"TASK-IMP-118 §1.5's blanket 'no lint'"*. Its
text at `118/spec.md:134-136` is scoped to TRACE-006 by name; its Non-Goal at `:113-114` is
scoped to *"this rule"*. **This is TRACE-007 condition 4 committed against 118 - a derivation
that supports a narrower scope than the claim asserts - inside the argument for departing from
118.** The 121 §3 defect, one document later.

And the discrimination, though real, is **degenerate**: rewrite 4's disclosure fails the label
check because the labels did not exist when it was written. *Every* disclosure in the corpus fails
it except rewrite 5's, which invented them. That is not discrimination between a defective
document and a sound one - it is discrimination between pre-rule and post-rule documents. The
actual defect (the false 1525) **passes** the shape check: a labelled disclosure with 1525 sitting
in CONFIRMED is green. The catch is entirely audit-side (§1.7), as §1.8 concedes.

**None of this makes the check wrong.** A shape check is legitimate *scaffolding* - it forces the
partition into existence so §1.7 has something to test. That is a good reason and the spec should
give it. What it should not do is dress scaffolding in 118's test and report a pass, because 118's
test asks whether a check can fail the motivating **defect**, and this one cannot.

Required: drop the departure framing (there is nothing to depart from), state the real
justification - COND-004 is already mechanical and the labels are a precondition for §1.7's
audit-side test - and stop claiming the shape check discriminates the motivating case.

### F-6 (MODERATE) - two originated citations that do not reproduce from the command the spec gives.

**(a) The `:242` quote.** `source_pages`:38 and Problem:74 both say `sed -n '242p'` returns:
> "Every numeric and line-number claim was re-measured against source at HEAD by this author
> during this rewrite."

It does not. I ran it:
```
$ git show 63705483:…/TASK-IMP-122…/spec.md | sed -n '242p'
  not a patch. Every numeric and line-number claim was re-measured against source at HEAD by this
```
The line begins mid-sentence ("not a patch.") and **ends mid-sentence** ("by this"). The quoted
sentence spans `:242-243` and does not stop where 124 stops it - it continues: *"- including those
inherited from the evidence file, four of which were wrong (`build.sh:357`, `memory/store/`
3-vs-8, "three separate places", `audit-fleet.sh:19`)."* 124 presents a truncation as what ":242
reads", closed with a period that is not in the source.

By 124's own §1.2 this is a CITATION whose derivation **does not reproduce** the claim, and by
§1.4 an anti-example must quote *"the false claim verbatim"*. It is also inconsistent with the
document's own practice - 124 cites ranges everywhere else (`:50-52`, `:108-112`, `:168-170`,
`:279-304`, `:447-457`).

The omission is material, and it cuts against the author. The dropped clause shows rewrite 4's
attestation reaching **toward the inherited set** - which is *better* evidence for 124's thesis
(round 4 scoped its diligence to what it inherited, and the originated 1525 fell outside) than the
truncated version. The author cut the sentence that helped them, to make it look flatter than it
is. That is what F-1 then trips over.

Fix: cite `63705483:242-243`, quote the whole sentence, and let it do more work.

**(b) `TASK-IMP-105 §1.5 forbids reuse.**` (`source_decisions`:51). `105/spec.md:96` reads:
> *"`next-id` MUST ignore gaps: it returns highest+1, never the lowest free number, because reusing
> a retired id makes two different tasks share a name in the history."*

The **rationale** forbids reuse; the **rule** is "return highest+1". With 123 retired and 122 the
highest survivor, `highest+1` = **123** - the retired id. Applied literally, 105 §1.5 hands out
exactly the id its rationale forbids. There is no "gap" to ignore here; 123 is past the end, not
interior.

The author's **decision is right** (use 124) and rightly motivated by the rationale. The
**citation** is loose: §1.5 is a rule about a tool's return value, not a prohibition addressed to
an author, and citing it as "forbids reuse" cites the *because* clause as the norm. Minor as
stated - but it surfaces a real latent bug worth its own task, and 124 is the document that
noticed it: **`next-id`, when implemented per 105 §1.5 as written, will hand out 123.** The spec
verified `unknown command 'next-id'` and stepped around the hazard by hand without filing it.

## §5 - PART B: clause-verb table - 4 of 10 weaker

| # | clause VERB | what the AC actually asserts | verdict |
|---|---|---|---|
| 1.1 | RUBRIC §9 MUST **CARRY** TRACE-007 naming four conditions, in the judgment family | AC 1: row exists, names all four, names the sequence, **MUST FAIL if it appears in a mechanical family** | **EQUAL** |
| 1.2 | MUST **DEFINE** three classes, each with both halves | AC 2: defines three, both halves, + UN names the own-candidate case, + citation row names the revision | **EQUAL** (but see F-4: numeric row has no revision requirement to assert) |
| 1.3 | MUST **DEFINE** originated + **REQUIRE** partition, originated first | AC 3: defines, requires, **MUST FAIL if partition described without the ordering** | **EQUAL** |
| 1.4 | MUST **CARRY** two anti-examples, each *"giving the command that reproduces it"* | AC 4: command **present**; 1525/1534/102dc507/86cafee8 **appear**; shas resolve via `git cat-file -e` | **WEAKER** (minor) - a wrong command beside correct literals passes; the AC never re-runs the cone |
| 1.5 | text MUST **STATE** kinship + **CITE** TRACE-006 by rule_id | AC 5: both, **MUST FAIL if either half alone** | **EQUAL** |
| 1.6 | COND-004 MUST **FORBID** an unscoped universal attestation | AC 6: the **prohibition text is present** in the rubric, and names 1525 as its reason | **WEAKER** - PRESENT-IN-DOCUMENT standing for FORBIDS. 118 §1.7's exact shape. Cannot discriminate R4 from R5 (F-1) |
| 1.7 | SKILL.md MUST **INSTRUCT** the auditor to test the disclosure | AC 7: instructs, names both findings, **MUST FAIL if the skill instructs the auditor to credit** | **EQUAL** |
| 1.8 | lint MUST **CHECK ONLY** shape · MUST **NOT EXECUTE** · message MUST **NAME** itself shape-only | AC 8: fails/passes the label arm; **sentinel-file fixture** + `exec`/`spawn`/`eval` grep for the negative; message **contains "shape"** | **WEAKER** (minor) - verbs 1 and 2 tested well (the sentinel is a real negative arm); verb 3 tested by substring, and untestable on the green arm (F-3) |
| 1.9 | SKILL.md MUST **INSTRUCT** recording at origination | AC 9: instructs, names the moment, **MUST FAIL if deferred to review time** | **EQUAL** |
| 1.10 | amended rubric MUST **FAIL** `63705483:280` · **FAIL** `15894b1e:spec.md:168` · **PASS** `15894b1e:spec.md:279-304` | AC 10: recorded re-audit **returns** FAIL/FAIL/PASS - but on **`TASK-IMP-121/spec.md:168`**, a bare path that already resolves to AC 4 | **WEAKER** - drops the revision the clause mandates; the object is wrong, so the evidence cannot be gathered as written (F-2) |

**Weaker count: 4 of 10** (AC 4, AC 6, AC 8, AC 10). Two are minor (4, 8); two are substantive
(6, 10), and both sit on the (b) half and its acceptance evidence.

For calibration: rewrite 4 of 122 scored 3/15 weak; rewrite 5 scored 0/15. 124 at 4/10 is a
weaker ratio than either, on a first round.

### AC 10's `verify: MANUAL` - honest, and I checked whether it was an excuse

**It is honest, and it is the best clause in the document.** I went looking for the excuse and
found the opposite.

The author's argument: a shell test asserting a recorded audit *says* FAIL is weaker than the
clause's verb, i.e. the 118 defect inside the task that generalises it. Test that against 118
itself:

> `118/spec.md:168-170` - **AC6**: re-auditing 108 §1.7 against the amended rubric FAILS on the
> original test and PASSES on the replacement. … **Test: `t06_rule_fails_the_case_that_motivated_it`.**

**118 declares a SHELL TEST for exactly this.** TRACE-006 is unmechanizable by 118's own §1.5 -
and 118's AC6 hands its verification to a named shell function, which can only assert that some
recorded artefact contains a string. That is the 108 §1.7 defect (`grep -q`, the string is in the
file) committed inside the task that generalises it, in the acceptance criterion that says *"a
rule that cannot fail its own motivating case is decoration"*.

124 saw that and refused to copy it. AC 10 going MANUAL is the **only** formulation equal to
§1.10's verb: a rule "fails" a document when an auditor applying it returns FAIL, and no shell
test can stand in for that without asserting something strictly weaker. **124 is right, 118 is
wrong, and 124 is the only document in this chain that noticed.** Credit where due.

One nit, not a finding: TRACE-002 permits manual verification *"only for ops/UI flows that can't
be automated, and must justify why"*. A judgment rule is not an ops/UI flow, so AC 10's MANUAL sits
outside TRACE-002's carve-out as written. 124 justifies why, which is the spirit. TRACE-002's
carve-out is too narrow for judgment-family rules and should be widened - **that is TRACE-002's
defect, not 124's**, and worth its own task.

## §6 - PART C: is the rule well-designed?

**Is TRACE-007 mechanizable enough to be checkable without pretending a test exists?** Yes, and
this is handled well. The split is clean: shell tests assert the rubric *carries* the rule text
(AC 1-9); the rule *firing* is AC 10, MANUAL and honestly labelled. Nothing claims a test where
none exists. §1.8 forbids the lint executing derivations, with a real sentinel-file negative arm
in AC 8 and a SAFE-001..004 rationale that is correct - a spec body is untrusted input and running
its commands would be a genuine vulnerability. **Sound.**

**Is "supports a narrower scope than the claim asserts" operable, or a judgement dressed as a
check?** **Operable - it is the strongest condition in the rule.** It is not "is this claim too
broad?" (unanswerable); it is "compare the QUANTIFIER of the derivation against the QUANTIFIER of
the claim" (mechanical to state, judgment to apply, exactly like TRACE-006). It has a worked
procedure: 121's §3 gives the template - the derivation quantifies over *one candidate rule*
("the candidate strip"), the claim quantifies over *all rules* ("no uninstall-side rule"), and
those two quantifiers are visible in the text without re-running anything. The `echo 1525` edge
case (§3) sharpens it further: a command that derives a literal never derives a count. **I applied
this condition three times in this audit** (F-5 against 124's own "blanket", F-6b against 105
§1.5, and the R4/R5 comparison in F-1) and it worked every time without ambiguity. That is the
test of an operable rule.

**Does the UNIVERSAL NEGATIVE witness-search requirement work, given you cannot prove a negative
by failing to find a witness?** **Yes - because it does not try to.** This is the objection to
beat and the spec beats it, though it never says so out loud. Read what §1.2 actually requires:
*"a record of the witness search naming what was tried and how each failed, and the audit MUST
attempt one counter-example before accepting it."* The rule is **asymmetric by construction**:
- finding a witness **DISPROVES** the negative outright - one witness is a proof, and that is
  exactly what happened to 121 (4 lines of awk).
- failing to find one **does not prove** the negative; it converts the claim into *"I searched
  here and here and found nothing"*, which is a **scoped** claim with a stated search space.

That is the whole mechanism, and §3 confirms the reading: *"'I could not measure this' is a
derivation of a different kind and discharges the rule; 'no rule can' is a universal negative and
does not."* The rule never licenses concluding the negative - it forces the author to either
publish a bounded search or stop asserting the unbounded claim. The asymmetry is doing real work:
the cost of the search is paid by the claimant, and the auditor's single counter-example attempt
is a cheap high-yield check (it cost this session's 121 auditor four lines and it toppled a
load-bearing proof). **Sound - but §1.2 should state the asymmetry explicitly**, because an
auditor reading it quickly may believe a clean search *establishes* the negative, which is the
one thing it must never do.

**Does (b) close the self-certification hole or relocate it?** **It relocates it, and narrows
it.** The partition is genuinely more falsifiable than "every claim was re-measured" - §1.7's two
findings (a CONFIRMED value no derivation reproduces; a governed originated claim in no set) are
real, checkable operations, and the second is the one that catches 1525. But the hole is not
closed: the author still decides *which* claims are governed and *which* set each lands in, and a
claim the author does not recognise as originated lands in no set **silently**. §1.7 makes that a
finding only if the auditor independently identifies it as governed and originated - which is the
work TRACE-007 exists to prompt, so the two halves lean on each other. That is acceptable and the
spec says as much (:52: "Neither works alone"). What is **not** acceptable is F-1: as drafted, the
prohibition that was supposed to do the narrowing has no operable content, so (b) currently
relocates the hole without narrowing it. Fix F-1 and this becomes sound.

**Is the (c) rejection sound, or too clever? n=2.** **SOUND. I tried to break it and could not.**

First the facts, verified:
- `63705483` - *"IMP-122 rewrite 4, written by a fresh author because the previous one had a
  pattern"* → fresh author, shipped the false 1525. ✓
- `7309cd80` - *"IMP-121 rewrite by a fresh author"* → fresh author, shipped the false
  impossibility. ✓

Both fresh-author rounds shipped an originated falsehood. 2 for 2.

**n=2 is not load-bearing, and that is why the argument survives.** The rejection does not rest on
the sample - it rests on an **analytic** claim: rotation redistributes *whose* claims are
inherited and *whose* are originated. It cannot reduce origination, because **someone must
originate**. A rotated author, by construction, originates more (a fresh author rewrites more than
they inherit) and re-derives the inherited set diligently - leaving a *larger* unchecked
originated set than the author they replaced. So rotation is not merely orthogonal to this defect;
it plausibly **worsens** it while improving the score, which is precisely what the ledger shows
(4/6/6/6 → 8, with a false guardrail number). The two measurements corroborate an argument that
would hold at n=0.

**Is the author "using the evidence FOR rotation to reject it"?** No - and this is the move worth
defending. They are distinguishing **two defects** that one body of evidence speaks to
differently: rotation demonstrably fixes *"the author patches only what the audit names"* (the
flat line broke), and demonstrably does not fix *"the author does not check what they originate"*
(both rotated rounds shipped one). Same evidence, two questions, two answers. That is not rhetoric,
that is disaggregation - and the spec **keeps rotation as practice** (:109, :138) rather than
rejecting it. It rejects it *as this task's answer*. That is the correct scope, and it is a
narrower claim than the one the evidence would license.

I looked for the version of (c) that would survive: *"a second author checks the first's
ORIGINATED claims"*. That is not rotation of the rewrite author - it is an independent derivation
check, which **is** TRACE-007's audit-side requirement. (c) as scoped ("the rewrite author must
not be the draft author") genuinely does not touch it. The rejection stands.

**Does anything in 124 CONTRADICT TRACE-006 / TASK-IMP-118?** No contradiction - one
mischaracterisation (F-5: "blanket") and one improvement 118 should adopt (AC 10's MANUAL, vs
118's AC6 shell test). §1.5's kinship is stated correctly and the verb/scope contrast is exact:
TRACE-006 fails a test asserting less than its clause's VERB; TRACE-007 fails a derivation
supporting less than its claim's SCOPE. §3 row 7 correctly inherits 118 §3 row 5 (two verbs, both
compared) and AC 8 honours it. §3's DEGRADATION rows correctly inherit 118 §3 rows 10 and 11.
The out-of-scope treatment of the corpus sweep follows 118's and 117's precedent. **124 is a
faithful extension of 118, and is more rigorous than 118 in the one place they overlap.**

## §7 - Required before re-audit

1. **F-1** - define "unscoped" in §1.6 operably (proposed: a universal attestation is scoped iff
   accompanied by an enumeration that makes it falsifiable). Make AC 6 assert the definition
   discriminates `63705483:242-243` from `15894b1e:289-291`. **Those two strings are the rule's
   real test** - if the rule cannot separate them, it is not a rule.
2. **F-2** - pin all four bare paths: `122/audit.md:6`, `122/audit.md:12`, `121/audit.md:5`, and
   **AC 10's `121/spec.md:168`**. AC 10 must carry the revision §1.10 already carries.
3. **F-3** - move the shape-only disclaimer somewhere a GREEN run can be read (rubric text +
   SKILL.md). Stop claiming a finding message carries it; a passing COND-004 emits nothing.
4. **F-4** - give the NUMERIC class the revision requirement the CITATION class has; extend AC 2
   to assert it; add the **masked-drift** row to §3 ("reproduces the stale value for a different
   reason" ≠ "reproduces a different value"). Re-derive the phase-corpus claim at a pinned
   revision - it is 550 at `15894b1e` and 551 at HEAD **because this task was filed**.
5. **F-5** - drop the "departure from 118 §1.5" framing entirely. COND-004 is already linted at
   `task-lint.mjs:447-457`; 118 §1.5 binds TRACE-006 only and is not blanket. State the real
   justification: the labels are a precondition for §1.7's audit-side test.
6. **F-6** - cite `63705483:242-243` and quote the full sentence including *"- including those
   inherited from the evidence file"*; it strengthens the thesis. Re-word the 105 §1.5 citation to
   cite the rationale as rationale.
7. **AC 4 / AC 8** - AC 4 should re-run one cone command rather than assert four literals appear.
   AC 8's message assertion should name the shape-only phrase, not substring "shape".
8. **File separately** (do not absorb): (a) TRACE-003 → `modified_files`, already named in §Scope
   and confirmed live at `118/spec.md:22`; (b) **`next-id` will hand out the retired 123** when
   105 §1.5 is implemented as written; (c) **TRACE-002's manual carve-out is too narrow for
   judgment-family rules** - AC 10 is right and TRACE-002 does not permit it.

## §8 - A note to the next author

You are going to be tempted to close F-2 by pinning the four paths I named. **Do not stop there.**
Re-read §1.2 against the whole document and find the fifth, because there is always a fifth - that
is what this task is about, and it is the failure mode `TASK-IMP-122/audit.md` §2 recorded across
four rounds: *close the named finding, introduce the same defect class one layer deeper.*

The document's own sentence is the instruction: **an originated claim arrives already believed.**
Every rule in this spec is originated. Apply each of them to this spec before you apply them to
the rubric - not to the instance that motivated the rule, but to the class the rule names. The
disclosure's *"Every `file:line` above was opened and read at HEAD `15894b1e`"* is true, and it is
the sentence §1.6 forbids, and four of those file:lines are not pinned. **That is the whole task,
in its own disclosure, in one sentence.**
