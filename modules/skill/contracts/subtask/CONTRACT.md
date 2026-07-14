---
# ── Identity ─────────────────────────────────────────────────────────
contract_id: subtask
contract_version: v1
template_literal: subtask@1
description: Canonical subtask@1 schema — comprehensive, addressable, assignable unit of work embedded inside a task@1. Each task is self-contained (no character limit), carries a runnable acceptance test, and declares assignability (human / ai-agent / either). Promoted to first-class contract in 2026-05-12 to support the `solo` chain_profile that collapses task-author + fr-to-tech-spec.
contract_kind: artefact_schema
locked_at: 2026-05-12
introduced_by: skills-Stage-1-collapse

# ── Stewardship ──────────────────────────────────────────────────────
steward_persona: cuo-cpo
escalation_on_breach:
  legal:    cuo-clo
  security: cuo-cseco
  compliance: cuo-clo

# ── Determinism ──────────────────────────────────────────────────────
determinism:
  reproducible: true
  fixity_notes: "Field set + structure is byte-stable. Task bodies are judgement-driven prose; field shape is not. Bumping field shape requires task@2 + MAJOR bumps on every consuming skill."

# ── Source-tier emitted ──────────────────────────────────────────────
emitted_source_freshness_tier: 10
---

# `subtask@1` — canonical task contract

A task is a comprehensive, ready-to-assign unit of work that lives inside a task@1 artefact. Each task is self-contained enough that an engineer or an AI agent can pick it up and execute without re-reading the parent FR's discussion.

## When to use

- Every `task@1` produced by the `solo` chain_profile carries an embedded `tasks:` list.
- The `standard` and `full` chain_profiles keep tasks in a sibling `tech_spec@1`; `subtask@1` is still the shape they conform to.
- External project-tracker tickets (Linear / Jira / GitHub) are derived from `subtask@1` instances via `cyberos proj sync`.

## Task ID format

`<FR-id>-T-<MM>` — e.g. `TASK-007-S-03` is the third task of FR-007. Sequential within an FR, two-digit zero-padded. Greppable, auditable, referenceable from PR descriptions ("closes TASK-007-S-03").

## Required fields

```yaml
id: FR-NNN-T-MM                       # required
title: <one-sentence imperative>      # required
description: |                        # required, no character cap
  Full description of what this task accomplishes. Includes context an
  engineer or AI agent needs to execute without re-reading the parent
  FR. Cite specific files, line numbers, runtimes, configs as relevant.
  Multi-paragraph encouraged.
preconditions:                        # required (list, may be empty)
  - <what must be true before starting>
deliverables:                         # required (list of concrete outputs)
  - <file paths, schema changes, audit rows, etc.>
acceptance_test:                      # required, exactly one of:
  shell: "<exact command that returns 0 if accepted>"
  # OR
  assertion: "<structured assertion: e.g. 'cyberos verify returns CRITICAL: 0'>"
sizing: S | M | L | XL                # required (S=<2h, M=<1d, L=<3d, XL=>3d)
dependencies: [<task-id>, ...]        # required (may be empty)
parallelisable: true | false          # required (can run alongside non-dep tasks)
assignable_to: [human, ai-agent]      # required (one or both)
agent_profile: <profile-id>           # required when "ai-agent" in assignable_to
  # e.g. "claude-sonnet-4-6, mcp_allowlist: [bash, edit, memory.read]"
estimated_tokens: <int>               # required when "ai-agent" in assignable_to
estimated_hours: <float>              # required when "human" in assignable_to
status: draft | ready | in_progress | done | blocked  # required; init = draft
runbook_hint: <skill-name or null>    # optional — points at a Layer-1 skill that helps
```

## Optional fields

```yaml
owner: subject:<slug> | null          # who picked it up (or null)
sprint: <sprint-id> | null            # when planning lands
linked_pr: <url> | null               # populated post-completion
notes: <free text>                    # operator annotations
review_cohort: [subject:..., ...]     # who must approve before done
```

## Body template

```markdown
### {id} — {title}

**Sizing**: {sizing}  ({estimated_hours}h human / {estimated_tokens} tokens AI)
**Assignable to**: {assignable_to}{ if ai-agent: " (profile: " + agent_profile + ")" }
**Status**: {status}{ if owner: " — owner: " + owner }
**Dependencies**: {dependencies | "none"}
**Parallelisable**: {parallelisable}

#### Description

{description}

#### Preconditions

{preconditions as bullet list, or "none"}

#### Deliverables

{deliverables as bullet list}

#### Acceptance test

{acceptance_test.shell or acceptance_test.assertion, in a code block}

{ if runbook_hint: "**Runbook hint**: `" + runbook_hint + "`" }
```

## Validation rules

A task is *valid* when:

1. `id` matches `^FR-\d+-T-\d{2}$`
2. `description` is ≥ 200 characters (forces comprehensive scope)
3. `acceptance_test` has exactly one of `shell` or `assertion`
4. `sizing` is one of `S|M|L|XL`
5. `parallelisable` is consistent with `dependencies`: if dependencies is non-empty AND every dependency is in the same FR's task list, `parallelisable: false` is required during the dependency window
6. `agent_profile` is set iff `"ai-agent"` is in `assignable_to`
7. `estimated_hours` is set iff `"human"` is in `assignable_to`
8. Every dependency in `dependencies` resolves to a real task in the same FR or an earlier FR

## Lifecycle

```
draft  ─►  ready  ─►  in_progress  ─►  done
                                  └►  blocked
```

Transitions audit-logged via op:str_replace on the parent FR file.

## Pipeline interface

- **Input**: a `task@1` body that includes a `tasks:` list field.
- **Output**: same; tasks are embedded, not separate files.
- **External integration**: `cyberos proj sync FR-NNN` reads the task list, creates project-tracker tickets, writes back `linked_pr` + `status` updates.

## Versioning

- `subtask@1` is the current version (2026-05-12).
- Bumping required fields = `task@2` + MAJOR bump on every consumer.
- Adding optional fields = minor (no version bump required).

## Relationships to other contracts

- **`task@1`** — parent; gains a new `tasks:` list field as of 2026-05-12.
- **`tech_spec@1`** — sibling; `standard`/`full` profiles put work-package breakdown there instead. Body shapes converge over time.
- **`impl_plan@1`** — downstream; converts a task list into a flat ticket list ready for project-tracker creation.

## Examples

See `template.md` for a worked example carrying every field.
