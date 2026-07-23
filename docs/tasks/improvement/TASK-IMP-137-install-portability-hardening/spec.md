---
id: TASK-IMP-137
title: Install portability - MCP loopback+token, shasum fallback, atomic vendor
template: task@1
type: improvement
module: improvement
status: reviewing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-076, TASK-IMP-103, TASK-IMP-140]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.1.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 10
service: tools/install
new_files:
  - tools/install/tests/test_install_portability.sh
modified_files:
  - tools/install/mcp/cyberos-mcp.mjs
  - tools/install/mcp/README.md
  - tools/install/bootstrap.sh
  - tools/install/build.sh
  - tools/install/install.sh
  - tools/install/README.md
  - CHANGELOG.md
source_pages:
  - "tools/install/mcp/cyberos-mcp.mjs:192-219 (--http mode: createServer(...).listen(port) with NO host argument - binds every interface; no authentication on POST /mcp; the served tools include task_install and task_gates, which rewrite the repo and eval shell commands; the header comment defers auth to 'a reverse proxy in production' while the default exposure is the LAN)"
  - "tools/install/bootstrap.sh:23 (grep ... SHA256SUMS | sha256sum -c - : sha256sum is GNU coreutils and absent on stock macOS, whose tool is `shasum -a 256`; the curl|bash channel therefore fails checksum verification on the platform CyberOS is developed on)"
  - "tools/install/build.sh:358 (generated payload package.json: engines { node: '>=18' }) vs tools/install/mcp/package.json ('>=24 <25') vs .nvmrc + tools/caf/.nvmrc + tools/install/mcp/.nvmrc (all 24.18.0) - the payload admits Node 18 while shipping an mcp/ subpackage that demands 24"
  - "tools/install/README.md:158 ('dist/cyberos/ci/github-action/action.yml is a composite action... Point a workflow at it after install.sh has committed .cyberos/ to the repo') vs measured 2026-07-23: install.sh vendors cuo/plugin/mcp/lib/docs-tools + files but never ci/, and the installed .cyberos/ tree has no ci dir - the documented CI channel cannot be used as documented"
  - "tools/install/install.sh:193-196 ('1. vendor the machine by module (replace any prior copy)': rm -rf $CY/cuo $CY/plugin $CY/mcp then cp -R each - between rm and cp-completion any READER of .cyberos/ sees a missing or partial tree; the TASK-IMP-103 .install.lock serializes concurrent installers but does not protect readers)"
source_decisions:
  - "2026-07-23 operator: CyberOS Hardening Plan approved; Phase 2 T7 'Install/portability' authored as an improvement task (plan file cyberos_hardening_plan_49404998; audit findings C4-http + medium portability items)."
  - "2026-07-23 authoring: Node engine reconciliation direction is UP to '>=24 <25' for the generated payload package.json - matching every .nvmrc (24.18.0) and the strictest shipped subpackage (mcp), rather than relaxing mcp to >=18, because nobody tests the payload's tools on 18 and an engines field that promises untested compatibility is the same lie class as an always-green stub. Recorded for the review gate."
  - "2026-07-23 authoring: the GitHub Action channel fix direction is VENDOR ci/ into .cyberos/ci/ (making the documented usage true) plus a README correction, rather than docs-only - the action is small, static, and referencing it from the consumer's committed tree is exactly how composite actions are consumed. Recorded for the review gate."
---

# TASK-IMP-137: Install portability - MCP loopback+token, shasum fallback, atomic vendor

## Summary

Five verified portability/exposure defects in the install channel: the MCP server's `--http` mode binds every interface with zero auth while serving repo-rewriting, shell-running tools; `bootstrap.sh` hardcodes GNU `sha256sum`, breaking checksum verification on stock macOS; the payload's `engines` field admits Node 18 while shipping a subpackage that demands 24; the README documents a GitHub Action channel at a path install never creates; and the vendor step's `rm -rf` + `cp -R` leaves a reader-visible window where `.cyberos/` is missing or partial. This task closes all five: loopback-by-default + optional bearer token, `shasum` fallback, one engine floor, a real `ci/` vendor + docs truth, and stage-then-swap vendoring.

