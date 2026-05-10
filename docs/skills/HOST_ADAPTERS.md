# HOST_ADAPTERS.md — running the manual chain on any agent host

> **Purpose.** [CHAIN_ORCHESTRATOR.md](./CHAIN_ORCHESTRATOR.md) describes fully automated mode (the agent drives everything; you give a pitch + answer HITL); [MANUAL_WORKFLOW.md](./MANUAL_WORKFLOW.md) describes manual mode (you drive every step). Both are host-neutral. This doc translates each abstract step into the specific commands / setup / fallbacks per agent host.

> **Audience.** Stephen, picking a host for the next project. Or a teammate / future contributor wanting to drive the chain on whatever they happen to use.

---

## Capability matrix

| Host | File tools | Shell / Bash | AGENTS.md auto-load | MCP | Skills auto-trigger | Filesystem write to user project | Recommended? |
|---|---|---|---|---|---|---|---|
| **Claude Cowork** (desktop app) | ✅ Read/Write/Edit on connected folders | ✅ sandboxed Linux env | ❌ (must be loaded explicitly) | ✅ | ✅ skills system | ✅ via connected folders | **★ Best for solo / small-team manual mode** |
| **Claude Code** (CLI) | ✅ native | ✅ native | ✅ on `AGENTS.md` / `CLAUDE.md` at root | ✅ | ✅ | ✅ native | **★ Best for terminal-driven engineers** |
| **Cursor** | ✅ native | ✅ terminal | via `.cursor/rules/<n>.mdc` | ✅ | partial | ✅ native | ★ Best for IDE-heavy workflows |
| **Codex CLI** (OpenAI) | ✅ native | ✅ shell | ✅ on `AGENTS.md` | ✅ | partial | ✅ native | ★ alternative to Claude Code |
| **Windsurf** | ✅ native | ✅ terminal | via `.windsurfrules` | ✅ | partial | ✅ native | OK |
| **GitHub Copilot CLI** | ✅ | ✅ | via `.github/copilot-instructions.md` | partial | partial | ✅ | OK |
| **Gemini CLI** | ✅ | ✅ | manual paste / `gemini-extension.json` | partial | partial | ✅ | OK |
| **OpenCode** | ✅ | ✅ | via `.opencode/INSTALL.md` plugin | ✅ | ✅ | ✅ | OK |
| **Aider / Continue.dev / Trae / Kiro** | ✅ | ✅ | manual or per-host config | varies | varies | ✅ | OK |
| **Claude.ai web** | ❌ no host filesystem | ❌ no shell | ❌ paste only | ❌ tool-list only | partial | ❌ | Degraded — chat + manual file management |
| **ChatGPT (Code Interpreter)** | partial (sandbox) | partial (Python only) | ❌ paste only | ❌ | ❌ | ❌ | Degraded — same shape as Claude.ai web |
| **Claude in Chrome** | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Browser-only; not recommended |

A **recommended** host runs the chain end-to-end without the user leaving the chat. A **degraded** host can still drive the conversational parts (interview, draft generation) but the user has to copy artefacts in/out of the agent and run `brain_writer.py` from a separate terminal.

---

## Adapter A — Claude Cowork (★ recommended for solo / small-team)

**This is what we're using right now.** Cowork is purpose-built for this kind of manual orchestration: connected folders + sandboxed bash + MCP + skills. Setup overhead: minimal.

### Setup (one-time per project)

