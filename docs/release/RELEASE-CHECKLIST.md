# Release-readiness checklist - CyberOS 1.0.0

`VERSION` says 1.0.0; this document defines what "ready to tag `v1.0.0`" means. It is a living,
tracked operator document: work every line to `checked` (or `waived` with a reason) before the tag.
Nothing in this file authorizes tagging, publishing, or pushing - the release gate is operator-held.

**Row contract.** Every checklist row carries: `#` (stable id), `Line`, `Owner` (`operator` =
human-only act, `agent` = machine-executable), `State` from the closed set {`open`, `checked`,
`waived`}, and `Evidence / command`. A `waived` state REQUIRES its reason in the Evidence cell -
an empty waiver is a structure violation. Machine lines name their command verbatim; human lines
say what evidence satisfies them. No credentials belong in this file.

**Cross-links.**

- Seed source: `../IMPROVEMENT_HANDOFF.md` - sibling of this repo checkout
  (`~/Projects/CyberSkill/IMPROVEMENT_HANDOFF.md`, deliberately not tracked in-repo). Its IMP-15
  defines the seven seed lines below (tagged `IMP-15.1`..`IMP-15.7`); its §5 is the evidence index
  and the 2026-07-16 channel research with source URLs.
- Batch-1 pre-work commits: `feff8cef`, `a882e705`, `81ac11a3` (handoff §1 applied table).
- Batch-1 governed run: `27292774..ca9ae490` - TASK-IMP-082/083/084 authored -> done; member
  suites 6/6, 13/13, 8/8; 19/19 repo suites; both HITL gates human-recorded per task.
- Batch-2 (in flight on `batch/2-workflow-helpers`): `e9cfb97a` - TASK-IMP-085/086/087 authored,
  machine-floor linted, audited ready_to_implement.
- Decision record (2026-07-16, batch-2 PLAN gate):
  `docs/tasks/.workflow/task-author.improvement-batch-2.manifest.json` (open questions Q2/Q3/Q4 =
  IMP-06/07/11) plus the `source_decisions` block in each batch-2 spec.

## (a) Code readiness

| # | Line | Owner | State | Evidence / command |
|---|---|---|---|---|
| A1 | IMP-15.1 rollup: every S1 handoff item landed or explicitly waived (waiver reason on its own row) - rows A2-A5 plus E1 | operator | checked | 2026-07-19: operator confirmed. Precondition met - A2-A5 and E1 all checked with evidence, and all 14 agent-owned rows are now checked. No S1 handoff item outstanding; no waiver rows required. |
| A2 | IMP-01 status-page stamp byte-stable (fp- corpus content fingerprint) | agent | checked | landed as TASK-IMP-082 in `27292774..ca9ae490`; suite `scripts/tests/test_render_stamp.sh` 6/6 |
| A3 | IMP-02 hooksPath-aware status hook install/uninstall (plus two uninstall defects fixed) | agent | checked | landed as TASK-IMP-083 in `27292774..ca9ae490`; suite `tools/install/tests/test_install_hygiene.sh` 13/13 |
| A4 | IMP-03 task-lint machine floor wired into task-audit | agent | checked | landed as TASK-IMP-084 in `27292774..ca9ae490`; suite `tools/install/tests/test_task_lint.sh` 8/8; first governed use caught a TRACE-001 in batch-2 authoring (`e9cfb97a`) |
| A5 | IMP-04 doc-driven helpers: ship-manifest writer + backlog mutator | agent | checked | landed as TASK-IMP-085 in `e9cfb97a..6ccafb6c` (batch 2, merged `24ce56e9`); `tools/install/tests/test_workflow_helpers.sh` 12/12; both CLIs dogfooded on their own batch |
| A6 | Payload trio green at the release commit | agent | checked | 2026-07-19 macOS host (bash 3.2): `build.sh` OK (chain 27 referenced / 56 vendored, parity OK, skills=56, plugin zip 1.1MB); `check-version-sync dist/cyberos` -> `sync OK 1.0.0 across 7 artifacts`; `run_all.sh` -> `suites: pass=36 fail=0 skip=1`, verified by the pre-commit hook at commit 6d91eefe. Getting there required fixing five macOS test-portability defects (bash-3.2 unparseable heredoc, GNU-only `sed -i` x2 suites, bash-4 `$BASHPID`, logical-vs-physical tmp path) - all in TEST HARNESSES; shipped payload code carries none of them. |
## (b) Artifact readiness

