---
artefact: architecture-decision-record@1
adr_id: ADR-001
task_id: TASK-IMP-058
status: accepted
created: 2026-07-25
dec_crosslinks: []
---
# ADR-001: Own chat from scratch (not Mattermost-forever)

## Context

Early chat delivery used a Mattermost fork path (TASK-CHAT-001 era). Running a
fork forever meant carrying upstream merge cost, plugin constraints, and a UX
that could not meet CyberOS identity / memory / Lumi requirements.

## Options considered

1. Stay on Mattermost fork indefinitely — rejected: permanent upstream tax; SSO
   and memory bridges stay second-class.
2. Buy/embed another chat SaaS — rejected: data residency and BRAIN coupling.
3. Own chat codebase (`services/` + `apps/web`) with import bridges from Slack /
   Zalo — CHOSEN.

## Decision

Build and operate native chat. Archived Mattermost tasks remain historical under
`docs/tasks/_archive/chat/`; active work uses the CHAT-1xx/2xx series.

## Consequences

- Chat is a first-party product surface, not a plugin host.
- Import/migration tasks (Slack/Zalo) are bridges into the native store.
- Agents must not re-propose "just stay on Mattermost" without a new ADR.
