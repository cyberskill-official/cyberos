---
id: TASK-IMP-127
title: Payload build MUST read from git, not the working tree
template: task@1
type: improvement
module: improvement
status: draft
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-20T00:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-074, TASK-IMP-122]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-20
memory_chain_hash: null
effort_hours: 4
service: tools/install
new_files:
  - (none)
modified_files:
  - tools/install/build.sh
  - tools/install/tests/test_release_assets.sh
source_pages:
  - "tools/install/build.sh:376 (rules_sha = find cuo plugin mcp cli memory -type f | LC_ALL=C sort | hash - walks the OUTPUT tree, whatever was copied into it)"
  - "tools/install/build.sh:28-74 (payload assembled with cp/cp -R from $repo/modules/... - the working tree, never consulting git)"
  - ".gitignore:15 (.DS_Store is gitignored and was still copied into the payload)"
  - "measured 2026-07-20: local build of 069d4dff -> rules_sha a9c848e5; CI build of the same commit -> 8745b0fd"
  - "measured 2026-07-20: diff of the two payloads = 6 files present only locally (cuo/gates/caf/caf/.DS_Store, cuo/gates/caf/caf/core/evals/code_audit_validator.egg-info/{PKG-INFO,SOURCES.txt,dependency_links.txt,entry_points.txt,top_level.txt}); zero content differences on any shared path"
source_decisions:
  - "2026-07-20 Stephen: PLAN gate - author as three separate tasks (build / CI / config durability); approved."
---

# TASK-IMP-127: Payload build MUST read from git, not the working tree

## Summary

`build.sh` assembles the payload by copying from the working tree, so any untracked or gitignored file sitting in `modules/` ships in the payload and changes `rules_sha`. The same commit therefore produces different fingerprints on different machines, which defeats the drift detection `rules_sha` exists to provide. Drive the payload copy from git so the payload is by construction the tracked content at that commit, and fail the build when the module tree is dirty.

## Problem

`rules_sha` is defined by TASK-IMP-074 as a content fingerprint over the distributed rule trees, and `update-check.sh` reports RULE DRIFT when an installed machine's fingerprint differs from the payload's at the same version. That signal is only meaningful if the fingerprint is a function of the commit.

It is not. `build.sh:28-74` assembles the payload with `cp` / `cp -R` from `$repo/modules/...`, never consulting git, and `build.sh:376` computes `rules_sha` by walking the resulting output tree. Anything a developer happens to have in `modules/` is copied in and hashed.

Measured on 2026-07-20 against `069d4dff`: a local build produced `a9c848e5`, CI produced `8745b0fd`. The diff is six files, all present locally and absent in CI, all inside the fingerprinted `cuo` tree - one `.DS_Store` (matched by `.gitignore:15`, so gitignored files are not excluded either) and five files of a `code_audit_validator.egg-info/` directory left by a local editable install. Zero content differences on any shared path: the payloads are functionally identical and only the fingerprint diverges.

The consequence is a false positive that is indistinguishable from a true one. A fingerprint that varies with untracked local state cannot tell "the rules changed" from "someone opened a folder in Finder", so RULE DRIFT stops being evidence of anything. It is not theoretical - installing a locally built payload across the estate on 2026-07-20 put 20 repos on `a9c848e5`, every one of which would have reported drift against the released payload with no rule change behind it.

## Proposed Solution

Make the tracked content at HEAD the only thing that can enter the payload:

- Drive the payload copy from git rather than the filesystem - `git archive` for whole subtrees, or `git ls-files` to enumerate what may be copied - so untracked and gitignored files cannot be selected at all.
- Fail the build when the module tree carries untracked files, rather than silently absorbing them, so the operator learns at build time instead of at a drift report weeks later.

## Alternatives Considered

- An `--exclude` list for `.DS_Store` and `*.egg-info`. Rejected: it fixes the two extensions observed today and stays silent on the next one. The defect is that the build selects by "what is on disk" instead of "what is committed"; excluding known offenders leaves that selection rule intact.
- Compute `rules_sha` from git object hashes instead of the output tree. Rejected as the primary fix: it would make the fingerprint reproducible while still shipping junk files in the payload. Correct the payload, and the fingerprint follows.
- Have the release job compare its `rules_sha` against a local build and fail on mismatch. Kept as a guardrail (below), not as the fix - it detects the divergence without preventing it.

## Success Metrics

- Primary: a build from a dirty working tree (untracked `.DS_Store` and an `egg-info/` dir planted under `modules/`) produces a payload byte-identical to a build from a clean checkout of the same commit, and therefore the same `rules_sha`. Baseline today: the two differ by six files and the fingerprint changes.
- Guardrail: no tracked file is dropped - the payload built from git contains exactly the file set the current build produces from a clean tree, so the fix cannot silently shrink the payload.

