# auto-work-harness — Stage 0/1 adoption for gam

This folder wires gam into the [auto-work-harness](https://github.com/zintaen/auto-work-harness)
agent-hardening toolkit. It is **additive** — gam's own `.agent/` rules, skills, and
`.simple-git-hooks.json` are untouched. The harness Claude Code hooks are separate from
gam's git hooks (they fire during an agent's tool use, not on commit/push).

## What's here

| File                       | Purpose                                                                                                                                           |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `../.claude/settings.json` | Fires the Stage-0 hooks (paths point at `~/Projects/auto-work-harness`).                                                                          |
| `gate.sh`                  | The evidence gate the Stop hook runs: `pnpm lint && pnpm test && tsc --noEmit`. The agent can't end its turn while this is red.                   |
| `policy.json`              | Makes test files read-only during agent runs (anti reward-hacking). Reads still allowed. gam's `.env` is already blocked by the harness defaults. |
| `goldenset.yaml`           | Stage-1 golden set (vitest, lint, typecheck) for the CI regression gate.                                                                          |
| `eval-baseline.json`       | **Generate this on your Mac** (see below), then commit it.                                                                                        |

## One-time setup (on your Mac)

```bash
# 0. PREREQUISITE: gam's own deps must be installed — the gate runs gam's real
#    toolchain (vitest/eslint/tsc). If node_modules is empty or pnpm errors with
#    ERR_PNPM_BROKEN_METADATA_JSON / registry timeouts, clean the store and reinstall:
cd ~/Projects/Personal/gam
pnpm store prune
pnpm install                 # needs a stable connection to registry.npmjs.org
#    (if it keeps timing out: pnpm install --network-concurrency 1 --fetch-timeout 120000)

# 1. install the harness CLI once (gives you `awh`)
pip install -e ~/Projects/auto-work-harness

# 2. SANITY CHECK first — run the gate's commands directly so you see output + timing
#    (awh eval captures output, so confirm these pass and aren't interactive):
cd ~/Projects/Personal/gam
pnpm test && pnpm lint && pnpm exec tsc --noEmit -p tsconfig.json

# 3. record gam's baseline. 1 seed (the checks are deterministic); each task has a
#    timeout in goldenset.yaml so a hung command is bounded, not forever.
awh eval .awh/goldenset.yaml --seeds 1 --out .awh/eval-baseline.json
git add .awh .claude && git commit -m "chore: adopt auto-work-harness Stage 0/1"
```

> If step 2 hangs, that command (likely a watch mode or an interactive prompt) is the
> culprit, not the harness — fix/replace it in `.awh/gate.sh` and `goldenset.yaml`.
> `awh eval` itself now kills any command that exceeds its `timeout_sec`.

## CI regression gate (add to gam's workflow, runs on macOS runners)

```bash
awh eval .awh/goldenset.yaml --seeds 1 --baseline .awh/eval-baseline.json --max-regression 0.0
```

## Verify the gate locally

```bash
# destructive command is blocked (prints a reason, exit 2):
echo '{"tool_name":"Bash","tool_input":{"command":"rm -rf /"}}' \
  | python3 ~/Projects/auto-work-harness/harness/stage0_verification/hooks/pretooluse_deny.py; echo $?
# reading gam's .env is blocked:
echo '{"tool_name":"Read","tool_input":{"file_path":".env"}}' \
  | python3 ~/Projects/auto-work-harness/harness/stage0_verification/hooks/pretooluse_deny.py; echo $?
```

## Notes

- `gate.sh` excludes `tauri build` and Playwright e2e (host/display-bound). Run those
  separately. Swap in `pnpm ready` if that's your canonical check.
- Tighten further by running agent sessions inside the harness `sandbox/` (default-deny
  egress + read-only test mounts) for a hard guarantee beyond the chmod/hook speed bump.
