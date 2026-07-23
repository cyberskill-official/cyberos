# TASK-IMP-137 — implementation evidence

Implementer: batch/8-audit-hardening worker (gates + install/portability lane), 2026-07-23.
Status frontmatter untouched per shared-tree rules; HITL gates remain the operator's.

## What changed and why

| File | Change | Traces to |
|---|---|---|
| `tools/install/mcp/cyberos-mcp.mjs` | `--http` binds `127.0.0.1` unless the new `--host <addr>` flag widens it; non-loopback + tokenless startup prints a warning naming the exposure (never a secret). `CYBEROS_MCP_TOKEN` set non-empty ⇒ every POST needs exact `Authorization: Bearer <token>` (401 + JSON-RPC error body otherwise); `GET /healthz` stays open; empty token == unset. Token compare hashes both sides to fixed length and uses `crypto.timingSafeEqual` (constant-time-safe in intent, no early-exit substring tricks). `node:http`/`node:crypto` imports moved to module top (no-inline-imports rule). | #1.1 #1.2 |
| `tools/install/mcp/README.md` | Engines truth (`>=24 <25`), new HTTP-mode section: loopback default, `--host`, token contract, explicit empty-token-is-unset semantics (audit ISS-003), no-token-logging note. | #1.1 #1.2 #1.4 |
| `tools/install/bootstrap.sh` | Checksum verifier chooser before the first download: `sha256sum -c` when present, else `shasum -a 256 -c` (the fallback VERIFIES), abort naming BOTH tools when neither exists. | #1.3 |
| `tools/install/build.sh` | Generated payload `package.json` engines `">=24 <25"` exactly (was `>=18`), with the reconciliation rationale in a comment. Also vendors `memory.schema.json` from the canonical package-data copy and `INTEROP.md` when present (TASK-MEMORY-303 coordination — see that section in the CUO-302 evidence file). | #1.4 |
| `tools/install/install.sh` | Vendors `ci/` → `.cyberos/ci/` (the phantom channel is now real; `uninstall.sh`'s wholesale `rm -rf "$CY"` already reclaims it). Vendor step is stage-then-swap: full `cp -R` into `"$CY/<name>.tmp.<nonce>"` first, then (a) first install = one rename, (b) identical re-vendor = fingerprint short-circuit (nothing moves), (c) changed payload = per-path `rename(2)` sync + prune of payload-dropped paths. Stray `*.tmp.*` staging dirs cleaned at next install start. | #1.5 #1.6 |
| `tools/install/README.md` | GitHub Action section rewritten: `uses: ./.cyberos/ci/github-action` with a working workflow example (install step in-job, or `git add -f .cyberos/ci` once); the false "dist path after install" claim removed. | #1.5 |
| `CHANGELOG.md` | Top `## [Unreleased]` entry records all five changes, "breaking" for engines + binding, migration guidance. | #1.7 |
| `tools/install/tests/test_install_portability.sh` (new) | t01–t07, one per AC; t01 asserts the OS-reported bound address AND a refused LAN connect (behavior, not configuration-echo); t03's masked-PATH runs prove verify-not-skip; t06 runs a tight reader across 20 identical reinstalls PLUS a changed-payload upgrade install, then the kill-simulation + stray cleanup. | AC 1–7 |

## Verbatim test output

```
$ bash tools/install/tests/test_install_portability.sh
building scratch payload...
test_install_portability.sh (TASK-IMP-137)
  ok   t01_loopback_default
  ok   t02_bearer_token_enforced
  ok   t03_shasum_fallback_verifies
  ok   t04_engines_unified
  ok   t05_ci_channel_real
  ok   t06_atomic_swap_no_reader_gap
  ok   t07_changelog_five_changes
----
pass=7 fail=0
```

Scratch demos (canonical rebuilt payload, /tmp fixtures):

```
# bind posture (lsof-reported + real LAN connect attempt from 192.168.101.31)
node .cyberos/mcp/cyberos-mcp.mjs --http 28417 &
  TCP 127.0.0.1:28417 (LISTEN)          # not *:28417
  loopback healthz: 200
  LAN connect: REFUSED
node ... --http 28419 --host 0.0.0.0 (tokenless) ->
  cyberos-mcp: WARNING --host 0.0.0.0 binds beyond loopback with NO token. ...

# token auth
tokenless POST: 401 / wrong token: 401 / right token: {"jsonrpc":"2.0","id":1,"result":{}} / healthz open: 200

# bootstrap on a masked PATH carrying shasum but NOT sha256sum (file:// payload)
good archive   -> rc=0, install marker written
corrupted      -> "shasum: WARNING: 1 computed checksum did NOT match"
                  "cyberos bootstrap: ERROR: checksum mismatch - aborting before touching ..." rc=1
neither tool   -> "ERROR: neither sha256sum nor shasum is on PATH - ... refusing to install unverified bits" rc=1

# payload truth
"engines": { "node": ">=24 <25" }      # dist/cyberos/package.json
.cyberos/ci/github-action/action.yml   # present after scratch install
```

## The atomic-vendor finding (material, measured)

The spec's Proposed-Solution sketch (`rm -rf old + mv staged`, later softened to mv-mv) was implemented first and FAILED AC 6 empirically: two separate `mv` processes leave a multi-millisecond absence window (fork/exec dominates the rename syscall), and the tight reader observed **5425 absences across 2,876,255 polls in 20 reinstalls**. The shipped form is the one that actually meets the normative bound in #1.6 ("bounded by rename/move operations, not by copy duration"):

- first install: one whole-tree rename;
- identical re-vendor (every reinstall loop): a content fingerprint short-circuits the swap — nothing moves, nothing can be observed missing;
- changed payload: per-path sync where each FILE is `rename(2)`'d over its destination — a path present in both old and new trees is never absent and never truncated; payload-dropped paths are pruned after.

Re-run after the fix: **0 absences** over the same reader (assertion also requires >1000 polls so the reader provably ran), including one changed-payload upgrade install under the reader (add + update + prune all verified). Trade-off recorded honestly: during a changed-payload sync a reader can briefly observe a mixed-version tree (new file A beside old file B) — absence and truncation are what the spec forbids, and neither occurs; the install lock still serializes writers.

## Deviations / open items for the reviewer

1. **`rollout.sh:23` remains `sha256sum`-only.** The spec scopes #1.3 to `bootstrap.sh`; rollout is the fleet tool and unnamed. Recommend a follow-up task or a one-line adoption of the same chooser.
2. **`docs/deploy/mcp-connector.md`** (outside this lane's ownership) still says the transport "ships UNAUTHENTICATED" — true pre-change, stale now. Flagged for the docs owner; the mcp README carries the current contract.
3. **Wide-bind + token startup line** appends "bearer auth ON" — names the condition only; no token value is ever logged (asserted in t02 against the 401 body).
4. **Sibling-caused failures observed while running neighbor suites** (not caused by this lane, verified by path): `test_e2e_skeleton.sh` t01 fails inside the sibling-modified `tools/install/docs-tools/backlog-mutate.mjs` (T2 HITL-lock, mid-flight); `test_check_version_sync.sh` t07 fails on the sibling-modified `.githooks/pre-commit` referencing `.pre-commit-hooks/awh-gate.sh` in fixtures (T4 CI hardening, mid-flight). Left untouched per shared-tree rules.