## Scope

In scope: how `build.sh` selects files for the payload, the dirty-tree guard, and arms in `test_release_assets.sh`.

### Out of scope / Non-Goals

- The definition of `rules_sha` itself (TASK-IMP-074) - the set of fingerprinted trees is unchanged.
- The RULE DRIFT message wording in `lib/update-check.sh`.
- Re-vendoring the fleet after the fix (an operator-gated action).
- Removing the `.DS_Store` and `egg-info` files currently in the working tree - housekeeping, and the point of this task is that the build must not care whether they are there.

## Dependencies

None blocking. Depends conceptually on TASK-IMP-074, which defines the fingerprint this task makes reproducible.

**Relationship to TASK-IMP-122 (p1, on_hold).** Both tasks concern `rules_sha` correctness and they are complementary, not overlapping - an implementer MUST NOT treat either as subsuming the other:

- TASK-IMP-122 governs the COMPARISON side: that the installed fingerprint is recomputed rather than read from a stored token, and which paths the cone covers. Its §1.6 Direction 1 fails the build on a path under `$CY` that no cone entry classifies.
- This task governs the PRODUCTION side: which files may enter the payload at all.

122 does not fix this defect. The six contaminating files sit at `cuo/gates/caf/caf/...`, beneath 122's `dir:cuo` cone entry, so they are classified and hashed - Direction 1 passes them cleanly while they still corrupt the fingerprint. Conversely this task does not deliver 122: a payload built from git is reproducible, but a comparator that recalls a stored token still reports a build rather than a tree.

They compose in one direction worth stating: once the payload cannot carry untracked files, 122's recomputation compares two trees that a clean checkout can reproduce, which is the premise 122's §1.10 ("byte-identical across the cone") assumes and cannot currently rely on.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the 2026-07-20 estate sweep. The two fingerprints and the six-file diff were measured directly - a released payload was downloaded from the v1.0.0 assets and diffed against the local `dist/` tree - not inferred. The claim "zero content differences on shared paths" is the output of `diff -rq` over the five fingerprinted directories.
- **Human review:** scope and granularity approved at the 2026-07-20 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `build.sh` MUST select payload content from git-tracked files at the built commit; a file that is untracked or gitignored MUST NOT be able to enter the payload, regardless of its presence in the working tree.
- 1.2 A build of a given commit MUST produce a byte-identical payload, and therefore an identical `rules_sha`, whether or not the working tree carries untracked or gitignored files under `modules/`.
- 1.3 `build.sh` MUST fail with a non-zero exit and name the offending paths when the module tree contains untracked files, rather than absorbing them into the payload.
- 1.4 The git-driven copy MUST NOT drop any tracked file the current working-tree copy produces from a clean checkout - the payload file set is unchanged for a clean build.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.2) - a fixture plants an untracked `.DS_Store` and an untracked `egg-info/` dir under `modules/`, builds, and the payload and `rules_sha` are byte-identical to a build of the same commit from a clean tree - test: `tools/install/tests/test_release_assets.sh::t_build_ignores_untracked`
- [ ] AC 2 (traces_to: #1.3) - a build with untracked files under `modules/` exits non-zero and its stderr names at least one offending path - test: `tools/install/tests/test_release_assets.sh::t_build_fails_on_dirty_module_tree`
- [ ] AC 3 (traces_to: #1.4) - the file list of a git-driven payload from a clean checkout equals the file list the pre-change build produces from the same checkout - test: `tools/install/tests/test_release_assets.sh::t_payload_file_set_unchanged`

## 3. Edge cases

- A gitignored file whose path is also tracked (added before the ignore rule) is tracked and MUST ship - "tracked" is the test, not "unignored".
- A submodule or symlink under `modules/` must be handled the same way the current copy handles it; the selection rule changes, the materialisation does not.
- Building from a detached HEAD or a tag (the release path dispatches against a tag) MUST work - the commit is whatever is checked out, not necessarily a branch tip.
- Building from an archive export with no `.git` present (a consumer building from a tarball) MUST either work or fail with a message naming the missing repository, never silently fall back to the working-tree copy - a silent fallback would reintroduce 1.1.
- The dirty-tree guard MUST scope to the module tree the payload is built from, not the whole repo: an untracked file under `docs/` or `/tmp` is irrelevant and MUST NOT fail the build.
- Security-class: the build reads files and executes nothing from them. Enumerating via git rather than the filesystem narrows the attack surface - an attacker who can drop a file in the working tree can no longer get it into a signed release asset.
