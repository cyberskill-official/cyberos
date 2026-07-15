---
id: TASK-APP-003
title: "macOS App Store distribution — sandboxed build, entitlements, notarization split, ASC submission"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: app
priority: p1
status: done
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-07-12
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-APP-001, TASK-APP-004, TASK-APP-005, TASK-APP-006]
depends_on: []
blocks: []
source_pages:
  - apps/desktop/src-tauri/tauri.conf.json
  - .github/workflows/release.yml
source_decisions: []
language: rust 1.81 (Tauri v2), bash (CI), swift/objc n/a (no native macOS code required)
service: apps/desktop/src-tauri
new_files:
  - apps/desktop/src-tauri/Entitlements.mas.plist
  - apps/desktop/src-tauri/Entitlements.mas.inherit.plist
  - apps/desktop/src-tauri/tauri.mas.conf.json
  - .github/workflows/release-mas.yml
  - docs/deploy/mac-app-store-submission.md
modified_files:
  - apps/desktop/src-tauri/tauri.conf.json
  - .github/workflows/release.yml
allowed_tools:
  - Tauri CLI (`tauri build --config tauri.mas.conf.json`)
  - Apple `productbuild`, `pkgbuild`, `codesign` (macOS CI runner only)
  - Apple `xcrun altool` / Transporter CLI for App Store Connect upload
  - App Store Connect API key (JWT) auth in CI, stored as a GitHub Actions secret Stephen provisions
disallowed_tools:
  - Any tool that would create the Apple Developer Program enrollment, App Store Connect record, or provisioning profile on Stephen's behalf
  - Any tool that would enter Apple ID credentials or App Store Connect API key material into a non-secret-manager location
effort_hours: 24
subtasks:
  - "Author Entitlements.mas.plist + Entitlements.mas.inherit.plist scoped to CyberOS's actual sandboxed capabilities (2h)"
  - "Author tauri.mas.conf.json overlay: bundle.macOS.entitlements, signingIdentity=3rd Party Mac Developer Application, provider short name (2h)"
  - "Audit every Tauri IPC command + Rust backend call for App Sandbox compatibility; enumerate blocked syscalls (6h)"
  - "Build local .pkg via productbuild + pkgbuild against the sandboxed .app, verify with `codesign --verify --deep` and `spctl -a -t install` (4h)"
  - "Wire release-mas.yml: separate CI job gated on repo variable MAS_RELEASE=true, uses App Store Connect API key auth (4h)"
  - "Write docs/deploy/mac-app-store-submission.md answer sheet (export compliance, content rights, age rating, macOS-specific privacy usage strings) (3h)"
  - "Dry-run upload to App Store Connect TestFlight-equivalent (macOS build processing) once Stephen has created the macOS app record (3h)"
risk_if_skipped: "CyberOS macOS users can only obtain the app via the GitHub Releases DMG (Developer ID signed, notarized). This works today and is not blocked by anything in this task, but it excludes CyberOS from macOS App Store search/discovery, sandboxed-only enterprise MDM deployment channels that require App Store or Apple Business Manager custom app distribution, and the trust signal a Mac App Store listing gives non-technical Vietnamese SMB buyers evaluating CyberSkill's own product. Deferring this task costs discoverability, not functionality — the existing Developer ID distribution path is not deprecated by anything here."
---

## §1 — Description

1. CyberOS's Tauri desktop build **MUST** gain a second, App Store–specific build target (`tauri.mas.conf.json`, an overlay merged over `tauri.conf.json`) that is fully independent of the existing Developer ID / GitHub Releases build target. The existing `apps/desktop/src-tauri/tauri.conf.json` **MUST NOT** be mutated in a way that changes the Developer ID build's behavior — the two targets diverge only in `bundle.macOS.entitlements`, `bundle.macOS.signingIdentity`, and `bundle.macOS.provider Short Name` [sic — see §11 note on case], and in the packaging step (`.dmg` vs `.pkg`).

2. The Mac App Store build **MUST** run inside the macOS App Sandbox (`com.apple.security.app-sandbox = true` in `Entitlements.mas.plist`). Every filesystem path, network destination, and IPC surface CyberOS's Rust backend touches **MUST** be enumerated (§3) and either (a) fall inside the sandbox container automatically, (b) be covered by an explicit entitlement, or (c) be removed/gated behind a "Mac App Store build" feature flag if no sandbox-compatible equivalent exists.

