# Changelog — CyberOS

This is the repo-level changelog for CyberOS. For module-specific changelogs, see the per-module pages on the documentation site.

## [1.0.4] - 2026-07-20

Fixed
- make Google sign-in work in the native shells

## [1.0.3] - 2026-07-20

Fixed
- rebuild the served bundle inside the version bump
- guard the android and ios jobs on tag == VERSION
- correct mobile layout on notched viewports

## [1.0.2] - 2026-07-20

Maintenance release.

## [1.0.0] - 2026-07-14

The first stable release of CyberOS - the deliberate 1.0.0 call. The 0.x line hardened the platform's governed development machinery end to end; 1.0.0 commits to it.

Added
- The 1.0 commitment: engineering-spec@1 and the 10-state task lifecycle are stable contracts; the /create-tasks and /ship-tasks workflows are hardened for multi-repo production use (resumable ship manifests, deterministic queue selection, gate autodetect across 9 stacks, per-repo config, audited backlog writes, chain/pair/anchor/version gates in CI).
- Visual deliverables: every task renders to its own CDS-styled page; one status hub (status-hub@2) with three lenses (board / table / releases) regenerates from task frontmatter + CHANGELOG + VERSION.
- Consumer CLI (final surface): `install` | `uninstall` | `version` | `status` | `help`, with matching Claude plugin slash commands `/install` `/uninstall` `/version` `/status` `/help`.
- Soft update-check on any `.cyberos` use; manual check is `version` (if stale and the user confirms, re-vendor via `install` only - no separate apply path).
- `status` opens `docs/status/index.html` in the default browser.
- Root `AGENTS.md` is a thin pointer to `.cyberos/AGENT-ENTRY.md` (same idea as `CLAUDE.md` / `GEMINI.md`); the Layer-1 memory protocol lives only at `.cyberos/memory/AGENTS.md`.
- GitHub release notes document each asset (payload vs plugin vs desktop installers vs signatures).

Changed
- Version bumps now carry the whole codebase (installers, store projects, manifests) and fire the release + docs pipelines natively - no [skip ci], no manual dispatch (TASK-IMP-071/072).
- The status page is ONE page (status-hub@2): Roadmap, Backlog and Changelog stopped being three tabs and became three lenses - board, table, releases - over one filtered task corpus, with a drawer carrying each task's full spec (lazy per-task chunks), relationship graph and metadata. The releases lens is generated from this CHANGELOG (task chips for cited ids + shipped-date matches). Extends TASK-DOCS-006 / TASK-DOCS-007 and the auto-sync of TASK-IMP-074.
- Renamed consumer entrypoints for 1.0.0: install → install, changelog → status (open page), update → version (check-only). No user-facing `install --page` / `--check`; page regen is internal (`lib/status-page.sh` for hooks and run-gates).
- Install/migration hardened by a 23-repo fleet roll: protocol dumps and protocol-symlinks at root AGENTS.md are replaced with the thin pointer; status-page freshness is proven by re-render + byte-compare. tools/install/{fleet-install-test,audit-fleet}.sh roll and PROVE the fleet.

Hardening - pre-1.0.0 improvement batches (2026-07-16 .. 2026-07-19)
- Determinism and provenance: the status stamp is a byte-stable corpus fingerprint instead of a commit sha that chased HEAD forever (TASK-IMP-082); regen_backlog emits every status and recomputes Totals from frontmatter truth, halting before any write on unparseable frontmatter (TASK-IMP-091); backlog headers retally from rows so no wrong baseline can propagate, and acceptance claims are measured on committed objects (TASK-IMP-092, TASK-IMP-116).
- Install and uninstall correctness: the status hook lands where core.hooksPath points (TASK-IMP-083); shared skills dir plus Devin/Windsurf pointers (TASK-IMP-094); gates.env no longer silently clobbered (TASK-IMP-095); non-git installs say so (TASK-IMP-096); an install concurrency lock with tri-state liveness and an owner-byte stamp (TASK-IMP-103); a version guard so an old payload cannot silently downgrade a repo (TASK-IMP-104); and uninstall now leaves the repo as it was found - MCP registration, dangling skill links, byte-exact hook and gitignore strips, marker-gated container removal (TASK-IMP-126, TASK-IMP-121).
- Audit and gate rigour: task-lint as a deterministic machine floor under the task-audit rubric (TASK-IMP-084); per-task coverage scoping (TASK-IMP-098); audits bind the normative half of a spec rather than fields the workflow itself rewrites (TASK-IMP-102); and TRACE-006 requires a cited test's assertion to be at least as strong as its clause's verb (TASK-IMP-118).
- Workflow doctrine: doc-driven ship-manifest and backlog-mutate helpers, dogfooded on their own batch (TASK-IMP-085); task-reconcile, a read-only evidence ladder for work that is already implemented but unaudited (TASK-IMP-100), wired into ship-tasks as a conditional third human gate (TASK-IMP-101); optional draft_reason and entered_via, spec-rejected route-back to draft, and a route-back ceiling that halts at an operator gate (TASK-IMP-108).
- Authoring and templates: consumer installs scaffold task_template: task@1 (TASK-IMP-088); the task@1 template drops its duplicate out-of-scope section (TASK-IMP-089); author manifests default to untracked session state (TASK-IMP-090); backlog index rows 068-081 backfilled to frontmatter truth (TASK-IMP-086).
- Release and reporting: the 1.0.0 release-readiness checklist itself (TASK-IMP-087) and batch economics on the status page, measured and not enforced (TASK-IMP-114).
- Test-suite portability: the repo suite now runs on macOS (bash 3.2 + BSD userland) as well as Linux. Five harness defects were fixed - an unparseable heredoc, GNU-only sed -i in two suites (one of which was a gate that could not fail), a bash-4-only BASHPID, and a logical-vs-physical temp path. No shipped payload code was affected.

