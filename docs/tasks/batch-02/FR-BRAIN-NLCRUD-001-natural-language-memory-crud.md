---
title: "BRAIN — natural-language memory CRUD (\"forget that\", \"remember I prefer\", \"what do you know about Acme?\")"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P0 / 2026-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the natural-language memory CRUD path specified in PRD §5.7. Members create, read, update, and delete memories conversationally — "remember that Acme is on a 90-day payment cycle", "what do you know about Acme?", "forget the note about the 2025 retreat budget", "Jane was just promoted to CTO, update her record" — through the Genie panel and the same surface is exposed through MCP for any agent. The path is **LLM-authored, human-confirmed**: a small CRUD-extractor LLM call interprets the utterance, drafts the proposed Layer 1 file edit (FR-BRAIN-001) plus the Layer 2 fact change (FR-BRAIN-002), shows a diff in the Genie panel for human acceptance, and only writes after explicit confirmation. There is no auto-write path. This FR is the user-facing layer that makes BRAIN feel like a memory rather than a database — the property the dogfooding bet (PRD §2.3 Bet 4) and the founder-cognitive-load goal (PRD §4.1 G8) depend on.

## Problem

The PRD's memory architecture (Layer 1 Markdown filesystem, Layer 2 vector + graph facts, Layer 3 archival corpus) is correct but inert without a conversational surface. A founder who has to open a Markdown editor, navigate to the right file, write the YAML frontmatter, and save — every time he wants the platform to remember something — will not use the memory. The dogfooding bet collapses; CUO answers degrade because the substrate is empty.

The PRD §5.7 text specifies exactly the four conversational primitives: *create* ("remember that..."), *read* ("what do you know about..."), *update* ("Jane is now CTO"), *delete* ("forget that"). The matching MCP tools must exist so that an agent in Claude.ai or Cursor can drive the same path on the user's behalf — the agent-parity invariant (PRD §8.6).

The path is also where the platform earns the most trust in P0: every CRUD action proposes-then-confirms, never silently mutates. A Member who watches the platform mis-interpret his words once is unforgiving; a Member who watches it propose, get corrected, and write the right thing trusts it deeply.

## Proposed Solution

The shape of the answer is a `cyberos-brain-nlcrud` orchestrator (a small LangGraph state machine) plus the chat-like surface in the Genie panel "Memory" tab plus the MCP tools that mirror the same flow.

**The four primitives.**

**CREATE.** User says: "remember that Acme is on a 90-day payment cycle".

1. The orchestrator extracts intent via a Haiku 4.5 prompt: `{operation: "CREATE", subject: "Acme Corp", predicate: "has_payment_cycle", object: "90 days", proposed_layer1_file: "clients/acme-corp.md", proposed_layer1_section: "## Payment terms", proposed_text: "Payment cycle: 90 days."}`.
2. The orchestrator computes the proposed Layer 1 file diff (apply the new section if file exists; create the file with the canonical client-template if it does not) and the proposed Layer 2 fact `(subject: client:acme-corp, predicate: has_payment_cycle, object: "90 days", confidence: 0.95)`.
3. The Genie panel renders a confirmation card showing both diffs side-by-side: the Markdown patch on the left, the structured fact on the right, with citation provenance ("source: this conversation, 2026-05-03 09:14 ICT").
4. User clicks "save" → the writes commit; an audit row in scope `brain.nlcrud.{tenant}` records the utterance + extraction + writes.
5. User clicks "edit" → an inline editor lets the user revise the proposed text before save.
6. User clicks "discard" → nothing persists; an audit row records the discard.

**READ.** User says: "what do you know about Acme?".

1. The orchestrator extracts intent: `{operation: "READ", subject: "Acme Corp"}`.
2. The orchestrator runs a Layer 2 hybrid retrieval (FR-BRAIN-002) with `subject_uri: client:acme-corp`, gets the top-K active facts, fetches Layer 1 file contents for the related paths, and asks the AI Gateway (Sonnet 4.6) to synthesise a one-paragraph summary citing the facts.
3. The Genie panel renders the summary with citation chips. Every cited fact is clickable — opens the source paragraph in Layer 3 (FR-BRAIN-003) inline.
4. No write side-effect.

**UPDATE.** User says: "Jane is now CTO, update her record".

1. The orchestrator extracts: `{operation: "UPDATE", subject: "Jane Doe / Acme", predicate: "title", new_object: "CTO", supersedes_predicate_match: true}`.
2. The orchestrator finds existing facts with `subject_uri: person:jane-doe-acme, predicate: has_title` and the matching Layer 1 file `people/jane-doe-acme.md`.
3. Generates the proposed UPDATE: the existing fact is marked `superseded_by` the new fact; the Layer 1 file's `title:` frontmatter and the body's "Title:" line are patched; a `decisions/2026-05-03-jane-promoted.md` decision-log entry is appended.
4. Confirmation card shows the diff on Layer 1, the fact-supersedes diagram on Layer 2, and the new decision-log entry.
5. User confirms → all three writes commit atomically (one Postgres transaction wrapping the Layer 1 mirror + Layer 2 fact insert + Layer 1 file write + Yjs CRDT broadcast).

