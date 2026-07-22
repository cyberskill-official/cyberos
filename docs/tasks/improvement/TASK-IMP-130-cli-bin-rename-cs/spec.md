---
id: TASK-IMP-130
title: Rename the public CyberOS CLI bin from `cyberos` to `cs`
template: task@1
type: improvement
module: improvement
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-22T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: [TASK-IMP-131, TASK-IMP-132, TASK-IMP-133, TASK-IMP-134, TASK-IMP-135]
related_tasks: [TASK-IMP-076]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.9"
owner: Stephen Cheng (CTO)
created: 2026-07-22
memory_chain_hash: null
effort_hours: 4
service: tools/install
new_files:
  - tools/install/tests/test_cli_rename.sh
modified_files:
  - tools/install/build.sh
  - tools/install/cli/bin/cli.mjs
  - tools/install/help.sh
  - tools/install/plugin/commands/help.md
  - tools/install/docs/index.md
  - tools/install/README.md
  - CHANGELOG.md
source_pages:
  - "docs/plans/PLAN-cli-module-namespacing-2026-07-22/plan.md §4 (Decision: rename the PUBLIC bin cyberos -> cs, not the internal memory CLI) and §5 Scope items 1, 2, 5"
  - "tools/install/build.sh:341-347 (payload package.json generation: name '@cyberskill/cyberos', bin { \"cyberos\": \"cli/bin/cli.mjs\" })"
  - "tools/install/cli/bin/cli.mjs:2 (top comment: 'npx cyberos <command> [args] — the single CLI entry point'), :32 (usage() header string), :48 (usage() prints 'Docs: https://cyberos.cyberskill.world/docs')"
  - "tools/install/help.sh:25 ('npx cyberos <command>   install | uninstall | version | status | create | gates | mcp | help'), :31 ('Docs: GUIDE.md · https://cyberos.cyberskill.world/docs')"
  - "tools/install/plugin/commands/help.md line 16 ('5. Docs: https://cyberos.cyberskill.world/docs')"
  - "tools/install/docs/index.md:27 ('Via the npm package the same eight commands are `npx cyberos <command>`: install, uninstall, version, status, create, gates, mcp, help.')"
  - "tools/install/README.md:186-188 ('npx cyberos install [dir]', 'npx cyberos-gates [dir]', 'npx cyberos-mcp')"
  - "modules/memory/pyproject.toml:6,29 (separate package `cyberos-memory`, console script `cyberos = \"cyberos.__main__:main\"` — confirmed by the plan not published to PyPI and internal-only; this task does not touch this file)"
  - "CHANGELOG.md:1-5 (current head; latest entry [1.0.9] 2026-07-22, 'Maintenance release')"
source_decisions:
  - "2026-07-22 Stephen: create-tasks PLAN gate — APPROVE as rendered (5-task decomposition, module improvement, ids TASK-IMP-130..134)."
  - "2026-07-22 authoring: while gathering source citations for the doc sweep in plan §5 item 5, found the stale domain string `https://cyberos.cyberskill.world/docs` (distinct from both domains PR #107 already fixed: `cyberos-wiki.cyberskill.world` and `docs.cyberos.world`) occurs in THREE files this task already touches — cli.mjs:48, help.sh:31, plugin/commands/help.md:16 — not just the one instance flagged to the operator earlier. Folded fixing all three into this task's scope (§1.5 below) rather than opening a sixth task for a one-line string fix in files already under edit; flagged here rather than silently expanded, since it was not itemized in plan.md §5."
  - "2026-07-22 authoring: plan.md §4 leaves the npm package-name question (`@cyberskill/cyberos` vs. renaming to match `cs`) open for this task to decide. Decided: keep the package name `@cyberskill/cyberos`, rename only the `bin` field to `cs`. npm supports a package name that differs from its bin command (used by many packages); renaming the published package name is the more disruptive break (it changes the `npm install` target, not just the invoked command) for no corresponding benefit, since the bin name — not the package name — is what a user types. See Alternatives Considered."
  - "2026-07-22 self-audit revision (score_pre_revision 6/10 -> score_post_revision 10/10): first draft cited new AC test cases against `tools/install/tests/test_channels.sh`, an existing file that uses sequential ok()/bad() checks with no named test functions - the wrong convention for the `file::test_name` citation style this contract requires. Replaced with a dedicated new file `tools/install/tests/test_cli_rename.sh` using the named-function convention (`t01_..`..`t07_..`) that `test_install_hygiene.sh` already establishes, and added it to `new_files`, which the first draft omitted entirely despite every AC requiring a new test. AC 4 originally asserted only the ABSENCE of the old `cyberos <command>` string, which a test could pass even if the replacement text were deleted rather than updated to `cs` (TRACE-006 gap) - revised to assert the POSITIVE replacement text is present in each file, not just that the old string is gone. Added prose in Dependencies explaining the relationship to TASK-IMP-076 (related_tasks listed it but the body never said why). Added an explicit release-cycle timeframe to both Success Metrics, which lacked one. Softened the npm-bin-symlink-cleanup edge case, which originally asserted a specific unverified npm upgrade behaviour with more confidence than the author had actually confirmed; reframed as an open question TASK-IMP-134's clean-machine regression must observe directly."
