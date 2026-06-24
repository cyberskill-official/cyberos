# Runbook: slice 0 (P0 security plus first hardening), branch auto/gam-absorb

This is the execution runbook for the first slice of the enterprise roadmap (see ENTERPRISE-HARDENING-AND-CYBEROS-ABSORPTION.md). It covers the steps that must run on your Mac, because they need the native toolchain (pnpm, Tauri, the awh CLI) or because they handle a real signing secret that must not pass through a sandbox.

## Session progress (2026-06-23, run on the Mac)

Done and verified green this session:

- Tauri updated to latest. JS `@tauri-apps/api` 2.11.1 and `@tauri-apps/cli` 2.11.3 (already latest via the dep bump); Rust crates bumped via cargo: `tauri` 2.10.2 to 2.11.3, `tauri-build` 2.5.5 to 2.6.3, plus dialog 2.7.1, fs 2.5.1, opener 2.5.4, updater 2.10.1, wry 0.54.2 to 0.55.1. `cargo check` clean, all 66 Rust tests pass.
- The gate is GREEN for the first time. Three root causes were fixed: typecheck failed on a TypeScript 6 `baseUrl` deprecation (removed `baseUrl`, added `src/vite-env.d.ts` for CSS types); the 11.6k lint errors were generated `src-tauri/gen/**` files now ignored plus a `lint:fix` pass; and 187 test failures were caused by `NODE_ENV=production` in the shell loading React's production build, now neutralized in `vitest.config.ts`. Full gate: lint 0, tsc 0, vitest 225/225.
- Step B below is done. `.awh/eval-baseline.json` re-recorded at weighted pass@1 = 100% (vitest, lint, typecheck all 1.0). The old baseline was 0.6.

Still to do (below): Step A (rotate the updater key, needs your secret handling), Step C (Apple secrets, optional), Step D (review and push; the gate is already green).

## What is already done (in the working tree, branch auto/gam-absorb, not committed)

Four file edits, all outside the gated source so they cannot regress lint, test, or typecheck:

- `.claude/settings.json`: harness hook paths no longer hardcode `/Users/stephencheng`. They resolve through `${AWH_HOME:-$HOME/Projects/auto-work-harness}`. On your Mac this resolves to the same path as before. After CyberOS absorption, set `AWH_HOME` to the vendored `tools/awh`.
- `docs/CODEBASE.md`: corrected drift. 20 to 21 IPC commands, added the missing `set_group_color` to the command table, test counts now read 225 frontend plus 66 Rust, and the crash-log path is now the real `com.github.zintaen.gam/crash.log` under `dirs::data_dir()` rather than `~/.gam/crash.log`.
- `pnpm-workspace.yaml`: removed the stray templated line `'@nestjs/core': set this to true or false` (NestJS is not a gam dependency).
- `.github/workflows/release.yml`: added the six Apple Developer ID env vars to the tauri-action step. They stay empty and the build stays green until you add the repo secrets, then macOS dmg builds sign and notarize automatically.

There is also a pre-existing staged dependency bump on this branch that was already in the tree before this work (Tauri 2.11.0 to 2.11.1, @types/node, puppeteer, lockfile). It is unrelated to this slice. Decide separately whether to include it in the same commit.

## Step A. Rotate the updater signing key (do first, the current key is compromised)

The current `.env` holds a live `TAURI_SIGNING_PRIVATE_KEY` with the password `[redacted]`. Treat it as compromised and rotate.

Important consequence to decide on first. The updater public key is embedded in every released binary. Already-installed copies of gam (v1.0.11 and earlier) carry the old public key and will only accept updates signed by the old private key. After rotation, the next release is signed by the new key, so existing installs will reject the auto-update and stay on their current version until users reinstall once. New installs from the next release onward auto-update normally. For an app at gam's scale this is acceptable with a one-line note in the release. The alternative, keeping a known-compromised key, leaves a silent auto-update RCE path open, so rotation is the right call.

```bash
cd ~/Projects/Personal/gam

# 1. Generate a fresh keypair with a STRONG password (not [redacted]).
pnpm tauri signer generate -w ~/.tauri/gam_updater.key
# This prints the public key and writes the private key to the path above.
# Copy the public key it prints.

# 2. Put the new public key into the updater config.
#    Edit src-tauri/tauri.conf.json -> plugins.updater.pubkey -> paste the new value.

# 3. Set the CI secrets (private key contents + the new password).
gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/gam_updater.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD   # paste the new strong password when prompted

# 4. Remove the plaintext secret from the working tree and store it in your manager.
#    (.env is already gitignored and was never committed, confirmed.)
rm .env
#    Keep a copy of ~/.tauri/gam_updater.key and the password in 1Password / your secret manager.
```

