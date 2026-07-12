---
title: Install, update and operate CyberOS in any repo · CyberOS
---

The linear walkthrough, from zero to your first shipped FR in a repo that is not CyberOS, plus how to keep it up to date. This page is the single source: it ships inside the payload as `GUIDE.md` and renders on the docs site. For the full list of install channels and their trade-offs, see the payload `README.md`.

Every operation below has two equal paths:

- Desktop app (no terminal): the CyberOS app's "CyberOS Ops" tab has buttons for Build payload, Check update, and Init/Update per project - see the [desktop ops guide](./guides/desktop-ops.html). This is the path for employees.
- CLI (scriptable): the `init.sh` commands shown inline. This is the path for CI, rollout scripts, and anyone who prefers a shell.

Both paths run the same canonical scripts, so the result is identical.

## What init installs

One action lays the CyberOS machine into your repo under a single gitignored `.cyberos/`, organised by module (this mirrors how CyberOS ships internally, so what you run here is what we run):

- `.cyberos/cuo/` - the workflow engine: `ship-feature-requests.md` + `EXECUTION-DISCIPLINE.md` + `STATUS-REFERENCE.md`, the author/audit skills, `gates/`, and `templates/`.
- `.cyberos/memory/` - the Layer-1 memory protocol (`AGENTS.md`) + schema + invariants.
- `.cyberos/plugin/` - the Claude/Cowork plugin (`/init`, `/update`, `/changelog`, `/help` + the `ship-feature-requests` skill).
- `.cyberos/AGENT-ENTRY.md` - the agent-independent entry point (plus `CLAUDE.md` / `GEMINI.md` / `.cursorrules` pointer stubs where absent).
- `.cyberos/gates.env`, `.cyberos/manifest.yaml`, `.cyberos/VERSION` - your gate commands, the build manifest, and the single CyberOS version stamp.
- `.cyberos/memory/store/` - your local BRAIN store (tenant data).

`.cyberos/` and `.cyberos/memory/store/` are both gitignored: the machine is regenerable via init, and the BRAIN holds local data. CyberOS carries one version (in `.cyberos/VERSION`); the modules version internally.

## Prerequisites

- An agent with shell and file access to the target repo (Claude Code, Cowork, Codex, Gemini, Cursor, or any shell-capable agent - they all enter through `.cyberos/AGENT-ENTRY.md`).
- The CyberOS payload: build it with the desktop app (Ops tab -> Build payload) or from a CyberOS checkout with `bash tools/cyberos-init/build.sh` (produces `dist/cyberos/`), or obtain it through any channel in the payload `README.md`.
- git, and your project's normal build/test toolchain.

## Steps

1. Get the payload (once). Desktop app: Ops tab -> Build payload. CLI, from a CyberOS checkout:

   ```bash
   bash tools/cyberos-init/build.sh        # writes dist/cyberos/
   ```

   Keep `dist/cyberos/` wherever is convenient - it does not need to live inside the target repo. (Contributors: a pre-commit hook rebuilds `dist/cyberos/` automatically whenever a vendored source changes, so it always reflects the current machine.)

2. Initialise the target repo. Desktop app: Ops tab -> pick the project from the list (or paste its path) -> Init. CLI:

   ```bash
   bash /path/to/dist/cyberos/init.sh /path/to/your/repo
   ```

   This auto-detects your build/lint/test, writes `.cyberos/gates.env`, scaffolds `docs/feature-requests/`, vendors the machine by module into `.cyberos/`, and stamps `.cyberos/VERSION`. By default it also sets up the BRAIN: a local `.cyberos/memory/store/` store plus the `AGENTS.md` memory rules (skip with `CYBEROS_NO_MEMORY=1`). It never clobbers an existing `AGENTS.md`, `BACKLOG.md`, `gates.env`, or BRAIN. (With the Claude plugin, run `/init` instead.)

