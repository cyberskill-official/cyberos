---
title: Install, update and operate CyberOS in any repo · CyberOS
---

Step-by-step: from zero to your first shipped task in any repo, and how to keep CyberOS current. This page ships in the payload as `GUIDE.md` and on the docs site.

Every operation has two equal paths:

- **Desktop app** (no terminal): CyberOS Ops tab — Build payload, per-project Install, Version check. See the [desktop ops guide](./guides/desktop-ops.html).
- **CLI** (scriptable): the five commands below. Same scripts the app runs.

## Final commands

| Shell | Slash (Claude plugin) | Purpose |
|-------|----------------------|---------|
| `bash install.sh [repo]` | `/install` | Install or **re-vendor** CyberOS into a repo |
| `bash uninstall.sh [repo]` | `/uninstall` | Remove the machine (keeps tasks; BRAIN kept by default) |
| `bash version.sh [repo]` | `/version` | Check for a newer CyberOS; if stale, ask → runs `install` on **y** |
| `bash status.sh [repo]` | `/status` | Open `docs/status/index.html` in the default browser |
| `bash help.sh` | `/help` | Print the CLI surface |

Also: `/ship-tasks`, `/create-tasks` for the task workflow.

**Day-to-day rule:** install once, then forget. Soft update-check runs automatically whenever anything under `.cyberos/` is used (gates, hooks, MCP, help, version, status). Manual check is only `/version`. Re-vendor is always `install` — there is no separate “apply” command.

## What install puts in your repo

Under a single gitignored `.cyberos/`:

- `.cyberos/cuo/` — workflow (`ship-tasks.md`), doctrine, skills, `gates/`
- `.cyberos/memory/` — Layer-1 protocol (`AGENTS.md`) + schema + invariants; live BRAIN at `memory/store/`
- `.cyberos/plugin/` — Claude plugin commands + skills
- `.cyberos/AGENT-ENTRY.md` — full agent one-pager
- `.cyberos/gates.env`, `manifest.yaml`, `VERSION`
- Root **`AGENTS.md`** — thin pointer to `.cyberos/AGENT-ENTRY.md` (same idea as `CLAUDE.md` / `GEMINI.md`). The dense memory protocol is **only** at `.cyberos/memory/AGENTS.md`.

Tracked (not gitignored): `docs/tasks/`, `docs/status/`, `CHANGELOG.md`, agent pointer files.

## Prerequisites

- An agent with shell/file access (Claude Code, Cowork, Codex, Cursor, Grok, …) — entry via root `AGENTS.md` → `.cyberos/AGENT-ENTRY.md`
- The CyberOS payload: desktop **Build payload**, or `bash tools/install/build.sh` → `dist/cyberos/`, or release tarball
- git + your project’s normal build/test tools

## Steps — first install

1. **Get the payload** (once). Desktop: Ops → Build payload. CLI from a CyberOS checkout:

   ```bash
   bash tools/install/build.sh        # → dist/cyberos/
   ```

   Or release:

   ```bash
   curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz \
     | tar -xz -C /tmp
   ```

2. **Install into the target repo.** Desktop: Ops → pick project → Install. CLI:

   ```bash
   bash /path/to/dist/cyberos/install.sh /path/to/your/repo
   # or: bash /tmp/cyberos/install.sh /path/to/your/repo
   ```

   With the Claude plugin: `/install`. Idempotent: re-running install re-vendors the machine, backs up `gates.env`, never destroys BACKLOG / tasks / BRAIN.

3. **Check gates.** Edit `.cyberos/gates.env` if needed, then:

   ```bash
   bash .cyberos/cuo/gates/run-gates.sh
   ```

4. **Write your first task** (folder-per-task):

   ```bash
   mkdir -p docs/tasks/<module>/TASK-001-my-first
   cp .cyberos/cuo/templates/task-TEMPLATE.md docs/tasks/<module>/TASK-001-my-first/spec.md
   ```

   Fill section 1, set `status: ready_to_implement`, `class: product` or `improvement`. Add a row to `docs/tasks/BACKLOG.md`.

5. **Ship.** Paste to your agent (or `/ship-tasks`):

   > Follow `.cyberos/cuo/ship-tasks.md`. Drive the next eligible task in `docs/tasks/BACKLOG.md`. HITL is required: halt at review acceptance and final acceptance; never set `done` yourself.

