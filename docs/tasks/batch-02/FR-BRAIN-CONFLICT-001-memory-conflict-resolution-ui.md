---
title: "BRAIN — memory conflict detection and resolution UI (chooser, keep-both, supersedes lifecycle)"
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

Ship the memory conflict detection and resolution flow specified in PRD §5.6 across all three BRAIN layers. When two facts about the same `(subject_uri, predicate)` are added with different `object` values and both are `active`, both Layer 2 facts are marked `disputed` and surfaced in a **chooser UI**: the user can keep version A, keep version B, keep both as a disputed pair until further evidence, or merge the two into a new fact. When two devices write contradictory edits to the same Layer 1 file (semantic conflict beyond what Yjs CRDT auto-resolves at character level), the Layer 1 file enters a `crdt_divergence_pending: true` state and the same chooser UI surfaces the diff. The chooser is the first place the platform exercises the principle that **memory is human-curated, not LLM-curated**: the LLM proposes, the human disposes; every resolution writes an audit row in scope `brain.conflict.{tenant}`. The P0 → P1 exit gate (PRD §14.1.3) requires "the conflict-resolution UI has been used by ≥ 3 distinct employees" — this FR delivers that surface.

## Problem

A platform whose memory accumulates monotonically over multiple years inevitably accumulates contradictions: the founder writes "Acme's primary contact is Jane Doe (VP Engineering)" in February; in April Jane is promoted to CTO and a CHAT-derived fact says "Acme's primary contact is Jane Doe (CTO)"; in May an old email body re-extracts the February version. Without an explicit resolution flow, three things go wrong:

- **Retrieval ambiguity.** A CUO answer based on disputed facts may pick either, which means consecutive answers can contradict each other — the brand-trust failure mode for the persona.
- **Citation drift.** If both facts are `active`, citation correctness becomes ambiguous; if one is silently dropped, the audit log loses information.
- **Dogfooding regression.** The founder will not trust a memory he cannot audit and edit; the chooser is the founder's surface for keeping the substrate honest.

The PRD §5.6 text is explicit on the resolution semantics: detection raises the conflict, the UI presents both versions side-by-side, the user can pick one or keep both as a disputed pair, and the resolution is recorded. The S0-3 sprint risk-gate ("citation drift bug is sprint-blocking") is upstream of this FR; this FR is the surface that makes citation drift fixable rather than fatal.

## Proposed Solution

The shape of the answer is a small `brain-conflict` GraphQL surface inside the BRAIN module + a Genie-panel UI flow + the supersedes/disputed lifecycle on `brain.fact` + the `crdt_divergence_pending` flag on `brain.layer1_file`.

**Layer 2 conflict detection (already in FR-BRAIN-002).** The ingestion-time conflict detector is established by FR-BRAIN-002 §"Conflict detection": a new fact whose `(subject_uri, predicate)` matches an existing `active` fact with a different `object_uri | object_literal` is marked `status: 'disputed'`, added to the existing fact's `disputed_with[]` array, and an event is published on `cyberos.{tenant}.brain.conflict.detected`. This FR adds the resolution side.

**Layer 1 semantic conflict detection.** The Yjs CRDT (FR-BRAIN-001) merges character-level edits non-destructively, but two devices that rewrite the same Markdown section with semantically conflicting content produce a file whose content is "merged but wrong". The Rust binary detects this case by computing a per-section hash before and after merge and comparing it against the originating-device's expected hash; mismatches set `frontmatter.crdt_divergence_pending: true` on the file and emit `cyberos.{tenant}.brain.conflict.l1_diverged`. The file is read-only until resolved.

**Layer 1 ↔ Layer 2 cross-conflict.** If the user edits a Layer 1 file in a way that contradicts an existing `active` Layer 2 fact (the file says Jane is CTO; the fact says Jane is VP Engineering), the Layer 1 → Layer 2 ingestion produces a candidate UPDATE; if confidence is high the UPDATE applies and the prior fact is `superseded_by`; if confidence is low or both facts are equally well-evidenced, the fact is marked `disputed` against the file's derived fact and the chooser surfaces.

**The chooser UI.** A modal in the Genie panel "Memory" tab. The header shows the conflict subject ("Acme Corp · primary contact"). Two side-by-side cards show:

- The current (older) fact + its source citation (Layer 3 deep-link or Layer 1 path) + when it was added + by whom.
- The new (newer) fact + its source + timestamp + author.

Four resolution buttons:

1. **Keep the new fact.** The old fact's `status` becomes `'superseded'` with `superseded_by = <new-fact-id>`; the new fact's `status` becomes `'active'`. An audit row in scope `brain.conflict.{tenant}` records the resolution with both fact IDs and the chooser's identity.
2. **Keep the old fact.** Symmetric: the new fact becomes `'rejected'` and the old fact remains `'active'`.
3. **Keep both as disputed.** Both facts remain `status: 'disputed'`. CUO retrieval excludes disputed facts from default answers; if the user explicitly asks "what conflicting information do you have about Acme?", retrieval surfaces both. Use case: an unresolved business question where the answer genuinely depends on context.
4. **Merge into a new fact.** Opens a small editor pre-populated with both fact texts; the user authors a synthesised new fact; the synthesis becomes `'active'` and both originals become `'superseded'` with `superseded_by = <merged-fact-id>`. Used when the truth is "Jane was VP Eng until Feb 2026, then CTO from Mar 2026" — neither isolated fact captures it.

For Layer 1 file divergence, the chooser shows a Markdown diff (left = pre-divergence, right = post-divergence with the conflicting sections highlighted) and four buttons: keep left, keep right, keep both as a `disputed` annotation block inside the file, or merge in an inline editor.

**Disputed-pair badge.** Disputed facts and disputed Layer 1 files surface a small "⚠ disputed" chip wherever they appear in the UI (panel cards, citation chips, KB embeds). Clicking the chip opens the chooser. The Member can dismiss without resolving (defer); deferred conflicts stay in a "Memory · Pending" tab for the founder + DPO to triage during the weekly review.

**Auto-resolution heuristics (deliberately conservative).** Three cases auto-resolve without human intervention to keep the queue tractable:

1. **Strict overwrite.** The new write explicitly cites the old fact as the basis and supersedes it (the LLM extractor is prompted to recognise "Jane was promoted to CTO; previously she was VP Engineering" as a strict overwrite, producing an `UPDATE` not an `ADD`).
2. **Trivial restatement.** Two facts whose texts cosine-similarity-match above 0.97 are NOOP'd (same as FR-BRAIN-002 §"Operation classifier").
3. **Stale supersession.** A new high-confidence fact whose source occurred more than 90 days *after* the existing fact's source AND whose extractor produced an explicit `supersedes` flag auto-resolves with `status: 'superseded'` on the older. The 90-day floor is conservative so the chooser is not bypassed for anything actually contestable.

All other conflicts go to the chooser. The auto-resolution decisions are themselves audit-logged and surfaced in the "Memory · Recent activity" panel so the user can spot a bad call and reverse it.

**Bulk resolution.** When a re-extraction (FR-BRAIN-003 §"Re-extraction support") produces dozens of conflicts at once, the chooser surfaces a bulk view: all conflicts grouped by subject, with a "review pattern" mode that lets the user accept "always pick the new one" for the batch. The bulk-mode resolution still writes per-fact audit rows.

**MCP tool surface.**

- `cyberos.brain.list_conflicts(scope?, status: "open"|"resolved"|"all" = "open")` — read.
- `cyberos.brain.resolve_conflict(conflict_id, resolution: "keep_a"|"keep_b"|"keep_both"|"merge", merged_text?)` — `destructive: true; requires_confirmation: true`.
- `cyberos.brain.flag_disputed(fact_id, reason)` — `destructive: true; requires_confirmation: true` (a Member can manually flag a fact they believe to be wrong; opens a conflict against an empty alternative until evidence appears).

CUO suggests resolutions in Notify mode but never auto-resolves (per the persona-scope contract, `brain.resolve_conflict` is in the `tools_forbidden_explicit` list for the CUO/CEO and CUO/COO skills; only the human acts).

**Audit integration.** Every detection event, every chooser presentation, and every resolution writes an audit row in scope `brain.conflict.{tenant}` with: detection time, both fact IDs, both source refs, the chooser's subject ID, the chosen resolution, and the merged-text content (when applicable).

**Notification rules.** Conflicts are surfaced as Notify-mode cards in the Genie panel with confidence-derived urgency:
- High-confidence conflicts (both facts well-cited, fundamental disagreement): immediate Notify.
- Low-confidence conflicts (extractor confidence < 0.5 on one side): batched daily.
- Founder + DPO + the fact's original author are notified by default; other Members are notified only if the fact appears in their workspace.

## Alternatives Considered

- **No chooser; auto-resolve every conflict by "newer wins".** Rejected: the citation-drift property collapses. The PRD's "no answer without a citation" principle requires the human to be in the resolution loop for non-trivial cases.
- **Block all writes on a disputed subject until the conflict is resolved.** Rejected: too disruptive for daily operation; the disputed status is sufficient to exclude from default retrieval.
- **Three-way merge automation (LLM resolves, human reviews).** Considered for a future iteration. Rejected for P0 because the auto-resolution audit pipeline must be calibrated first; the LLM-merge mode is the natural extension once acceptance-rate metrics are stable. Tracked as OQ-BRAIN-LLM-MERGE.
- **Send all conflicts to the founder regardless.** Rejected: founder cognitive load (PRD §4.1 G8) is itself a metric we want to *reduce*; per-author-and-context routing is the floor.
- **Treat Layer 1 CRDT divergence as a non-event (Yjs auto-merges).** Rejected: character-level merge produces incoherent prose for semantic conflicts; pretending otherwise erodes trust in the substrate.

