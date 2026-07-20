---
id: TASK-PLUGIN-004
title: "Skill playbooks bundle — Anthropic-Agent-Skills SKILL.md files teaching hosts how to chain plugin tools correctly"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-19T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PLUGIN
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PLUGIN-001, TASK-PLUGIN-002, TASK-PLUGIN-003, TASK-SKILL-111, TASK-SKILL-112, TASK-SKILL-113, TASK-SKILL-114, TASK-SKILL-115]
depends_on: [TASK-PLUGIN-002, TASK-SKILL-111]
blocks: [TASK-PLUGIN-007]

source_pages:
  - "[Plugin docs](https://cyberos-wiki.cyberskill.world/modules/plugin/) §1 (Skill playbooks layer)"
  - "[SKILL Appendix L](https://cyberos-wiki.cyberskill.world/modules/skill/appendices.html) (SKB-020..066 conformance)"
  - "[SKILL Appendix J](https://cyberos-wiki.cyberskill.world/modules/skill/appendices.html) (Anthropic Skills portability)"

source_decisions:
  - DEC-2430 2026-05-19 — Plugin ships 12 skill playbooks at modules/plugin/skills/<name>/SKILL.md following Anthropic Agent Skills spec
  - DEC-2431 2026-05-19 — Playbooks are routing+discipline, not tools — they teach hosts how to chain TASK-PLUGIN-002 tools correctly; they do NOT add new MCP tools
  - DEC-2432 2026-05-19 — Every playbook MUST pass SKILL_BUNDLE_RUBRIC SKB-020..023 (description format + XML-free frontmatter) per TASK-SKILL-111/113
  - DEC-2433 2026-05-19 — Every playbook MUST carry acceptance/TRIGGER_TESTS.md with 4 positive + 4 negative fixtures per TASK-SKILL-112
  - DEC-2434 2026-05-19 — Playbooks are organised by use-case (orchestration, memory, skill discovery, governance) not by tool — single playbook MAY reference multiple tools
  - DEC-2435 2026-05-19 — Playbook v1 set is 12; subsequent additions via task-PLUGIN-004a/b/c successor tasks gated on usage data

language: markdown
service: modules/plugin/skills/
new_files:
  - modules/plugin/skills/run-cuo-workflow/SKILL.md
  - modules/plugin/skills/run-cuo-workflow/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/route-natural-language/SKILL.md
  - modules/plugin/skills/route-natural-language/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/audit-trail-query/SKILL.md
  - modules/plugin/skills/audit-trail-query/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/audit-trail-append/SKILL.md
  - modules/plugin/skills/audit-trail-append/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/discover-skills/SKILL.md
  - modules/plugin/skills/discover-skills/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/invoke-cyberos-skill/SKILL.md
  - modules/plugin/skills/invoke-cyberos-skill/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/persona-discovery/SKILL.md
  - modules/plugin/skills/persona-discovery/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/workflow-inspection/SKILL.md
  - modules/plugin/skills/workflow-inspection/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/cross-workflow-chain/SKILL.md
  - modules/plugin/skills/cross-workflow-chain/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/memory-write-discipline/SKILL.md
  - modules/plugin/skills/memory-write-discipline/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/auth-and-scopes/SKILL.md
  - modules/plugin/skills/auth-and-scopes/acceptance/TRIGGER_TESTS.md
  - modules/plugin/skills/audit-emission-discipline/SKILL.md
  - modules/plugin/skills/audit-emission-discipline/acceptance/TRIGGER_TESTS.md
  - modules/plugin/tests/test_playbooks_conform_to_skb.py
  - modules/plugin/tests/test_playbooks_have_trigger_tests.py
  - modules/plugin/tests/test_playbooks_reference_valid_tools.py

modified_files:
  - modules/plugin/manifests/cyberos@1.0.0.plugin.json (skills array)
  - website docs (Plugin page)