---

# TASK-IMP-130: Rename the public CyberOS CLI bin from `cyberos` to `cs`

## Summary

The npm-published CyberOS CLI exposes itself as the bin command `cyberos`, which collides on `$PATH` with an unrelated internal tool that happens to already claim that name. This task renames the public bin to `cs`, updates every place the CLI describes its own invocation, and folds in a stale-domain fix discovered in the same files.

## Problem

`brew install --cask cyberos` followed by `brew trust` installs the Homebrew-packaged CLI correctly, but running `cyberos -h` afterward showed an unrelated 39-subcommand BRAIN-store CLI instead of the Homebrew-installed one. Root cause, confirmed by direct repo inspection (`docs/plans/PLAN-cli-module-namespacing-2026-07-22/plan.md` §2): the npm-published `@cyberskill/cyberos` package (`tools/install/build.sh:341-347`) declares its bin as `cyberos` (`tools/install/cli/bin/cli.mjs`), and a completely separate, PyPI-unpublished internal package (`modules/memory`, `cyberos-memory`) independently declares a console-script entry also named `cyberos` (`modules/memory/pyproject.toml:29`). Whichever one lands later on `$PATH` (in the reported case, a local/dev pyenv install of `modules/memory`) wins, and there is no error — just the wrong tool answering to the name, silently.

`cyberos-memory` is confirmed internal-only with no plan to publish (plan §2), so it was never actually the thing that needed to change to solve the founder's goal, which is "the only public command is `cs`" (plan §4). Renaming the public bin directly delivers that and needs no coordination with the internal package's own naming.

## Proposed Solution

Change the npm package's `bin` field (generated by `build.sh:341-347`) from `{ "cyberos": "cli/bin/cli.mjs" }` to `{ "cs": "cli/bin/cli.mjs" }`, keeping the package name `@cyberskill/cyberos` unchanged (see Alternatives Considered). Update every place the CLI's own text describes its invocation — `cli.mjs`'s top comment and `usage()` output, `help.sh`'s "Channels" section, the plugin's `help.md`, `docs/index.md`'s "same eight commands" line, and the `npx cyberos ...` examples in `tools/install/README.md` — to read `cs` instead of `cyberos`. Add a CHANGELOG entry calling out the rename as breaking, per the plan's explicit instruction not to ship it as a silent swap (plan §7). While editing `cli.mjs`, `help.sh`, and `help.md` for the rename, also correct the stale domain string `https://cyberos.cyberskill.world/docs` found in all three to the canonical `https://os.cyberskill.world/docs` already in use elsewhere in the repo (e.g. `README.md:7`) since PR #107.

## Alternatives Considered

- Rename the npm package name (`@cyberskill/cyberos` → `@cyberskill/cs` or similar) along with the bin. Rejected: npm allows a package's name to differ from the command(s) it installs (many published CLIs do this), and the package name is what appears in `npm install <name>` / the registry URL / `repository`/`homepage` metadata already tied to the OIDC trusted-publishing pipeline (plan §7 flags that pipeline as fragile — pinned to org+repo+workflow-filename trust). Renaming the package name changes more surface for zero user-facing benefit, since the thing a user types day to day is the bin command, not the package name.
- Keep `cyberos` as an alias alongside `cs` during a transition window. Rejected per the plan (§7): the founder explicitly rejected keeping `cyberos` as an alias, since an alias would leave the exact collision this task exists to remove.
- Rename `modules/memory`'s console-script entry instead of the public bin (the original, since-superseded plan revision). Rejected: superseded same day by the founder directly (plan §9 revision log) once it was confirmed `cyberos-memory` was never going to be published — renaming an internal-only tool to protect a name the public CLI was going to leave anyway solves nothing.
- Leave the stale domain fix for a separate task. Rejected: all three instances are single-line strings inside files this task already opens for the rename; deferring them means a sixth reviewer touching the same three lines for an unrelated one-word change.

