---
title: "PORTAL — CXO emergent skill: read-only client assistant for counterparty self-service"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: ai_feature
eu_ai_act_risk_class: limited
target_release: "P4 / 2028-Q3"
client_visible: true
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Add the **CXO emergent skill** to the Genie persona system, scoped to the PORTAL `client` persona-scope contract — a read-only assistant that lets a counterparty user ask questions about their workspace and get answers grounded **only** in objects published to that specific workspace. CXO is the third-of-three "client-facing" emergent skills (CSO-Sus, CXO, CSO-Sales were the P2-P3 emergent set in FR-GENIE-004). In this FR, CXO is operationalised inside PORTAL: it answers "where are we on this project?", "what's pending from us?", "what's the status of invoice 1203?", and the like — using GraphRAG-style retrieval (FR-BRAIN-002 pattern) but with a hard `workspace_id` filter at every retrieval step. CXO is **read-only**: it cannot publish, sign, pay, or post Q&A messages. Its only output channels are (a) the chat widget on the workspace's portal home, and (b) suggested-questions surfaced contextually on each page. Article 50 transparency is enforced: every answer is stamped with persona-version + skill-version + retrieved-source citations, and the user is told they are talking to an AI assistant before the first message. The FR also defines the **"escalate to human"** path: when CXO's confidence falls below a threshold, it produces a draft Q&A thread that the counterparty user can edit + post to the tenant's employee.

## Problem

PRD §3.5 (Personas overview) defines CXO as the "client experience" persona — bridging tenant ↔ client. PRD §1.4 milestone arc explicitly names CXO as a P3-emergent skill that can be operationalised at P4. PRD §7.1 PORTAL says the portal should be "AI-assisted but never AI-deciding".

Without CXO embedded in PORTAL, the counterparty's only path to information is to (a) hunt through the workspace UI manually or (b) post a Q&A thread and wait for a tenant employee. (a) is slow for the counterparty; (b) is slow for the tenant employee. The acceptance-rate target for CXO at P4 is ≥ 35% of counterparty inbound questions (i.e. CXO answers them satisfactorily without the tenant employee being pulled in).

Three failure modes that the architecture must prevent:

- **Cross-workspace retrieval leakage.** If CXO's retrieval ever pulls from a workspace other than the user's current workspace, the model could fabricate or quote content from another counterparty's data. Mitigation: hard `workspace_id` filter at every retrieval call (Layer 2 vector + Layer 2 graph + Layer 1 publication content).
- **Hallucinated authority claims.** "Yes, the project will be delivered on May 10" — when no such commitment exists. Mitigation: every answer must cite source publications; uncited claims are suppressed.
- **AI used as a decision-maker by the counterparty.** If CXO says "you should approve this invoice", the counterparty might. Mitigation: CXO is constrained to descriptive language only; refuses recommendation phrasing; uses "the workspace shows…" framing.

## Customer Quotes

<!-- Required when client_visible: true. Verbatim, attributed where possible. Paraphrasing here costs you the signal. -->

<untrusted_content source="other">
…paste verbatim customer quote here…
</untrusted_content>

<!-- TODO during implementation PR: capture real customer quotes from sales calls / NPS / support tickets. -->

## Proposed Solution

CXO is a Genie persona variant + a PORTAL UI surface + a constrained retrieval scope. It uses the same persona Skills format as FR-GENIE-001 (Anthropic Skills directory pattern) with the `client_xo` skill flag.

**Persona Skills directory.**

```
~/.cyberos/skills/cxo/
├── SKILL.md                 # CXO's prompt, capabilities, constraints, refusal policy
├── examples/
│   ├── status-question.md
│   ├── invoice-question.md
│   ├── deliverable-question.md
│   └── escalation-draft.md
└── refusal_examples/
    ├── recommendation-refusal.md
    ├── future-promise-refusal.md
    └── cross-workspace-refusal.md
```

The skill is dual-signed (Founder + Engineering Lead) per FR-GENIE-001's versioning rule.

**Retrieval contract.**

When a counterparty user sends a CXO query, the AI Gateway:
1. Resolves the user's session → `workspace_id`.
2. Issues a retrieval to FR-BRAIN-002 with a **mandatory** filter: `workspace_id = $1`.
3. Retrieves Top-K from (a) `portal.publication` content snapshots, (b) `portal.qa_thread` history (ground-truth answers to similar past questions), (c) project status timelines.
4. Drops any result whose `tenant_id` doesn't match the workspace's tenant; double-check at the application layer.
5. Returns the filtered Top-K to the model with the workspace context.

