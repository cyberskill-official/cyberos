---
title: "KB — \"ask this page\" AI Q&A, GraphRAG cross-page navigation, promote-to-canonical suggestions"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Layer the AI-native KB features on top of FR-KB-001: **"ask this page"** Q&A in the page-side panel — a question-and-answer surface scoped to the current page, with answers cited back to specific blocks; **"ask the KB"** broader Q&A that fans out across spaces with relevance ranking; **GraphRAG cross-page navigation** powered by the Auto Dream community summaries from FR-BRAIN-002 ("Pages related to this one"); **promote-to-canonical** suggestions when CUO/CTO detects two or more pages converging on the same topic ("merge these into one canonical guide?"); **stale-page detection** (a page no Member has read or edited in 90 days that still cites old facts) with a Notify card to the page's primary author; and **draft assistance** (a "rewrite for clarity" or "expand this section" CUO action that surfaces a Review-mode draft). All AI surfaces operate on BRAIN-ingested page versions (FR-KB-001), render the EU AI Act Article 50 transparency disclosure chip, and respect the KB ACL — a Member's question can only retrieve from pages they have read access to.

## Problem

A KB without AI is a faster Notion. A KB with AI grounded in citations is what changes how the team uses knowledge — the founder asks "what's our policy on weekend on-call?" and gets an answer with a clickable citation to the canonical KB page in 5 seconds rather than searching for the page, opening it, and skimming.

PRD §9.12 commits to "AI Q&A grounded in cited KB pages; promote-to-canonical suggestions; GraphRAG-backed cross-page navigation". Without AI features, KB adoption stalls because the Notion-replacement bar is high and dry text-searchable docs do not move the needle.

Three failure modes a small team must avoid:

- **Hallucinated answers.** A Q&A surface that answers without citations defeats the purpose; the user cannot verify, the platform's trust is eroded.
- **Knowledge fragmentation despite ingestion.** Two pages cover overlapping topics with diverging recommendations; without merge/canonical detection, the divergence persists.
- **Stale-knowledge drift.** A page describing a 2024 process is still authoritative-by-default; without freshness signals, the team relies on out-of-date guidance.

## Proposed Solution

The shape of the answer is six AI surfaces layered on KB, all running through the AI Gateway (FR-AI-001) with the CUO/CTO skill (default for technical KB content) and CUO/COO (default for operational content), all citing BRAIN Layer 2 facts + Layer 3 raw blocks.

**1. Ask this page.**

Each KB page renders a small "Ask this page" input in the right-side panel. The input accepts a natural-language question:

1. CUO/CTO retrieval scoped to: this page's blocks (Layer 3 docs with `metadata.kb_page_id == <id>`); plus the BRAIN community summary for this page's community.
2. Cross-encoder rerank.
3. Answer generated with citations to specific blocks (clickable; scrolls the page to the block + highlights it for 3s).
4. The answer is *scoped*; it does not pull in unrelated KB pages or other modules.
5. If the answer requires content not on this page, the answer surfaces the limitation: "This page does not contain that information; would you like me to search across the KB?" (one-click escalation to "Ask the KB").

Latency budget: p95 ≤ 3 s end-to-end.

**2. Ask the KB.**

A broader Q&A surface accessible from the KB sidebar or the command palette (`Cmd-K → "Ask KB: ..."`):

1. Retrieval scoped to: KB-derived BRAIN Layer 2 facts + Layer 3 docs across spaces the asking Member can read.
2. Same rerank + answer pattern.
3. Citations span multiple pages; the answer's structure follows the underlying source distribution ("This is documented in three places; the canonical guide is X; supporting context is Y; the deprecated guide is Z").
4. The answer also surfaces "Pages most relevant to this question" as a list of clickable links.

Latency budget: p95 ≤ 5 s.

**3. GraphRAG cross-page navigation.**

A right-rail panel on every KB page shows **"Related pages"** computed by the Auto Dream nightly job (FR-BRAIN-002 §"GraphRAG community summaries"). Adjacent pages in the same Leiden community + the bridging pages between communities are surfaced; each item carries a one-sentence preview of its relevance to the current page.

