---
id: TASK-IMP-131
title: Add a `memory` verb to `cs`, gated on local availability
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-22T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-130]
blocks: [TASK-IMP-134]
related_tasks: [TASK-IMP-076]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.9"
owner: Stephen Cheng (CTO)
created: 2026-07-22
memory_chain_hash: null
effort_hours: 3
service: tools/install
new_files:
  - tools/install/tests/test_cli_memory_verb.sh
modified_files:
  - tools/install/cli/bin/cli.mjs
  - tools/install/help.sh
  - tools/install/docs/index.md
source_pages:
  - "docs/plans/PLAN-cli-module-namespacing-2026-07-22/plan.md §5 Scope item 3 ('Add memory as a new top-level verb under cs, dispatching into modules/memory's existing logic (mechanism ... is an implementation decision, not fixed by this plan)') and §3 Option A ('extend cli.mjs to add memory ... verbs that spawnSync into the Python packages as subprocesses ... Cost: makes Python a hard runtime dependency')"
  - "tools/install/build.sh:25,159,161-162 (payload's memory/ directory is populated ONLY with AGENTS.md, memory.schema.json, memory.invariants.yaml — the Layer-1 protocol spec vendored into a CONSUMER repo's .cyberos/memory/, not modules/memory's Python implementation)"
  - "tools/install/build.sh:356 (package.json 'files' list includes \"memory\" — confirmed to mean the protocol dir above, not the Python package, by cross-checking against build.sh:159-162)"
  - "modules/memory/cyberos/__main__.py:1-22 (module docstring: invoked as `python -m cyberos <verb>`; canonical ops view/put/move/delete/audit/verify/export/search/checkpoint)"
  - "modules/memory/pyproject.toml:6,28-29 (package name cyberos-memory, console-script entry `cyberos = \"cyberos.__main__:main\"`, confirmed by the plan not published to PyPI)"
  - "tools/install/cli/bin/cli.mjs:69,78,83 (existing spawnSync dispatch pattern for mcp/gates/repo-scoped commands — the pattern this task follows for a fourth dispatch shape)"
  - "tools/install/build.sh:273 (existing precedent for a conditional `command -v python3` check elsewhere in the build, confirming this is an established pattern for optional-Python-availability handling in this codebase)"
source_decisions:
  - "2026-07-22 Stephen: create-tasks PLAN gate — APPROVE as rendered."
  - "2026-07-22 authoring: while grounding this task's dispatch mechanism against the actual payload contents (not just the plan's prose), discovered the npm payload does NOT currently vendor modules/memory's Python implementation at all — only the Layer-1 protocol/schema files ship under memory/. A `cs memory <cmd>` verb therefore cannot reach a real BRAIN store on a machine that only ran `npm install -g @cyberskill/cyberos`; it can only reach one on a machine that separately has `cyberos-memory` pip-installed (today, effectively CyberSkill's own engineers working in a full monorepo checkout). This materially narrows what 'dispatching into modules/memory's existing logic' can mean for a first cut, and is scoped explicitly below rather than silently promising full end-user reach. Flagged in the batch's SPEC_DEFECTS_FOUND report as a new fact the plan did not anticipate."
  - "2026-07-22 self-audit revision (score_pre_revision 6/10 -> score_post_revision 10/10): AC 1 originally asserted only that 'unknown command' was absent, not that a working resolution actually routes further (TRACE-006 gap, same pattern as TASK-IMP-130 ISS-001) - added a positive half using a working stub. AC 2 originally described the non-$PATH-lookup requirement without a concretely distinguishing test setup - revised to two differently-labelled fake binaries so the test can prove WHICH mechanism actually ran. Success Metrics' primary lacked an explicit baseline statement - added. Clause 1.4 / AC 4 said 'non-zero' without pinning a specific code, inconsistent with cli.mjs's own established convention (exit 2 for both 'unknown command' and 'gates missing') - tightened to exit code 2. Added an edge case naming the unverified-module-identity risk in the resolution check, previously unaddressed. Added a Dependencies note on TASK-IMP-131/132 sibling coordination (both modify the same dispatch table, depend only on 130, and could be implemented in parallel with no cross-reference otherwise)."
