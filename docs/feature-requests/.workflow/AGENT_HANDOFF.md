# Cross-Agent Handoff Packets

Long FR-drain runs must be resumable by Codex, Claude Code, Cursor, Cline, or another capable local agent without relying on chat context. The handoff unit is a generated packet under `target/cuo-workflow/handoffs/`.

## Generate

Run this before an agent reaches usage limits, before switching agents, or after routing back an FR:

```bash
python3 scripts/agent_handoff.py \
  --reason usage-limit \
  --active-fr FR-AI-022 \
  --note "Current phase, latest tests, and any blockers."
```

The command writes:

- `HANDOFF.md` — human-readable state and rules.
- `RESUME_PROMPT.md` — prompt to paste into the next agent.
- `STATE.json` — machine-readable branch, HEAD, dirty state, active FR, and eligible queue.
- `git-status.txt`, `diff-stat.txt`, `staged-diff-stat.txt`, `recent-commits.txt`, `ready-queue.txt` — verification context.

`target/` is gitignored, so packets are operational state. Commit the generator and workflow rules, not the generated packets.

## Resume

The next agent must read the packet before editing, run `git status --short --branch`, and continue the `chief-technology-officer/ship-feature-requests` workflow. If `active_fr` is set and the worktree is dirty, finish or route back that FR before selecting another FR.

The per-FR commit rule still applies: exactly one commit for each FR that reaches `done`, including code, tests, docs, BACKLOG status, and FR frontmatter. Exclude `.cyberos-memory/` and `target/` artifacts.

## Switch Back

When switching back to Codex, paste the packet's `RESUME_PROMPT.md`. Codex should treat `STATE.json` and the current worktree as authoritative, not stale chat context.