| # | Line | Owner | State | Evidence / command |
|---|---|---|---|---|
| B1 | IMP-15.2a npm pack dry-run of the payload package `@cyberskill/cyberos` | agent | checked | 2026-07-19: `cd dist/cyberos && npm pack --dry-run` -> `@cyberskill/cyberos@1.0.0`, `bin cyberos -> cli/bin/cli.mjs`, 1530 files, 768.4 kB packed / 5.2 MB unpacked; all 21 `package.json` `files[]` entries verified present in the tarball listing. |
| B2 | IMP-15.2b npx smoke on a scratch repo | agent | checked | 2026-07-19: `npm pack` -> `cyberskill-cyberos-1.0.0.tgz` (768386 B). In a fresh `git init -b main` scratch repo: help printed `CyberOS 1.0.0` usage, and install laid down `.cyberos/` (16 entries) plus the agent channel dirs (.agents .claude .codex .cursor .devin .github ...). CAVEAT: the literal `npx --yes <path-to-tgz> help` in this cell does NOT work on npm 11 - npx tries to exec the path (`Permission denied`). Working forms: `npx --yes file:<tgz> help` or `npx --yes --package=<tgz> cyberos help`. Row command should be updated. |
| B3 | IMP-15.3 plugin zip loaded into a live session | operator | checked | 2026-07-19 COMPLETE - all three commands reached their first gate, driven via the Claude Code CLI against `~/Projects/CyberSkill/sachviet`. PLUGIN: registers correctly (Plugins menu; manifest name=cyberos version=1.0.0; 8 commands + 57 skills; descriptions render from our frontmatter). `/install`: end-to-end pass - machine, gates.env, backlog seed, CHANGELOG, status page, pre-commit hook, agent surface, BRAIN, managed .gitignore all landed, and the installed rules_sha fbbc899a...3b00052 matches the shipped payload exactly. `/create-tasks`: halted at PLAN with 2 blocking HITL questions (target-repo binding, author identity); on approval authored 5 specs, each audited 6/10 pre-revision -> 10/10 post, 5 rows inserted via backlog-mutate, task-lint clean. The audit caught 5 would-have-failed defects of the TRACE-006 class (AC weaker than its clause verb) on specs it had just written. `ship-tasks`: ran batch-select as step -1, correctly excluded TASK-SEC-001 by reading depends_on from frontmatter rather than the backlog index, excluded 3 more on cone conflict, then halted before step 1 under EXECUTION-DISCIPLINE §2 condition 1 (repo is docs-only, nothing to implement into). Operator parked all five on_hold; queue drained to eligible: []. ROW WORDING CORRECTED: `ship-tasks` ships as a SKILL, not a command - there is no `/ship-tasks` slash command; invoke the skill by name. HOST DEFECT (not ours, no payload change): the FIRST slash command in a fresh Cowork task is dropped with `Unknown command: /<plugin>:<cmd>`; the 2nd and later dispatch normally. Proven across 2 tasks x 3 commands. Workaround: send any message first. SURFACE LIMIT: Cowork runs in an isolated cloud sandbox with no local filesystem access, so /install cannot reach a local repo there. It failed closed and diagnosed this correctly. The CLI is the surface where install works - this row was completed there. |
| B4 | IMP-15.5 release.yml tag-flow dry-run | operator | checked | 2026-07-20 - operator moved the v1.0.0 tag to current main and dispatched release.yml. Run 29705961028 = success on sha 9355f7cb; all 9 jobs green (payload, npm, docs, channels, ios, android, desktop x3). VERIFIED INDEPENDENTLY: the tag now resolves to 9355f7cb, byte-equal to origin/main, so the stale-tag hazard is closed - assets were rebuilt from today's hardened code (uploaded 22:20:27Z) rather than the 2026-07-16 commit the tag previously named. `cyberos.plugin` is 1149156 bytes on the release, byte-identical to the locally built artifact, which demonstrates the payload is reproducible across machines - not previously proven. The payload job ran the Tag-equals-VERSION guard, build.sh, check-version-sync (TASK-IMP-068), release-assets.sh and the idempotent create-or-upload. The channels job did real end-to-end work (docker channel installed against a real repo; github-action entry point resolved and ran) and npm published @cyberskill/cyberos via OIDC. Release remains a DRAFT; publishing is a separate act. CORRECTION TO EARLIER NOTES IN THIS ROW'S HISTORY: `test_release_assets.sh` did NOT run. The payload job runs `release-assets.sh` (the producer), not the test. No workflow runs `run_all.sh` or that suite, and it self-skips on macOS for lack of GNU tar - so it has never executed on any machine. That is a CI gap, not a payload defect, and is filed in POST-1.0.0-IMPROVEMENT-BACKLOG.md. |
## (c) Channel readiness

