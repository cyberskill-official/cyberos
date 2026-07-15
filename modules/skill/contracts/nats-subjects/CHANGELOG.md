# CHANGELOG — `cyberos/docs/contracts/nats-subjects/`

> Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). SemVer at the contract-major level: MAJOR breaks any payload shape (removes a field, changes an enum, renames a subject). MINOR adds a new subject or extends a payload additively. PATCH is editorial / clarification.

---

## v1.0.0 — 2026-05-06 (initial release)

### Added

- `CONTRACT.md` — wire-protocol contract documenting subject naming convention, QoS levels, durability tiers, the inventory table, and the "how to add a new subject" procedure.
- `schema.json` — JSON Schema for every subject's payload shape: `task_written`, `batch_complete`, `hitl_pause`, `audit_written`, `audit_batch_complete`, `refinement_proposed`, `session_lifecycle` (covers both `session_start` and `session_end`).
- `protocol.md` — operational protocol: connection management, ack semantics, dedup window, subject discovery, error semantics, retention + replay, cross-tenant isolation, observability.

### Driver

Pre-deployment audit of `cuo/cpo/task-author` + `cuo/cpo/task-audit` v0.2.1 (registry v0.2.2 audit) surfaced the gap: both skills declare `chat.notify` and `audit.append` MCP tools but the underlying NATS subjects they publish (`cuo.task_author.task_written`, `cuo.task_audit.audit_written`, etc.) were undocumented anywhere — neither skill nor contract namespace owned the wire format. Without a canonical contract, two future skills could independently invent overlapping subject names. This contract fixes the names at the registry level.

The contract's first draft over-specified the naming convention as `<sub-persona>.<skill_id>.<event>` (e.g. `cuo_cpo.task_author.task_written`); the audit-fix-audit loop caught the drift between the over-specified contract and the pre-existing skill body convention `<top-level-persona>.<skill_id>.<event>` (e.g. `cuo.task_author.task_written`) within the same release. Reality won — the contract was corrected before merge to match the existing skill bodies.

Tier-2 finding `B2` (audit log entry: registry v0.2.2 audit, finding ID `B2-nats-wire-protocol-undocumented`) requested a `wire_protocol` contract under `cyberos/docs/contracts/nats-subjects/` with payload schemas for the 7 currently-emitted subjects and a forward-compatible "how to add" procedure.

### Backwards compatibility

First version. No prior contract to migrate from. Skills v0.2.2+ declare `depends_on_contracts:` to this version; v0.2.0 / v0.2.1 skills published these subjects without a declared contract (de-facto ad-hoc) and will be migrated forward at their next bump.

### Acceptance evidence

- All 7 currently-emitted subjects across `cuo/cpo/task-author` + `cuo/cpo/task-audit` are listed in `CONTRACT.md`'s inventory table.
- All 7 subjects have a payload schema in `schema.json` under `payloads/<event_name>`.
- `protocol.md` operational rules align with NATS JetStream defaults at the time of writing (NATS 2.10+).
- One acceptance test pending under `cyberos/docs/skills/cuo/cpo/task-author/acceptance/` to publish + subscribe + assert payload shape (v0.3.0 milestone).

## How to add a future entry

Standard sub-sections:

- **Added** — new subjects, new payload fields (additive), new QoS levels.
- **Changed** — payload fields with non-breaking semantics changes, QoS upgrades.
- **Removed** — subject deprecations (always MAJOR).
- **Backwards compatibility** — what payloads from prior versions still validate, what migrates automatically.
- **Acceptance evidence** — pointer to the test that validated the release.