allowed_tools:
  - file_read: modules/plugin/skills/**
  - file_write: modules/plugin/skills/**
  - bash: python -m pytest modules/plugin/tests/test_playbooks_*.py

disallowed_tools:
  - bypass TASK-SKILL-111 description format (per DEC-2432)
  - skip TRIGGER_TESTS (per DEC-2433)
  - extend playbook set without task (per DEC-2435)

effort_hours: 6
subtasks:
  - "0.4h: README.md scaffold for skills/ folder"
  - "3.0h: 12 SKILL.md files (15min each)"
  - "1.5h: 12 TRIGGER_TESTS.md files"
  - "1.1h: 3 conformance tests"

risk_if_skipped: "Without skill playbooks, hosts know HOW to call the tools (manifest tells them) but don't know WHEN to use which tool — leading to mis-routing, misuse of destructive tools, and unnecessary calls to expensive workflows. Adoption metric falls because plugin appears 'powerful but unusable'. Without DEC-2432 SKB conformance, playbooks don't trigger correctly in description-match routers (Claude Code's skill router). Without DEC-2433 TRIGGER_TESTS, the description-routing accuracy is invisible to CI and degrades silently over time."
---

## §1 — Description (BCP-14 normative)

The PLUGIN module **MUST** ship 12 skill playbooks at `modules/plugin/skills/<name>/SKILL.md` following the Anthropic Agent Skills spec. Each playbook is a markdown file with YAML frontmatter teaching hosts WHEN and HOW to chain the MCP tools exposed in TASK-PLUGIN-002. Playbooks do NOT add new MCP tools — they are documentation that the host's skill router uses to inject just-in-time discipline into the model's prompt.

1. **MUST** ship exactly 12 playbooks in v1 per DEC-2430 + DEC-2435, grouped by use-case:
- **Orchestration (3)**: `run-cuo-workflow`, `route-natural-language`, `cross-workflow-chain`
- **Memory (3)**: `audit-trail-query`, `audit-trail-append`, `memory-write-discipline`
- **Discovery (3)**: `discover-skills`, `persona-discovery`, `workflow-inspection`
- **Governance (3)**: `invoke-cyberos-skill`, `auth-and-scopes`, `audit-emission-discipline`

2. **MUST** conform to SKILL_BUNDLE_RUBRIC SKB-020..023 per DEC-2432 + TASK-SKILL-111/113:
- SKB-020: description 60-480 chars
- SKB-021: description contains ≥4 quoted trigger examples
- SKB-022: description has no XML/HTML tags
- SKB-023: description verb stems are recognised (router fingerprint)

3. **MUST** carry `acceptance/TRIGGER_TESTS.md` per DEC-2433 + TASK-SKILL-112 with at least:
- 4 positive fixtures (user-input strings that SHOULD trigger this playbook)
- 4 negative fixtures (strings that should NOT trigger it, especially adjacent-skill confusables)

4. **MUST** reference only tools registered in TASK-PLUGIN-002 — validator test `test_playbooks_reference_valid_tools.py` checks every tool name mentioned in the playbook body against the 8-tool registry.

5. **MUST** explain in body sections: WHEN to use the playbook (one paragraph), WHICH tools it chains (bulleted), WHAT scopes are required (list), WHAT side effects occur (list), and a worked example (code block showing 2-4 sample tool calls).

6. **MUST** declare the playbook frontmatter shape per Anthropic Agent Skills spec:
   ```yaml
   ---
   name: <skill-id>                       # kebab-case, matches folder name
   description: >
     One paragraph (60-480 chars) describing what this skill is for AND 4 quoted trigger examples
     e.g. "use this when the user asks 'run a workflow', 'execute a CUO chain', 'kick off the architect-new-system workflow', or 'start a persona workflow'"
   license: Apache-2.0
   ---
   ```

7. **MUST** pass TASK-SKILL-114 baseline check — once SKILL.md hits v1.0 maturity, an `acceptance/BASELINE.md` MUST be created freezing the trigger-test pass rate so regressions are detectable.

8. **MUST** be host-portable per DEC-2434 — same SKILL.md works in Claude Code, Cowork, Codex CLI without modification. Cursor doesn't render Skills (its MCP integration surfaces tools only); the Cursor adapter (TASK-PLUGIN-007) omits skills/.

9. **MUST** be subject to lazy-load discipline — host routers load only the SKILL.md description, not the body, at fingerprinting time. Body content is fetched only when the skill is triggered. Authors MUST treat the description as the load-bearing copy.

10. **MUST NOT** add new playbooks in v1.x.y without a successor task (task-PLUGIN-004a, etc.) per DEC-2435.

11. **MUST NOT** rename a playbook within v1.x.y — rename = breaking change requiring major bump (mirrors TASK-PLUGIN-003 clause 11).

12. **MUST NOT** declare destructive operations in a playbook body without flagging in the description's trigger examples — users routed to the playbook MUST see the side effects upfront.

---

## §2 — Why this design

**Why playbooks AND commands AND tools (clauses 1+)?** Three layers of host-facing surface:
- **Tools** (TASK-PLUGIN-002) — *what* the plugin can do; called by the host's tool-calling subsystem.
- **Commands** (TASK-PLUGIN-003) — *how the user invokes* it explicitly; a UI affordance.
- **Playbooks** (this task) — *when and why* the host model should pick a tool; injected into the model's prompt when the description matches the user's input.

Without playbooks, the host model sees tool names + brief descriptions but has no discipline to follow. Playbooks add the "use this tool ONLY when X" prose that prevents misuse.

**Why exactly 12 playbooks (DEC-2430)?** Same logic as TASK-PLUGIN-003 — small, learnable, well-considered. 12 covers the major use-cases with room to teach edge-cases (memory-write-discipline, audit-emission-discipline). More playbooks dilute the router's discrimination.

**Why conform to SKB-020..023 (DEC-2432)?** Plugin playbooks SHIP THROUGH the same Anthropic Agent Skills surface as everything else in `modules/skill/`. The SKB rubric is how all skills get discovered correctly. Playbooks must clear the same bar.

**Why TRIGGER_TESTS (DEC-2433)?** TASK-SKILL-112 introduced TRIGGER_TESTS.md as the way to measure routing accuracy. Without it, a description that triggers spuriously (false positive) or fails to trigger (false negative) goes unnoticed. With TRIGGER_TESTS, CI catches the regression.

**Why reference only registered tools (clause 4)?** Hallucinated tool names in playbook bodies waste the model's call attempts and produce confusing error envelopes. Validator test catches the divergence at PR time.

**Why required body sections (clause 5)?** Same lazy-load discipline as TASK-PLUGIN-003 commands — the body teaches the model when triggered. Without explicit "scopes required" and "side effects" sections, the model uses the playbook without knowing the consequences.

**Why mandatory BASELINE.md at v1.0 (clause 7)?** TASK-SKILL-114 establishes this as the regression-prevention mechanism. Playbook descriptions drift as authors tune triggers; without a baseline pass-rate snapshot, the team can't tell if a tweak improved routing or broke it.

**Why omit skills from Cursor adapter (clause 8)?** Cursor's MCP integration surfaces tools to the model directly. Skills (Anthropic Agent Skills) are an Anthropic-spec concept not implemented by Cursor as of 2026. Shipping them adds bloat without benefit. The canonical skills still ship in the canonical bundle for hosts that do support.

**Why lazy-load discipline (clause 9)?** Anthropic Skills router loads descriptions into a fingerprint index at install; bodies are fetched on match. If author treats body as load-bearing for routing, the playbook never triggers. Description is the only path to triggering.

**Why no playbook renames in v1.x.y (clause 11)?** Same reason as command renames: user scripts and host caches reference by slug; renames break references silently.

**Why flag destructive ops in description (clause 12)?** Users routed by description never see the body before the model decides to act. If "delete some memory rows" is in the body but not the description, the model may invoke without the user's expected warning. Surfacing in description triggers gives the user a glance-able heads-up.

---

## §3 — API contract

### Playbook folder layout (per playbook)

```
modules/plugin/skills/<name>/
├── SKILL.md                       (frontmatter + body)
└── acceptance/
    ├── TRIGGER_TESTS.md           (4 positive + 4 negative fixtures)
    └── BASELINE.md                (frozen pass rate at v1.0 promotion — added by TASK-SKILL-114)
```

### Sample playbook — `run-cuo-workflow/SKILL.md`

```markdown
---
name: run-cuo-workflow
description: >
  Use this skill when the user wants to execute a CyberOS persona-aware workflow chain end-to-end.
  Triggers on user requests like "run a workflow", "execute the architect-new-system workflow",
  "kick off CUO for the CTO", or "run the ADR quick-capture flow". Routes the model to call
  cyberos.cuo.execute_workflow with the right persona+workflow slugs and surfaces task status
  back to the user as the long-running execution proceeds.
license: Apache-2.0
---

## When to use

User wants to execute a CyberOS workflow they (or you) have identified. The workflow is a multi-step
chain belonging to one of the 47 active personas. Workflows emit memory audit rows at every step.

If the user only knows the goal ("architect a new payment system") and not the workflow name,
use `route-natural-language` FIRST to translate the natural-language ask into a persona+workflow
pair, then come here.

## Tools chained

- `cyberos.cuo.execute_workflow` — primary call; returns task_id, status becomes "running"
- `tasks/get` — poll for status (called by host's task subsystem, not directly by you)

## Scopes required

- `cyberos:cuo:execute`
- `cyberos:memory:write` (for audit emission)

## Side effects

- Spawns one Task per invocation (long-running, 30 seconds – several minutes typical)
- Emits 1× `plugin.invoked` audit row at task start
- Emits N× `cuo.step_completed` audit rows during execution
- Emits 1× `cuo.workflow_completed` audit row at finish

## Worked example

```text
User: Run the ADR quick-capture workflow for the CTO with title "Adopt PostgreSQL 16" You: { tool: cyberos.cuo.execute_workflow,
       args: { persona: "chief-technology-officer",
               workflow: "adr-quick-capture",
               inputs: { title: "Adopt PostgreSQL 16" } } }
Host: { task_id: "t-abc123", status: "running" } [host polls via tasks/get; you tell user "started, polling..."] Host: { status: "completed", output: { adr_number: "ADR-2402", artifact_uri: "..." } } You: "ADR-2402 published: Adopt PostgreSQL 16. Took 4.2s."
```
```

### Sample `acceptance/TRIGGER_TESTS.md`

```markdown
# Trigger tests — run-cuo-workflow

## Positive (should trigger)

- "Run a CUO workflow"
- "Execute the architect-new-system workflow for the CTO"
- "Kick off ADR quick-capture for adopting PostgreSQL 16"
- "Start a persona workflow"

## Negative (should NOT trigger)

- "Find the right workflow for me" → routes to route-natural-language
- "Show me which workflows exist" → routes to workflow-inspection
- "What did the workflow do last time?" → routes to audit-trail-query
- "Cancel the running workflow" → not a separate skill; this is a tasks/cancel call from host
```

---

## §4 — Acceptance criteria

1. **Exactly 12 playbook folders exist** — `ls -d modules/plugin/skills/*/ | wc -l` → 12.
2. **Every folder has SKILL.md and acceptance/TRIGGER_TESTS.md** — test asserts presence.
3. **Every SKILL.md has frontmatter with name + description + license** — test loads YAML.
4. **Every description is 60-480 chars (SKB-020)** — test asserts length.
5. **Every description has ≥4 quoted trigger examples (SKB-021)** — test counts quotes.
6. **No description contains XML/HTML tags (SKB-022)** — regex `<[a-zA-Z]+` MUST NOT match.
7. **Every description's verb stems are recognised (SKB-023)** — test runs against VERB_STEMS allowlist from TASK-SKILL-115 tooling.
8. **Every TRIGGER_TESTS.md has 4+ positive + 4+ negative fixtures** — test parses lists.
9. **Every tool name referenced in body matches TASK-PLUGIN-002 registry** — test greps `cyberos.<...>` patterns and validates.
10. **Body has "When to use" section** — test grep.
11. **Body has "Tools chained" section** — test grep.
12. **Body has "Scopes required" section** — test grep.
13. **Body has "Side effects" section** — test grep.
14. **Body has "Worked example" section with a code block** — test grep + code-fence check.
15. **3 orchestration playbooks present** — explicit check of folder names.
16. **3 memory playbooks present** — explicit check.
17. **3 discovery playbooks present** — explicit check.
18. **3 governance playbooks present** — explicit check.
19. **Destructive playbook (audit-trail-append) flags side effect in description** — test parses description, asserts "write" or "append" phrase.
20. **Manifest skills[] array references all 12** — manifest validator check.

---

## §5 — Verification

```python
# modules/plugin/tests/test_playbooks_conform_to_skb.py
import re, yaml
from pathlib import Path