Implementation:
- The nightly job extends to compute per-KB-page community membership.
- The right rail's GraphQL query: `kbRelatedPages(pageId: ID!): [KbRelatedPage!]!` reads `brain.kb_related_page` (a denormalised view).
- Each related-page card includes the relevance preview ("Both pages discuss Acme's deployment; this one focuses on database migrations whereas yours focuses on edge config").

Latency: < 100 ms (denormalised reads).

**4. Promote-to-canonical suggestions.**

The Auto Dream nightly job runs a duplicate-detection pass:
- For each KB page, find other pages whose BRAIN community + cosine-similarity-of-summaries exceed thresholds (e.g. cosine > 0.85 and shared community).
- If a cluster of ≥ 2 pages exceeds, surface a Notify-mode card to the primary authors: "Pages X, Y, Z look like they describe the same topic. Merge into one canonical guide?".
- The card has actions: "merge into X" (X becomes canonical; Y + Z become redirects with a deprecation banner), "keep separate (annotate)" (each page gets a "see also" cross-link), "ignore" (the suggestion is dismissed; not re-surfaced for 60 days).

Default acceptance threshold tuned conservatively to avoid noise; refined per-tenant by acceptance metrics.

**5. Stale-page detection.**

A weekly job at Sunday 22:00 ICT scans all `kb.page` rows:

- A page is **stale** if: `(now - last_read_at > 90 days) AND (now - updated_at > 180 days) AND (cited_facts_count > 0) AND ANY cited_fact has been superseded`.
- A stale page surfaces a Notify card to the primary author: "Page hasn't been touched in 6 months and cites superseded facts. Review?".
- The card actions: "Review and update", "Mark archived", "Confirm still current".

Stale-detection is informational; it never auto-archives or auto-edits.

**6. Draft assistance.**

Inside the editor, two slash commands:

- `/cuo rewrite for clarity` — selects current block or selection; CUO/CTO produces a Review-mode draft of an alternative wording with the same meaning, citing source. The author sees the draft side-by-side; accepts / edits / rejects.
- `/cuo expand this section` — selects current heading + content under it; CUO/CTO produces a Review-mode draft expanding with relevant detail from BRAIN. Same accept/edit/reject pattern.

Draft assistance is opt-in (slash command); it never modifies the page silently. The persona-version stamp + EU AI Act disclosure chip surface on every draft card.

**Permissions.**

All AI surfaces respect the KB ACL:
- A Member who cannot read page X cannot retrieve facts from page X.
- The retrieval pipeline filters at the BRAIN Layer 2 query level by inserting a Member-permitted-page-IDs predicate; this is the same pattern as PROJ's read-RLS (FR-PROJ-001).
- Cross-tenant retrieval is forbidden by design.

**MCP tool surface (extends FR-KB-001).**

- `cyberos.kb.ask_page(page_id, question)` — read; the "ask this page" tool. Returns the answer + citations.
- `cyberos.kb.ask_kb(question, space_ids?)` — read.
- `cyberos.kb.related_pages(page_id)` — read.
- `cyberos.kb.detect_duplicates(page_id?)` — read; founder + HR/Ops Lead for full-tenant scan; per-page for the page's primary author.
- `cyberos.kb.list_stale_pages(threshold_days)` — read.
- `cyberos.kb.draft_rewrite(block_id, intent: "clarity"|"expand"|"shorten")` — read; produces a draft, not a write.

CUO scope contracts: CUO/CTO + CUO/COO declare these tools in `tools_allowed`; mutation tools (write a page, publish, restore) remain in `tools_forbidden_explicit` — CUO drafts; the human commits.

## Alternatives Considered

