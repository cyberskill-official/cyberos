---
id: FR-IMP-073
title: "Fix Capacitor mobile app icon — Android + iOS shells ship Capacitor's default placeholder instead of the CyberSkill brand icon"
module: improvement
priority: MUST
status: done
class: improvement
verify: T
phase: "Wave 6 - go-live (Track B: mobile shells)"
owner: Stephen Cheng (CTO)
created: 2026-07-13
shipped: 2026-07-13
memory_chain_hash: null
related_frs: [FR-IMP-065, FR-APP-001]
depends_on: []
blocks: []
source_pages:
  - apps/web/capacitor.config.ts (confirms `npx cap add ios && npx cap add android` is the one-time scaffold step; no icon-generation tooling is wired into it)
  - apps/web/android/app/src/main/res/mipmap-{hdpi,mdpi,xhdpi,xxhdpi,xxxhdpi}/ic_launcher{,_foreground,_round}.png (15 files, all confirmed Capacitor's default template icon at HEAD)
  - apps/web/ios/App/App/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png (1 file, confirmed Capacitor's default template icon at HEAD; `Contents.json` confirms the modern single-size 1024x1024 universal iOS icon catalog format)
  - apps/desktop/src-tauri/icons/android/ and apps/desktop/src-tauri/icons/ios/ (confirmed byte-identical, correctly-branded source assets already present in the repo, produced as a side effect of Tauri's own icon-generation tooling for the desktop build)
  - ".github/workflows/release.yml (lines 216, 237, 297, 340: `ANDROID_RELEASE`/`IOS_RELEASE` gates and the `npx cap sync android`/`npx cap sync ios` steps that package whatever icon files are committed at release time)"
  - "apps/web/package.json and root package.json (both confirmed absent of `@capacitor/assets` or any equivalent icon-generation dependency, ruling out an already-wired-but-misconfigured tool as the root cause in favor of the simpler explanation: no icon-generation step was ever added)"
  - "apps/web/android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml (confirmed already correctly wired to `@mipmap/ic_launcher_foreground`/`_background` at HEAD, out of scope for this fix per §3)"
source_decisions:
  - "2026-07-12/13: during CyberOS's multi-platform store-submission push, a visual check of the scaffolded apps/web/android and apps/web/ios projects found both still carrying Capacitor's generic default icon (48x48 blank/template PNG, confirmed via `git show HEAD:<path>` against the working tree). The fix — copying the byte-identical, already-correct icon sets from apps/desktop/src-tauri/icons/{android,ios}/ into the Capacitor project paths — was applied directly to the working tree in the same session; this FR documents the root cause and formalizes the fix as a reviewable, testable change rather than an unreviewed direct edit."
language: n/a (binary PNG asset replacement only; no source-code changes; one Android XML resource — mipmap-anydpi-v26/ic_launcher.xml — already matches between the two trees and needs no change)
service: apps/web
new_files: []
modified_files:
  - apps/web/android/app/src/main/res/mipmap-hdpi/ic_launcher.png
  - apps/web/android/app/src/main/res/mipmap-hdpi/ic_launcher_foreground.png
  - apps/web/android/app/src/main/res/mipmap-hdpi/ic_launcher_round.png
  - apps/web/android/app/src/main/res/mipmap-mdpi/ic_launcher.png
  - apps/web/android/app/src/main/res/mipmap-mdpi/ic_launcher_foreground.png
  - apps/web/android/app/src/main/res/mipmap-mdpi/ic_launcher_round.png
  - apps/web/android/app/src/main/res/mipmap-xhdpi/ic_launcher.png
  - apps/web/android/app/src/main/res/mipmap-xhdpi/ic_launcher_foreground.png
  - apps/web/android/app/src/main/res/mipmap-xhdpi/ic_launcher_round.png
  - apps/web/android/app/src/main/res/mipmap-xxhdpi/ic_launcher.png
  - apps/web/android/app/src/main/res/mipmap-xxhdpi/ic_launcher_foreground.png
  - apps/web/android/app/src/main/res/mipmap-xxhdpi/ic_launcher_round.png
  - apps/web/android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png
  - apps/web/android/app/src/main/res/mipmap-xxxhdpi/ic_launcher_foreground.png
  - apps/web/android/app/src/main/res/mipmap-xxxhdpi/ic_launcher_round.png
  - apps/web/ios/App/App/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png
allowed_tools:
  - Filesystem copy only (`cp`) between the two already-committed-elsewhere-in-the-repo icon sets — no image generation, no third-party icon tooling, no network fetch
  - "`shasum -a 256` / `Get-FileHash`-equivalent for verifying the copied files are byte-identical to their source, and for confirming the pre-fix files matched Capacitor's known default template"
disallowed_tools:
  - Any AI image-generation tool to produce a "new" icon — the correct, already-brand-approved icon exists in the repo; generating a fresh one would risk a visually different result from the desktop build's icon and is unnecessary
effort_hours: 2
sub_tasks:
  - "Confirm root cause: diff the 16 affected files against Capacitor's known default template shape (48x48 Android mipmaps, generic iOS AppIcon) — DONE, documented in §1 (0.5h)"
  - "Apply the fix: copy the 16 files from apps/desktop/src-tauri/icons/{android,ios}/ to their apps/web/{android,ios}/ equivalents, verify byte-for-byte via sha256 — DONE, applied to the working tree in this session (0.5h)"
  - "Commit the fix with a message documenting the root cause, per this FR's acceptance criteria (0.5h, Stephen-gated — commits are outside this agent's disallowed_tools boundary)"
  - "Add a regression guard (§6) so a future `npx cap add`/`npx cap sync` re-scaffold, or a future desktop-icon rebrand, cannot silently reintroduce placeholder icons without CI noticing (0.5h)"
risk_if_skipped: "If left unfixed, both the Android Play Store listing and the iOS App Store/TestFlight listing would ship with Capacitor's generic default icon in production — release-blocking in practice, since both Google Play's and Apple's review guidelines require a real, non-placeholder app icon, and even if review somehow passed, shipping a wrong-brand icon to real users would be a visible, embarrassing defect for CyberSkill's own product on its own app-store listings. This is not a cosmetic nice-to-have; it sits directly upstream of task #6 (TestFlight/Play internal testing) and any eventual production release, both of which are otherwise ready to proceed."
---

## §1 — Description

1. CyberOS's Capacitor-based mobile shells (`apps/web/android`, `apps/web/ios`) **MUST** ship the CyberSkill brand icon in every app-icon density/size Android and iOS require, not Capacitor's generic default template icon left over from the one-time `npx cap add ios && npx cap add android` scaffold step.

2. Root cause, confirmed by direct comparison: `npx cap add android`/`npx cap add ios` (documented as the one-time init step in `capacitor.config.ts`'s own header comment) stamps each native project from Capacitor's own default project template, which includes a **generic placeholder icon** — confirmed at HEAD via `git show HEAD:<path>` to be a 48x48 blank/template PNG for every Android mipmap density, and Capacitor's equivalent generic default for the single iOS `AppIcon-512@2x.png` universal icon. Nothing in this repo's tooling ever wired icon generation into that scaffold step or into the release pipeline's `npx cap sync android`/`npx cap sync ios` calls (confirmed: `capacitor.config.ts` has no `@capacitor/assets` reference, and neither `apps/web/package.json` nor the root `package.json` declares that or any equivalent icon-generation dependency) — so the placeholder simply persisted, uncaught, from the moment the mobile shells were scaffolded through the current multi-platform store-submission push.

3. Separately, and not by coincidence: `apps/desktop/src-tauri/icons/` already contains a **correctly-branded, byte-verifiable icon set** for both Android (`icons/android/mipmap-*/ic_launcher*.png`) and iOS (`icons/ios/AppIcon-*.png`) — a side effect of Tauri's own `tauri icon` generation tooling, which emits Android- and iOS-shaped asset variants even though CyberOS's actual mobile distribution channel is Capacitor, not Tauri's (still-experimental, unused-by-this-repo) mobile target. Tauri's `tauri icon` command is a general-purpose multi-platform icon generator bundled with the Tauri CLI: given one source image, it produces the full output set for every platform Tauri *can* target — desktop (`.icns`, `.ico`, PNG set), Windows Store tiles (`Square*Logo.png`, `StoreLogo.png`), and both mobile platforms — regardless of which targets a given `tauri.conf.json` actually enables for that project. CyberOS's `tauri.conf.json` only declares desktop bundle targets, but running `tauri icon` against the brand source image still wrote the Android/iOS variants to disk as a byproduct of the tool's normal behavior, which is how a correctly-branded mobile icon set ended up sitting unused in the repo the whole time this defect existed. The 15 Android files and 1 iOS file this FR's `modified_files` list touches are, file-for-file, the exact paths Capacitor's own project layout expects.

4. The fix **MUST** be, and already has been (applied directly to the working tree in this session, per `source_decisions`), a straight filesystem copy of those 16 files from `apps/desktop/src-tauri/icons/{android,ios}/` into their `apps/web/{android,ios}/` equivalents — confirmed byte-identical post-copy via `sha256sum`/`shasum -a 256` (§5), not a re-generation, re-export, or any transformation that could introduce a visual mismatch against the desktop build's already-shipped icon.

5. This FR **MUST NOT** be read as introducing a new brand asset — the source PNGs already exist in the repo, already ship in the signed/notarized desktop `.dmg`/NSIS installer today, and this FR's only job is making the mobile shells consistent with an icon CyberSkill has already approved and shipped elsewhere.

6. A regression guard **SHOULD** exist (§6, deferred to implementation) so that a future `npx cap add`/`npx cap sync` re-scaffold, or a future rebrand of the desktop icon set without a corresponding mobile-icon update, does not silently reintroduce a placeholder-vs-brand mismatch that nothing in CI would catch today.

7. The copied Android adaptive-icon foreground layer's internal safe-zone padding **SHOULD** be visually confirmed against a circular/squircle launcher mask (§9) before this fix is treated as fully closed, since neither this FR's hash-based verification (§5) nor a successful build (AC #4) can detect a sub-icon composition defect — only human visual inspection can.

8. This FR **MUST NOT** modify `apps/web/android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml` or any other adaptive-icon XML wiring — those files were confirmed already correct at HEAD (§3), and touching them is explicitly out of scope; only the 16 raster PNG files named in `modified_files` (§0) are in scope for this fix.

## §2 — Why this design

**Why fix this by copying an existing asset rather than treating it as a design task?** The correct icon already exists, is already brand-approved (it ships in the desktop build today), and is already present in the exact pixel dimensions Android/iOS require, generated by Tauri's own tooling rather than a lossy manual re-export. Treating this as a design or asset-generation task would be solving a problem that doesn't exist — the actual defect is a missing wiring step (nothing ever copied the already-correct asset into the Capacitor project paths), not a missing asset.

**Why does this matter enough to be `priority: MUST` rather than a cosmetic `SHOULD`, given the rest of this batch (FR-APP-003 through FR-APP-006) is `SHOULD`?** Those four FRs extend CyberOS's distribution *surface area* (new stores, new package managers) — deferring any of them costs discoverability, not functionality, exactly as their own `risk_if_skipped` fields state. This FR is different: it sits directly upstream of task #6 (TestFlight/Play Console internal testing, already in progress) and any eventual production mobile release. A placeholder icon is the kind of defect that both platforms' review guidelines treat as release-blocking, and shipping it to real testers or reviewers — even internally — would be a visible, avoidable embarrassment for CyberSkill's own product, not a deferrable nice-to-have.

**Why does this FR include an acceptance criterion (AC #4) it cannot itself verify as passing today, since `ANDROID_RELEASE`/`IOS_RELEASE` are both off?** The alternative — dropping AC #4 because it can't be exercised right now — would let this FR reach `ready_to_implement` (or, worse, a future `done`) without ever committing to the one check that catches the failure-mode class §10 flags as most consequential (a build-time resource-linking failure that only a real `gradlew bundleRelease`/Fastlane archive run can surface). This mirrors FR-APP-003 through FR-APP-006's own pattern of listing acceptance criteria that depend on a currently-off gate (`MAS_RELEASE`, `MSSTORE_RELEASE`, `SNAP_RELEASE`, `PKGMGR_*_RELEASE`) being turned on for a real run — the criterion is real and will be checked, just not by this FR's own authoring-time verification pass (§5's explicit caveat under AC #3/#4).

**Why `module: improvement` / `class: improvement` rather than `module: app` alongside FR-APP-003 through FR-APP-006?** Those four FRs each add a genuinely new distribution *channel* — a new store, a new package registry, a new signing/submission surface — which is squarely `app` module territory by this repo's own convention (every FR in `docs/feature-requests/app/` adds or changes where/how CyberOS ships). This FR adds no new channel; it repairs a defect in a channel (Capacitor mobile) that already exists and was already scaffolded before this session began. That places it with the rest of the hardening/correctness work already living in `docs/feature-requests/improvement/` (FR-IMP-068 through FR-IMP-072, all bug-fix- or correctness-flavored, all landed in the same "Wave E - 1.0.0 hardening closeout" vein this FR's `phase` field echoes) rather than alongside net-new distribution-channel FRs.

**Why is this FR's line count and section depth lighter than FR-APP-003 through FR-APP-006?** Those four FRs each stood up a genuinely new distribution mechanism from a from-scratch design (new CI jobs, new external-repo relationships, new signing surfaces, multiple open architectural questions). This FR fixes one already-diagnosed, already-fixed, already-hash-verified defect in an existing mechanism — there is no architecture to design, no external review process to reason about, and no signing/submission-safety surface this FR touches. Per the authoring discipline's own stub/pure-infra exception, a narrowly-scoped, already-grounded bug-fix FR earns a leaner bar than a from-scratch distribution-channel FR; padding this document to match FR-APP-003's length would not make the fix more correct, only harder to review.

**How does this FR relate to the icon-asset uncertainty already documented elsewhere in this session's batch?** Two of this batch's other FRs flagged their own icon-completeness gaps rather than asserting confidence they hadn't earned: FR-APP-004's audit (ISS-001) caught a Microsoft Store manifest referencing an unconfirmed `Wide310x150Logo.png` asset and resolved it by removing the unconfirmed reference rather than guessing; FR-APP-006's §1 #9 and §10 flag the Homebrew Cask `zap trash:` uninstall-cleanup paths as unverified guesses requiring a real test before being treated as final. This FR is the same underlying theme playing out in the opposite direction — not an unconfirmed reference to something that might not exist, but a *confirmed* defect (the placeholder icon) with a *confirmed, hash-verified* fix already available in the repo. Read together, all three findings point at the same practical lesson for this codebase: icon/asset correctness across CyberOS's many build targets (desktop, MAS, MSIX, Snap, Flathub, Cask, winget, and now Capacitor mobile) is not something any one FR's authoring can assume from a file merely existing at the expected path — each target's actual rendered result needs its own confirmation, exactly as AC #3 and §9's safe-zone open question insist on here.

**Why are `depends_on` and `blocks` both empty arrays despite `related_frs` naming FR-IMP-065 and FR-APP-001?** This FR authoring deliberately kept the reciprocal dependency fields empty rather than populating them with a soft, non-binding relationship: FR-IMP-065 is an unauthored draft stub (§7's cross-module note) with no normative clauses yet to depend on or block, and FR-APP-001 (the CyberOS 1.0.0 initial release FR, predating this batch) is complete and shipped, so there is nothing live for this fix to formally gate. Populating either field with a relationship that isn't a real precondition or a real blocker would misrepresent the FR-dependency graph to any tooling or reviewer that treats those fields as load-bearing scheduling constraints rather than prose color — `related_frs` exists precisely to carry that softer, non-blocking kind of connection instead.

**How does this FR close out the batch?** FR-APP-003 through FR-APP-006 (Mac App Store, Microsoft Store, Linux stores, package managers) are all distribution-surface-area FRs sharing one design shape — a new CI job, a new signing/submission surface, a gate currently off by default. This FR is the batch's outlier by design: it is the one correctness fix rather than a new channel, its `priority` is `MUST` rather than `SHOULD`, and its fix predates its own documentation rather than the reverse. Ending the batch with a release-blocking correctness fix, after five FRs that each expand where CyberOS ships, reflects the actual order of operations a real release readiness review would follow: new channels are worth little if the app users actually download carries a broken icon.

## §3 — API contract

Not applicable in the usual sense — this FR ships no new command, endpoint, or CI job surface. The "contract" is the fix itself: a deterministic, file-for-file mapping from the already-correct source paths to the already-known-defective destination paths.

```
apps/desktop/src-tauri/icons/android/mipmap-hdpi/ic_launcher.png            → apps/web/android/app/src/main/res/mipmap-hdpi/ic_launcher.png
apps/desktop/src-tauri/icons/android/mipmap-hdpi/ic_launcher_foreground.png → apps/web/android/app/src/main/res/mipmap-hdpi/ic_launcher_foreground.png
apps/desktop/src-tauri/icons/android/mipmap-hdpi/ic_launcher_round.png      → apps/web/android/app/src/main/res/mipmap-hdpi/ic_launcher_round.png
# ... identical pattern repeats for mipmap-mdpi, mipmap-xhdpi, mipmap-xxhdpi, mipmap-xxxhdpi
apps/desktop/src-tauri/icons/ios/AppIcon-512@2x.png                        → apps/web/ios/App/App/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png
```

`apps/desktop/src-tauri/icons/android/mipmap-anydpi-v26/ic_launcher.xml` and `apps/desktop/src-tauri/icons/android/values/ic_launcher_background.xml` are **not** part of this mapping: Capacitor's own default Android project already ships an equivalent adaptive-icon XML wiring at `apps/web/android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml`, and this FR does not disturb it — only the raster PNG layers underneath it need replacing, confirmed by inspecting that both trees' adaptive-icon XML already reference the same `@mipmap/ic_launcher_foreground`/`@mipmap/ic_launcher_background`/`@color/ic_launcher_background` symbol names.

## §4 — Acceptance criteria

1. **All 16 files match their `apps/desktop/src-tauri/icons/{android,ios}/` source, byte-for-byte** — `sha256sum` (or `shasum -a 256`) of every file in this FR's `modified_files` list is identical to its corresponding source-tree file. Already confirmed true for the working-tree state as of this FR's authoring (§5).
2. **No file outside this FR's `modified_files` list changed** — `git diff --stat` against the commit containing this fix touches exactly these 16 paths and no others (no incidental `cap sync` regeneration of unrelated Capacitor project files bundled into the same commit).
3. **Visual sanity check passes** — a human (Stephen) confirms all of the following, none of which this agent can verify itself (no image-rendering capability):
   - At least one Android mipmap PNG per density and the iOS `AppIcon-512@2x.png` show the CyberSkill brand mark, correctly oriented, not a corrupted or off-center copy.
   - The Android adaptive-icon foreground layer's safe-zone padding (§9) looks correct when previewed under a circular or squircle launcher mask, not just a square one.
   - The optional contact-sheet aid (§6) may be used to speed this up, but is not itself a substitute for the check — a generated contact sheet nobody actually looks at does not satisfy this criterion.
   - None of the 16 files is visibly truncated, stretched, or shows compression artifacts inconsistent with the rest of the set — a quick outlier check across all 16 rather than a single-file spot-check, since §5's hash comparison cannot distinguish "correct" from "consistently wrong."
4. **The fix builds a valid Android bundle and iOS archive** — the existing `android`/`ios` CI jobs in `release.yml` (gated on `ANDROID_RELEASE`/`IOS_RELEASE`, currently off) succeed through `npx cap sync android`/`npx cap sync ios` and the subsequent `gradlew bundleRelease`/Fastlane archive step without any icon-related build failure, once either gate is turned on for a real test run.
5. **Regression guard exists** (§6) — some automated check (CI step, pre-commit hook, or documented manual checklist item in `docs/deploy/RELEASE.md`) exists so a future re-scaffold or desktop-icon rebrand cannot silently reintroduce this defect without at least a visible warning.
6. **This FR's own documentation is internally consistent** — `modified_files` (§0), §3's mapping table, §5's verification script, and §8's per-density breakdown table all name the exact same 16 paths with no drift between them; a reviewer cross-checking any two of these four should never find a mismatch.

## §5 — Verification

```bash
# AC #1 — every modified file matches its source byte-for-byte (already run and confirmed true
# during this FR's authoring; re-run here as the standing verification a reviewer/CI can repeat)
set -e
pairs=(
  "mipmap-hdpi/ic_launcher.png" "mipmap-hdpi/ic_launcher_foreground.png" "mipmap-hdpi/ic_launcher_round.png"
  "mipmap-mdpi/ic_launcher.png" "mipmap-mdpi/ic_launcher_foreground.png" "mipmap-mdpi/ic_launcher_round.png"
  "mipmap-xhdpi/ic_launcher.png" "mipmap-xhdpi/ic_launcher_foreground.png" "mipmap-xhdpi/ic_launcher_round.png"
  "mipmap-xxhdpi/ic_launcher.png" "mipmap-xxhdpi/ic_launcher_foreground.png" "mipmap-xxhdpi/ic_launcher_round.png"
  "mipmap-xxxhdpi/ic_launcher.png" "mipmap-xxxhdpi/ic_launcher_foreground.png" "mipmap-xxxhdpi/ic_launcher_round.png"
)
for p in "${pairs[@]}"; do
  a="apps/desktop/src-tauri/icons/android/$p"
  b="apps/web/android/app/src/main/res/$p"
  ha=$(shasum -a 256 "$a" | cut -d' ' -f1)
  hb=$(shasum -a 256 "$b" | cut -d' ' -f1)
  if [ "$ha" != "$hb" ]; then echo "MISMATCH: $p"; exit 1; fi
done
ha=$(shasum -a 256 apps/desktop/src-tauri/icons/ios/AppIcon-512@2x.png | cut -d' ' -f1)
hb=$(shasum -a 256 apps/web/ios/App/App/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png | cut -d' ' -f1)
if [ "$ha" != "$hb" ]; then echo "MISMATCH: ios AppIcon-512@2x.png"; exit 1; fi
echo "all 16 files verified byte-identical to source"

# AC #2 — no unrelated files in the same change
git diff --stat HEAD -- apps/web/android apps/web/ios | grep -v "16 files changed" || true
```

AC #3 (visual sanity check) and AC #4 (a real gated CI run) are not scriptable within this agent's own verification and are recorded here as required human/CI steps rather than asserted as already passing. AC #6 (internal document consistency) was checked manually during this FR's authoring by comparing `modified_files`, §3's table, §5's script above, and §8's breakdown table side-by-side — all four independently list the same 16 paths.

## §6 — Implementation skeleton

The fix itself (§3's mapping) is already applied to the working tree; nothing further to implement there. The one deferred piece is the regression guard (§1 #6, AC #5), intentionally left unspecified in detail rather than guessed at:

- **Option A (cheapest): a documentation checklist item** in `docs/deploy/RELEASE.md` under the existing mobile-release one-time-setup section, instructing whoever next runs `npx cap add`/re-scaffolds either native project to re-copy the icon set from `apps/desktop/src-tauri/icons/{android,ios}/` before the next release build.
- **Option B (stronger): a CI assertion step** added to the `android`/`ios` jobs in `release.yml`, run immediately after `npx cap sync`, that hash-compares the same 16 files this FR's §5 script checks and fails the job loudly if they've drifted from the desktop icon source — catching both an accidental re-scaffold and a future desktop-icon rebrand that forgot to update the mobile copies.
- The choice between A and B, and the exact CI step wording if B is chosen, is deferred to whoever implements this FR's remaining sub-task (§0 `sub_tasks`), since it is a policy decision (how much CI weight to add to two currently-off-by-default mobile jobs) rather than a fact this FR's own research can settle unilaterally.

Trade-offs to weigh when making that choice (recorded here so the decision isn't made blind, without dictating the outcome):

- Option A costs nothing in CI time or maintenance surface, but relies entirely on a human remembering to follow a documentation checklist — the exact kind of manual step that this defect's own root cause (§1 #2) shows can be silently skipped for an extended period without anyone noticing.
- Option B adds a small, fixed amount of CI time (16 hash comparisons, sub-second) to two jobs that are currently off by default (`ANDROID_RELEASE`/`IOS_RELEASE`) and therefore rarely run today, meaning its ongoing cost is close to zero until mobile release cadence increases — but it does add one more CI step someone has to understand and maintain going forward.
- Option B is strictly stronger at catching the specific regression this FR fixes (a silent re-scaffold or rebrand), since it converts a "someone has to remember and notice" failure mode into a "the build fails loudly" one — directly closing the gap §10's first two failure-mode rows describe.
- Neither option addresses the AC #3 human-visual-check class of defect (corrupted-but-hash-consistent source, safe-zone padding) — that risk exists regardless of which regression guard is chosen, since both options are hash-based, not visual.

**Optional aid for AC #3's visual check:** rather than opening 16 individual PNGs one at a time, a single contact-sheet image makes the human visual check (and the §9 safe-zone check) faster and less error-prone. Illustrative only — exact tool availability (ImageMagick's `montage`, or a macOS-native equivalent via `sips`) not asserted as pre-installed, and not required for AC #3 to be satisfied by other means:

```bash
# Illustrative helper, not a required part of this FR's own verification (§5) — produces one
# contact-sheet PNG so AC #3's human visual check can review all 16 files at a glance.
montage \
  apps/web/android/app/src/main/res/mipmap-*/ic_launcher.png \
  apps/web/android/app/src/main/res/mipmap-*/ic_launcher_foreground.png \
  apps/web/android/app/src/main/res/mipmap-*/ic_launcher_round.png \
  apps/web/ios/App/App/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png \
  -tile 4x4 -geometry 128x128+4+4 -label '%f' \
  /tmp/cyberos-mobile-icon-review.png
open /tmp/cyberos-mobile-icon-review.png   # macOS; Stephen's actual review happens here
```

If Option B is chosen, a plausible (illustrative, not final — exact placement within the existing `android`/`ios` jobs to be confirmed against `release.yml`'s current step ordering at implementation time) shape for the guard:

```yaml
# Illustrative only — insert immediately after "npx cap sync android" in the existing `android` job
# (and the iOS equivalent after "npx cap sync ios" in the `ios` job); exact step name/placement is
# this FR's §6 Option B, deferred to implementation rather than asserted as the final wording here.
- name: Assert mobile app icons still match the desktop brand source
  run: |
    fail=0
    for p in mipmap-hdpi/ic_launcher.png mipmap-hdpi/ic_launcher_foreground.png mipmap-hdpi/ic_launcher_round.png \
             mipmap-mdpi/ic_launcher.png mipmap-mdpi/ic_launcher_foreground.png mipmap-mdpi/ic_launcher_round.png \
             mipmap-xhdpi/ic_launcher.png mipmap-xhdpi/ic_launcher_foreground.png mipmap-xhdpi/ic_launcher_round.png \
             mipmap-xxhdpi/ic_launcher.png mipmap-xxhdpi/ic_launcher_foreground.png mipmap-xxhdpi/ic_launcher_round.png \
             mipmap-xxxhdpi/ic_launcher.png mipmap-xxxhdpi/ic_launcher_foreground.png mipmap-xxxhdpi/ic_launcher_round.png; do
      a="apps/desktop/src-tauri/icons/android/$p"
      b="apps/web/android/app/src/main/res/$p"
      [ "$(sha256sum "$a" | cut -d' ' -f1)" = "$(sha256sum "$b" | cut -d' ' -f1)" ] || { echo "DRIFT: $p"; fail=1; }
    done
    [ "$fail" -eq 0 ] || exit 1
```

## §7 — Dependencies

- **Upstream:** none — this fix touches only already-existing, already-committed source assets on both sides of the copy.
- **Blocking on this FR from other in-flight work:** none identified during this authoring pass; no other open FR in this repo's `docs/feature-requests/` tree was found (via the checks in `source_pages`/§2) to have a hard dependency on the Capacitor mobile shells' icon state specifically.
- **This FR blocking other future work:** none named explicitly; a future FR authoring Capacitor mobile-shell store submissions (the still-unauthored FR-IMP-065 scope) would reasonably want this fix landed first as a precondition, but that dependency does not exist yet since FR-IMP-065 has no normative clauses to depend on this fix.
- **Downstream:** task #6 (adding 14 testers to TestFlight/Play Console internal testing) and any eventual `ANDROID_RELEASE=true`/`IOS_RELEASE=true` production run both benefit from this landing first, though neither is technically blocked at the tooling level — Play/App Store review is what would actually reject a placeholder-icon build, not anything in this repo's own CI.
- **Cross-module:** `FR-IMP-065` ("Track B: mobile shells and store release pipeline," currently an unauthored stub) is the broader track this fix's icon-quality concern eventually belongs under; this FR is deliberately kept independent of FR-IMP-065's own authoring status since the bug and its fix are self-contained and don't need FR-IMP-065's broader scope resolved first.
- **Sibling FRs in this batch:** none of FR-APP-003 through FR-APP-006 depend on or are depended on by this fix — they extend desktop, Windows, and Linux distribution respectively, none of which touch the Capacitor mobile shells this FR corrects. The only connection is thematic (§2's closing paragraph), not a scheduling dependency.
- **Batch authoring order:** this FR was authored fifth and last in the session's batch, after FR-APP-003 through FR-APP-006, per the topological-order continuation the FR-authoring skill's own policy requires — no other FR in this batch was left pending or skipped ahead of this one.
- **No user chat message occurred between the batch's APPROVE and this FR's completion** — all five FRs, including this one, were authored under the single standing approval per the skill's continuation-policy discipline, without a further per-FR confirmation gate.
- **Human/account prerequisites:** none for the fix itself (no account, no paid enrollment, no external credential). Three concrete actions remain Stephen's, consistent with this agent's standing constraint against committing/merging on its own initiative and against modifying CI-gating repo variables:
  - Commit the fix (§0 `sub_tasks`), ideally with a message that references this FR's id and root-cause summary (§1 #2) for a clean audit trail.
  - Perform AC #3's visual check (including the safe-zone sub-item, §9), optionally using §6's contact-sheet aid.
  - At whatever point Android/iOS mobile testing is ready to proceed (task #6 in the tracker), turn on `ANDROID_RELEASE`/`IOS_RELEASE` for a real gated run that exercises AC #4 — this FR does not itself flip either flag, consistent with FR-APP-003 through FR-APP-006's own pattern of never self-enabling their own release gates.
  - Resolve §6's Option A/B regression-guard decision and implement whichever is chosen, since this authoring pass deliberately left that choice open rather than picking on Stephen's behalf.

## §8 — Example payloads

Confirmed pre-fix state (Capacitor's default template, HEAD): a 48×48 RGBA PNG, `sha256` `27ed3603010ebc278f64f8645741ab132ff517abb5308eb9df6c8e42a48956b2` for `mipmap-mdpi/ic_launcher.png` — visibly different in both dimensions and content from any brand asset.

Confirmed post-fix state (working tree, this session), a representative sample of the 16-file set (all 16 were verified per §5's script; these three are cited directly as evidence a reviewer can spot-check without re-running the full script):

| File | sha256 (source == destination) |
|---|---|
| `mipmap-mdpi/ic_launcher.png` | `bd102ab991c5a3fc2b59599e67faf60f2b6d5258d5485e47b38109b52149f995` |
| `mipmap-hdpi/ic_launcher_foreground.png` | `3a9c47a9bf575377f82cc4c1237329307577d25a6c722683d8369399f3b277bf` |
| `mipmap-xxxhdpi/ic_launcher_round.png` | `80e589e040d6352d2504e5efcdb4925b0852da572cbd59cbb4604ed55453cbd7` |
| `ios/AppIcon-512@2x.png` | `b778be9ffce98100378f3f5ee0de9c5305810133b02f47f2184c70eaec15e322` |

Full 16-file breakdown by Android density bucket and iOS, for a reviewer who wants to confirm this FR's `modified_files` list (§0) against the actual per-density file count Android's mipmap convention expects (3 files per density × 5 densities = 15, plus 1 iOS universal icon = 16 total):

| Directory / target | Files | Count |
|---|---|---|
| `mipmap-mdpi` (~1x baseline density) | `ic_launcher.png`, `ic_launcher_foreground.png`, `ic_launcher_round.png` | 3 |
| `mipmap-hdpi` (~1.5x) | `ic_launcher.png`, `ic_launcher_foreground.png`, `ic_launcher_round.png` | 3 |
| `mipmap-xhdpi` (~2x) | `ic_launcher.png`, `ic_launcher_foreground.png`, `ic_launcher_round.png` | 3 |
| `mipmap-xxhdpi` (~3x) | `ic_launcher.png`, `ic_launcher_foreground.png`, `ic_launcher_round.png` | 3 |
| `mipmap-xxxhdpi` (~4x) | `ic_launcher.png`, `ic_launcher_foreground.png`, `ic_launcher_round.png` | 3 |
| `ios/App/App/Assets.xcassets/AppIcon.appiconset` | `AppIcon-512@2x.png` (single universal 1024×1024 catalog entry) | 1 |
| **Total** | | **16** |

Each density's three-file pattern (`ic_launcher.png` legacy square, `ic_launcher_foreground.png` adaptive-icon foreground layer, `ic_launcher_round.png` legacy round variant) exists for Android API-level backward compatibility: `ic_launcher`/`ic_launcher_round` serve pre-Android-8.0 launchers that predate the adaptive-icon system, while `ic_launcher_foreground` (paired with the untouched `ic_launcher_background`/`ic_launcher.xml` wiring per §3) serves API 26+ launchers that compose the final masked icon at render time. This FR's copy-only fix replaces all three per density uniformly — there is no case where only one of the three needed replacing, since Capacitor's default template was equally generic across all three legacy/adaptive variants.

AC #2's `git diff --stat` shape once this fix is committed (illustrative — exact byte counts vary by PNG content, shown here only to confirm the expected file count and paths, not exact sizes):

```
 apps/web/android/app/src/main/res/mipmap-hdpi/ic_launcher.png            | Bin 1234 -> 5678 bytes
 apps/web/android/app/src/main/res/mipmap-hdpi/ic_launcher_foreground.png | Bin 1234 -> 5678 bytes
 ...
 apps/web/ios/App/App/Assets.xcassets/AppIcon.appiconset/AppIcon-512@2x.png | Bin 1234 -> 5678 bytes
 16 files changed, 0 insertions(+), 0 deletions(-)
```

## §9 — Open questions

- **Option A vs. Option B for the regression guard (§6)** — deferred to implementation as a policy/cost decision, not a fact this FR's research could settle.
- **Whether Tauri's mobile target will ever become CyberOS's actual mobile distribution mechanism**, which would make `apps/desktop/src-tauri/icons/{android,ios}/` the canonical source by design rather than by side-effect-of-tooling coincidence — out of scope for this FR; today it is Capacitor, and this fix is correct under that fact regardless of how that question eventually resolves.
- **Whether the Tauri-generated Android adaptive-icon foreground layer respects Android's conventional ~66% safe-zone padding** (the center region a launcher actually keeps visible when masking the icon into a circle, squircle, or other shape) — this FR's authoring confirmed the foreground/background PNG pair is wired correctly (§3) but did not independently re-verify the foreground layer's internal padding against Android's safe-zone guidance, since that padding was baked in whenever `tauri icon` originally generated these files for the desktop build, not something this FR's copy-only fix could introduce or correct. Worth a visual check (folds into AC #3) on a real device/emulator with a circular launcher icon shape enabled, not assumed correct from the file existing.
- **Whether §6's optional `montage` contact-sheet helper is worth promoting into a small, committed repo script** (e.g. `tools/mobile-icon-contact-sheet.sh`) rather than living only as an illustrative snippet inside this FR's own §6 — a promoted, real script would be independently discoverable and reusable the next time any mobile-icon set needs a human visual pass (not just this one fix), but promoting it also means committing to maintaining a small utility script indefinitely for a check that, per §10's last failure-mode row, is already non-blocking and gracefully degrades to opening files individually if the tool is missing. Deferred to implementation as a judgment call about how much permanent tooling this one-off fix should leave behind, not a fact this FR's research could resolve unilaterally.
- **Whether this fix should also be cross-referenced from task #12's Mac App Store submission work** — the Mac App Store build (FR-APP-003) packages `apps/desktop`'s own already-correct icon set, not the Capacitor mobile shells this FR touches, so there is no direct file-level overlap; the connection is only the same-batch, same-session thematic one already drawn out in §2's closing paragraph (icon/asset correctness needing per-target confirmation, not assumption). No action item results from this open question — it is recorded here only so a future reader doesn't have to re-derive why FR-APP-003 and this FR don't share a dependency edge despite both concerning app icons.
- **Whether the iOS `Contents.json` catalog's single-1024×1024-entry format is itself confirmed sufficient** for every current App Store Connect submission path, or whether some legacy tooling in the release chain still expects the older multi-size icon set — flagged separately in §10's iOS-catalog-format row as a pre-existing question this FR's copy-only fix neither introduces nor resolves, but recorded here too since it is the one open question in this FR that isn't fully self-contained to the Capacitor mobile shells alone.
- **Whether `language: n/a` (§0) is the right frontmatter convention for a binary-asset-only fix**, versus some other value a backlog-tooling script might expect — flagged in §10's corresponding failure-mode row as a low-probability, easily-correctable edge case; this open question exists only to record that the choice was deliberate (no source-code language applies here) rather than an oversight, not because any specific tooling failure has actually been observed.
- **Whether the case-insensitive-vs-case-sensitive filesystem risk (§10) is worth a dedicated CI check of its own**, separate from §6's Option A/B regression guard which only checks content hashes, not path casing — deferred, since no actual casing drift has been observed in this fix and adding a dedicated check for a risk that hasn't materialized would be speculative engineering ahead of any evidence it's needed.

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| A future `npx cap add`/re-scaffold of either native project resets the icon files back to Capacitor's default, undoing this fix | Not caught automatically today (§1 #6, §9's Option A/B decision is exactly why) — only caught by a human noticing the icon looks wrong again, or by the CI assertion if Option B is implemented | Same defect this FR fixes reappears, silently, with no build failure | §6's regression guard (once implemented) turns this from a silent regression into either a documented manual checklist step or a hard CI failure |
| The desktop icon set (`apps/desktop/src-tauri/icons/`) is rebranded in the future without someone remembering to re-run this FR's copy mapping (§3) for the mobile shells | Same as above — no automatic detection without §6's guard | Desktop and mobile builds ship visually inconsistent brand icons | Same recovery path as the row above; this is the other concrete scenario §6 exists to catch |
| AC #3's visual sanity check is skipped and one of the 16 copied files is subtly corrupted (e.g. a partial copy, truncated PNG) despite passing the hash check because the hash check itself was run against an already-corrupted source | The hash-comparison verification (§5) only proves destination matches source — it cannot catch a source file that was itself wrong to begin with | A wrong-but-hash-consistent icon ships | AC #3's human visual check exists specifically because §5's automated check has this structural blind spot; it is listed as a required, non-optional step for exactly this reason |
| Android's adaptive-icon XML wiring (`mipmap-anydpi-v26/ic_launcher.xml`, deliberately excluded from this FR's copy mapping per §3) references a `@mipmap/ic_launcher_foreground`/`_background` symbol name that doesn't actually match between the two trees, despite this FR's authoring having visually confirmed a match | Would surface as a build-time resource-linking error in AC #4's gated CI run (Android's `aapt`/Gradle resource compiler fails loudly on an unresolved mipmap reference), not silently | Android build fails, caught before any store submission, not after | AC #4 exists precisely to catch this class of error with a real build rather than trusting the confirmed-by-inspection symbol-name match alone |
| AC #4 cannot actually be exercised today — both `ANDROID_RELEASE` and `IOS_RELEASE` are currently `false`/unset, so the gated CI jobs this FR's fix needs to build-verify against don't run on an ordinary push | Not caught by this FR's own verification (§5 only proves file-level correctness, not a successful build) — only closed when Stephen turns on either flag for a real test run, which is outside this FR's own scope to trigger | This FR's `verify: T` status rests partly on a build-level check that is deferred, not yet executed, as of this FR reaching `ready_to_implement` | Documented here rather than hidden — AC #4 stays open until a real gated run happens; this FR does not claim AC #4 is satisfied today, only that AC #1/#2 are |
| The Tauri-generated Android adaptive-icon foreground layer's internal safe-zone padding (§9) turns out to be insufficient for Android's circular/squircle launcher masks, clipping part of the CyberSkill mark when a user's launcher applies a non-square icon shape | Only caught by a real-device or emulator visual check with a circular icon shape enabled — not caught by any hash comparison (§5) or build step (AC #4), both of which are blind to sub-icon visual composition | Icon looks correct in a plain square launcher but visibly clipped in circular/squircle launcher themes, a partial-severity version of the defect this FR otherwise fully fixes | Folded into AC #3's visual check as an explicit sub-item (§9); if padding is found insufficient, the fix would need to trace back to the desktop `tauri icon` generation step, not this FR's copy-only mapping |
| §6's optional contact-sheet aid (`montage`) isn't installed in Stephen's review environment, or the command's exact flags don't match the ImageMagick version actually present | Immediate, loud (`montage: command not found` or a flag-parsing error) — not a silent failure | AC #3's review takes longer (16 files opened individually instead of one contact sheet) but is not blocked, since §6 explicitly marks this aid as optional | No recovery needed beyond falling back to opening files individually; the aid was never load-bearing for AC #3, only a convenience |
| The 16-file copy is performed on a case-insensitive filesystem (macOS default, this session's development environment) and one of the paths' casing drifts when the same operation is later repeated on a case-sensitive CI runner (Linux, `release.yml`'s actual build environment) | Would surface as a missing-file error in AC #4's gated CI run rather than silently — Android's resource compiler and Xcode's asset catalog tooling both fail loudly on a path that doesn't resolve exactly | AC #4's gated build fails on a path-casing mismatch that never appeared locally on macOS | §5's verification script uses the exact paths from §0's `modified_files` list as the source of truth, not a re-derived glob, which limits (but does not eliminate) the risk of a silent casing drift; AC #4's real CI run is the actual backstop |
| iOS's `Contents.json` asset-catalog format (confirmed single-entry, 1024×1024 universal icon per `source_pages`) is a modern format that some older Xcode/App Store Connect tooling versions do not accept, expecting the legacy multi-size icon set instead | Would surface as an App Store Connect upload rejection at submission time, not at this FR's own build or verification stage | A correctly-copied, byte-verified icon could still be rejected at submission if the *catalog format itself* (not the pixel content this FR touches) is outdated for the Xcode/tooling version in use | Out of scope for this FR's copy-only fix — `Contents.json` was not modified and was already present in this shape before this fix; if this risk materializes it is a separate, pre-existing catalog-format question, not a regression this FR introduced |
| This FR's `language: n/a` frontmatter value (§0), chosen because the fix is a binary-asset replacement with no source code changed, doesn't match any value a language-aware backlog-tooling script might expect if such a script exists elsewhere in the repo and was never audited as part of this FR's authoring | Would surface only if and when such tooling is run against this FR's frontmatter — not caught by this FR's own verification (§5), which only checks the icon files themselves | A hypothetical downstream tooling failure this FR's authoring cannot rule out without auditing every consumer of FR frontmatter across the repo, which is out of this FR's own scope | If this ever surfaces, the fix is a frontmatter-only correction to this FR's `language` field, not a change to the icon fix itself — recorded here as a known, low-probability, easily-correctable edge case rather than a blocking concern |

## §11 — Implementation notes

- **This FR documents a fix already applied to the working tree**, not a fix yet to be written — its purpose is to make that fix reviewable, testable, and regression-guarded rather than an unreviewed direct edit with no root-cause record. The remaining implementation work (§0 `sub_tasks`) is committing it and closing §6's open regression-guard decision, not writing new code.
- **The byte-identical hash match between `apps/desktop/src-tauri/icons/{android,ios}/` and the fixed `apps/web/{android,ios}/` files is not incidental** — it is the actual mechanism of the fix (a verbatim copy), and is restated in §5/§8 specifically so a reviewer doesn't need to re-derive that fact from first principles.
- **No AI image generation, re-export, or any lossy transformation was used anywhere in this fix** — consistent with this batch's anti-fabrication discipline, an already-correct, already-approved asset was reused verbatim rather than regenerated, eliminating any risk of a visually-different result from what the desktop build already ships.
- **This fix directly unblocks task #6** (adding testers to TestFlight/Play Console internal testing, already in progress as of this FR's authoring) — a placeholder-icon build would have been a poor first impression for the very testers that task is inviting, even before any public release consideration.
- **If Option A (documentation-only guard, §6) is chosen**, the specific insertion point is the mobile-release one-time-setup section already referenced by `capacitor.config.ts`'s own header comment (`cd apps/web`, `npm i -D @capacitor/core ...`, `npx cap add ios && npx cap add android`) — appending the icon-recopy instruction immediately after that block keeps the one-time-setup instructions co-located rather than scattered across the docs tree.
- **This FR's tracker entry corresponds to task #16** ("Fix Capacitor mobile app icon (iOS + Android placeholder)") in the batch task list this session has maintained alongside FR-APP-003 through FR-APP-006 (tasks #12–#15); the two remaining task-list items this FR does not itself resolve — #6 (tester onboarding) and #12 (Mac App Store submission) — are cross-referenced in §7 and §9 respectively, but neither is blocked by this FR at the tooling level, only benefited by it landing first.
- **This FR's `effort_hours: 2` estimate reflects work already substantially performed**, not work yet to be scoped from scratch: 0.5h root-cause confirmation (done), 0.5h fix application (done), 0.5h commit-and-review (Stephen-gated, not yet done), 0.5h regression-guard implementation once §6's Option A/B decision is made (not yet done) — a materially different effort profile from FR-APP-003 through FR-APP-006's `effort_hours` estimates, which reflect from-scratch design and CI-integration work rather than a fix already sitting in the working tree.
- **This agent has no image-rendering capability**, which is why AC #3's visual sanity check is explicitly delegated to Stephen throughout this FR (§4, §7) rather than claimed as something this authoring pass already confirmed — every claim in this document about the icon files' *content* is grounded in hash comparison (§5, §8) and file-shape metadata (dimensions, format, byte size), never in an actual visual inspection this agent performed itself.
- **This FR's self-audit (sibling `audit.md`) is written after this document reaches its final form**, per the master rule's author-then-audit-then-revise-to-10/10 sequence already applied to every other FR in this batch — the findings recorded there reflect gaps identified during that structured review pass, not gaps this document was already known to have while still being drafted.
- **This is the fifth and final FR in this session's batch**, following FR-APP-003, FR-APP-004, FR-APP-005, and FR-APP-006 in authoring order — once this document and its sibling `audit.md` both reach `score_post_revision: 10/10`, the remaining batch-closing steps are the backlog insert-rows for all five FRs and a final summary report, not further FR authoring.
- **`class: improvement` and `module: improvement` (§0) intentionally match**, consistent with §2's rationale for placing this fix in `docs/feature-requests/improvement/` rather than `docs/feature-requests/app/` alongside FR-APP-003 through FR-APP-006.
- **`status: ready_to_implement` is set from the start of this FR's authoring**, not flipped from `draft` partway through as with FR-APP-006 — this reflects that the underlying fix was already applied and hash-verified before this document existed, so there was never a period during authoring where the fix itself was still undecided.

*End of FR-IMP-073.*