3. `apps/desktop/src-tauri/Entitlements.mas.inherit.plist` **MUST** exist and be attached to every Tauri-spawned child process (Tauri's sidecar / shell-plugin binaries, if CyberOS uses any) via the `com.apple.security.inherit` entitlement, per Apple's requirement that sandboxed apps propagate sandboxing to their own subprocesses.

4. The build **MUST** produce a signed, App Store–distribution `.pkg` installer (via `productbuild`/`pkgbuild`), not the `.dmg` used by the Developer ID channel. App Store Connect ingestion (`xcrun altool --upload-app` or Transporter) accepts `.pkg` only for macOS.

5. CI **MUST** gate the Mac App Store build behind a repo variable `MAS_RELEASE=true` (mirroring the existing `MOBILE_RELEASE` pattern used for Capacitor mobile jobs — see `apps/web/capacitor.config.ts`), defaulting to off, so this task ships inert until Stephen has an Apple Developer Program enrollment, a macOS App Store Connect app record, and a "3rd Party Mac Developer Application" + "3rd Party Mac Developer Installer" signing identity pair.

6. Signing for this channel **MUST** use certificate types distinct from the Developer ID certificate already used for the GitHub Releases DMG (`Developer ID Application: <team>`). Mac App Store distribution requires `3rd Party Mac Developer Application` (app signing) and `3rd Party Mac Developer Installer` (pkg signing) — using a Developer ID cert on an App Store submission is rejected by App Store Connect at ingestion.

7. This task **MUST NOT** attempt to acquire, request, or enter Apple Developer Program enrollment, App Store Connect API keys, or any signing certificate — those are Stephen's account-creation and credential-entry actions per standing operating constraints. The task's own §9 Open Questions records exactly which prerequisites block `MAS_RELEASE=true` from ever being flippable.

8. A `docs/deploy/mac-app-store-submission.md` answer sheet **MUST** be authored capturing every App Store Connect macOS submission field that requires a human decision (export compliance / encryption declaration, content rights, macOS-specific privacy usage strings such as `NSCameraUsageDescription` if CyberOS ever requests camera access, age rating questionnaire) — mirroring the existing `docs/deploy/play-store-submission.md` pattern already in the repo for Android.

## §2 — Why this design

**Why a separate `tauri.mas.conf.json` overlay instead of conditionally branching the existing config (§1 #1)?** Tauri's config merge model (`--config <path>` flag deep-merges a second JSON file over the base) is the tool's own supported mechanism for exactly this "same source, two distribution channels" case — Tauri's own documentation examples use this pattern for Developer-ID-vs-App-Store macOS builds. Branching logic inside a single `tauri.conf.json` would require templating (Tauri config is static JSON, no conditionals), which is more fragile and harder to diff in code review than two files.

**Why full sandbox-compatibility audit before writing any entitlement (§1 #2)?** App Sandbox entitlements are an allowlist, not a denylist — every capability CyberOS's Rust backend uses that isn't automatically inside the sandbox container (temp dir, app's own Application Support dir) needs an explicit, minimal entitlement. Guessing at entitlements and iterating via App Store rejection is slow (each Apple review cycle is 24–48h); enumerating actual IPC/filesystem/network usage up front against the source is faster and produces a minimal, review-friendly entitlement set (Apple review scrutinizes entitlement scope-creep, e.g. requesting `com.apple.security.files.all` gets challenged).

**Why gate on a NEW repo variable `MAS_RELEASE` rather than reusing `MOBILE_RELEASE` (§1 #5)?** `MOBILE_RELEASE` governs the Capacitor iOS/Android jobs, which are architecturally unrelated to the Tauri desktop build this task extends. Coupling them would mean flipping mobile release on/off also toggles the (unrelated, prerequisite-gated) Mac App Store job, which is exactly the kind of accidental coupling TASK-IMP-071's "durable release trigger" work (see `related_tasks` — not formally linked here since it predates this task and isn't itself gating this work) was designed to avoid.

**Why does this task explicitly refuse to acquire Apple Developer Program access (§1 #7)?** This mirrors the standing constraint that account creation and credential entry are not actions an agent performs on the user's behalf. Recording this explicitly in the task (rather than silently omitting it) means a future implementer reading this spec in isolation — without the chat history that produced it — still knows exactly what's blocked and why, rather than discovering the blocker mid-implementation.

## §3 — API contract

`apps/desktop/src-tauri/tauri.mas.conf.json` (overlay, deep-merged over `tauri.conf.json` at build time via `tauri build --config tauri.mas.conf.json`):

```json
{
  "bundle": {
    "macOS": {
      "entitlements": "Entitlements.mas.plist",
      "signingIdentity": "3rd Party Mac Developer Application",
      "providerShortName": null,
      "hardenedRuntime": false
    }
  }
}
```

`apps/desktop/src-tauri/Entitlements.mas.plist` (initial minimal set — expanded only per §6 audit findings, never speculatively):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.app-sandbox</key>
    <true/>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
</dict>
</plist>
```

`apps/desktop/src-tauri/Entitlements.mas.inherit.plist` (attached to sidecar/child processes only — Tauri config field `bundle.macOS.entitlements` covers the main app binary; child-process entitlements are applied via a post-build `codesign --entitlements Entitlements.mas.inherit.plist` pass wired into the CI packaging step, since Tauri v2 does not expose a first-class config field for child-process entitlements as of this writing):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.app-sandbox</key>
    <true/>
    <key>com.apple.security.inherit</key>
    <true/>
</dict>
</plist>
```

`.github/workflows/release-mas.yml` (new, gated job — skeleton showing the CI contract, not the full workflow file):

```yaml
name: release-mas
on:
  workflow_dispatch:
jobs:
  assert-mas-gate-inert:
    # Runs UNCONDITIONALLY (no `if:`), so AC #6 has something to assert against —
    # this job's own success is meaningless in isolation; its purpose is to give
    # `gh run view` a completed workflow run in which build-and-submit-mas's
    # conclusion (skipped vs ran) can be inspected regardless of MAS_RELEASE state.
    runs-on: ubuntu-latest
    steps:
      - run: echo "workflow executed; see build-and-submit-mas conclusion for gate state"

  build-and-submit-mas:
    if: vars.MAS_RELEASE == 'true'
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - name: Import MAS signing certificates into CI keychain
        env:
          MAS_APP_CERT_P12_BASE64: ${{ secrets.MAS_APP_CERT_P12_BASE64 }}
          MAS_INSTALLER_CERT_P12_BASE64: ${{ secrets.MAS_INSTALLER_CERT_P12_BASE64 }}
          MAS_CERT_PASSWORD: ${{ secrets.MAS_CERT_PASSWORD }}
          MAS_KEYCHAIN_PASSWORD: ${{ secrets.MAS_KEYCHAIN_PASSWORD }}
        run: |
          security create-keychain -p "$MAS_KEYCHAIN_PASSWORD" mas-build.keychain
          security set-keychain-settings -lut 21600 mas-build.keychain
          security unlock-keychain -p "$MAS_KEYCHAIN_PASSWORD" mas-build.keychain
          for cert_var in MAS_APP_CERT_P12_BASE64 MAS_INSTALLER_CERT_P12_BASE64; do
            echo "${!cert_var}" | base64 --decode > /tmp/"$cert_var".p12
            security import /tmp/"$cert_var".p12 -k mas-build.keychain -P "$MAS_CERT_PASSWORD" \
              -T /usr/bin/codesign -T /usr/bin/productbuild
            rm /tmp/"$cert_var".p12
          done
          security list-keychains -d user -s mas-build.keychain login.keychain
          security set-key-partition-list -S apple-tool:,apple: -s -k "$MAS_KEYCHAIN_PASSWORD" mas-build.keychain
      - name: Build MAS bundle
        run: |
          cd apps/desktop
          npx tauri build --config src-tauri/tauri.mas.conf.json --bundles app
      - name: Sign child-process entitlements
        env:
          MAS_APP_SIGNING_IDENTITY: ${{ secrets.MAS_APP_SIGNING_IDENTITY }}   # e.g. "3rd Party Mac Developer Application: CyberSkill Software Solutions Consultancy and Development Joint Stock Company (TEAMID)"
        run: |
          codesign --force --deep --entitlements src-tauri/Entitlements.mas.inherit.plist \
            --sign "$MAS_APP_SIGNING_IDENTITY" \
            src-tauri/target/release/bundle/macos/CyberOS.app/Contents/MacOS/*
      - name: Build pkg installer
        env:
          MAS_INSTALLER_SIGNING_IDENTITY: ${{ secrets.MAS_INSTALLER_SIGNING_IDENTITY }}   # e.g. "3rd Party Mac Developer Installer: <same org, different cert type>"
        run: |
          productbuild --component src-tauri/target/release/bundle/macos/CyberOS.app /Applications \
            --sign "$MAS_INSTALLER_SIGNING_IDENTITY" \
            CyberOS.pkg
      - name: Upload to App Store Connect
        env:
          APP_STORE_CONNECT_KEY: ${{ secrets.ASC_API_KEY }}
          APP_STORE_CONNECT_KEY_ID: ${{ secrets.ASC_KEY_ID }}
          APP_STORE_CONNECT_ISSUER_ID: ${{ secrets.ASC_ISSUER_ID }}
        run: |
          xcrun altool --upload-app -f CyberOS.pkg -t macos \
            --apiKey "$APP_STORE_CONNECT_KEY_ID" --apiIssuer "$APP_STORE_CONNECT_ISSUER_ID"
      - name: Clean up keychain
        if: always()
        run: security delete-keychain mas-build.keychain || true
```

Two distinct certificate types are deliberately kept as two distinct secrets (`MAS_APP_SIGNING_IDENTITY` for `codesign`, `MAS_INSTALLER_SIGNING_IDENTITY` for `productbuild`) rather than one team-name secret interpolated into two identity strings — see §11 for why collapsing these into a single secret is the most common first-time mistake in Mac App Store CI pipelines.

## §4 — Acceptance criteria

1. **Sandbox audit is exhaustive** — every `#[tauri::command]` handler and every direct filesystem/network/process call in `apps/desktop/src-tauri/src/` is listed in a table (new file `docs/deploy/mac-app-store-submission.md` §"Sandbox surface audit") with a disposition of `sandbox-native`, `entitlement:<name>`, or `gated-behind-MAS-feature-flag`.
2. **Entitlements are minimal** — `Entitlements.mas.plist` contains no entitlement without a corresponding row in the sandbox surface audit table justifying it; a CI lint (new script, `tools/mas-entitlement-lint.sh`) fails the build if an entitlement key exists in the plist with no matching audit-table row.
3. **Two build targets stay independent** — running `tauri build` (no `--config` flag, the existing Developer ID path) produces byte-for-byte the same `bundle.macOS` output as before this task merged, verified by a CI diff step comparing `tauri.conf.json`'s resolved `bundle.macOS` block pre/post this task.
4. **Pkg is correctly signed** — `pkgutil --check-signature CyberOS.pkg` reports a valid "3rd Party Mac Developer Installer" signature chain in the CI job's log, when `MAS_RELEASE=true` and real certificates are present.
5. **Sandbox validity check passes locally** — `codesign --verify --deep --strict CyberOS.app` and `spctl -a -t install --context context:primary-signature CyberOS.app` both exit 0 against a locally-built MAS bundle before any CI wiring is trusted.
6. **CI job is inert by default** — with `MAS_RELEASE` unset or `false` (the repo's current state), `release-mas.yml`'s `build-and-submit-mas` job is skipped entirely (verified via a CI run showing the job as `skipped`, not `failed`).
7. **Answer sheet is complete** — `docs/deploy/mac-app-store-submission.md` has a filled-in row for every App Store Connect macOS submission field enumerated in Apple's macOS app submission checklist (export compliance, content rights, age rating, macOS privacy usage strings actually requested by CyberOS's entitlements from AC #1's audit table), each marked `human-confirmed` (Stephen) or `not-applicable` with a one-sentence reason.
8. **No credential material committed** — a CI secret-scan step (reusing the existing repo secret-scanning gate referenced in `docs/deploy/web-and-desktop-deploy.md`) passes against every file this task adds.

## §5 — Verification

```bash
# AC #3 — Developer ID build target unaffected.
# Uses a disposable worktree checked out at the pre-task commit rather than
# `git stash`, so the comparison can never lose or clobber uncommitted work
# in the primary working tree.
cd /sessions/ecstatic-sharp-lovelace/mnt/cyberos
PRE_TASK_SHA=$(git merge-base HEAD "$(git log --diff-filter=A -- \
  apps/desktop/src-tauri/tauri.mas.conf.json | tail -1)^")
git worktree add /tmp/pre-task-worktree "$PRE_TASK_SHA"
( cd /tmp/pre-task-worktree/apps/desktop/src-tauri && \
  npx tauri build --bundles app --ci --config /dev/null > /tmp/pre-task-bundle.json )
( cd apps/desktop/src-tauri && \
  npx tauri build --bundles app --ci --config /dev/null > /tmp/post-task-bundle.json )
diff /tmp/pre-task-bundle.json /tmp/post-task-bundle.json  # MUST be empty
git worktree remove /tmp/pre-task-worktree

# AC #5 — local sandbox validity (macOS runner only)
codesign --verify --deep --strict target/release/bundle/macos/CyberOS.app
echo "exit: $?"  # MUST be 0
spctl -a -t install --context context:primary-signature target/release/bundle/macos/CyberOS.app
echo "exit: $?"  # MUST be 0 once real MAS certs are present; documented as expected-fail with reason otherwise

# AC #2 — entitlement lint
tools/mas-entitlement-lint.sh apps/desktop/src-tauri/Entitlements.mas.plist \
  docs/deploy/mac-app-store-submission.md
# exits 1 and prints the unjustified entitlement key if audit-table coverage is incomplete
```

```yaml
# AC #6 — CI job inert-by-default, as a workflow assertion (excerpt from a test workflow run)
- name: Assert MAS job skipped when MAS_RELEASE unset
  run: |
    gh run view ${{ github.run_id }} --json jobs -q '.jobs[] | select(.name=="build-and-submit-mas") | .conclusion' | grep -q skipped
```

## §6 — Implementation skeleton

(API contract above is the skeleton — the two config files, the CI job YAML, and the entitlement-lint script are the entirety of the net-new surface. No new Rust code is required by this task; the sandbox audit in AC #1 may surface follow-up tasks if a specific IPC command turns out to need refactoring for sandbox compatibility, but that refactor is explicitly out of scope here — see §9.)

## §7 — Dependencies

- **Upstream:** none — this task reads the existing `apps/desktop/src-tauri/tauri.conf.json` and `.github/workflows/release.yml` but does not require any other task to land first.
- **Downstream:** none currently — a future task to fix any sandbox-incompatible IPC command found by AC #1's audit would depend on this task's audit table existing, but no such task is drafted yet (§9).
- **Cross-module:** none.
- **Human/account prerequisites (not task dependencies, but hard blockers on `MAS_RELEASE=true` ever being flippable):** Apple Developer Program enrollment (Stephen), macOS platform added to the CyberOS App Store Connect app record (Stephen), "3rd Party Mac Developer Application" + "3rd Party Mac Developer Installer" certificates issued and stored as CI secrets (Stephen).

## §8 — Example payloads

Sandbox surface audit table (excerpt, illustrative — the real table is populated during implementation from the actual source tree, not fabricated here per anti-fabrication discipline):

```markdown
| Symbol | File:line | Capability used | Disposition |
|---|---|---|---|
| `<pending — populated during implementation from actual src/ audit>` | | | |
```

`mas-entitlement-lint.sh` failure output shape:

```
ERROR: Entitlements.mas.plist declares "com.apple.security.files.downloads.read-write"
  but no row in docs/deploy/mac-app-store-submission.md §"Sandbox surface audit"
  justifies it. Add a row or remove the entitlement.
exit 1
```

## §9 — Open questions

Deferred:
- **Apple Developer Program enrollment status** — deferred to Stephen; blocks `MAS_RELEASE=true` entirely. `human-confirmed` marker required before this flag can ever be set.
- **Whether CyberOS's Rust backend spawns any child processes today** (relevant to whether `Entitlements.mas.inherit.plist` is load-bearing or defensive-only) — deferred to the AC #1 sandbox audit, to be resolved during implementation by reading `apps/desktop/src-tauri/src/` rather than guessed here.
- **Whether any sandbox-incompatible capability is found by the AC #1 audit that has no entitlement equivalent** (e.g. arbitrary-path filesystem access outside the sandbox container, if CyberOS's desktop app does anything like reading files a user hasn't explicitly selected via an Open panel) — if found, this becomes a follow-up task to gate that capability behind a "MAS build" feature flag; not resolved here since it depends on the audit's outcome.
- **App Store Connect macOS app record bundle ID** — whether it reuses `os.cyberskill.world.desktop` (the existing Tauri identifier) or requires a distinct one is an App Store Connect account-setup decision for Stephen, not an engineering decision this spec can make.
- **`related_tasks` forward-references** — this task lists TASK-APP-004/005/006 in `related_tasks` even though they are drafted in the same authoring batch immediately after this one (all five approved together in a single PLAN). This is a deliberate same-batch forward reference, not a placeholder-task situation under §3.1 rule #3 (that rule scopes to `depends_on:`/`blocks:`, both empty here) — resolved once the batch completes and all referenced tasks exist on disk.

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Sandbox audit misses an IPC command that touches an un-entitled path | `spctl -a -t install` fails at local verification (AC #5) with a sandbox violation in `log show --predicate 'subsystem == "com.apple.sandbox"'` | Local build verification catches it before any CI/App Store submission | Add the missing entitlement or gate the command behind a MAS feature flag; re-run AC #1 audit |
| Entitlement lint passes locally but a new IPC command is added later without updating the audit table | CI entitlement-lint step (wired into the same PR gate as other repo lints) fails on the PR that added the new command, if the PR also touches `Entitlements.mas.plist` — otherwise silently drifts | Entitlement stays broader than actual usage (over-permissioned, App Store review risk) until caught | Periodic re-audit; add entitlement-lint as a required check on any PR touching `src-tauri/src/` per §11 |
| Developer ID and MAS builds silently diverge because someone edits `tauri.conf.json` assuming the MAS overlay auto-inherits changes | AC #3's diff check fails in CI | MAS build could ship stale config (e.g. old `productName`) | CI gate blocks merge; developer manually reconciles `tauri.mas.conf.json` |
| `codesign --deep` re-signs child-process binaries with the wrong entitlements because Tauri's build output layout changes in a future Tauri version | AC #5 local verification fails after a `tauri` CLI upgrade | MAS build silently ships without sandbox inheritance on subprocesses | Pin Tauri CLI version in CI; re-verify `Entitlements.mas.inherit.plist` application path after any Tauri upgrade |
| Apple rejects the submission for entitlement over-scoping despite the lint passing (lint only checks "is it justified in our own docs", not "does Apple agree it's minimal") | App Store Connect review rejection email (external signal, not a repo-internal detection) | Submission delayed 24–48h per review cycle | Narrow the entitlement per Apple's specific rejection reason; this is expected iteration, not a design flaw |
| `MAS_RELEASE=true` is flipped before certificates exist in CI secrets | CI job fails at the `codesign` step with "no identity found" | Job fails loudly, no partial/corrupt artifact produced | AC #6 explicitly tests the `false`/unset state; the `true`-but-no-certs state is an operator error, not a spec gap — job failure is the correct behavior |
| The `.pkg` installer's component plist references the wrong install location (not `/Applications`) | `productbuild` step produces a pkg that fails `pkgutil --check-signature` or installs to an unexpected path in manual QA | User-facing install failure or wrong install location if ever manually distributed outside App Store Connect | Component plist is generated by `productbuild --component <app> /Applications` per §3 contract — path is hardcoded correctly in the CI step, verified by AC #4 |
| Two build targets' `productName`/`version` drift because `tauri.mas.conf.json` doesn't override `version` and the base `tauri.conf.json`'s version bump (via the repo's stamper, TASK-IMP-072) isn't picked up by the overlay merge | Version mismatch visible in App Store Connect build processing vs. the GitHub Release tag | Confusing version numbers across distribution channels for end users comparing "which build is newer" | `tauri.mas.conf.json` MUST NOT declare its own `version` field — Tauri's config merge takes the base file's `version` when the overlay omits it, so this is a "don't add the field" discipline, verified by AC #3's diff check implicitly (adding a version field to the overlay would show up as a diff) |
| Secret scan (AC #8) has a false negative because App Store Connect API key JSON is base64-encoded inline in a workflow file instead of referenced via `secrets.*` | Manual code review catches it, or it ships to a public repo | Credential leak | This task's `disallowed_tools` explicitly forbids entering credential material anywhere non-secret-manager; the `release-mas.yml` skeleton in §3 uses `${{ secrets.* }}` exclusively as the only sanctioned pattern |
| `MOBILE_RELEASE` and `MAS_RELEASE` get accidentally coupled in a future refactor of `release.yml` that "simplifies" the release gating | Any future PR touching release gating that doesn't preserve independent boolean gates would need to pass AC #6's inert-by-default test for `MAS_RELEASE` specifically | Mac App Store job runs unexpectedly when mobile release is flipped on, before certs exist | AC #6's CI assertion is a standing regression test, not a one-time check — it runs on every workflow change |
| The ephemeral `mas-build.keychain` created per-CI-run isn't deleted if the job fails before the cleanup step, leaving orphaned keychains on a (hypothetically) reused self-hosted runner | Not detectable on GitHub-hosted `macos-14` runners (fresh VM per run, so this is moot there); would surface as keychain-list pollution if ever migrated to a self-hosted runner | No functional impact on GitHub-hosted runners; latent risk only if the runner strategy changes | `if: always()` on the cleanup step (§3) covers the common failure case; a self-hosted-runner migration would need an additional pre-job keychain-sweep step, out of scope while runners stay GitHub-hosted |
| `security import` succeeds but `security set-key-partition-list` fails silently on a keychain ACL edge case (documented macOS `security` CLI quirk on certain Xcode/macOS version combinations) | `codesign` step fails immediately after with a keychain-access prompt hang or `errSecInternalComponent`, since CI has no interactive session to approve the ACL prompt | Build fails at the signing step, no partial artifact produced | Pin the `macos-14` runner image version in `release-mas.yml`; this is a known class of CI flake independent of this task's design, mitigated by image pinning rather than a code fix |

## §11 — Implementation notes

- **`providerShortName` casing:** Tauri v2's `bundle.macOS` config key is `providerShortName` (camelCase) even though this task's §1 #1 prose used a space for readability — the §3 JSON contract uses the correct camelCase key. This is a real Tauri config field, used when the Apple Developer Team has multiple providers associated (agencies signing on behalf of multiple clients); CyberSkill likely doesn't need it (`null` is valid and means "use the default provider"), but the field is included in the contract for completeness since getting it wrong silently produces the wrong provider in App Store Connect for teams that do need it.
- **Why `hardenedRuntime: false` in the MAS overlay:** the Hardened Runtime entitlement is for Developer-ID-distributed, notarized apps outside the App Store — Mac App Store apps use App Sandbox instead, and enabling both simultaneously is redundant/conflicting per Apple's own guidance. The Developer ID build (unaffected by this task) presumably already has `hardenedRuntime: true` for notarization; this overlay explicitly turns it off for the MAS target rather than leaving it ambiguous.
- **Two-cert signing (app cert + separate installer cert) is not optional** — this is the single most common first-time mistake in Mac App Store CI pipelines (using one cert for both `codesign` and `productbuild`). The §3 CI skeleton uses two distinct `secrets.MAS_TEAM_NAME`-scoped identity strings deliberately to make this explicit in the contract, even though both resolve to the same team name string in practice — the identity *type prefix* (`3rd Party Mac Developer Application` vs `3rd Party Mac Developer Installer`) is what differs and what a copy-paste error would most likely get wrong.
- **This task deliberately does not attempt to resolve the sandbox-audit findings** — AC #1 requires the audit table to exist and be exhaustive, but any IPC command found to be sandbox-incompatible is explicitly out of scope for *this* task to fix (§9). This keeps the task's own scope bounded to "can we build and submit a sandboxed bundle at all," with any deeper Rust refactoring work spun into its own future task once the audit reveals whether such work is even needed — writing that follow-up task speculatively now would violate the anti-fabrication discipline (inventing scope for work whose necessity isn't yet known).

*End of TASK-APP-003.*
