# Changelog — CyberOS

This is the repo-level changelog for CyberOS. For module-specific changelogs, see the per-module pages on the documentation site.

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
