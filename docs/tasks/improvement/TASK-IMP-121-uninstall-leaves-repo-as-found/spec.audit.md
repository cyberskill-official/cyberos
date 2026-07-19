---
audit_template_version:     "task_rubric@1.0"
audited_file:               "./spec.md"
audited_file_sha256:        "77ed93f82fcaee79ce21637c3528cbbd0088bcbb34d4e4c92ad98640b678e2a3"
audited_file_sha256_prefix: "77ed93f82fcaee79"
audited_body_sha256:        "1b23f18afc2aa7c9b474f6f3c06cd18d5c116fa8744f1df73c44afb9d6aa22a6"
audited_body_sha256_prefix: "1b23f18afc2aa7c9"
body_hash_method:           "sha256 of spec.md with frontmatter lines matching ^(status|shipped|routed_back_count|memory_chain_hash): removed (3 present: status, routed_back_count, memory_chain_hash; shipped absent) — TASK-IMP-102 §12"
rubric_version:             "audit_rubric@2.0"
skill_id:                   "task-audit"
skill_version:              "1.0.0"
prompt_revision:            "task_audit@2.0.0"
template_detected:          "task@1"
last_audit_at:              "2026-07-19T00:00:00Z"
overall_status:             "needs_human"
iterations:                 2
issue_counts:
  total:                    3
  open:                     1
  needs_human:              0
  fixed:                    1
  wontfix:                  1
trace_id:                   "independent-reaudit-2026-07-19-imp121"
caller_persona:             "independent-auditor"
audit_kind:                 "independent RE-AUDIT (auditor did NOT author the spec; auditor did NOT author the prior audit)"
head_commit:                "b2221ffa (worktree; install.sh/uninstall.sh/test_install_hygiene.sh all clean == HEAD; spec.md uncommitted/modified, spec.audit.md overwritten uncommitted)"
supersedes_audit:           "prior audit at audited_file_sha256 a03d4b5302f8af73 (needs_human 8/10, blocker ISS-001); spec revised since — re-evaluated fresh per §2 re-entrancy (hash changed → prior open/needs_human issues reset)"
---

# Independent RE-AUDIT — TASK-IMP-121 (uninstall must leave the repo as it found it)

> Gate: `draft → ready_to_implement`. Rubric: `audit_rubric@2.0` (FM / SEC / COND / QA /
> SAFE / TRACE families); type `improvement` adds no extra family. Machine floor
> `task-lint.mjs` run first (exit 0), then the judgment families (QA semantics, SAFE,
> TRACE-006) and full citation verification against `install.sh` / `uninstall.sh` at HEAD.
> This is a RE-AUDIT of a spec revised after a PRIOR independent audit returned NEEDS_HUMAN
> 8/10 on one blocker (ISS-001: the native-channel `rmdir` at `uninstall.sh:165` was omitted /
> denied). The revision added clause 1.6 + AC 6 and corrected the false statements. The crux
> (ISS-001 closure) was verified by RE-DERIVING `uninstall.sh:165` at HEAD and REPRODUCING the
> `.claude/skills` destruction end-to-end from a real `build.sh` payload under `/tmp`. The
> auditor authored neither the spec nor the prior audit.

## §1 — Verdict summary

`spec.md` is now a **169-line (168 `\n`), `template: task@1`** improvement spec with **6
normative §1 clauses (1.1–1.6)**, **6 acceptance criteria (AC1–AC6)** in clean 1:1
clause→AC→test correspondence, a 9-item §3 edge-case set (incl. a security-class row and a
new native-parent row), and 7 required sections + the AI-authorship disclosure, all present
and non-empty. Deterministic lint exits **0**.

**The ISS-001 blocker is CLOSED — the crux this re-audit was chartered to verify.** The prior
audit's error-severity finding — that the spec asserted in three places that only
`.agents/skills` and `.agents` could ever be `rmdir`'d, while `uninstall.sh:165` also `rmdir`s
five native channel parents and destroys an operator's pre-existing empty `.claude/skills` —
is fully resolved:
- **Clause 1.6 (new)** requires REMOVING the per-family `rmdir "$root/$_sd"` at
  `uninstall.sh:165` so an operator's pre-existing empty native channel dir survives; I opened
  `uninstall.sh` at HEAD and confirmed `:165` is exactly that per-family loop `rmdir` over
  `.claude/skills` `.grok/skills` `.commandcode/skills` `.codex/skills` `.opencode/skill`.
