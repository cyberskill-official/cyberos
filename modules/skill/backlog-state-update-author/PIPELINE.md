# `backlog-state-update-author` - pipeline

The ONLY governed write path to BACKLOG.md: status-cell rewrites (ship transitions) and audited row inserts (create), with optimistic concurrency.
Artefact: `backlog-state-update@2`. This document binds the skill into the ship chain; the step semantics live in SKILL.md.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| any ship phase completion / create-tasks step 3 | Default chain | mutation payload |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| backlog-state-update-audit | Default chain | artefact + pre/post images |

## HALT points

- Pre-image drifted (old_line no longer byte-matches) -> reject + re-read; repeated drift -> HALT (concurrent writer suspected).
- Nonstandard section headers in BACKLOG.md -> needs_human, never a guessed placement.

*Added by TASK-SKILL-118 (contract parity). Phases and step prose: SKILL.md is normative.*
