# Cross-Agent Handoff Packets

Long FR-drain runs must be resumable by Codex, Claude Code, Cursor, Cline, or another capable local agent without relying on chat context. The handoff unit is a generated packet under `target/cuo-workflow/handoffs/`.

The coordination unit is an advisory local claim at `target/cuo-workflow/agent-session/CLAIM.json`. Packets describe the work; claims say which agent is currently allowed to edit. Both live under `target/` and must not be committed.

## Generate

Run this before an agent reaches usage limits, before switching agents, or after routing back an FR:

```bash
python3 scripts/agent_handoff.py create \
  --reason usage-limit \
  --agent codex \
  --handoff-to claude-code \
  --active-fr FR-AI-022 \
  --note "Current phase, latest tests, and any blockers."
```

The command writes:

- `HANDOFF.md` — human-readable state and rules.
- `RESUME_PROMPT.md` — prompt to paste into the next agent.
- `STATE.json` — machine-readable branch, HEAD, dirty state, active FR, and eligible queue.
- `git-status.txt`, `diff-stat.txt`, `staged-diff-stat.txt`, `recent-commits.txt`, `ready-queue.txt` — verification context.

`target/` is gitignored, so packets are operational state. Commit the generator and workflow rules, not the generated packets.

The pre-existing shorthand still works and is treated as `create`:

```bash
python3 scripts/agent_handoff.py --reason usage-limit --active-fr FR-AI-022
```

## Resume

The next agent should validate the latest packet, claim the local session, then continue the `chief-technology-officer/ship-feature-requests` workflow:

```bash
python3 scripts/agent_handoff.py resume --agent claude-code --packet latest
git status --short --branch
```

`resume` prints the packet's `RESUME_PROMPT.md` and writes `CLAIM.json` unless `--no-claim` is passed. If another unexpired claim exists, the command fails unless the operator intentionally uses `--force`.

If `active_fr` is set and the worktree is dirty, finish or route back that FR before selecting another FR.

The per-FR commit rule still applies: exactly one commit for each FR that reaches `done`, including code, tests, docs, BACKLOG status, and FR frontmatter. Exclude `.cyberos-memory/` and `target/` artifacts.

## Validate

Use validation whenever the handoff packet was generated in another tool or before a long break:

```bash
python3 scripts/agent_handoff.py validate --packet latest
python3 scripts/agent_handoff.py status
```

Validation checks the packet shape, branch, HEAD drift, dirty-state drift, and active claim state. HEAD or dirty-state drift is a warning by default and an error with `--strict`; branch mismatch is always an error.

## Release

When an agent is done editing or is handing the run to another agent, release its claim:

```bash
python3 scripts/agent_handoff.py release --agent claude-code
```

If usage limits are approaching, generate a fresh packet first and then release:

```bash
python3 scripts/agent_handoff.py create \
  --reason usage-limit \
  --agent claude-code \
  --handoff-to codex \
  --active-fr FR-OBS-003 \
  --note "FR phase, commands run, failures, and next command." \
  --release-claim
```

## Switch Back

When switching back to Codex, run:

```bash
python3 scripts/agent_handoff.py resume --agent codex --packet latest
```

Paste the printed prompt into Codex. Codex should treat `STATE.json`, `CLAIM.json`, and the current worktree as authoritative, not stale chat context.

## Emergency Recovery

If Codex hits a hard usage limit before creating a fresh packet, the next agent can create a recovery packet from the current worktree:

```bash
python3 scripts/agent_handoff.py create \
  --reason emergency-recovery \
  --agent claude-code \
  --note "Codex stopped before a fresh handoff packet; inspect git-status and diffs first."
python3 scripts/agent_handoff.py resume --agent claude-code --packet latest
```

The recovery agent must inspect `git-status.txt`, `diff-stat.txt`, and `git diff` before editing. If an in-flight FR cannot be identified, do not mark any FR done; route the uncertain work back or ask the operator.
