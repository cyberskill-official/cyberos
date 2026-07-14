---
title: Install, update and operate CyberOS in any repo · CyberOS
---

Step-by-step: from zero to your first shipped FR in any repo, and how to keep CyberOS current. This page ships in the payload as `GUIDE.md` and on the docs site.

Every operation has two equal paths:

- **Desktop app** (no terminal): CyberOS Ops tab — Build payload, per-project Install, Version check. See the [desktop ops guide](./guides/desktop-ops.html).
- **CLI** (scriptable): the five commands below. Same scripts the app runs.

## Final commands

| Shell | Slash (Claude plugin) | Purpose |
|-------|----------------------|---------|
| `bash install.sh [repo]` | `/install` | Install or **re-vendor** CyberOS into a repo |
| `bash uninstall.sh [repo]` | `/uninstall` | Remove the machine (keeps FRs; BRAIN kept by default) |
| `bash version.sh [repo]` | `/version` | Check for a newer CyberOS; if stale, ask → runs `install` on **y** |
| `bash status.sh [repo]` | `/status` | Open `docs/status/index.html` in the default browser |
| `bash help.sh` | `/help` | Print the CLI surface |

Also: `/ship-tasks`, `/create-tasks` for the FR workflow.

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
- The CyberOS payload: desktop **Build payload**, or `bash tools/cyberos-init/build.sh` → `dist/cyberos/`, or release tarball
- git + your project’s normal build/test tools

## Steps — first install

1. **Get the payload** (once). Desktop: Ops → Build payload. CLI from a CyberOS checkout:

   ```bash
   bash tools/cyberos-init/build.sh        # → dist/cyberos/
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

   With the Claude plugin: `/install`. Idempotent: re-running install re-vendors the machine, backs up `gates.env`, never destroys BACKLOG / FRs / BRAIN.

3. **Check gates.** Edit `.cyberos/gates.env` if needed, then:

   ```bash
   bash .cyberos/cuo/gates/run-gates.sh
   ```

4. **Write your first FR** (folder-per-FR):

   ```bash
   mkdir -p docs/tasks/<module>/FR-001-my-first
   cp .cyberos/cuo/templates/FR-TEMPLATE.md docs/tasks/<module>/FR-001-my-first/spec.md
   ```

   Fill section 1, set `status: ready_to_implement`, `class: product` or `improvement`. Add a row to `docs/tasks/BACKLOG.md`.

5. **Ship.** Paste to your agent (or `/ship-tasks`):

   > Follow `.cyberos/cuo/ship-tasks.md`. Drive the next eligible FR in `docs/tasks/BACKLOG.md`. HITL is required: halt at review acceptance and final acceptance; never set `done` yourself.

6. **Human gates.** You record review (`reviewing → ready_to_test`) and final acceptance (`testing → done`). The agent never self-accepts.

7. **Status page.** After FRs exist:

   ```bash
   bash .cyberos/status.sh          # opens docs/status/index.html
   # or /status
   ```

   The page also regenerates automatically on commit (when FR sources change) and after `run-gates`.

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

## Where to go next

- Payload channel catalog: payload `README.md`
- Consumer update detail: `docs/CONSUMER_UPDATE.md` (in the monorepo / pack tools)
- Site: https://cyberos.cyberskill.world/docs
