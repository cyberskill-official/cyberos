# Step by step: run CyberOS on another project

The linear walkthrough, from zero to your first shipped FR in a repo that is not CyberOS, plus how to keep it up to date. For the full list of install channels and their trade-offs, see `README.md`; this file is the "just do it in order" runbook.

## What init installs

One command lays the CyberOS machine into your repo under a single gitignored `.cyberos/`, organised by module (this mirrors how CyberOS ships internally, so what you run here is what we run):

- `.cyberos/cuo/` - the workflow engine: `ship-feature-requests.md` + `EXECUTION-DISCIPLINE.md` + `STATUS-REFERENCE.md`, the author/audit skills, `gates/`, and `templates/`.
- `.cyberos/memory/` - the Layer-1 memory protocol (`AGENTS.md`) + schema + invariants.
- `.cyberos/plugin/` - the Claude/Cowork plugin (`/ship-fr`, `/fr-init`).
- `.cyberos/gates.env`, `.cyberos/manifest.yaml`, `.cyberos/VERSION` - your gate commands, the build manifest, and the single CyberOS version stamp.
- `.cyberos-memory/` - your local BRAIN store (tenant data).

`.cyberos/` and `.cyberos-memory/` are both gitignored: the machine is regenerable via init, and the BRAIN holds local data. CyberOS carries one version (in `.cyberos/VERSION`); the modules version internally.

## Prerequisites

- An agent with shell and file access to the target repo (Claude Code, Cowork, or Codex).
- The CyberOS payload: build it from a CyberOS checkout with `bash tools/cyberos-init/build.sh` (produces `dist/cyberos/`), or obtain it through any channel in `README.md`.
- git, and your project's normal build/test toolchain.

## Steps

1. Get the payload (once). From a CyberOS checkout:

   ```bash
   bash tools/cyberos-init/build.sh        # writes dist/cyberos/
   ```

   Keep `dist/cyberos/` wherever is convenient - it does not need to live inside the target repo. (Contributors: a pre-commit hook rebuilds `dist/cyberos/` automatically whenever a vendored source changes, so it always reflects the current machine.)

2. Initialise the target repo by pointing init at it:

   ```bash
   bash /path/to/dist/cyberos/init.sh /path/to/your/repo
   ```

   This auto-detects your build/lint/test, writes `.cyberos/gates.env`, scaffolds `docs/feature-requests/`, vendors the machine by module into `.cyberos/`, and stamps `.cyberos/VERSION`. By default it also sets up the BRAIN: a local `.cyberos-memory/` store plus the `AGENTS.md` memory rules (skip with `CYBEROS_NO_MEMORY=1`). It never clobbers an existing `AGENTS.md`, `BACKLOG.md`, `gates.env`, or BRAIN. (With the Claude plugin, run `/fr-init` instead.)

3. Check the gates. Open `.cyberos/gates.env` and confirm `BUILD_CMD` / `LINT_CMD` / `TEST_CMD` are right for your repo; edit if autodetect missed anything. If you have a caf baseline or an awh goldenset, set `CAF_ENABLED` / `AWH_ENABLED` to `true` and point them at your files. Then confirm a clean tree is green:

   ```bash
   bash .cyberos/cuo/gates/run-gates.sh
   ```

4. Write your first FR:

   ```bash
   cp .cyberos/cuo/templates/FR-TEMPLATE.md docs/feature-requests/FR-001-my-first.md
   ```

   Fill section 1 with numbered normative clauses (each a testable promise), set `status: ready_to_implement`, and set `class: product` (or `class: improvement` for hardening work).

5. List it in the backlog. Add a row under the ready section of `docs/feature-requests/BACKLOG.md`:

   ```
   - [ready_to_implement] FR-001-my-first - my first feature
   ```

6. Trigger the workflow. Paste this to your agent (or run `/ship-fr` with the plugin):

   > Follow `.cyberos/cuo/ship-feature-requests.md`. Drive the next eligible FR in `docs/feature-requests/BACKLOG.md`. repo_root is this repo. HITL is required: halt at review acceptance and at final acceptance for my verdict, and never set `done` yourself.

7. The agent runs to the first human gate. It maps the repo, writes the edge-case matrix, implements with observability and coverage on touched files, reviews the diff against every section-1 clause, and stops at review acceptance (`reviewing -> ready_to_test`). It shows you the review packet.

8. Record the review verdict. If it holds up, tell the agent to advance (you are the human verdict). If not, tell it what is missing; it routes the FR back to `ready_to_implement` and reworks.

9. The agent runs the test phase to the final gate. Coverage, your configured gates, and caf/awh if enabled must be green. It stops at final acceptance (`testing -> done`).

10. Record final acceptance. Confirm, and the agent sets `done`, updates `BACKLOG.md`, and commits the diff per phase. You run `git push` yourself; the agent never pushes.

11. Next FR. The workflow picks the next eligible FR on its own; repeat from step 7. Add more FRs any time by repeating steps 4 and 5.

## Staying up to date

CyberOS carries one version, stamped in `.cyberos/VERSION` when you init and in the payload's `VERSION`. To keep a project current:

- Check for updates (manual, safe, read-only). Point a fresh payload at the repo with `--check`:

  ```bash
  bash /path/to/dist/cyberos/init.sh --check /path/to/your/repo
  ```

  It prints `installed=<x> available=<y>` and tells you whether an update exists. Wire this into CI or a periodic job to get notified automatically when a project falls behind.

- Apply an update. Re-run init with the newer payload:

  ```bash
  bash /path/to/dist/cyberos/init.sh /path/to/your/repo
  ```

  init is idempotent: it re-vendors `.cyberos/cuo`, `.cyberos/memory`, and `.cyberos/plugin`, refreshes the manifest and `VERSION`, backs up `gates.env` before rewriting it, and never touches your `BACKLOG.md`, your FRs, your `AGENTS.md`, or your BRAIN. So an update swaps the machine, not your work.

Automatic vs manual: `--check` is the notify half (run it on a schedule to be told); re-running init is the apply half (run it when you choose). There is no silent in-place mutation - an update is always an explicit init run.

## Troubleshooting

- Gates run the wrong command: edit `.cyberos/gates.env`; `run-gates.sh` reads it directly.
- The agent tried to set `done` itself: that breaks HITL. Point it back at the HITL section of `.cyberos/cuo/ship-feature-requests.md` and the two acceptance gates.
- reduced vs full profile: check `.cyberos/manifest.yaml`. Reduced still gates on your own build/lint/test plus coverage plus the two human gates; full adds the vendored caf/awh deterministic gates.
- `--check` says `installed=none`: the repo was never inited (or `.cyberos/VERSION` predates this feature); run init once to stamp it.
