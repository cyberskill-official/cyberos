---
id: TASK-IMP-133
title: "Homebrew tap: update cyberos-cli.rb for the cs rename"
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-22T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-130]
blocks: []
related_tasks: [TASK-IMP-076]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.9"
owner: Stephen Cheng (CTO)
created: 2026-07-22
memory_chain_hash: null
effort_hours: 1
service: homebrew-tap (external — cyberskill-official/homebrew-tap)
new_files:
  - (none)
modified_files:
  - "Formula/cyberos-cli.rb (in cyberskill-official/homebrew-tap — NOT this repo)"
source_pages:
  - "docs/plans/PLAN-cli-module-namespacing-2026-07-22/plan.md §5 Scope item 6 and §7 Risks (Homebrew tap break flagged explicitly, 'will silently start asserting against a binary that no longer exists once the npm package renames, unless task 6 lands in the same window')"
  - "Formula/cyberos-cli.rb:12-32 in cyberskill-official/homebrew-tap, re-fetched fresh 2026-07-22 (shallow clone, HEAD at time of authoring): url pins `cyberos-1.0.9.tgz`, `bin.install_symlink Dir[\"#{libexec}/bin/*\"]` (line 23, picks up whatever bin name the installed npm package declares — needs no change itself), `test do` block (lines 26-32) hardcodes `assert_predicate bin/\"cyberos\", :exist?` and `:executable?`"
  - "Formula/cyberos-cli.rb:1-8 (header comment explains the Formula is named cyberos-cli, distinct from this tap's separate cyberos GUI Cask, and states 'They share the name \"cyberos\" upstream' — this specific sentence becomes inaccurate once the upstream npm bin renames to cs, independent of the test-block fix)"
source_decisions:
  - "2026-07-22 Stephen: create-tasks PLAN gate — APPROVE as rendered, including this task's cross-repo placement (spec lives in cyberos's backlog; implementation targets the separate homebrew-tap repo)."
  - "2026-07-22 authoring: re-cloned homebrew-tap fresh (rather than relying on an earlier read from the same session) to confirm the Formula's current exact content before writing this spec, per the anti-fabrication discipline for a task whose evidence lives outside this repo's own git history."
  - "2026-07-22 authoring: discovered a sequencing dependency beyond what plan.md §5 item 6 states outright: bin.install_symlink alone does NOT make the rename land — the Formula's own `url`/`sha256` are pinned to the CURRENT 1.0.9 tarball (which still ships bin `cyberos`). Updating only the test assertion, without also bumping url/sha256 to a released version that actually contains the `cs` bin, would make `brew test` deterministically fail (it would assert bin/\"cs\" exists against an installed 1.0.9 payload that only has bin/\"cyberos\"). This is now the task's primary normative clause, not an afterthought."
  - "2026-07-22 self-audit revision (score_pre_revision 5/10 -> score_post_revision 10/10): discovered mid-audit that homebrew-tap has NO .github/workflows/ directory at all (confirmed by directory listing) - the first draft's AC 2 cited 'homebrew-tap's own CI workflow' as a test authority that does not exist, a genuine fabricated-citation defect, not a style issue. Revised AC 2 to a local, PR-documented `brew test` run instead. AC 3 originally deferred to 'manual review... not mechanically testable' when a grep for the exact stale sentence plus a positive marker is straightforwardly mechanical - revised to a grep-based check. AC 1 didn't verify the same-commit atomicity clause 1.1 actually demands - added a `git log -p` check. AC 5 was redundant with AC 2 once AC 2 is corrected (a passing local brew test is only possible if the pinned release genuinely exists, which is exactly what clause 1.5 demands) - removed and retraced 1.5 to AC 2 directly. Added an edge case naming that no task in this five-task batch actually covers cutting and publishing the npm release both this task and TASK-IMP-134 depend on - a real plan-level gap, not this task's to close alone, but worth surfacing rather than leaving implicit."
---

# TASK-IMP-133: Homebrew tap: update cyberos-cli.rb for the cs rename

## Summary

Once TASK-IMP-130 ships an npm release with the renamed `cs` bin, the separate `cyberskill-official/homebrew-tap` repo's `Formula/cyberos-cli.rb` needs its pinned version bumped to that release and its `test do` assertion updated from `bin/"cyberos"` to `bin/"cs"` — done together, since bumping one without the other makes `brew test` fail.

## Problem

