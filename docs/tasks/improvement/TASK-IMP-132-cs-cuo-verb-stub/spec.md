---
id: TASK-IMP-132
title: Add a `cuo` verb to `cs` as a redirect stub
template: task@1
type: improvement
module: improvement
status: done
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-22T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-130]
blocks: [TASK-IMP-134]
related_tasks: [TASK-IMP-076, TASK-IMP-131]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.9"
owner: Stephen Cheng (CTO)
created: 2026-07-22
memory_chain_hash: null
effort_hours: 2
service: tools/install
new_files:
  - tools/install/tests/test_cli_cuo_verb.sh
modified_files:
  - tools/install/cli/bin/cli.mjs
  - tools/install/help.sh
  - tools/install/docs/index.md
source_pages:
  - "docs/plans/PLAN-cli-module-namespacing-2026-07-22/plan.md §5 Scope item 4 ('Add cuo as a new top-level verb ... as a redirect stub (prints the matching slash command to run inside Claude Code); do not implement standalone execution') and §2 ('modules/cuo/pyproject.toml — cyberos-cuo, persona-aware orchestration ... backs /plan, /create-tasks, /ship-tasks, /improve. Those slash commands are multi-step, LLM-orchestrated skills ... not deterministic scripts')"
  - "tools/install/plugin/commands/ (ls: create-tasks.md, help.md, improve.md, install.md, plan.md, status.md, uninstall.md, version.md — 8 files; ship-tasks is NOT among them)"
  - "tools/install/plugin/skills/ship-tasks/ , .cyberos/cuo/ship-tasks.md , .claude/skills/ship-tasks (ship-tasks ships as a natively-installed SKILL, invocable as `/ship-tasks` per every agent's skills folder, per tools/install/README.md:150 — not a plugin/commands/*.md file like the other seven)"
  - "tools/install/cli/bin/cli.mjs:19-26 (SCRIPTS table — install/uninstall/version/status/help/create are already flat cs verbs, none behind a `cuo` namespace)"
  - "tools/install/README.md:28 ('Plugin slash commands (Claude Code): /install /uninstall /version /status /help plus /ship-tasks and /create-tasks') cross-checked against plan §2's narrower 'backs /plan, /create-tasks, /ship-tasks, /improve' — the CUO-backed subset is plan, create-tasks, ship-tasks, improve specifically, not install/uninstall/version/status/help (those are deterministic shell scripts with their own existing cs verbs already)"
source_decisions:
  - "2026-07-22 Stephen: create-tasks PLAN gate — APPROVE as rendered."
  - "2026-07-22 authoring: cross-referenced plan §2's list of CUO-backed slash commands (plan, create-tasks, ship-tasks, improve) against the plugin's actual commands/ directory listing to confirm which four subcommands `cs cuo <name>` should recognise, since install/uninstall/version/status/help are deterministic shell scripts already exposed as their OWN top-level cs verbs and are not \"cuo\" work at all — scoping `cs cuo` to exactly the four LLM-orchestrated names avoids a stub that appears to also cover ground it does not."
  - "2026-07-22 self-audit revision (score_pre_revision 6/10 -> score_post_revision 10/10): original AC 1 tested only that 'unknown command' was absent for bare `cuo`, which AC 4 (listing behaviour) already subsumed more strongly - removed as redundant and retraced clause 1.1 to the `cs cuo plan` AC, which positively proves recognition. Split clause 1.4 into 1.4 (bare invocation, exit 0, orientation) and 1.4a (unrecognised name, exit 2, a usage mistake) - the original single clause conflated two different exit-code semantics that cli.mjs's own established convention treats differently. AC 5's original test mechanism ('a process-spy wrapper around child_process.spawnSync') doesn't match this repo's actual bash-test-harness convention of tripwire stand-in binaries on PATH - revised to that style, consistent with TASK-IMP-131's test design. AC 6's original doc-check ('contains a word from a small set') was satisfiable by an unrelated match anywhere in the file - tightened to require proximity to the actual `cuo` mention, and added a check that docs/index.md's hardcoded 'eight commands' count is updated rather than left stale once this verb (and TASK-IMP-131's) land. Clause 1.5 originally forbade local-availability probing without explaining why that's a deliberate asymmetry with TASK-IMP-131's memory verb (which DOES probe availability) - added the explanation so a reader comparing the two sibling specs doesn't read it as an unexplained inconsistency."
---

# TASK-IMP-132: Add a `cuo` verb to `cs` as a redirect stub

## Summary

Add `cuo` as a top-level verb in `cs` that recognises the four LLM-orchestrated workflow names (`plan`, `create-tasks`, `ship-tasks`, `improve`) and prints which slash command to run inside an agent session, rather than attempting to execute anything standalone.

## Problem

