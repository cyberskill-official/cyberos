# Changelog — CyberOS

This is the repo-level changelog for CyberOS. For module-specific changelogs, see the per-module pages on the documentation site.

## [Unreleased]

Breaking
- `run-gates.sh` now exits RED (code 3, distinct from 1 = a gate failed and 2 = missing/malformed config) when ZERO floor gate commands (build, lint, test, coverage) are configured — the old floor-only green verified nothing and lied to both downstream human gates. Consumer repos that relied on RED-on-empty's predecessor (vacuous green) must configure `gates.*` in `.cyberos/config.yaml`, re-run install (autodetect gained a monorepo fallback tier that seeds `bash scripts/tests/run_all.sh` or a Makefile `test:` target, provenance `fallback:*`), or — for intentionally gate-less repos — export `CYBEROS_ALLOW_EMPTY_GATES=1` (the literal `1`), which prints a distinct `GATES: EMPTY-ACKNOWLEDGED` line instead of green. (TASK-CUO-302)
- `backlog-mutate.mjs flip` now REFUSES the two human-acceptance gate transitions (`reviewing -> ready_to_test`, `testing -> done`) with exit code 8 unless a recorded human verdict accompanies the flip (`--verdict-by <actor>` + `--verdict-evidence <existing non-empty file>`) — breaking for tooling that automates those two bare flips (STATUS-REFERENCE §1.4; TASK-CUO-303). On a gated flip with a resolvable BRAIN store, one `status_overridden` audit row is appended before the index moves; a present store that cannot take the row fails the flip (exit 9).
- the payload's npm `engines` floor is raised from `node >=18` to `>=24 <25`, matching every `.nvmrc` (24.18.0) and the shipped mcp subpackage — the payload admitted a Node nobody tests. Node-18 consumers now get a clear npm engines error; adopt 24.18.0. (TASK-IMP-137)
- the MCP server's `--http` mode now binds `127.0.0.1` by default instead of every interface — the served tools rewrite repos and run shell commands, and the LAN was reachable with zero auth. Agent UIs that connected from another host must run the server on that host or opt in deliberately via the new `--host <addr>` flag. (TASK-IMP-137)

