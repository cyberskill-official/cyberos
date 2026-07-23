---
id: TASK-CUO-208
title: "task template profile - /create-tasks resolves engineering-spec@1 vs task@1 per repo, and the audit rubric follows"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: cuo
priority: p1
status: done
verify: T
phase: Wave C - strengthen the workflows
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-CUO-205, TASK-CUO-207]
depends_on: [TASK-CUO-207]
blocks: []
source_pages:
  - tools/install/plugin/commands/create-tasks.md
  - modules/skill/task-author/SKILL.md
  - modules/skill/task-audit/RUBRIC.md
source_decisions:
  - "2026-07-12 investigation: the repo runs two task templates - engineering-spec@1 (cyberos-native §1..§11; ~470 files incl. every improvement-class task) and task@1 (generic; 6 files). The plugin command says task@1 while the author skill's §12 authors engineering-spec@1; audit_rubric@2.0's FM/SEC/COND families target task@1 while TRACE targets the §-sections. External repos inherit this ambiguity on day one."
  - "Resolution: template becomes an explicit per-repo profile (config key from TASK-CUO-207), defaulting to engineering-spec@1; both templates stay first-class."
language: markdown (skill contracts + command doc)
service: modules/skill/ + tools/install/plugin/
new_files:
  - modules/skill/task-author/references/TEMPLATE_PROFILES.md
modified_files:
  - modules/skill/task-author/SKILL.md
  - modules/skill/task-audit/SKILL.md
  - modules/skill/task-audit/RUBRIC.md
  - tools/install/plugin/commands/create-tasks.md
---

# TASK-CUO-208: task template profile

## §1 - Description

Make the template decision explicit, per repo, in one place - instead of a plugin command, an author skill, and a rubric each implying a different answer.

Normative clauses:

1. A reference `TEMPLATE_PROFILES.md` MUST define both templates normatively side by side: `engineering-spec@1` (frontmatter field set; `## §1 - Description` .. `## §11 - Implementation notes` section grammar; end marker) and `task@1` (its frontmatter incl. `template:` key; `## Summary` .. `## Dependencies` + conditional sections). Each profile lists its applicable audit rule families.
2. `/create-tasks` MUST resolve the active template as: explicit per-invocation operator override, else `.cyberos/config.yaml` `task_template` (TASK-CUO-207), else default `engineering-spec@1` - and MUST echo the resolved template in its PLAN so the operator approves template + content together.
3. `task-author` MUST accept the template as an input-envelope field and emit the selected profile faithfully; its §12 authoring rules apply to engineering-spec@1, and TEMPLATE_PROFILES.md carries the equivalent authoring rules for task@1.
4. `task-audit` MUST select rule families by detected template: task@1 -> FM + SEC + COND + QA + SAFE (+ TRACE only where the grafted §4/§5 sections are present, as RUBRIC §9 already states); engineering-spec@1 -> the §12 sub-rule set + TRACE-001..005 + QA + SAFE. The 10/10 verdict bar and needs_human semantics MUST be identical across templates. Template detection MUST come from the file itself (`template:` key present -> task@1; §-section grammar -> engineering-spec@1) and a file matching neither or both MUST be needs_human.
5. The plugin command doc MUST stop naming task@1 as THE format (current step-1 wording) and instead name the resolution chain from #2.
6. Mixed-template repos MUST be supported: the template is resolved per invocation batch, and the audit judges each file by its own detected template regardless of the repo default.

## §2 - Why this design

The two templates serve different consumers - engineering-spec@1 is the build-grade contract with traceability into tests; task@1 is the lighter product/compliance shape - so deleting either would break real users of the other. A per-repo default with per-file detection keeps the common path one-line simple while making the audit honest about what it is scoring. Defaulting to engineering-spec@1 follows the repo's own revealed preference (470:6) and the ship workflow's TRACE dependency.

## §3 - Contract

