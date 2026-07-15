---
id: TASK-SKILL-117
title: "Author the architectural-spike-author/-audit pair - the missing ADR input artefact (architectural-spike@1)"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: SKILL
priority: p0
status: done
verify: T
phase: Wave B - finish the children
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-SKILL-116, TASK-SKILL-118, TASK-CUO-209]
depends_on: []
blocks: [TASK-CUO-209]
source_pages:
  - modules/skill/architecture-decision-record-author/SKILL.md
  - modules/cuo/docs/appendices.md
source_decisions:
  - "2026-07-12 investigation: architecture-decision-record-author declares its input as 'an approved SRS plus architectural-spike output', but no architectural-spike skill exists anywhere in modules/skill/ (256 entries scanned). It is the only referenced artefact with no producer."
  - "Layout must match the full-pair convention (task-author/-audit file set) so TASK-SKILL-118's parity checker covers it from day one."
language: markdown + JSON (skill contracts)
service: modules/skill/
new_files:
  - modules/skill/architectural-spike-author/SKILL.md
  - modules/skill/architectural-spike-author/PIPELINE.md
  - modules/skill/architectural-spike-author/INVARIANTS.md
  - modules/skill/architectural-spike-author/envelopes/input.json
  - modules/skill/architectural-spike-author/envelopes/output.json
  - modules/skill/architectural-spike-author/references/FAILURE_MODES.md
  - modules/skill/architectural-spike-author/acceptance/TRIGGER_TESTS.md
  - modules/skill/architectural-spike-audit/SKILL.md
  - modules/skill/architectural-spike-audit/RUBRIC.md
  - modules/skill/architectural-spike-audit/AUDIT_LOOP.md
  - modules/skill/architectural-spike-audit/REPORT_FORMAT.md
  - modules/skill/architectural-spike-audit/envelopes/input.json
  - modules/skill/architectural-spike-audit/envelopes/output.json
  - modules/skill/architectural-spike-audit/acceptance/TRIGGER_TESTS.md
modified_files:
  - modules/skill/architecture-decision-record-author/SKILL.md
---

# TASK-SKILL-117: architectural-spike-author / -audit

## §1 - Description

A spike is the time-boxed investigation that turns "we have 2+ plausible architectures" into evidence an ADR can cite. The ADR skill already demands spike output; nothing can produce it. This task authors the pair and its artefact contract.

Normative clauses:

1. A new artefact type `architectural-spike@1` MUST be defined with frontmatter: `spike_id` (SPIKE-<task-ID>-<n>), `task_id`, `question` (the single decision under investigation), `timebox_hours` (int, recorded up front), `options` (array, each `{name, hypothesis, evidence[], cost_estimate, risks[]}`), `recommendation` (names exactly one option), `confidence` (low|medium|high), `discarded` (array of `{name, reason}`), `created`. Body sections: `## Question`, `## Options probed`, `## Evidence log`, `## Recommendation`, `## Discard log`.
2. `architectural-spike-author` MUST: take an audited task + repo-context-map as inputs (input envelope), declare its trigger conditions (an ADR is pending AND >= 2 viable options exist, or the task introduces a dependency the repo has never used), enforce the timebox by recording planned vs actual hours, and HALT for the operator when actual exceeds planned by more than 50%.
3. Every `evidence[]` entry MUST cite something checkable: a file path in the repo, a command + its observed output, or an external URL. Unsupported assertions ("X is faster") without a citation MUST NOT count as evidence.
4. `architectural-spike-audit` MUST score /10 against a RUBRIC.md with at least these rule families: SPK-STRUCT (frontmatter + sections complete, recommendation names exactly one probed option), SPK-EVID (>= 2 options probed, every option has >= 1 checkable evidence entry, recommendation cites evidence), SPK-BOX (timebox recorded, overrun HALT honored), SPK-DISC (discard log non-empty whenever options were rejected). Only 10/10 passes; the audit report format mirrors REPORT_FORMAT.md conventions of the existing full pairs.
5. `architecture-decision-record-author/SKILL.md` MUST be updated to name `architectural-spike@1` as its spike input (replacing the dangling prose reference), and to state the fallback when no spike exists (lean profile: ADR options table carries the evidence inline).
6. Both skills MUST carry acceptance `TRIGGER_TESTS.md` with >= 6 cases each (>= 3 positive, >= 3 negative), following the conventions of the existing trigger-test files (TASK-SKILL-112 lineage).
7. File layout MUST match the full-pair convention exactly as listed in `new_files`, so the TASK-SKILL-118 parity checker passes over this pair with zero exemptions.

## §2 - Why this design

The artefact is deliberately evidence-first: the rubric makes uncited claims worthless, which is what separates a spike from an opinion. Timebox recording (plan vs actual, HALT on 1.5x) encodes the whole point of a spike - bounded spend - as a checkable invariant rather than advice. Reusing the full-pair file layout means no new tooling: parity checks, chain-coverage checks, and the plugin build all treat the pair like the existing ten.