- **Reproduced end-to-end** (real payload, scratch git repo, `/tmp`): a pre-existing EMPTY
  `.claude/skills`, default install (drops 3 managed symlinks into it), then uninstall →
  **`.claude/skills` is DESTROYED** (managed links removed by the family loop first, then the
  emptied operator dir `rmdir`'d at `:165`). `.agents/skills` destroyed by `:141` in the same
  run; `.agents` survived via the `:597` rules pointer. Clause 1.6 targets a real defect.
- **The three false statements are gone.** The only surviving "only two dirs" phrase (L50)
  explicitly labels it the FALSE earlier claim being retracted; L74/L105/L110/L118 now
  affirmatively state the five native parents ARE `rmdir`'d at `:165` and align the Primary
  metric to include them. The prior audit's secondary FALSE claim g (`grep` omits `:165`) and
  loose claim h (`~78` lines) are both corrected (`:141 :142 :165 :226`; `~52` / 230→282).

**One NEW, lighter finding blocks a clean 10/10 — introduced by the surgical edit.** The
`## Proposed Solution` section (a required SEC-003 heading) still opens "**Three** independent,
testable changes" and enumerates exactly three bold items (marker gate §1.1–1.3, `.gitignore`
§1.4, hook §1.5). It describes **no solution for §1.6** — the `:165` native-parent `rmdir`
removal — anywhere in the section, even though §1.6 is a normative MUST with its own AC 6 and
the effort breakdown two paragraphs later (L106) counts it as a distinct 0.5h code change.
The contract is complete and correct (§1.6 + AC 6 + Scope + Problem + Metrics all express the
fix, so an implementer builds it), so this is a warning-severity internal-consistency /
completeness gap, not a correctness error and not a contract gap — but it keeps the spec off
the 10/10 shipping bar until the section is squared with the six-clause reality.

**Verdict: NEEDS_HUMAN — score 9 / 10.** ISS-001 is CLOSED (big improvement over the prior
8/10 error-blocker); route back for a ~2-line author fix: add a fourth Proposed-Solution item
for §1.6 (remove the `:165` `rmdir`, restoring the pre-126 leave-in-place) and change "Three"
→ "Four" (or reword the lead to fold §1.6 explicitly under the data-loss fix). No operator
HITL decision is required — this is a mechanical author revision.

## §2 — Per-rule-family verdicts