## [0.4.0] - 2026-07-12

Added
- TASK-DOCS-007 status hub v2 - dashboard UI on CDS tokens + zero-touch HTML regeneration

Fixed
- remove manual CODE_SIGN_IDENTITY overrides - automatic signing + ASC API key owns identity

## [0.3.0] - 2026-07-12

Added
- TASK-SKILL-120 authoring wiring - Wave D complete (visual deliverables shipped end-to-end)
- TASK-DOCS-006 status hub - deck + Roadmap|Backlog|Changelog tabs, roadmap superseded (6/6 + 7/7 AC)
- TASK-DOCS-005 per-task CDS pages - 491 self-contained deliverable pages, media support, catalog links (6/6 AC)
- TASK-TPL-001 templates module - CDS shells (template@1), vendored tokens+glass, 4/4 AC
- TASK-DOCS-004 folder-per-task layout - 491 tasks migrated, corpus 100% strict-yaml, loud regen
- Wave D batch - visual deliverables (5 tasks audited, ready_to_implement)

## [0.2.0] - 2026-07-12

Added
- TASK-DOCS-003 release roadmap visualization - generated page on every deploy + release
- TASK-SKILL-119 stale-reference sweep - 388 files repointed + doc-anchor checker in CI
- TASK-CUO-208 template profiles - resolution chain, per-file detection, TEMPLATE_PROFILES.md
- TASK-CUO-207 gate autodetect for Go/JVM/.NET/PHP/Ruby + .cyberos/config.yaml per-key overrides
- TASK-SKILL-118 thin-pair contract parity - 86 files across 8 pairs + parity gate
- TASK-CUO-206 ship run-state manifest (ship-manifest@1) - resumable chain + deterministic queue

Fixed
- pin t04B payload version - suite independent of repo VERSION (post-0.1.0-rollback); TASK-SKILL-118 -> reviewing

## [0.1.1] - 2026-07-12

Fixed
- authenticate xcodebuild with the ASC key and sign Release as distribution

## [0.1.0] - 2026-07-12

CyberOS has never completed a store release, so the version now says so. The 1.x line was never shipped to a public store; carrying a 1.x number implied a stability commitment that had not been earned. Versions below run 0.x until the 1.0.0 call is made deliberately.

Changed
- VERSION rolled back to 0.1.0. The 1.x tags and releases are withdrawn.
- Store build numbers are decoupled from the semver and now come from a new monotonic `BUILD_NUMBER` file, seeded at 10701. Google Play permanently remembers every `versionCode` it has accepted (10700, from 1.7.0) and rejects anything at or below the highest it has seen. The old derived formula (`major*10000 + minor*100 + patch`) would have turned 0.1.0 into versionCode 100 and made every future Android upload unshippable, irreversibly. The stamper now hard-fails on any BUILD_NUMBER <= 10700.
- `feat!:` / `BREAKING CHANGE:` no longer auto-declares 1.0.0. While the major is 0, a breaking change bumps the minor - which is also what semver means by 0.x. Reaching 1.0.0 requires an explicit `--set 1.0.0` or a `Release-As: 1.0.0` trailer.

---

Older history: the pre-reset 1.x line (1.0.0 [2026-07-10] through 1.9.1, retired when the version line was reset to 0.1.0 on 2026-07-12) is archived unchanged in [docs/CHANGELOG-legacy.md](docs/CHANGELOG-legacy.md).