`Formula/cyberos-cli.rb` (`cyberskill-official/homebrew-tap`, re-fetched fresh 2026-07-22) pins `url "https://registry.npmjs.org/@cyberskill/cyberos/-/cyberos-1.0.9.tgz"` and asserts `bin/"cyberos"` exists and is executable (lines 15-16, 26-32). `bin.install_symlink Dir["#{libexec}/bin/*"]` (line 23) itself needs no code change — it will pick up whatever bin name a newly-pinned npm tarball declares, automatically. But the Formula is pinned to a SPECIFIC already-released tarball (1.0.9), which still ships the old `cyberos` bin (TASK-IMP-130 has not shipped yet as of this task's authoring). Updating only the `test do` block's string to `bin/"cs"` while leaving `url`/`sha256` pointed at 1.0.9 would make `brew test` fail every time — the installed payload would still only contain a `cyberos` bin, and the test would assert for a `cs` bin that isn't there. The plan (§7) already flagged that this Formula "will silently start asserting against a binary that no longer exists," but the specific mechanism — that the fix requires bumping the pinned release, not just editing a string — is a fact this task's authoring surfaced by reading the Formula directly, not something the plan itself stated.

## Proposed Solution

Once an npm release carrying the `cs` bin (TASK-IMP-130) is published, update `Formula/cyberos-cli.rb` in the SAME commit: (1) `url` to that release's tarball URL, (2) `sha256` to that tarball's actual digest (re-derived via the Formula's own documented method, line 11: `curl -sL -o t.tgz "$(npm view @cyberskill/cyberos dist.tarball)" && sha256sum t.tgz`), (3) the `test do` block's two assertions from `bin/"cyberos"` to `bin/"cs"`, and (4) the header comment's "They share the name 'cyberos' upstream" sentence, which becomes inaccurate once the upstream bin is `cs` — reworded to describe the current state accurately. The Formula's own name (`cyberos-cli`, `brew install cyberos-cli`) does NOT change — that identifier disambiguates this Formula from the tap's separate `cyberos` GUI Cask (line 4-8) and is independent of the wrapped CLI's own bin name.

## Alternatives Considered

- Update the Formula's test assertion immediately, ahead of TASK-IMP-130 shipping. Rejected: there would be nothing yet to test against — `brew test` would fail deterministically until a real `cs`-bin release exists, for no benefit over waiting.
- Rename the Formula itself (e.g. to `cs-cli`) to match the new bin name. Rejected: out of scope per the plan, and the Formula's name was already a deliberate disambiguation choice (TASK-APP-006, cited in the file's own header) independent of the wrapped tool's bin name; renaming it would be a second breaking change (`brew install cyberos-cli` stops resolving) layered onto the one this plan already accepts.
- Leave the header comment as-is since it's "just a comment." Rejected: the comment specifically explains a naming decision by asserting a fact (upstream shares the name "cyberos") that this very task makes false; leaving it uncorrected actively misleads the next person who reads it to understand why the Formula is named the way it is.

## Success Metrics

- Primary: after this change, `brew install cyberos-cli` (or `brew test cyberos-cli` in CI) installs a working `cs` binary and the Formula's own test passes. Baseline today: the Formula asserts `bin/"cyberos"`, which will stop existing the moment a `cs`-bin npm release ships, with nothing in this Formula ready for that day.
- Guardrail: `url`/`sha256` and the `test do` assertions are bumped in the SAME commit — never a state where one has moved to the new bin name and the other has not, which is exactly the deterministic-`brew test`-failure window this task exists to avoid ever existing.

## Scope

In scope: `Formula/cyberos-cli.rb`'s `url`, `sha256`, `test do` block, and header comment, in `cyberskill-official/homebrew-tap`.

### Out of scope / Non-Goals

- Renaming the Formula itself away from `cyberos-cli`.
- The tap's separate `cyberos` GUI Cask — a different product, unaffected by this rename (Formula header comment lines 4-6 already establish this distinction).
- `depends_on "node"` or the `install` method's `npm install` invocation — neither references the bin name and neither needs to change.
- Publishing the actual npm release this task depends on — that is TASK-IMP-130's job; this task only reacts to it once it exists.

## Dependencies

Depends on TASK-IMP-130 — specifically, on a PUBLISHED npm release carrying the `cs` bin existing, not merely on TASK-IMP-130's code being merged. This task cannot land (in the sense of passing `brew test`) until that release exists, even though its diff could technically be drafted earlier. This task is a soft (non-status-gating) prerequisite for TASK-IMP-134's manual release-time checklist, which the plan (§6 item 7) states must include "a fresh Homebrew install once the tap is updated" — TASK-IMP-134's own `depends_on` deliberately excludes this task so its fully-automated offline portion isn't blocked on this task's externally-gated completion (see TASK-IMP-134's Dependencies section).

