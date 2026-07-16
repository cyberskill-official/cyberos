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
| A1 | IMP-15.1 rollup: every S1 handoff item landed or explicitly waived (waiver reason on its own row) - rows A2-A5 plus E1 | operator | open | flips when A5 and E1 leave `open`; waiver verdicts are human and land per-row |
| A2 | IMP-01 status-page stamp byte-stable (fp- corpus content fingerprint) | agent | checked | landed as TASK-IMP-082 in `27292774..ca9ae490`; suite `scripts/tests/test_render_stamp.sh` 6/6 |
| A3 | IMP-02 hooksPath-aware status hook install/uninstall (plus two uninstall defects fixed) | agent | checked | landed as TASK-IMP-083 in `27292774..ca9ae490`; suite `tools/install/tests/test_install_hygiene.sh` 13/13 |
| A4 | IMP-03 task-lint machine floor wired into task-audit | agent | checked | landed as TASK-IMP-084 in `27292774..ca9ae490`; suite `tools/install/tests/test_task_lint.sh` 8/8; first governed use caught a TRACE-001 in batch-2 authoring (`e9cfb97a`) |
| A5 | IMP-04 doc-driven helpers: ship-manifest writer + backlog mutator | agent | open | in flight as TASK-IMP-085 on `batch/2-workflow-helpers` - `docs/tasks/improvement/TASK-IMP-085-workflow-helpers/spec.md`; flips with its landing commit + suite |
| A6 | Payload trio green at the release commit | agent | open | `bash tools/install/build.sh && bash tools/install/check-version-sync.sh dist/cyberos && bash scripts/tests/run_all.sh` - sync OK across 7 artifacts, all suites pass |

## (b) Artifact readiness

| # | Line | Owner | State | Evidence / command |
|---|---|---|---|---|
| B1 | IMP-15.2a npm pack dry-run of the payload package `@cyberskill/cyberos` | agent | open | `cd dist/cyberos && npm pack --dry-run` - manifest shows `@cyberskill/cyberos@1.0.0`, bin `cyberos -> cli/bin/cli.mjs`, and every entry of the package.json `files` list present |
| B2 | IMP-15.2b npx smoke on a scratch repo | agent | open | `cd dist/cyberos && npm pack` (produces `cyberskill-cyberos-1.0.0.tgz`), then in a scratch repo after `git init -b main`: `npx --yes <path-to-tgz> help && npx --yes <path-to-tgz> install` - help prints, install lays down `.cyberos/` |
| B3 | IMP-15.3 plugin zip loaded into a live session | operator | open | human evidence: `dist/cyberos/cyberos.plugin` (the release asset) installed into a real Claude Code or Cowork session; `/install`, `/create-tasks`, `/ship-tasks` each triggered once and reaching their first gate; record session date + observations in this cell |
| B4 | IMP-15.5 release.yml tag-flow dry-run | operator | open | `gh workflow run release.yml` (the `workflow_dispatch` path of `.github/workflows/release.yml`; real releases fire on `v*` tags) - verify payload + plugin assets attach to the draft release; link the run in this cell |

## (c) Channel readiness

| # | Line | Owner | State | Evidence / command |
|---|---|---|---|---|
| C1 | IMP-15.4 re-verify the channel matrix below against current tool conventions BEFORE the tag - the matrix is research dated 2026-07-16 and conventions moved twice in H1 2026; do not trust it at tag time without a fresh pass | operator | open | a fresh research pass recorded in this cell with its date and any changed rows; current sources: IMPROVEMENT_HANDOFF.md §5 (docs.windsurf.com, thepromptshelf.dev, opencode.ai/docs/skills, developers.openai.com/codex/skills, codex.danielvaughan.com) |

Channel matrix (agent surface per tool, researched 2026-07-16):

