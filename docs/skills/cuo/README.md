# `cuo/` — Chief Universal Officer skill namespace

> CUO is the outer persona surface (PRD Part 6); the 14 sub-personas listed
> below are CUO's specialists. Each sub-persona is an Anthropic Skill (DEC-061)
> with its own scope contract and its own bag of workflow skills.

This README is the namespace index. It does NOT define how skills work — that
contract lives one level up at `cyberos/docs/skills/README.md`. Read that
first.

## 1. The 14 sub-personas (locked: DEC-052)

### 1.1 The 10 canonical roles (Indeed.com taxonomy)

| ID | Role | Phase available | Owns workflows like |
| --- | --- | --- | --- |
| `ceo`  | Chief Executive Officer            | P1+ | strategy memos, OKR roll-up, board updates |
| `coo`  | Chief Operating Officer            | P1+ | weekly ops review, runbook execution |
| `cfo`  | Chief Financial Officer            | P1+ | cashflow projection, payroll narration, budget variance |
| `cmo`  | Chief Marketing Officer            | P2+ | campaign brief, content calendar, attribution review |
| `cto`  | Chief Technology / Information Officer | P0  | tech-spec drafting, architecture review notes, incident triage |
| `chro` | Chief Human Resources Officer      | P1+ | onboarding plan, performance-cycle prep, leave summary |
| `cseco`| Chief Security Officer (was CSO)   | P1+ | threat-model review, breach-response coordination |
| `clo`  | Chief Legal Officer                | P1+ | EU AI Act conformity check, contract redline summary |
| `cdo`  | Chief Data Officer                 | P2+ | data-quality digest, lineage explainer, schema migration |
| **`cpo`**  | **Chief Product Officer**      | **P0** | **FR backlog, FR audit, tech-spec from FR, roadmap rollup** |

### 1.2 The 4 emergent roles (added 2025–2026)

| ID | Role | Phase available | Owns workflows like |
| --- | --- | --- | --- |
| `caio` | Chief AI Officer                   | P1+ | model-card drafting, EU AI Act Annex IV pack, model-eval review |
| `cxo`  | Chief Experience Officer           | P2+ | NPS digest, journey-friction surfacing |
| `cro`  | Chief Revenue Officer              | P2+ | pipeline review, win/loss synthesis |
| `cso-sustainability` | Chief Sustainability Officer | P3+ | ESG roll-up, scope-3 emissions narrative |

A persona is "available" when its `<role>/SKILL.md` exists with a non-empty
workflow set AND its acceptance test passes (SRS §6.10). Stale persona-cards
that still exist but lack a workflow set are not selectable by the router.

## 2. Routing into a sub-persona

Per SRS §6.1.1 + PRD §6.3, a request enters CUO's LangGraph and hits the
classify_act node. The classifier returns `{persona_id, skill_id, confidence}`.
Disambiguation rules:

1. If the user names a persona explicitly ("ask the CFO…"), confidence
   override = 1.0; route to that persona's owned skill set.
2. If the requested action implies a regulated domain (REW / LEARN / ESOP /
   compliance / legal), an automatic CC to the matching persona is added —
   e.g., a CHRO action that touches comp gets the CFO and CLO on the
   audit row's `cc_personas:` field. Cross-persona CC is informational; it
   does NOT change who acts.
3. If multiple personas could plausibly own the request, escalate via
   the Question primitive (SRS §6.6.2).
4. Below `defer_below` confidence (per skill frontmatter), surface
   "I'm not sure which workflow you mean — here are the candidates" to
   the user.

## 3. Persona-card contract (one per role folder)

Each `<role>/SKILL.md` is a *persona card*, NOT a workflow. It declares:

- The role's voice + decision style (cites PRD §6.2 globally; lists per-role
  deltas only).
- The role's `allowed_brain_scopes` and `allowed_mcp_tools` ceilings — every
  workflow under this role inherits these and may declare a strict subset
  but never a superset.
- The role's owned workflow folders by name.
- The role's `escalation:` graph — which personas this one defers to on
  legal / security / compliance issues.
- The role's confidence-band defaults (overridable per workflow).

A persona card MUST NOT carry an `expects:` / `produces:` envelope. Personas
are containers for workflows; only workflows have pipeline interfaces.

## 4. The `_shared/` subdirectory

Skills that don't naturally belong to one persona live here. DEC-061's
worked example is `draft-payslip-explanation`, jointly used by CFO and CHRO.
Rules:

- A `_shared/` skill's `owner_role:` is `_shared`.
- Its `allowed_brain_scopes` and `allowed_mcp_tools` MUST be the intersection
  (not union) of every persona that calls it. The router enforces this at
  invocation.
- Its CHANGELOG is the canonical history; persona cards that reference it
  link without copying.

Currently in `_shared/`:

| Skill | Used by | Purpose |
| --- | --- | --- |
| `feature-request-template` | `cpo/fr-create`, `cpo/fr-audit`, future tech-spec workflows | holds the `feature_request@1` schema + canonical body skeleton |

## 5. Phase availability gates (P0 → P4)

CUO ships incrementally per PRD Part 14. A persona becomes operational only
when its prerequisite modules ship. Current state (P0 in flight):

- **P0 (Months 1–3, current)** — `cpo` (this is what we're building today),
  `cto` (tech spec drafting alongside `cpo` for product → engineering
  handoff). All other personas are placeholder folders only.
- **P1 (Months 4–6)** — `ceo`, `coo`, `cfo`, `chro`, `cseco`, `clo`, `caio`
  come online once HR / REW / TIME / CRM modules ship.
- **P2+** — remaining roles light up per PRD §14.

Adding a workflow under a future-phase persona is allowed *as documentation*
but the runtime classifier will not route to it until the persona's gate
passes. The persona-card frontmatter MUST carry
`gated_until_phase: P<n>` until the gate clears; routing returns
`E_PERSONA_GATED` for premature invocations.

## 6. Index of workflow skills (per persona)

| Persona | Skill | Status |
| --- | --- | --- |
| `cpo`   | `fr-create` | v0.1.0 (port of `feature-request/v2.0.0` create half) |
| `cpo`   | `fr-audit`  | v0.1.0 (port of `feature-request/v2.0.0` audit half) |

Future persona folders are intentionally absent from the tree. Create them
on demand — `mkdir cuo/<role>/` — at the moment the first workflow lands.

## 7. Citations

- 14-persona registry → SRS §6.3 + DEC-052.
- Anthropic Skill format → SRS §6.2.1 + DEC-061.
- LangGraph + classify_act node → SRS §6.1.1 + DEC-027.
- Notify / Question / Review → SRS §6.6.
- Acceptance gate + drift detection → SRS §6.10 + §6.12 + DEC-055.
- Phase plan → PRD Part 14.
- Voice + decision style → PRD §6.2.
