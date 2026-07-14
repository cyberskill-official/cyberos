---
id: TASK-APP-005
title: "Linux store distribution — Snap Store (snapcraft, strict confinement) and Flathub (flatpak-builder, external-manifest submission)"
module: app
priority: SHOULD
status: done
verify: T
phase: P1
milestone: P1 · slice 1
slice: 3
owner: Stephen Cheng
created: 2026-07-12
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-APP-001, TASK-APP-003, TASK-APP-004, TASK-APP-006]
depends_on: []
blocks: []
source_pages:
  - apps/desktop/src-tauri/tauri.conf.json
  - apps/desktop/src-tauri/icons/ (32x32.png, 128x128.png, 128x128@2x.png — confirmed present via tauri.conf.json bundle.icon array)
  - .github/workflows/release.yml (lines 60–139: existing `desktop` job, ubuntu-latest matrix leg, Linux build deps, tauri-action signing-gate idiom)
source_decisions: []
language: bash / YAML (snapcraft.yaml, Flatpak manifest), Rust 1.81 (Tauri v2, unchanged)
service: apps/desktop/src-tauri
new_files:
  - apps/desktop/src-tauri/snap/snapcraft.yaml
  - .github/workflows/release-snap.yml
  - flathub-manifest/os.cyberskill.world.desktop.yml
  - docs/deploy/linux-store-submission.md
modified_files: []
allowed_tools:
  - Tauri CLI (`tauri build --bundles deb` — reuses the existing GitHub-Releases Linux build, no new bundle target)
  - Snapcraft CLI (`snapcraft pack`, `snapcraft upload`, `snapcraft release`) for the Snap Store track
  - "`flatpak-builder` for local Flathub manifest validation only — the actual Flathub build happens on Flathub's own CI, not CyberOS's"
allowed_tools_note: "This FR does not use `snapcraft login`/store credential provisioning itself; it wires the CI step to consume a pre-provisioned SNAPCRAFT_STORE_CREDENTIALS secret Stephen generates via his own `snapcraft export-login` run."
disallowed_tools:
  - Any tool that would create the Snapcraft/Ubuntu One account, register the snap name `cyberos` in the Snap Store namespace, or run `snapcraft export-login` to mint store credentials on Stephen's behalf
  - Any tool that would open a pull request against the external `flathub/flathub` GitHub repository without a fresh, explicit chat-turn confirmation from Stephen (per the standing "publishing/modifying public content" permission gate — this applies even though Stephen approved the overall FR-authoring PLAN, because opening an external PR is a distinct, later, irreversible action from drafting the manifest)
  - Any tool that would enter Snapcraft/Ubuntu One credentials into a non-secret-manager location
effort_hours: 22
subtasks:
  - "Author apps/desktop/src-tauri/snap/snapcraft.yaml: strict confinement, gnome extension for core22 (webkit2gtk-4.1 runtime libs), plugs for desktop/desktop-legacy/wayland/x11/network/opengl (4h)"
  - "Author .github/workflows/release-snap.yml: separate CI job gated on repo variable SNAP_RELEASE=true, ubuntu-latest, snapcraft pack + upload + release-to-channel (4h)"
  - "Author flathub-manifest/os.cyberskill.world.desktop.yml: flatpak-builder manifest referencing org.freedesktop.Platform runtime, finish-args sandbox permissions, .desktop entry + icon (5h)"
  - "Author the .desktop entry (os.cyberskill.world.desktop.desktop) and confirm/adapt icon assets to the Flatpak app-id naming convention (2h)"
  - "Validate the Flatpak manifest locally with flatpak-builder --repo=repo build-dir manifest.yml before ever proposing a Flathub PR (3h)"
  - "Write docs/deploy/linux-store-submission.md answer sheet (Snap Store metadata, Flathub review checklist, app-id reverse-DNS decision, confinement/sandbox justification for reviewers) (4h)"
