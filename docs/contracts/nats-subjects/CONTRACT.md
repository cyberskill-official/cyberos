---
# ── Identity ─────────────────────────────────────────────────────────
contract_id: nats-subjects
contract_version: v1
template_literal: nats_subjects@1
description: Canonical naming and shape contract for every NATS subject emitted or subscribed by a CyberOS skill. Skills publish events on the bus (e.g. `cuo.fr_author.fr_written`) so the LangGraph supervisor's classify-act node can decide whether to chain a follow-up skill. This contract pins subject names, payload shapes, and the durability/QoS promises so the bus is interoperable across skills + persona namespaces.
contract_kind: wire_protocol        # artefact_schema | envelope_schema | wire_protocol
locked_at: 2026-05-06

# ── Stewardship ──────────────────────────────────────────────────────
steward_persona: cuo-cto             # CTO owns the runtime wire surfaces
escalation_on_breach:
  legal:    null
  security: cuo-cseco                # subject leakage / unauthorised pub-sub is a security event
  compliance: null

# ── Determinism ──────────────────────────────────────────────────────
determinism:
  reproducible: true
  fixity_notes: "Subject naming convention is byte-stable. Bumping a subject's payload shape (adding a required field, removing a field, changing an enum) requires a MAJOR contract_version bump (nats_subjects@2) and a coordinated update to every publishing skill."

# ── Source-tier emitted ──────────────────────────────────────────────
emitted_source_freshness_tier: 15   # high authority — this IS the wire protocol
---

# `nats_subjects@1` — canonical NATS subject contract

> A **wire-protocol contract**, not a skill. Pins the names, payload shapes, and QoS promises of every NATS subject that flows between CyberOS skills, the supervisor, and the audit ledger. Loaded by every skill that publishes or subscribes to a subject; loaded by the supervisor to know which subjects to react to. New subjects MUST be added here before any skill emits them.

## Why this contract exists

Skills decoupled via NATS pub-sub (per DEC-029) need a shared vocabulary so the supervisor's classify-act node can route events to follow-up skills deterministically. Without a contract, two teams could independently invent `cuo.fr.create.done` and `cuo.fr_author.completed` for the same event — the supervisor would catch one and miss the other. This contract fixes the names at the registry level, not the skill level, so every consumer has a single source of truth.

## How a skill consumes this contract

```yaml
# In the skill's SKILL.md frontmatter:
depends_on_contracts:
  - id:        nats-subjects
    version:   v1
    purpose:   wire_protocol_emission       # human-readable
    pin_path:  cyberos/docs/contracts/nats-subjects/
```

The skill body MUST reference subjects by their canonical name listed in `schema.json`. The registry validator confirms every `nats.publish` / `nats.subscribe` call cites a subject declared here.

## Subject naming convention

Subjects use **dot-separated lowercase tokens** in the form:

```
<top_level_persona>.<skill_id_with_underscores>.<event_name>
```

- **`<top_level_persona>`** — the **outer** lowercase persona namespace (`cuo`, future `cao`, `cmo`, etc. — whatever sits at the registry root). Sub-personas (`cpo`, `cto`, `clo`, …) are NOT included in the subject name; they're implicit in the `skill_id`. Rationale: subjects exist at the routing layer (the supervisor consumes them), and the supervisor routes at the top-level persona granularity. Including the sub-persona makes the subject more verbose without adding routing power. The persona's specific sub-namespace IS preserved in the `skill_id` field of the payload (e.g. `"skill_id": "cuo/cpo/fr-author"`).
- **`<skill_id_with_underscores>`** — the skill folder name with hyphens replaced by underscores (`fr-author` → `fr_author`).
- **`<event_name>`** — past-tense lowercase phrase describing what happened. Examples: `fr_written`, `audit_complete`, `hitl_pause`, `refinement_proposed`.

This naming matches the pre-existing skill-body convention used by `cuo/cpo/fr-author` since v0.1.0 (e.g. `cuo.fr_author.fr_written`). The contract documents the existing convention; it does not redefine it.

### Reserved tokens

The following token positions are reserved and may not be used as `<event_name>`:

- `*` — NATS single-token wildcard. Reserved for subscriber side only.
- `>` — NATS multi-token wildcard. Reserved for subscriber side only.
- `_` (leading) — internal-only subjects. Skills must not subscribe to underscore-prefixed events.

### Examples (current registry inventory)

