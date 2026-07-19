# TRACE-006 corpus-sweep sizing (2026-07-18)

Sizing measurement for the TRACE-006 corpus sweep that TASK-IMP-118 deferred. IMP-118 shipped the
rule (`modules/skill/task-audit/RUBRIC.md` §9 TRACE-006 — for every §1 clause that cites a test, the
audit MUST name the clause's VERB and what the cited test ASSERTS, and FAIL when the assertion is
weaker than the verb) but scoped the re-audit of the rest of the `done` corpus out: *"Re-auditing the
other 180 done tasks against TRACE-006 … is a corpus sweep and its own decision … Sizing it belongs in
the handoff."* This file is that sizing — so the operator can decide **whether and how** to run the
sweep. **It is NOT the sweep.** No task is re-audited, no status is flipped, no `.audit.md` is written,
no verdict is rendered on any task.

Per the session's standing finding — authors do not check what they originate — **every number below is
re-derived from a stated command, not estimated and not recollected.** IMP-118 said "180"; a stale task
note said "179". Both are wrong at this HEAD; the re-derived figure is **187 done / 166 in scope** (§1).

Measured at HEAD `a644019e`. Repo root assumed as `$(pwd)`. Commands are grep/awk over
`docs/tasks/**/spec.md` plus one ad-hoc Node pass (regexes stated in the Method note; not committed —
this finding ships exactly one file per the sweep-sizing constraint).

## The question

A full TRACE-006 sweep would re-audit every `done` task that has a §1 clause citing a test, comparing
each such clause's verb to what its cited test asserts. **How big is that job, and where does the risk
concentrate** — so the operator can pick batch size and decide all-at-once vs. high-risk-verbs-first?

---

## 1. POPULATION — 187 done, 166 in TRACE-006 scope

**Done total (two independent methods, reconciled to the same number).**

```
# (a) BACKLOG index rows:
grep -cE '^- \[done\] ' docs/tasks/BACKLOG.md
#   -> 187   (the Totals header also states "187 done")

# (b) frontmatter, corpus-wide:
grep -rl '^status: done$' docs/tasks --include=spec.md | grep -oE 'TASK-[A-Z]+-[0-9]+' | sort -u | wc -l
#   -> 187

# reconcile the row's own id (first id per row) against frontmatter — both gaps empty:
comm -23 <(grep -rl '^status: done$' docs/tasks --include=spec.md | grep -oE 'TASK-[A-Z]+-[0-9]+' | sort -u) \
         <(grep -E '^- \[done\] ' docs/tasks/BACKLOG.md | sed -E 's/^- \[done\] (TASK-[A-Z]+-[0-9]+).*/\1/' | sort -u)
#   -> (empty)  and the reverse -> (empty)
```

Both methods agree exactly: **187 done.** Two apparent discrepancies were chased and closed: (i) a
naive id-grep over BACKLOG done rows yields 188 because `TASK-EMAIL-011`'s row title *mentions*
`TASK-PORTAL-008` ("…chained memory audit hashes for TASK-PORTAL-008 bundle") — PORTAL-008 itself is
`draft`, not done; (ii) `TASK-TPL-001` (docs/tasks/templates/) is a genuine `done` task indexed in the
BACKLOG, not a fixture. Neither changes the total.

**Of the 187, which are TRACE-006-relevant** — i.e. carry a §1-style normative clause block AND cite at
least one test. TRACE-006 only examines §1 clauses that cite a test (RUBRIC §9 scopes it to the
cyberos §1/§4/§5 grammar; a clause with no cited test is TRACE-001/004 territory). Measured by an
ad-hoc Node pass (Method note below) that extracts each spec's §1 block and counts test citations
(`#[test]`/`#[tokio::test]`, `Test:` lines, `traces_to`, `path.rs::fn`, and test-file path references
`tests/…​.{rs,sh,py,ts,tsx,mjs}`):