3. Check the gates. Open `.cyberos/gates.env` and confirm `BUILD_CMD` / `LINT_CMD` / `TEST_CMD` are right for your repo; edit if autodetect missed anything. If you have a caf baseline or an awh goldenset, set `CAF_ENABLED` / `AWH_ENABLED` to `true` and point them at your files. Then confirm a clean tree is green:

   ```bash
   bash .cyberos/cuo/gates/run-gates.sh
   ```

4. Write your first FR:

   ```bash
   cp .cyberos/cuo/templates/FR-TEMPLATE.md docs/feature-requests/FR-001-my-first.md
   ```

   Fill section 1 with numbered normative clauses (each a testable promise), set `status: ready_to_implement`, and set `class: product` (or `class: improvement` for hardening work).

5. List it in the backlog. There is exactly one backlog for all work - product and improvement FRs together, never a separate improvement file. Add a row under the ready section of `docs/feature-requests/BACKLOG.md`, tagging hardening rows `(improvement)`:

   ```
   - [ready_to_implement] FR-001-my-first - my first feature
   - [ready_to_implement] FR-002-rate-limit - login rate limiting (improvement)
   ```

6. Trigger the workflow. Paste this to your agent (or run `/ship-feature-requests` with the plugin):

   > Follow `.cyberos/cuo/ship-feature-requests.md`. Drive the next eligible FR in `docs/feature-requests/BACKLOG.md`. repo_root is this repo. HITL is required: halt at review acceptance and at final acceptance for my verdict, and never set `done` yourself.

7. The agent runs to the first human gate. It maps the repo, writes the edge-case matrix, implements with observability and coverage on touched files, reviews the diff against every section-1 clause, and stops at review acceptance (`reviewing -> ready_to_test`). It shows you the review packet.

8. Record the review verdict. If it holds up, tell the agent to advance (you are the human verdict). If not, tell it what is missing; it routes the FR back to `ready_to_implement` and reworks.

9. The agent runs the test phase to the final gate. Coverage, your configured gates, and caf/awh if enabled must be green. It stops at final acceptance (`testing -> done`).

10. Record final acceptance. Confirm, and the agent sets `done`, updates `BACKLOG.md`, and commits the diff per phase. You run `git push` yourself; the agent never pushes.

11. Next FR. The workflow picks the next eligible FR on its own; repeat from step 7. Add more FRs any time by repeating steps 4 and 5.

## Staying up to date

CyberOS carries one version, stamped in `.cyberos/VERSION` when you init and in the payload's `VERSION`. To keep a project current:

- Check for updates (safe, read-only). Desktop app: Ops tab -> select the project -> Check (the project list also shows each repo's installed version at a glance). CLI:

  ```bash
  bash /path/to/dist/cyberos/init.sh --check /path/to/your/repo
  ```

  It prints `installed=<x> available=<y>` and tells you whether an update exists. Wire the CLI form into CI or a periodic job to get notified automatically when a project falls behind.

- Apply an update. Desktop app: Init on the same project (init IS the update - it is idempotent). CLI:

  ```bash
  bash /path/to/dist/cyberos/init.sh /path/to/your/repo
  ```

  init re-vendors `.cyberos/cuo`, `.cyberos/memory`, and `.cyberos/plugin`, refreshes the manifest and `VERSION`, backs up `gates.env` before rewriting it, and never touches your `BACKLOG.md`, your FRs, your `AGENTS.md`, or your BRAIN. So an update swaps the machine, not your work.

Automatic vs manual: `--check` is the notify half (run it on a schedule to be told); re-running init is the apply half (run it when you choose). There is no silent in-place mutation - an update is always an explicit init run.

## Rolling out to several repos

One payload initialises many repos. Desktop app: run Init per project from the list. CLI, from a CyberOS checkout:

```bash
bash tools/cyberos-init/build.sh                      # dist/cyberos/ (once)
PAYLOAD="$(pwd)/dist/cyberos"

for repo in ~/Projects/CyberSkill/ssl ~/Projects/CyberSkill/gam ~/Projects/CyberSkill/cyber-click; do
  bash "$PAYLOAD/init.sh" "$repo"                      # vendors .cyberos/, keeps each repo's own BACKLOG/FRs/BRAIN
done
```

Fleet-wide with guard rails: `bash tools/cyberos-init/rollout.sh` (skips dirty repos, prints a per-repo summary).

Each repo gets its own gitignored `.cyberos/` (autodetected gates for its stack) and its own BRAIN. Nothing is committed by init - review `.cyberos/gates.env` per repo, then commit only the files you intend to (the FRs and backlog, never `.cyberos/`).

To see which repos have fallen behind after you cut a new CyberOS version:

```bash
for repo in ~/Projects/CyberSkill/*; do
  [ -d "$repo/.git" ] && bash "$PAYLOAD/init.sh" --check "$repo"
done
```

Re-run `init.sh <repo>` (without `--check`) on any that report an update - or press Init in the Ops tab.

## Troubleshooting

- Gates run the wrong command: edit `.cyberos/gates.env`; `run-gates.sh` reads it directly.
- The agent tried to set `done` itself: that breaks HITL. Point it back at the HITL section of `.cyberos/cuo/ship-feature-requests.md` and the two acceptance gates.
- reduced vs full profile: check `.cyberos/manifest.yaml`. Reduced still gates on your own build/lint/test plus coverage plus the two human gates; full adds the vendored caf/awh deterministic gates.
- `--check` says `installed=none`: the repo was never inited (or `.cyberos/VERSION` predates this feature); run init once to stamp it.

## Guides

- [Operate CyberOS from the desktop app](./guides/desktop-ops.html) - the UI path for employees: build, check, init/update, settings, troubleshooting.
- Ship your first feature request (cuo module -> Guides) - the day-one workflow walkthrough (source: `modules/cuo/docs/guides/ship-your-first-fr.md`).


## SDP lifecycle map (stages 1-14)

Every stage of the software development process ships in this payload. "Invoked by"
says which of the two commands automates the stage; `standalone` pairs are invoked on
request ("draft a runbook", "author the SRS", ...). Contract level: `full` pairs carry
RUBRIC/PIPELINE/envelopes/acceptance; `thin` pairs carry SKILL.md + trigger tests until
FR-SKILL-118-class deepening reaches them.

| stage | skill pair | invoked by | contract |
|---|---|---|---|
| 1 SOW | statement-of-work-author/-audit | standalone | thin |
| 2 PRD | product-requirements-document-author/-audit | standalone | thin |
| 3 SRS | software-requirements-specification-author/-audit | standalone (feeds /create-feature-requests) | thin |
| 4 NFR | nfr-certification-author + nfr-evaluator + nfr-test-runner + nfr-regression-handler | standalone | thin |
| 5 FR | feature-request-author/-audit | /create-feature-requests | full |
| 6 Architecture | architectural-spike-author/-audit + architecture-decision-record-author/-audit + threat-model-author/-audit | /ship-feature-requests (ADR steps 3-4; spike and threat-model standalone) | full (spike, ADR) / thin (threat-model) |
| 7 SDD | software-design-document-author/-audit | standalone | thin |
| 8 Implementation | repo-context-map + implementation-plan + edge-case-matrix + mock-contract-test + observability-injection + backlog-state-update pairs | /ship-feature-requests | full (implementation-plan) / thin (rest) |
| 9 Review | code-review-author/-audit | /ship-feature-requests | full |
| 10 Test | coverage-gate + debugging-cycle pairs (+ test-strategy standalone) | /ship-feature-requests | thin |
| 11 Deploy | deployment-checklist-author/-audit | standalone | thin |
| 12 Release | release-notes-author/-audit | standalone | thin |
| 13 Runbook | runbook-author/-audit | standalone | thin |
| 14 Retro / decommission | retrospective-author/-audit + postmortem-author/-audit + decommissioning-author/-audit | standalone | thin |
