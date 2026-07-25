# CyberOS module manifest (TASK-IMP-055)
#
# Machine-readable companion: modules/manifest.yaml
# Kind meanings:
#   ships_code     — module tree carries executable code vendored into the payload
#                    (Python package, Rust crate, JS helpers, HTML/CSS templates)
#   docs_skills_only — CHANGELOG / audit-profile / skill stubs only; no runtime code
#                    in modules/<id>/. Platform services (if any) live under services/,
#                    not here. Specs under docs/tasks/<module>/ remain the backlog truth.
#
# Generated for CyberOS 1.x payload honesty (deep-audit R9). Do not invent phantom
# importable packages.

| Module | Kind | Notes |
|---|---|---|
| ai | docs_skills_only | Spec / changelog surface; AI gateway code is `services/` |
| auth | docs_skills_only | Spec / changelog surface |
| chat | docs_skills_only | Spec / changelog; native chat is `services/` + `apps/` |
| crm | docs_skills_only | Spec stub |
| cuo | ships_code | Python `cuo` package + persona workflows |
| doc | docs_skills_only | Spec stub |
| email | docs_skills_only | Spec / changelog |
| esop | docs_skills_only | Spec stub |
| hr | docs_skills_only | Spec stub |
| inv | docs_skills_only | Spec stub |
| kb | docs_skills_only | Spec stub |
| learn | docs_skills_only | Spec stub |
| mcp | docs_skills_only | Module page/changelog; install MCP server is `tools/install/mcp/` |
| memory | ships_code | Python memory protocol + schema + tools |
| obs | docs_skills_only | Spec / changelog |
| okr | docs_skills_only | Spec stub |
| plugin | docs_skills_only | Spec stub; host plugins under `tools/install/plugin/` |
| portal | docs_skills_only | Spec stub |
| proj | docs_skills_only | Spec / changelog |
| res | docs_skills_only | Spec stub |
| rew | docs_skills_only | Spec stub |
| skill | ships_code | Skill corpus + Rust helpers + runners |
| templates | ships_code | Status-hub HTML/CSS/JS templates |
| ten | docs_skills_only | Spec stub |
| time | docs_skills_only | Spec stub |
| website | docs_skills_only | Module page stub |

## How to use

- Agents: treat `docs_skills_only` modules as **specs**, not importable libraries.
- Payload build: only `ships_code` module trees are expected to contribute runtime artefacts (see `tools/install/build.sh` cone).
- Source of truth for work items remains `docs/tasks/BACKLOG.md`, not this table.
