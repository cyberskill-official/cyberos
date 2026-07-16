---
id: TASK-IMP-084
title: task-lint, a deterministic machine floor under the task-audit rubric
template: task@1
type: improvement
module: improvement
status: reviewing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T11:45:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-SKILL-113]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 hardening"
owner: Stephen Cheng (CTO)
created: 2026-07-16
shipped: null
memory_chain_hash: null
effort_hours: 5
service: tools/install/docs-tools
new_files:
  - tools/install/docs-tools/task-lint.mjs
  - tools/install/tests/test_task_lint.sh
modified_files:
  - modules/skill/task-audit/SKILL.md
source_pages:
  - "modules/skill/task-audit/RUBRIC.md §1-§2 (FM-001..114 enums and formats), §3 (SEC-001..009), §4 (COND-001..004), §9 (TRACE-001..003 structural halves)"
  - "modules/skill/task-audit/RUBRIC.md line 1 ('machine-checkable Task rubric') - the name promises what nothing executes today"
  - "tools/install/build.sh:165-171 (docs-tools vendors what exists - a new .mjs lands in the payload automatically; presence still needs a gate)"
  - "IMPROVEMENT_HANDOFF.md IMP-03 (sachviet run 2026-07-16: all six audits were model-applied; the mechanical majority of findings rode on model diligence alone)"
source_decisions:
  - "2026-07-16 Stephen: PLAN batch 1 approved with this item at p1."
---

# TASK-IMP-084: task-lint, a deterministic machine floor under the task-audit rubric

## Summary

audit_rubric@2.0 calls itself machine-checkable, but nothing executes it - every FM enum, required section, and traceability citation is re-derived by a model on each audit. Ship `task-lint.mjs` in docs-tools: a zero-dependency node CLI that checks the mechanical rule families deterministically, emits rule_id-tagged findings, and exits non-zero on error severity. The task-audit skill runs it first and judges only what needs judgment (QA semantics, SAFE content, TRACE meaning). Consumer repos inherit the tool through the payload's docs-tools copy.

## Problem

On the 2026-07-16 sachviet run, six spec audits were performed entirely by the model. The findings that mattered were judgment calls, but the majority of rubric surface walked each time was mechanical: frontmatter enums, title length, snake_case keys, required H2s, conditional sections, clause-to-AC citation presence, test-path resolution. A model re-checking those is spending its diligence budget where a 300-line script is strictly better - byte-stable, instant, and immune to fatigue. The rubric's own header makes the promise:

<untrusted_content source="modules/skill/task-audit/RUBRIC.md:1">
# `audit_rubric@2.0` - machine-checkable Task rubric
</untrusted_content>

## Proposed Solution

`node .cyberos/docs-tools/task-lint.mjs <spec.md ... | dir>` (in the platform repo: `tools/install/docs-tools/task-lint.mjs`). Node stdlib only. It parses frontmatter with a strict minimal YAML reader (scalars, lists, quoted strings - the subset the template uses; anything beyond that is itself an FM-001 finding), walks the body headings, and checks the mechanical families. Output: one line per finding, `SEVERITY rule_id file:line message`, sorted; `--json` for machines; exit 0 only when zero error-severity findings. The task-audit skill's loop gains one normative step: run the lint first when present, seed the report's mechanical findings from it, then perform the judgment families. The lint never replaces the audit; it floors it.

## Alternatives Considered

- A real YAML dependency (js-yaml) for parsing. Rejected: docs-tools is stdlib-only by convention (md.mjs, render-status-hub.mjs), payload stays dependency-free, and the template's frontmatter subset is small; a strict subset parser that FAILS LOUDLY on exotic YAML is safer than silently accepting what the audit families never defined.
- Extending an existing tool (repair_task_yaml.py). Rejected: that script mutates (repairs); the lint must be read-only and rule_id-faithful, and mixing repair with verdicts blurs who changed what.
- Wiring the lint as a hard gate in run-gates.sh now. Rejected for this task: gates belong to per-repo config; the skill-level wiring gets the value everywhere immediately, and repos can add it to config.yaml gates themselves. Revisit after adoption.

## Success Metrics

- Primary: the mechanical rule families run in milliseconds with zero model involvement - lint verdict on the three batch-1 specs and a green corpus sample matches the model audits' mechanical findings (no false errors). Baseline: 0 percent of rubric surface machine-executed. Deadline: this task's final acceptance.
- Guardrail: seeded violation fixtures (one per family) each produce exactly their rule_id and a non-zero exit, on every suite run.

## Scope

In scope: the lint CLI, its fixture-driven test suite, one normative wiring line in task-audit SKILL.md, payload presence gating.

### Out of scope / Non-Goals

- Judgment families: QA-001..009 semantics (vanity metrics, risk-class dodging), SAFE-003 content scanning beyond marker presence, TRACE semantic sufficiency, XCHAIN/STALE manifest cross-checks.
- Auto-fixing anything (read-only tool).
- engineering-spec@1 profile rules (the lint targets template: task@1 detection; other profiles report `template_ambiguous` and stop, mirroring RUBRIC §10).
- run-gates.sh wiring (per-repo choice, later item).

## Dependencies