| # | Line | Owner | State | Evidence / command |
|---|---|---|---|---|
| C1 | IMP-15.4 re-verify the channel matrix below against current tool conventions BEFORE the tag - the matrix is research dated 2026-07-16 and conventions moved twice in H1 2026; do not trust it at tag time without a fresh pass | operator | checked | 2026-07-19 fresh research pass (row demanded one; prior research was 2026-07-16, only 3 days old, so this is largely confirmatory). Verified against docs.devin.ai, cursor.com/docs, agents.md, developers.openai.com/codex/skills, codex.danielvaughan.com and secondary 2026 guides. CHANGED ROWS: (1) Cursor - `.cursorrules` is legacy and silently ignored in Cursor Agent mode; `.cursor/rules/*.mdc` is the live channel (we ship both, so we are covered, but the shim must not be relied on). (2) `.agents/skills/` - promoted from `candidate` to shipped; Agent Skills is now a ratified open standard under the Linux Foundation AAIF. (3) Devin Desktop - promoted from `candidate` to shipped; rebrand confirmed 2026-06-02 and the row's precedence/fallback description is exactly correct. CONFIRMED UNCHANGED: AGENTS.md (now LF/AAIF-stewarded, 28+ tools), CLAUDE.md, GEMINI.md, `.github/copilot-instructions.md`, `.grok/GROK.md`. NOT INDEPENDENTLY VERIFIED this pass: the `.agents/rules/cyberos.md` open-rules-dir row - I found no authoritative source for `.agents/rules/` as a convention distinct from `.agents/skills/`. It is harmless (create-if-absent) but should not be claimed as a standard until someone confirms it. |
Channel matrix (agent surface per tool, researched 2026-07-16; RE-VERIFIED 2026-07-19 per C1):