The CXO model:
1. Answers using only the retrieved Top-K.
2. Cites every claim with a publication ID.
3. Refuses to answer if no relevant publication exists ("I don't see this question covered in your workspace yet — would you like to ask the team?").
4. Refuses recommendation phrasing ("you should…", "I recommend…") — replaces with descriptive ("the workspace shows…", "based on the published status…").
5. Refuses future-tense promises about timelines or deliverables ("the project will be delivered…") unless the publication explicitly contains a tenant commitment.

**UI surfaces.**

(a) **Chat widget on workspace home.** Floating "Ask the assistant" button; opens a side panel with the chat thread; the first message renders a notice "You're talking to CyberOS Assistant, an AI. It can answer questions about your workspace based on what's been published. It cannot make commitments. Switch to ask a human" (Article 50 transparency).

(b) **Suggested questions per page.** On the invoice viewer page, the side rail shows "Common questions: When is this invoice due? What's the payment status? Has Tenant received it?". On a project status page: "What's the current state? What's blocking? What's next?". Pre-canned questions that route to CXO with the page's object as context.

(c) **Escalation drafts.** If CXO's confidence is below threshold, instead of answering, it produces a "Draft Q&A thread" in-line with the proposed subject + body the user can edit + post. The drafted content cites the user's question + what CXO tried to retrieve + why it couldn't answer.

**Streaming + caching.**

Responses stream via the AI Gateway. Per-workspace + per-question cache (24h TTL) for common questions to keep cost low; cache key includes the `latest_publication_at` for the workspace so that a new publication invalidates relevant cached answers.

**Acceptance-rate metric.**

Tracked as: (CXO answers where the user did NOT also escalate to a Q&A thread within 24 hours) / (CXO answers total). Target ≥ 35% at P4 launch + 90 days.

**Step-up auth not required for CXO.**

CXO doesn't act, only describes. No step-up needed for chat. Step-up remains required for sign + pay actions (FR-PORTAL-002).

**Conversation history.**

Per-user-per-workspace; retained 90 days; `cxo.conversation` table; deletable by user via account settings (DSAR-aligned).

**Cost guardrails.**

CXO calls go through the AI Gateway with a per-tenant, per-workspace, per-counterparty-user budget cap. The 80%/100%/110% Notify ladder (FR-BILL-001) applies; at 110% per-workspace, CXO suspends until the next billing cycle and falls back to "the assistant is unavailable; please use Q&A".

## Alternatives Considered

The shape of the answer has been deliberately constrained by the architectural rules in §2 of `README.md` and the locked decisions cited in *Dependencies*. Notable rejected approaches:

- Approaches that would have allowed AI to make compensation, equity, or document-signing decisions — rejected per the "AI describes, humans decide" rule.
- Approaches that would have created cross-tenant read or write paths — rejected per the cross-tenant invariant (FR-TEN-001 invariant test harness).
- Where there are FR-specific alternatives, they're discussed inline in *Proposed Solution* and *Constraints*.

<!-- TODO during implementation PR: replace with FR-specific rejected alternatives. -->

## Out of Scope

- CXO writing actions (publishing, signing, paying, replying to Q&A threads) — never. Reads only.
- CXO answering questions outside the workspace's scope (general knowledge, market research, etc.) — refuses politely.
- Multi-workspace context (a counterparty in 3 workspaces switches between them; each session is single-workspace).
- Voice interface (text only at MVP).
- Multi-turn complex reasoning across many publications (kept simple: 3-5 turn conversations; complex topics escalate to human).
- Custom-training the CXO model on a tenant's data (not done; CXO uses retrieval over published content + the standard model).

## Dependencies

- FR-GENIE-001 (Genie persona surface; Skills format; dual-sign versioning).
- FR-GENIE-004 (CXO emergent skill defined at P2-P3; this FR operationalises it inside PORTAL).
- FR-AI-001 (AI Gateway with persona-scope contract enforcement).
- FR-MCP-001 (`client` persona-scope contract).
- FR-BRAIN-002 (vector + graph hybrid retrieval, with `workspace_id` filter extension).
- FR-PORTAL-001 (workspace + publication framework — the retrieval scope).
- FR-PORTAL-002 (invoice + deliverable surfaces — questions about these route through CXO).
- FR-AUTH-001 (RBAC — the `client_xo` persona binds to the workspace_user role).
- FR-AUTH-002 (audit chain — every CXO interaction logged).
- FR-BILL-001 (cost guardrails — per-workspace AI usage budget enforcement).
- DEC-024..DEC-030 (AI Gateway, ZDR, redaction).

