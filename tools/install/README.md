# cyberos pack - run ship-tasks in any repo

This is the portable form of the single `ship-tasks` workflow. It runs in any repo, any language, with no CyberOS clone required. It is exposed through many channels so a user can pick whatever fits their setup.

New to it? `GUIDE.md` is the step-by-step walkthrough (zero to your first shipped task). This README is the channel catalog and reference.

`install` sets up two things by default: the task workflow AND the BRAIN memory protocol. It scaffolds a local `.cyberos/memory/store/` store (gitignored tenant data) and drops the `AGENTS.md` Layer-1 memory rules, so the project gets both the workflow and the memory discipline. Skip the memory half with `CYBEROS_NO_MEMORY=1`.

`install` also runs the task migration automatically (skip with `CYBEROS_NO_MIGRATE=1`): pre-existing tasks move to the folder-per-task layout (root-level flat tasks included - module comes from frontmatter `module:`, else the task id segment), a `CHANGELOG.md` is seeded once if the repo has none, and the status page (Roadmap | Backlog | Changelog tabs) is (re)generated at `docs/status/` - a folder holding `index.html` plus `assets/` (stylesheet + favicon), titled after the target repo (never "CyberOS" for someone else's project). That one page REPLACES the old standalone documents: a pre-existing `docs/BACKLOG.md` is adopted into `docs/tasks/BACKLOG.md` and a `docs/CHANGELOG.md` into the root `CHANGELOG.md` (content preserved, each only when the canonical home is empty), then any remaining `docs/ROADMAP.md` / `docs/BACKLOG.md` / `docs/CHANGELOG.md` is REMOVED - the page is those documents now, and git history keeps the old text. The migration ends with a machine-readable verify line (`cyberos-migrate verify: task_specs=N flat_task_files_remaining=0 task_folders_missing_spec=0 status_page=present`) and WARNs for anything it could not place.

The page stays synced with the markdown it renders - markdown remains the record of truth, the page only renders it. Two auto-sync touchpoints: a managed `pre-commit` hook (installed only when no foreign hook exists; never blocks a commit; `CYBEROS_NO_HOOK=1` skips) regenerates `docs/status/` whenever `docs/tasks/**`, `CHANGELOG.md`, or `VERSION` is staged, and `run-gates.sh` regenerates it after every gates run. Manual refresh any time: `bash .cyberos/lib/status-page.sh`.

## What init writes: tracked vs gitignored

One managed block in `.gitignore` (between `# >>> cyberos ... >>>` and `# <<< cyberos <<<` markers) carries every ignore rule init needs. Re-running init regenerates the block in place - it never appends duplicates and never touches anything outside the markers, so your own rules survive. Legacy entries appended by older inits are lifted into the block on first contact.

| Artifact                                                                                            | Fate        | Why                                                          |
| --------------------------------------------------------------------------------------------------- | ----------- | ------------------------------------------------------------ |
| `.cyberos/` (machine, gates.env, config.yaml, BRAIN store, render intermediates, migration kit)    | gitignored  | regenerable via init; BRAIN is local tenant data              |
| `docs/status/` (index.html + assets/ - the generated Roadmap / Backlog / Changelog page)           | tracked     | the repo's published status view; replaces standalone docs    |
| `.git/hooks/pre-commit` (cyberos-status-hook, when no foreign hook exists)                          | local       | auto-regenerates docs/status/ when its inputs are committed   |
| skill symlinks (`.claude/skills/ship-tasks`, ...)                                        | gitignored  | they point INTO the ignored `.cyberos/`                     |
| skill copies (`CYBEROS_COPY_SKILLS=1`)                                                              | tracked     | self-contained; commit them                                   |
| `AGENTS.md` + pointer files (`CLAUDE.md`, `GEMINI.md`, `.cursorrules`, `.grok/GROK.md`, ...) | tracked     | teammates' agents need them                                   |
| `.mcp.json`, `.cursor/mcp.json`                                                                   | tracked     | project MCP registration for the whole team                   |
| `docs/tasks/**` (BACKLOG.md, specs, audits)                                              | tracked     | the work record itself                                        |
| `docs/tasks/.workflow/*.ship.json`                                                        | gitignored  | run state (nested .gitignore, TASK-CUO-206)                     |
| `CHANGELOG.md`                                                                                      | tracked     | release history; feeds the status page's Changelog tab        |

Existing files are never clobbered: `BACKLOG.md`, `CHANGELOG.md`, `config.yaml`, pointer files, and `.mcp.json` are create-if-absent; `AGENTS.md` gets a marked append at most once; `gates.env` is regenerated with a timestamped backup; the vendored machine (`.cyberos/cuo|plugin|mcp`) is replaced wholesale on every init (that is the update path).

## What it is

Two layers:

- The machine (doc-driven, always works): three normative docs - `ship-tasks.md`, `EXECUTION-DISCIPLINE.md`, `STATUS-REFERENCE.md`. An agent (Claude Code, Cowork, Codex) given these drives a task through implement -> review -> test -> done.
- The gates (repo-local): a runner that shells out to the target repo's own build/lint/test/coverage. Full deterministic gates (caf, awh) vendor in only if the source repo has them; otherwise the reduced-profile floor applies.

Two rules hold on every channel:

- HITL is required. The agent halts at review acceptance (`reviewing -> ready_to_test`) and final acceptance (`testing -> done`) for a recorded human verdict, and never sets `done` itself.
- Improvement is not separate. Hardening/refactor/audit work is a task with `class: improvement`; same lifecycle.

Profiles: `reduced` = doc-driven + the repo's own build/lint/test + coverage + code review + the two human gates (works anywhere). `full` = reduced plus the vendored caf/awh deterministic gates (when the pack was built from a repo that has them). `manifest.yaml` in a built pack records which you have.

## Agent support

`AGENTS.md` (repo root) is the canonical, cross-tool spine - the one file the most agents read natively. `install.sh` writes it, then layers each agent's own preferred file / native skill / MCP registration on top. Everything is create-if-absent: your existing files are never clobbered. One `install.sh` run wires them all.

| Agent        | Reads (instruction file)                                 | Native skill dir                     | MCP                         |
| ------------ | -------------------------------------------------------- | ------------------------------------ | --------------------------- |
| Claude Code  | `CLAUDE.md` (+ `AGENTS.md`)                          | `.claude/skills/`                  | `.mcp.json` (auto)        |
| Codex        | `AGENTS.md`                                            | `.codex/skills/`                   | `~/.codex/config.toml`    |
| Cursor       | `AGENTS.md`, `.cursor/rules/*.mdc`, `.cursorrules` | -                                    | `.cursor/mcp.json` (auto) |
| Gemini CLI   | `GEMINI.md` (+ `AGENTS.md`)                          | -                                    | yes                         |
| Antigravity  | `AGENTS.md` + `GEMINI.md`, `.agents/rules/`        | -                                    | yes                         |
| Grok CLI     | `AGENTS.md`, `.grok/GROK.md`                         | `.grok/skills/`                    | yes                         |
| zcode        | `AGENTS.md`                                            | global (`CYBEROS_GLOBAL_SKILLS=1`) | yes                         |
| Command Code | `AGENTS.md` (+ `CLAUDE.md`)                          | `.commandcode/skills/`             | `/mcp`                    |
| Hermes       | `AGENTS.md`                                            | global (`CYBEROS_GLOBAL_SKILLS=1`) | gateway                     |
| Copilot      | `.github/copilot-instructions.md` (+ `AGENTS.md`)    | -                                    | -                           |
| Windsurf     | `.windsurfrules` (+ `AGENTS.md`)                     | -                                    | yes                         |

Controls: `CYBEROS_AGENTS=claude-code,codex,...` restricts the set; `CYBEROS_COPY_SKILLS=1` copies skills instead of symlinking (committable, self-contained); `CYBEROS_GLOBAL_SKILLS=1` also installs into `$HOME` agent dirs; `CYBEROS_NO_MCP=1` skips `.mcp.json`. Adding an agent is a one-line `pointer`/`install_skill` entry in `install.sh` - see "Add your own agent" below.

## Build the pack

From a CyberOS checkout:

```bash
bash tools/install/build.sh          # assembles dist/cyberos/ (self-contained)
```

`dist/cyberos/` is the portable bundle. Everything below consumes it.

## Channels (pick one)

### 1. Copy the folder (available)

```bash
cp -R dist/cyberos /path/to/your/repo/.cyberos-install
cd /path/to/your/repo && bash .cyberos-install/install.sh
```

The copied `.cyberos-install/` removes itself after a successful init - everything the repo needs onward lives under `.cyberos/` (machine, gates, migration kit, MCP server), so the payload copy is redundant and must not end up committed. Keep it with `CYBEROS_KEEP_PAYLOAD=1`. Only the canonical `<repo>/.cyberos-install` self-cleans: payloads outside the repo, other in-repo paths, and git submodules (channel 2) are never removed.

### 2. Git submodule or subtree (available)

Publish `dist/cyberos` as its own repo, then in the target:

```bash
git submodule add <payload-repo-url> .cyberos-install
bash .cyberos-install/install.sh
```

### 2a. What the payload covers (TASK-CUO-209)

The payload vendors the FULL 14-stage SDP skill catalog (52 skills: 24 author/audit
pairs + the four NFR singles) - SOW through decommissioning. The two commands automate
stages 5-10; everything else is standalone-invocable. See the lifecycle map in GUIDE.md.

### 2b. Update awareness (TASK-IMP-070)

`install.sh --check <repo>` reports three values - `installed=`, `payload=`, `latest=` (the newest
published release, resolved by `check-latest.sh` with a 3s budget; `CYBEROS_OFFLINE=1` skips it) -
plus one `verdict=` line (`up_to_date` | `repo_stale` | `payload_stale`) and the exact `next:`
command. Machine-parseable key=value lines; the desktop Ops tab and `/version` consume them.

### 3. One-liner curl | sh (from GitHub Releases - TASK-IMP-069)

```bash
tar -czf cyberos.tar.gz -C dist cyberos     # publish this to a URL
curl -fsSL https://raw.githubusercontent.com/cyberskill-official/cyberos/main/tools/install/bootstrap.sh | bash
# fetches https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz,
# verifies it against the SHA256SUMS asset, and runs install.sh on the current repo.
# Pin a version:  CYBEROS_PAYLOAD_URL=.../releases/download/vX.Y.Z/cyberos-payload-X.Y.Z.tar.gz curl -fsSL .../bootstrap.sh | bash
# Claude desktop/Cowork: download cyberos.plugin from the same release page and pick the file.
# Claude Code: download + unpack cyberos-payload.tar.gz, then /plugin marketplace add <dir>.
# Fleet:  bash tools/install/rollout.sh --from-release [vX.Y.Z] <repo> [<repo>...]
```

### 4. Claude plugin (available)

The payload IS a plugin marketplace: `dist/cyberos/.claude-plugin/marketplace.json` catalogs the plugin at `dist/cyberos/plugin/` (its own manifest at `plugin/.claude-plugin/plugin.json`; the `/install`, `/version`, `/status`, `/help` commands and the `ship-tasks` skill (typeable as `/ship-tasks`, and used automatically when you ask to ship a task)). Install:

- Claude Code: `/plugin marketplace add /path/to/dist/cyberos`, then `/plugin install cyberos@cyberos`.
- Claude desktop / Cowork: the Add picker wants a FILE - use `dist/cyberos/cyberos.plugin` (the one-file bundle build.sh produces; selecting a folder greys the Open button). The folder route works where marketplaces are supported: add `dist/cyberos` as a marketplace (its root carries `.claude-plugin/marketplace.json`).

Then run `/install` in a repo and `/ship-tasks` (or just ask to ship the next task) to drive the backlog; `/version` and `/status` keep the repo current, `/help` orients a new user.

### 4b. Every other agent (Codex, Cursor, Gemini, Antigravity, Grok, zcode, Command Code, Copilot, Windsurf) - agent-independent

The core is doc-driven, so no plugin is required for any agent. `install.sh` writes the canonical `AGENTS.md` spine plus each agent's preferred pointer file (all create-if-absent), and installs the `ship-tasks` skill natively into every skill-aware agent's folder (`.claude/skills`, `.grok/skills`, `.commandcode/skills`, `.codex/skills`, `.opencode/skill`) so it is invocable as `/ship-tasks` or `$ship-tasks`, not just prose. It also drops `.cyberos/AGENT-ENTRY.md`, the one-page canonical trigger. See the Agent support matrix above. Point any file-and-shell agent at `AGENTS.md` (or `.cyberos/AGENT-ENTRY.md`) and it drives the same workflow, the same gates, the same required human verdicts. The Claude plugin is convenience sugar, not a dependency.

### 5. GitHub Action (available)

`dist/cyberos/ci/github-action/action.yml` is a composite action that runs the machine gates in CI. Point a workflow at it after `install.sh` has committed `.cyberos/` to the repo. CI runs the machine gates only; final acceptance stays a human verdict.

### 6. Docker image (available, scaffold only)

```bash
docker build -t cyberos dist/cyberos
docker run --rm -v "$PWD":/work cyberos     # inits the mounted repo
```

The image scaffolds a repo; run the gates on a runner that has your toolchain.

### 7. Makefile / just target (available, two lines)

```make
cyberos:  ; bash /path/to/dist/cyberos/install.sh .
cyberos-gates: ; bash .cyberos/cuo/gates/run-gates.sh
```

### 8. MCP server (available) - any MCP agent, zero files

`dist/cyberos/mcp/cyberos-mcp.mjs` is a zero-dependency Node stdio MCP server exposing `task_init`, `task_gates`, `task_status`, and `ship_task`. Any MCP-capable agent (Codex, zcode, Antigravity, Cursor, Claude Code, Command Code) triggers the workflow tool-natively. `install.sh` vendors it to `.cyberos/mcp/` and writes `.mcp.json` (and `.cursor/mcp.json`) when absent. Registration snippets for every agent: `mcp/README.md`. `ship_task` hands the agent the HITL-gated trigger - it never self-accepts. Quick check:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
              '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | node dist/cyberos/mcp/cyberos-mcp.mjs
```

### 9. npx CLI (available)

The payload root carries a `package.json` with three bins:

```bash
npx cyberos install [dir]     # vendor the machine + wire every agent (default: cwd)
npx cyberos-gates [dir]    # run the machine gates
npx cyberos-mcp            # launch the MCP server (for a client's config)
```

Run `npx .` from `dist/cyberos`, `npm i -g ./dist/cyberos`, or `npx github:<owner>/<repo>` once the payload is published as its own repo.

### 10. Template repo / `create.sh` (available) - fresh projects

```bash
bash dist/cyberos/create.sh ../my-new-project     # git init + skeleton + install.sh
```

`create.sh` seeds `template/` (never clobbering) then runs `install.sh`. Host `template/` as a GitHub template repo ("Use this template") or `degit` it, then run `install.sh` once.

### Planned channels (say the word and I will build them)

- Homebrew tap and Nix flake - `brew install cyberos` / `nix run`.
- Published npm package (`npx cyberos install`). The curl one-liner + hosted payload shipped via GitHub Releases (TASK-IMP-069).

### Add your own agent

Every agent is one data row in `install.sh`. For an instruction pointer file: add `pointer <key> <path> <md|plain|mdc>`. For a native skill install: add `install_skill <skills-dir> <key>`. Both are create-if-absent and honor `CYBEROS_AGENTS`. Because `AGENTS.md` is the spine, most new agents already work with no change at all.

## After install: trigger, gate, sign off

1. Write a task: `cp .cyberos/cuo/templates/task-TEMPLATE.md docs/tasks/TASK-001-<slug>.md`, fill section 1, set `status: ready_to_implement`, add the row to `BACKLOG.md`.
2. Trigger: tell your agent to follow `.cyberos/cuo/ship-tasks.md` and drive the next eligible task, HITL required, `repo_root` = this repo. (Or `/ship-tasks` with the plugin.)
3. Gate: `bash .cyberos/cuo/gates/run-gates.sh`.
4. Sign off: you record the review verdict and the final acceptance. The agent never sets `done`.

## Staying in sync

The pack is a build artifact. When the workflow improves in CyberOS, rebuild (`build.sh`) and re-distribute; consumers re-run `install.sh` (it backs up `gates.env` and never clobbers your BACKLOG).

## Gate autodetection + per-repo config (TASK-CUO-207)

`/install` detects gate commands per stack (union across stacks; first claim per gate wins; a command is
never invented when its marker file is absent - root-only scanning):

| stack  | marker             | build                      | lint                                  | test                                  | coverage                                 |
| ------ | ------------------ | -------------------------- | ------------------------------------- | ------------------------------------- | ---------------------------------------- |
| rust   | Cargo.toml         | cargo build                | clippy                                | cargo test                            | llvm-cov (when installed)                |
| node   | package.json       | scripts.build              | scripts.lint                          | scripts.test / pm test                | scripts.coverage                         |
| python | pyproject/setup    | -                          | ruff (when installed)                 | pytest                                | coverage (when installed)                |
| go     | go.mod             | go build ./...             | go vet (golangci-lint when installed) | go test ./...                         | go test -coverprofile                    |
| maven  | pom.xml            | mvn -q -DskipTests package | -                                     | mvn -q verify                         | - (config only, jacoco is repo-specific) |
| gradle | build.gradle(.kts) | ./gradlew or gradle build  | -                                     | ./gradlew or gradle test              | -                                        |
| dotnet | *.sln / *.csproj   | dotnet build               | -                                     | dotnet test                           | -                                        |
| php    | composer.json      | -                          | composer validate --strict            | vendor/bin/phpunit (when present)     | -                                        |
| ruby   | Gemfile            | -                          | -                                     | rspec (spec/) or rake test (Rakefile) | -                                        |
| make   | Makefile           | make build (per target)    | make lint                             | make test                             | make coverage                            |

Overrides live in `.cyberos/config.yaml` (scaffolded once, all-commented, detected values shown as
comments): `gates.build/lint/test/coverage` (each overrides only its own gate), `coverage_threshold`
(default 90, exported as CYBEROS_COVERAGE_THRESHOLD), `task_template`, `profile`. `run-gates.sh` prints
one provenance line per gate: `gate <name>: <cmd> (source: config|autodetect:<stack>|absent)`.
A malformed config fails loudly with its line number and runs no gate. Unknown keys warn only.