| Agent / tool | Surface file(s) | Status |
|---|---|---|
| Cross-agent spine | `AGENTS.md` (thin pointer to `.cyberos/AGENT-ENTRY.md`) | shipped - CONFIRMED 2026-07-19: AGENTS.md is stewarded by the Linux Foundation Agentic AI Foundation, ships in 28+ tools and 60k+ repos, and is now read natively by Claude Code too (CLAUDE.md remains its richer format). The spine choice is stronger than when researched |
| Claude Code / Cowork | `CLAUDE.md` | shipped |
| Gemini CLI | `GEMINI.md` | shipped |
| Cursor | `.cursorrules` + `.cursor/rules/cyberos.mdc` | shipped - CHANGED 2026-07-19: `.cursorrules` is legacy and is SILENTLY IGNORED in Cursor Agent mode; `.cursor/rules/*.mdc` is the live channel. We ship both, so agentic use is covered - treat `.cursorrules` as a back-compat shim only, not a channel to rely on |
| Grok CLI | `.grok/GROK.md` | shipped |
| GitHub Copilot | `.github/copilot-instructions.md` | shipped |
| Open rules dir | `.agents/rules/cyberos.md` | shipped |
| Shared skills dir - Agent Skills open standard (read by Codex, Copilot, Cursor, Gemini CLI, OpenCode) | `.agents/skills/` | shipped (E3 checked, TASK-IMP-094) - CHANGED 2026-07-19: Agent Skills is now a ratified open standard governed by the Linux Foundation Agentic AI Foundation; Codex scans `.agents/skills` from cwd up to repo root; portable across 30+ agent platforms |
| Devin Desktop (Windsurf rebrand, June 2026) | `.devin/rules/` preferred, `.windsurf/rules/` fallback; legacy `.windsurfrules` still read, kept | shipped (E3 checked, TASK-IMP-094) - CONFIRMED 2026-07-19: rebrand landed 2026-06-02; `.devin/rules/` takes precedence, `.windsurf/rules/` remains a fallback, `.windsurfrules` is still read and there is no `.devinrules` single-file equivalent. Row is exactly right |

MCP registrations (`.mcp.json`, `.cursor/mcp.json`) ride the same install step; B2's scratch
install exercises them.

## (d) Docs readiness

| # | Line | Owner | State | Evidence / command |
|---|---|---|---|---|
| D1 | IMP-15.6 CHANGELOG 1.0.0 release section final | operator | checked | 2026-07-19: operator chose option (a) - the full shipped set, thematic. `CHANGELOG.md` `[1.0.0]` now carries a `Hardening - pre-1.0.0 improvement batches (2026-07-16 .. 2026-07-19)` block grouped into seven themes (determinism/provenance, install-uninstall correctness, audit and gate rigour, workflow doctrine, authoring/templates, release and reporting, test-suite portability), naming the shipped improvement tasks inline. Covers 082..092 per the handoff resume plan AND the row's 082..087, plus the rest of the 41 improvement tasks that are `done` and postdate the section's 2026-07-14 date. Row condition - 'the section names the batch tasks' - satisfied. |
| D2 | GUIDE pass | operator | checked | 2026-07-19: read end-to-end. ONE REAL GAP FOUND AND FIXED - the CLI ships 8 commands but the GUIDE documented only 5; `create`, `gates` and `mcp` were undocumented (their apparent mentions were false matches: `create` only inside `/create-tasks`, `gates` as the `.cyberos/gates/` concept, `mcp` in a list of update-check triggers). Fixed in the source `tools/install/docs/index.md`: three table rows added with correct shell forms (`create.sh [dir]`, `.cyberos/cuo/gates/run-gates.sh [repo]`, `mcp/cyberos-mcp.mjs`), each marked as having no slash command, plus a note on the `npx cyberos <command>` surface and the additionally-shipped `/plan` and `/improve`. Rebuilt: `dist/cyberos/GUIDE.md` is byte-identical to source and carries the new rows. MACHINE HALF RE-RUN per this row: build.sh OK (skills=56, plugin_zip=1149156), `sync OK 1.0.0 across 7 artifacts`, suite `pass=36 fail=0 skip=1` - unchanged by the edit. THREE FALSE ALARMS DISMISSED on inspection: `ship-tasks.md` exists at `plugin/skills/ship-tasks/cuo/ship-tasks.md` (GUIDE cites the post-install path); `gates.env` is generated at install time, not shipped; `docs/CONSUMER_UPDATE.md` is explicitly labelled '(in the monorepo / pack tools)'. No stale version strings and no npx claims in GUIDE, so the npm-11 npx caveat from B2 does not affect it. |
| D3 | IMP-15.7 fresh-clone consumer test | agent | checked | 2026-07-19: fresh clone of sachviet at branch `batch/1-web-workspace` (HEAD a6cb4d9) per recorded decision D3 (main does not yet carry the workspace). `npm ci` rc=0 (150 packages), `npm run coverage` rc=0 -> 4 test files, **22/22 tests passed**, coverage 100% stmts/branch/funcs/lines - matches the batch-1 baseline exactly. NOTE: a first attempt failed with `vitest: command not found` purely because the runner shell had NODE_ENV=production (npm omit=dev), which skips devDependencies; re-run with NODE_ENV unset. Not a repo or payload defect. |
## (e) Decided items (operator decisions 2026-07-16, batch-2 PLAN gate)

