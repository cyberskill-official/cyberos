---
audit_template_version:    "task_rubric@1.0"
audited_file:              "./spec.md"
audited_file_sha256:       "355c6466f489945a2a023c58ff1261a10a576d090a503367ca8a6c9c4e097349"
audited_file_sha256_prefix: "355c6466f489945a"
audited_body_sha256:       "bffa0845c57525951642e14bcfd03d4798e899aae8976a8ae0cc55e2942b7d18"
audited_body_sha256_prefix: "bffa0845c5752595"
body_hash_method:          "sha256 of spec.md with frontmatter lines matching ^(status|shipped|routed_back_count|memory_chain_hash): removed (3 present: status, routed_back_count, memory_chain_hash; shipped absent) — TASK-IMP-102 §12"
rubric_version:            "audit_rubric@2.0"
skill_id:                  "task-audit"
skill_version:             "1.0.0"
prompt_revision:           "task_audit@2.0.0"
template_detected:         "task@1"
last_audit_at:             "2026-07-19T00:00:00Z"
overall_status:            "pass"
iterations:                1
issue_counts:
  total:                   2
  open:                    0
  needs_human:             0
  fixed:                   0
  wontfix:                 2
trace_id:                  "independent-audit-2026-07-19"
caller_persona:            "independent-auditor"
audit_kind:                "independent (auditor did not author the spec)"
---

# Independent spec-correctness audit — TASK-IMP-126 (uninstall completeness)

> Gate: `draft → ready_to_implement`. Rubric: `audit_rubric@2.0` (FM / SEC / COND / QA /
> SAFE / TRACE families), composed as `contracts/task/rubrics/common.md` (type
> `improvement` adds no extra family). Machine floor `task-lint.mjs` run first, then the
> judgment families (QA semantics, SAFE, TRACE-006) and full citation verification by the
> model auditor. The auditor did NOT author this spec.

## §1 — Verdict summary

`spec.md` is a **184-line, `template: task@1`** improvement spec with **4 normative §1 clauses (1.1–1.4)**, **4 acceptance criteria (AC1–AC4)** in clean 1:1 clause→AC→test correspondence, a documented §3 edge-case set (5 items incl. a security-class row), and 7 required sections all present and non-empty. The deterministic lint exits **0** (zero error-severity findings). **All five factual citations about `install.sh` / `uninstall.sh` were opened at the cited lines and independently confirmed TRUE** — including that the hook-strip newline-leak is a real, *current* bug (not already healed). No citation error, no missing trace, no weak family, no security/safety gap. Two INFO-level clarity observations recorded (both non-blocking, neither an error-severity rule hit).

**Verdict: PASS — score 10 / 10.**

## §2 — Per-rule-family verdicts