Verify: the new `pubkey` is in `tauri.conf.json`, `.env` is gone, and `gh secret list` shows both TAURI secrets updated.

## Step B. Re-baseline the awh gate on a green tree

The committed `.awh/eval-baseline.json` was captured while lint and typecheck were failing (both score 0.0). A regression gate built on it tolerates lint and type errors. Re-capture on a verified-green tree.

```bash
cd ~/Projects/Personal/gam
pnpm install                      # if node_modules is stale; uses the committed lockfile

# 1. Confirm the three gate commands are actually green now, one at a time.
pnpm test                         # expect 225 passing
pnpm lint                         # expect no issues
pnpm exec tsc --noEmit -p tsconfig.json   # expect no errors
# If any of these is red, fix it before baselining. Do not baseline a red tree.

# 2. Re-record the baseline (1 seed; the checks are deterministic).
pip install -e ~/Projects/auto-work-harness     # if awh is not already installed
awh eval .awh/goldenset.yaml --seeds 1 --out .awh/eval-baseline.json

# 3. Confirm the new baseline is fully green.
python3 -c "import json;d=json.load(open('.awh/eval-baseline.json'));print('weighted_pass_at_1=',d['weighted_pass_at_1'])"
# expect weighted_pass_at_1 = 1.0
```

## Step C. (Optional now, needed for signed macOS releases) Apple signing secrets

The workflow is already wired. To activate macOS signing and notarization, add these six repo secrets. An Apple Developer Program membership is required (99 USD per year, available in Vietnam).

| Secret                     | What it is                                                                           |
| -------------------------- | ------------------------------------------------------------------------------------ |
| APPLE_CERTIFICATE          | base64 of your exported "Developer ID Application" certificate (.p12)                |
| APPLE_CERTIFICATE_PASSWORD | the password you set on that .p12 export                                             |
| APPLE_SIGNING_IDENTITY     | the identity string, for example "Developer ID Application: Trinh Thai Anh (TEAMID)" |
| APPLE_ID                   | your Apple ID email                                                                  |
| APPLE_PASSWORD             | an app-specific password generated at appleid.apple.com                              |
| APPLE_TEAM_ID              | your 10-character Apple Team ID                                                      |

```bash
# Example: encode the .p12 and set the certificate secret.
base64 -i DeveloperIDApplication.p12 | gh secret set APPLE_CERTIFICATE
gh secret set APPLE_CERTIFICATE_PASSWORD
gh secret set APPLE_SIGNING_IDENTITY
gh secret set APPLE_ID
gh secret set APPLE_PASSWORD
gh secret set APPLE_TEAM_ID
```

Windows signing is a separate decision (Azure Trusted Signing is geo-limited and may not accept a Vietnam-registered entity; an OV certificate is the geography-independent fallback). See decision 2 in the roadmap. macOS notarization is the higher-impact, unblocked move, so start there.

## Step D. Gate, commit, push

```bash
cd ~/Projects/Personal/gam
git branch --show-current        # should be auto/gam-absorb

# Stage this slice's files (the four edits plus the two new docs).
git add .claude/settings.json docs/CODEBASE.md pnpm-workspace.yaml .github/workflows/release.yml \
        docs/ENTERPRISE-HARDENING-AND-CYBEROS-ABSORPTION.md docs/RUNBOOK-SLICE-0.md
# If you also rotated the key, include the config:
git add src-tauri/tauri.conf.json
# Decide whether to include the pre-existing dep bump (package.json, pnpm-lock.yaml)
# in this commit or a separate one.

# Run the gate one more time; do not commit while it is red.
pnpm lint && pnpm test && pnpm exec tsc --noEmit -p tsconfig.json

git commit -m "chore(p0): rotate updater key, portable harness paths, fix doc/config drift, wire macOS notarization"
git push -u origin auto/gam-absorb
```

## Done criteria for slice 0

- New updater keypair in place, old `.env` deleted, secrets in CI and in a manager, release notes mention a one-time manual reinstall for existing users.
- `.awh/eval-baseline.json` re-recorded with weighted_pass_at_1 = 1.0 on a green tree.
- The four file edits committed on auto/gam-absorb with the gate green.
- macOS notarization secrets added (or consciously deferred), Windows signing decision recorded.

After this slice, the next block is the rest of P1 (capability scoping, Rust integration and Tauri-shell e2e tests, the shell-history consent surface), then the P2 absorption work.
