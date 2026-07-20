---
id: TASK-IMP-081
title: "CI leg rebuilds + recommits apps/console/web on real source changes - structural follow-up to TASK-IMP-080's served-bundle version-drift fix"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-13T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: improvement
priority: p1
status: done
verify: T
phase: "Wave 6 - go-live (web channel)"
owner: Stephen Cheng (CTO)
created: 2026-07-13
shipped: 2026-07-13
memory_chain_hash: null
related_tasks: [TASK-IMP-080, TASK-IMP-071, TASK-IMP-068, TASK-IMP-079]
depends_on: [TASK-IMP-080]
blocks: []
source_pages:
  - "docs/tasks/improvement/TASK-IMP-080-served-bundle-version-drift/spec.md §9: 'Structural follow-up: a CI leg that rebuilds apps/web on apps/web/**/VERSION changes and ships it like the docs job (ship.sh pattern), retiring the tracked-output model. Decide together with the deploy.sh caddy-bind implications; not blocking 1.0.0.'"
  - "apps/web/scripts/stamp-sw.mjs lines 32-34: 'the badge on os.cyberskill.world sat at 1.2.0 while the platform shipped 1.7.0' - the exact incident class this closes for good"
  - ".github/workflows/deploy.yml line 6-8: 'A non-services push (apps/web, console, compose) skips the gate + build and just rolls (the bundle ships via the VPS git pull)' - confirms no CI leg currently rebuilds apps/console/web"
  - ".github/workflows/version.yml: the proven bot-commit-back pattern (cyberos-bot identity, VERSION_BUMP_SSH_KEY deploy-key ruleset bypass, graceful degrade-to-warning on blocked push) this task reuses rather than inventing a second one"
source_decisions:
  - "TASK-IMP-080 §9 (2026-07-13): explicitly deferred the structural fix as not blocking 1.0.0, to be decided together with the deploy.sh caddy-bind implications"
  - "2026-07-13 (this task): Option A (bot commit-back, mirroring version.yml) chosen over Option B (gitignore apps/console/web + ship it like the docs job's ship.sh rsync pattern). Option B is architecturally cleaner and matches §9's own suggested phrasing, but the one-time migration (untracking a directory the VPS currently git-pulls into a live-served path) carries a real outage window: the VPS's next `git pull` after the untracking commit would delete the tracked files before any new ship step could repopulate them, and nothing in this session can rehearse that sequencing against the real VPS. Option A is a small, additive diff with no migration step and no change to deploy.sh's documented caddy directory-bind behavior - the exact trade TASK-IMP-080 §9 asked to make deliberately rather than defer forever."
language: yaml (GitHub Actions)
service: .github/workflows/deploy.yml
new_files: []
modified_files:
  - .github/workflows/deploy.yml
effort_hours: 1
subtasks:
  - "Extend the `changes` job's dorny/paths-filter with a `web_src` output (apps/web/**, VERSION) alongside the existing `services` output"
  - "New `rebuild-web` job: gated on `needs.changes.outputs.web_src == 'true'`; checkout with the same VERSION_BUMP_SSH_KEY deploy key as version.yml; `cd apps/web && npm ci && npm run build`; if `apps/console/web` diffs, commit as cyberos-bot with `[skip ci]` and push, degrading to a GITHUB_STEP_SUMMARY warning (not a failure) if the branch ruleset blocks the push"
  - "Extend the `deploy` job's `needs:` to include `rebuild-web` so the VPS's `git pull` (deploy.sh) always runs after any bundle-refresh push has landed, not racing ahead of it"