## Success Metrics

- **Primary metric.** P0 → P1 exit gate criterion (PRD §14.1.3): "the conflict-resolution UI has been used by ≥ 3 distinct employees" — measured by counting distinct `subject_id` values in `brain.conflict.{tenant}` audit rows with `action: 'resolve'` over the P0 window.
- **Quality metric.** Per-week conflict-detection precision: of the conflicts the auto-detector raised, what fraction were resolved (vs. dismissed as false-positive)? Target ≥ 70% by P0 exit. Below 50% means the detector is too sensitive and the threshold is loosened.
- **Latency metric.** Notify card latency from detection to panel render p95 ≤ 5 seconds.

## Scope

**In-scope (S0-5 and S0-6).**
- The chooser modal in the Genie panel "Memory" tab with all four resolution actions.
- The disputed-pair badge surface in panel cards, citation chips, and KB embeds.
- The Layer 1 file diff view with the four resolution actions.
- The bulk-resolution view for re-extraction-driven conflicts.
- The three auto-resolution heuristics (strict overwrite, trivial restatement, stale supersession ≥ 90 days), with audit-row coverage.
- The "Memory · Pending" review tab.
- The three MCP tools (list, resolve, flag).
- Notify-mode card rendering for conflict events.
- The founder + DPO + original-author notification rule.
- Audit integration in scope `brain.conflict.{tenant}`.

**Out-of-scope (deferred).**
- LLM-as-merger automation (OQ-BRAIN-LLM-MERGE).
- Cross-tenant conflict surfacing (forbidden by design).
- Conflict-trend dashboards beyond the simple acceptance metric (P1 OBS-002 surfaces them).
- Mobile UI for resolution (P3 mobile).

## Dependencies

- FR-INFRA-001 (host shell + Genie panel slot).
- FR-AUTH-001 / FR-AUTH-002 (RBAC + audit).
- FR-AI-001 (the merge-mode editor uses the AI Gateway for "suggest a synthesis" hint button).
- FR-MCP-001 (destructive-confirmation gate on resolve).
- FR-BRAIN-001 (Layer 1 divergence detection).
- FR-BRAIN-002 (Layer 2 conflict detection at ingestion).
- FR-GENIE-001 (Notify-card surface for conflict alerts).
- Compliance: PDPL Decree 13 (every resolution touches personal data; the audit trail is the control). EU AI Act Article 14 (human oversight: the chooser is the human-in-the-loop floor for memory mutations).
- Locked decisions referenced: DEC-056 (chooser UI shape), DEC-057 (auto-resolution heuristics 90-day floor).

## AI Risk Assessment

The chooser surface is the human-in-the-loop control for AI-driven memory mutations. EU AI Act risk class: `limited`.

### Data Sources

The chooser surfaces facts produced by the BRAIN ingestion extractor (FR-BRAIN-002). No third-party training data; per-tenant data only. The "suggest a synthesis" hint button calls the AI Gateway through the same persona-stamped path; the suggestion is shown but not auto-applied.

### Human Oversight

The entire feature *is* the oversight surface: every resolution requires a human action, every action is audit-logged, the auto-resolution heuristics are conservative and reviewable. The founder + DPO have visibility into deferred conflicts via the "Memory · Pending" tab; the weekly founder sync (PRD §3.5) reviews the queue.

### Failure Modes

- **Auto-resolution false positive.** A new fact wrongly auto-supersedes an old one. Mitigation: the audit row is reversible — the founder can run `cyberos.brain.resolve_conflict(<id>, "keep_a")` to flip the resolution; the prior `superseded_by` link is restored from the audit row.
- **Conflict storm after re-extraction.** 1000+ conflicts surface at once. Mitigation: bulk-resolution view + Notify-card batching prevent the Genie panel from being unusable.
- **Disputed-pair retrieval suppresses important context.** A user asks about Acme and the answer omits relevant disputed facts. Mitigation: the citation chip in CUO answers carries an "N disputed facts not shown — review" link.
- **Layer 1 divergence on a high-velocity file.** Mitigation: the file is read-only until resolved; on a synced device the user sees the lock immediately and the chooser surfaces.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted chooser UI flow, auto-resolution heuristics, MCP surface, failure-modes block.
- **Human review:** `@stephen-cheng` reviewed; product UX details to be re-walked by a designer when the role is hired (P1).