## Success Metrics

- Primary: by the next CyberOS release after 1.0.9, `npx cs install`, `npx cs -h`, and every subcommand in `cli.mjs`'s `SCRIPTS` table resolve correctly, with zero remaining `npx cyberos` or bare `cyberos <verb>` references in `tools/install/{cli/bin/cli.mjs,help.sh,plugin/commands/help.md,docs/index.md,README.md}`. Baseline today: all five files instruct the reader to type `cyberos`.
- Guardrail: in that same release, the payload's `package.json` `name` field remains `@cyberskill/cyberos` (unchanged), so `npm install @cyberskill/cyberos` continues to resolve to the same package post-rename — only the invoked command changes, not the install target.

## Scope

In scope: `build.sh`'s package.json generation (`bin` field only — `name` stays), `cli.mjs`'s dispatch/usage text, `help.sh`, the plugin's `help.md`, `docs/index.md`, the `npx cyberos ...` examples in `tools/install/README.md`, a CHANGELOG entry, and the three stale-domain instances found in the same files touched for the rename.

### Out of scope / Non-Goals

- Renaming `modules/memory`'s own console-script entry (`modules/memory/pyproject.toml:29`) — confirmed internal-only, never needed for this collision, left untouched per the plan.
- Renaming the npm package name `@cyberskill/cyberos` — decided against; see Alternatives Considered.
- Adding a `memory` or `cuo` verb to the renamed dispatch table — that is TASK-IMP-131 / TASK-IMP-132.
- Updating `Formula/cyberos-cli.rb` in the separate `homebrew-tap` repo — that is TASK-IMP-133.
- The end-to-end regression proving the rename works on a clean machine — that is TASK-IMP-134.
- Any pre-existing inconsistency in `tools/install/README.md` unrelated to the `cyberos`→`cs` rename itself (e.g. lines 186-188 describe `npx cyberos-gates`/`npx cyberos-mcp` as if they were separate bin names, which does not match `cli.mjs`'s actual single-bin dispatch design) — flagged as a pre-existing documentation inconsistency, not introduced or fixed by this task.

## Dependencies

None blocking — this is the root task the other four in this batch depend on. Touches the OIDC trusted-publishing pipeline in `release.yml` (plan §7): a bin-name change should go through the normal release flow and be re-tested end to end by TASK-IMP-134, not verified in isolation here.

**Relationship to TASK-IMP-076 (done).** That task shipped the original `install`/`uninstall`/`version`/`status`/`help` root CLI surface and the `mcp`/`gates` verbs inside `cli.mjs` — the exact dispatch table and usage text this task renames. It established the "three channels cannot drift" design (plugin slash commands, `help.sh`, `cli.mjs` mirror the same command set 1:1) that this task's doc sweep must preserve under the new name, not just the bin field itself.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill inside Cowork.
- **Scope:** every `source_pages` line above was read at HEAD in this checkout during authoring; the stale-domain finding in `help.sh` and `plugin/commands/help.md` (beyond the single `cli.mjs` instance already surfaced to the operator in conversation) was discovered while gathering these citations, not asserted from memory.
- **Human review:** task decomposition and scope approved at the 2026-07-22 PLAN gate (create-tasks). The package-name decision and the domain-fix scope addition are recorded above as authoring-time calls, not yet independently reviewed beyond that PLAN approval — flagged for the operator to revisit if either call is unwanted.

## 1. Description (normative)

- 1.1 The payload's generated `package.json` MUST declare its `bin` field as `{ "cs": "cli/bin/cli.mjs" }` and MUST NOT change the `name` field from `@cyberskill/cyberos`.
- 1.2 `cli.mjs`'s top comment and `usage()` output MUST refer to the invocation as `cs <command>` (or `npx cs <command>`), not `cyberos <command>`.
- 1.3 `help.sh`'s "Channels" section MUST describe the npm channel as `npx cs <command>`, not `npx cyberos <command>`.
- 1.4 The plugin's `help.md`, `docs/index.md`'s "same eight commands" line, and every `npx cyberos ...` example in `tools/install/README.md` MUST read `cs` in place of `cyberos`.
- 1.5 The domain string `https://cyberos.cyberskill.world/docs` MUST be corrected to `https://os.cyberskill.world/docs` in every file this task modifies that contains it (`cli.mjs`, `help.sh`, `plugin/commands/help.md`).
- 1.6 `CHANGELOG.md` MUST gain an entry documenting the rename as a breaking change, naming the old (`cyberos`) and new (`cs`) command explicitly.
- 1.7 This task MUST NOT modify `modules/memory/pyproject.toml`'s console-script entry.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - a scratch build's `dist/*/package.json` has `bin: {"cs": "cli/bin/cli.mjs"}` and `name: "@cyberskill/cyberos"` - test: `tools/install/tests/test_cli_rename.sh::t01_bin_renamed_to_cs`
- [ ] AC 2 (traces_to: #1.2) - `node dist/*/cli/bin/cli.mjs --help` prints usage text containing `cs <command>` and contains no substring `cyberos <command>` - test: `tools/install/tests/test_cli_rename.sh::t02_usage_text_says_cs`
- [ ] AC 3 (traces_to: #1.3) - `bash dist/*/help.sh` output contains `npx cs <command>` and does not contain `npx cyberos` - test: `tools/install/tests/test_cli_rename.sh::t03_help_sh_says_cs`
- [ ] AC 4 (traces_to: #1.4) - for each of `tools/install/plugin/commands/help.md`, `tools/install/docs/index.md`, and `tools/install/README.md`'s `npx cyberos install [dir]` example line: the file now contains the replacement `cs`-based text (`/help` → mentions `cs`; `docs/index.md` → `npx cs <command>`; `README.md` → `npx cs install [dir]`) AND a grep for the literal string `cyberos <command>` across the three returns zero matches - test: `tools/install/tests/test_cli_rename.sh::t04_docs_sweep_replaced_not_just_removed`
- [ ] AC 5 (traces_to: #1.5) - a grep of `cli.mjs`, `help.sh`, and `plugin/commands/help.md` for `cyberos.cyberskill.world` returns zero matches, and each now contains `os.cyberskill.world/docs` - test: `tools/install/tests/test_cli_rename.sh::t05_stale_domain_fixed`
- [ ] AC 6 (traces_to: #1.6) - `CHANGELOG.md`'s top entry mentions both `cyberos` and `cs` and the word "breaking" - test: `tools/install/tests/test_cli_rename.sh::t06_changelog_entry_present`
- [ ] AC 7 (traces_to: #1.7) - `git diff` for this change touches no file under `modules/memory/` - test: `tools/install/tests/test_cli_rename.sh::t07_memory_module_untouched`

## 3. Edge cases

- A consumer repo that already ran `npx cyberos install` before this ships has an installed `.cyberos/` machine with no dependency on the bin name (the machine itself never shells out to `cyberos`/`cs` internally) - re-running install under the new bin name MUST behave identically to a fresh install.
- `bash dist/*/help.sh` and `bash dist/*/install.sh` (invoked directly, not through the npm bin) are unaffected by the bin rename - only the npm-published entry point's name changes, not the underlying shell scripts' own invocation.
- A user who globally installed an earlier version of `@cyberskill/cyberos` (bin `cyberos`) and upgrades to the version carrying this rename: whether npm cleanly removes the now-undeclared `cyberos` bin symlink or leaves it dangling is version/install-method dependent and NOT independently verified by this task - this is exactly the gap TASK-IMP-134's clean-machine end-to-end regression exists to observe directly, rather than this task asserting a specific npm bin-symlink behaviour it has not verified. Either outcome is the accepted breaking-change cost the plan (§7) already recorded; only the precise mechanics are unverified here.
- The Grok/Claude plugin channel (`grok plugin install`, `claude plugin install cyberos@cyberos`) is a **separate** name from the npm bin - the plugin identifier `cyberos@cyberos` (`tools/install/README.md:121,123,136`) is the Claude/Grok marketplace+plugin name, unaffected by this task; renaming it is out of scope and not implied by 1.1-1.6.
- Security-class: this task only changes which string a package.json/shell script prints and which key an object uses; it grants no new filesystem or network capability and narrows nothing that needs a security review beyond the standard release-pipeline retest called out in Dependencies.