Changed
- `cyberos-cuo drain --halt-on-repeat-rework` default 2 → 3 (`modules/cuo` api.py + cli.py, and the `cyberos workflow` wrapper in `modules/memory`), matching the ship-tasks.md §11b route-back ceiling: default drains now permit the third cycle before halting (TASK-CUO-304).
- four NFR stub skills (`nfr-certification-author`, `nfr-evaluator`, `nfr-test-runner`, `nfr-regression-handler`) are delisted from the vendored payload (superseding TASK-CUO-209's vendoring decision); they remain as unvendored scaffolds under `modules/skill/`. Injection-discipline (`untrusted_inputs` + `references/UNTRUSTED_CONTENT.md`) backported to 20 repo-reading skills; pair-parity SCOPE expanded 11 → 25. (TASK-SKILL-202)
- corpus hygiene (mechanical half): 251 task specs had `module:` values lowercased to match their `docs/tasks/<module>/` folder; task-lint gains FM-117 (lowercase + folder match); 12 reconcile dossiers prepared for the stuck-`implementing` triage. UNREVIEWED fork + Gate-2 verdicts remain operator-gated (pending). (TASK-IMP-139)

Added
- `memory-append.mjs` accepts the `status_overridden` kind with validated payload `{actor, task_id, prior_status, new_status, reason}` (TASK-CUO-303).
- `modules/cuo/tests/test_doctrine_constants.py` pins the Python route-back-ceiling defaults to the number parsed from ship-tasks.md §11b (TASK-CUO-304).
- optional bearer-token auth on the MCP HTTP transport: set `CYBEROS_MCP_TOKEN` non-empty and every `POST /mcp` must carry `Authorization: Bearer <token>` (401 with a JSON-RPC error body otherwise); `GET /healthz` stays open for probes; binding non-loopback without a token warns loudly; an empty token is treated as unset. (TASK-IMP-137)
- `install.sh` vendors the GitHub Action channel to `.cyberos/ci/github-action/` — the README documented that path while install never created it; the docs now show a working `uses: ./.cyberos/ci/github-action` example. (TASK-IMP-137)
- run-gates gains a presence-gated `doctor` gate: when `.cyberos/memory/store/` exists and the cyberos memory module is importable, `python3 -m cyberos doctor` joins the machine floor (doctor FAIL = gate RED); repos without memory see exactly one SKIP provenance line and no behavior change. (TASK-MEMORY-303 §1.6, implemented with the gates hardening batch)
- root CI gate `caf-evals-gate.yml`: the CAF eval suite (`validate.py --all`, all 40 fixtures) + `caf_precommit_check.sh` now run on PRs touching `tools/caf/**` / `scripts/caf_*` and on a weekly cron — previously the only workflow naming them sat nested under `tools/caf/.github/` where GitHub Actions never reads, so the suite ran in no CI at all. The same workflow's second job runs the TASK-IMP-140 benchmark-gate checkers. (TASK-IMP-136)
- `.githooks/pre-commit` now runs the awh module gate (`.pre-commit-hooks/awh-gate.sh`) for staged `modules/` sources via the hook's matches() idiom; a missing awh harness (or missing gate script) warns and never blocks, a RED gate blocks the commit. (TASK-IMP-136)
- `docs/verification/benchmark-gates.md` — the sixteen benchmark gates (G1–G16) from the 2026-07-23 deep audit, published as re-checkable pass/fail criteria with severities, tiers, checked files, and one owning checker per gate; the status table is the report-only/enforcing coordination surface. (TASK-IMP-140)
- `scripts/tests/test_benchmark_gates` suite — automated checkers for the six unowned gates: G3 status-enum cross-check, G4 README headline-count truth, G5 payload reference walker, G6 vendored-gate executability smoke, G13 stuck-WIP detector (report-only, never mutates), G16 reinstall idempotency + config survival. Registered via the run_all glob and the caf-evals-gate CI workflow. (TASK-IMP-140)
- `docs/reference/risk-register.md` gains R-EXT-01..07 — the audit's risk classes (self-approval, vacuous green gates, config wipe, prompt injection, payload divergence, partial-install window, frozen BRAIN), each with detection, preventing gate, and recovery. (TASK-IMP-140)
- BRAIN recording of the audit verdict + gates + wave decisions: prepared as a guarded script (`brain-record.sh`, refuses below READY) and executed after TASK-MEMORY-303's store repair per the spec's depends_on edge. (TASK-IMP-140)
- memory contract hardening: schema copies unified (StoreAcl-bearing), `INTEROP.md` (≤6k chars), walker allowlist + invariants for `sessions/`+`dreams/`, `extra.session_id` stamping, doctor gate wiring. Live store layout repair remains operator-gated. (TASK-MEMORY-303)

Removed
- `.pre-commit-config.yaml` — dead mechanism: the repo's hook path is `core.hooksPath=.githooks` and no tool read the framework config; its every live claim (payload build, docs build, awh gate) is covered by `.githooks/pre-commit` directly. (TASK-IMP-136)
- the 9 always-green stub workflows (single-echo placeholders, auto-generated 2026-05-17): 9 deleted, 0 implemented — an always-green check manufactures false confidence under a gate-shaped name. Per-file disposition + declaring tasks in `docs/tasks/improvement/TASK-IMP-136-ci-caf-evals-and-stub-truth/stub-disposition.md`; `test_ci_truth.sh` fails the moment the placeholder marker regrows. (TASK-IMP-136)

Fixed
- `bootstrap.sh` checksum verification now works on stock macOS: it falls back from GNU `sha256sum` to `shasum -a 256` — the fallback VERIFIES (a corrupted payload still aborts) — and aborts naming both tools when neither exists. (TASK-IMP-137)
- the generated `gates.env` header stopped saying "edit freely": the file is machine-owned and regenerated on every install; durable overrides belong in `.cyberos/config.yaml` (`gates.*` keys). (TASK-CUO-302)
- `install.sh` replaces each vendored machine subtree via stage-then-swap (staged `<name>.tmp.<nonce>` copy, then two rename-class moves), closing the window where a reader of `.cyberos/` saw a missing or partial tree for the whole copy duration; stray staging dirs from killed installs are cleaned at the next install start. The payload's `memory.schema.json` also now vendors from the canonical package-data copy (`modules/memory/cyberos/data/`). (TASK-IMP-137, TASK-MEMORY-303)

## [1.1.0] - 2026-07-22

Breaking
- renamed the published npm CLI's bin command from `cyberos` to `cs` (`npx cs <command>` in place of `npx cyberos <command>`) — the old name collided on `$PATH` with an unrelated, internal-only `modules/memory` console script also named `cyberos`. The npm package name is unchanged (`@cyberskill/cyberos`); only the invoked command renamed. There is no `cyberos` alias during a transition window — update any script or muscle memory calling `npx cyberos ...` to `npx cs ...`. (TASK-IMP-130)

Added
- `cs memory <args>` — dispatches to a locally installed `cyberos-memory` via `python3 -m cyberos` (never a bare `$PATH` `cyberos` lookup); exits 2 with a clear message when the package is not available. The Python package is not bundled with the npm install. (TASK-IMP-131)
- `cs cuo <name>` — redirect stub that prints the matching Claude Code slash command (`/plan`, `/create-tasks`, `/ship-tasks`, `/improve`); no subprocess execution. (TASK-IMP-132)

Fixed
- CLI help / docs surfaces that still pointed at `https://cyberos.cyberskill.world/docs` now use `https://os.cyberskill.world/docs`. (TASK-IMP-130)

Distribution
- `@cyberskill/cyberos@1.1.0` publishes `bin.cs` → `cli/bin/cli.mjs` (TASK-IMP-135). Homebrew formula `cyberos-cli` installs the `cs` binary pinned to this release (TASK-IMP-133). Offline e2e covers the rename composition (TASK-IMP-134).

## [1.0.9] - 2026-07-22

Maintenance release.

## [1.0.8] - 2026-07-21

Fixed
- release-mas

## [1.0.7] - 2026-07-20

Fixed
- stop the message action bar covering text on touch devices

## [1.0.6] - 2026-07-20

Fixed
- separate the sign-in footer links

## [1.0.5] - 2026-07-20

Fixed
- let users choose which Google account to sign in with

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