---

# TASK-IMP-131: Add a `memory` verb to `cs`, gated on local availability

## Summary

Add `memory` as a top-level verb in `cs`'s dispatch table that subprocess-dispatches into the locally available `cyberos-memory` BRAIN-store CLI when present, and fails with a clear, actionable message when it is not — because the npm payload does not currently vendor that CLI's implementation, only its protocol spec.

## Problem

`docs/plans/PLAN-cli-module-namespacing-2026-07-22/plan.md` §5 item 3 asks for a `memory` verb under `cs` that reaches `modules/memory`'s BRAIN-store operations, leaving the dispatch mechanism to implementation. `cli.mjs` already dispatches to two external processes this way — `mcp` (a Node subprocess, `cli.mjs:69`) and `gates` (a bash script inside the TARGET repo's vendored machine, `cli.mjs:78`) — so a subprocess-dispatch verb is a well-precedented shape, not a new pattern.

What is NOT precedented, and was not established by the plan, is that the actual `cyberos-memory` Python package is reachable from an installed `cs`. Reading `build.sh:25,159,161-162` shows the payload's `memory/` directory is populated with exactly three files — `AGENTS.md`, `memory.schema.json`, `memory.invariants.yaml` — the Layer-1 memory *protocol* that gets vendored into a consumer repo's own `.cyberos/memory/` for humans and agents to follow. None of `modules/memory/cyberos/`'s actual Python source ships in the npm package. `cyberos-memory` (`modules/memory/pyproject.toml:6`) is also confirmed not published to PyPI (plan §2). So on a machine where a user only ran `npm install -g @cyberskill/cyberos` (or `npx cyberos install`, soon `npx cs install`), there is no `cyberos-memory` for a `memory` verb to reach — `python -m cyberos` (`modules/memory/cyberos/__main__.py`'s own documented invocation) will not resolve to anything meaningful unless that machine separately has the internal package pip-installed from a full monorepo checkout.

## Proposed Solution

Add a `memory` entry to `cli.mjs`'s dispatch: on `cs memory <args...>`, first resolve whether a working `cyberos-memory` is reachable — try the console-script name `cyberos` is already taken by the OLD public bin and by the BRAIN CLI itself, so resolution must not rely on bare `$PATH` lookup of `cyberos` (that is the exact collision this whole plan exists to remove); instead resolve via `python3 -m cyberos --help` succeeding in the current environment (matching the module's own documented invocation form), or a `CYBEROS_MEMORY_STORE`-adjacent override the operator sets explicitly. If that resolution succeeds, `spawnSync("python3", ["-m", "cyberos", ...rest], { stdio: "inherit" })`. If it fails, print a clear message naming that `cyberos-memory` is an internal package not bundled with this CLI today, and exit non-zero — never a raw Python traceback or a silent no-op.

## Alternatives Considered

- Vendor `modules/memory`'s full Python source (and its dependencies — `msgspec`, `cryptography`, etc. per `pyproject.toml`'s dynamic dependency file) into the npm payload so `cs memory` works out of the box for any installer. Rejected for THIS task: this is the packaging expansion the plan's Option A cost warning anticipated ("makes Python a hard runtime dependency of what is currently a pure-Node CLI") taken to its full conclusion, and is a materially larger lift (vendoring a Python runtime + deps inside an npm package, or shipping a bundled interpreter) than "add a dispatch entry." Flagged as an explicit follow-up decision in Scope, not silently absorbed into this task's estimate.
- Make `cs memory` fail closed with a permanent "not available" message regardless of local environment, deferring even the gated dispatch to a future task. Rejected: the plan's item 3 asks for a real verb now, and gating on local availability (rather than refusing to try) still delivers value today for the only population that currently has a working BRAIN-store install — CyberSkill's own engineers — while being honest with everyone else.
- Resolve availability by checking for the `cyberos` bin on `$PATH` and assuming it is the memory CLI if found. Rejected: this is exactly the ambient-name collision this entire plan exists to eliminate; a `$PATH` lookup for `cyberos` could still resolve to a leftover old install of the RENAMED public CLI on a machine that hasn't fully re-provisioned, silently mis-dispatching.