| family | scope | verdict | basis |
| ------ | ----- | ------- | ----- |
| **FM** (FM-001..114) | frontmatter shape + per-field enums | **PASS** | `node tools/install/docs-tools/task-lint.mjs spec.md` → **exit 0**. `template: task@1` ✓, `type: improvement` ✓, `status: draft` ✓, `priority: p1` ✓, `author: "@stephencheng"` ✓ (FM-102), `department: engineering` ✓, `created_at` ISO-8601+tz ✓, `ai_authorship: generated_then_reviewed` ✓, `eu_ai_act_risk_class: not_ai` ✓ (shell-installer plumbing), `client_visible: false` ✓, `severity` correctly absent (FM-114 forbids for non-bug) ✓, no `# UNREVIEWED` (FM-112) ✓. |
| **SEC** (SEC-001..009) | required H2s present + non-empty | **PASS** | Summary, Problem, Proposed Solution, Alternatives Considered, Success Metrics, Scope, Dependencies — all present, all non-empty (SEC-008 ✓); hierarchy well-formed (SEC-009). NB: SEC only checks presence/non-emptiness — the Proposed-Solution *completeness* gap is a QA/consistency finding (ISS-002), not a SEC failure. |
| **COND** (COND-001..004) | conditional sections keyed on FM | **PASS** | `ai_authorship != none` → COND-004 requires `## AI Authorship Disclosure` with `Tools used:` / `Scope:` / `Human review:` — all three present (L135-137). `client_visible: false` → COND-001/002 n/a. `not_ai` → COND-003 n/a. |
| **COND (clause quality — task framing)** | every §1 clause a testable MUST | **PASS** | 1.1 (MUST write parent marker iff install creates the dir + MUST NOT mark a pre-existing dir), 1.2 (MUST gate `rmdir` on marker AND empty; emptiness alone MUST NOT authorise; unmarked MUST be kept+reported), 1.3 (MUST remove marker before `rmdir`, whether or not it succeeds), 1.4 (MUST delete the `\n` separator byte; byte-identical N≥1 cycles), 1.5 (MUST restore a no-trailing-newline hook byte-identical; keep the newline case exact), 1.6 (MUST NOT `rmdir` the five native parents; the `:165` `rmdir` MUST be removed) — all observable, testable MUSTs. |
| **QA** (QA-001..009, QA-TODO) | prose quality / anti-patterns | **PASS w/ 1 finding** | Alternatives: 7 distinct with rejection rationale (QA-005 ✓), incl. the corrected "mark all seven `rmdir`'d dirs" row (2+5=7); Scope has `### Out of scope / Non-Goals` with 6 bullets (QA-006 ✓); Dependencies name TASK-IMP-106 (`done`) (QA-008 ✓); no `TODO:` stubs. Primary metric now names a baseline + reproduced observation and aligns to the delivered scope incl. native dirs (QA-004 no longer hit — the prior audit's QA-004 target-not-achievable is resolved by §1.6). **Finding (ISS-002):** the `## Proposed Solution` section is internally inconsistent — "Three independent, testable changes" + no §1.6 item vs the six-clause / four-code-change reality the effort breakdown states. |
| **SAFE** (SAFE-001..004) | untrusted-content discipline + operator-file safety | **PASS / ADEQUATE** | No `<untrusted_content>` blocks; none nested/unclosed; no injection markers; no auditor-directed commands (SAFE-001..004 ✓). §3 security-class row (L168) is adequate for a task that mutates the destructive uninstall path AND has install write ownership markers: ownership decided by the marker FILE's content (not a path pattern / dir NAME an operator could forge), no operator string interpolated into `rmdir`/`rm` targets (fixed literals under `$root`), in-repo confinement, nothing read from disk executed, and §1.1 never marks a pre-existing dir. §1.6 REMOVES a destructive op (strictly safer; no new attack surface), so the posture holds and — unlike the prior audit — the completeness of that posture is no longer overclaimed (the `:165` twin is now addressed by §1.6). |
| **TRACE-001..003** (structural) | §1→AC→§5 traceability | **PASS** | 1:1: 1.1→AC1, 1.2→AC2, 1.3→AC3, 1.4→AC4, 1.5→AC5, 1.6→AC6 (TRACE-001 ✓, every clause a BCP-14 MUST cited by `traces_to:`). Each AC names a concrete arm `test_install_hygiene.sh::t23..t28` (TRACE-002 ✓) — all follow the file's `t<NN>_<slug>` convention, unique numbers next after the existing `t22` (highest on disk), no collision; the newline-control arm `t_hook_strip_byte_identical_across_cycles` referenced by AC 5 exists at `:651`. Test file resolves on disk (729 B lines) and is in `modified_files`; t23–t28 unwritten — correct for a draft (TRACE-003 ✓). |
| **TRACE-006** (judgment) | cited test exercises clause's VERB | **PASS** | Per-clause verb→assertion comparison in §4; all six ACs describe assertions that discharge their clause's verb, incl. the new 1.6 (**preserve** — the pre-existing native dir still EXISTS after uninstall) whose baseline (dir destroyed today) I reproduced, so AC 6 is non-vacuous. |
| **XCHAIN / STALE** | cross-skill / staleness | **N/A** | Standalone independent audit; no author manifest and no `provenance.source_hash` supplied to diff. |

## §3 — Citation verification (independent — ISS-001 closure is the crux)

Worktree clean except `spec.md` (modified) and `spec.audit.md` (untracked); the `.fuse_hidden*`
entries are FUSE-mount artefacts, not repo changes. `install.sh` 1055 lines, `uninstall.sh`
282, `test_install_hygiene.sh` 729 — all == HEAD `b2221ffa`. Payload built with
`bash tools/install/build.sh /tmp/pay121reaudit`. All scratch installs under `/tmp`; no real
install touched.

### ISS-001 closure — the native-parent `:165` data-loss (crux)

**(a) Clause 1.6 targets the real `:165` `rmdir`. → CONFIRMED.** Opened `uninstall.sh` at HEAD:
`:165 rmdir "$root/$_sd" 2>/dev/null || true` sits inside the per-family loop
(`for _fs in ".claude/skills:ship-tasks" … ".opencode/skill:ship-tasks"`, `:151-166`), where
`_sd="${_fs%%:*}"` iterates the five distinct native parents `.claude/skills` `.grok/skills`
`.commandcode/skills` `.codex/skills` `.opencode/skill`. Clause 1.6 (L146) requires REMOVING
exactly this line ("the per-family loop's `rmdir "$root/$_sd"` at `:165` … MUST be removed").
`grep -n rmdir uninstall.sh` → **`:141 :142 :165 :226`** (+ a `:174` *comment*, prose), matching
the spec's corrected enumeration (L50/L74). `:226` is the BRAIN-restore tmpdir, correctly
flagged unrelated.

**(b) Reproduced baseline AC 6 asserts against. → CONFIRMED, destruction observed.** Scratch
git repo with operator's PRE-EXISTING EMPTY `.claude/skills` (and `.agents/skills`), default
install, then uninstall:
- After install: `.claude/skills` holds 3 managed symlinks (`ship-tasks`, `task-audit`,
  `task-author` → `../../.cyberos/plugin/skills/<cmd>`); NO `.cyberos-owned` written anywhere
  (default symlink install).
- After uninstall: uninstall log shows `removed .claude/skills/ship-tasks (managed skill link)`
  (+ the other two) — the family loop removes the managed links — then **`.claude/skills` is
  GONE** (`[ -d .claude/skills ]` → false). `.claude` (the parent) survives empty (nothing
  `rmdir`s `.claude` itself). `.agents/skills` GONE too (destroyed by `:141`); `.agents`
  survived only because `.agents/rules/cyberos.md` (the `:597` pointer) keeps it non-empty.
- This is the exact emptiness-only mechanism the lead `.agents/skills` bug uses, and it is the
  baseline the spec's Primary metric (L110) and AC 6 (L155) both assert against — verified real.

**(c) No false "only two dirs" / "cannot reach" statement survives. → CONFIRMED.** Full-text
scan: the sole "only two dirs" occurrence (L50 `source_decisions`) reads "…the earlier claim
that only two dirs are ever `rmdir`'d was FALSE (reproduced…)" — a retraction, not an assertion.
L74 (Problem), L105 (Alternatives), L110 (Metrics), L118 (Scope-in), L125 (Scope-out) all now
state the five native parents ARE `rmdir`'d / are removed by §1.6. No "sit under dirs the prune
cannot reach" and no "returns only `:141 :142 :226`" phrasing remains anywhere.

**(d) AC 6 traces to §1.6, names a real-shaped arm, guards IMP-126. → CONFIRMED.** AC 6
(`traces_to: #1.6`) asserts a pre-existing empty native dir "still EXISTS after uninstall",
MUST-FAILs on today's `:165` `rmdir` (the reproduced baseline), AND "MUST confirm the managed
skill link inside it is still removed (this task must not regress IMP-126's link cleanup)" —
the reproduction confirms that link removal is a live behavior (`removed …/ship-tasks (managed
skill link)`), so the regression guard is meaningful. Arm `t28_native_channel_parent_survives`
is convention-correct and non-colliding.

### Still-load-bearing prior claims — spot-re-verified, citations intact

| # | claim | at HEAD | verdict |
| - | ----- | ------- | ------- |
| a | `uninstall.sh:141-142` `rmdir`s `.agents/skills` then `.agents` on emptiness, no ownership test | `:141`/`:142` confirmed; `.agents/skills` destruction re-reproduced this run | CONFIRMED (cited §1.2/AC2) |
| b | marker written one dir too low — `install.sh:684` into `$_sdest` (`:662` = `.agents/skills/$_sc`, child) | `:662`/`:681-684` confirmed; default symlink install writes NO marker (reproduced) | CONFIRMED (cited §1.1) |
| c | hook awk `uninstall.sh:87-93` NOT byte-exact for a no-trailing-newline hook (33→34) | `:87-93` awk confirmed; install append heredocs `:860-862` / `:922-924` (first line blank) confirmed; prior audit reproduced 33→34 to the byte | CONFIRMED, citations unchanged (cited §1.5/AC5) |
| d | `.gitignore` strip `uninstall.sh:106` leaves `install.sh:756`'s separator byte (20→21) | `:106` sed, `:756` separator, `:715` seed, `:742-754` trim all confirmed; prior audit reproduced 20→21 | CONFIRMED, citations unchanged (cited §1.4/AC4) |
| e | `t22` dropped honestly (self-relative; needs no re-point) | `t22_uninstall_behavior_unchanged` at `test_install_hygiene.sh:545` exists; summary anchor `:230`; edits before it | CONFIRMED (Scope L126, Deps L131) |
| f | `mcp_json()` 105 bytes; MCP out of scope | `install.sh:694` / `uninstall.sh:176` confirmed | consistent (recorded only to bound the split) |

## §4 — TRACE-006 per-clause records (verb demanded vs. assertion described)

Arms t23–t28 are unwritten (draft gate), so each clause verb is compared against the assertion
its AC *describes*.

- **1.1 — verbs `write` (emit) + `MUST NOT mark` (guard).** Demands: the two parent markers
  exist with dir-naming text + the adoption sentence WHEN install creates the dir; NO marker on
  a pre-existing dir. **AC1** asserts both branches (pre-existing → NEITHER marked; neither dir
  → BOTH marked with correct text) across the `:597`-alone and `:669`-chain creation paths and
  MUST-FAILs if any pre-existing dir is marked. Discharges both verbs. ✓
- **1.2 — verb `remove … only when marked AND empty` (guarded delete).** Demands: emptiness
  alone MUST NOT delete; an unmarked dir kept+reported. **AC2** asserts pre-existing-empty →
  both EXIST + kept; neither → both REMOVED; MUST-FAIL if emptiness alone removes an unmarked
  dir (the reproduced baseline). Discharges the guard. ✓
- **1.3 — verb `remove` marker before `rmdir`, unconditionally (emit-absence) + `kept`.**
  Demands: no `.cyberos-owned` survives anywhere incl. a KEPT `.agents`; an operator-adopted
  (marker-deleted) dir survives + reported. **AC3** asserts exactly these and MUST-FAILs on a
  surviving marker or a removed adopted dir. Discharges. ✓
- **1.4 — verb `preserve` (byte-identical N≥1 cycles).** Demands: pre-existing
  newline-terminated `.gitignore` byte-identical after 1 and 3 cycles. **AC4** asserts `cmp`
  byte-identity at 1 and 3 cycles, MUST-FAILs on today's 20→21 leak; install-created stays
  clean. Before/after equality → discharges `preserve`. ✓
- **1.5 — verb `preserve` (byte-identical incl. no trailing newline).** Demands:
  no-trailing-newline foreign hook byte-identical after uninstall + across cycles, AND the
  newline control stays exact. **AC5** asserts both via `cmp`, MUST-FAILs on the 33→34 leak.
  Discharges `preserve`; §3(c) confirms non-vacuous. ✓
- **1.6 — verbs `MUST NOT rmdir` (preserve the dir) + `MUST be removed` (delete the `:165`
  line).** Demands: a pre-existing empty native channel dir still EXISTS after uninstall.
  **AC6** asserts the dir "still EXISTS after uninstall", MUST-FAILs on today's `:165` `rmdir`
  (reproduced destroying it), and additionally asserts the managed link inside is still
  removed. Before/after existence → discharges `preserve`; the reproduction makes the assertion
  non-vacuous today. ✓

**Security-class edge case (SAFE / adequacy).** §3's security row (L168) names the threat
(name/path forgery inducing a deletion), the invariant (ownership by marker-file content), the
no-exec property, and the fixed-literal `rmdir`/`rm` targets under `$root`. Unlike the prior
audit — where the same posture was asserted to cover a prune that `:165` left ungated — the
`:165` instance is now closed by §1.6 (removal, not gating), so the completeness claim is no
longer overclaimed. Adequate.

## §5 — Findings (itemised)

```
ISSUE
id:              ISS-001
rule_id:         QA-004 / citation-verification crux (task-audit SKILL §3) / SAFE completeness — the PRIOR audit's error-severity blocker
status:          fixed
severity:        error
category:        scope_boundary
location:        clause §1.6 (L146); AC 6 (L155); frontmatter L50; Problem L74; Alternatives L105; Metrics L110; Scope L118/L125 — vs uninstall.sh:165
evidence:        "PRIOR: 'the only two dirs uninstall can rmdir (:141-142)' / native channels 'sit under dirs the prune cannot reach' — while uninstall.sh:165 rmdirs each native parent and destroys a pre-existing empty .claude/skills. NOW: clause 1.6 requires removing the :165 rmdir; L50 retracts the false claim; L74/L105/L110/L118 affirm the five native parents ARE rmdir'd."
description:     "CLOSED by the revision and CONFIRMED this re-audit. (1) Clause 1.6 (new,
                 normative MUST) requires REMOVING the per-family `rmdir \"$root/$_sd\"` at
                 uninstall.sh:165 — verified :165 is that loop rmdir over the five native
                 parents. (2) Reproduced end-to-end: pre-existing empty .claude/skills, default
                 install, uninstall → .claude/skills DESTROYED at :165 (managed links removed
                 first), .agents/skills destroyed at :141 — the exact baseline AC 6 and the
                 Primary metric assert against. (3) No false 'only two dirs' / 'cannot reach' /
                 'returns only :141 :142 :226' statement survives; the lone 'only two dirs'
                 phrase (L50) is an explicit retraction. (4) AC 6 traces_to #1.6, names a
                 convention-correct arm (t28_native_channel_parent_survives), and guards the
                 IMP-126 managed-link removal (reproduced live). The prior audit's secondary
                 FALSE claim g (grep omits :165) and loose claim h (~78 lines) are also
                 corrected (:141 :142 :165 :226; ~52 / 230→282), as are the prior INFO items
                 (metric wording 'per cycle content-shape' → 'one-time +1, stays 34')."
auto_fix_applied: false
resolution:      "Resolved by the author's revision (clause 1.6 + AC 6 + three-site correction). Verified closed; reproduced the targeted defect."
opened_at:       "2026-07-19T00:00:00Z"
resolved_at:     "2026-07-19T00:00:00Z"

ISSUE
id:              ISS-002
rule_id:         QA (internal consistency / completeness of a required section; master-rule §0 "complete"); no error→needs_human lint rule — author-fixable revision
status:          open
severity:        warning
category:        n/a (author revision, not an operator HITL decision)
location:        Proposed Solution L82 (lead sentence) + L84-95 (the three bold items) — vs clause §1.6 (L146), AC 6 (L155), and the effort breakdown L106
evidence:        "'Three independent, testable changes, each grounded in a re-verified HEAD citation.' (L82) — the section's three bold items cover only the .agents marker gate (§1.1-1.3), .gitignore (§1.4), and hook (§1.5). No item describes §1.6 (remove the :165 native-parent rmdir). The data-loss bold item (L84) describes ONLY the .agents pair and ends 'TASK-IMP-094's mechanism applied one directory up'. Meanwhile L106 lists 'delete the native-parent rmdir at :165 (§1.6) 0.5h' as a distinct change."
description:     "Surgical-edit consistency gap. The revision added clause 1.6 + AC 6 and
                 updated Problem, Scope, Metrics, Alternatives and §3 — but the ## Proposed
                 Solution section (required SEC-003 heading) was not squared with §1.6: its
                 lead count says 'Three' while the delivered scope is FOUR code changes across
                 SIX clauses, and it describes no solution for §1.6 at all. A reader of Proposed
                 Solution alone would not learn the native-parent rmdir removal is part of the
                 work. This is NOT a correctness error and NOT a contract gap — §1.6 + AC 6 +
                 Scope + Problem + Metrics fully and correctly express the fix, so an
                 implementer builds it; the marker-decision table (L86-91) legitimately covers
                 only the .agents pair because markers apply only there. It is a warning-level
                 internal-consistency / completeness defect that keeps the spec off the 10/10
                 shipping bar (master rule §0: 'complete' AND 'perfectly matched to core
                 requirements'). Cheap to fix; no operator decision required."
suggestion:      "Add a fourth bold Proposed-Solution item for §1.6 — e.g. 'Stop pruning the
                 native channel parents. Remove the per-family rmdir \"$root/$_sd\" at
                 uninstall.sh:165 so an operator's pre-existing empty .claude/skills (etc.)
                 survives, restoring the pre-126 leave-in-place; the managed link inside is
                 still removed by the family loop.' — and change the lead count 'Three' → 'Four'
                 (or reword to fold §1.6 explicitly under the data-loss fix). ~2 lines."
auto_fix_applied: false
resolution:      null
opened_at:       "2026-07-19T00:00:00Z"

ISSUE
id:              ISS-003
rule_id:         QA-004-adjacent (enumeration precision; no error-severity rule hit)
status:          wontfix
severity:        info
location:        frontmatter L50; Problem L74
evidence:        "'`grep -n rmdir uninstall.sh` returns :141, :142, :165 and :226' — actual grep also matches :174 (a comment: 'Never rmdir .cursor/ …')."
description:     "The enumeration lists the four actual `rmdir` COMMANDS (:141 :142 :165 :226)
                 and omits the :174 line, which is a comment mentioning the word 'rmdir', not an
                 rmdir invocation. Enumerating the commands is the correct intent and is
                 accurate; the omission of the prose match is harmless. Noted for completeness;
                 the prior audit treated the same detail parenthetically and non-blocking."
suggestion:      "Optional: say 'the four rmdir statements are :141 :142 :165 :226' or add '(+ a :174 comment)' to be exact."
auto_fix_applied: false
resolution:      "Left to author discretion; non-blocking."
opened_at:       "2026-07-19T00:00:00Z"
```

No findings were manufactured to pad the count (per the skill's MUST-NOT "invent rule
violations"): ISS-001's closure was reproduced, ISS-002 is a factual internal contradiction
(section count vs the effort breakdown in the same file), and ISS-003 is a genuine minor
enumeration note. Equally, ISS-002 was not suppressed to hand back a clean pass — a required
section that omits a normative clause's solution and carries a self-contradicting count is a
real completeness gap.

```
SUMMARY
verdict:         needs_human
score:           9 / 10
issues_total:    3
issues_open:     1
issues_human:    0
issues_fixed:    1
issues_wontfix:  1
iterations:      2
machine_floor:   "task-lint.mjs → exit 0"
iss_001_status:  "CLOSED — clause 1.6 removes the uninstall.sh:165 rmdir; reproduced .claude/skills DESTROYED end-to-end at HEAD; three false 'only two dirs' statements corrected; AC 6 traces to §1.6, names t28, guards IMP-126 link cleanup"
citations:       "1.6→:165 CONFIRMED + reproduced; a–f (prior load-bearing claims) still hold with citations intact; secondary g/h corrected by the revision"
blocker:         "ISS-002 (warning) — Proposed Solution says 'Three independent, testable changes' and omits §1.6; inconsistent with AC 6 + the L106 effort breakdown; keeps it below the 10/10 bar"
next_action:     "re-author (add the §1.6 Proposed-Solution item + fix the count) → re-audit; no operator HITL decision required"
---
```

---

*Independent RE-AUDIT of TASK-IMP-121 spec.md — audit_rubric@2.0 — 2026-07-19. Auditor
authored neither the spec nor the prior audit. Only this file (spec.audit.md) was written; no
`.git` path was touched; all scratch builds/installs were under `/tmp`.*
