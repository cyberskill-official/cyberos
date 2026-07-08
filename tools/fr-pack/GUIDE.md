# Step by step: run ship-feature-requests on another project

The linear walkthrough, from zero to your first shipped FR in a repo that is not CyberOS. For the full list of install channels and their trade-offs, see `README.md`; this file is the "just do it in order" runbook.

## Prerequisites

- An agent with shell and file access to the target repo (Claude Code, Cowork, or Codex).
- The fr-pack bundle: build it from a CyberOS checkout with `bash tools/fr-pack/build-pack.sh` (produces `dist/fr-pack/`), or obtain it through any channel in `README.md`.
- git, and your project's normal build/test toolchain.

## Steps

1. Put the pack in your repo. Pick one channel (details in `README.md`); the simplest is to copy the folder:

   ```bash
   cp -R dist/fr-pack /path/to/your/repo/.fr-pack
   ```

2. Initialize the repo:

   ```bash
   cd /path/to/your/repo
   bash .fr-pack/init.sh
   ```

   This auto-detects your build/lint/test, writes `.cyberos/fr.gates.env`, scaffolds `docs/feature-requests/`, and copies the workflow machine to `.cyberos/fr-pack/`. (With the Claude plugin, run `/fr-init` instead.) By default it also sets up the BRAIN memory protocol: a local `.cyberos-memory/` store plus the `AGENTS.md` memory rules (skip with `FRPACK_NO_MEMORY=1`).

3. Check the gates. Open `.cyberos/fr.gates.env` and confirm `BUILD_CMD` / `LINT_CMD` / `TEST_CMD` are right for your repo; edit if the autodetect missed anything. If you have a caf baseline or an awh goldenset, set `CAF_ENABLED` / `AWH_ENABLED` to `true` and point them at your files. Then confirm a clean tree is green:

   ```bash
   bash .cyberos/fr-pack/gates/run-gates.sh
   ```

4. Write your first FR:

   ```bash
   cp .cyberos/fr-pack/templates/FR-TEMPLATE.md docs/feature-requests/FR-001-my-first.md
   ```

   Fill section 1 with numbered normative clauses (each a testable promise), set `status: ready_to_implement`, and set `class: product` (or `class: improvement` for hardening work).

5. List it in the backlog. Add a row under the ready section of `docs/feature-requests/BACKLOG.md`:

   ```
   - [ready_to_implement] FR-001-my-first - my first feature
   ```

6. Trigger the workflow. Paste this to your agent (or run `/ship-fr` with the plugin):

   > Follow `.cyberos/fr-pack/machine/ship-feature-requests.md`. Drive the next eligible FR in `docs/feature-requests/BACKLOG.md`. repo_root is this repo. HITL is required: halt at review acceptance and at final acceptance for my verdict, and never set `done` yourself.

7. The agent runs to the first human gate. It maps the repo, writes the edge-case matrix, implements with observability and coverage on touched files, reviews the diff against every section-1 clause, and stops at review acceptance (`reviewing -> ready_to_test`). It shows you the review packet.

8. Record the review verdict. If it holds up, tell the agent to advance (you are the human verdict). If not, tell it what is missing; it routes the FR back to `ready_to_implement` and reworks.

9. The agent runs the test phase to the final gate. Coverage, your configured gates, and caf/awh if enabled must be green. It stops at final acceptance (`testing -> done`).

10. Record final acceptance. Confirm, and the agent sets `done`, updates `BACKLOG.md`, and commits the diff per phase. You run `git push` yourself; the agent never pushes.

11. Next FR. The workflow picks the next eligible FR on its own; repeat from step 7. Add more FRs any time by repeating steps 4 and 5.

## Troubleshooting

- Gates run the wrong command: edit `.cyberos/fr.gates.env`; `run-gates.sh` reads it directly.
- The agent tried to set `done` itself: that breaks HITL. Point it back at the HITL section of `machine/ship-feature-requests.md` and the two acceptance gates.
- reduced vs full profile: check `.cyberos/fr-pack/manifest.yaml`. Reduced still gates on your own build/lint/test plus coverage plus the two human gates; full adds the vendored caf/awh deterministic gates.

## Updating the pack

When the workflow improves in CyberOS, obtain a newer bundle and re-run `init.sh`. It backs up `fr.gates.env` and never clobbers your `BACKLOG.md`.