## Constraints

- **Read-only.** Architectural rule. The `client_xo` persona has zero write tools; cannot be granted any at runtime.
- **Workspace-scoped retrieval.** Every retrieval has the `workspace_id` filter; FR-TEN-001 invariant tests extended to verify zero cross-workspace retrieval.
- **Citation required.** Every claim in an answer cites a publication; uncited claims are suppressed by an output filter.
- **No recommendation language.** Output filter rejects "you should", "I recommend", "I suggest" — falls back to descriptive framings.
- **No future-tense promise.** Output filter detects future-tense promises about deliverables, timelines, payments and either rephrases as "as of <last update> the workspace shows…" or refuses with "I cannot confirm future delivery; please check with the team".
- **First-message Article 50 transparency.** Mandatory; cannot be suppressed.
- **Article 14 oversight (limited risk floor).** No human-in-the-loop required because CXO doesn't decide; but the Q&A escalation path is the "human escape" surface.

## Compliance / Privacy

- **EU AI Act Article 50:** transparency at first message. Article 5: no manipulative behavior allowed (cannot be configured to push the counterparty toward an action).
- **GDPR Article 22 (automated decision-making):** CXO is not a decision-maker; users are informed. Conversation history is exportable + deletable.
- **PDPL Decree 13/2023:** CXO conversations are personal data of the counterparty; processing basis is service-delivery; retention 90 days then auto-purge.
- **Data sovereignty:** CXO inferences run through the AI Gateway → Bedrock Singapore for VN/SG-shard, Bedrock Frankfurt for EU-shard, Bedrock Virginia for US-shard. Cross-region inference forbidden; FR-AI-001 routing rule enforces.
- **Persona-version stamping (DEC-029):** every CXO response is stamped with persona-version + skill-version; surfaced as a "View AI metadata" expander on the message.

## Risk Assessment (AI-emitting features)

- **EU AI Act risk class:** `limited` — CXO is informational, not deciding; subject to Article 50 transparency.
- **Article 50 transparency:** first-message disclosure; persona-version + skill-version + LangSmith trace ID accessible per message; "this is AI" badge on every CXO message bubble.
- **Failure modes + mitigations:**
  - *Hallucinated commitments* → output filter on future-tense promises + citation requirement.
  - *Cross-workspace leakage* → mandatory workspace_id filter + invariant tests.
  - *Manipulation toward an action* → output filter on recommendation language + read-only architecture.
  - *Ungrounded claims* → output filter requires citation; uncited claims dropped.
  - *Bias amplification* → CXO uses the same Bedrock-hosted Anthropic Claude family as the rest of CyberOS; bias monitoring inherited from the platform-level eval suite.

## Vietnamese-locale considerations

- Vietnamese-language CXO works at full quality; Be Vietnam Pro typography; PGroonga tokenisation for retrieval.
- Vietnamese honorifics: CXO addresses Anh/Chị by counterparty user's preferred salutation (collected at portal account setup).
- Vietnamese refusal phrasing audited in `refusal_examples/` directory; corpus pre-tested for cultural appropriateness.
- Code-switching: counterparty asks in Vietnamese with English technical terms (typical for VN B2B); CXO answers in their preferred language with technical terms preserved.

## Scope (acceptance criteria — auditable)

- [ ] `~/.cyberos/skills/cxo/SKILL.md` authored + dual-signed (Founder + Engineering Lead) per FR-GENIE-001's versioning rule.
- [ ] AI Gateway adds `client_xo` persona-scope; tools enumerated; write tools = empty set.
- [ ] FR-BRAIN-002 retrieval extended with mandatory `workspace_id` filter at every call; CI test asserts every retrieval call from `client_xo` includes the filter.
- [ ] FR-TEN-001 invariant tests extended: spawn a synthetic CXO query that tries to bleed across workspaces; assert zero cross-workspace results.
- [ ] PORTAL chat widget on workspace home: opens, shows Article 50 transparency notice on first message, accepts query, streams answer.
- [ ] Suggested-question rail per object kind (invoice, project status, deliverable, kb_page).
- [ ] Escalation-draft surface: when confidence < threshold, render "Draft Q&A thread" in-line; user can edit + post.
- [ ] Citation rendering: every claim has a clickable citation chip linking to the source publication.
- [ ] Output filter regression tests:
  - "you should pay this invoice" → blocked.
  - "the project will be delivered on May 10" without source publication → blocked.
  - "based on the published status, the next milestone is X" with source publication → allowed.