SKILLS = Path(__file__).parent.parent / "skills"

def test_exactly_twelve_playbooks():
    assert sum(1 for _ in SKILLS.iterdir() if _.is_dir()) == 12

def test_description_skb_020_023():
    for skill_dir in SKILLS.iterdir():
        fm = load_frontmatter(skill_dir / "SKILL.md")
        desc = fm["description"]
        assert 60 <= len(desc) <= 480, f"{skill_dir.name}: SKB-020 length"
        quote_count = desc.count('"')
        assert quote_count >= 8, f"{skill_dir.name}: SKB-021 needs ≥4 quoted examples (saw {quote_count // 2})"
        assert not re.search(r"<[a-zA-Z]+", desc), f"{skill_dir.name}: SKB-022 XML tag"
```

```python
# modules/plugin/tests/test_playbooks_have_trigger_tests.py
def test_trigger_tests_present_and_balanced():
    for skill_dir in SKILLS.iterdir():
        tt = skill_dir / "acceptance" / "TRIGGER_TESTS.md"
        assert tt.exists(), f"{skill_dir.name}: missing TRIGGER_TESTS.md"
        text = tt.read_text()
        pos_section = text.split("## Positive")[1].split("## Negative")[0]
        neg_section = text.split("## Negative")[1]
        pos_count = sum(1 for line in pos_section.splitlines() if line.startswith("- "))
        neg_count = sum(1 for line in neg_section.splitlines() if line.startswith("- "))
        assert pos_count >= 4 and neg_count >= 4, \
            f"{skill_dir.name}: need ≥4 positive + ≥4 negative (got {pos_count}/{neg_count})"
