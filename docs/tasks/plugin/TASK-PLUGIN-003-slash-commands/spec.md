---
id: TASK-PLUGIN-003
title: "Canonical slash-commands — /cyberos-run, /cyberos-memory, /cyberos-skill-list, /cyberos-route markdown definitions in modules/plugin/commands/"
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
new_files:
  - modules/plugin/commands/SCHEMA.md
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PLUGIN-001, TASK-PLUGIN-002, TASK-PLUGIN-004]
depends_on: [TASK-PLUGIN-001, TASK-PLUGIN-002]
blocks: []

source_pages:
  - modules/plugin/README.md §3 (commands/ folder)

source_decisions:
  - DEC-2420 2026-05-19 — Slash commands are markdown files at modules/plugin/commands/<name>.md with YAML frontmatter declaring shape + tool bindings
  - DEC-2421 2026-05-19 — Exactly 4 commands ship in v1 — /cyberos-run, /cyberos-memory, /cyberos-skill-list, /cyberos-route; new commands need a successor task
  - DEC-2422 2026-05-19 — Command files MUST be host-portable — Claude Code + Cowork render natively; Cursor + Codex CLI use the SKILL.md compat layer in TASK-PLUGIN-007
  - DEC-2423 2026-05-19 — Command argument schema MUST mirror the underlying MCP tool's input_schema — single source of truth per TASK-PLUGIN-001
  - DEC-2424 2026-05-19 — Commands MUST include before-use trigger discipline per TASK-SKILL-111 — description (60-480 chars) + 4 trigger examples

