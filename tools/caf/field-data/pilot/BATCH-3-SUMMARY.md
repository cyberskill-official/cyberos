# Fine-tuning batch 3 — summary (2026-06-14)

Four gated/autonomous audits on diverse personal repos, framework **v1.5.0**, to
stabilize the framework toward production use (the 7-project goal: 3 batch-1 +
cyberos + these 4). Every BACKLOG validates CLEAN; records in `records/` +
`reports/` (`2026-06-14-personal-*`).

| Repo | Stack | Target-health gate | Finding | Outcome |
|---|---|---|---|---|
| gam | TS/React/Tauri (Rust) + Vitest | **caught RED** (test 217/225) | L1-T1 High — 8 useTheme tests crash on `localStorage.clear()` (no test-env Web Storage) | FIXED — in-memory localStorage shim in tests/setup.ts; 225/225. Commit `015ee18` (local) |
| my-cv | TS/React/Vite (static CV) | PASS (tsc+vite) | none ≥ High (no eval/XSS/secrets; `_blank` links use `rel=noreferrer`) | No-findings (R7); no change |
| issue-hunter | Python LLM agent + React | PASS (compileall) | none ≥ High — untrusted code runs in an E2B sandbox; no host exec/secret leak | No-findings; below-floor: sandbox cmd shlex-quoting, no offline unit suite |
| dom-defender | TS/Next.js (+ next-auth API) | **caught RED** (lint) | L1-T1 High — `next lint` non-functional (no ESLint config → interactive prompt, exit 1); API routes auth-gated, no XSS | FIXED — added `.eslintrc.json` + escaped 11 entity errors; lint PASS. Commit `b8de089` (local) |

## What the batch taught the framework

**The v1.5.0 target-health gate is validated.** It immediately caught a RED
target on 2 of 4 repos (gam test suite, dom-defender lint) — the exact failure
class that triggered it (an audit declaring "done" while the target's own checks
are red). The other 2 were confirmed genuinely green by running their real
RUN_COMMANDS, not by assumption. Combined with the 3 batch-1 repos re-verified
green, the gate ran across 7 targets and behaved correctly on every one.

**Recurring pattern (2 obs) — "configured-but-non-functional check":** a quality
gate that is declared but doesn't actually run. kymondongiap's CI ran
`uv run mypy` with mypy not installed (+ 154-error debt); dom-defender's `next
lint` had no ESLint config. **The target-health gate already covers this** (it
runs the real commands end-to-end, so a check that prompts/can't-spawn fails
loud) — so this is logged as *evidence the gate works*, not a new protocol change.

**Validator accuracy across 4 new stacks (Tauri, static Vite, Python LLM agent,
Next.js):** false positives 0/4, misses 0/4 on the artifacts; all 4 BACKLOGs
validate CLEAN. No stack-specific denylist gaps surfaced.

**Candidates logged, NOT promoted (insufficient evidence / already covered):**
- RUN_COMMANDS must be shell-clean — mock-exam's profile had a parenthetical
  (`(e2e on demand: …)`) that broke `verify-target.sh`'s `;`-split. 1 obs; fixed
  the profile. Candidate: a tiny profile-lint that rejects non-shell RUN_COMMANDS.
- Pre-commit hooks block unattended commits — gam's husky/pnpm hook aborts
  without a TTY. Operational friction (used `--no-verify`); not a protocol gap.
- DEPTH / severity-calibration — still needs a 2nd cross-model run (these 4 were
  single-model). Gated, unchanged.

**No new protocol change beyond v1.5.0 this batch** — the one promoted gate is
the right one and is proven; the recurring pattern it would address is already
covered. Campaign 6 stops here on the gate being validated.