| Subject | Publisher | Payload shape ref | QoS | Durability |
| --- | --- | --- | --- | --- |
| `cuo.fr_author.fr_written` | `cuo/cpo/fr-author` | `schema.json#/payloads/fr_written` | at-least-once | `WorkQueue` retain ≥7d |
| `cuo.fr_author.batch_complete` | `cuo/cpo/fr-author` | `schema.json#/payloads/batch_complete` | at-least-once | `WorkQueue` retain ≥7d |
| `cuo.fr_author.hitl_pause` | `cuo/cpo/fr-author` | `schema.json#/payloads/hitl_pause` | at-least-once | `Memory` retain ≥30d |
| `cuo.fr_audit.audit_written` | `cuo/cpo/fr-audit` | `schema.json#/payloads/audit_written` | at-least-once | `WorkQueue` retain ≥7d |
| `cuo.fr_audit.audit_batch_complete` | `cuo/cpo/fr-audit` | `schema.json#/payloads/audit_batch_complete` | at-least-once | `WorkQueue` retain ≥7d |
| `cuo.fr_audit.hitl_pause` | `cuo/cpo/fr-audit` | `schema.json#/payloads/hitl_pause` | at-least-once | `Memory` retain ≥30d |
| `cuo.refinement_proposed` | any skill | `schema.json#/payloads/refinement_proposed` | at-least-once | `Memory` retain ≥90d |
| `cuo.supervisor.session_start` | the supervisor | `schema.json#/payloads/session_lifecycle` | at-most-once | ephemeral |
| `cuo.supervisor.session_end` | the supervisor | `schema.json#/payloads/session_lifecycle` | at-most-once | ephemeral |

The full payload shapes live in `schema.json` (one schema per `<event_name>`). Note the supervisor's session lifecycle subjects use `cuo.supervisor.<event>` — `supervisor` is treated as a pseudo-skill_id since the supervisor is the routing layer, not a skill in `cuo/<persona>/<skill>/`. This keeps the three-token shape consistent across all subjects.

## QoS levels

| Level | NATS mechanism | When to use |
| --- | --- | --- |
| `at-most-once` | Core NATS pub-sub | Best-effort signals where loss is acceptable (heartbeats, session lifecycle). |
| `at-least-once` | JetStream `WorkQueue` or `Memory` | Domain events that drive downstream skill execution; loss = a missed downstream chain. |
| `exactly-once` | JetStream `Stream` + dedup-window + idempotent consumer | Reserved; not currently used. Document a use-case before adding a subject at this QoS. |

## Durability tiers

| Tier | NATS retention | When to use |
| --- | --- | --- |
| `ephemeral` | No JetStream | Session-scoped events the supervisor consumes in real-time and never replays. |
| `WorkQueue` retain ≥7d | JetStream `WorkQueue`, max-age 7 days | Domain events that drive downstream skill execution; consumer must ack. After 7d, considered processed. |
| `Memory` retain ≥30d | JetStream `Memory`, max-age 30 days | HITL pauses + refinement proposals — humans need a multi-week window to respond. |
| `Memory` retain ≥90d | JetStream `Memory`, max-age 90 days | Refinement proposals where the cycle is multi-month. |

## How to add a new subject

1. Open this folder's `schema.json` and add a payload schema under `properties.payloads.<event_name>` with `type: object`, required fields, and a description.
2. Add a row to the inventory table above naming publisher / QoS / durability.
3. Update `CHANGELOG.md` with a v1.x entry (MINOR if added, MAJOR if changed).
4. Update every publishing skill's `depends_on_contracts:` to reference this contract version.
5. Add an acceptance test that publishes the subject + a subscriber that asserts the payload shape.

## Forbidden patterns

- **Camel-case or hyphen in subject tokens.** Subjects are dot-separated lowercase. `cuo.frCreate.fr_written` and `cuo.fr-author.fr_written` are both rejected.
- **Verb-first event names.** Use past-tense `<noun>_<verb_past>` not `<verb>_<noun>`. Good: `fr_written`. Bad: `write_fr`.
- **Subjects with PII or secrets in payload.** Same denylist as `.cyberos-memory/` (AGENTS.md §9.3): no salaries, government IDs, bank, secrets, raw API keys. Use a pointer to the secret store instead.
- **Cross-tenant subscriptions.** A skill MUST only subscribe to subjects emitted by skills in its own tenant (matched on the `tenant.id` from `manifest.json`). The supervisor enforces at the bus level.

## Citations

- **DEC-029** — NATS event bus baseline (CyberOS-PRD §4.10).
- **DEC-090** — registry v0.2.0; introduces `cyberos/docs/contracts/`.
- **DEC-091** — `depends_on_contracts:` declaration mechanism.
- **DEC-092** — `cuo.refinement_proposed` subject is the wire form of the auto-refinement loop's surface to the supervisor.
- **fr-author** + **fr-audit** v0.2.2 — first concrete consumers of this contract.