build_envelope:
  language: markdown
  service: modules/plugin/commands/
  new_files:
    - modules/plugin/commands/cyberos-run.md
    - modules/plugin/commands/cyberos-memory.md
    - modules/plugin/commands/cyberos-skill-list.md
    - modules/plugin/commands/cyberos-route.md
    - modules/plugin/commands/SCHEMA.md
    - modules/plugin/tests/test_commands_have_frontmatter.py
    - modules/plugin/tests/test_commands_bind_to_valid_tools.py
    - modules/plugin/tests/test_commands_description_length.py
    - modules/plugin/tests/test_commands_trigger_count.py

  modified_files:
    - modules/plugin/manifests/cyberos@1.0.0.plugin.json (commands array)
    - website docs (Plugin page §3)

  allowed_tools:
    - file_read: modules/plugin/commands/**
    - file_write: modules/plugin/commands/**
    - bash: python -m pytest modules/plugin/tests/test_commands_*.py

  disallowed_tools:
    - inline tool input schemas (per DEC-2423 — single source of truth in manifest)
    - exceed 4 commands in v1 (per DEC-2421 — successor task required)

effort_hours: 4
subtasks:
  - "0.5h: SCHEMA.md — frontmatter contract for command files"
  - "0.6h: cyberos-run.md"
  - "0.6h: cyberos-memory.md"
  - "0.6h: cyberos-skill-list.md"
  - "0.6h: cyberos-route.md"
  - "1.1h: 4 validator tests"

risk_if_skipped: "Without slash commands, Claude Code + Cowork users have no inline UI affordance — they must manually request MCP tool calls every time. Adoption stalls because the plugin appears 'invisible' inside the host UI. Without DEC-2423 single-source schema discipline, command arguments drift from tool inputs and users get cryptic 'unknown field' errors. Without DEC-2424 trigger discipline, hosts that route by description match (Claude Code's skill router) misroute or skip the commands entirely."
---

## §1 — Description (BCP-14 normative)

The PLUGIN module **MUST** ship 4 canonical slash-commands at `modules/plugin/commands/<name>.md`. Each command is a markdown file with YAML frontmatter binding the command to one or more MCP tools exposed by TASK-PLUGIN-002.

1. **MUST** ship exactly 4 commands in v1 per DEC-2421:
   - `/cyberos-run <persona> <workflow>` — execute a CUO workflow chain (calls `cyberos.cuo.execute_workflow`)
   - `/cyberos-memory <read|append> ...` — read or append memory audit rows (calls `cyberos.memory.read_audit` or `cyberos.memory.append_audit`)
   - `/cyberos-skill-list [filter]` — list available skills from catalog (calls `cyberos.skill.list_catalog`)
   - `/cyberos-route <query>` — natural-language routing to persona+workflow (calls `cyberos.cuo.route`)

2. **MUST** use the frontmatter schema documented at `modules/plugin/commands/SCHEMA.md` (clause 6 below for the schema itself).

3. **MUST** mirror the underlying MCP tool's `input_schema` exactly per DEC-2423. A command argument list is derived from the tool's `properties` object; required-argument list is derived from the tool's `required` array. Authors MUST NOT inline alternative schemas.

4. **MUST** declare 4 example trigger phrases per command per TASK-SKILL-111 + DEC-2424. Phrases live in the frontmatter `triggers:` array. Hosts that route by description match (Claude Code's skill router) use these to disambiguate.

5. **MUST** include a `description:` field of 60-480 characters per TASK-SKILL-111. This is the host-rendered command summary. The 60-char floor forces meaningful copy; the 480-char ceiling forces conciseness.

6. **MUST** carry frontmatter conforming to this YAML shape:
   ```yaml
   ---
   name: <slash-command-name>           # without leading /, kebab-case
   description: <60-480 char summary>
   binds_to:
     - tool: cyberos.<module>.<verb>_<noun>   # SEP-986 from TASK-PLUGIN-001
       when: <natural-language disambiguator OR "always">
   arguments:
     - name: <arg-name>
       description: <human-friendly>
       required: <true|false>
       type: <string|number|boolean|object|array>
   triggers:
     - "<example user input that would invoke this command>"
     - "<another example>"
     - "<third example>"
     - "<fourth example, including edge-case phrasing>"
   destructive: <true|false>            # if any bound tool is destructive (per TASK-MCP-006)
   ---
   ```

7. **MUST** validate via `modules/plugin/tests/test_commands_*.py` that:
   - Every command file parses as YAML frontmatter + markdown body
   - Every `binds_to[*].tool` matches a tool name registered in TASK-PLUGIN-002's static registry
   - Every command's `arguments[]` is a subset (by name + type) of the bound tool's input_schema
   - Every command has `len(description) ∈ [60, 480]`
   - Every command has `len(triggers) == 4`

8. **MUST** include a body section per command that explains: when to invoke, what scopes are required, what side-effects occur, and a worked example. Body is rendered in the host's command-detail view (Claude Code's `?` button on a command).

9. **MUST NOT** declare a tool binding to a non-existent tool (validated by test_commands_bind_to_valid_tools).

10. **MUST NOT** add new commands without a successor task (task-PLUGIN-003a, etc.) per DEC-2421.

11. **MUST NOT** rename a command between v1.x.y releases — rename = breaking change, requires major bump.

---

## §2 — Why this design

**Why markdown + frontmatter (clause 2)?** Markdown commands are the standard format Claude Code expects (per Anthropic Agent Skills spec) and Cowork rendering uses. YAML frontmatter gives structured metadata (bindings, triggers) while keeping the body rich for human readers.

**Why exactly 4 commands in v1 (DEC-2421)?** Each command is a learnable UI primitive — too many and users can't remember what's available. The 4 cover orchestration (run + route), memory (memory read/append), and discovery (skill-list). Future commands land via successor tasks after usage data.

**Why mirror tool input_schema (DEC-2423, clause 3)?** Two sources of truth for the same shape drift. The manifest schema (TASK-PLUGIN-001) is canonical. Commands MUST derive from it.

**Why 4 trigger phrases (DEC-2424, clause 4)?** TASK-SKILL-111 calibrated 4 as the minimum count that gives the description-match router enough fingerprint to disambiguate. Two phrases is too noisy; eight is overkill for a slash command that the user already typed.

**Why 60-480 char description (clause 5)?** Same as TASK-SKILL-111 description-enrichment range. Below 60 = unhelpful blurb; above 480 = wall-of-text in cramped UI panels.

**Why destructive flag in frontmatter (clause 6)?** Hosts surface destructive-command warnings differently. The flag is the host's hook to render that UI. Without it, every command looks the same and users invoke destructive ops without warning.

**Why body section with worked example (clause 8)?** Slash commands are user-facing. The body teaches the user how to use the command — required scopes, expected behaviour, sample output. Without the body, hosts surface a stub that says "no description provided" and adoption stalls.

**Why no command renames within v1.x.y (clause 11)?** Renames break user muscle memory and any scripts that invoke commands by name. Rename = major version bump.

---

## §3 — API contract

### `commands/SCHEMA.md` (excerpt)

```yaml
# CyberOS slash-command frontmatter — v1 contract
name: string                          # required, kebab-case, no leading /
description: string                   # required, 60-480 chars
binds_to:                             # required, ≥1 entry
  - tool: string                      # required, SEP-986 pattern
    when: string                      # optional; "always" if omitted
arguments:                            # optional; subset of bound tool's input_schema
  - name: string
    description: string
    required: boolean
    type: enum[string,number,boolean,object,array]
triggers:                             # required, exactly 4 entries
  - string
destructive: boolean                  # required; true if any bound tool has destructive: true
```

### `commands/cyberos-run.md` (excerpt)

```markdown
---
name: cyberos-run
description: Execute a CyberOS workflow chain for a chosen persona. Used when the user wants to run a structured multi-step process (e.g. architect a system, prepare an investor update) and benefit from CUO's persona-aware orchestration with memory audit emission.
binds_to:
  - tool: cyberos.cuo.execute_workflow
    when: always
arguments:
  - name: persona
    description: Persona slug (e.g. chief-technology-officer)
    required: true
    type: string
  - name: workflow
    description: Workflow slug (e.g. architect-new-system)
    required: true
    type: string
  - name: inputs
    description: Workflow-specific input parameters as JSON
    required: false
    type: object
triggers:
  - "/cyberos-run chief-technology-officer architect-new-system"
  - "Run the architect-new-system workflow for the CTO"
  - "Execute CUO workflow"
  - "Kick off the ADR-quick-capture flow"
destructive: false
---

## When to use

Use this command when you have a CUO-defined workflow ready to execute end-to-end. Workflows are
multi-step chains that run through the CyberOS supervisor and emit memory audit rows for every step.

## Required scopes

- `cyberos:cuo:execute`
- `cyberos:memory:write` (for audit emission)

## Side effects

- Spawns one Task per invocation (long-running, async)
- Emits 1× `plugin.invoked` audit row at task start
- Emits N× `cuo.step_completed` audit rows during execution
- Emits 1× `cuo.workflow_completed` audit row at finish

## Example

```text
You: /cyberos-run chief-technology-officer adr-quick-capture
Plugin: Started task t-abc123. Polling status...
Plugin: Step 1/3 complete — issue_authored
Plugin: Step 2/3 complete — adr_drafted
Plugin: Step 3/3 complete — adr_published
Plugin: Workflow completed in 4.2s. ADR-2402 published.
```
```

### Frontmatter validators (Python pseudocode)

```python
def validate_command_file(path: Path) -> List[ValidationError]:
    fm, body = parse_frontmatter(path.read_text())
    errors = []
    if not 60 <= len(fm["description"]) <= 480:
        errors.append(("description", "length out of [60,480]"))
    if len(fm["triggers"]) != 4:
        errors.append(("triggers", f"expected 4, got {len(fm['triggers'])}"))
    for bind in fm["binds_to"]:
        if bind["tool"] not in REGISTERED_TOOL_NAMES:
            errors.append(("binds_to", f"tool '{bind['tool']}' not in TASK-PLUGIN-002 registry"))
    return errors
```

---

## §4 — Acceptance criteria

1. **Exactly 4 command files exist** — `ls modules/plugin/commands/cyberos-*.md | wc -l` → 4.
2. **Every command has parseable YAML frontmatter** — test parses each file.
3. **Every command's description is 60-480 chars** — test asserts length.
4. **Every command has exactly 4 triggers** — test asserts `len(triggers) == 4`.
5. **Every binds_to.tool exists in TASK-PLUGIN-002 registry** — test loads tool list, checks each bind.
6. **Every argument name + type is a subset of bound tool's input_schema** — test loads manifest, compares.
7. **/cyberos-run binds to cyberos.cuo.execute_workflow** — explicit fixture check.
8. **/cyberos-memory binds to cyberos.memory.read_audit AND cyberos.memory.append_audit** — explicit fixture check (2 bindings with `when` disambiguators).
9. **/cyberos-skill-list binds to cyberos.skill.list_catalog** — explicit fixture check.
10. **/cyberos-route binds to cyberos.cuo.route** — explicit fixture check.
11. **/cyberos-memory append variant has destructive: true** — test asserts.
12. **Other 3 commands have destructive: false** — test asserts.
13. **Frontmatter validator rejects missing description** — fixture file fails validation.
14. **Frontmatter validator rejects 3 triggers** — fixture file fails validation.
15. **Frontmatter validator rejects unknown tool binding** — fixture with `tool: foo.bar.baz` fails.
16. **Body section has worked example** — test grep-asserts `## Example` appears in every command body.
17. **Body section lists required scopes** — test grep-asserts `## Required scopes` appears in every command body.

