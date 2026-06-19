# Changelog — CyberOS

This is the repo-level changelog for CyberOS. For module-specific changelogs, see the per-module pages on the documentation site.

## [Unreleased] - awh absorption (2026-06-19, branch auto/awh-absorb)

Platform-wide verification substrate. Agent self-certification at `testing -> done` is replaced at the seam by an out-of-band gate that reruns the real tests against a sealed baseline and blocks on regression.

Added
- awh vendored under `tools/awh/` (out-of-band verification gate; source sha c1f2c77, pure stdlib + PyYAML).
- Per-module golden sets: `modules/memory/.awh/`, `modules/skill/.awh/`.
- `ship-feature-requests` workflow: new step 28 `awh-gate`; `testing -> done` now requires an independent GREEN rerun against a sealed, read-only baseline (the done-flip is conditional; RED routes back to `ready_to_implement`).
- CI merge gate `.github/workflows/awh-gate.yml`; pre-commit hook `.pre-commit-hooks/awh-gate.sh`.
- `scripts/rebaseline_fr_status.py` (deterministic, idempotent FR status re-baseline).
- FR-MEMORY-121 (draft): `memory.awh_gate_result` audit row, gated on protocol change P23 §6.
- Maturity ledger migrated to `.awh/evolution-log.jsonl` (6 prior adoptions).

Changed
- FR statuses re-baselined: 116 `done` -> `ready_to_test` (independent awh re-verification pending; the code already exists on `main`).

Verified
- MEMORY green under the awh gate (pilot FR-MEMORY-116, weighted pass@1 = 100%).

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