## Success Metrics

- Primary: on a machine with `cyberos-memory` pip-installed and `python3 -m cyberos` resolving, `cs memory doctor` (or any memory subcommand) reaches the real BRAIN store and returns its actual output, by the same release that ships TASK-IMP-130's rename. Baseline today: `memory` is not in `cli.mjs`'s `SCRIPTS` table at all, so `cyberos memory` (or, post-rename, `cs memory`) currently falls through to the "unknown command" branch regardless of what is installed locally.
- Guardrail: on a machine WITHOUT `cyberos-memory` installed, `cs memory <anything>` exits with code `2` (the same code `cli.mjs` already uses for "unknown command" and "gates missing" — a recognised-but-unusable command, not a crash) and a message identifying memory as an internal package not bundled with this install — never a Python traceback, never a silent success, never the generic "unknown command" text (since `memory` IS a known command; it's the backing tool that's missing).

## Scope

In scope: the `memory` dispatch entry in `cli.mjs`, its `python3 -m cyberos` resolution check and clear failure message, and the doc updates in `help.sh`/`docs/index.md` describing the new verb and its local-availability caveat.

### Out of scope / Non-Goals

- Vendoring `modules/memory`'s Python implementation and its dependencies into the npm payload so `cs memory` works without a separate local install — a materially larger packaging decision, explicitly deferred (see Alternatives Considered).
- Any change to `modules/memory`'s own CLI behaviour, schema, or the BRAIN store's internals.
- Publishing `cyberos-memory` to PyPI — the plan confirms no such plan exists.
- The `cuo` verb — that is TASK-IMP-132.

## Dependencies

Depends on TASK-IMP-130 (the `cs` bin rename must land first; this task adds a verb to the SAME dispatch table `cli.mjs` exposes under the new name). Blocks TASK-IMP-134's end-to-end regression, which must exercise `cs memory <cmd>` per the plan's own success criterion (plan §6 item 7).

**Relationship to TASK-IMP-076.** That task established `cli.mjs`'s `spawnSync`-based dispatch pattern for `mcp` and `gates`; this task reuses the same pattern for a third external-process verb rather than inventing a new dispatch shape.

**Sibling-task coordination with TASK-IMP-132.** Both this task and TASK-IMP-132 add a new top-level entry to the same `cli.mjs` dispatch table and depend only on TASK-IMP-130, not on each other — nothing stops them from being implemented in parallel. Whichever lands second MUST rebase its dispatch-table addition against the first rather than assuming a clean apply; this is a merge-conflict/sequencing risk worth naming explicitly rather than leaving implicit, since neither task's spec alone would surface it.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill inside Cowork.
- **Scope:** the finding that the npm payload does not vendor `modules/memory`'s Python source was made by reading `build.sh`'s actual file-copy lines during authoring, not carried over from the plan's own text (the plan does not state this). Every `source_pages` line was read at HEAD in this checkout.
- **Human review:** task decomposition approved at the 2026-07-22 PLAN gate. The gated-availability scope decision (rather than full vendoring) is an authoring-time call, flagged in the batch report for the operator to revisit.

## 1. Description (normative)