---

## §5 — Verification

```python
# modules/plugin/tests/test_commands_have_frontmatter.py
from pathlib import Path
import yaml

COMMANDS_DIR = Path(__file__).parent.parent / "commands"

def test_exactly_four_commands():
    files = sorted(COMMANDS_DIR.glob("cyberos-*.md"))
    assert len(files) == 4, f"expected 4, got {len(files)}: {[f.name for f in files]}"

def test_each_has_yaml_frontmatter():
    for f in COMMANDS_DIR.glob("cyberos-*.md"):
        raw = f.read_text()
        assert raw.startswith("---\n"), f"{f.name} missing frontmatter"
        fm_end = raw.find("\n---\n", 4)
        fm = yaml.safe_load(raw[4:fm_end])
        assert "name" in fm and "description" in fm and "binds_to" in fm
```

```python
# modules/plugin/tests/test_commands_bind_to_valid_tools.py
REGISTERED = {
    "cyberos.cuo.list_personas", "cyberos.cuo.list_workflows",
    "cyberos.cuo.route", "cyberos.cuo.execute_workflow",
    "cyberos.memory.read_audit", "cyberos.memory.append_audit",
    "cyberos.skill.list_catalog", "cyberos.skill.invoke_skill",
}

def test_all_bindings_exist():
    for f in COMMANDS_DIR.glob("cyberos-*.md"):
        fm = load_frontmatter(f)
        for bind in fm["binds_to"]:
            assert bind["tool"] in REGISTERED, \
                f"{f.name}: tool '{bind['tool']}' not in TASK-PLUGIN-002 registry"
```