`docs/plans/PLAN-cli-module-namespacing-2026-07-22/plan.md` §2 identifies `modules/cuo` as backing four multi-step, LLM-orchestrated workflows — `/plan`, `/create-tasks`, `/ship-tasks`, `/improve` — each involving interview, drafting, self-audit, and HITL approval gates. These are not deterministic scripts `cli.mjs` can `spawnSync` and get a meaningful result from outside an agent session; running `cyberos-cuo`'s own Python entry point directly (as README.md:47 shows for local dev, `cyberos-cuo list-personas`) exposes internal persona/workflow machinery, not the guided, gated experience the slash commands provide. Today, `cs` (post TASK-IMP-130) has no `cuo` verb at all — a user who types `cs cuo plan` gets "unknown command," with no signal that the capability exists under a different invocation.

## Proposed Solution

Add a `cuo` entry to `cli.mjs`'s dispatch. `cs cuo <name>` where `<name>` is one of `plan`, `create-tasks`, `ship-tasks`, `improve` prints a short message naming the matching slash command (`/plan`, `/create-tasks`, `/ship-tasks`, `/improve` respectively) to run inside a Claude Code (or other agent) session, and exits `0` — it is documentation output, not an error. `cs cuo` with no name, or an unrecognised name, lists all four valid names. No subprocess is spawned; no attempt is made to run the workflow headlessly.

## Alternatives Considered