- 1.1 `cli.mjs`'s dispatch table MUST recognise `memory` as a known top-level command (not fall through to the "unknown command" branch).
- 1.2 On `cs memory <args>`, the CLI MUST attempt to resolve a working `cyberos-memory` via `python3 -m cyberos --help` (or equivalent resolution) before dispatching, and MUST NOT resolve availability by a bare `$PATH` lookup of the name `cyberos`.
- 1.3 When resolution succeeds, the CLI MUST spawn `python3 -m cyberos <args>` with inherited stdio and MUST forward the child process's exit code.
- 1.4 When resolution fails, the CLI MUST print a message identifying `cyberos-memory` as an internal package not bundled with this install, and MUST exit with code `2` (matching `cli.mjs`'s existing convention for a recognised-but-unusable command) without invoking Python.
- 1.5 `help.sh` and `docs/index.md` MUST document the `memory` verb and MUST state its local-availability gating, not present it as universally available.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - two runs against the SAME `cs memory --help` invocation: (a) with no `python3` stub present, output does NOT contain "unknown command 'memory'"; (b) with a working `python3 -m cyberos` stub present, its stub output IS printed - proving `memory` is routed to the dispatch/resolution logic in both cases, not merely that the unknown-command message is absent - test: `tools/install/tests/test_cli_memory_verb.sh::t01_memory_is_known_command`
- [ ] AC 2 (traces_to: #1.2) - with a fake `cyberos` executable on `$PATH` that prints `WRONG-PATH-DISPATCH` and exits 0, alongside a working `python3 -m cyberos` stub that prints `CORRECT-DISPATCH`, `cs memory doctor` prints `CORRECT-DISPATCH` and never prints `WRONG-PATH-DISPATCH` - proves resolution goes through the `python3 -m cyberos` check, not a bare `$PATH` lookup of `cyberos` - test: `tools/install/tests/test_cli_memory_verb.sh::t02_resolution_not_via_path_cyberos`
- [ ] AC 3 (traces_to: #1.3) - with a stub `python3 -m cyberos` that echoes its args and exits 3, `cs memory foo bar` prints the echoed args and the CLI process exits 3 - test: `tools/install/tests/test_cli_memory_verb.sh::t03_dispatch_forwards_args_and_exit_code`
- [ ] AC 4 (traces_to: #1.4) - with no `python3` on `$PATH` at all, `cs memory doctor` exits with code exactly `2`, prints a message containing "cyberos-memory" and "not bundled" (or equivalent), and produces no Python traceback text - test: `tools/install/tests/test_cli_memory_verb.sh::t04_missing_python_clear_error`
- [ ] AC 5 (traces_to: #1.5) - `help.sh`'s output and `docs/index.md` both mention `memory` as a verb and both contain a caveat sentence about local availability (grep for a shared marker string) - test: `tools/install/tests/test_cli_memory_verb.sh::t05_docs_state_gating`

## 3. Edge cases

- `python3` exists on `$PATH` but `cyberos-memory` is not installed under it: `python3 -m cyberos --help` exits non-zero (module not found) - the resolution check MUST treat this the same as "no python3 at all," not attempt to run the real subcommand anyway and surface a raw `ModuleNotFoundError`.
- A machine with only `python` (no `python3`) on `$PATH`: out of scope for this task's resolution check - `modules/memory`'s own docstring documents `python -m cyberos`, but this task standardises on `python3` to match `build.sh:273`'s existing convention; a `python`-only machine is treated as unavailable, not probed further.
- Arguments containing spaces or shell metacharacters passed to `cs memory <args>`: forwarded via `spawnSync`'s argv array (not a shell string), so no additional escaping is needed and none should be added - matches the existing `mcp`/`gates` dispatch pattern already in `cli.mjs`.
- Running `cs memory` with no further arguments: forwarded as `python3 -m cyberos` with an empty arg list, whose own `argparse`-based `--help`-on-no-args behaviour (if any) is `modules/memory`'s concern, not this task's - this task does not special-case the empty-args case beyond normal forwarding.
- Resolution as specified (`python3 -m cyberos --help` succeeds) does not verify the resolved module is genuinely `cyberos-memory` rather than some unrelated Python package coincidentally also importable as `cyberos` on that machine - a real if low-probability misdispatch risk. Accepted, named limitation for this task's scope: adding a signature check (e.g. matching a known string in the real CLI's own `--help` output) is deferred as disproportionate hardening for a first cut; a future task can add it if the risk is ever observed in practice.
- Security-class: this task adds a new subprocess-spawn path from an untrusted `$PATH` resolution (`python3`). Unlike `mcp` (spawns a fixed script inside the payload) and `gates` (spawns a fixed script inside the target repo's vendored machine), `memory` spawns whatever `python3` resolves to on the operator's own machine - explicitly the same trust boundary the operator's shell already has, and no new privilege; the CLI does not search a wider or different `$PATH` than the shell it was invoked from.