1. **Connect the new project's folder.** When you start a Cowork session, ask Claude to `request_cowork_directory` for `/path/to/your/new/project`. Approve the prompt.
2. **Connect cyberos folder** (so the agent can read AGENTS.md + cyberos/docs/skills/*): same `request_cowork_directory` for `/path/to/cyberos`.
3. **Load AGENTS.md into context.** Cowork doesn't auto-load AGENTS.md the way Claude Code does. Ask the agent at the start of each session: *"Read `cyberos/docs/CyberOS-AGENTS-CORE.md` and follow it for this conversation."* The CORE.md is short (~10K tokens); it loads fast.
4. **Bootstrap BRAIN.** Cowork's bash sandbox can run `python3 outputs/brain_writer.py session-start <actor>` against the connected workbench folder. The agent will auto-bootstrap on first write per AGENTS.md §13.1.

### Per-step shape

- **Load a SKILL.md**: `Read cyberos/docs/skills/cuo/cpo/<skill>/SKILL.md` — agent reads via the file tool.
- **Run the interview**: agent conducts in chat; Cowork's chat surface handles the back-and-forth fine.
- **Save an artefact**: agent uses Write tool to create `<project-root>/planning/<date>-<slug>/<artefact>.md`.
- **Append audit row**: agent runs `python3 outputs/brain_writer.py write <actor> <relpath> <content_file>` via bash (cwd = project repo root).
- **Session end**: agent runs `python3 outputs/brain_writer.py session-end <actor>` via bash.

### Quirks / gotchas

- The bash sandbox's view of paths differs from the file tool's view. The agent already knows this — but if you run bash commands by hand, paths look like `/sessions/<id>/mnt/<folder>/...` not `/Users/<you>/...`.
- Cowork sessions don't persist across closing the app — but the BRAIN's audit ledger DOES persist (it's on your real filesystem). Resume any time.
- Skills installed at the Cowork level (the ones loaded via `Skill` tool) DO NOT include the CyberOS skills system — those live in your `cyberos/` repo and are loaded on-demand by the agent reading the SKILL.md files.
- **§0.1 sandbox guard (Bundle Q, 2026-05-11) — `outputs/brain_writer.py` refuses to run from inside Cowork's bash sandbox**, because the sandbox's view of the project lives at `/sessions/<id>/mnt/<folder>/...` which matches the AGENTS.md §0.1 forbidden-paths list (paths containing `/sessions/`, `local-agent-mode-sessions`, etc.). The bind-mount IS backed by your real filesystem, but §0.1 cannot tell that from inside; refusing is defense-in-depth against the case where it ISN'T. **Net effect:** the agent can edit non-BRAIN files (docs, source code) directly via the Write/Edit tools, but BRAIN-touching ops (audit-row appends, manifest mutations, `meta/protocol-history/` archives) require the user to run the apply script from a macOS terminal where the path resolves to `/Users/<you>/Projects/...` (no §0.1 forbidden substring). See "The cowork → macOS handoff" below.

### The cowork → macOS handoff (apply-script pattern)

Established 2026-05-11 during Bundle Q. The pattern:

1. **Agent in cowork edits non-BRAIN files** (AGENTS.md, CHANGELOG, README, skills runbooks, source code). These are ordinary file mutations — no audit row needed. Cowork's Write/Edit tools work fine.
2. **Agent stages BRAIN-touching ops** as a deterministic apply script + memory templates under `outputs/<bundle-name>/`. The script:
   - performs preflight sanity (paths, SHAs, deps);
   - calls `python3 outputs/brain_writer.py` for each op (session-start, write, str-replace, protocol-upgrade, self-audit, session-end);
   - exits non-zero on any failure so the chain stays consistent.
3. **User runs the script from macOS terminal**:
   ```bash
   cd /Users/<you>/Projects/<repo>
   bash outputs/<bundle-name>/apply.sh
   ```
   The §0.1 guard passes (path is real-filesystem), the writer runs, the chain advances.
4. **Agent reviews the resulting chain** in the next cowork session via `verify --bit-perfect` output the user pastes back.

**When this pattern fits:** any cluster of BRAIN mutations the agent wants to land as one atomic-ish unit — protocol upgrades (§0.5 + §0.6), bulk content refreshes, doctor-style cleanup passes, multi-memory ingestion of an external corpus.

**When it doesn't:** single-memory writes during exploratory work — the cowork → macOS round-trip is overkill. For those, the user can drop into the macOS terminal directly and run the writer themselves (the `python3 outputs/brain_writer.py write <actor> <path> <content>` form is short).

**Reference apply scripts** in `outputs/`:

- `outputs/apply-bundle-Q.sh` — protocol-upgrade flow (§0.5 + §0.6 follow-on; canonical pattern for future protocol upgrades)
- `outputs/doctor/<date>-cleanup.py` — bulk-cleanup pattern (str_replace many memories in one session, with rollback-on-failure)

### Recommendation

Cowork is the smoothest fit for **automated mode** RIGHT NOW. The trigger phrase from [CHAIN_ORCHESTRATOR.md](./CHAIN_ORCHESTRATOR.md) + the orchestrator runbook give you a single-message kickoff. The agent drives the rest — file reads, interviews, artefact writes, audit loops, brain_writer.py, all of it. You answer ~10-30 HITL questions over ~3 hours of standard-profile work; that's the entire user-facing UX.

Everything in [MANUAL_WORKFLOW.md](./MANUAL_WORKFLOW.md) (manual mode) also works in Cowork if you want the step-by-step driver experience.

---

## Adapter B — Claude Code (CLI)

The native Claude Code experience. Best fit if you live in a terminal.

### Setup (one-time per project)

1. **Symlink AGENTS.md** at the new project's root:

   ```bash
   cd /path/to/new/project
   ln -s /path/to/cyberos/docs/CyberOS-AGENTS-CORE.md AGENTS.md
   ```

   Claude Code auto-loads `AGENTS.md` and `CLAUDE.md` at session start. If you only want Claude Code to see it (not Codex / Cursor), use `CLAUDE.md` instead.

2. **Optional: also symlink CLAUDE.md** (some Claude Code installs check both):

   ```bash
   ln -s AGENTS.md CLAUDE.md
   ```

3. **Bootstrap BRAIN.** The first session detects `PRISTINE` and runs §13.1 silently.

### Per-step shape

Identical to Cowork — the only difference is that Claude Code drops you straight into a terminal-style chat without needing the connected-folder dance. Everything else (file tools, bash, MCP) maps 1:1.

### Recommendation

Use Claude Code if you'd rather work in `tmux` than a desktop chat app. Hooks (`PreToolUse`, etc., per the SRE notebook pattern from `module/agent-patterns/cookbook-deep-dives.md`) are first-class here — you can wire safety checks per Plan v1.1 / M3 directly into the Claude Code config.

---

## Adapter C — Cursor

Same capability footprint as Claude Code but inside an IDE.

### Setup (one-time per project)

1. **Create `.cursor/rules/cyberos-memory.mdc`** at the new project's root:

   ```markdown
   ---
   description: CyberOS Universal Agent Memory Protocol (AGENTS-CORE.md)
   globs: ["**/*"]
   alwaysApply: true
   ---

   <paste contents of cyberos/docs/CyberOS-AGENTS-CORE.md here>
   ```

   Cursor's `.cursor/rules/` folder is the equivalent of Claude Code's auto-loaded `AGENTS.md`.

2. **Run the chain** in Cursor's Agent chat (Cmd+L → ask).

### Per-step shape

Identical to Claude Code; Cursor's file tools and terminal both work. One quirk: Cursor's chat history is per-file by default, not per-session — you may want to use the chat sidebar's "New Session" button between skills to stay clean.

### Recommendation

Use Cursor if you're already pair-coding in it. The MANUAL_WORKFLOW chain runs unchanged.

---

## Adapter D — Codex CLI (OpenAI)

OpenAI's terminal agent. Auto-loads `AGENTS.md` natively (this is the file convention Codex actually defined first).

### Setup (one-time per project)

1. **Symlink AGENTS.md** — same as Claude Code:

   ```bash
   ln -s /path/to/cyberos/docs/CyberOS-AGENTS-CORE.md AGENTS.md
   ```

2. **Run `codex` in the project root.** It'll pick up AGENTS.md automatically.

### Per-step shape

Same as Claude Code. The chain is host-neutral; Codex's tool-call shape differs internally but the user-facing flow is identical: paste SKILL.md, follow the questions, save artefact.

### Recommendation

Use Codex if you'd rather pay OpenAI than Anthropic, or if your team's standard model is GPT-5 / o-series. Behaviour quality of the chain depends on the model; Sonnet 4.6 / Opus 4.7 perform measurably better on the rubric-driven audit-fix loop than smaller models.

---

## Adapter E — Other CLIs (Gemini, OpenCode, Windsurf, Copilot, Aider, Continue, Trae, Kiro)

Generic recipe for any agent CLI that supports file tools + shell:

### Setup

1. **Find the host's auto-load convention** (see ECC's `manifests/` for examples — every host has its own `.foo/<file>` shape).
2. **Either symlink or copy** `cyberos/docs/CyberOS-AGENTS-CORE.md` into the auto-load location.
3. **Bootstrap BRAIN** by asking the agent to follow §13.1.

### Per-step shape

If the host has file tools + shell, the chain runs identically to the recommended hosts above. Hosts without `Skill`-style auto-trigger (most of them) just need the user to paste the SKILL.md content explicitly at the start of each step — that's not a worse experience, just a slightly more verbose one.

### Recommendation

Use whatever CLI your team standardises on. The chain doesn't care.

---

## Adapter F — Claude.ai web / ChatGPT (degraded mode)

For when you can't run a CLI — e.g., you're on a borrowed machine, on mobile, or your project is on a system without a terminal.

### Capabilities lost

- ❌ Can't read your project's filesystem (the agent only sees what you paste / upload).
- ❌ Can't write artefacts to your filesystem (agent emits markdown in chat; you copy-paste into local files).
- ❌ Can't run `brain_writer.py` against your real BRAIN (the audit ledger gap is filled manually after the session).
- ❌ Can't run MCP tools that touch your local environment (e.g., `proj.create_issue` against your Linear instance).

### What still works

- ✅ The interview shape — questions in chat, answers in chat.
- ✅ Markdown generation — the agent produces `project-brief.md` / `prd-*.md` / `fr-*.md` / `impl-plan.md` content as chat output.
- ✅ Audit-loop reasoning — the rubric runs in the agent's head; you read the verdict.

### Setup

1. **Paste AGENTS-CORE.md as the first message** of every new session (or use a Custom GPT / Custom Project to bake it in).
2. **Paste the SKILL.md** of the current step as the second message.
3. **Provide the inputs** (project brief / prior artefacts) as subsequent messages.

### Per-step shape

1. Run the interview / generation in chat.
2. Copy the agent's generated markdown into a local file in your project (`./planning/<date>-<slug>/<artefact>.md`).
3. After all skills run, **on a machine with a terminal**, re-create the audit ledger:

   ```bash
   cd /path/to/project
   python3 outputs/brain_writer.py session-start human:stephen-cheng
   for f in <list of artefact files>; do
     python3 outputs/brain_writer.py write human:stephen-cheng <relpath> <abspath>
   done
   python3 outputs/brain_writer.py session-end human:stephen-cheng
   ```

   This restores chain integrity. The `actor_kind: "human"` makes it explicit these were human-driven (vs. agent-driven) writes.

### Recommendation

Avoid degraded mode for serious work — the manual file shuffling kills the time savings the chain otherwise gives you. Keep this option for exploratory drafts only.

---

## When to switch hosts mid-project

It's fine. The BRAIN audit ledger + the artefacts on disk + CONTEXT.md + .out-of-scope/ are all host-agnostic. You can:

- Start in Cowork for the interview (fast back-and-forth).
- Switch to Claude Code or Cursor for FR-author / spec-to-impl-plan (denser tool-use).
- Drop into Claude.ai web on your phone if you want to draft a quick PRD amendment.

The only constraint: don't run **two hosts simultaneously** against the same `.cyberos-memory/` directory. The `.lock` file in the BRAIN root coordinates write access; concurrent writes from two agents will trigger `op:"rejected" reason:"lock-contention"` at best, and chain corruption at worst.

---

## Picking a host: a quick decision tree

```
Are you working solo or small-team?
├── Yes → Claude Cowork (★ recommended)
│         OR Claude Code (★ if you live in tmux)
│         OR Cursor (★ if you're already in it)
│
└── No, agency / multi-tenant work
    ├── Need audit trail → Claude Code with hooks
    ├── Need OpenAI compliance → Codex CLI
    └── Need IDE → Cursor or Windsurf

Are you on a borrowed machine / mobile?
└── Claude.ai web / ChatGPT (degraded; expect manual file shuffling)
```

For Stephen's next project: **Cowork is the move**. It's what we used to build this whole evolution, it has the best file-tool + bash story for solo work, and the BRAIN sits at `~/Projects/CyberSkill/workbench/.cyberos-memory/` already wired up.

## History

- 2026-05-11 — Initial creation. Author: Claude Opus 4-7 in Cowork session 7. Companion to MANUAL_WORKFLOW.md; resolves the "must I use Claude Code?" question (no — fully host-agnostic).