## §3 - Contract

Input envelope (author): `{fr_path, repo_context_map_path, question, timebox_hours, output_dir}`. Output envelope: `{spike_path, spike_id, recommendation, confidence, halted}`. Audit input: `{spike_path}`; audit output: `{verdict: pass|fail|needs_human, score, findings[]}`. Envelope JSONs are normative in `envelopes/`.

## §4 - Acceptance criteria

1. **Artefact contract is complete** (§1 #1) - SKILL.md's artefact section defines every frontmatter field with types and the five body sections; a sample spike in the acceptance fixtures parses against it.
2. **Trigger conditions are declared and tested** (§1 #2, #6) - author TRIGGER_TESTS.md contains >= 6 cases including "ADR pending with 2 options" (fire) and "single obvious option" (no fire).
3. **Timebox HALT is normative** (§1 #2) - PIPELINE.md contains the plan/actual recording step and the > 1.5x HALT with operator wording; INVARIANTS.md lists it as an invariant.
4. **Evidence rule is enforceable** (§1 #3, #4) - RUBRIC.md's SPK-EVID rules reject a fixture spike whose option carries only an uncited claim; the fixture and expected finding appear in audit acceptance tests.
5. **Rubric families complete, 10/10 gate** (§1 #4) - RUBRIC.md defines SPK-STRUCT/EVID/BOX/DISC with rule IDs and states the 10/10 pass bar.
6. **ADR input is wired** (§1 #5) - architecture-decision-record-author/SKILL.md names architectural-spike@1 and the lean-profile fallback; the dangling prose is gone.
7. **Layout parity** (§1 #7) - every file in `new_files` exists; the TASK-SKILL-118 parity check (or, before it lands, a manual `ls` against the task pair layout) shows no missing file class.

## §5 - Verification

Acceptance-driven (skills are contracts, not code):

- `modules/skill/architectural-spike-author/acceptance/TRIGGER_TESTS.md` - AC 2 cases (>= 6).
- `modules/skill/architectural-spike-audit/acceptance/TRIGGER_TESTS.md` - AC 4 fixture + expected SPK-EVID finding; plus a 10/10 clean fixture.
- Structural check for AC 1, 3, 5, 6, 7: `bash tools/cyberos-init/check-chain-coverage.sh` pair rule (from TASK-SKILL-116) plus `grep` assertions listed inline in the two TRIGGER_TESTS.md preambles (each acceptance file opens with its own "how to verify this pair" block, executable line by line).

## §6 - Implementation skeleton

Author SKILL.md follows the section order of implementation-plan-author (description, triggers, inputs, artefact spec, pipeline pointer, HALT semantics). RUBRIC.md follows task-audit/RUBRIC.md's family/rule-ID formatting. Envelope JSONs mirror the existing pairs' shape (json-schema-lite objects with required arrays).

## §7 - Dependencies

None upstream (authoring is self-contained). Blocks TASK-CUO-209 (the pair joins the vendored set there). TASK-SKILL-118's parity checker adopts this pair automatically once both land.

## §8 - Example payloads

```yaml
# spike frontmatter (fixture)
spike_id: SPIKE-TASK-MEMORY-130-1
task_id: TASK-MEMORY-130
question: "MMR chain vs plain hash chain for audit checkpoints?"
timebox_hours: 6
options:
  - name: mmr
    hypothesis: "O(log n) inclusion proofs justify complexity"
    evidence: ["bench: scripts/bench_mmr.sh output 2026-07-12", "modules/memory/cyberos/data/AGENTS.md §6.4"]
    cost_estimate: "3-4 days"
    risks: ["new dependency surface"]
recommendation: mmr
confidence: medium
discarded: [{name: plain-hash, reason: "linear proof size fails NFR-PERF-012"}]
```

## §9 - Open questions

None blocking. Whether ship-tasks gains an optional spike step between steps 2 and 3 is TASK-CUO-209 territory (workflow change), not this pair's.

## §10 - Failure modes inventory

1. Spike becomes a design doc (unbounded) - SPK-BOX fails the audit when actual hours are absent or the HALT was skipped; the timebox is data, not vibes.
2. Evidence rot (cited file later deleted) - audit checks citations resolve AT AUDIT TIME; later rot is caught by TASK-SKILL-119's anchor checker class, out of scope here.
3. Recommendation names an unprobed option - SPK-STRUCT explicit rule; fixture covers it.
4. Author invoked with no real alternatives - negative trigger tests pin "single obvious option -> do not fire"; the ADR proceeds without a spike per §1 #5 fallback.
5. Confidence inflation - confidence is an enum the audit cross-checks: `high` with a single evidence entry per option triggers a SPK-EVID finding.

## §11 - Implementation notes

Keep artefact version pinned at @1 in both SKILL.md files; version bumps follow the contracts CHANGELOG discipline. The pair is NOT added to build.sh here - vendoring is TASK-CUO-209's single expansion, kept out of this task to avoid two writers on build.sh:28.

*End of TASK-SKILL-117.*