**DELETE.** User says: "forget the note about the 2025 retreat budget".

1. The orchestrator extracts: `{operation: "DELETE", subject_hint: "2025 retreat budget"}`.
2. Runs a retrieval to find candidate matches. If exactly one Layer 2 fact + one Layer 1 file section match, the orchestrator proposes the deletion. If multiple match, the panel shows a chooser ("which one?") before proceeding.
3. The proposed deletion: Layer 2 fact's `status` becomes `archived`; Layer 1 section is removed (the file is preserved with the section excised; the removed section is captured in the audit-row payload for reversibility); Layer 3 raw documents are *not* deleted (Layer 3 is the archival corpus and follows the RTBE flow from FR-CP-002 only when the user explicitly invokes erasure).
4. Confirmation card shows the deletion preview with a "purge from Layer 3 too" checkbox (off by default); checking it routes through the RTBE flow with its own multi-step confirmation.
5. User confirms → the writes commit.

**Cross-cutting properties.**

- **Confidence threshold.** If the extractor's confidence is below 0.7, the orchestrator surfaces a clarifying question instead of a write proposal: "I think you want to record Acme's payment cycle — is that right?" The user's reply is fed back through the orchestrator. Confidence < 0.4 prompts a free-text confirmation: "Could you rephrase what you'd like me to remember?"
- **Disambiguation.** When the subject reference is ambiguous (e.g. two Acmes exist), the panel asks "Acme Corp (long-term retainer) or Acme Beta (closed deal Q1)?". The user selects.
- **Source provenance.** Every CRUD write records the originating utterance verbatim in the audit row + the Layer 1 frontmatter `authors[]` (the human is the author; the LLM is recorded in `provenance.llm_assist_version`).
- **Cite-on-create.** A CREATE that contradicts an existing active fact triggers the conflict flow (FR-BRAIN-CONFLICT-001) rather than the simple confirmation card.
- **Per-Member preferences.** A Member can opt into "auto-confirm trivial creations" (a low-stakes preference: "remember I prefer Vietnamese in chat") in their account settings; auto-confirmed creations still write audit rows and are reversible from the panel's recent-activity tab.

**Genie panel surface.** A persistent text input at the bottom of the "Memory" tab. The top of the tab shows the most recent CRUD operations as a timeline (auto-refreshing) so the user can see what was just written and undo. Each operation has a one-click "undo" that triggers a reverse write through the same NLCRUD path with the audit-row's reversal payload.

**MCP tools.**

- `cyberos.brain.nlcrud_propose(utterance, layer_hint?)` — `read_only: false; destructive: false; idempotent: true`. Returns the proposed operation, diff, and confirmation token. Does not commit.
- `cyberos.brain.nlcrud_commit(confirmation_token)` — `destructive: true; requires_confirmation: true`. Commits a previously proposed operation. The agent UI must show the diff to the human before passing `client_confirmed: true`.
- `cyberos.brain.nlcrud_clarify(utterance, prior_question, answer)` — read; supports the clarification round-trip.
- `cyberos.brain.nlcrud_undo(audit_row_id)` — `destructive: true; requires_confirmation: true`. Reverses a prior committed operation.

The two-step propose-then-commit pattern is the architectural floor for the human-in-the-loop property; even if a future automation removes the visual confirmation card, the commit cannot proceed without the orchestrator producing a token that the caller obtained from `propose`.

**Persona-scope constraint.** The CUO/CEO + CUO/COO + CUO/CTO skills include `cyberos.brain.nlcrud_propose` and `cyberos.brain.nlcrud_clarify` (read-side) but **not** `cyberos.brain.nlcrud_commit` or `cyberos.brain.nlcrud_undo`. Only the human Member, calling through their own MCP client, can commit. CUO can suggest a memory write in Notify mode ("you said Jane was promoted — want me to update her record?") but the commit requires the human to click. This is the architectural enforcement of "LLM proposes, human disposes".

## Alternatives Considered

- **Auto-commit if confidence > 0.95.** Rejected: even at 0.95 the trust property breaks the first time the LLM mis-extracts a Member's name; recovery is more expensive than the always-confirm friction.
- **A separate command syntax instead of natural language ("/remember Acme · 90 days").** Rejected: the slash-syntax surface narrows usage to power users; the founder-cognitive-load metric depends on memory feeling like conversation.
- **Surface every proposed write as a CHAT-bot reply, no separate panel.** Rejected: the panel surface keeps memory operations adjacent to memory inspection (the timeline view); intermixing with channel chatter buries the recent-activity feed.
- **Allow CUO to commit when the user is offline.** Rejected: violates the agent-parity invariant; an agent and a human must operate under the same RBAC, and the commit role is human-only.

## Success Metrics