- None upstream. Cone-disjoint from TASK-IMP-082 (renderer) and TASK-IMP-083 (install.sh); ships in the same parallel batch. Downstream: IMPROVEMENT_HANDOFF IMP-04's helpers can reuse its frontmatter reader.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted by the model from IMPROVEMENT_HANDOFF.md IMP-03 plus rubric source mapping; implementation follows under ship-tasks supervision.
- **Human review:** PLAN approved by the operator on 2026-07-16; spec audit and both HITL acceptance gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 The lint MUST accept one or more spec paths or directories (directories recurse to `*/spec.md`) and MUST run on node stdlib alone.
- 1.2 The lint MUST implement the FM family mechanically: FM-001 (fences + parseable frontmatter per the strict subset), FM-002 (snake_case keys), FM-003 (duplicate keys), FM-004 (template equals task@1), FM-101 (title 1-72 after trim), FM-102 (author regex), FM-103/104/105/107/108/109 (closed enums, with FM-109 rejecting `unacceptable`), FM-106 (ISO 8601 with timezone), FM-110 (semver-or-quarter when present), FM-111 (real YAML boolean), FM-112 (no `# UNREVIEWED` marker), FM-113 (duplicate_of iff status duplicate, resolving to an existing task folder), FM-114 (severity iff type bug).
- 1.3 The lint MUST implement SEC mechanically: SEC-001..007 required H2s present, SEC-008 each non-empty (at least one non-blank body line before the next heading), SEC-009 single H1 and no skipped heading levels (warning severity).
- 1.4 The lint MUST implement COND triggers mechanically: COND-001/002 sections when client_visible true, COND-003 with its three H3s in order when risk class is limited or high, COND-004 with the three labeled bullets when ai_authorship is not none.
- 1.5 The lint MUST implement the structural TRACE halves: TRACE-001 presence form (every numbered §1 clause containing a BCP-14 keyword is cited by at least one AC via `§1 #N` or `traces_to`), TRACE-002 presence form (every AC line carries a `test:` or `verify:` entry), TRACE-003 (every `test:` path segment before `::` is listed in frontmatter new_files or exists on disk). Semantic sufficiency stays with the model audit.
- 1.6 Findings MUST be emitted one per line as `SEVERITY rule_id file:line message`, sorted bytewise, with `--json` emitting the same findings as a JSON array; two runs on identical input MUST be byte-identical.
- 1.7 Exit code MUST be 0 with no error-severity findings, 2 otherwise; unreadable input or non-task@1 template detection MUST report `template_ambiguous` at error severity rather than guessing.
- 1.8 `modules/skill/task-audit/SKILL.md` MUST gain one normative loop step: when the lint is present (`.cyberos/docs-tools/task-lint.mjs` in installed repos, `tools/install/docs-tools/task-lint.mjs` in the platform repo), run it FIRST and seed the report's mechanical findings from its output; the model then audits the judgment families only.
- 1.9 A gating suite MUST land at `tools/install/tests/test_task_lint.sh`: the tool passes on this batch's three specs; one fixture per family (FM, SEC, COND, TRACE) produces exactly its rule_id and exit 2; the assembled payload carries `docs-tools/task-lint.mjs` and a scratch install lays it into `.cyberos/docs-tools/`.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: §1 #1.1, #1.6, #1.7) - CLI shape, deterministic sorted output, exit codes - test: `tools/install/tests/test_task_lint.sh::t01_cli_and_determinism`
- [ ] AC 2 (traces_to: §1 #1.2) - FM family fixtures each yield their rule_id - test: `tools/install/tests/test_task_lint.sh::t02_fm_family`
- [ ] AC 3 (traces_to: §1 #1.3) - SEC family fixtures - test: `tools/install/tests/test_task_lint.sh::t03_sec_family`
- [ ] AC 4 (traces_to: §1 #1.4) - COND trigger fixtures - test: `tools/install/tests/test_task_lint.sh::t04_cond_family`
- [ ] AC 5 (traces_to: §1 #1.5) - TRACE structural fixtures (uncited clause, AC without test, dangling test path) - test: `tools/install/tests/test_task_lint.sh::t05_trace_family`
- [ ] AC 6 (traces_to: §1 #1.9) - green corpus: batch-1 specs lint clean - test: `tools/install/tests/test_task_lint.sh::t06_green_corpus`
- [ ] AC 7 (traces_to: §1 #1.9) - payload carries the tool and install lays it down - test: `tools/install/tests/test_task_lint.sh::t07_payload_and_install`
- [ ] AC 8 (traces_to: §1 #1.8) - the skill wiring line exists and names the lint-first order - test: `tools/install/tests/test_task_lint.sh::t08_skill_wiring_present`

## 3. Edge cases

- Frontmatter using YAML the strict subset does not parse (anchors, multiline blocks): FM-001 error naming the line - loud, never a silent skip (t02 includes one).
- A spec with zero numbered §1 clauses (pure PRD shape): TRACE-001 has nothing to check - the lint reports an info-level note, not an error (template allows judgment there; the model audit decides).
- `verify:` ACs (justified ops verification) satisfy TRACE-002's structural half by design - the lint checks presence, the model checks the justification (t05 includes one passing verify case).
- CRLF files, BOM, trailing whitespace in headings - normalized for heading matching, reported nowhere (content bytes are the corpus's business).
- Huge corpora: per-file processing, findings streamed after sort; no cross-file state except FM-113 resolution, which scans task folder names only.
- Unicode titles: length counted in code points after trim (documented; FM-101's 72 is a display bound, not a byte bound).
- Security-class: the lint reads files and never executes content; `--json` output is data, not code. The injection-marker scan (SAFE-003) stays with the model by scope - noted so nobody assumes the lint covers it.

## 4. Out of scope / non-goals

Duplicated intentionally with `## Scope` for template conformance: judgment families, auto-fix, other template profiles, and gate wiring are excluded.

## 5. Protected invariants this task must not weaken

- The lint floors the audit, never replaces it - the skill text keeps the model responsible for judgment families and the 10/10 verdict.
- docs-tools stays node-stdlib-only.
- Payload sync doctrine: rebuild dist, version-sync, full suite before commit.
- HITL: both human-acceptance gates are recorded verdicts; the agent never sets done.

*End of TASK-IMP-084.*