| family | scope | verdict | basis |
| ------ | ----- | ------- | ----- |
| **FM** (FM-001..116) | frontmatter shape + per-field enums | **PASS** | `node task-lint.mjs spec.md` → exit 0. `template: task@1` ✓, `type: improvement` ✓, `status: draft` ✓, `priority: p2` ✓, `author: @stephencheng` ✓, `department: engineering` ✓, `created_at` ISO-8601+tz ✓, `ai_authorship: generated_then_reviewed` ✓, `eu_ai_act_risk_class: not_ai` ✓ (correct — shell-installer plumbing, no AI, no Annex III / Art. 5 domain), `client_visible: false` ✓. No `# UNREVIEWED` marker (FM-112) ✓. `severity` correctly absent (FM-114 forbids it for non-bug) ✓. |
| **SEC** (SEC-001..009) | required H2 sections present + non-empty | **PASS** | Summary, Problem, Proposed Solution, Alternatives Considered, Success Metrics, Scope, Dependencies — all present, all substantive; lint-confirmed. |
| **COND** (COND-001..004) | conditional sections keyed on FM | **PASS** | `ai_authorship != none` → COND-004 requires `## AI Authorship Disclosure` with `Tools used:` / `Scope:` / `Human review:` bullets — all three present (lines 135–143). `client_visible: false` → COND-001/002 not triggered. `not_ai` → COND-003 not triggered. |
| **COND (clause quality — task framing)** | every §1 clause a testable MUST | **PASS** | 1.1 (MUST remove + MUST NOT remove operator's), 1.2 (MUST remove every managed entry → zero dangling; MUST leave unmarked dir), 1.3 (MUST be exact inverse → byte-identical across cycles), 1.4 (operator artefacts MUST survive) — all four are observable, testable MUSTs. |
| **QA** (QA-001..009, QA-TODO) | prose quality / anti-patterns | **PASS** | Success Metrics carry baselines ("Baseline: all three survive today") + suite-assertion (no QA-004 vanity metric); 3 distinct Alternatives with rejection rationale (QA-005 ✓); Scope has `### Out of scope / Non-Goals` with multiple bullets (QA-006 ✓); risk class not dodged (QA-001/002/003 n/a); Dependencies "None blocking" with owned cross-refs to landed TASK-IMP-083/094 (QA-008 ✓); no `TODO:` stubs. |
| **SAFE** (SAFE-001..004) | untrusted-content discipline | **PASS** | No `<untrusted_content>` blocks (spec quotes no external content); no nested/unclosed blocks; no injection markers; no auditor-directed second-person commands. Security-class §3 row present + adequate (see §4). |
| **TRACE-001..003** (structural) | §1→AC→§5 traceability | **PASS** | 1:1: 1.1→AC1, 1.2→AC2, 1.3→AC3, 1.4→AC4 (TRACE-001 ✓). Each AC names a concrete arm `test_install_hygiene.sh::<arm>` (TRACE-002 ✓). Test file resolves on disk (37,963 B) and is in `modified_files` (TRACE-003 ✓); the four arms are not yet authored — correct for a draft. Lint-confirmed. |
| **TRACE-006** (judgment) | cited test exercises clause's VERB | **PASS** | Per-clause verb→assertion comparison in §5 below; all four ACs describe assertions that discharge their clause's verb. |
| **XCHAIN / STALE** | cross-skill / staleness | **N/A** | Standalone audit (no author manifest supplied); no `provenance.source_hash` to diff. |

## §3 — Citation verification (independent — the crux)

Each source_pages / §Problem claim was opened in the working-tree files (`git --no-optional-locks status`: install.sh & uninstall.sh both clean, worktree == HEAD, 1055 / 230 lines) and confirmed at the cited lines.

| # | Spec claim | Confirmed in code | Verdict |
| - | ---------- | ----------------- | ------- |
| 1 | `install.sh:694-697` writes `.mcp.json` + `.cursor/mcp.json` (MCP registration), each → `.cyberos/mcp/cyberos-mcp.mjs` | Block 693–698. L694 `mcp_json()` emits `"args":[".cyberos/mcp/cyberos-mcp.mjs"]`; **L696** writes `.mcp.json` (create-if-absent); **L697** writes `.cursor/mcp.json` under `want_agent cursor`. | **TRUE** — exact |
| 2 | `uninstall.sh` has ZERO mcp handling (`grep -ci mcp` == 0) | `grep -ci mcp tools/install/uninstall.sh` → **0**; case-sensitive `grep -ni` → **no matches**. No `.mcp.json` / `.cursor/mcp.json` unregistration anywhere. | **TRUE** — exact |
| 3 | `install.sh:632-637` installs skills for grok / command-code / codex / opencode (plus claude-code) | **L632-636** are the five `install_skill` family calls: claude-code (632), grok (633), command-code (634), codex (635), opencode (636); L637 begins the create-tasks comment. `install_skill` (L609/622) symlinks `→ …/.cyberos/plugin/skills/<skill>`. | **TRUE** — exact |
| 4 | `uninstall.sh:98-127` removes only the `.agents/skills` trio + `.claude/skills` create-tasks pair; `:125` leaves `.claude/skills/ship-tasks`; never touches grok/command-code/codex/opencode | Section 2b (L97–135). **L106** loops exactly `ship-tasks task-author task-audit`; `.agents/skills` handled L107–123; **L126** guards `.claude/skills` with `[ "$_sc" != "ship-tasks" ]` (so only task-author + task-audit), L124-125 comment states ship-tasks is left in place. No `.grok/.commandcode/.codex/.opencode` path appears anywhere in the file → those four families' `ship-tasks` entries are never removed. | **TRUE** — exact |
| 5 | `install.sh:860-861` heredoc's FIRST line is a blank separator before the `# >>> cyberos-status-hook v2 … >>>` marker; `uninstall.sh:78` strips `>>>`…`<<<` INCLUSIVE — the leading blank is OUTSIDE the range → accumulates each cycle | **L860** `cat >> "$hk" <<'HOOK'`; **L861** confirmed **empty** (`cat -A` → bare `$`); **L862** `# >>> …v2… >>>`; **L894** `# <<< cyberos-status-hook <<<`. **uninstall.sh L78** `sed '/# >>> cyberos-status-hook/,/# <<< cyberos-status-hook <<</d'` deletes marker-to-marker inclusive — the L861-style blank is not in the range and survives. v1→v2 upgrade path (install.sh L855-857, cited `:856`) shares the shape and the same inclusive strip. | **TRUE** — real & CURRENT bug, not healed |

**Citation findings: none.** All five claims hold at (or within 0–2 lines of) the cited locations; the line numbers are accurate against the current worktree. The author also correctly declined to adopt the handoff's unverifiable "5 dangling symlinks" count, specifying the invariant (zero dangling links, clause 1.2) instead — sound discipline.

## §4 — TRACE-006 per-clause records (verb demanded vs. assertion described)

Tests are unwritten (draft gate), so each clause verb is compared against the assertion the AC *describes* its arm will make.

- **1.1 — verbs `remove` + `MUST NOT remove`.** Demands: cyberos-written `.mcp.json` absent after uninstall; operator's own file present. **AC1** asserts uninstall removes the written `.mcp.json` AND a pre-existing operator `.mcp.json` is left untouched → both verbs discharged (absence of ours; preservation of theirs). Secondary limb `.cursor/mcp.json` removal: carried by the arm name `t_mcp_registration_removed` + §3 edge case, not spelled out in AC1 prose (see ISS-001, non-blocking).
- **1.2 — verb `remove … leaving zero`.** Demands: zero symlinks resolve into `.cyberos/plugin/skills` after machine removal, across every family install writes. **AC2** asserts exactly that ("zero skill links resolve … checked across every family") — discharges the observable, not a mere "logged removed". The "unmarked operator dir left in place" limb is discharged by **AC4**. ✓
- **1.3 — verb `preserve` (exact inverse / byte-identical across cycles).** Demands: foreign hook byte-identical to pre-install content, and stable across cycles. **AC3** runs install→uninstall→install→uninstall and asserts byte-identity with "no accumulated blank line" — a before/after equality that specifically witnesses the per-cycle accumulation the bug produces (2 cycles is a sufficient witness: buggy code shows 2 stray blanks, fixed shows 0). ✓
- **1.4 — verb `survive` (preserve).** Demands: operator `.mcp.json`, unmarked `.agents/skills/<cmd>` dir, and foreign-hook lines outside the managed block all survive. **AC4** asserts all three survive uninstall — direct 1:1 with the clause's three limbs. ✓

**Security-class edge case (SAFE/adequacy).** §3's final row states uninstall reads paths and content and executes nothing, and confines paths under the repo root on the same `relUnderRoot` rule the other helpers use ("a crafted target cannot walk out"). Adequate for a task that mutates the destructive uninstall path: it names the threat (path traversal via a crafted target), the invariant (repo-root confinement), and the no-exec property. The operator-file-preservation safety (clause 1.4 / AC4, reinforced by the Success-Metrics Guardrail) is present and adequate.

## §5 — Findings (itemised)

```
ISSUE
id:              ISS-001
rule_id:         TRACE-006 (advisory; clause cited & primary verb discharged — TRACE-001/006 PASS)
status:          wontfix
severity:        info
location:        line 166 (AC1) vs line 147-149 (clause 1.1)
evidence:        "AC1 prose names only `.mcp.json`; clause 1.1 also requires removing `.cursor/mcp.json` when present."
description:     "Clause 1.1's second limb (remove `.cursor/mcp.json` when present) is covered
                 by the arm name `t_mcp_registration_removed` and the §3 edge case ('uninstall
                 MUST remove it when present and MUST NOT fail when absent'), but AC1's prose
                 asserts only `.mcp.json`. Traceability is intact (clause 1.1 IS cited by AC1)
                 and the primary verb is discharged; this is a prose-explicitness nit, not a
                 gap that lets untested behaviour ship."
suggestion:      "Optional: have AC1 (or a dedicated arm) name the `.cursor/mcp.json`
                 removal + absent-no-fail assertion explicitly, so the cursor limb is visible
                 at the AC layer and not only in §3."
auto_fix_applied: false
resolution:      "Left to author discretion; non-blocking for the draft→ready_to_implement gate."
opened_at:       "2026-07-19T00:00:00Z"

ISSUE
id:              ISS-002
rule_id:         QA-006-adjacent (clarity; no error-severity rule hit)
status:          wontfix
severity:        info
location:        line 113 (## Success Metrics, Guardrail)
evidence:        "'the existing spec 1.3 \"never touch operator files\" promise is not weakened'"
description:     "The Guardrail cites 'spec 1.3' for the never-touch-operator-files promise. In
                 THIS task that promise is clause 1.4; 'spec §1.3' is the prior installer-spec
                 numbering echoed in uninstall.sh's own comments (e.g. L120 'Spec §1.3: never
                 touch operator files'). Faithful to the code's vocabulary, but a reader of this
                 spec could momentarily conflate it with this task's clause 1.3 (the hook-strip)."
suggestion:      "Optional: qualify as 'the prior uninstall spec's §1.3' or point at this task's
                 clause 1.4 to remove the numbering collision."
auto_fix_applied: false
resolution:      "Left to author discretion; non-blocking."
opened_at:       "2026-07-19T00:00:00Z"
```

No error-severity or `needs_human` issues were found. Per the skill's MUST-NOT ("invent rule violations"), no findings were manufactured to pad a count — the spec is clean and the two items above are the genuine, non-blocking observations.

```
SUMMARY
verdict:         pass
score:           10 / 10
issues_total:    2
issues_open:     0
issues_human:    0
issues_fixed:    0
issues_wontfix:  2
iterations:      1
machine_floor:   "task-lint.mjs → exit 0 (zero error-severity findings)"
citations:       "5 / 5 confirmed TRUE at cited lines (incl. current, un-healed hook newline bug)"
next_action:     "ship — eligible for draft → ready_to_implement"
```

---

*Independent audit of TASK-IMP-126 spec.md — audit_rubric@2.0 — 2026-07-19.*