## Problem

All verified first-hand 2026-07-23:

1. **LAN exposure by default (audit C4).** `cyberos-mcp.mjs:219` calls `.listen(port)` with no host, binding `0.0.0.0`. `POST /mcp` accepts unauthenticated JSON-RPC and the tool set includes `task_install` (rewrites the repo) and `task_gates` (runs `run-gates.sh`, which `eval`s configured commands). Anyone on the local network can drive both. The code comment defers auth to a production reverse proxy, but the *default* posture is the exposure.
2. **Checksum verification fails on macOS.** `bootstrap.sh:23` pipes to `sha256sum -c`; stock macOS ships `shasum`, not GNU coreutils. The security step of the curl|bash channel errors out on the platform the project is developed on - inviting users to bypass it.
3. **Engines contradiction.** The generated payload admits `node >=18` (`build.sh:358`); the shipped `mcp/package.json` demands `>=24 <25`; every `.nvmrc` pins 24.18.0. A Node-18 user passes npm's engine check and then runs components nobody has ever tested on 18.
4. **Phantom CI channel.** README:158 tells operators to point a workflow at the composite action "after `install.sh` has committed `.cyberos/` to the repo" - but install.sh never copies `ci/`, so the documented path does not exist in any installed repo.
5. **Partial-vendor window.** Between `rm -rf "$CY/cuo"` and the completed `cp -R`, an agent reading `.cyberos/` (which agents do constantly - it is their entry point) sees a half-machine. The TASK-IMP-103 lock serializes installers only.

## Proposed Solution

**MCP:** `--http` binds `127.0.0.1` by default; a new `--host <addr>` flag opts into wider binding and its help text names the exposure; when the env var `CYBEROS_MCP_TOKEN` is set non-empty, every `POST /mcp` must carry `Authorization: Bearer <token>` (401 otherwise), while `GET /healthz` stays open for probes; binding non-loopback WITHOUT a token prints a loud warning. **bootstrap.sh:** detect `sha256sum` else fall back to `shasum -a 256`; fail with a clear message only when neither exists. **Engines:** the generated payload package.json pins `"node": ">=24 <25"`, matching the .nvmrc floor and the mcp subpackage. **CI channel:** install.sh vendors `ci/` into `.cyberos/ci/` (ownership-marked like the other vendored trees), and README:158's instructions are corrected to reference the installed path with a working `uses:` example. **Atomic vendor:** each vendored subtree is staged as `"$CY/<name>.tmp.<nonce>"` then swapped into place (`rm -rf` old + `mv` staged) so the reader-visible gap per subtree shrinks from the full copy duration to a rename; the existing install lock continues to serialize installers. A new suite covers all five behaviors.

## Alternatives Considered

- **Full TLS/OAuth on the MCP HTTP mode.** Rejected for this task: the connector is a local-first developer channel; loopback default + optional bearer token closes the unauthenticated-LAN hole without dragging in cert management. Production deployments keep the documented reverse-proxy story, now as defense-in-depth rather than the only defense.
- **Deny `--http` entirely unless a token is set.** Rejected: loopback-only unauthenticated use (the overwhelmingly common local case) is not the vulnerability; the LAN binding is. Requiring tokens for localhost adds friction with no threat-model gain.
- **Relax mcp/package.json to `>=18` instead of raising the payload floor.** Rejected: no CI or developer machine tests Node 18 (every .nvmrc is 24.x); an engines field is a compatibility *promise*, and promising untested compatibility is the stub-workflow lie in a different file. Raising the floor is honest; users on older Node get a clear npm engines error instead of a runtime surprise.
- **Docs-only fix for the CI channel (point at the npm package path).** Rejected: a composite action consumed from the repo's own committed tree (`.cyberos/ci/github-action`) is the standard, zero-download pattern and matches what the README already promised; making the promise true is smaller than re-teaching it.
- **Whole-`.cyberos/` staged swap for atomicity.** Rejected: `.cyberos/` contains machine-local state that must survive installs (`config.yaml`, `gates.env` backups, `memory/store/`); swapping the whole dir risks the config-wipe class. Per-subtree swaps touch exactly the machine-owned trees.