| Agent / tool | Surface file(s) | Status |
|---|---|---|
| Cross-agent spine | `AGENTS.md` (thin pointer to `.cyberos/AGENT-ENTRY.md`) | shipped |
| Claude Code / Cowork | `CLAUDE.md` | shipped |
| Gemini CLI | `GEMINI.md` | shipped |
| Cursor | `.cursorrules` + `.cursor/rules/cyberos.mdc` | shipped |
| Grok CLI | `.grok/GROK.md` | shipped |
| GitHub Copilot | `.github/copilot-instructions.md` | shipped |
| Open rules dir | `.agents/rules/cyberos.md` | shipped |
| Shared skills dir - Agent Skills open standard (read by Codex, Copilot, Cursor, Gemini CLI, OpenCode) | `.agents/skills/` | candidate - line E3 |
| Devin Desktop (Windsurf rebrand, June 2026) | `.devin/rules/` preferred, `.windsurf/rules/` fallback; legacy `.windsurfrules` still read, kept | candidate - line E3 |

MCP registrations (`.mcp.json`, `.cursor/mcp.json`) ride the same install step; B2's scratch
install exercises them.

## (d) Docs readiness

| # | Line | Owner | State | Evidence / command |
|---|---|---|---|---|
| D1 | IMP-15.6 CHANGELOG 1.0.0 release section final | operator | open | `CHANGELOG.md` head carries `## [1.0.0]` (currently dated 2026-07-14) but predates improvement batches 1-2; extend it to cover TASK-IMP-082..087 before the tag - satisfied when the section names the batch tasks |
| D2 | GUIDE pass | operator | open | read `dist/cyberos/GUIDE.md` end-to-end for 1.0.0 truth (source `tools/install/docs/index.md`, ships via `build.sh` line 195); after any edit, the machine half is the A6 trio |
| D3 | IMP-15.7 fresh-clone consumer test | agent | open | `git clone sachviet && npm ci && npm run coverage` - the reference consumer repo (`~/Projects/CyberSkill/sachviet`); gates GREEN (batch-1 baseline: 22/22 tests, 100 percent coverage on touched files) |

## (e) Decided items (operator decisions 2026-07-16, batch-2 PLAN gate)

| # | Line | Owner | State | Evidence / command |
|---|---|---|---|---|
| E1 | IMP-06 decision: scaffold `task_template: task@1` in consumer `config.yaml` on install, so the task-author chain resolves task@1 with its source named | agent | open | decision recorded 2026-07-16 at the batch-2 PLAN gate (manifest Q2 + spec `source_decisions`); implement pre-tag as a batch-3 task, or defer past 1.0.0 by operator note in this cell |
| E2 | IMP-07 decision: drop `## 4. Out of scope / non-goals` from TASK-TEMPLATE.md (and the contracts source it mirrors); template schema test updated | agent | open | decision recorded 2026-07-16 at the same gate (manifest Q3); same flip rule as E1 |
| E3 | IMP-08 channel implementation: create-if-absent `.agents/skills/ship-tasks` symlink plus `.devin/rules/cyberos.md` + `.windsurf/rules/cyberos.md` pointers, keep legacy `.windsurfrules`; extend the gitignore block and uninstall accordingly | agent | open | handoff IMP-08 ([auto], scheduled at the same gate) + the candidate matrix rows above; flips with its landing commit + `test_channels.sh` / `test_install_hygiene.sh` extensions |
| E4 | IMP-11 decision: task-author manifests are untracked session state - default `manifest_path` to `docs/tasks/.workflow/` and extend `.workflow/.gitignore`; document the policy in the skill | agent | open | decision recorded 2026-07-16 at the same gate (manifest Q4); same flip rule as E1 |

---

Maintained by the operator. Rows flip by editing this file with the Evidence cell filled; after
batch 3 lands A5/E1-E4, flip those rows to `checked` citing their commits. TASK-IMP-087's
acceptance verifies this document's shape, not its final states - working the lines IS the release.