- Attempt to drive the CUO workflow headlessly from the CLI (e.g. shelling out to `cyberos-cuo`'s Python entry point with a scripted agent loop). Rejected explicitly by the plan (§5 item 4: "do not implement standalone execution in this task") — building real headless/agentic execution is deferred as its own follow-up decision (plan §5, out of scope bullet), not something this task should quietly attempt.
- Have `cs cuo <name>` shell out to `cyberos-cuo list-personas` or similar to at least show *something* real rather than a static message. Rejected: this would expose internal persona/workflow machinery not meant for direct end-user consumption, and risks implying standalone execution works when it does not.
- Recognise install/uninstall/version/status/help under `cuo` as well, for a single unified namespace. Rejected: those five are deterministic shell scripts with their own existing flat `cs` verbs (`cli.mjs:19-26`); nesting them under `cuo` too would contradict plan §2's own framing of what CUO backs, and would make `cs cuo install` seem like a legitimate synonym for `cs install` when it is not.

## Success Metrics

- Primary: `cs cuo plan`, `cs cuo create-tasks`, `cs cuo ship-tasks`, and `cs cuo improve` each print the correct matching slash command name and exit `0`. Baseline today: `cuo` is not in `cli.mjs`'s `SCRIPTS` table; any `cyberos cuo ...` (or post-rename `cs cuo ...`) invocation falls through to "unknown command."
- Guardrail: `cs cuo` with no argument, and `cs cuo bogus-name`, both list all four valid names and exit `0` — neither is treated as an error, since the stub's entire job is orientation, not validation.

## Scope

In scope: the `cuo` dispatch entry in `cli.mjs` recognising exactly `plan`, `create-tasks`, `ship-tasks`, `improve`; its no-argument/unrecognised-argument listing behaviour; and the `help.sh`/`tools/install/docs/index.md` mentions of the new verb.

### Out of scope / Non-Goals

- Any standalone or headless execution of a CUO workflow — explicitly deferred by the plan.
- Recognising `install`/`uninstall`/`version`/`status`/`help` under the `cuo` namespace — they already have their own flat `cs` verbs and are not CUO-backed work.
- Changes to `modules/cuo`'s own Python implementation or persona system.
- The `memory` verb — that is TASK-IMP-131.

## Dependencies

Depends on TASK-IMP-130 (adds a verb to the same `cli.mjs` dispatch table under the renamed bin). Blocks TASK-IMP-134's end-to-end regression.

**Relationship to TASK-IMP-076.** Same dispatch-table precedent as TASK-IMP-131 (`mcp`/`gates` established the `spawnSync`-based pattern this family of verbs follows); this task, notably, does NOT spawn any subprocess at all — it is the one verb in this batch that is pure static text, which is itself a deliberate scope decision (see Alternatives Considered), not an oversight.

**Sibling-task coordination with TASK-IMP-131.** Both tasks add a new entry to the same `cli.mjs` dispatch table and depend only on TASK-IMP-130; see TASK-IMP-131's Dependencies section for the shared merge-conflict/sequencing note — it applies symmetrically to this task and is not repeated in full here to avoid two specs disagreeing if one is edited later.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill inside Cowork.
- **Scope:** the four-name CUO-backed subset was cross-checked directly against `tools/install/plugin/commands/`'s actual file listing and `README.md:28`'s broader slash-command list, not assumed from the plan's prose alone — the plan's own §2 citation was the primary source, confirmed rather than merely repeated.
- **Human review:** task decomposition approved at the 2026-07-22 PLAN gate.

## 1. Description (normative)

- 1.1 `cli.mjs`'s dispatch table MUST recognise `cuo` as a known top-level command.
- 1.2 `cs cuo plan` MUST print a message naming `/plan` as the slash command to run, and MUST exit `0`.
- 1.3 `cs cuo create-tasks`, `cs cuo ship-tasks`, and `cs cuo improve` MUST each print a message naming their respective matching slash command (`/create-tasks`, `/ship-tasks`, `/improve`) and MUST each exit `0`.
- 1.4 `cs cuo` with no argument MUST print a listing of all four valid names and MUST exit `0` (orientation, not an error — matching the bare top-level `cs` invocation's own exit-`0` convention at `cli.mjs:59`).
- 1.4a `cs cuo <unrecognised-name>` MUST print the same listing but MUST exit with code `2` (matching `cli.mjs`'s established convention for a usage mistake, `cli.mjs:87`) — distinct from the bare-invocation case, since a mistyped name is a usage error a caller may want to detect, while a bare invocation asking for orientation is not.
- 1.5 `cs cuo <any argument>` MUST NOT spawn any subprocess, MUST NOT invoke Python, and MUST NOT attempt to execute a CUO workflow, and MUST NOT probe whether `cyberos-cuo` is locally installed — unlike TASK-IMP-131's `memory` verb, which does detect and report local tool availability, the plan constrains `cuo` more tightly ("do not implement standalone execution," no mechanism left open for later): this task's stub prints fixed text only, with no local-environment awareness at all. The two sibling verbs are deliberately asymmetric because the plan treats them differently, not by oversight.
- 1.6 `help.sh` and `tools/install/docs/index.md` MUST document the `cuo` verb and MUST describe it, on the same line or the immediately adjacent line as the mention, as a redirect/orientation aid rather than standalone execution. `tools/install/docs/index.md:27`'s "the same eight commands" sentence MUST be updated to the correct count once `cuo` (and, per TASK-IMP-131, `memory`) are added — whichever of TASK-IMP-131/132 lands second is responsible for the numeral, not just appending its own verb's name.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.2, #1.1) - `cs cuo plan` output contains the literal substring `/plan` and the process exits `0` - this alone proves `cuo` was recognised as a known command (clause 1.1), since an unrecognised top-level command falls through to the "unknown command" branch and could never reach this output - test: `tools/install/tests/test_cli_cuo_verb.sh::t01_plan_redirect_and_recognition`
- [x] AC 2 (traces_to: #1.3) - each of `cs cuo create-tasks`, `cs cuo ship-tasks`, `cs cuo improve` prints its matching slash-command substring and exits `0` - test: `tools/install/tests/test_cli_cuo_verb.sh::t02_other_three_redirects`
- [x] AC 3 (traces_to: #1.4) - `cs cuo` (no args) prints all four valid names and exits `0` - test: `tools/install/tests/test_cli_cuo_verb.sh::t03_bare_invocation_lists_and_exits_0`
- [x] AC 4 (traces_to: #1.4a) - `cs cuo nonexistent-workflow` prints all four valid names and exits with code exactly `2` - test: `tools/install/tests/test_cli_cuo_verb.sh::t04_unrecognised_name_lists_and_exits_2`
- [x] AC 5 (traces_to: #1.5) - with tripwire `python3` and `bash` stand-ins on `$PATH` that each write a marker file if invoked, running `cs cuo plan`, `cs cuo create-tasks`, `cs cuo ship-tasks`, and `cs cuo improve` in sequence leaves both marker files absent afterward - test: `tools/install/tests/test_cli_cuo_verb.sh::t05_no_subprocess_spawned` (mirrors the tripwire-binary style already used in TASK-IMP-131's test suite, matching this repo's bash-test-harness convention rather than a JS-level module spy)
- [x] AC 6 (traces_to: #1.6) - `help.sh` output and `tools/install/docs/index.md` each mention `cuo` with a redirect-describing word on the same or adjacent line, and `tools/install/docs/index.md`'s command-count sentence reads a number matching the actual verb count in `cli.mjs`'s `SCRIPTS` table at the time of the check (not hardcoded to eight) - test: `tools/install/tests/test_cli_cuo_verb.sh::t06_docs_describe_as_redirect_and_count_correct`

## 3. Edge cases

- Argument casing or a leading slash typed by habit (`cs cuo /plan` instead of `cs cuo plan`) - out of scope for this task to normalise; the stub matches the four bare names exactly and falls into the "unrecognised" listing branch (AC 4) for anything else, which itself still orients the user correctly.
- `cs cuo` invoked from a plain terminal with no agent session active at all: the printed message still names the slash command correctly - the stub does not attempt to detect whether an agent session exists, since doing so is unnecessary complexity for a text-printing command with no side effects either way.
- A future fifth CUO-backed workflow is added to `modules/cuo` without a corresponding update to this stub's four-name list: the stub silently under-lists it. Not a defect this task introduces, but named here as a maintenance risk - whichever task adds a fifth CUO workflow should update this list, and TASK-IMP-134's regression does not itself guard against this drift (it only proves the current four resolve correctly).
- Security-class: this verb's entire behaviour is printing a fixed string keyed on `argv[1]` - it reads no file, spawns no process, and accepts no input that reaches a shell or filesystem call, so it carries no new attack surface beyond argument parsing itself.