**Cross-repo note.** This task's spec, audit, and backlog row live in `cyberskill-official/cyberos`'s `docs/tasks/` (this repo's `.cyberos/` machine is what generated it), but its actual code change lands in the separate `cyberskill-official/homebrew-tap` repository, which has no `.cyberos/` machine of its own. `/ship-tasks` driving this task will need to operate against a checkout of `homebrew-tap`, not this repo — flagged explicitly since every other task in this batch is a same-repo change and this one is not.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill inside Cowork.
- **Scope:** `Formula/cyberos-cli.rb`'s content was re-fetched via a fresh shallow clone of `cyberskill-official/homebrew-tap` during this authoring session, not carried over from an earlier read in the same conversation without re-verification, per the anti-fabrication discipline for cross-repo evidence.
- **Human review:** task decomposition and cross-repo placement approved at the 2026-07-22 PLAN gate.

## 1. Description (normative)

- 1.1 `Formula/cyberos-cli.rb`'s `url` and `sha256` MUST be updated together, in the same commit, to point at the first published npm release of `@cyberskill/cyberos` that ships the `cs` bin.
- 1.2 In that same commit, the `test do` block's two `bin/"cyberos"` assertions MUST become `bin/"cs"`.
- 1.3 The header comment's sentence asserting the CLI and the Cask "share the name 'cyberos' upstream" MUST be reworded to state the current, accurate naming (the CLI's public bin is `cs`; the GUI Cask remains a separate product named `cyberos`).
- 1.4 The Formula's own name (`cyberos-cli`, i.e. `brew install cyberos-cli`) MUST NOT change.
- 1.5 This task MUST NOT be merged before a real npm release containing the `cs` bin exists — the commit may be prepared in advance, but landing it early would deterministically break `brew test` (traced by AC 2: a passing local `brew test` run is only possible once such a release genuinely exists at the pinned `url`, so AC 2 discharges this clause directly rather than needing a separate procedural check).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - `Formula/cyberos-cli.rb`'s `url` references a version string matching a real published `@cyberskill/cyberos` release that contains a `"cs"` key in that release's `package.json` `bin` field, `sha256` matches that exact tarball's digest, and `git log -p` for the landing commit shows both fields changed together in one commit (not staged across two) - test: manual, ops flow (`homebrew-tap` has no CI workflow at all, confirmed by directory listing, so no automated check can run against the live npm registry from either repo) - the release engineer re-runs the Formula's own documented derivation command (line 11), diffs the result against what's committed, and confirms via `git log` that url+sha256 landed in the same commit as this task's other changes, as part of the release checklist
- [ ] AC 2 (traces_to: #1.2, #1.5) - `brew install --build-from-source cyberos-cli && brew test cyberos-cli`, run locally by the implementer before opening the PR (there is no CI in `homebrew-tap` to run this automatically — confirmed no `.github/workflows/` exists in this repo), passes against the updated Formula - test: documented in the PR description as a local run's output, not inferred from a CI check that does not exist; this same passing run is the evidence clause 1.5 requires (a real release must exist for `brew test` to succeed at all)
- [ ] AC 3 (traces_to: #1.3) - a grep of the header comment block (lines 1-8) for the literal string `They share the name "cyberos" upstream` returns zero matches, and the same block contains the string `` `cs` `` adjacent to a mention of "bin" or "command" - test: `grep -c 'They share the name "cyberos" upstream' Formula/cyberos-cli.rb` returns `0` AND `grep -c '`cs`' Formula/cyberos-cli.rb` returns `>=1`, run as part of the same PR-review pass as AC 2 (still no CI; a reviewer command, not an automated gate)
- [ ] AC 4 (traces_to: #1.4) - the Formula's class name (`CyberosCli`) and filename (`cyberos-cli.rb`) are unchanged in the diff - test: `git diff --stat` for this change shows no rename, only in-place edits to `cyberos-cli.rb`

## 3. Edge cases

- None of the five tasks in this batch (TASK-IMP-130 through 134) has its own acceptance criterion requiring an actual npm release be CUT and PUBLISHED — TASK-IMP-130's own ACs only prove a scratch build's `package.json` has the right `bin` field, not that a release reaches the registry. Cutting a release is an operational step (`scripts/release.sh`, per this repo's own README "Versioning" section) outside any single task's scope, but this task and TASK-IMP-134 both silently depend on it having happened. Named here explicitly rather than assumed, since it is a real gap in the plan's task set (plan §6) that this task's authoring surfaced, not something this task can close on its own - flagged in the batch report for the operator's awareness.
- If TASK-IMP-130 ships but the package NAME also somehow changes (rejected in TASK-IMP-130's own Alternatives Considered, but if a future decision reverses that) - this task's `url` would need to reference the new package name too, not just a new version of the same package; not applicable under TASK-IMP-130's current, approved decision, but named here since this task's own correctness depends on that upstream decision holding.
- A user with an existing `cyberos-cli` Homebrew install upgrades via `brew upgrade`: Homebrew's own symlink-relinking on upgrade is standard Homebrew behaviour, not something this Formula controls beyond declaring the correct bin via `install_symlink` - no special handling needed in this task.
- This task's own spec, audit, and backlog entry exist only in `cyberskill-official/cyberos`'s `docs/tasks/` - if that BACKLOG row is marked `done` before the actual `homebrew-tap` PR merges, the two repos' records would disagree about whether the work shipped. `/ship-tasks`'s human final-acceptance gate is the safeguard here - it MUST NOT be granted until the `homebrew-tap` PR is confirmed merged, not merely opened.
- Security-class: this task edits `sha256` and `url` fields that Homebrew uses to verify tarball integrity before install - an incorrect (rather than merely outdated) `sha256` would cause `brew install` to fail closed (checksum mismatch), not install something unverified, so the failure mode of a mistake here is safe (loud failure) rather than silent (installing unverified content).
