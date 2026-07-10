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

The payload IS a plugin marketplace: `dist/cyberos/.claude-plugin/marketplace.json` catalogs the plugin at `dist/cyberos/plugin/` (its own manifest at `plugin/.claude-plugin/plugin.json`; the `/fr-init` command and the `ship-feature-requests` skill (typeable as `/ship-feature-requests`, and used automatically when you ask to ship an FR)). Install:

- Claude Code: `/plugin marketplace add /path/to/dist/cyberos`, then `/plugin install cyberos@cyberos`.
- Claude desktop / Cowork: the Add picker wants a FILE - use `dist/cyberos/cyberos.plugin` (the one-file bundle build.sh produces; selecting a folder greys the Open button). The folder route works where marketplaces are supported: add `dist/cyberos` as a marketplace (its root carries `.claude-plugin/marketplace.json`).

Then run `/fr-init` in a repo and `/ship-feature-requests` (or just ask to ship the next FR) to drive the backlog.

### 4b. Any other agent (Codex, Gemini, Cursor, Grok, CLI agents) - agent-independent

The core is doc-driven, so NO plugin is required for any agent. `init.sh` writes `.cyberos/AGENT-ENTRY.md` - a one-page canonical trigger any agent can follow - and creates thin pointer stubs when absent (`CLAUDE.md`, `GEMINI.md`, `.cursorrules`; your own `AGENTS.md` is never clobbered). Point any agent that can read files and run shell at `.cyberos/AGENT-ENTRY.md` (or paste its 5 rules as the prompt) and it drives the same workflow with the same gates and the same required human verdicts. The Claude plugin is convenience sugar, not a dependency.

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
fr-init:  ; bash .cyberos-init/init.sh
fr-gates: ; bash .cyberos/cuo/gates/run-gates.sh
```

### Planned channels (say the word and I will build them)

- npx CLI - `npx cyberos init` / `... gates` (Node wrapper around these scripts).
- MCP server - expose `fr_init`, `fr_gates`, `ship_fr` as MCP tools so any MCP agent triggers it with no files.
- GitHub template repo - a pre-scaffolded repo you clone or `degit` for a fresh project.
- Homebrew tap and Nix flake - `brew install cyberos` / `nix run`.

## After install: trigger, gate, sign off

1. Write an FR: `cp .cyberos/cuo/templates/FR-TEMPLATE.md docs/feature-requests/FR-001-<slug>.md`, fill section 1, set `status: ready_to_implement`, add the row to `BACKLOG.md`.
2. Trigger: tell your agent to follow `.cyberos/cuo/ship-feature-requests.md` and drive the next eligible FR, HITL required, `repo_root` = this repo. (Or `/ship-feature-requests` with the plugin.)
3. Gate: `bash .cyberos/cuo/gates/run-gates.sh`.
4. Sign off: you record the review verdict and the final acceptance. The agent never sets `done`.

## Staying in sync

The pack is a build artifact. When the workflow improves in CyberOS, rebuild (`build.sh`) and re-distribute; consumers re-run `init.sh` (it backs up `gates.env` and never clobbers your BACKLOG).