risk_if_skipped: "CyberOS Linux users can already obtain .deb, .rpm, and .AppImage artifacts from GitHub Releases today (confirmed via the existing `desktop` job's ubuntu-latest matrix leg and 'Linux build deps' step in release.yml). This remains fully functional. Deferring costs Snap Store / Flathub discovery and one-command install (`snap install cyberos`, `flatpak install flathub os.cyberskill.world.desktop`) plus each platform's own auto-update channel — not core functionality, and unlike TASK-APP-003/004, neither channel requires a paid developer account, so the cost of deferring is purely discoverability, not blocked revenue."
---

## §1 — Description

1. CyberOS **MUST** gain two independent Linux app-store distribution paths — Snap Store and Flathub — layered on top of the existing `.deb`/`.rpm`/`.AppImage` GitHub-Releases artifacts (confirmed produced today by the `desktop` job's `ubuntu-latest` matrix leg in `.github/workflows/release.yml`, lines 69–139), not replacing them.

2. The two paths **MUST** be treated as architecturally distinct, not parallel instances of the same pattern used in TASK-APP-003/TASK-APP-004, because their submission mechanics genuinely differ: **Snap Store** accepts a CI-built, CI-uploaded `.snap` artifact via `snapcraft upload` (a CI-automatable, credential-gated upload, matching the TASK-APP-003/004 pattern); **Flathub** does **NOT** accept a pre-built artifact upload at all — Flathub's own build infrastructure clones a submitted manifest repository and builds the Flatpak itself, so CyberOS's role is limited to maintaining a correct, buildable manifest and (with Stephen's explicit per-instance approval) opening a pull request against the external `flathub/flathub` repository.

3. `apps/desktop/src-tauri/snap/snapcraft.yaml` **MUST** declare `confinement: strict` (not `devmode` or `classic`) using the `gnome` extension for `base: core22`, which provisions the WebKitGTK/GTK runtime libraries a Tauri app needs without CyberOS having to stage them manually — this mirrors the exact library set the existing `Linux build deps` CI step already installs for the GitHub-Releases build (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`), confirming `webkit2gtk-4.1` (not the older `4.0` API) is the correct runtime generation to target.

4. Snap's `plugs` **MUST** be limited to what CyberOS's actual functionality needs: `desktop`, `desktop-legacy`, `wayland`, `x11` (display/input), `network` (the app is a thin shell loading `os.cyberskill.world/web`, confirmed by the existing release workflow's own comment: "the desktop app is a thin shell that loads os.cyberskill.world/web, so there is no web build step here" — meaning outbound network access is a hard functional requirement, not optional), and `opengl` (WebKitGTK's hardware-accelerated compositing). No broader plugs (e.g. `home`, `removable-media`, `raw-usb`) **MUST** be requested without a documented functional justification in §9, since over-broad plug requests are a common cause of manual Snap Store review delay.

5. Flathub's manifest **MUST** use a reverse-DNS app ID distinct from (though may be textually similar to) the existing Tauri `identifier` field (`os.cyberskill.world.desktop`, confirmed in `tauri.conf.json`) — Flatpak app IDs have their own naming convention and validation rules (must match the `.desktop` file's basename, must be a syntactically valid reverse-DNS string under Flathub's ownership-verification model, which typically expects the domain segment to correspond to a domain CyberSkill controls) — this FR **MUST** decide and document the actual app ID as an explicit choice, not silently reuse the Tauri identifier string without verifying it satisfies Flathub's conventions.

6. CI **MUST** gate the Snap Store CI-automation job behind a repo variable `SNAP_RELEASE=true`, following the exact `vars.<FLAG> == 'true' && secrets.<X> || ''` gating idiom already established in `release.yml`'s `desktop` job for `MACOS_SIGN`/`DESKTOP_UPDATER_SIGN`, defaulting to off, independent of `MOBILE_RELEASE`/`MAS_RELEASE`/`MSSTORE_RELEASE`. Flathub submission has **no equivalent CI-gated build job** (per §1 #2) — the closest analogue is a documented, Stephen-approved-per-instance process for opening the Flathub PR, not a flippable CI flag.

7. This FR **MUST NOT** attempt to register the `cyberos` snap name in the Snap Store namespace, run `snapcraft export-login` to mint store credentials, or open the Flathub submission PR without Stephen's explicit, fresh chat-turn confirmation at the time of submission — those are Stephen's account-setup and publishing actions respectively.

8. A `docs/deploy/linux-store-submission.md` answer sheet **MUST** be authored covering the Snap Store submission metadata (summary, description, category) and, separately, Flathub's actual review checklist items (app-id conventions, sandbox `finish-args` justification, `.desktop`/AppStream metadata completeness), mirroring the `docs/deploy/play-store-submission.md` pattern but structured as two distinct sections given §1 #2's architectural split.

## §2 — Why this design

**Why is Flathub handled so differently from every other store FR in this batch (§1 #2)?** TASK-APP-003 (Mac App Store) and TASK-APP-004 (Microsoft Store) both follow the same shape: CyberOS's CI builds a signed package and pushes it to the store via that store's own submission API. Flathub does not offer an equivalent artifact-upload API for the initial or ongoing publishing flow — Flathub's documented, current process is: a developer submits a *manifest* (the recipe describing how to build the app from source, including which runtime/SDK and which upstream sources to fetch) as a pull request against `flathub/flathub`, and Flathub's own build infrastructure clones that manifest and builds the actual Flatpak on every update. Pretending this fits the same "CI job with a `FLATHUB_RELEASE=true` gate that uploads a built artifact" shape as the other three store FRs would misdescribe how Flathub actually works and would produce a CI job with no real function to perform. Treating it honestly as "maintain a correct manifest, propose a PR when ready" is the accurate design, even though it breaks the batch's otherwise-consistent CI-gate pattern.

**Why strict confinement rather than classic (§1 #3)?** `classic` confinement grants full system access with no sandbox and is reserved by the Snap Store for apps with a specific technical justification (typically dev tools needing arbitrary filesystem/process access) — approval for `classic` snaps requires a manual Snap Store review process that's slower and less certain than `strict`. CyberOS is a webview-based app loading a remote URL; it has no functional need for unsandboxed system access, so `strict` confinement with a minimal, justified plug list (§1 #4) is both the more secure choice and the faster review path.

**Why the `gnome` extension instead of manually bundling WebKitGTK (§1 #3)?** Manually staging every GTK/WebKitGTK shared library and its transitive dependencies inside a strictly-confined snap is exactly the kind of packaging complexity Canonical's platform `extensions` mechanism exists to eliminate — the `gnome` extension (for `core22`) provisions the GNOME/GTK/WebKitGTK runtime stack via a shared content snap, keeping `snapcraft.yaml` from needing to hand-declare CyberOS's own copies of libraries the existing GitHub-Releases Linux build already depends on (`libwebkit2gtk-4.1-dev` et al., confirmed in `release.yml`).

**Why does the Flatpak app ID need explicit decision-making rather than reusing the Tauri `identifier` (§1 #5)?** The two identifier systems serve different validation purposes — Tauri's `identifier` only needs to be a valid bundle identifier for OS-level app registration (no external ownership-verification step), while Flathub's app-id convention is tied to a domain-ownership model reviewers check during submission. Assuming they're interchangeable without verifying against Flathub's actual current documentation risks a rejected or delayed submission over a naming technicality that's cheap to get right up front and expensive to discover during review.

## §3 — API contract

`apps/desktop/src-tauri/snap/snapcraft.yaml` (structural skeleton):

```yaml
name: cyberos
base: core22
version: '1.0.0'
summary: CyberOS — Turn Your Will Into Real
description: |
  CyberOS is CyberSkill's desktop shell for the CyberOS platform. This snap
  packages the Tauri-based desktop client, a thin shell over os.cyberskill.world/web.
grade: stable
confinement: strict
# `architectures:` syntax below (per-entry build-on objects) is Snapcraft's documented form as
# of core22-era schemas; a simpler `architectures: [amd64, arm64]` list form also exists in
# some schema versions. WORKER phase MUST confirm the exact accepted form against Snapcraft's
# current schema for core22 before this file is treated as final — flagged here rather than
# asserted with false confidence, since both forms are plausible and this FR's authoring did
# not execute `snapcraft pack` against a real Snapcraft installation to confirm.
architectures:
  - build-on: amd64
  - build-on: arm64

apps:
  cyberos:
    command: usr/bin/cyberos
    extensions: [gnome]
    plugs:
      - desktop
      - desktop-legacy
      - wayland
      - x11
      - network
      - opengl

parts:
  cyberos:
    plugin: dump
    source: dist/          # populated by the CI staging step from the existing `tauri build --bundles deb` .deb payload — see §3's CI skeleton, not a separate Cargo build.
    organize:
      usr/bin/cyberos: usr/bin/cyberos
    stage-packages:
      - libwebkit2gtk-4.1-0
      - libayatana-appindicator3-1
```

`flathub-manifest/os.cyberskill.world.desktop.yml` (structural skeleton — filename and app-id are the §9-tracked open decision, shown here with the current best candidate):

```yaml
app-id: os.cyberskill.world.desktop
runtime: org.freedesktop.Platform
runtime-version: '23.08'
sdk: org.freedesktop.Sdk
command: cyberos

finish-args:
  - --share=network      # thin shell loading os.cyberskill.world/web — hard functional requirement
  - --socket=wayland
  - --socket=fallback-x11
  - --device=dri         # WebKitGTK hardware-accelerated compositing
  - --share=ipc

modules:
  - name: cyberos
    buildsystem: simple
    build-commands:
      # Flathub's own build infra runs this from source per §2 — placeholder build-commands
      # here are a structural skeleton only; the real commands depend on which upstream source
      # archive/tag Flathub is instructed to fetch, decided during WORKER-phase submission prep,
      # not fabricated here.
      - install -Dm755 cyberos /app/bin/cyberos
      - install -Dm644 os.cyberskill.world.desktop.desktop /app/share/applications/os.cyberskill.world.desktop.desktop
      - install -Dm644 icons/128x128.png /app/share/icons/hicolor/128x128/apps/os.cyberskill.world.desktop.png
    sources:
      - type: git
        url: https://github.com/cyberskill-official/cyberos.git
        tag: v1.0.0  # pinned per release; Flathub rebuilds only when this manifest is updated
```

`.github/workflows/release-snap.yml` (Snap Store CI skeleton — Flathub has no CI-upload equivalent, per §1 #2):

```yaml
name: release-snap
on:
  workflow_dispatch:
jobs:
  build-and-upload-snap:
    if: vars.SNAP_RELEASE == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { ref: "${{ github.event.inputs.tag || github.ref_name }}" }
      - name: Linux build deps
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
      - name: Build .deb payload (source for the snap's dump part)
        run: |
          cd apps/desktop
          npx tauri build --bundles deb
      - name: Stage snap dump source from the .deb payload
        run: |
          mkdir -p apps/desktop/src-tauri/snap/dist
          dpkg-deb -x apps/desktop/src-tauri/target/release/bundle/deb/*.deb apps/desktop/src-tauri/snap/dist
      - uses: snapcore/action-build@v1
        id: snapcraft
        with:
          path: apps/desktop/src-tauri
      - uses: snapcore/action-publish@v1
        env:
          SNAPCRAFT_STORE_CREDENTIALS: ${{ secrets.SNAPCRAFT_STORE_CREDENTIALS }}
        with:
          snap: ${{ steps.snapcraft.outputs.snap }}
          release: stable
```

## §4 — Acceptance criteria

1. **`snapcraft.yaml` is schema-valid and packs locally** — `snapcraft pack apps/desktop/src-tauri` exits 0 against a locally-staged `dist/` directory containing a stub `usr/bin/cyberos` binary, without requiring real Snap Store credentials.
2. **Strict confinement, no `classic`/`devmode`** — a CI lint (`grep -q "^confinement: strict$" apps/desktop/src-tauri/snap/snapcraft.yaml`) asserts the confinement level never silently regresses to a weaker mode.
3. **Plug list matches §1 #4 exactly, no silent additions** — a CI lint asserts the `plugs:` block in `snapcraft.yaml` contains exactly the six-entry set from §1 #4 (`desktop`, `desktop-legacy`, `wayland`, `x11`, `network`, `opengl`) and no others, via a fixed-string comparison, not a fuzzy diff. Automated cross-checking that any *future* plug addition is accompanied by a §9 justification entry is a code-review norm (§11), not a CI-enforceable mechanism this FR builds — a script that parses prose §9 entries and correlates them to YAML list changes would be real but disproportionate engineering for this FR's scope, and asserting it exists without designing it would be exactly the kind of unspecified-but-claimed logic this authoring discipline forbids.
4. **Flatpak manifest validates structurally** — a `flatpak-builder` invocation against `flathub-manifest/os.cyberskill.world.desktop.yml` completes manifest-parsing without a YAML/schema error (full build success is not required for this AC, since Flathub's own infra performs the authoritative build per §2 — this AC only guards against manifest syntax/schema errors CyberOS can catch before ever proposing a PR). The exact `flatpak-builder` flag combination for a parse-only/dry-run check (§5) **MUST** be confirmed against the installed `flatpak-builder` version's actual `--help` output at implementation time rather than assumed from this spec — flatpak-builder's CLI surface was not exercised during this FR's authoring.
5. **CI job is inert by default** — with `SNAP_RELEASE` unset or `false`, `release-snap.yml`'s `build-and-upload-snap` job is skipped (verified the same way as TASK-APP-003/004 — an unconditional anchor job plus a `gh run view` conclusion check).
6. **No Flathub PR is opened by CI or by this FR's own tooling under any circumstance** — there is no `FLATHUB_RELEASE` flag, no workflow step, and no script in this FR's `new_files` that calls `gh pr create` against `flathub/flathub`; this is a structural, not merely a documented, guarantee — verified by `grep -r "flathub/flathub" .github/ tools/` returning zero matches (the answer sheet at `docs/deploy/linux-store-submission.md` lives under `docs/`, outside the two directories this check scans, so no exclusion pattern is needed).
7. **Answer sheet is complete** — `docs/deploy/linux-store-submission.md` has a filled-in row for every Snap Store metadata field and every Flathub review-checklist item, each marked `human-confirmed` or `not-applicable` with a reason.
8. **No credential material committed** — the repo's existing secret-scan gate passes against every file this FR adds.

## §5 — Verification

```bash
# AC #1 — snapcraft.yaml packs locally against a stub payload, no real credentials needed
mkdir -p /tmp/snap-verify/dist/usr/bin
echo '#!/bin/sh
exit 0' > /tmp/snap-verify/dist/usr/bin/cyberos
chmod +x /tmp/snap-verify/dist/usr/bin/cyberos
cp apps/desktop/src-tauri/snap/snapcraft.yaml /tmp/snap-verify/snapcraft.yaml
( cd /tmp/snap-verify && snapcraft pack --destructive-mode )
echo "exit: $?"  # MUST be 0

# AC #2 — confinement lint
grep -q "^confinement: strict$" apps/desktop/src-tauri/snap/snapcraft.yaml
echo "exit: $?"  # MUST be 0

# AC #4 — Flatpak manifest structural validation. Exact flag TBD against the installed
# flatpak-builder version's --help output (§4 AC #4) — a full, non-destructive local build
# into a scratch repo is the fallback if no dedicated parse-only flag exists:
flatpak-builder --force-clean --repo=/tmp/flathub-test-repo /tmp/flathub-test-build \
  flathub-manifest/os.cyberskill.world.desktop.yml
echo "exit: $?"  # MUST be 0 (or the confirmed parse-only flag's equivalent, per AC #4)

# AC #6 — no Flathub PR automation exists anywhere in the repo's tooling. Scoped to .github/
# and tools/ only — docs/deploy/linux-store-submission.md legitimately mentions
# "flathub/flathub" in prose and lives outside both scanned directories, so no exclusion
# pattern is needed (a prior draft of this check carried a now-removed, dead `grep -v`
# exclusion for a path that was never in scope to begin with).
MATCHES=$(grep -rl "flathub/flathub" .github/ tools/ 2>/dev/null | wc -l)
echo "match count: $MATCHES"  # MUST be 0
```

```yaml
# AC #5 — CI job inert-by-default, mirroring TASK-APP-003/004's pattern
- name: Assert Snap job skipped when SNAP_RELEASE unset
  run: |
    gh run view ${{ github.run_id }} --json jobs -q \
      '.jobs[] | select(.name=="build-and-upload-snap") | .conclusion' | grep -q skipped
```

## §6 — Implementation skeleton

The API contract in §3 covers `snapcraft.yaml`, the Flatpak manifest, and the Snap CI workflow. Two pieces of logic are intentionally deferred to WORKER phase rather than fully specified here, each for a distinct, stated reason (not scope-avoidance):

- **The `.desktop` entry file's exact content** (`os.cyberskill.world.desktop.desktop`, referenced by both the snap and the Flatpak manifest) — a small, mechanical XDG desktop-entry file whose exact `Exec=`/`Icon=`/`Categories=` values depend on the final app-id decision (§9), so it is authored once that decision lands rather than duplicated here with a value that might need to change.
- **The exact `tag:` / source-archive pinning strategy in the Flatpak manifest's `sources:` block** — Flathub's manifest-review process has specific, current expectations about source provenance (git tag vs. release tarball with checksum) that are best confirmed against Flathub's live submission documentation at proposal time rather than guessed here.

## §7 — Dependencies

- **Upstream:** none. Reuses the existing `desktop` job's `ubuntu-latest` matrix leg build deps (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`) as the grounding for `snapcraft.yaml`'s `stage-packages`/extension choice, but does not modify that job.
- **Downstream:** none currently drafted.
- **Cross-module:** none.
- **Human/account prerequisites:** Snapcraft/Ubuntu One account + `cyberos` snap name registration (Stephen — free, no payment), `snapcraft export-login` to mint `SNAPCRAFT_STORE_CREDENTIALS` (Stephen). **Hard blocker on ever opening the Flathub PR:** the app-id/domain-ownership decision (§9) must resolve first — an unresolved app-id means the manifest's `app-id:` field, filename, and `.desktop` entry (§6) are all still provisional, and submitting a PR with a provisional app-id risks exactly the rejected-on-a-naming-technicality outcome §2 warns against. Layered on top of that: Stephen's explicit per-instance approval is required before this FR's tooling (or a human directly) opens the Flathub submission PR regardless of app-id status (§1 #7, §4 AC #6 — structurally guaranteed, not just policy).

## §8 — Example payloads

`snapcraft pack` local verification failure output shape (illustrative — a plausible, not fabricated-as-real, snapcraft error for a missing stage-package):

```
Failed to install snap 'gnome' extension content package.
Ensure the snapcraft.yaml declares a supported `base` for the requested extension.
```

Flathub manifest structural-validation failure shape (`flatpak-builder --show-manifest` on a malformed manifest):

```
Failed to parse arguments: builddir must be an existing directory
```

## §9 — Open questions

Deferred:
- **Flathub app-id: exact reverse-DNS string** — `os.cyberskill.world.desktop` is used as the working candidate throughout §3, matching the existing Tauri `identifier`, but this **MUST** be verified against Flathub's current domain-ownership submission requirements before the manifest is finalized — deferred to Stephen (may require confirming `cyberskill.world` DNS ownership through whatever mechanism Flathub currently documents).
- **Snap Store channel strategy** — §3's CI skeleton releases directly to the `stable` channel on every `SNAP_RELEASE=true` run; Stephen may prefer routing through `candidate`/`beta` first with manual promotion via `snapcraft release`, mirroring a staged-rollout posture. Deferred; the skeleton's `stable`-direct choice is a starting default, not a locked-in decision.
- **Any future need for a broader Snap plug** (e.g. `home` for local file access, if CyberOS ever gains local-file features) — per §1 #4, any such addition requires a documented functional justification added here at the time it's needed; none is required today because the app is a thin remote-URL shell.
- **`related_tasks` references TASK-APP-006, which does not yet exist on disk** — deliberate same-batch forward reference (this FR and TASK-APP-006 were approved together in the same PLAN and are being authored sequentially in one session; TASK-APP-003 and TASK-APP-004 are already landed). `depends_on`/`blocks` are both empty, so no inline placeholder annotation is mechanically required; documented here for the same disclosure reasons TASK-APP-003 §9 and TASK-APP-004 §9 recorded their own forward references.

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| `gnome` extension version mismatch with `base: core22` (Snapcraft extensions are versioned per base) | `snapcraft pack` fails at extension-resolution time with an explicit error naming the unsupported combination | Local pack verification (AC #1) fails before any CI run is attempted | Confirm the `gnome` extension's currently-supported base list against Snapcraft's live documentation at implementation time; core22 was the most recent stable base as of this FR's authoring but Snapcraft's supported-base list changes over time |
| Strict-confinement snap fails to launch WebKitGTK due to a missing plug not anticipated in §1 #4's minimal list (e.g. a missing `gsettings` or `password-manager-service` plug some GTK apps implicitly need) | Manual smoke-test of the packed `.snap` installed via `snap install --dangerous` fails to render the webview or crashes on launch | Snap builds successfully but doesn't run correctly for end users if shipped as-is | Documented as a required manual smoke-test step in `docs/deploy/linux-store-submission.md` before any `SNAP_RELEASE=true` promotion to `stable`; if a missing plug is discovered, it's added to §1 #4's list with the justification §3's AC #3 lint requires |
| `dpkg-deb -x` staging step (§3's CI skeleton) extracts a `.deb` payload whose internal path layout doesn't match the `organize:` mapping in `snapcraft.yaml` (e.g. Tauri's `.deb` output nests the binary under a different path than assumed) | `snapcraft pack` succeeds but the resulting snap's `apps.cyberos.command` points at a nonexistent file, caught by `snap install --dangerous` smoke-testing, not by `pack` itself | Snap builds "successfully" but is non-functional | This is exactly why AC #1's verification uses a stub binary at the assumed path rather than trusting the real `.deb` layout sight-unseen — the WORKER phase **MUST** confirm the actual `.deb` internal path against a real `tauri build --bundles deb` output before finalizing `organize:`, not assume the skeleton's guessed path is correct |
| Flathub reviewers reject the `--device=dri` or `--share=network` `finish-args` as insufficiently justified during manual review | Flathub PR review comments (external, human, asynchronous — not a CI-detectable failure) | Submission delayed pending manifest revision and re-review | Documented in the answer sheet (AC #7) with the functional justification each permission maps to (`--share=network` → thin remote-shell architecture, confirmed via `release.yml`'s own comment; `--device=dri` → WebKitGTK hardware compositing) so the justification is ready at submission time, not improvised during review |
| CyberOS's `identifier` (`os.cyberskill.world.desktop`) turns out not to satisfy Flathub's domain-ownership verification convention once actually checked against current documentation | Discovered during §9's deferred verification step, ideally before PR submission | App-id rename required, touching the manifest, the `.desktop` file, and icon install paths together | Because §1 #5 explicitly scoped this as a decision requiring verification rather than assuming reuse was safe, a rename at this stage touches only files this FR itself introduces — no coupling to already-shipped identifiers elsewhere in the repo (Tauri's own `identifier` field is untouched by this FR) |
| `SNAPCRAFT_STORE_CREDENTIALS` macaroon expires (Snapcraft export-login credentials have a bounded validity period) | `snapcore/action-publish@v1` step fails with an authentication error | CI job fails at the upload step; no partial/corrupt Snap Store revision created | Stephen re-runs `snapcraft export-login` and rotates the `SNAPCRAFT_STORE_CREDENTIALS` secret; expected periodic operational task, documented in the answer sheet |
| A future contributor adds a `FLATHUB_RELEASE=true`-style CI gate under the mistaken assumption Flathub works like the other three stores | Code review, or AC #6's structural grep-based guard, which would start failing the moment such a step appears | Would silently misrepresent how Flathub actually works if merged unnoticed | AC #6 makes this a standing, automatically-enforced regression test rather than relying on institutional memory of §2's architectural explanation |
| Snap's `stage-packages` list (`libwebkit2gtk-4.1-0`, `libayatana-appindicator3-1`) drifts out of sync with the actual `apt-get install` package list the existing GitHub-Releases Linux build depends on (`-dev` packages vs. runtime packages are named differently in Debian/Ubuntu, e.g. `libwebkit2gtk-4.1-dev` for building vs. `libwebkit2gtk-4.1-0` for running) | A snap built successfully but crashing at runtime due to a missing shared library, caught by the same manual smoke-test as the plug-list failure mode above | Non-functional snap shipped if not caught pre-release | Both failure modes route through the same manual smoke-test gate documented in the answer sheet; this is a known category of Snapcraft packaging mistake (confusing `-dev` build-time packages with runtime `-N` versioned packages) worth naming explicitly rather than leaving implicit |

## §11 — Implementation notes

- **This FR deliberately does not attempt to automate Flathub submission end-to-end** — per §2's architectural explanation, doing so would misrepresent how Flathub's review-and-build model actually works. The manifest is prepared to a locally-verifiable standard (AC #4) and the PR-opening step remains a Stephen-gated action, structurally guaranteed by AC #6, not merely documented as a should-not.
- **The `.deb`-payload-as-snap-source approach (§3's CI staging step) is chosen over a separate `rust`-plugin Cargo build inside the snap** to avoid maintaining two independent build configurations for the same binary — the existing `tauri build --bundles deb` invocation is already exercised by the GitHub-Releases pipeline, so reusing its output as the snap's `dump`-plugin source means the snap always ships exactly what the GitHub-Releases `.deb` ships, with no drift risk between the two distribution channels.
- **Snap Store and Flathub both require zero payment and zero paid developer account**, unlike TASK-APP-003 (Apple Developer Program) and TASK-APP-004 (Microsoft Partner Center, though Partner Center itself is also free — the constraint there was Azure AD/EV-cert tooling, not a listing fee). This is noted in `risk_if_skipped` and worth keeping visible: the friction in this FR is entirely process/verification friction (confinement review, manifest correctness, app-id conventions), not financial gatekeeping.
- **The exact Flathub source-pinning strategy (§6) and the `.desktop` file's final content (§6) are named as explicitly deferred, not silently absent** — both depend on the §9 app-id decision landing first, and authoring them against a placeholder app-id risks producing files that need rework the moment §9 resolves.

*End of TASK-APP-005.*