```python
# modules/plugin/tests/test_commands_description_length.py
def test_description_length_in_range():
    for f in COMMANDS_DIR.glob("cyberos-*.md"):
        fm = load_frontmatter(f)
        n = len(fm["description"])
        assert 60 <= n <= 480, f"{f.name}: description length {n} not in [60,480]"
```

```python
# modules/plugin/tests/test_commands_trigger_count.py
def test_each_has_four_triggers():
    for f in COMMANDS_DIR.glob("cyberos-*.md"):
        fm = load_frontmatter(f)
        assert len(fm["triggers"]) == 4, \
            f"{f.name}: expected 4 triggers, got {len(fm['triggers'])}"
        for t in fm["triggers"]:
            assert isinstance(t, str) and len(t) > 0
```

---

## §6 — Implementation skeleton

Markdown commands are content, not code. The skeleton is: write each of 4 files following the SCHEMA.md frontmatter contract; write 4 validator tests; wire commands into the manifest's `commands[]` array.

---

## §7 — Dependencies

- **Upstream:** TASK-PLUGIN-001 (manifest schema declares `commands` array shape), TASK-PLUGIN-002 (provides the bound tools).
- **Downstream:** TASK-PLUGIN-007 (adapters export commands to per-runtime format — Claude Code reads `.md` directly; Cursor doesn't render commands but ignores them).
- **Cross-module:** TASK-SKILL-111 (description enrichment discipline, shipped), TASK-SKILL-112 (TRIGGER_TESTS.md pattern — the `triggers:` array follows the same shape); TASK-MCP-006 (destructive flag mirrors tool annotation).

---

## §8 — Example payloads

(See §3 for full /cyberos-run example.)

Manifest `commands` array entry:
```json
{
  "name": "/cyberos-run",
  "file": "commands/cyberos-run.md",
  "description": "Execute a CyberOS workflow chain for a chosen persona."
}
```

---

## §9 — Open questions

All resolved.

- ~~Should commands support typed argument completion?~~ → Yes, via `arguments[].type` per clause 6. Hosts that support tab-complete (Claude Code) use the type hint.
- ~~Should /cyberos-memory be one command or split into /cyberos-memory-read and /cyberos-memory-append?~~ → One command with mode prefix per clause 1. Symmetric with how /cyberos-route handles both reads and writes via inputs.
- ~~Should we add /cyberos-doctor for the bundle health check?~~ → No, TASK-PLUGIN-001 ships `cyberos-plugin doctor` as a CLI; not surfaced as a slash command. Successor task may add if user demand emerges.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Command file missing frontmatter | parser raises | validator test fails | Author adds `---` fences |
| Frontmatter YAML invalid | yaml.safe_load raises | validator test fails | Author fixes YAML |
| Description too short | length check | validator test fails | Author expands to ≥60 chars |
| Description too long | length check | validator test fails | Author trims to ≤480 chars |
| Triggers count != 4 | length check | validator test fails | Author adds/removes triggers |
| Tool binding doesn't exist | set membership | validator test fails | Author fixes tool name OR ships task-PLUGIN-002a to add the tool |
| Argument type mismatch | type check vs input_schema | validator test fails | Author fixes argument type |
| Destructive flag missing on append-memory command | explicit fixture assertion | validator test fails | Author adds `destructive: true` |
| New command added without task | manifest commands[] grows | validator test fails (5 instead of 4) | Author rolls back OR ships task-PLUGIN-003a |
| Command rename within v1 | git diff on filename | manual review caught at PR | Revert or bump to v2 |
| Host renders empty command body | manual smoke test | UX bug surfaces in install | Author adds body |
| Body missing worked example | grep test for `## Example` | validator test fails | Author adds Example section |
| Two commands with same name | filesystem uniqueness | install fails | inherent — filenames unique |
| Trigger phrase duplicate across commands | not detected (acceptable) | host router may misroute | Authors choose distinct phrasings |

---

## §11 — Implementation notes

- §11.1 **Why no slash-command schema in `manifest.schema.json` is needed.** The manifest just lists `{name, file, description}` per command. The full frontmatter contract lives in the markdown file itself and is validated by the tests in this task.

- §11.2 **Why `name` in frontmatter doesn't include leading `/`.** Hosts that render commands prepend the `/` themselves. Storing it without the slash keeps the value usable as a slug in URLs and filesystem paths.

- §11.3 **Why `when:` field on `binds_to[*]`.** A command can bind to multiple tools (e.g. /cyberos-memory binds to both read and append). The `when:` natural-language disambiguator helps the host route to the right tool based on the user's argument shape (`when: arguments.mode == "append"`).

- §11.4 **Why triggers in YAML, not in body.** Hosts that match descriptions extract triggers from frontmatter — body content is markdown for humans and isn't parsed for routing. Keeping triggers in frontmatter makes them machine-readable.

- §11.5 **Test fixture for invalid commands.** `modules/plugin/tests/fixtures/invalid_*.md` files exercise each rejection path so the validator suite covers happy + failure cases.

- §11.6 **Cursor doesn't render slash commands.** Cursor's MCP integration surfaces tools but not commands. The TASK-PLUGIN-007 Cursor adapter omits the `commands/` directory from the Cursor bundle. The canonical command files still ship in the canonical bundle for hosts that do render them.

- §11.7 **Claude Code custom command rendering.** Claude Code reads markdown from a `.claude/commands/` directory (relative to project). The Claude Code adapter (TASK-PLUGIN-007) copies `modules/plugin/commands/*.md` to `dist/.claude/commands/` in the packed bundle.

- §11.8 **Future command expansion path.** When usage shows demand for additional commands, ship task-PLUGIN-003a covering each new command at 10/10 with the same validator coverage. Don't accumulate command sprawl.

---

*End of TASK-PLUGIN-003 spec.*
