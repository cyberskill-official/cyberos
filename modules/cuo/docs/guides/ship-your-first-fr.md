# Ship your first feature request

The day-one guide for anyone at CyberSkill. It takes you from "I have a repo and an idea" to a shipped, human-accepted change, using the ship-feature-requests workflow. No prior CyberOS knowledge assumed.

## What you are about to do

CyberOS turns work into feature requests (FRs): small markdown files that state what to build and how to prove it works. An AI agent drives each FR through implement -> review -> test -> done, and you (the human) accept it at two gates. You never review raw diffs blind; you review a finished, self-tested change with evidence.

Two roles in every step below:

- You: decide, trigger, and accept. You are the only one who can approve the two human gates, push, merge, or deploy.
- The agent (Claude or any other): does the work between your decisions and always stops to wait for you at the gates.

## Step 1: install CyberOS into your repo (once per repo)

Option A - desktop app (no terminal): open the CyberOS app, CyberOS Ops tab, pick your project from the list (or paste its path), press Init. The same button updates an already-initialised project later.

Option B - terminal:

    bash /path/to/cyberos/dist/cyberos/init.sh /path/to/your-repo

Either way, the result is the same: a gitignored `.cyberos/` folder (the workflow engine, the memory protocol, the plugin), a `docs/feature-requests/` folder with a `BACKLOG.md`, and agent entry files (`.cyberos/AGENT-ENTRY.md` plus `CLAUDE.md` / `GEMINI.md` / `.cursorrules` stubs where absent). Nothing about your code changes.

Optional but recommended: install the Claude plugin so the workflow is one slash-command away. In Claude: Settings -> Plugins -> Add, and pick the file `dist/cyberos/cyberos.plugin`.

## Step 2: write the FR (5 minutes)

Create one file: `docs/feature-requests/<module>/FR-<MODULE>-<NNN>-<slug>.md`. Copy the template from `.cyberos/cuo/templates/` or start from this skeleton:

    ---
    id: FR-SHOP-001
    title: Add a login rate limit
    module: shop
    class: product          # product = new capability; improvement = hardening/refactor/fix
    status: ready_to_implement
    priority: MUST
    depends_on: []
    routed_back_count: 0
    ---

    # FR-SHOP-001 - Add a login rate limit

    ## Context
    Why this matters, in 2-4 sentences.

    ## 1. Normative clauses
    1. The login endpoint MUST reject more than 5 attempts per minute per account.
    2. A rejected attempt MUST return 429 with a Retry-After header.

    ## 2. Acceptance criteria
    - [ ] 6th attempt within a minute returns 429.
    - [ ] Tests cover the limit and the reset.

Then add one line to `docs/feature-requests/BACKLOG.md` in the module's section (improvement-class rows get an `(improvement)` tag). The FR file's `status:` field is the record of truth; the backlog is just the index.

Rule of thumb for scope: an FR should be shippable in one sitting. If yours has more than roughly five clauses, split it.

## Step 3: trigger the agent

With the plugin: type `/ship-feature-requests` in your repo's session. Without it, paste this to any agent:

    Follow .cyberos/AGENT-ENTRY.md and drive the next eligible FR in
    docs/feature-requests/BACKLOG.md. HITL required. repo_root = this repo.

The agent implements the clauses, runs your repo's own gates (`bash .cyberos/cuo/gates/run-gates.sh` - build, lint, test, whatever `init` autodetected into `.cyberos/gates.env`), reviews its own work, and moves the FR's status forward as it goes.

## Step 4: your two gates

The agent stops and asks you twice. These are the only two moments the process needs you, and it cannot proceed without you:

1. reviewing -> ready_to_test: the agent presents what it built and its review findings. You read the FR's clauses against the change. Say "approved" to let it move to testing, or route it back with what is wrong.
2. testing -> done: the agent presents its test evidence (gate output, proof per acceptance criterion). You accept ("done") or route back.

An agent that marks its own work done is broken behavior - report it. Statuses only you can set: `ready_to_test` (gate 1) and `done` (gate 2).

## Step 5: land it

Once you have accepted, the change is yours to land the normal way: commit (if the agent has not already), push, and open the PR yourself. Agents never push, merge, or deploy on your behalf.

## When something goes wrong

- Gates fail: the agent must fix and re-run them before ever reaching you. If it asks you to accept with red gates, refuse.
- The FR was wrong: route back at either gate with one sentence about why; `routed_back_count` in the FR frontmatter tracks the loop.
- Not sure of a status meaning: `.cyberos/cuo/STATUS-REFERENCE.md` defines all ten states and who may set each.
- CyberOS itself is outdated in your repo: CyberOS Ops tab -> Check, or `init.sh --check <repo>`; re-run Init to update.

## Where everything lives

| Thing | Path |
|---|---|
| Workflow doctrine (full rules) | `.cyberos/cuo/ship-feature-requests.md` |
| Execution discipline (how agents behave) | `.cyberos/cuo/EXECUTION-DISCIPLINE.md` |
| Status contract (the 10 states) | `.cyberos/cuo/STATUS-REFERENCE.md` |
| Your FRs + backlog index | `docs/feature-requests/` |
| Gate wiring for this repo | `.cyberos/gates.env` |
| Agent entry point (any agent) | `.cyberos/AGENT-ENTRY.md` |
| Operator guide (deeper than this page) | `tools/cyberos-init/GUIDE.md` in the CyberOS repo |