| metric | value |
|---|---|
| `done` total | **187** |
| carry a §1 normative clause block | **186** |
| **cite ≥1 named/path test → TRACE-006 population** | **166** |
| type mix of the 166 | 125 `feature` · 39 `improvement` · 2 `chore` |

**The 166 by module:** improvement 35 · ai 22 · memory 21 · skill 20 · proj 18 · auth 17 · cuo 14 ·
email 5 · docs 5 · mcp 3 · chat 3 · obs 2 · templates 1. (Full ID list attached at the end.)

**The 21 that fall out, and why** (this is the population boundary, stated honestly — see §"could not
measure cleanly"):

- **1 has no §1 clauses at all:** `TASK-CUO-301` is a `type: bug` on the bug template
  (Reproduction / Root cause / Fix / Regression test) — no BCP-14 §1 clauses, so nothing for TRACE-006
  to examine. Correctly out.
- **20 have §1 clauses but verify them via CI-gate greps / acceptance fixtures / manual runs rather
  than a named or file test my detector could bind:** `APP-003/004/005/006` (codesign/spctl + CI diff),
  `CHAT-101` (`cargo test`, no test path), `CUO-205/208` + `SKILL-117` (`acceptance/*.md` case-tables),
  `DOCS-002`, `IMP-071/072/073/074/075/076/077/078/079/080/081` (`.github/workflows` grep-gates and
  operator-observed runs). A **maximal** sweep could still point TRACE-006 at these (their clauses are
  real), but most have **no test file to open** — their "test" is often itself a grep-in-payload, which
  is the weaker-assertion smell TRACE-006 targets, not a place to read one. Folding all 20 in gives an
  **upper bound of ~186**; the defensible core a sweep would principally read is **166**.

**Known-positive anchor:** `TASK-IMP-108` (the §1.7 render-vs-present-in-payload case that motivated
TRACE-006) **is `done` and IS in the 166** — confirmed present in the relevant set. It is the
calibration case: its §1.7 "MUST render a staleness report" cites
`tools/docs-site/tests/test_render_status_hub.sh::t11_draft_staleness_report`, whose original
assertion was `grep`-in-payload (TRACE-006 FAIL) and whose replacement asserts visible markup
(TRACE-006 PASS). A sweep that cannot re-flag its pre-fix form is miscalibrated.

---

## 2. CLAUSE / TEST VOLUME — the unit of work

The unit of TRACE-006 work is one **verb-vs-assertion comparison per §1 clause that cites a test.**
Restricted to the 166-spec population (Node pass; §1 block extracted per spec, clause = a numbered
item `N.` / `N.N` / `- N.N`, counted when it carries a BCP-14 keyword):

```
# §1 BCP-14 clauses across the 166  (MUST / MUST NOT / SHALL / SHOULD / SHOULD NOT / MAY):
#   -> 2210    (~13 clauses/spec)
# test-citation signals across the 166:
#   672  #[test] / #[tokio::test] named functions
#   405  traces_to:  (explicit clause->AC/test links; present in the newer 46 specs)
#   185  path.rs::fn  citations
#  1061  test-file path references (tests/…​.{rs,sh,py,ts,tsx,mjs})
```

**Order of magnitude:** ~**2,200 §1 clauses** across 166 specs is the ceiling of comparisons. Not every
clause cites a test (deferred/structural clauses are TRACE-006-exempt), so the true comparison count is
somewhat below that; the 672 named tests + 405 explicit `traces_to` links bound the "definitely has a
nameable, bound test" subset from below. **A full sweep is a low-thousands of clause-comparisons, not
tens of thousands.** Only 46 specs carry machine-readable `traces_to` clause→test links; the older 120
bind clauses to §5 tests by AC position and prose, which is why the exact clause↔test map cannot be
machined and the comparison must be read (§"could not measure cleanly").

---

## 3. VERB DISTRIBUTION — where the danger concentrates

TRACE-006 names six recurring high-risk verbs (render · reject · refuse · halt · emit · preserve — "the
recurring cases, not a closed set"), where a weaker test is common and dangerous. Count of §1 clauses
in the 166 population whose text contains each verb stem (Node pass; word-boundary stems e.g.
`reject|rejects|rejected|rejection`, `refus(e|es|ed|al)`, `preserv(e|es|ed|ation)`):

| high-risk verb | clauses | what a weak test looks like (per RUBRIC verb→evidence table) |
|---|---:|---|
| **emit** | **410** | emitting fn was *called* / intent recorded — vs. the row actually at its sink |
| **reject** | **102** | a log line saying "rejected" / happy path — vs. a non-zero exit / 4xx / raised error |
| **render** | **63** | value present in a data payload no view reads — vs. present in the rendered DOM (the 108 case) |
| **refuse** | **60** | caller was told — vs. the guarded effect did NOT happen AND a refusal was signalled |
| **preserve** | **41** | value still exists / was copied — vs. byte/value-equal before-vs-after |
| **halt** | **9** | a warning printed while execution continued — vs. execution stopped before the guarded effect |
| **union (≥1 of the six)** | **624** | ~28% of the 2,210 §1 clauses touch a high-risk verb |

**The risk signal:** ~**624 of ~2,210** §1 clauses (~28%) sit in the danger zone. **`emit` dominates it
(410, ~66% of the danger-zone)** — expected in an audit-chain / OTel-heavy corpus — and is exactly the
"called the emit function" vs. "the audit row / metric is observable at its sink" trap. The **acute
security/observable verbs** — refuse + reject + render + halt + preserve = **275 clauses** — are the
sharp end, where a weaker test is not just imprecise but a shipped hole (a security refusal discharged
by a log line; a render discharged by present-in-payload). (Superset caveat in §"could not measure
cleanly": a stem match is a superset of "the clause's *operative* verb is X", so 624 is an upper bound.)

---

## 4. FACE-VALUE TRIAGE — 10 high-risk clauses (SMELLS, not verdicts)

Ten clauses using a high-risk verb, spread across ai / cuo / improvement / skill (+ auth, memory) and
old / new. For each: the verb, and — **from the cited test's name / one-line AC description ONLY, without
opening the test body** — whether the test OBVIOUSLY matches the verb, or the assertion strength cannot
be told without opening it. **These are smells to estimate quick-clear-vs-deep-read fractions. They are
NOT TRACE-006 verdicts — a verdict requires reading the test body, which is the sweep itself, the
operator's call.**

| task · clause | age | verb | cited test (name / AC one-liner) | smell |
|---|---|---|---|---|
| AI-007 §1.6 | old | preserve | `cost_table_test.rs::hot_reload_invalid_preserves_cache` | **quick-clear (looks-match)** — name encodes "invalid → preserves cache" |
| AUTH-001 §1.14 | old | reject | `admin_tenant_create_test.rs::create_tenant_rejects_reserved_root_slug` | **quick-clear (looks-match)** — name encodes reject + the exact input |
| AI-001 §1.3 | old | refuse | AC#2 "Refuse on over-budget… MUST return `Refuse{BudgetCapExceeded}`; MUST NOT insert any hold row; handler returns 402" (`cost_precheck_test.rs`) | **quick-clear (looks-match)** — AC one-liner names both arms (402 + no side-effect) |
| SKILL-101 §1.1 | old | emit | `memory_audit_test.rs` (+ panic / concurrent / trace variants) | **lean-match, verify** — an audit_test reads the sink; the "BEFORE dispatch" ordering half is invisible by name |
| IMP-108 §1.7 | new | render | `test_render_status_hub.sh::t11_draft_staleness_report` (AC6 "renders drafts by reason and age; no status change") | **deep-read** — the ANCHOR; name is compatible with BOTH a render assertion and the original present-in-payload grep. Known: pre-fix weaker, post-fix matches |
| CUO-101 §1.13 | old | render | `test_supervisor_graph.py`, `test_applier_paths.py` | **deep-read** — generic graph tests; "disclosure surfaced in the return value a caller sees" vs. "present in an internal dict" invisible by name |
| MEMORY-106 §1.2 | new | refuse | `ingest_test.rs`, `sync_class_property_test.rs`, `structural_exclusion_test.rs` | **deep-read** — names name the topic, not whether the row is asserted **not** ingested (refuse's absent-side-effect arm) |
| CUO-106 §1.12 | new | halt | (no named negative-arm test; §5 shows dispatch-routing tests) | **deep-read** — a "MUST NOT bypass HITL halt" negative needs a negative arm; none is cited by name |
| SKILL-113 §1.7 | new | preserve | `marker_validator_test.rs` (only test file listed) | **deep-read** — name does not advertise the whitespace / hash-chain before-vs-after equality the verb demands |
| IMP-072 §1.4 | new | refuse | CI hook + `stamp --check --exit-code` grep-gate (boundary/§1 spec, no named test) | **deep-read** — "refuse a staged change" discharged by a printed fix message is the refuse-by-log-line trap; can't tell from the one-liner |

**Sample split: ~4 quick-clear : ~6 deep-read (≈40% / 60%).** The pattern behind it is the real signal:
tests **quick-clear when the name is descriptive and self-evidently encodes the verb + observable**
(`hot_reload_invalid_preserves_cache`, `create_tenant_rejects_reserved_root_slug`); they **need deep
read when the name is generic** (`test_supervisor_graph`, `marker_validator_test`, `ingest_test`) **or
the clause is a `MUST NOT` negative** whose negative arm a name almost never reveals. Note there are
**zero "obvious-mismatch"** in the sample — a mismatch is invisible by name (that is precisely why the
108 anchor slipped both human gates), so it can only ever surface as a deep-read, never a quick-clear.

---

## 5. EFFORT SHAPE

- **Full sweep (all clauses):** 166 specs × ~13 §1 clauses ≈ **~2,200 clause-reads**; the test-citing
  subset (most of them) each needs one verb-vs-assertion comparison.
- **Danger zone (high-risk verbs):** **624 clauses** across the 166 specs (~3.8 per spec). `emit`
  carries 410 of these; the **acute subset** refuse+reject+render+halt+preserve = **275 clauses**.
- **Quick-clear vs deep-read:** from the sample, ~40% quick-clear by test name, ~60% need the body
  opened. Applied to the 624 danger-zone clauses → ~**250 quick-clear / ~370 deep-read**; applied to the
  275 acute clauses → ~**110 quick-clear / ~165 deep-read**. Deep-read (open a test, compare meaning; a
  few minutes each) is where the cost sits — call it **~370 deep-reads for the whole danger zone**,
  **~165 for the acute-verb-only pass.**

**One line for the operator:** the sweep is **~166 specs / ~2,200 clauses**, the risk concentrates in
**~624 high-risk-verb clauses (275 acute + ~410 emit)**, and roughly **~60% of those need a test opened
to judge**. It is a low-thousands-of-comparisons job, not a heroic one — and it partitions cleanly by
verb, so it does not have to be run all at once.

---

## Recommended sweep approach (a RECOMMENDATION for the operator, not a decision)

1. **High-risk-verbs-first, acute-verbs-first-of-those.** Sweep the 275 acute clauses (order
   **refuse → reject → render → halt → preserve**) BEFORE the 410 `emit` clauses. The acute verbs are
   where a weaker test is both common AND high-blast-radius (a security refusal discharged by a log
   line; a render discharged by present-in-payload — the 108 case). `emit` is high-volume but mostly
   OTel/audit observability — the trap is real, the blast radius lower — so sweep it second, in bulk.
2. **Batch by module, ~15–20 specs per batch (≈ one module).** That is **~9–11 batches** for the full
   166, or **~4–5 batches** for an acute-verb-only pass. Largest relevant buckets to plan around:
   improvement 35 · ai 22 · memory 21 · skill 20 · proj 18 · auth 17 · cuo 14.
3. **Calibrate every batch against `IMP-108 §1.7`** (known positive: its pre-fix `grep`-in-payload test
   FAILS TRACE-006, its post-fix visible-markup test PASSES). Run it first in each batch so the
   auditor's "weaker-than" threshold is anchored to the motivating case.
4. **Spend the deep-read budget where names hide the assertion:** quick-clear descriptive-named tests
   by name; reserve reading for **generic-named tests** (`test_*_graph`, `*_validator_test`,
   `ingest_test`) and **every `MUST NOT` negative clause** (names rarely reveal a negative arm).
5. **Defer the ~20 CI-gate/fixture/manual-verified boundary specs to a second, separate pass.** They
   often have no test file to open (their "test" is a CI grep, frequently itself the weaker assertion),
   so they are lower-yield and need a different lens than "read the cited test."

---

## Confidence, and what I could NOT measure cleanly

**High confidence** on the headline counts — **187 done** (two methods, zero gap), **166 in scope**,
**~2,210 §1 clauses**, verb tally **emit 410 / reject 102 / render 63 / refuse 60 / preserve 41 /
halt 9 (union 624)**. All re-derived at HEAD `a644019e`; the done count reconciles across the BACKLOG
index and frontmatter exactly.

**Could not measure cleanly — stated so the operator does not over-trust a number:**

1. **The population boundary is genuinely fuzzy (166 core, ~186 max).** "Cites a test" is unambiguous
   for 166 (named/path/file tests). The other 20 §1-carrying specs verify via CI-gate greps /
   acceptance case-tables / operator-observed runs — TRACE-006-examinable in principle, but usually
   with no test file to read. 166 is the defensible sweep target; ~186 is the maximalist ceiling.
2. **The per-clause clause↔test binding is not machined for the 120 older specs.** Only 46 specs carry
   `traces_to` links; the rest bind §1 clauses to §5 tests by AC position and prose. So "clauses that
   cite a test" is bounded (2,210 §1 clauses; 672 named tests; 405 explicit links), **not an exact
   clause→test map** — building that map IS reading the specs, i.e. part of the sweep.
3. **The verb tally counts stem-in-clause, a superset of operative-verb.** A clause that merely mentions
   "emit" or "reject" in passing is counted, so **624 is an upper bound** on true danger-zone clauses;
   the operative-verb count is somewhat lower (most visibly for `emit`).
4. **The §4/§10 triage is smells from test names only** — by design, because names rarely reveal
   assertion strength (the whole reason TRACE-006 exists). The ~40/60 quick-clear/deep-read split is an
   estimate from 10 clauses, not a projection anyone should bank to two significant figures.
5. **Automated §1 parsing across heterogeneous grammars is ~99%, not 100%.** Two specs needed
   heading-specific handling (`CUO-204` uses "Section 1", `CUO-301` is a bug with no §1), and the newer
   `- 1.N` bullet clause style initially undercounted verbs — including the 108 anchor's own render
   clause — until the splitter was broadened and re-run. Residual undercount in the newer numbered specs
   is small but non-zero.

---

## Method note (Node pass, not committed)

The grep-able numbers (§1 population, done reconciliation) use the commands shown inline. The
clause/verb/citation tallies use one ad-hoc Node script run over the 187 `done` `spec.md` files (not
committed — this finding ships exactly one file). Its logic, for reproduction: extract the §1 block
(heading `^#{2,4}\s*(§?1|Section 1)` with a description/normative/clause/requirement cue, up to the next
`§2..§9` / `Acceptance` / `Why` / `Out of scope` heading); split into clauses on
`^\s*(?:[-*]\s+)?(?:\*\*)?\d+(?:\.\d+)*[.)]?\s+`; a clause is BCP-14 if it matches
`MUST NOT|MUST|SHALL|SHOULD NOT|SHOULD|MAY`; verb stems
`render(s|ed|ing)?` · `reject(s|ed|ion)?` · `refus(e|es|ed|al)` · `halt(s|ed|ing)?` · `emit(s|ted|ting)?`
· `preserv(e|es|ed|ation)`; "cites a test" = any of `#[test]`/`#[tokio::test]`, `^…Test:`, `traces_to`,
`path.{rs,sh,py,ts,tsx,mjs}::fn`, or a `tests/…` test-file path.

---

Filed as a sizing measurement only. **No task status flipped, no `.audit.md` written, no re-audit
performed, no verdict rendered, no spec frontmatter or body touched, `task-lint` and the RUBRIC
unchanged.** The next actor owns the decision to run the sweep, and how.

### Attachment — the 166 TRACE-006-relevant `done` task IDs

```
ai: AI-001 AI-002 AI-003 AI-004 AI-005 AI-006 AI-007 AI-008 AI-009 AI-010 AI-011 AI-012 AI-013 AI-014
    AI-015 AI-016 AI-017 AI-018 AI-019 AI-020 AI-021 AI-022
auth: AUTH-001 AUTH-002 AUTH-003 AUTH-004 AUTH-005 AUTH-006 AUTH-101 AUTH-102 AUTH-103 AUTH-104 AUTH-105
      AUTH-106 AUTH-107 AUTH-108 AUTH-109 AUTH-110 AUTH-111
chat: CHAT-267 CHAT-268 CHAT-269
cuo: CUO-101 CUO-102 CUO-103 CUO-104 CUO-105 CUO-106 CUO-200 CUO-201 CUO-202 CUO-203 CUO-204 CUO-206
     CUO-207 CUO-209
docs: DOCS-003 DOCS-004 DOCS-005 DOCS-006 DOCS-007
email: EMAIL-001 EMAIL-004 EMAIL-005 EMAIL-009 EMAIL-011
improvement: IMP-068 IMP-069 IMP-070 IMP-082 IMP-083 IMP-084 IMP-085 IMP-086 IMP-087 IMP-088 IMP-089
             IMP-090 IMP-091 IMP-092 IMP-093 IMP-094 IMP-095 IMP-096 IMP-097 IMP-098 IMP-099 IMP-100
             IMP-101 IMP-102 IMP-103 IMP-104 IMP-106 IMP-107 IMP-108 IMP-109 IMP-110 IMP-114 IMP-115
             IMP-116 IMP-118
mcp: MCP-001 MCP-002 MCP-004
memory: MEMORY-101 MEMORY-102 MEMORY-103 MEMORY-104 MEMORY-105 MEMORY-106 MEMORY-107 MEMORY-108
        MEMORY-109 MEMORY-110 MEMORY-111 MEMORY-112 MEMORY-113 MEMORY-114 MEMORY-115 MEMORY-116
        MEMORY-117 MEMORY-118 MEMORY-119 MEMORY-120 MEMORY-121
obs: OBS-002 OBS-006
proj: PROJ-001 PROJ-002 PROJ-003 PROJ-004 PROJ-005 PROJ-006 PROJ-007 PROJ-008 PROJ-009 PROJ-010 PROJ-011
      PROJ-012 PROJ-013 PROJ-014 PROJ-015 PROJ-016 PROJ-017 PROJ-018
skill: SKILL-101 SKILL-102 SKILL-103 SKILL-104 SKILL-105 SKILL-106 SKILL-107 SKILL-108 SKILL-109
       SKILL-110 SKILL-111 SKILL-112 SKILL-113 SKILL-114 SKILL-115 SKILL-116 SKILL-118 SKILL-119
       SKILL-120 SKILL-201
templates: TPL-001
```

Boundary specs excluded from the 166 (examinable at most in a second pass): CUO-301 (bug, no §1
clauses); APP-003/004/005/006, CHAT-101, CUO-205/208, DOCS-002, IMP-071/072/073/074/075/076/077/078/
079/080/081, SKILL-117 (§1 clauses verified via CI-gate greps / acceptance fixtures / manual runs, no
named test to read).
