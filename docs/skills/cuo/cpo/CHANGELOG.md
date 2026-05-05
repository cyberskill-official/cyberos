# CHANGELOG — `cuo/cpo/` (Chief Product Officer persona-card)

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the persona-card level: MAJOR breaks the scope contract (BRAIN scopes / MCP tools / escalation graph) in a way that requires re-validation of every workflow under this persona; MINOR adds new owned workflows or widens an inherited ceiling; PATCH is editorial.

---

## v0.2.0 — 2026-05-06 (registry v0.2.0 contract expansion)

### Added

- Frontmatter blocks per registry README v0.2.0:
  - `invocation_modes: [persona_routing_only]` — explicit declaration that persona cards have no envelope; they're routing targets only.
  - `exposable_as` — `internal` + `agent_plugin` true; `mcp_tool: false` (persona cards aren't tools).
  - `self_audit:` block — persona-card-level invariants are inherited from owned workflows; this block instead tracks routing-miss rate as the primary anomaly signal.
  - `human_fine_tune:` block — review gates for persona-level changes (voice deltas, scope-ceiling adjustments, escalation graph rewrites).

### Changed

- Updated owned workflows' `Used by` references in §"`_shared/`" of `cuo/README.md` — `feature-request-template` was promoted to a contract under `cyberos/docs/contracts/feature-request/v1/`.
- No scope-contract change. fr-create and fr-audit continue to inherit the same BRAIN scopes and MCP tool ceiling.

### Driver

Registry v0.2.0 — DEC-090..093. Persona cards get a strict subset of the new fields: invocation_modes, exposable_as, self_audit (routing slice only), human_fine_tune. Workflow-only fields (`expects`, `produces`, `depends_on_contracts`) remain `null` per the persona-card rule in §"Persona-card contract" of `cuo/README.md`.

### Backwards compatibility

Pure additions. Routing semantics unchanged. The `persona_card_loaded` audit row format is unchanged. Existing workflows under this persona continue to inherit identical scope ceilings.

---

## v0.1.0 — 2026-05-05 (initial CPO persona-card)

### Added

- `SKILL.md` — CPO persona-card. Voice deltas from PRD §6.2 codified. Scope contract: read across project / module / locked-decisions / decisions / projects / cross-persona memories; write to project / decisions / projects only. MCP tool surface: BRAIN / KB / PROJ read+write, CHAT / EMAIL draft-only, AUDIT mandatory. Escalation graph: CLO on legal, CSecO on security, human on irreversible.
- Workflow registrations:
  - `fr-create/` — port of `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 create half (sections §0–§14 + §18).
  - `fr-audit/` — port of the audit half (sections §15–§17 + shared §7, §12).

### Status: P0 (operational on day one)

CPO has no `gated_until_phase` because product-artefact management is the first sub-persona needed in CyberOS's lifecycle (PRD §14.1 P0 scope).

## How to add a future entry

Standard sub-sections:

- **Added** — new workflow folders, new escalation paths, new MCP tools.
- **Changed** — voice deltas, scope ceilings, defer-trigger refinements.
- **Promoted** — workflow moved from beta-flagged to production.
- **Deprecated** — workflow scheduled for retirement; cite replacement skill.