risk_if_skipped: "The exact TASK-IMP-080 incident recurs on every future version bump that nobody manually rebuilds for: the served badge silently lags the platform VERSION again, and (per TASK-IMP-080 §1) only the pre-commit/payload-gate/version.yml/release check-version-sync.sh gate catches it - and only if someone runs one of those paths locally before the next deploy, which is exactly the manual step that was skipped the first time."
---
## §1
1. The `changes` job (`.github/workflows/deploy.yml`) **MUST** gain a `web_src` paths-filter output covering `apps/web/**` and `VERSION`, alongside the existing `services` output - so a source or VERSION change to the web app is machine-detectable the same way a services change already is.
2. A new `rebuild-web` job **MUST** run only when `web_src == 'true'` (never unconditionally): `stamp-sw.mjs`'s cache id is `new Date().toISOString()`-derived, not content-hash-derived, so an ungated rebuild would produce a spurious diff - and a spurious bot commit - on every single deploy.yml trigger, including ones that never touched the web app.
3. When gated in, the job **MUST** rebuild `apps/web` (`npm ci && npm run build`, identical to the command TASK-IMP-080 §1 already prints as the manual rebuild instruction) and, if `apps/console/web` differs from the committed copy, commit it back to `main` as `cyberos-bot <bot@cyberskill.world>` and push using the same `VERSION_BUMP_SSH_KEY` deploy key `version.yml` already uses to bypass the branch ruleset - no new secret, no new bot identity.
4. The commit message **MUST** carry `[skip ci]`: unlike `version.yml`'s bump commit (which must stay visible to tag-triggered workflows per TASK-IMP-071), this commit only refreshes a served artifact and has no reason to re-trigger deploy.yml a second time for itself.
5. A push blocked by the branch ruleset **MUST** degrade to a `GITHUB_STEP_SUMMARY` warning, exactly like `version.yml`'s existing degrade path - never a hard job failure, so a ruleset misconfiguration cannot turn a client-only push red.
6. The `deploy` job's `needs:` **MUST** include `rebuild-web`, so `deploy.sh`'s VPS-side `git pull` always runs after any bundle-refresh push from this job has landed on `main` - closing the ordering gap, not just the detection gap.
7. `rebuild-web` **MUST** be `continue-on-error: true`. It is additive to the services roll, not a gate on it: `apps/web` and `services/` are independent deployables, and a real build failure in this job (not just a blocked push) must not trip `deploy`'s shared `needs:`/`failure()` check and block an otherwise-healthy services roll for an unrelated breakage. The job still shows red in the Actions UI on a genuine failure - only its blast radius is contained.

*Lean profile: one workflow file, one new job, one extended paths-filter output, one extended `needs:` list; the commit-back mechanics are a direct reuse of `version.yml`'s already-proven pattern rather than a new design.*

## §2 — Why this shape (Option A over Option B)
TASK-IMP-080 §9 named two possible shapes: (A) a CI leg that rebuilds and commits the bundle back, or (B) retiring the tracked-output model entirely - gitignoring `apps/console/web` and shipping it via a new job mirroring the `docs` job's `ship.sh` rsync pattern (which already gitignores `apps/console/docs` for exactly this reason).

Option B is the architecturally cleaner end state and is what §9's own phrasing leans toward ("ships it like the docs job... retiring the tracked-output model"). It was not chosen here because it is a *migration*, not just a new job: `apps/console/web` is currently a git-pull-tracked directory that Caddy serves via the directory bind `deploy/vps/deploy.sh` documents. The commit that adds it to `.gitignore` would, on the VPS's very next `git pull`, remove those tracked files from the working tree - before any new `ship.sh`-style step could repopulate them - opening a live-site outage window with no way to rehearse or time the sequencing against the real VPS from this session. TASK-IMP-080 §9 explicitly flagged the deploy.sh caddy-bind implications as something to "decide together," not something to force through unrehearsed to close out a SHOULD-priority follow-up.

Option A carries none of that risk: it is a strictly additive job that reuses `version.yml`'s already-battle-tested commit-back mechanics (same bot identity, same deploy key, same graceful-degrade-on-blocked-push behavior) and changes nothing about how `apps/console/web` is tracked or served. It converts TASK-IMP-080's fix from "the next commit fails loudly if someone forgets to rebuild" into "nothing needs to remember" - the same posture upgrade `check-version-sync.sh` already gave every other artifact, now closing the one gap that check can only detect, not prevent.

Sequencing note: because `version.yml`'s bump commit and this job's own trigger both watch `VERSION`, a push that changes both `apps/web/**` source and lands moments before a version bump can produce two `rebuild-web` runs in quick succession - one against the pre-bump VERSION, one (triggered by the bump commit itself, since `VERSION` is in the filter) against the correct post-bump VERSION. This is self-correcting by construction: the second run's diff against the first run's (stale-version) commit is non-empty, so it supersedes it with the correct state. No new race class is introduced beyond the one `version.yml` and `deploy.yml` already share today (a bump commit already re-triggers a second `deploy` roll); this job's `[skip ci]` commit message additionally prevents it from re-triggering yet another full workflow run for itself.