Author input envelope gains: `"template": "engineering-spec@1" | "task@1"` (optional; default per §1 #2 chain). Audit output gains: `"template_detected"` echoing the per-file detection. PLAN echo line: `template: engineering-spec@1 (source: config)`.

## §4 - Acceptance criteria

1. **Profiles are complete and normative** (§1 #1) - TEMPLATE_PROFILES.md defines both frontmatter sets, both section grammars, end markers, and each profile's rule-family list; the repo's newest exemplar of each template validates against its profile as written.
2. **Resolution chain honored** (§1 #2) - fixtures: invocation override beats config; config beats default; absent both -> engineering-spec@1; the PLAN echo names value + source in every case.
3. **Author emits both faithfully** (§1 #3) - one sample task authored per template from the same interview fixture carries the correct sections and frontmatter for its profile (acceptance fixtures under the author skill).
4. **Audit families switch on detection** (§1 #4) - a task@1 fixture missing `## Alternatives Considered` fails SEC-004; an engineering-spec@1 fixture missing §10 fails the §12 structural rule; the same 10/10 bar gates both.
5. **Ambiguity is needs_human** (§1 #4) - a fixture with a `template: task@1` key AND §1..§11 sections routes to needs_human naming the conflict.
6. **Command doc updated** (§1 #5) - create-tasks.md names the chain and no longer asserts a single format.
7. **Per-file judgment in mixed repos** (§1 #6) - a batch containing one file of each template audits each against its own families (fixture pair, both pass).

## §5 - Verification

Acceptance-driven (contract work):

- `modules/skill/task-author/references/TEMPLATE_PROFILES.md` - carries its own "verify this document" preamble: the two exemplar-validation checklists used by AC 1.
- Author acceptance fixtures (extend `modules/skill/task-author/acceptance/TRIGGER_TESTS.md` with the template-resolution cases) - AC 2, 3.
- Audit acceptance fixtures (extend `modules/skill/task-audit/acceptance/TRIGGER_TESTS.md` with the four detection/family cases) - AC 4, 5, 7.
- Doc assertion for AC 6: create-tasks.md contains the resolution chain wording; `grep -c "task@1 task markdowns"` returns 0.

## §6 - Implementation skeleton

TEMPLATE_PROFILES.md: two mirrored halves + a comparison table + the detection rules. SKILL.md diffs: input-envelope field, PLAN echo, pointer to profiles. RUBRIC.md: a short "family selection by template" preamble above the existing families (no rule rewrites - FM-004 stays as-is for task@1; engineering-spec files are simply not subject to FM-004).

## §7 - Dependencies

Depends on TASK-CUO-207 (`task_template` config key). Composes with TASK-CUO-205 (same command doc, different step - #5 here touches step 1, 205 touches step 3).

## §8 - Example payloads

```
PLAN (4 tasks) - template: engineering-spec@1 (source: default)
  TASK-ACME-001-payment-webhooks (product)
  ...
approve to write files.
```

## §9 - Open questions

None blocking. A third template slot (client-specific house style) is deliberately out: profiles are code-reviewed contracts, not config-invented shapes; a new template means a new TEMPLATE_PROFILES.md entry by task.

## §10 - Failure modes inventory

1. Config says task@1 but the repo's existing tasks are engineering-spec - per-file detection (#6) keeps audits honest; the PLAN echo warns when resolved template differs from the majority of existing tasks (informational line, not a block).
2. Author asked to CONVERT between templates - out of scope; the audit's needs_human on hybrids prevents silent half-conversions.
3. Rubric drift (a rule added to one family list but not the profile doc) - TEMPLATE_PROFILES.md's family lists cite RUBRIC.md family names, not rule IDs, so rule additions inherit automatically; family additions require touching both files and AC 1's checklist catches a miss.
4. Detection false-positive on prose containing "## Summary" - detection requires the frontmatter `template:` key for task@1, not the section alone (§1 #4); the section grammar is the tiebreaker only for the engineering-spec side.
5. External repo with zero config and zero existing tasks - default engineering-spec@1 + PLAN echo makes the choice visible at the approval gate where it is cheapest to change.

## §11 - Implementation notes

Keep FM-004 untouched (it correctly binds `template: task@1` files). The engineering-spec §12 sub-rules stay in the author SKILL.md as today; TEMPLATE_PROFILES.md references them rather than duplicating, so there is exactly one normative home per rule set.

*End of TASK-CUO-208.*
