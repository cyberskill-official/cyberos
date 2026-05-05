# CHANGELOG — `cuo/cpo/` (Chief Product Officer persona-card)

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/).
> SemVer at the persona-card level: MAJOR breaks the scope contract (BRAIN
> scopes / MCP tools / escalation graph) in a way that requires re-validation
> of every workflow under this persona; MINOR adds new owned workflows or
> widens an inherited ceiling; PATCH is editorial.

---

## v0.1.0 — 2026-05-05 (initial CPO persona-card)

### Added

- `SKILL.md` — CPO persona-card. Voice deltas from PRD §6.2 codified.
  Scope contract: read across project / module / locked-decisions / decisions
  / projects / cross-persona memories; write to project / decisions / projects
  only. MCP tool surface: BRAIN / KB / PROJ read+write, CHAT / EMAIL draft-only,
  AUDIT mandatory. Escalation graph: CLO on legal, CSecO on security, human
  on irreversible.
- Workflow registrations:
  - `fr-create/` — port of `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0
    create half (sections §0–§14 + §18).
  - `fr-audit/` — port of the audit half (sections §15–§17 + shared §7, §12).

### Status: P0 (operational on day one)

CPO has no `gated_until_phase` because product-artefact management is the
first sub-persona needed in CyberOS's lifecycle (PRD §14.1 P0 scope).

## How to add a future entry

Standard sub-sections:

- **Added** — new workflow folders, new escalation paths, new MCP tools.
- **Changed** — voice deltas, scope ceilings, defer-trigger refinements.
- **Promoted** — workflow moved from beta-flagged to production.
- **Deprecated** — workflow scheduled for retirement; cite replacement skill.