6. **Human gates.** You record review (`reviewing → ready_to_test`) and final acceptance (`testing → done`). The agent never self-accepts.

7. **Status page.** After tasks exist:

   ```bash
   bash .cyberos/status.sh          # opens docs/status/index.html
   # or /status
   ```

   The page also regenerates automatically on commit (when task sources change) and after `run-gates`.

## Keeping CyberOS current

```bash
bash .cyberos/version.sh            # or /version
# if stale and you type y → install re-vendors from the payload
```

Or force re-vendor without the prompt:

```bash
bash /path/to/dist/cyberos/install.sh /path/to/your/repo
```

Soft warnings also appear when you run gates or other `.cyberos` tools (throttled ~12h). Env: `CYBEROS_UPDATE_CHECK=soft|always|strict|0`, `CYBEROS_OFFLINE=1`, `CYBEROS_NONINTERACTIVE=1` (version never prompts).

## Uninstall

```bash
bash .cyberos/uninstall.sh          # or /uninstall
```

Keeps `docs/tasks/`, `docs/status/`, pointer files, and BRAIN by default. Drop BRAIN with `CYBEROS_UNINSTALL_KEEP_BRAIN=0`.

## Product vs platform version

| File | Meaning |
|------|---------|
| `.cyberos/VERSION` | **CyberOS platform** version |
| Your app’s `package.json` / `VERSION` | **Product** version (independent) |

## Running CyberOS under sandboxed agents

Some agent runtimes execute every shell command inside a sandbox: a per-command time cap, no
process outliving the command that spawned it, and the project reachable only through a synced
mount. CyberOS works there. These are the recurring failure shapes and the patterns that work,
each as symptom → cause → working pattern.

**Commit hooks and package installs are killed mid-run.**
Symptom: `git commit` dies while the pre-commit chain rebuilds the payload; a package-manager
install is cut off before it finishes; anything started with `&` is gone the moment the command
returns.
Cause: the per-command time cap kills the whole process group — hook chains and installs
regularly need more than one command's budget, and background processes die with the call.
Pattern: replay each hook obligation manually as its own command (the payload rebuild, the
version-sync check, the status-page regeneration — whatever the chain runs), then commit with
`--no-verify`, and record the replayed obligations and their outputs in the commit message or
the task's gate log. `--no-verify` with recorded evidence is the gate executed by hand; without
the record it is a skipped gate.

**Builds and test suites crawl or time out over the synced mount.**
Symptom: a build that takes seconds on plain disk hits the time cap against the mounted repo.
Cause: every read and write crosses the mount's sync layer.
Pattern: clone the mounted repo to a local working copy inside the sandbox (for example
`git clone /mnt/<repo> /tmp/work`), then build, test, and commit there. Land the result back on
the mounted repo from its own side: `git fetch /tmp/work <branch>` then
`git merge --ff-only FETCH_HEAD`. That is a local ref move, not a remote push — both
repositories sit on the same disk, no remote is touched, and the workflow's no-push policy
(a human pushes to remotes) stays intact.

**Package-manager churn over the mount is impractical.**
Symptom: dependency installs into the mounted tree run for minutes, half-sync, or die at the cap.
Cause: package managers write tens of thousands of small files — the worst load a synced mount
can carry, and rarely finishable within one capped command.
Pattern: install dependencies in the local working copy and keep them there; never let a
dependency tree sync back through the mount.

**Deletes fail or fresh files read wrong on the mount.**
Symptom: transient permission errors on unlink, or a file that was just written reads empty or
stale.
Cause: the mounted view is eventually consistent; sync lag surfaces as phantom errors.
Pattern: wait a few seconds and re-check before treating it as real corruption; keep anything
latency-sensitive in the local working copy.

The workflow-side rules these patterns serve — one writer through one filesystem view,
acceptance evidence read from the committed object — are normative in `ship-tasks.md` (§11a and
§9); this section is the environment runbook that complements them.

## Where to go next

- Payload channel catalog: payload `README.md`
- Consumer update detail: `docs/CONSUMER_UPDATE.md` (in the monorepo / pack tools)
- Site: https://cyberos.cyberskill.world/docs