```

```python
# modules/plugin/tests/test_playbooks_reference_valid_tools.py
REGISTERED = {
    "cyberos.cuo.list_personas", "cyberos.cuo.list_workflows",
    "cyberos.cuo.route", "cyberos.cuo.execute_workflow",
    "cyberos.memory.read_audit", "cyberos.memory.append_audit",
    "cyberos.skill.list_catalog", "cyberos.skill.invoke_skill",
}

def test_all_referenced_tools_exist():
    pattern = re.compile(r"\bcyberos\.[a-z][a-z0-9]*\.[a-z][a-z0-9_]*\b")
    for skill_dir in SKILLS.iterdir():
        body = (skill_dir / "SKILL.md").read_text()
        for match in pattern.findall(body):
            assert match in REGISTERED, \
                f"{skill_dir.name}: tool '{match}' not in TASK-PLUGIN-002 registry"
```

---

## §6 — Implementation skeleton

Each of 12 playbooks follows the pattern in §3 sample. Author writes:
- SKILL.md with frontmatter + 5 body sections (When to use / Tools chained / Scopes required / Side effects / Worked example)
- acceptance/TRIGGER_TESTS.md with 4 positive + 4 negative fixtures
- (later, at v1.0 promotion per TASK-SKILL-114) acceptance/BASELINE.md

Three validator tests enforce conformance.

---

## §7 — Dependencies

- **Upstream:** TASK-PLUGIN-002 (provides the 8 tools referenced); TASK-SKILL-111 (description enrichment discipline applied here); TASK-SKILL-113 (XML-free frontmatter — SKB-022).
- **Downstream:** TASK-PLUGIN-007 (multi-runtime adapters copy `skills/` into per-target bundle for hosts that support Skills).
- **Cross-module:** TASK-SKILL-112 (TRIGGER_TESTS.md format), TASK-SKILL-114 (BASELINE.md once v1.0), TASK-SKILL-115 (placeholder sweep tooling reused for verb-stem check).

---

## §8 — Example payloads

(See §3 for full sample playbook and TRIGGER_TESTS.md.)

Manifest skills[] array entry:
```json
{
  "id": "run-cuo-workflow",
  "path": "skills/run-cuo-workflow/SKILL.md"
}
```

---

## §9 — Open questions

All resolved.

- ~~Should playbooks be merged into commands (single concept)?~~ → No. Commands are user-invoked UI; playbooks are host-router-injected discipline. Different consumers, different lifecycles. Keep separate.
- ~~Should we ship 6 playbooks now and grow to 12 in v1.1?~~ → No, ship 12 at once per DEC-2430. Half-coverage means routing misses obvious use-cases and adoption stalls.
- ~~Should BASELINE.md ship in v1 or wait for v1.0 promotion?~~ → Wait for v1.0 per clause 7 + TASK-SKILL-114 — baselines on pre-stable skills get bumped repeatedly and the discipline erodes.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Description too short | SKB-020 length check | test fails | Author expands ≥60 |
| Description too long | SKB-020 length check | test fails | Author trims ≤480 |
| Description missing trigger quotes | SKB-021 quote count | test fails | Author adds ≥4 quoted examples |
| Description has XML tag | SKB-022 regex | test fails | Author rewrites without tags |
| Verb stems unrecognised | SKB-023 allowlist | test fails | Author uses canonical verbs or extends VERB_STEMS via TASK-SKILL-115 |
| Missing TRIGGER_TESTS.md | filesystem check | test fails | Author writes the file |
| TRIGGER_TESTS imbalanced (<4 pos/neg) | line count | test fails | Author adds fixtures |
| Tool name typo in body | regex + registry check | test fails | Author corrects to SEP-986 name |
| Body missing required section | grep check | test fails | Author adds section |
| Playbook count != 12 | folder count | test fails | Author adds/removes per task-PLUGIN-004a (additions need task) |
| Two playbooks with same name | filesystem | install fails | inherent |
| Playbook rename within v1 | git diff | manual review | Revert or bump major |
| Worked example references missing tool | regex + registry | test fails | Author fixes |
| Description has unbalanced quotes | YAML parse fails | YAML loader raises | Author closes quotes |
| Side-effect missing from description for destructive playbook | manual review | reviewer flags | Author adds phrase like "appends to memory" |

---

## §11 — Implementation notes

- §11.1 **Why 12 not 10 or 14.** 12 = 3 use-cases × 4 playbooks each, evenly balanced. 10 leaves discovery undercovered; 14 dilutes router discrimination. Sweet spot from TASK-SKILL-111 calibration data.

- §11.2 **VERB_STEMS reuse.** TASK-SKILL-115 shipped a verb-stem allowlist (`tools/sweep-placeholders/verb_stems.py`). The test imports it directly. Adding new verbs requires the TASK-SKILL-115 extension procedure.

- §11.3 **Why description carries trigger quotes inline.** Anthropic Skills router fingerprints on the description alone — body is loaded later. Quoted phrases inside the description directly contribute to router accuracy. Conventional "see TRIGGER_TESTS.md for examples" approach fails the router.

- §11.4 **Cursor + Codex CLI compatibility.** Cursor doesn't render Skills; the TASK-PLUGIN-007 Cursor adapter strips the skills/ directory from the bundle. Codex CLI reads SKILL.md as-is. Both behaviours are tested in TASK-PLUGIN-007.

- §11.5 **Playbook-vs-command crossover.** /cyberos-run command and run-cuo-workflow playbook overlap. By design: command is the explicit UI affordance; playbook is the model-side injection. Both reference the same underlying tool. Authors keep them consistent (description phrasing, scope list) but don't merge.

- §11.6 **Triggering one playbook may suggest another.** "Worked example" sections often suggest a follow-up playbook ("use route-natural-language first"). This is intentional — cross-playbook chaining is good prompt engineering.

- §11.7 **Manifest skills[] is the source of distribution.** Even though the playbooks live in `modules/plugin/skills/` on disk, the manifest's `skills[]` array is what the packer reads. Adding a playbook without updating the manifest = playbook ships but isn't advertised.

- §11.8 **Avoid playbook author drift.** Annual review: re-check every playbook's description against shipped TASK-PLUGIN-002 tool list. Add a CI annotation when a new tool lands without a playbook covering it.

---

*End of TASK-PLUGIN-004 spec.*
