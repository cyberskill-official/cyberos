---
memory_id: __MEM_ID__
scope: project:cyberos
classification: operational
authority: human-edited
version: 1
created_at: __ISO_TS__
created_by: subject:stephen-cheng
last_updated_at: __ISO_TS__
updated_by: subject:stephen-cheng
provenance:
  source: chat
  source_ref: cowork-session-2026-05-11:bundle-q-impl-files-and-close-pattern
  confidence: 1.0
consent:
  has_consent: true
  consent_event: null
  consent_scope: []
tags: [section-0-6, implementation-files, brain-vs-source-tree, gitignore, bundle-q, decision]
---

# DEC-109 — Implementation files live in project source tree, not in `.cyberos-memory/` (Bundle Q)

## Decision

The reference implementations of the protocol — `outputs/brain_writer.py` for §7.2/§4.4/§5.2, `cyberos/.protocol-signing-key` for §0.5, and any future tools that mutate the BRAIN — MUST live in the project source tree (versioned in git), NOT inside `.cyberos-memory/`. The BRAIN itself is local operational state and is gitignored on this project (per the user's expressed intent), so any tool placed inside the BRAIN ships only as long as the BRAIN persists. When the BRAIN is reinitialised, migrated to a new machine, or cloned to a co-worker, tools placed inside it are lost; the protocol they implement remains, but the implementation surface vanishes.

The canonical location is `outputs/brain_writer.py`. Alternative paths (e.g. `runtime/tools/cyberos_brain_writer.py`) are acceptable provided the §0.6 implementation-files registry is updated in the same protocol-upgrade.

## Why

Real-world trigger 2026-05-11: `brain_writer.py` was prescribed by 8 separate documents (CHAIN_ORCHESTRATOR, HOST_ADAPTERS, MANUAL_WORKFLOW, skills/CHANGELOG, AGENTS.CHANGELOG, AGENTS.README, AGENTS.md §0.6, PRD.CHANGELOG) as a tool the agent runs for every audit-row append. None of those docs caused the file to actually exist. It was never tracked in git because three of the prescriptions pointed at `.cyberos-memory/.brain_writer.py` (inside the gitignored BRAIN), one pointed at `outputs/brain_writer.py` (the correct location, which didn't exist either), and one pointed at "PRD §5.10.11" (a section that doesn't exist in any markdown file in the repo). Discovered when a Phase-1 BRAIN repair needed to append an audit row.

The fix is structural, not just a path rename: by mandating that implementation files live OUTSIDE the BRAIN, we ensure (1) git tracks them by default, (2) the next contributor / agent can find them, (3) reinitialising the BRAIN doesn't erase the writer, (4) the failure mode "writer is missing" surfaces as a regular CI / clone-time problem rather than a session-time mystery.

## How to use this memory

When asked to add a new tool that mutates the BRAIN (e.g., a verifier, a maintenance script, a migration helper), place it under `outputs/` or `runtime/tools/` and register it in AGENTS.md §0.6 line 175 in the same protocol-upgrade as the change that introduces it. NEVER place such tools under `.cyberos-memory/`.

## History

- 2026-05-11 — DEC-109 created as part of Bundle Q (sha transition `617f5aef…07759` → `71a276c7…3688`). Trigger: missing `brain_writer.py` discovered during cowork session.