## Success Metrics

- Primary: by the next CyberOS release - `--http` with no flags refuses remote connections (bind 127.0.0.1, verified by connect attempt from a non-loopback source address in the test harness); with `CYBEROS_MCP_TOKEN` set, tokenless POSTs get 401; `bootstrap.sh` completes checksum verification on a machine with only `shasum`; the payload package.json engines equals `>=24 <25`; `.cyberos/ci/github-action/action.yml` exists after a scratch install; and no reader observes a missing vendored subtree during an install loop (sampled reader in the test). Baselines today: all five fail.
- Guardrail: stdio MCP mode (the default channel) is byte-for-byte unaffected; existing install suites (`test_install_hygiene.sh`, `test_install_lock.sh`, `test_e2e_skeleton.sh`) stay green.

## Scope

In scope: the five fixes above, their README/mcp-README corrections, the new test suite, CHANGELOG.

### Out of scope / Non-Goals

- TLS termination, OAuth flows, or multi-user token management for the MCP connector (reverse-proxy guidance remains the production story).
- The fail-closed gates behavior and gates.env header - TASK-CUO-302.
- Uninstall-side preservation of `.cyberos/ci/` (uninstall completeness is TASK-IMP-126's domain; the new tree is ownership-marked so existing uninstall logic handles it).
- The G16 reinstall-idempotency benchmark definition - TASK-IMP-140 (its checker exercises the atomic-vendor behavior this task ships; soft forward reference, no cycle).

## Dependencies

None blocking. TASK-IMP-076 (done) shipped the MCP server and its `--http` mode; TASK-IMP-103 (done) shipped the installer lock whose reader gap this task closes the other half of. TASK-IMP-140's G16 gate builds on the atomic-vendor guarantee - forward reference only.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill in Cursor, as the task-authoring wave of the 2026-07-23 hardening plan.
- **Scope:** the bind call, the missing auth check, the `sha256sum` hardcode, the three engines values, the phantom `ci/` path, and the rm/cp window were each verified by direct source read at HEAD; the payload files list (`"ci"` present in npm `files`, absent from install.sh's vendor list) was cross-checked both sides.
- **Human review:** the hardening plan was operator-approved 2026-07-23; the engines-up and vendor-ci decisions are recorded in `source_decisions` for the review acceptance gate.

## 1. Description (normative)

- 1.1 `cyberos-mcp.mjs --http` MUST bind `127.0.0.1` when no host is specified; a new `--host <addr>` argument MUST be the only way to bind any other address, and starting non-loopback without a token MUST print a warning naming the exposure.
- 1.2 When the environment variable `CYBEROS_MCP_TOKEN` is set non-empty, every `POST /mcp` request MUST be rejected 401 (JSON-RPC error body) unless it carries `Authorization: Bearer <token>` with an exact token match; `GET /healthz` MUST remain unauthenticated. Token comparison MUST be constant-time-safe in intent (no early-exit substring tricks) though the threat model is LAN, not timing labs.
- 1.3 `bootstrap.sh` MUST verify the payload checksum via `sha256sum -c` when available, else `shasum -a 256 -c`, and MUST abort with a message naming both tools when neither exists. The fallback MUST verify, not skip - absence of GNU coreutils is not permission to trust the network.
- 1.4 The payload package.json generated by `build.sh` MUST declare `"engines": { "node": ">=24 <25" }`, matching the repo's `.nvmrc` floor and the shipped mcp subpackage.
- 1.5 `install.sh` MUST vendor the payload's `ci/` tree into `.cyberos/ci/` (ownership-marked consistently with the other vendored trees), and `tools/install/README.md`'s GitHub Action section MUST reference the installed path with a valid `uses: ./.cyberos/ci/github-action` example.
- 1.6 Each vendored subtree replacement in `install.sh` MUST be staged (`cp -R` into `"$CY/<name>.tmp.<nonce>"` first) and swapped into place so the reader-visible absence window per subtree is bounded by directory rename/move operations, not by copy duration. Stray `*.tmp.*` staging dirs from killed installs MUST be cleaned at the next install start.
- 1.7 `CHANGELOG.md` MUST record all five changes, marking the engines raise and the loopback default as breaking for consumers who relied on Node 18 or LAN binding.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - `--http` with no host accepts loopback connections and refuses non-loopback (asserted via the OS-reported bound address AND a failed connect from a secondary address where the harness supports it); `--host 0.0.0.0` binds wide and tokenless startup prints the exposure warning - test: `tools/install/tests/test_install_portability.sh::t01_loopback_default`
- [ ] AC 2 (traces_to: #1.2) - with `CYBEROS_MCP_TOKEN=secret`: tokenless POST /mcp gets 401, wrong token gets 401, correct Bearer succeeds, GET /healthz succeeds tokenless - test: `tools/install/tests/test_install_portability.sh::t02_bearer_token_enforced`
- [ ] AC 3 (traces_to: #1.3) - on a PATH without `sha256sum` but with `shasum`, bootstrap's verification step succeeds against a good archive and FAILS against a corrupted one (the fallback verifies); with neither tool, it aborts naming both - test: `tools/install/tests/test_install_portability.sh::t03_shasum_fallback_verifies`
- [ ] AC 4 (traces_to: #1.4) - a scratch payload's package.json engines equals `>=24 <25` exactly - test: `tools/install/tests/test_install_portability.sh::t04_engines_unified`
- [ ] AC 5 (traces_to: #1.5) - after a scratch install, `.cyberos/ci/github-action/action.yml` exists, and README's section contains the `uses: ./.cyberos/ci/github-action` form with no remaining claim that dist paths work post-install - test: `tools/install/tests/test_install_portability.sh::t05_ci_channel_real`
- [ ] AC 6 (traces_to: #1.6) - a reader loop polling `.cyberos/cuo/ship-tasks.md` existence during 20 reinstall iterations observes zero absences; a simulated kill between stage and swap leaves the OLD tree intact and the next install cleans the stray staging dir - test: `tools/install/tests/test_install_portability.sh::t06_atomic_swap_no_reader_gap`
- [ ] AC 7 (traces_to: #1.7) - CHANGELOG's top entry mentions all five changes and the word "breaking" for engines + binding - test: `tools/install/tests/test_install_portability.sh::t07_changelog_five_changes`

## 3. Edge cases

- **Agent UIs that connected to the LAN-bound port yesterday:** after upgrade they must either run on the same host (loopback) or pass `--host` deliberately - the CHANGELOG breaking note plus the startup warning carry the migration; silence would re-create the exposure by habit.
- **Token set but empty (`CYBEROS_MCP_TOKEN=""`):** treated as unset (no auth), because an empty bearer token is unusable as a credential; the mcp README documents this explicitly.
- **`/healthz` information disclosure:** the health body includes server name/version only (as today); the token gate deliberately excludes it so probes work, and no tool metadata is served there.
- **macOS with Homebrew coreutils installed:** `sha256sum` exists and wins - the fallback ordering preserves today's behavior wherever it already worked.
- **Node 18 consumer pinned by their own CI:** npm refuses install with a clear engines error - the intended outcome; the CHANGELOG names the floor and the .nvmrc value to adopt.
- **Kill mid-swap (between rm of old and mv of staged):** the window is the pathological remnant - the test's kill simulation targets stage-complete/pre-swap (old tree intact); a kill inside the rm+mv pair itself is bounded by two syscalls and the next install's staging cleanup + full re-vendor restores the machine; the install lock keeps a second installer out of the gap either way.
- **Security-class:** this task is itself a security fix (default exposure removal + verified checksums). The bearer token lives in an env var, never in repo files; tests use throwaway values; no token is logged (the warning names the CONDITION, not the secret).