## §5 (run 2026-07-13)
- `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/deploy.yml'))"`: parses clean. Job graph confirmed: `changes -> {gate -> build-images, rebuild-web} -> deploy`, `changes.outputs` carries both `services` and the new `web_src`, `deploy.needs` now `[changes, build-images, rebuild-web]`. PASS
- `actionlint v1.7.12` (downloaded fresh, network egress available in-session) against `.github/workflows/deploy.yml`: zero findings (expression syntax, job-dependency graph, and embedded-shellcheck all clean). Exit 0. PASS
- Dry-run of the new commit-back logic in an isolated scratch git repo (not the real remote; no push target configured, so any push attempt necessarily fails - exercising the exact degrade path):
- No-diff branch: `git diff --quiet -- apps/console/web` correctly short-circuits to the "nothing to commit" message and `exit 0` without staging or committing anything. PASS
- Diff branch: staged + committed as `cyberos-bot <bot@cyberskill.world>` with message `chore(web): rebuild served bundle @ 1.0.0 [skip ci]` (exact format); `git status --short` clean immediately after. PASS
- Push-failure branch: `git push origin HEAD:main` against a remote-less scratch repo fails as expected, and the `if/else` correctly falls through to the `::warning::` + `GITHUB_STEP_SUMMARY` degrade path under `set -euo pipefail` without killing the script - confirming this mirrors `version.yml`'s existing graceful-degrade behavior rather than failing the job red. PASS
- Not exercised (cannot be, from this sandbox): an actual push through the real `VERSION_BUMP_SSH_KEY` deploy key against the live branch ruleset, and a real `npm ci && npm run build` run inside this specific job's checkout. Both use commands already proven elsewhere in this repo's own CI (this exact rebuild command is what TASK-IMP-080 §1 ran and verified green this same day; the deploy-key push pattern is what `version.yml` already runs on every bump). Flagging this gap explicitly for the human review gate rather than claiming an end-to-end run that did not happen.

### Testing pass (2026-07-13, post gate-1 "go")
Stephen approved gate 1 (human review) in chat. Re-ran the machine-checkable verification set unchanged since review, to confirm nothing drifted between review and test:
- `actionlint v1.7.12` against `.github/workflows/deploy.yml`: exit 0, zero findings (unchanged from the review-time run). PASS
- `python3 -c "import yaml; yaml.safe_load(...)"` re-parse: clean. Re-asserted programmatically: `rebuild-web.continue-on-error == true`, `rebuild-web.needs == "changes"`, `deploy.needs == [changes, build-images, rebuild-web]`, `changes.outputs.web_src` present. All PASS.
- No code changes were made between the reviewing-gate verdict and this testing pass, so the scratch-repo dry-run results recorded above (no-diff short-circuit, diff-branch commit-and-push, push-failure degrade-to-warning) still stand as-is; not re-run since nothing they exercise changed.
- Same "not exercised" gap as review time still applies and is still unclosed from this sandbox: no real push through the live deploy key, no real `npm ci && npm run build` inside this job's actual checkout. This task proceeds to the human acceptance gate with that gap disclosed, not silently.

### Acceptance (2026-07-13)
Stephen approved gate 2 (human acceptance) in chat: "i approve". TASK-IMP-081 lands as `done`, `shipped: 2026-07-13`. The two sandbox-inherent gaps flagged above (live-deploy-key push, real build inside the actual CI checkout) remain unexercised — they close naturally on the first real `deploy.yml` run this job is present for, not before.

## §9
- If `apps/console/web`'s serving model is ever revisited (Option B above), this job becomes dead weight to remove, not a blocker: nothing about it constrains that future migration, and the paths-filter/`needs:` wiring this task adds is straightforward to delete in the same change that gitignores the directory.
- Not covered: a Capacitor `cap sync` step. `apps/web/android/**` and `apps/web/ios/**` are release.yml's concern (mobile shell sync on tag), not deploy.yml's; this job only ever touches `apps/console/web`.

## §10
| Failure | Detection | Recovery |
|---|---|---|
| `apps/web/**` changes but the branch ruleset blocks the bot's push | `GITHUB_STEP_SUMMARY` warning on the run, same as `version.yml`'s existing degrade path | add/rotate the deploy key in the ruleset bypass list (docs/deploy/RELEASE.md), or rebuild manually per TASK-IMP-080's printed command |
| `apps/web` genuinely fails to build (real `tsc`/`vite` error on main, not a blocked push) | `rebuild-web` job shows red in the Actions UI (`continue-on-error: true` does not hide the failure, only its blast radius) | fix the web-app source; `continue-on-error` deliberately keeps this from blocking the unrelated services roll in the same run |
| two `rebuild-web` runs race a version bump | second run's diff is non-empty against the first (pre-bump) commit and supersedes it | none needed - self-correcting |
| `deploy` job's VPS `git pull` races ahead of this job's push | `needs: [..., rebuild-web]` on the `deploy` job forces ordering | none needed by design |
| this job's own commit somehow lacks `[skip ci]` in a future edit | it would re-trigger a redundant (but idempotent, harmless) deploy.yml run - not a correctness bug, just wasted CI minutes | restore `[skip ci]` in the commit message |
*End of TASK-IMP-081.*
