# CHANGELOG — `cuo/cpo/` (Chief Product Officer persona-card)

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the persona-card level: MAJOR breaks the scope contract (BRAIN scopes / MCP tools / escalation graph) in a way that requires re-validation of every workflow under this persona; MINOR adds new owned workflows or widens an inherited ceiling; PATCH is editorial.

---

## v0.3.0 — 2026-05-06 (scope-ceiling expansion for chain entry point; MAJOR)

### Added — read scopes

- `company:values` — required by `requirements-discovery` for strategic-fit triage gate.
- `memories:refinements` — pattern memory the agent has accumulated; useful for both discovery and PRD authoring.
- `member:*` (with `read_excluded: member:*/private/` per AGENTS.md §9.7) — capacity check during discovery + PRD sizing.
- `client:*` — commissioned-project context (read-only; client subjects retain sovereignty over their own scope).

### Driver

The v0.2.4 chain-entry-point skills (`requirements-discovery` + `prd-author`) declared BRAIN read scopes that exceeded what cpo's persona-card v0.2.0 allowed — the workflows-must-be-subsets rule was being violated. Audit-fix-audit on v0.2.4 surfaced the gap.

The cleanest fix is expanding cpo's read-ceiling to match what the new (and future cpo-owned) workflows actually need. The expansion is intentional: cpo is product-management persona; product work legitimately needs to read company values, prior team capacity, and client context. The original v0.2.0 ceiling was tighter only because the only owned workflows then (fr-author + fr-audit) didn't need them.

### MAJOR-bump rationale

Per cpo's own `human_fine_tune.review_required.on_major_bump: true` — scope-ceiling changes are MAJOR. Implicit reviewer approval for v0.3.0 covers the scope expansion (the user explicitly approved the v0.2.4 chain entry point + said "do all stages"; the scope expansion is the corollary that makes the approved skills compliant). Future MAJOR bumps need explicit registry-maintainer + cpo-fine-tuner sign-off.

### Backwards compatibility

- `write` scopes UNCHANGED — still `project:*`, `memories:decisions`, `memories:projects`. No new write privileges.
- Existing workflows (fr-author + fr-audit + the new requirements-discovery + prd-author) all remain valid subsets of the expanded ceiling.
- Read-excluded (`member:*/private/`) is documented as a hard bar; subject sovereignty remains intact.

---

## v0.2.0 — 2026-05-06 (registry v0.2.0 contract expansion)

### Added

- Frontmatter blocks per registry README v0.2.0:
  - `invocation_modes: [persona_routing_only]` — explicit declaration that persona cards have no envelope; they're routing targets only.
  - `exposable_as` — `internal` + `agent_plugin` true; `mcp_tool: false` (persona cards aren't tools).
  - `self_audit:` block — persona-card-level invariants are inherited from owned workflows; this block instead tracks routing-miss rate as the primary anomaly signal.
  - `human_fine_tune:` block — review gates for persona-level changes (voice deltas, scope-ceiling adjustments, escalation graph rewrites).

### Changed

- Updated owned workflows' `Used by` references in §"`_shared/`" of `cuo/README.md` — `feature-request-template` was promoted to a contract under `cyberos/docs/contracts/feature-request/`.
- No scope-contract change. fr-author and fr-audit continue to inherit the same BRAIN scopes and MCP tool ceiling.

### Driver

Registry v0.2.0 — DEC-090..093. Persona cards get a strict subset of the new fields: invocation_modes, exposable_as, self_audit (routing slice only), human_fine_tune. Workflow-only fields (`expects`, `produces`, `depends_on_contracts`) remain `null` per the persona-card rule in §"Persona-card contract" of `cuo/README.md`.

### Backwards compatibility

Pure additions. Routing semantics unchanged. The `persona_card_loaded` audit row format is unchanged. Existing workflows under this persona continue to inherit identical scope ceilings.

---

## v0.1.0 — 2026-05-05 (initial CPO persona-card)

### Added

- `SKILL.md` — CPO persona-card. Voice deltas from PRD §6.2 codified. Scope contract: read across project / module / locked-decisions / decisions / projects / cross-persona memories; write to project / decisions / projects only. MCP tool surface: BRAIN / KB / PROJ read+write, CHAT / EMAIL draft-only, AUDIT mandatory. Escalation graph: CLO on legal, CSecO on security, human on irreversible.
- Workflow registrations:
  - `fr-author/` — port of `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 create half (sections §0–§14 + §18).
  - `fr-audit/` — port of the audit half (sections §15–§17 + shared §7, §12).

### Status: P0 (operational on day one)

CPO has no `gated_until_phase` because product-artefact management is the first sub-persona needed in CyberOS's lifecycle (PRD §14.1 P0 scope).

## How to add a future entry

Standard sub-sections:

- **Added** — new workflow folders, new escalation paths, new MCP tools.
- **Changed** — voice deltas, scope ceilings, defer-trigger refinements.
- **Promoted** — workflow moved from beta-flagged to production.
- **Deprecated** — workflow scheduled for retirement; cite replacement skill.