- **Primary metric.** S0-5 demo passes: the founder issues each of the four primitives via voice-to-text into the Genie panel; every operation surfaces a confirmation card, commits on click, and is reversible from the timeline within the same session.
- **Adoption metric.** ≥ 200 captured memories per active employee at P0 → P1 exit (PRD §14.1.3); ≥ 30% of those memories arrive through the NLCRUD path (the rest through CHAT/EMAIL/PROJ ingestion).
- **Quality metric.** Of operations the user discards, the discard rate is ≤ 15% on a 14-day rolling window — above that, the extractor prompt is revised.
- **Latency metric.** Propose response p95 ≤ 2.5 s end-to-end (extractor + retrieval + diff render); commit p95 ≤ 800 ms.

## Scope

**In-scope (S0-5 + S0-6).**
- The four primitives (CREATE, READ, UPDATE, DELETE) end-to-end.
- The Genie panel "Memory" tab with the input + timeline + recent-activity feed.
- The propose-confirm two-step flow with `confirmation_token`.
- Disambiguation and clarification round-trips.
- Atomic Layer 1 + Layer 2 + decisions-ledger writes.
- Per-Member auto-confirm preference for trivial creations (default off).
- Undo from the timeline (within 30 days; older entries require manual reverse-CRUD).
- The four MCP tools.
- Persona-scope contract excluding CUO from `nlcrud_commit` / `nlcrud_undo`.
- Audit integration in scope `brain.nlcrud.{tenant}`.

**Out-of-scope (deferred).**
- Voice-input native handling beyond the OS-level speech-to-text (P3 mobile handles native voice).
- Multi-step compound writes ("remember everything about today's Acme call as a meeting record") — handled by a separate `cyberos.brain.ingest_meeting_transcript` tool surfaced via the meetings module in P2.
- Cross-tenant memory federation (forbidden by design).
- LLM-suggested rewrites of pre-existing Layer 1 files for clarity (P2; tracked as OQ-BRAIN-LLM-REWRITE).

## Dependencies

- FR-INFRA-001 (host shell + panel slot).
- FR-AUTH-001 / FR-AUTH-002.
- FR-AI-001 (extractor and synthesis calls).
- FR-MCP-001 (destructive-confirmation gate; agent commit blocked without explicit `client_confirmed`).
- FR-BRAIN-001 (Layer 1 writes).
- FR-BRAIN-002 (Layer 2 fact ops).
- FR-BRAIN-003 (Layer 3 citation provenance for read responses).
- FR-BRAIN-CONFLICT-001 (CREATE-on-conflict routes here).
- FR-GENIE-001 (panel substrate; Notify-mode suggestions feed proposals here).
- Compliance: PDPL Decree 13 (every CRUD touches personal data; the consent posture is captured in the user's onboarding ToS plus the per-write audit row); EU AI Act Article 14 (the propose-confirm flow is the human-oversight enforcement).
- Locked decisions referenced: DEC-058 (LLM proposes, human disposes; `nlcrud_commit` is human-only).

## AI Risk Assessment

The NLCRUD path is the single most user-visible AI surface in P0 after CUO itself. EU AI Act risk class: `limited`.

### Data Sources

The extractor and synthesis prompts run through the AI Gateway (FR-AI-001) using per-tenant residency. Inputs are the user's utterance plus retrieval context from the same tenant's BRAIN. No third-party data, no cross-tenant leakage. The user's utterance is itself ingested into Layer 3 with the standard denylist (so a user accidentally typing a CCCD into the input is dropped at ingestion).

### Human Oversight

- Two-step propose-then-commit; no auto-write at confidence ≥ threshold.
- Confirmation card shows full diff before commit.
- Undo from timeline within 30 days.
- Persona-scope contract forbids CUO from committing — only the human can.
- Audit row captures the original utterance + extracted intent + diff + final state, so a misextraction is forensically recoverable.

### Failure Modes

- **Extractor mis-identifies subject.** Mitigation: low-confidence path triggers a clarifying question; user can correct; the audit-row records the correction so the next iteration of the prompt can include the case.
- **Disambiguation chooser shows wrong candidates.** Mitigation: the panel offers "none of these — let me describe more" as an explicit option that re-prompts the extractor with the additional context.
- **Atomic write partial-failure.** Mitigation: Postgres transaction wraps Layer 1 mirror + Layer 2 fact + decisions ledger; the Layer 1 file write happens *after* the transaction commits and is retried on filesystem failure with the Postgres state as the source of truth (the file is rebuilt from the table if the file write fails).
- **Conflicting CREATE without conflict-detector engagement.** Mitigation: the orchestrator always runs the conflict check before commit, even when the user's intent is CREATE (the check is what turns a CREATE into a conflict-flow if appropriate).
- **Undo of a write whose downstream effects have propagated.** Mitigation: the undo path replays the reverse audit-row payload; downstream effects (CUO answers grounded on the now-undone fact) re-run on next retrieval. The user is shown a "consequences may have already propagated" banner on undo.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted the four-primitive flows, propose-confirm pattern, extractor prompts shape, MCP surface, failure-modes block.
- **Human review:** `@stephen-cheng` reviewed; the extractor prompt's exact wording lands in PR-review with the Engineering Lead.