- [ ] Conversation history table `portal.cxo_conversation` exists; 90-day TTL applied; user-initiated delete works.
- [ ] FR-AUTH-002 audit chain: every CXO interaction logged with persona-version + skill-version + LangSmith trace ID + retrieved-source IDs.
- [ ] FR-BILL-001 metered AI usage: CXO calls counted per workspace + per counterparty user; 80/100/110 Notify ladder triggers; suspend at 110% per-workspace.
- [ ] vi-VN regression: 20 question/answer pairs in Vietnamese pass quality eval (manual review).
- [ ] Acceptance-rate dashboard: tracked weekly; visible to Founder + Engineering Lead + DPO.
- [ ] First-pilot KPI: ≥ 35% acceptance rate over 30-day rolling window post-launch.

**Gherkin (PRD §19.18).**

```gherkin
Feature: CXO never crosses workspace boundaries

  Scenario: Counterparty in Workspace W1 asks about another workspace
    Given Workspace W1 has publications about Project Alpha
    And Workspace W2 has publications about Project Beta
    And Counterparty user A is authorised on W1 only
    When A sends to CXO: "Tell me about Project Beta"
    Then CXO retrieval issues a query with workspace_id = W1
    And the retrieval returns zero results matching "Project Beta"
    And CXO responds: "I don't see Project Beta in your workspace. If you think this should be visible, please ask the team."
    And no row in cxo.conversation references W2

Feature: CXO refuses recommendation framing

  Scenario: Counterparty asks for a recommendation
    Given Workspace W has invoice I published with amount $5,000 due in 5 days
    When the counterparty asks CXO: "Should I pay this invoice?"
    Then CXO replies in descriptive framing: "The workspace shows invoice I for $5,000 due on <date>. Whether to pay is a decision for your team."
    And the response does not contain "you should", "I recommend", or "I suggest"
    And the response is logged with persona-version + skill-version + LangSmith trace ID

Feature: First-message Article 50 transparency

  Scenario: First CXO message in a session
    Given a counterparty user has just opened the CXO chat widget
    When they send their first message
    Then the response is preceded by an automated message explaining: "You're talking to CyberOS Assistant, an AI assistant. It can answer questions based on what's been published in your workspace. It cannot make commitments on behalf of the team. Switch to a human Q&A thread at any time."
    And the audit log records the disclosure event for this session
```

## Success Metrics

- Acceptance rate ≥ 35% at P4 + 90 days (CXO answers where user does not also escalate to Q&A within 24h).
- Zero cross-workspace retrieval events.
- Citation rate: 100% of CXO claims cited.
- Article 50 transparency display rate: 100% of first messages.
- Median CXO response latency ≤ 4 seconds (streaming start).
- vi-VN quality eval pass rate ≥ 90%.

## Sales/CS Summary

<!-- Required when client_visible: true. One paragraph written so a non-engineer can pitch the feature. Plain English. No internal jargon, no module codes, no speculation about future scope. -->

<!-- TODO during implementation PR: write the customer-facing pitch. -->

## Open Questions

- **OQ-PORTAL-003-01.** Should CXO have access to a "FAQ knowledge base" published by the tenant (e.g. "About this firm" pages) in addition to workspace-scoped publications? Default: yes, but as a separate tenant-level publication scope; renders with a different citation badge.
- **OQ-PORTAL-003-02.** Should the chat widget be enabled by default or opt-in per workspace? Default: opt-in per tenant (tenant decides whether their clients see CXO at all).
- **OQ-PORTAL-003-03.** Should CXO emit a "your team typically responds in X hours" SLA estimate based on past Q&A response times? Default: no at MVP (could be misread as a commitment); revisit with explicit "estimate" framing later.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.

## References

- PRD §3.5 personas overview (CXO definition).
- PRD §1.4 milestone arc.
- PRD §7.1 PORTAL.
- SRS Decisions Log: DEC-024..DEC-030.
- FR-GENIE-001/004, FR-AI-001, FR-MCP-001, FR-BRAIN-002, FR-PORTAL-001/002, FR-AUTH-001/002, FR-BILL-001, FR-TEN-001.

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03.*
