# cyberos pack - run ship-feature-requests in any repo

This is the portable form of the single `ship-feature-requests` workflow. It runs in any repo, any language, with no CyberOS clone required. It is exposed through many channels so a user can pick whatever fits their setup.

New to it? `GUIDE.md` is the step-by-step walkthrough (zero to your first shipped FR). This README is the channel catalog and reference.

`init` sets up two things by default: the FR workflow AND the BRAIN memory protocol. It scaffolds a local `.cyberos/memory/store/` store (gitignored tenant data) and drops the `AGENTS.md` Layer-1 memory rules, so the project gets both the workflow and the memory discipline. Skip the memory half with `CYBEROS_NO_MEMORY=1`.

## What it is

Two layers:

- The machine (doc-driven, always works): three normative docs - `ship-feature-requests.md`, `EXECUTION-DISCIPLINE.md`, `STATUS-REFERENCE.md`. An agent (Claude Code, Cowork, Codex) given these drives a feature-request through implement -> review -> test -> done.
- The gates (repo-local): a runner that shells out to the target repo's own build/lint/test/coverage. Full deterministic gates (caf, awh) vendor in only if the source repo has them; otherwise the reduced-profile floor applies.

Two rules hold on every channel:

- HITL is required. The agent halts at review acceptance (`reviewing -> ready_to_test`) and final acceptance (`testing -> done`) for a recorded human verdict, and never sets `done` itself.
- Improvement is not separate. Hardening/refactor/audit work is a feature-request with `class: improvement`; same lifecycle.

Profiles: `reduced` = doc-driven + the repo's own build/lint/test + coverage + code review + the two human gates (works anywhere). `full` = reduced plus the vendored caf/awh deterministic gates (when the pack was built from a repo that has them). `manifest.yaml` in a built pack records which you have.

## Agent support

`AGENTS.md` (repo root) is the canonical, cross-tool spine - the one file the most agents read natively. `init.sh` writes it, then layers each agent's own preferred file / native skill / MCP registration on top. Everything is create-if-absent: your existing files are never clobbered. One `init.sh` run wires them all.

| Agent | Reads (instruction file) | Native skill dir | MCP |
| --- | --- | --- | --- |
| Claude Code | `CLAUDE.md` (+ `AGENTS.md`) | `.claude/skills/` | `.mcp.json` (auto) |
| Codex | `AGENTS.md` | `.codex/skills/` | `~/.codex/config.toml` |
| Cursor | `AGENTS.md`, `.cursor/rules/*.mdc`, `.cursorrules` | - | `.cursor/mcp.json` (auto) |
| Gemini CLI | `GEMINI.md` (+ `AGENTS.md`) | - | yes |
| Antigravity | `AGENTS.md` + `GEMINI.md`, `.agents/rules/` | - | yes |
| Grok CLI | `AGENTS.md`, `.grok/GROK.md` | `.grok/skills/` | yes |
| zcode | `AGENTS.md` | global (`CYBEROS_GLOBAL_SKILLS=1`) | yes |
| Command Code | `AGENTS.md` (+ `CLAUDE.md`) | `.commandcode/skills/` | `/mcp` |
| Hermes | `AGENTS.md` | global (`CYBEROS_GLOBAL_SKILLS=1`) | gateway |
| Copilot | `.github/copilot-instructions.md` (+ `AGENTS.md`) | - | - |
| Windsurf | `.windsurfrules` (+ `AGENTS.md`) | - | yes |

Controls: `CYBEROS_AGENTS=claude-code,codex,...` restricts the set; `CYBEROS_COPY_SKILLS=1` copies skills instead of symlinking (committable, self-contained); `CYBEROS_GLOBAL_SKILLS=1` also installs into `$HOME` agent dirs; `CYBEROS_NO_MCP=1` skips `.mcp.json`. Adding an agent is a one-line `pointer`/`install_skill` entry in `init.sh` - see "Add your own agent" below.

## Build the pack

From a CyberOS checkout:

```bash
bash tools/cyberos-init/build.sh          # assembles dist/cyberos/ (self-contained)
```

`dist/cyberos/` is the portable bundle. Everything below consumes it.

## Channels (pick one)

### 1. Copy the folder (available)

```bash
cp -R dist/cyberos /path/to/your/repo/.cyberos-init
cd /path/to/your/repo && bash .cyberos-init/init.sh
```

### 2. Git submodule or subtree (available)

Publish `dist/cyberos` as its own repo, then in the target:

```bash
git submodule add <payload-repo-url> .cyberos-init
bash .cyberos-init/init.sh
```

### 3. One-liner curl | sh (available once you host a tarball)

```bash
tar -czf cyberos.tar.gz -C dist cyberos     # publish this to a URL
CYBEROS_PACK_URL=<url-to-cyberos.tar.gz> curl -fsSL <raw-url>/bootstrap.sh | bash
```

### 4. Claude plugin (available)

The payload IS a plugin marketplace: `dist/cyberos/.claude-plugin/marketplace.json` catalogs the plugin at `dist/cyberos/plugin/` (its own manifest at `plugin/.claude-plugin/plugin.json`; the `/init`, `/update`, `/changelog`, `/help` commands and the `ship-feature-requests` skill (typeable as `/ship-feature-requests`, and used automatically when you ask to ship an FR)). Install:

- Claude Code: `/plugin marketplace add /path/to/dist/cyberos`, then `/plugin install cyberos@cyberos`.
- Claude desktop / Cowork: the Add picker wants a FILE - use `dist/cyberos/cyberos.plugin` (the one-file bundle build.sh produces; selecting a folder greys the Open button). The folder route works where marketplaces are supported: add `dist/cyberos` as a marketplace (its root carries `.claude-plugin/marketplace.json`).

Then run `/init` in a repo and `/ship-feature-requests` (or just ask to ship the next FR) to drive the backlog; `/update` and `/changelog` keep the repo current, `/help` orients a new user.

### 4b. Every other agent (Codex, Cursor, Gemini, Antigravity, Grok, zcode, Command Code, Copilot, Windsurf) - agent-independent

The core is doc-driven, so no plugin is required for any agent. `init.sh` writes the canonical `AGENTS.md` spine plus each agent's preferred pointer file (all create-if-absent), and installs the `ship-feature-requests` skill natively into every skill-aware agent's folder (`.claude/skills`, `.grok/skills`, `.commandcode/skills`, `.codex/skills`, `.opencode/skill`) so it is invocable as `/ship-feature-requests` or `$ship-feature-requests`, not just prose. It also drops `.cyberos/AGENT-ENTRY.md`, the one-page canonical trigger. See the Agent support matrix above. Point any file-and-shell agent at `AGENTS.md` (or `.cyberos/AGENT-ENTRY.md`) and it drives the same workflow, the same gates, the same required human verdicts. The Claude plugin is convenience sugar, not a dependency.

### 5. GitHub Action (available)

`dist/cyberos/ci/github-action/action.yml` is a composite action that runs the machine gates in CI. Point a workflow at it after `init.sh` has committed `.cyberos/` to the repo. CI runs the machine gates only; final acceptance stays a human verdict.

### 6. Docker image (available, scaffold only)

```bash
docker build -t cyberos dist/cyberos
docker run --rm -v "$PWD":/work cyberos     # inits the mounted repo
```

The image scaffolds a repo; run the gates on a runner that has your toolchain.

### 7. Makefile / just target (available, two lines)

```make
cyberos-init:  ; bash /path/to/dist/cyberos/init.sh .
cyberos-gates: ; bash .cyberos/cuo/gates/run-gates.sh
```

### 8. MCP server (available) - any MCP agent, zero files

`dist/cyberos/mcp/cyberos-mcp.mjs` is a zero-dependency Node stdio MCP server exposing `fr_init`, `fr_gates`, `fr_status`, and `ship_fr`. Any MCP-capable agent (Codex, zcode, Antigravity, Cursor, Claude Code, Command Code) triggers the workflow tool-natively. `init.sh` vendors it to `.cyberos/mcp/` and writes `.mcp.json` (and `.cursor/mcp.json`) when absent. Registration snippets for every agent: `mcp/README.md`. `ship_fr` hands the agent the HITL-gated trigger - it never self-accepts. Quick check:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
              '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | node dist/cyberos/mcp/cyberos-mcp.mjs
```

### 9. npx CLI (available)

The payload root carries a `package.json` with three bins:

```bash
npx cyberos-init [dir]     # vendor the machine + wire every agent (default: cwd)
npx cyberos-gates [dir]    # run the machine gates
npx cyberos-mcp            # launch the MCP server (for a client's config)
```

Run `npx .` from `dist/cyberos`, `npm i -g ./dist/cyberos`, or `npx github:<owner>/<repo>` once the payload is published as its own repo.

### 10. Template repo / `create.sh` (available) - fresh projects

```bash
bash dist/cyberos/create.sh ../my-new-project     # git init + skeleton + init.sh
```

`create.sh` seeds `template/` (never clobbering) then runs `init.sh`. Host `template/` as a GitHub template repo ("Use this template") or `degit` it, then run `init.sh` once.

### Planned channels (say the word and I will build them)

- Homebrew tap and Nix flake - `brew install cyberos` / `nix run`.
- Published npm package + hosted `bootstrap.sh` URL (so `npx cyberos-init` and the curl one-liner work without a local checkout).

### Add your own agent

Every agent is one data row in `init.sh`. For an instruction pointer file: add `pointer <key> <path> <md|plain|mdc>`. For a native skill install: add `install_skill <skills-dir> <key>`. Both are create-if-absent and honor `CYBEROS_AGENTS`. Because `AGENTS.md` is the spine, most new agents already work with no change at all.

## After install: trigger, gate, sign off

1. Write an FR: `cp .cyberos/cuo/templates/FR-TEMPLATE.md docs/feature-requests/FR-001-<slug>.md`, fill section 1, set `status: ready_to_implement`, add the row to `BACKLOG.md`.
2. Trigger: tell your agent to follow `.cyberos/cuo/ship-feature-requests.md` and drive the next eligible FR, HITL required, `repo_root` = this repo. (Or `/ship-feature-requests` with the plugin.)
3. Gate: `bash .cyberos/cuo/gates/run-gates.sh`.
4. Sign off: you record the review verdict and the final acceptance. The agent never sets `done`.

## Staying in sync

The pack is a build artifact. When the workflow improves in CyberOS, rebuild (`build.sh`) and re-distribute; consumers re-run `init.sh` (it backs up `gates.env` and never clobbers your BACKLOG).
