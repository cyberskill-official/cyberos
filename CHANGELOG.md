# Changelog — CyberOS

This is the repo-level changelog for CyberOS. For module-specific changelogs, see the per-module pages on the documentation site.

## [1.8.3] - 2026-07-12

Fixed
- add the mic/camera/photos purpose strings before the first TestFlight build

## [1.8.2] - 2026-07-12

Fixed
- resolver tries the releases/latest redirect before the rate-limited API (live 403 field fix)

## [1.8.1] - 2026-07-12

Fixed
- release-assets.sh honors TAG on workflow_dispatch (release #35 field fix)

## [1.8.0] - 2026-07-12

Added
- remote update awareness - check-latest.sh + three-value --check verdicts (FR-IMP-070)
- publish payload + plugin as GitHub Release assets (FR-IMP-069)
- chain-coverage check + vendor debugging-cycle pair (FR-SKILL-116)
- payload-version drift gate - comparator, CI gate, githooks wiring, fail-fast build (FR-IMP-068)

## [1.7.2] - 2026-07-12

Fixed
- build macOS universal so Intel Macs get an installer at all

## [1.7.1] - 2026-07-12

Fixed
- stop the pipeline reporting success for things it never shipped

## [1.7.0] - 2026-07-11

Added
- UGC controls — reporting, blocking, moderation queue

Fixed
- take the SSO display name from the ID token, not the email

## [1.6.0] - 2026-07-11

Added
- Play store screenshots, captured from the real app

## [1.5.0] - 2026-07-11

Added
- UGC compliance FRs - reporting, blocking, moderation queue + the SSO display-name defect

## [1.4.0] - 2026-07-11

Added
- decouple notarization, stamp VERSION into installers, publish to Play, enable the updater

## [1.3.0] - 2026-07-11

Added
- rename to /create-feature-requests + bundle every chained skill + suggested_prompts
- version parity with CyberOS + /new-fr authoring command

Fixed
- keep bundled skill descriptions under the 1024-char host limit

## [1.2.0] - 2026-07-11

Added
- auto-version from Conventional Commits + advisory commit-msg hook

Fixed
- tolerate a ruleset-protected main in the auto-bump
- retry the desktop build once on transient failure (Apple notary -1009)

## [1.1.0] - 2026-07-11

Multi-agent distribution for the `cyberos-init` payload, plus three new install channels. Backward compatible: `init.sh` never clobbers an existing operator file, so re-running it on a 1.0.0 repo only adds what is missing.

Added
- Agent surface in `init.sh`: `AGENTS.md` is the canonical cross-agent spine, with create-if-absent pointer files per agent (`CLAUDE.md`, `GEMINI.md`, `.cursorrules`, `.cursor/rules/*.mdc`, `.grok/GROK.md`, `.github/copilot-instructions.md`, `.agents/rules/`, `.windsurfrules`) and native installs of the `ship-feature-requests` skill into `.claude/skills`, `.grok/skills`, `.commandcode/skills`, `.codex/skills`, `.opencode/skill`. Controls: `CYBEROS_AGENTS`, `CYBEROS_COPY_SKILLS`, `CYBEROS_GLOBAL_SKILLS`, `CYBEROS_NO_MCP`. Covers Claude Code, Codex, Cursor, Gemini, Antigravity, Grok CLI, zcode, Command Code, Hermes, Copilot, Windsurf.
- MCP server channel (`tools/cyberos-init/mcp/cyberos-mcp.mjs`): zero-dependency Node stdio server exposing `fr_init`, `fr_gates`, `fr_status`, `ship_fr` (HITL-gated; never self-accepts). `init.sh` writes `.mcp.json` / `.cursor/mcp.json` when absent.
- npx CLI channel: root `package.json` with `cyberos-init`, `cyberos-gates`, `cyberos-mcp` bins.
- Template channel: `create.sh` scaffolder + `template/` skeleton for a fresh project or a GitHub template repo.
- Auto-versioning ("auto version, manual release"): `scripts/cyberos-version.mjs` computes the next version from Conventional Commits; `.github/workflows/version.yml` auto-commits the bump to `main` on push (never tags or deploys); `.githooks/commit-msg` is an advisory Conventional-Commit nudge that shows the projected next version. Cutting a release stays a manual `vX.Y.Z` tag. See `docs/deploy/RELEASE.md`.

Changed
- Root `AGENTS.md` produced by `init.sh` is now a concise workflow spine (was the full memory protocol); the dense Layer-1 protocol lives only at `.cyberos/memory/AGENTS.md`, referenced from the spine.
- `init.sh` errors clearly when run from an un-built source tree.

## [1.0.0] - 2026-07-10

The first platform release. One version (`VERSION`) across every surface: services (GHCR images + VPS), the web console and PWA at os.cyberskill.world, the desktop app (Tauri, with the CyberOS Ops tab), the distributable init payload (`dist/cyberos`), the Claude plugin, and the generated docs site.

Added
- ship-feature-requests as the single governed workflow (product + improvement classes, one backlog, HITL at the two human-acceptance gates).
- cyberos-init payload: `init.sh` (idempotent install/update, `--check`), agent-independent entry (`.cyberos/AGENT-ENTRY.md` + pointer stubs), `rollout.sh`, plugin marketplace + one-file `cyberos.plugin` bundle.
- Claude plugin 1.0.0: `/init`, `/update`, `/changelog`, `/help` commands + the `ship-feature-requests` skill.
- Desktop CyberOS Ops tab (FR-APP-001): build payload, list projects with installed versions, check, init/update - over the canonical scripts.
- Documentation single source of truth (FR-DOCS-002): markdown sources (module-owned + global), generated site at `dist/website`, Vercel hosting wiring for cyberos.cyberskill.world/docs, docs-prerender CI gate + pre-commit build check.
- CI/local parity: `scripts/local_verify.sh` runs the same migrations + per-crate DB suites as the services workflow; pre-push hook runs it when Docker is up.

Changed
- BRAIN store canonical location: `.cyberos/memory/store/` (legacy `.cyberos-memory/` removed platform-wide).
- Improvement work folded into `docs/feature-requests/` (`(improvement)` tags); separate improvement trees retired.

## awh absorption (2026-06-19, shipped in 1.0.0)

Platform-wide verification substrate. Agent self-certification at `testing -> done` is replaced at the seam by an out-of-band gate that reruns the real tests against a sealed baseline and blocks on regression.

Added
- awh vendored under `tools/awh/` (out-of-band verification gate; source sha c1f2c77, pure stdlib + PyYAML).
- Per-module golden sets: `modules/memory/.awh/`, `modules/skill/.awh/`.
- `ship-feature-requests` workflow: new step 28 `awh-gate`; `testing -> done` now requires an independent GREEN rerun against a sealed, read-only baseline (the done-flip is conditional; RED routes back to `ready_to_implement`).
- CI merge gate `.github/workflows/awh-gate.yml`; pre-commit hook `.pre-commit-hooks/awh-gate.sh`.
- `scripts/rebaseline_fr_status.py` (deterministic, idempotent FR status re-baseline).
- FR-MEMORY-124 (draft): `memory.awh_gate_result` audit row, gated on protocol change P23 §6. (Renumbered from FR-MEMORY-121; 121/122/123 now carry the BRAIN capture trio.)
- Maturity ledger migrated to `.awh/evolution-log.jsonl` (6 prior adoptions).

Changed
- FR statuses re-baselined: 116 `done` -> `ready_to_test` (independent awh re-verification pending; the code already exists on `main`).

Verified
- MEMORY green under the awh gate (pilot FR-MEMORY-116, weighted pass@1 = 100%).

## CAF (code-audit) absorption (2026-06-20, shipped in 1.0.0)

Second verification axis. Where awh reruns the tests, CAF reruns the target's own build/lint/typecheck/test and audits the code, catching the class awh cannot see (a build/lint break, a route that 404s, a changed data contract - e.g. the CCAF V2 regression).

Added
- CAF vendored under `tools/caf/` (from `CyberSkill/code-audit-framework`; validator self-test `code_audit_validator --all` = 40/40 GREEN, no install) + `tools/caf/field-data/` (calibration records from `code-audit-field-data`).
- `scripts/caf_gate.sh` - deterministic floor: target health (`tools/caf/core/evals/verify-target.sh` runs the module's own RUN_COMMANDS) + `code-audit-validate` of a sealed `modules/<m>/.caf/` audit when present. Fail-closed.
- `scripts/caf_precommit_check.sh` - structural fail-closed (every gated module must declare an `audit-profile.yaml`).
- `modules/<m>/audit-profile.yaml` for all 8 gated modules (ai, auth, proj, email, skill, chat, cuo, memory).
- `tools/caf/RETIREMENT.md`; design at `docs/verification/caf-absorption-design.md`.

Changed
- `ship-feature-requests` workflow -> v2.1.0: new step 29 `caf-gate` (awh-gate is 28; the done-flip steps renumber to 30/31); `testing -> done` now requires `awh GREEN AND caf CLEAN` (the done-flip dual condition); §10 outcome table adds a caf-RED rework row. New output `caf_gate_report` (emits `memory.caf_gate_result`).

Verified
- In-sandbox (no toolchain): validator `--all` 40/40 exit 0; pre-commit check 8/8 GREEN; verify-target.sh PASS / FAIL(exit 1) / fail-closed(exit 2) all correct; the 8 profiles parse to the expected commands. The cargo/pytest/make target-health runs are owner-run on a build machine (ai needs Redis).

## Per-module changelogs

| Module | Changelog |
|---|---|
| [Memory](https://cyberos-wiki.cyberskill.world/modules/memory/changelog.html) | Universal memory protocol |
| [CUO / Genie](https://cyberos-wiki.cyberskill.world/modules/cuo/changelog.html) | Persona orchestration |
| [Skill](https://cyberos-wiki.cyberskill.world/modules/skill/changelog.html) | Skill catalog |
| [Auth](https://cyberos-wiki.cyberskill.world/modules/auth/changelog.html) | Authentication |
| [AI Gateway](https://cyberos-wiki.cyberskill.world/modules/ai/changelog.html) | AI Gateway |
| [MCP Gateway](https://cyberos-wiki.cyberskill.world/modules/mcp/changelog.html) | MCP Gateway |
| [OBS](https://cyberos-wiki.cyberskill.world/modules/obs/changelog.html) | Observability |
| [Chat](https://cyberos-wiki.cyberskill.world/modules/chat/changelog.html) | Chat |
| [Email](https://cyberos-wiki.cyberskill.world/modules/email/changelog.html) | Email |
| [PROJ](https://cyberos-wiki.cyberskill.world/modules/proj/changelog.html) | Project tracking |
| [TIME](https://cyberos-wiki.cyberskill.world/modules/time/changelog.html) | Time tracking |
| [CRM](https://cyberos-wiki.cyberskill.world/modules/crm/changelog.html) | Customer relationships |
| [KB](https://cyberos-wiki.cyberskill.world/modules/kb/changelog.html) | Knowledge base |
| [HR](https://cyberos-wiki.cyberskill.world/modules/hr/changelog.html) | People & HR |
| [REW](https://cyberos-wiki.cyberskill.world/modules/rew/changelog.html) | Compensation |
| [LEARN](https://cyberos-wiki.cyberskill.world/modules/learn/changelog.html) | Learning |
| [INV](https://cyberos-wiki.cyberskill.world/modules/inv/changelog.html) | Invoicing |
| [ESOP](https://cyberos-wiki.cyberskill.world/modules/esop/changelog.html) | Stock options |
| [RES](https://cyberos-wiki.cyberskill.world/modules/res/changelog.html) | Resourcing |
| [OKR](https://cyberos-wiki.cyberskill.world/modules/okr/changelog.html) | Objectives & KRs |
| [DOC](https://cyberos-wiki.cyberskill.world/modules/doc/changelog.html) | Documents & signatures |
| [PORTAL](https://cyberos-wiki.cyberskill.world/modules/portal/changelog.html) | Client portal |
| [TEN](https://cyberos-wiki.cyberskill.world/modules/ten/changelog.html) | Tenants |
| [Website](https://cyberos-wiki.cyberskill.world/modules/website/changelog.html) | Website & Infrastructure |

## Repository-level changes

### 2026-05-18 — Consolidation pass

Moved all CyberOS-related artifacts into a single umbrella at `cyberos/`:

- `workbench/CyberOS-docs/` → `website/docs/`
- `workbench/CYBEROS_STRATEGY.md` → `playground/CYBEROS_STRATEGY.md`
- `workbench/cyberskill-vn-skills/` → `playground/cyberskill-vn-skills/`