- **Skip the page-scoped surface; only ship cross-KB Q&A.** Rejected: page-scoped is the most-used surface and avoids citation noise.
- **Auto-merge duplicate pages without human confirmation.** Rejected: too risky; merge is a destructive operation and the canonical-author intent is required.
- **Skip stale-page detection.** Rejected: knowledge drift is the silent killer of a wiki; stale signals are the floor.
- **Use a hosted Q&A service (e.g. Glean, Mendable).** Rejected: residency + lock-in + the platform's persona stamping cannot be enforced server-side.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate progress: ≥ 80% of "ask this page" answers cite a block on the same page; ≥ 70% of "ask the KB" answers cite at least one page; promote-to-canonical acceptance rate ≥ 30%; stale-page Notify acceptance rate ≥ 50%.
- **Quality metric.** Citation correctness ≥ 95% on a sampled review (founder + DPO; 30 questions per week).
- **Adoption metric.** ≥ 100 "ask this page" calls per week across the team by P1 → P2 exit (proves the surface is on the daily path).
- **Latency NFR.** "Ask this page" p95 ≤ 3 s; "Ask the KB" p95 ≤ 5 s; "Related pages" rail ≤ 200 ms.

## Scope

**In-scope.**
- The six AI surfaces.
- BRAIN Layer 2 + Layer 3 retrieval scoped by KB ACL.
- The right-rail "Related pages" panel powered by GraphRAG community summaries.
- The Auto Dream extensions for KB-page community membership and duplicate detection.
- Weekly stale-page detection job.
- The six MCP tools.
- Persona scope contract updates.
- Audit integration in scope `kb.ai.{tenant}`.
- OBS dashboard panels: ask-page acceptance, citation correctness, stale-page detection rate.

**Out-of-scope (deferred).**
- Multi-language Q&A across vi-VN ↔ en-US KB pages (P2 — translation surface).
- Auto-page-creation from CHAT or EMAIL ("Genie, save this thread as a KB page") — P2.
- Voice Q&A (P3 mobile).
- Per-Member personalised relevance ranking on "Related pages" (P3).

## Dependencies

- FR-KB-001 (schema + editor + BRAIN ingestion).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001.
- FR-BRAIN-001 / FR-BRAIN-002 / FR-BRAIN-003 (retrieval substrate + community summaries + raw block sources).
- FR-GENIE-001 / FR-GENIE-002 (Notify/Review modes; persona-scope).
- FR-OBS-001 / FR-OBS-002 (dashboards).
- Compliance: EU AI Act Article 50 (transparency on every AI surface); Article 14 (human oversight: drafts are Review-mode; merges are human-confirmed; stale flags are informational).
- Locked decisions referenced: DEC-131 (page-scoped + cross-KB Q&A as the two surfaces), DEC-132 (promote-to-canonical requires human confirmation), DEC-133 (stale detection is informational, never auto-archive).

## AI Risk Assessment

KB AI surfaces emit AI-derived content visible to natural persons. EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: KB pages + BRAIN data. No third-party. Retrieval is ACL-scoped.

### Human Oversight

- All Q&A surfaces show citations; users can verify.
- Promote-to-canonical suggestions are Notify-mode; merge requires explicit human action through `kbMergePages` mutation (introduced in FR-KB-003).
- Stale-page detection is informational.
- Draft assistance produces Review-mode drafts; the author commits.

### Failure Modes

- **Hallucinated citation.** Caught by the citation-correctness regression suite; persona-version regression blocks the PR.
- **Cross-ACL leak.** Mitigated by retrieval-time ACL filtering; a regression test attempts cross-ACL retrieval and asserts denial.
- **Duplicate-detection false positive.** Conservative thresholds + 60-day re-suggest cooldown; acceptance metrics tune the threshold.
- **Stale-detection on actively-correct pages.** Mitigation: "confirm still current" action + a 90-day silence on subsequent triggers.
- **Merge proposal cascades** (every duplicate proposes merging into another). Mitigation: cluster-level proposal (one Notify per cluster, not per pair).

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted six-surface architecture, retrieval scoping, persona scope contract, failure modes.
- **Human review:** `@stephen-cheng` reviewed; KB-specific eval cases authored at PR-review.