| # | Line | Owner | State | Evidence / command |
|---|---|---|---|---|
| E1 | IMP-06 decision: scaffold `task_template: task@1` in consumer `config.yaml` on install, so the task-author chain resolves task@1 with its source named | agent | checked | landed as TASK-IMP-088 in `7b593daf` (batch 3); consumer installs scaffold `task_template: task@1` live, platform repo keeps the comment; `tools/install/tests/test_install_hygiene.sh` 17/17 incl. t06 x3; live scratch install -> `.cyberos/config.yaml:10 task_template: task@1` |
| E2 | IMP-07 decision: drop `## 4. Out of scope / non-goals` from TASK-TEMPLATE.md (and the contracts source it mirrors); template schema test updated | agent | checked | landed as TASK-IMP-089 in `7b593daf` (batch 3); template drops section 4, invariants renumbered to `## 4.`; `scripts/tests/test_template_schema.sh` 10/10 incl. t08 x3 (shape oracle, reintroduction canary, payload byte-parity) |
| E3 | IMP-08 channel implementation: create-if-absent `.agents/skills/ship-tasks` symlink plus `.devin/rules/cyberos.md` + `.windsurf/rules/cyberos.md` pointers, keep legacy `.windsurfrules`; extend the gitignore block and uninstall accordingly | agent | checked | landed as TASK-IMP-094 in `1f575c3d` (batch 4): `.agents/skills/{ship-tasks,task-author,task-audit}` relative symlinks (copy fallback) + `.devin/rules/cyberos.md` + `.windsurf/rules/cyberos.md` pointers, legacy `.windsurfrules` kept; gitignore + uninstall extended; `tools/install/tests/test_channels.sh` 24/24, hygiene 19/19; C1's pre-tag matrix re-verify remains operator-owned |
| E4 | IMP-11 decision: task-author manifests are untracked session state - default `manifest_path` to `docs/tasks/.workflow/` and extend `.workflow/.gitignore`; document the policy in the skill | agent | checked | landed as TASK-IMP-090 in `7b593daf` (batch 3); author manifests default to `docs/tasks/.workflow/`, seed covers `*.manifest.json` (append-once), three manifests left the index, approval record at `docs/tasks/_audits/IMPROVEMENT-BATCHES-2026-07-16.md`; hygiene t07 green |
| E5 | IMP-17 regen_backlog emits every status + recomputes Totals from frontmatter (the drift root cause TASK-IMP-086 backfilled) | agent | checked | landed as TASK-IMP-091 in `7b593daf` (batch 3); new suite `scripts/tests/test_regen_backlog.sh` 3/3 - t01 byte-compares the regenerated section against `git show HEAD:docs/tasks/BACKLOG.md`; unparseable frontmatter now halts before any write |
| E6 | IMP-18 lost-update hardening: headers retally from rows; one-writer-one-view + committed-object evidence in ship-tasks doctrine (v2.6.3) | agent | checked | landed as TASK-IMP-092 in `7b593daf` (batch 3, p0); `tools/install/tests/test_workflow_helpers.sh` 12/12 incl. t10-t12; retally dogfooded through 20 of batch 3's own mutations; closes the enablers of the 2026-07-16 TASK-IMP-086 evidence incident (its corrective addendum: `19752e64`) |

---

Maintained by the operator. Rows flip by editing this file with the Evidence cell filled; after
batch 3 lands A5/E1-E4, flip those rows to `checked` citing their commits. TASK-IMP-087's
acceptance verifies this document's shape, not its final states - working the lines IS the release.
