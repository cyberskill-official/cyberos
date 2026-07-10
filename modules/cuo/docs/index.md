---
title: CUO · 47/48 personas · 221 workflows live · Phase 4 handlers shipped · CyberOS
source: website/docs/modules/cuo/index.html
migrated: FR-DOCS-002
---

**CUO is one persona** — Genie — backed by **the 47-persona workflow catalog** that load on demand. It is deliberately minimal: parse the query, score candidates, pick one, invoke through the Skill host, write the decision row to memory. The router is rule-based today (deterministic, sub-millisecond) and will layer in an LLM cascade at Phase 2 (the trade is latency for ambiguous queries). Every decision is itself an audit-chained memory, so every routing choice CUO ever made is replayable from disk. 

Strategic role

Agent orchestrator

Genie · 47 persona workflows · Lumi at P3+

Status

Phases 1–4 shipped

supervisor + 5 handlers

LoC (core)

~1,600

Python · supervisor + handlers

Tests

49/50

1 skip = catalog-completeness invariant

CLI subcommands

5

list-personas · list-workflows · route · dry-run · execute

Personas

47/48 with workflows

221 workflows · 1 EXTINCT

Lumi readiness

P3 unlock

tenant JWT + sync orchestrator

Routing latency p95

~ 0.4 ms

in-process · catalog cached

Confidence threshold

≥ 0.30

below → defer-to-human

Audit-chain coverage

100%

every decision → memory row

Depends on

memory · SKILL

\+ AI · MCP · AUTH (P2+)

Used by

All user-facing modules

CHAT · EMAIL · PROJ · …

★

## CUO Python supervisor + 6 Handler dispatchers

**Where we are.** The markdown-driven persona/workflow rebuild is complete and now sits under a Python supervisor with all four phases shipped (Phase 4 — the 6 Handler subclasses — landed on 2026-05-18). Every C-suite role from `../../../modules/cuo/README.md` §5 ships as `modules/cuo/<persona-slug>/` with a 9-block-schema README. Each persona's `workflows/<workflow>.md` declares a chain of SKILL module skills, and the supervisor auto-dispatches one of 6 Handler subclasses (Linear / TimeCritical / PerInstance / MultiOutput / SequentialApproval / PersonaPair) per the workflow's `pattern:` frontmatter. 

**48 persona folders, 47 with shipped workflows (chief-metaverse-officer EXTINCT) = 221 workflows live as of 2026-05-18** (post Tier-C1 depth additions across 14 priority personas). Persona first-coverage was closed Sessions D–N; Tier-C1 added 27 depth workflows across ceo/cfo/cto/chro/cso-sales/coo/cmo/ciso/cdo-data/cpo-product/chief-of-staff/cro-revenue/caio/cpo-privacy. Acronym collisions resolved by full `chief-{role}-officer` slug normalisation (2026-05-18). 

**Chief Technology Officer is the canonical reference persona** with fully-wired workflows: `architect-new-system` (10-step SRS→ADR→threat-model→SDD→impl-plan chain) · `adr-quick-capture` · `post-incident-review` · `deploy-readiness-review` · `threat-model-refresh`. Tests: **49/50 green** (one skip is the catalog-completeness invariant). The supervisor CLI has `list-personas · list-workflows · route · dry-run · execute` with flags `--explain · --no-handler-dispatch · --invoker {mock,subprocess,llm} · --memory-emit · --actor`. 

**Catalog steady-state.** `modules/cuo/docs/NEEDED_SKILLS.md` initially identified 66 author+audit pairs across Tiers 1-3. Sessions A-C shipped them. Sessions D-H surfaced 17 more across Tiers 4-7. All 83 skills shipped at 10/10. Sessions I-N shipped 124 niche-persona workflows with zero new skills — validating steady-state. Tier-C1 added depth workflows through the same stable catalog. Current SKILL catalog: **104 author+audit pairs / 208 bundles / 108 contracts** ; 221 workflows across 47/48 personas. 

See `modules/cuo/README.md` §4 for the canonical catalog; `modules/cuo/AGENTS.md` for protocol normativity; `modules/cuo/docs/ROUTING.md` for the two-stage (persona → workflow → skill chain) routing algorithm. 

0

## The bigger picture — three strategic roles

CUO is not "an AI chatbot in a corner." It is the **orchestrator that decides what skill runs, who authorised it, and where the receipt is stored**. Three roles converge in one Python package, and the design treats them as equal-weight requirements, not nice-to-have add-ons. 

Role 1 · Skill-routing memory

Natural-language in, audit-chained skill out

Every NL request to CyberOS that isn't a direct CRUD on a known entity passes through CUO. Parse (NFC-normalise) → score against the Skill catalog (rule-based today, LLM cascade at P2) → invoke the winner through the Skill host's capability broker → write the decision row to memory. Phase 1 is deterministic: same query + same catalog → same decision. The replay-equivalence rate is 100%. 

Role 2 · Persona catalogue

Genie + 47 C-suite persona workflows, agent-equal

The user sees one face — Genie — but the routing layer hands off to one of the 47 C-suite persona workflows (CEO, COO, CFO, CMO, CTO, CHRO, CRO-Revenue/CRO-Risk/CRO-Restructuring, CDO-Data/CDO-Digital/CDO-Diversity, CSO-Strategy/CSO-Sales/CSO-Security/CSO-Sustainability, CLO-Legal/CLO-Learning, CCO-Customer/CCO-Communications/CCO-Compliance/CCO-Commercial, CPO-People/CPO-Product/CPO-Privacy/CPO-Procurement, plus Chief-of-Staff, Chief-Architect, Chief-Ethics, Chief-Trust, Chief-Innovation, Chief-Knowledge, Chief-Automation, Chief-ESG, Chief-Brand, Chief-Transformation, Chief-Digital, CAO-Admin/CAO-Accounting, CIO-Information/CIO-Investment, CAIO, CISO, CGO, CXO, Chief-Medical, Chief-Remote, Chief-Happiness). Each persona carries its own keyword bank, voice contract, and defer-to-human matrix. Personas are _agent-equal members_ : same AUTH subject shape as human Members, same RBAC scope checks, same memory sync_class rules. The persona is a contract, not a chatbot skin. 

Role 3 · Lumi tenant identity

The org-tenant face that owns Lumi's memory sync

At P3+, every tenant's CUO instance is the local face of **Lumi** — the org-tenant persona that owns sync to Lumi's memory, cross-team synthesis, and tenant-aware policy enforcement. The AUTH JWT carries `agent_persona: cuo-cpo@0.4.1` \+ `tenant_id: org:<slug>`; Lumi's memory verifies and writes a chained audit row per sync push. The compounding moat is here: as more tenants feed Lumi, the cross-tenant synthesis improves the personas everyone uses. 

### CUO's place in the runtime — every user-facing module touches CUO

flowchart TB subgraph users["User surfaces"] CHAT["💬 CHAT @lumi mention"] EMAIL["✉ EMAIL inbound parse"] PROJ["📋 PROJ inline genie"] CRM["🤝 CRM next-action"] KB["📚 KB ask-the-docs"] TIME["⏱ TIME assist"] PORTAL["🚪 PORTAL client"] end CUO["🎯 CUO · agent orchestrator  
catalog · router · invoker · trace · memory bridge"] subgraph dnstream["Downstream"] SKILL["🛠 Skill host  
(capability broker)"] AIGW["⚡ AI Gateway  
(LLM cascade P2+)"] memory["🧠 memory  
(audit chain · sync_class gate)"] AUTH["🔐 AUTH  
(subject · scope_grants)"] LUMI["☁ Lumi's memory  
(P3+ cross-tenant)"] end CHAT --> CUO EMAIL --> CUO PROJ --> CUO CRM --> CUO KB --> CUO TIME --> CUO PORTAL --> CUO CUO --> SKILL CUO -. "ambiguous tail (≤ 0.50)".-> AIGW AIGW -. "ranked pick".-> CUO CUO --> memory CUO -. "JWT verify".-> AUTH memory -. "P3+ sync_class=shareable".-> LUMI classDef hub fill:#fef6e0,stroke:#9c750a,stroke-width:3px,color:#45210e classDef user fill:#e0e7ff,stroke:#3730a3 classDef ds fill:#f5ede6,stroke:#45210e classDef memory fill:#fef6e0,stroke:#9c750a class CUO hub class CHAT,EMAIL,PROJ,CRM,KB,TIME,PORTAL user class SKILL,AIGW,AUTH ds class memory,LUMI memory 

CUO sits between every user surface and every downstream system. The router is the single audit-emission point for "what did the AI decide on behalf of this Member?".

### Auto vs human-in-loop operations matrix

Operation| How it happens| Why this split  
---|---|---  
Routing decision (skill pick)| **Auto** when confidence ≥ 0.30| Deterministic at P1 + replay-equivalence 100%; LLM cascade at P2 still records full prompt + seed for replay.  
Skill invocation| **Auto** when scope_grants permit AND skill is non-destructive| Capability broker (§3.6) gates this; destructive ops require explicit confirmation regardless of confidence.  
Destructive invocation (purge / send-money / public post)| **Human confirm always**|  EU AI Act Art. 14 oversight + Anthropic-style human-in-loop policy on irreversible operations.  
Defer-to-human (low confidence)| **Auto-defer** when conf < 0.30 (Phase 1 binary) — show top 3 candidates so user can pick| The user picks; the pick becomes a training signal for future LLM cascade tuning. Phase 2 replaces this binary with a 4-tier cascade — see Flow 3 (≥ 0.70 auto · 0.50–0.70 ask · 0.10–0.50 LLM cascade · < 0.10 defer).  
Persona switch (Genie → CPO/COO)| **Auto-elide** in user-facing UI; **recorded** in audit row| The user shouldn't context-switch; the audit chain records which persona answered.  
memory audit row emission| **Auto** on every decision| 100% audit coverage is a protocol invariant; opt-out is at sync_class level, not per-decision.  
Lumi sync (P3+)| **Auto** when memory.sync_class ∈ {shareable, team-public, org-only}| Privacy floor is the sync_class; never auto-elevate from private/personal.  
Cross-persona handoff (multi-skill chain)| **Auto-chain** when `depends_on` is satisfied (P3+)| Topological walk through dependency DAG; each step emits its own audit row.  
  
1

## Why CUO exists

Internal operations is full of _obvious_ requests that should resolve in one round trip: "validate this MST", "generate a VAT invoice for ACME", "draft a 1:1 prep doc for Thursday with Hanh". Without an orchestrator, every such request goes through a 200ms LLM call, every catalog lookup costs another 200ms, and every audit fact has to be re-implemented in the calling skill. CUO is the single layer that absorbs that pattern: **rule-based fast path for unambiguous queries, LLM cascade for the ambiguous tail, memory-anchored audit for every decision**. 

⚡

Latency budget

90% of queries are obvious. Rule-based decisions resolve in < 1 ms. Only the ambiguous 10% need a Phase-2 LLM call.

📜

Replay & audit

Same query + same catalog → same decision (Phase 1). LLM Phase 2 logs full prompt + model + temperature so the audit row is still replay-sufficient.

🧭

Defer-to-human

Below the confidence threshold, CUO refuses to invoke. It surfaces the top three candidates and asks the user to choose — a hard EU AI Act Art. 26 oversight guarantee.

CUO is also the entry-point that lets the company hire **one** persona — Genie — and gradually grow the 47 C-suite specialist personas behind it without changing the user-facing interface. The same Slack thread / chat box that asked Genie to "validate MST 0123456789" yesterday can ask it to "draft the Q3 OKR cascade" tomorrow, and the right C-level persona workflow will load. 

2

## What it does — 5W1H2C5M

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is CUO?| An agentic orchestrator. Parses NL → scores catalog skills → invokes top match → records decision. The orchestrator state is _per-request_ ; CUO itself is stateless between requests.  
**5W · Who**|  Who interacts?| **Users:** every CyberSkill member via the Genie chat box. **Agents:** external Claude / Codex / Cursor sessions that hand off to CUO when they hit a CyberOS surface. **Owner:** CEO seat today; CXO seat at P3+.  
**5W · When**|  When is it invoked?| On every NL request to the platform that is not a direct CRUD on a known entity. Phase 1 is request-scoped; Phase 3 introduces multi-step chains that re-enter routing for each step.  
**5W · Where**|  Where does it run?| Co-located with the user (Tauri / CLI / IDE). LLM cascade calls the AI Gateway (LiteLLM). All decisions land in the same memory as the user's other memories.  
**5W · Why**|  Why this design?| Because the alternative is asking the user to know which skill to call, or shipping an enormous monolithic prompt that re-derives the catalog every turn. CUO is the cheapest layer that gives Genie a real, replayable decision policy.  
**1H · How**|  How does it route?| Score per candidate = `5.0 if skill_name in query else 0 + 3.0 × keyword_hits + 2.0 if VN-diacritic AND skill.region=VN else 0`. Top scorer wins if score > 3.0 (confidence ≥ 0.30); otherwise emit `routed:false`.  
**2C · Cost**|  Cost per decision?| Phase 1 ≈ 0.4 ms per route (in-process). Phase 2 LLM cascade: only when ≤ 0.50 confidence AND ≥ 0.10 candidate exists; typical 150 ms. Memory: 1 row per decision in memory, ~700 bytes.  
**2C · Constraints**|  Constraints?| (a) Phase 1 MUST be deterministic — same query + same catalog → same decision. (b) Below threshold MUST defer-to-human; no auto-invoke. (c) Skill capabilities MUST be respected via Skill host's broker; CUO cannot bypass.  
**5M · Materials**|  What does it use?| The Skill catalog (read from disk in sorted-path order), a per-skill keyword bank in `cuo/core/router.py`, optional memory context for Phase 2 LLM cascade, and the AI Gateway for inference.  
**5M · Methods**|  Method choices?| Rule-based scoring (Phase 1), LLM cascade (Phase 2 LangGraph + LiteLLM), topological chain walking (Phase 3), per-persona keyword bank (Phase 4). Each layered on top of the previous, none replacing it.  
**5M · Machines**|  Where does it run?| Locally with the user (Tauri host) for the rule-based path; AI Gateway for LLM calls. Postgres checkpointer for LangGraph state (Phase 2+) — required for EU AI Act Art. 12 logging.  
**5M · Manpower**|  Who maintains?| 1 IC owner today. By P1 exit the CPO and CTO co-own the keyword bank + persona definitions. At P3 the dedicated CXO seat appears.  
**5M · Measurement**|  How measured?| Routing confidence distribution, escalation rate, decision latency, defer-to-human rate. KPIs in §13.  
  
3

## Architecture

Six modules in `cuo/cuo/core/` form the entire surface. Catalog discovers skills off disk. Router scores. Invoker delegates to the Skill module. Memory-bridge writes the decision. Trace renders the structured row. 

graph TB subgraph CLIENTS ["Clients"] USER["User · Tauri / CLI"] AGENT["External agent  
(Claude / Codex / Cursor)"] end subgraph CUO ["CUO router (cuo/core/)"] PARSE["parse  
NFC-normalise query"] CATALOG["catalog.py  
read Skill manifests"] ROUTER["router.py  
score & pick"] EXTRACT["arg extractors  
(per-skill, pure)"] INVOKER["invoker.py  
shell out to Skill"] TRACE["trace.py  
structured row"] BRIDGE["memory_bridge.py  
write to memory"] end subgraph DOWNSTREAM ["Downstream"] SKILL["🛠 Skill host  
(Rust cyberos-skill-cli)"] AI["⚡ AI Gateway  
(Phase 2 LLM cascade)"] memory["🧠 memory  
(audit chain)"] end USER --> PARSE AGENT --> PARSE PARSE --> CATALOG CATALOG --> ROUTER ROUTER --> EXTRACT ROUTER -. "ambiguous (≤ 0.50)".-> AI AI -. "ranked pick".-> ROUTER ROUTER --> INVOKER INVOKER --> SKILL SKILL -. "stdout / stderr / exit".-> INVOKER INVOKER --> TRACE TRACE --> BRIDGE BRIDGE --> memory classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#fef6e0,stroke:#9c750a class PARSE,CATALOG,ROUTER,EXTRACT,INVOKER,TRACE,BRIDGE,SKILL,memory shipped class AI planned 

### Internal components

Component| File| Responsibility  
---|---|---  
`catalog.py`| core/catalog.py| Discover Skill manifests off disk in sorted-path order. Cached per-request.  
`router.py`| core/router.py| Phase 1 rule-based scorer. Per-skill `_KEYWORD_BANK` \+ `ARG_EXTRACTORS`. Returns `(decision, alternatives)`.  
`invoker.py`| core/invoker.py| Shell out to `cyberos-skill-cli run <skill>`. Capture stdout / stderr / exit-code. No in-process skill execution.  
`trace.py`| core/trace.py| Render structured trace row: query, decision, alternatives, result, timestamps.  
`memory_bridge.py`| core/memory_bridge.py| Write trace row to memory. Phase 1: flat file under `meta/cuo-decisions/<ts_ns>.md`. Phase 2: through canonical Writer.  
  
### Phase roadmap

Phase| What changes| Why| Status  
---|---|---|---  
**Phase 1**|  Filesystem catalog scanner · two-stage route (persona → workflow) · memory dry-run| Markdown-driven, deterministic; no LLM dependency for routing| shipped  
**Phase 2**|  Pluggable Invoker (Mock / Subprocess) · `execute_chain()` walks workflow with filesystem hand-off| End-to-end chain execution against SKILL CLI subprocess| shipped  
**Phase 3**|  LLMInvoker (mock-llm + Anthropic Messages API) · memory audit-chain emission per step + per chain| LLM-driven authoring with full audit trail through Writer| shipped  
**Phase 4**|  6 Handler subclasses dispatched by workflow `pattern:` frontmatter (Linear · TimeCritical · PerInstance · MultiOutput · SequentialApproval · PersonaPair) · 8 new memory audit kinds| Heterogeneous workflow types — incidents, breach response, per-customer loops, multi-output writes, approval gates, cross-persona handoffs| shipped  
  
3.5

## Lumi identity wrapper — local CUO ↔ org-tenant persona

At P3+, every tenant's local CUO instance is the face of **Lumi** — the org-tenant persona whose JWT is verified by AUTH, whose audit rows feed Lumi's memory, and whose cross-tenant synthesis is the long-game compounding moat. This section locks the contract between local CUO (a stateless Python orchestrator) and Lumi (the org-tenant identity it acts under). 

### Lumi vs Genie vs local CUO — the three names

Name| Scope| Who sees it| What it signs / writes  
---|---|---|---  
**Genie**|  The user-facing mascot. One face.| Members in the SPA, chat box, IDE plugin.| Nothing directly — Genie is the UX skin. Behind Genie sits CUO + persona.  
**Local CUO**|  The in-process Python orchestrator on the user's machine / tenant Fargate task.| Engineers, audit consumers.| memory rows under `meta/cuo-decisions/<ts_ns>.md`; signed locally; never escapes the local store.  
**Lumi**|  The org-tenant persona at P3+ — the identity under which CUO acts when the work crosses team boundaries or syncs to the cloud memory.| Other tenants (cross-team synthesis), Anthropic-style transparency reports, compliance auditors.| memory sync push to Lumi's memory; the AUTH JWT carries `agent_persona` \+ `tenant_id` \+ `scope_grants`; Lumi's memory writes a chained audit row per sync push.  
  
The same Python process is "local CUO" on disk and "Lumi" over the wire. The naming is intentional: Genie is for the user, CUO is for the engineer, Lumi is for the audit chain that crosses tenant boundaries.

### AUTH JWT shape for Lumi (per AUTH §2.7)
    
    
    {
      "iss": "https://auth.cyberos.io/<tenant>",
      "sub": "agent:cuo@cyberskill.world",
      "aud": "lumi-memory.cyberos.io",
      "iat": 1763112131,
      "exp": 1763115731,
      "tenant_id": "org:cyberskill",
      "tenant_residency": "sg-1",
      "agent_persona": "cuo-cpo@0.4.1",
      "scope_grants": [
        {"resource": "memory", "actions": ["read", "write"], "sync_class_max": "shareable"},
        {"resource": "skill",  "actions": ["invoke"], "skills": ["product-requirements-document-author@0.4.1", "vietnam-mst-validate@0.2.0"]},
        {"resource": "proj",   "actions": ["read", "write"], "engagements": ["acme-q3-platform-build"]}
      ],
      "jti": "01HZK…"
    }

The `agent_persona` claim is the cryptographic anchor: every audit row Lumi writes to memory can be verified back to this exact JWT (and its issuer key in JWKS).

### Cross-tenant synthesis — what Lumi actually _does_ at P3+

Lumi runs on Lumi's memory (the cloud-hosted org-tenant store). Per tenant, it ingests `sync_class ∈ {shareable, team-public, org-only}` memories from each Member's personal memory and from CUO's decision rows. The compounding moat comes from one fact: **the cross-team synthesis improves the personas everyone uses**. When the CPO persona gets better at product-brief authoring because Lumi has seen 200 product briefs across 50 tenants, every tenant's CPO improves. The synthesis is bounded by sync_class — never private/personal — and by jurisdictional residency (Lumi's memory shards by region). 

Synthesis output| Source| Cadence| Privacy floor  
---|---|---|---  
Updated persona prompts (CPO/COO/etc.)| cross-tenant CUO decisions tagged `persona_handoff` with sync_class=shareable| Monthly| shareable — never private  
Updated keyword banks| cross-tenant query → decision pairs that escalated to LLM cascade (i.e. the "ambiguous tail" we keep missing)| Weekly| org-only or shareable  
Cross-tenant lessons (curated)| tenant-published "what we learned" memories with sync_class=team-public| Quarterly| team-public + tenant opt-in  
Vertical-pack updates (cyberskill-vn / sg / id / …)| jurisdiction-tagged skill performance data + regulatory drift signals| Quarterly| shareable + jurisdiction-pinned  
  
3.6

## Skill broker contract — capability-gate at every invocation

CUO does not invoke skills directly. Every invocation passes through the Skill host's **capability broker** — the same mechanism the Skill module page §3.5 documents. The broker is the protocol-level guarantee that a skill never gets a tool it didn't request and a Member never invokes a skill outside their scope_grants. This section locks the contract from CUO's side. 

### The seven-step broker flow (per invocation)

sequenceDiagram autonumber participant U as Member · "validate MST 0123…" participant C as 🎯 CUO router participant CAT as Catalog participant BR as 🛠 Skill broker participant AU as 🔐 AUTH (scope check) participant BA as 🧠 memory audit pre-write participant SK as Skill executable participant BP as 🧠 memory audit post-write U->>C: NL query C->>CAT: load skill catalog (sorted) CAT-->>C: 208 skills (104 author+audit pairs) C->>C: score · pick (conf=0.7) C->>BR: invoke(skill, args, agent_persona, subject) BR->>AU: scope_grants.invoke include this skill? AU-->>BR: allow BR->>BA: write pre-invocation audit row  
(seq=N, op=put, path=…/cuo-decisions/…) BR->>SK: spawn subprocess · pass only requested tools SK-->>BR: stdout / stderr / exit_code BR->>BP: write post-invocation audit row  
(chain link to pre row + result hash) BR-->>C: result C-->>U: rendered response (Genie skin) 

Steps 7 + 11 are the dual audit rows that make every CUO invocation cryptographically reconstructible: pre-row proves intent, post-row proves outcome, both chain back to the user's memory.

### CUO ↔ Skill broker contract (locked)

Contract item| Who enforces| Failure mode  
---|---|---  
Skill must be in catalog at decision time| CUO router (catalog fingerprint locked in audit row)| Skill removed mid-invocation → broker rejects · CUO emits `routed:false reason:catalog_drift`.  
Subject's scope_grants must include this skill| AUTH (in-broker check)| Permission denied → broker rejects · audit row emitted with `denied:true reason:scope`.  
Skill's `allowed_tools` must be respected| Skill broker subprocess sandbox| Skill tries to call a non-allowed tool → broker kills subprocess · escalation to CSO.  
Destructive op (purge, send-money, public-post) requires explicit human confirm| Broker + CUO defer-to-human matrix| Skill flagged `destructive=true` \+ no confirm token → broker rejects.  
Pre + post audit rows on every invocation| Skill broker (memory Writer)| memory write fails → broker rejects invocation start; never partial-invoke.  
Tenant isolation (subject.tenant_id matches skill scope)| AUTH + RLS| Cross-tenant invocation attempt → broker rejects · CSO alert.  
Skill version pinning (no auto-upgrade mid-decision)| CUO catalog fingerprint includes version hash| Skill upgraded after route, before invoke → catalog fingerprint mismatch → re-route required.  
  
### Defer-to-human matrix (per persona)

Each C-level persona declares a defer-to-human matrix. The matrix is normative; CUO refuses to auto-invoke any operation in the "defers" column regardless of confidence. 

Persona| Auto-OK| Defers to human  
---|---|---  
**CEO**|  strategy summary · briefing draft · OKR cascade draft| external announcement · fund-raise communication · board email  
**COO**|  cycle review draft · process doc · runbook scaffold| incident postmortem publish · vendor cancel · org-wide policy push  
**CFO**|  variance commentary · journal entry draft · reconciliation| payment send · investment trade · payroll release  
**CMO**|  blog draft · campaign plan · social post draft| public post send · paid-ads launch · press release send  
**CTO**|  code review draft · architecture ADR draft · runbook| prod deploy · migration apply · access grant  
**CHRO**|  policy lookup · interview-plan draft · onboarding checklist| offer send · termination · comp adjustment  
**CSO**|  risk register entry · vulnerability summary| access revoke · incident escalation send · firewall change  
**CLO**|  contract redline · NDA triage · compliance check| contract sign · litigation response · regulator filing  
**CDO**|  SQL draft · dashboard scaffold · stats summary| prod query execute · data export to third party · model train trigger  
**CPO**|  product-requirements-document-author draft · product-requirements-document-audit · roadmap update draft| roadmap publish · public roadmap send · spec freeze announce  
  
3.7

## Cross-module CUO surfaces — where Genie appears

Genie is not a separate app. Every user-facing module exposes a CUO surface — chat box, command palette, inline action — that routes through the same orchestrator. The table below is the canonical list of surfaces; each surface declares how it parses NL, what context it ships to CUO, and what UI affordance it returns. 

Module| Surface| Trigger| Context shipped to CUO| Affordance returned  
---|---|---|---|---  
**💬 CHAT**|  @lumi mention in any channel| parser detects `@lumi` at start of message| last N messages of thread (sync_class permitting) + author subject + channel id| inline reply card with skill name + confidence + result; "expand" reveals audit row  
**✉ EMAIL**|  "Genie: …" subject prefix OR sidebar action| inbound mail with subject pattern; or user clicks "Ask Genie" sidebar| email thread (subject + body) + recipient context| draft reply (never auto-sent); appended to thread as a draft  
**📋 PROJ**|  inline issue genie + cycle-review draft| user clicks the genie icon on an Issue; or cycle close trigger| issue (+ comments + history) or cycle data| suggestion card (status change, label, link memory) — Member confirms; AM accepts review draft  
**🤝 CRM**|  "What's next on this deal?" action| user opens a Deal record| deal stage history + last contact + linked account| ranked next-actions; one-click create draft (email / meeting / task)  
**📚 KB**|  "Ask the docs" search box| user types a question| search context + permissions| cited answer with source-doc links; never claims certainty above the source corpus  
**⏱ TIME**|  "Where did my hours go this week?" prompt| weekly Friday auto-prompt| user's time entries for the week| summary + flagged anomalies (unallocated, suspicious overrides)  
**🧾 INV**|  "Review this invoice" pre-send check| AM clicks "Pre-send check" on a draft invoice| invoice draft + linked engagement + rate card snapshot| diff vs prior cycle; flagged discrepancies; one-click apply suggested fix  
**🚪 PORTAL (P2+)**|  Client-facing Genie (read-only persona)| client logs in, asks a question| client's accessible Project view only| scoped answer — never reveals private engagement data; sync_class=client-visible enforced  
**👁 OBS**|  "Explain this alert" action| SRE clicks an alert in OBS dashboard| alert metadata + relevant runbook hits| structured triage: probable cause, runbook link, suggested first step  
  
### Per-surface latency budgets

Surface| Route latency p95| Total response p95| Note  
---|---|---|---  
CHAT @lumi| ≤ 10 ms (routing only)| ≤ 4 s (incl. LLM if cascade)| User expects chat-app responsiveness; first-token streamed when LLM engaged.  
EMAIL Genie| ≤ 50 ms| ≤ 15 s (draft generation)| Async-acceptable; user sees "drafting…" indicator.  
PROJ inline| ≤ 5 ms| ≤ 800 ms| Must feel like sync-engine; if > 800 ms, fall back to "Genie is thinking…" inline indicator.  
CRM next-action| ≤ 5 ms| ≤ 2 s| Sales reps lose attention past 2 s — measured target.  
KB ask-the-docs| ≤ 10 ms| ≤ 6 s (RAG over corpus)| Streamed answer; first chunk < 1.5 s.  
OBS triage| ≤ 50 ms| ≤ 3 s| SRE under stress; must be fast; never the bottleneck during an incident.  
  
4

## Data model

CUO owns three entities. A `SkillEntry` is the projection of a Skill manifest into the router's catalog. A `RoutingDecision` is the result of scoring. An `InvocationResult` captures what the Skill host actually did. 

erDiagram CATALOG ||--o{ SKILL_ENTRY: "contains" REQUEST ||--|| ROUTING_DECISION: "produces" ROUTING_DECISION ||--o{ ALTERNATIVE_CANDIDATE: "ranks" ROUTING_DECISION ||--o| INVOCATION_RESULT: "invokes (optional)" INVOCATION_RESULT ||--|| TRACE_ROW: "emits" TRACE_ROW ||--|| MEMORY_AUDIT_ROW: "persisted as" ROUTING_DECISION ||--o{ CHAIN_STEP: "Phase 3: chained calls" CATALOG { string fingerprint PK "sha256 of catalog snapshot" int64 scanned_at_ns int skill_count } SKILL_ENTRY { string name PK string version string description string region "VN | global" string keywords string depends_on "Phase 3" string allowed_tools } REQUEST { string request_id PK string query string actor int64 ts_ns string persona "CEO|COO|...|null" } ROUTING_DECISION { string request_id FK string skill_name "null if routed=false" obj arguments float confidence "0.0–1.0" string rationale bool routed string router_phase "phase1 | phase2" } ALTERNATIVE_CANDIDATE { string request_id FK string skill_name float score int rank } INVOCATION_RESULT { string request_id FK int exit_code string stdout string stderr int64 started_at_ns int64 ended_at_ns } CHAIN_STEP { string request_id FK int step_index string skill_name obj arguments string status "ok | failed | skipped" } TRACE_ROW { string trace_id PK obj decision obj result obj chain "Phase 3: list of CHAIN_STEP" } MEMORY_AUDIT_ROW { int64 seq PK string path "meta/cuo-decisions/<ts_ns>.md" string body_hash string chain } 

5

## API surface

### GraphQL subgraph (planned · P0+)
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key"])
    
    type RoutingDecision @key(fields: "requestId") {
     requestId: ID!
     query: String!
     actor: String!
     skillName: String # null when routed=false
     arguments: JSON
     confidence: Float!
     rationale: String!
     alternatives: [Candidate!]!
     routed: Boolean!
     routerPhase: RouterPhase!
     invokedAt: DateTime
     result: InvocationResult # null if --invoke not requested
    }
    
    type Candidate {
     skillName: String!
     score: Float!
     rank: Int!
    }
    
    type InvocationResult {
     exitCode: Int!
     stdout: String!
     stderr: String!
     startedAt: DateTime!
     endedAt: DateTime!
    }
    
    enum RouterPhase { phase1_rule phase2_llm phase3_chain phase4_persona }
    
    type Query {
     route(query: String!, persona: Persona): RoutingPreview!
     decision(requestId: ID!): RoutingDecision
     decisions(actor: String, since: DateTime, limit: Int = 50): [RoutingDecision!]!
    }
    
    type Mutation {
     routeAndInvoke(query: String!, record: Boolean = true): RoutingDecision!
     invokeSkill(skillName: String!, arguments: JSON!): InvocationResult!
    }
    
    type RoutingPreview {
     decision: RoutingDecision!
     catalogFingerprint: String! # for replay
    }

### MCP tool catalogue

Tool name| Inputs| Outputs| Annotations  
---|---|---|---  
`cuo.route`| query, persona?| `RoutingDecision`| readonly · pure · scope=route  
`cuo.route_and_invoke`| query, record=true| `RoutingDecision + result`| destructive=true · scope=invoke  
`cuo.catalog`| —| `SkillEntry`| readonly · cached · scope=read  
`cuo.explain`| requestId| `rationale + alternatives`| readonly · scope=audit  
  
### CLI — `cyberos-cuo` (5 subcommands shipped)

Subcommand| Purpose| Example  
---|---|---  
`list-personas`| Enumerate the 48 persona folders (47 with workflows)| `cyberos-cuo list-personas`  
`list-workflows`| Enumerate all 221 workflows (filter by persona)| `cyberos-cuo list-workflows --persona chief-technology-officer`  
`route`| Two-stage route a natural-language query (persona → workflow)| `cyberos-cuo route "validate MST 0123456789"`  
`dry-run`| Walk a workflow's skill chain without executing| `cyberos-cuo dry-run chief-technology-officer/architect-new-system`  
`execute`| Execute a workflow chain through the selected invoker; optionally emit a memory audit row per step| `cyberos-cuo execute chief-technology-officer/adr-quick-capture --invoker llm --memory-emit --actor stephen@cyberskill`  
  
Flags: `--explain` (print handler dispatch reasoning), `--no-handler-dispatch` (force linear walk), `--invoker {mock,subprocess,llm}`, `--memory-emit`, `--actor <subject>`, `--output-dir <path>`.

6

## Key flows

### Flow 1 — Single-skill route decision (Phase 1)

sequenceDiagram autonumber participant U as User participant C as cuo.route participant CAT as catalog.scan participant R as router.score participant E as extractor participant B as memory_bridge U->>C: "tạo hoá đơn cho ACME, MST 0123456789, số tiền 10 triệu" C->>C: NFC normalise · preserve diacritics C->>CAT: load catalog (sorted-path) CAT-->>C: [vietnam-mst-validate, vietnam-vat-invoice, …] C->>R: score each candidate R->>R: vietnam-vat-invoice: 3 keyword hits ×3.0 + VN region ×2.0 = 11.0 → saturate 10.0 → conf 1.0 R->>R: vietnam-mst-validate: 1 keyword hit ×3.0 + VN region ×2.0 = 5.0 → conf 0.5 R-->>C: decision={skill:vietnam-vat-invoice, conf:1.0, alts:[vietnam-mst-validate@0.5]} C->>E: extract args (vietnam-vat-invoice extractor) E-->>C: {mst:"0123456789", amount_vnd:10_000_000} alt --invoke C->>SkillHost: cyberos-skill-cli run vietnam-vat-invoice --args... SkillHost-->>C: {exit:0, stdout:Invoice XML 'fpt-2024-001'} end alt --record C->>B: write trace to memory B-->>C: {seq:14935, chain:"e3f7..."} end C-->>U: {routed:true, decision, result?, recorded_at?} 

### Flow 2 — Multi-step chain (Phase 3 preview)

sequenceDiagram autonumber participant U as User participant C as cuo.route participant R as router participant TOPO as chain_planner participant H as Skill host U->>C: "issue VAT invoice for ACME" C->>R: route(query) R-->>C: pick vietnam-vat-invoice (chain_root) C->>TOPO: walk depends_on TOPO-->>C: list[vietnam-mst-validate buyer, vietnam-mst-validate seller, vietnam-vat-invoice] loop for each step C->>H: invoke step H-->>C: result alt step failed C-->>U: chain aborted at step N · reason · partial trace end end C->>memory: composite audit row (root) + N sub-rows C-->>U: {chain_status:ok, steps:[...], total_elapsed_ms:N} 

### Flow 3 — Confidence cascade (Phase 1 → Phase 2)

flowchart TB Q[User query] --> P1[Phase 1 rule-based scorer] P1 --> SCORE{Top score?} SCORE -- "0.70+ auto" --> INV[Auto-invoke top skill] SCORE -- "0.50 to 0.70 ask" --> CLAR[Ask clarification · surface top 3] SCORE -- "0.10 to 0.50 escalate" --> P2[Phase 2 LLM cascade] SCORE -- "below 0.10 defer" --> DEFER[Defer to human · no candidate] P2 --> P2SCORE{LLM confidence?} P2SCORE -- "0.70+" --> INV P2SCORE -- "below 0.70" --> CLAR INV --> REC[Record decision + result in memory] CLAR --> REC DEFER --> REC classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#fef6e0,stroke:#9c750a class P1,SCORE,INV,CLAR,DEFER,REC shipped class P2,P2SCORE planned 

Phase 1 ships the ≥ 0.30 threshold; the four-tier cascade above lands at Phase 2 once the LLM router is online.

### Flow 4 — Persona switch (Phase 4 preview)

sequenceDiagram autonumber participant U as User participant C as cuo (Phase 4) participant PR as persona_router participant CEO as CEO sub-router participant CTO as CTO sub-router U->>C: "draft Q4 OKRs for the eng team" C->>PR: classify_persona(query) PR-->>C: persona=CEO (strategic intent) C->>CEO: route within CEO skill subset CEO-->>C: pick okr-cascade-draft (conf 0.83) C->>memory: audit row with persona-version stamp alt followup: "what's the test coverage on the auth module?" U->>C: new query, same session C->>PR: classify_persona PR-->>C: persona=CTO (technical intent) C->>CTO: route within CTO skill subset CTO-->>C: pick test-coverage-report (conf 0.91) end 

7

## Decision lifecycle

stateDiagram-v2 [*] --> Received Received --> Routing: NL query arrives Routing --> Picked: top candidate at-or-above threshold Routing --> Deferred: no candidate at threshold Routing --> Escalated: borderline score Phase-2 cascade Escalated --> Picked: LLM picks Escalated --> Deferred: LLM also abstains Picked --> Invoking: invoke flag true Picked --> Recorded: invoke flag false - decision only Invoking --> Succeeded: exit code 0 Invoking --> Failed: exit code non-zero Succeeded --> Recorded: trace plus result to memory Failed --> Recorded: trace plus error to memory Deferred --> Recorded: routed false row to memory Recorded --> [*] 

8

## The 47 C-suite personas

Each persona is a curated subset of the Skill catalog. Today the catalog lives flat under `modules/skill/<name>` with full-format names; the supervisor routes via two stages (persona → workflow) then auto-dispatches one of 6 Handlers per workflow `pattern:` frontmatter. The _Auto OK_ column lists actions that may complete without explicit operator approval; the _Defers_ column lists actions that always escape to the human. 

### 8.1 — All 47 active personas (per-row catalog)

Every persona folder lives at `modules/cuo/<persona-slug>/` with a 9-block-schema README and its own `workflows/` directory. **221 workflows live across 47 active personas.** The supervisor routes a natural-language query → persona → workflow → skill chain, then auto-dispatches one of 6 Handlers per workflow `pattern:`. The **Tier** column reflects the staged-introduction wave that first shipped that persona's workflows (Now / Series-A / Scale-up / Enterprise / Niche); operators can ignore tier ordering at runtime — it only governs the authoring backlog. The **Representative workflow** column links to a concrete, file-verified workflow under that persona's `workflows/` folder; some personas carry 5–8 workflows after Tier-C1 depth additions (chief-of-staff, chief-financial-officer, chief-information-security-officer, etc.).

#### Now-tier Founding rhythm of any startup (5 personas, ~28 workflows)

Persona| Mandate (per `README.md` §1)| Workflows| Representative workflow  
---|---|---|---  
[chief-executive-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-executive-officer>)| Highest authority; sets vision; allocates capital; owns the board relationship and the external narrative.| 7| `quarterly-board-update`  
[chief-financial-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-financial-officer>)| Owns FP&A, capital, controls, investor relations; modern CFOs co-own data & tech investment decisions (Deloitte 2025).| 8| `monthly-close`  
[chief-technology-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-technology-officer>)| Owns outward-facing technical proposition — architecture, eng org, build-vs-buy — accountable for the system's ability to scale, evolve, stay defensible.| 5| `architect-new-system`  
[chief-of-staff](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-of-staff>)| CEO leverage multiplier; rhythm-of-business owner; OKR/decision tracker; special-projects lead. One of the most important early hires.| 6| `weekly-rhythm-of-business`  
[chief-legal-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-legal-officer>)| Contracts, regulatory posture, IP, litigation; at Series-A onward typically combines with Compliance & Privacy until those split.| 5| `msa-contract-review`  
  
#### Series-A-tier Adds when scaling beyond the founders (4 personas, ~16 workflows)

Persona| Mandate (per `README.md` §1)| Workflows| Representative workflow  
---|---|---|---  
[chief-operating-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-operating-officer>)| Owns "did we ship". Run-the-business across functions; cycle status; cross-team coordination; vendor performance; SLA enforcement.| 6| `quarterly-operating-review`  
[chief-human-resources-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-human-resources-officer>)| People operations: workforce plan, comp bands, talent review, eNPS, onboarding, DEI program. Reports to CEO; partners w/ CFO on comp.| 7| `quarterly-workforce-plan`  
[chief-sales-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-sales-officer>)| Owns the number. Pipeline coverage, account plans, GTM strategy, NPS program; at Series-A often co-owned with CRO until split.| 6| `weekly-pipeline-review`  
[chief-information-security-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-information-security-officer>)| Threat posture + SOC2 + vuln management + breach response readiness. Time-critical surface; partners w/ CTO on architecture.| 6| `quarterly-security-posture-review`  
  
#### Scale-up-tier Adds when the org passes ~100 people (2 personas, ~10 workflows)

Persona| Mandate (per `README.md` §1)| Workflows| Representative workflow  
---|---|---|---  
[chief-revenue-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-revenue-officer>)| Top of the GTM org: sales + marketing + customer-success + revenue-ops aligned on revenue architecture, comp plan, churn analysis.| 6| `monthly-revenue-forecast`  
[chief-privacy-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-privacy-officer>)| Owns PDPL / GDPR / CCPA posture, DSR pipeline, PIA portfolio, 72-hour breach-notification readiness. Time-critical surface.| 6| `breach-response-cycle`  
  
#### Enterprise-tier Adds at Growth/Enterprise stage (5 personas, ~22 workflows)

Persona| Mandate (per `README.md` §1)| Workflows| Representative workflow  
---|---|---|---  
[chief-product-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-product-officer>)| Owns "are we building the right thing". Roadmap, requirements discovery, user-research synthesis, FR catalogue maintenance.| 5| `quarterly-roadmap-planning`  
[chief-data-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-data-officer>)| Owns the data estate: governance, quality, lineage, data products, customer-360, regulator data-subject responses.| 6| `quarterly-data-governance-review`  
[chief-ai-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-ai-officer>)| Owns AI strategy & portfolio; model cards, bias audits; partners w/ CTO on infra and Ethics on governance.| 6| `quarterly-ai-portfolio-review`  
[chief-customer-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-customer-officer>)| Owns post-sale value: customer health, CAB, churn analysis (w/ CRO), expansion plays, success engagement.| 4| `quarterly-customer-health-review`  
[chief-knowledge-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-knowledge-officer>)| Owns the org's collective know-how: knowledge taxonomy, codification pipeline, knowledge graph (memory governance partner).| 4| `quarterly-knowledge-pipeline`  
  
#### Niche-tier Niche / specialty seats added through stable catalog (31 personas, ~124 workflows)

Authored across Sessions I–N as "no new skills needed" waves — every workflow chains through the already-shipped Tier-1/Tier-2 catalog. This validates the v3.0.0 supervisor hypothesis: a stable ~100-skill catalog can serve the long tail.

Persona| Mandate (per `README.md` §1)| Workflows| Representative workflow  
---|---|---|---  
[chief-marketing-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-marketing-officer>)| Brand + demand; pressure to narrow as some CMOs absorb into CRO. Owns campaigns, content calendar, analyst relations.| 6| `per-campaign-plan`  
[chief-communications-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-communications-officer>)| Internal newsletter cadence, press releases, narrative consistency across audiences. Crisis-comm playbook owner.| 4| `monthly-internal-newsletter`  
[chief-risk-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-risk-officer>)| Enterprise risk management (ERM) framework, KRI dashboards, board risk chapter, per-incident post-mortems.| 4| `quarterly-kri-dashboard`  
[chief-ethics-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-ethics-officer>)| Use-case ethics review, bias portfolio audit, model card ethics sign-off, ethics program governance.| 4| `per-model-card-ethics-sign-off`  
[chief-transformation-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-transformation-officer>)| Owns multi-quarter change programs; portfolio reviews; change-management discipline; transformation roadmap.| 4| `quarterly-portfolio-review`  
[chief-brand-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-brand-officer>)| Brand-strategy, brand campaigns, analyst-brand briefings; partner w/ CMO when both seats exist.| 4| `annual-brand-strategy`  
[chief-innovation-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-innovation-officer>)| Innovation portfolio governance, charter authoring, horizon-3 bets; partner w/ CTO + CSO-Strategy.| 4| `per-innovation-charter`  
[chief-trust-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-trust-officer>)| External-facing trust posture: trust portal, transparency report, time-critical trust-incident updates.| 4| `per-trust-incident-update`  
[chief-compliance-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-compliance-officer>)| Compliance program, control testing, regulatory filings, board compliance chapter.| 4| `quarterly-control-testing`  
[chief-strategy-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-strategy-officer>)| Corporate strategy, portfolio strategy, M&A theses, quarterly strategy review.| 4| `per-mna-thesis`  
[chief-digital-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-digital-officer>)| Digital transformation roadmap, channel program charters, digital portfolio review.| 4| `annual-digital-transformation-roadmap`  
[chief-esg-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-esg-officer>)| ESG strategy + reporting, compliance, stakeholder engagement; partner w/ Sustainability.| 4| `annual-esg-report`  
[chief-sustainability-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-sustainability-officer>)| Emissions inventory, target tracking, annual sustainability report, sustainability strategy.| 4| `annual-emissions-inventory`  
[chief-automation-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-automation-officer>)| Automation roadmap, charter authoring, portfolio review, operating-model impact assessment.| 4| `annual-automation-roadmap`  
[chief-procurement-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-procurement-officer>)| Procurement strategy, supplier scorecard, sourcing-event-per-category, savings tracker.| 4| `quarterly-supplier-scorecard`  
[chief-commercial-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-commercial-officer>)| Partner program, channel strategy, partner scorecard, strategic-partnership-per-deal.| 4| `annual-channel-strategy`  
[chief-growth-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-growth-officer>)| Growth-loop ownership, weekly growth cadence, experimentation portfolio, monetization review.| 4| `weekly-growth-cadence`  
[chief-medical-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-medical-officer>)| Clinical protocol, quarterly safety report, medical-affairs charters; healthcare-vertical only.| 4| `per-clinical-protocol`  
[chief-investment-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-investment-officer>)| Investment thesis-per-target, portfolio review, LP letters; PE/VC-vertical or scale-corp-development.| 4| `per-investment-thesis`  
[chief-restructuring-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-restructuring-officer>)| Turnaround plan, stakeholder-communication-per-event; distressed-situation specialist.| 4| `per-turnaround-plan`  
[chief-accounting-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-accounting-officer>)| Monthly-close execution, quarterly audit readiness; partners w/ CFO at scale-up onward.| 4| `monthly-close-execution`  
[chief-administrative-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-administrative-officer>)| Weekly back-office cadence, vendor consolidation, administrative operating model.| 4| `annual-vendor-consolidation`  
[chief-remote-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-remote-officer>)| Distributed-work program, remote effectiveness review, remote-policy authoring.| 4| `annual-remote-policy`  
[chief-happiness-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-happiness-officer>)| eNPS deep-dive, wellbeing intervention, happiness program; people-experience specialist.| 4| `quarterly-enps-deep-dive`  
[chief-experience-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-experience-officer>)| End-to-end customer-experience strategy, journey charter, CX review, customer-360 engagement.| 4| `per-journey-charter`  
[chief-architect](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-architect>)| Architecture vision, design reviews, per-decision ADRs, threat-model reviews. Reports to CTO at most companies.| 4| `per-architecture-decision`  
[chief-information-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-information-officer>)| Internal IT estate; vendor scorecard, IT operating review, IT strategy + IT security strategy.| 4| `annual-it-strategy`  
[chief-diversity-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-diversity-officer>)| DEI program, DEI progress review, ERG charters, DEI strategy; partner w/ CHRO.| 4| `annual-dei-strategy`  
[chief-learning-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-learning-officer>)| Learning strategy, leadership development program, learning effectiveness review.| 4| `annual-learning-strategy`  
[chief-security-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-security-officer>)| Converged security (cyber + physical + insider); per-converged-incident-postmortem, physical-security charter.| 4| `annual-converged-security-strategy`  
[chief-people-officer](<https://github.com/cyberskill/cyberos/tree/main/modules/cuo/chief-people-officer>)| Sister-title for CHRO at some companies (people-first framing). Employee value proposition, people strategy, people review. Synonym-pointer README to CHRO when both seats coexist.| 4| `annual-employee-value-proposition`  
  
Total: 47 active personas (chief-metaverse-officer is EXTINCT — see §8.3) · 221 workflows live · all chain through the SKILL module's **104 author+audit pairs / 108 contracts**. [The C-Suite Reference §5](<../../../modules/cuo/README.md>) is the canonical role atlas; the staged-introduction order is documented in [REPORTS.md §2](<https://github.com/cyberskill/cyberos/blob/main/docs/feature-requests/REPORTS.md>).

Alternative view: personas grouped by functional family (13 buckets)

Executive (4)

chief-executive-officer · chief-operating-officer · chief-financial-officer · chief-of-staff

Technology & AI (5)

chief-technology-officer · chief-information-officer · chief-ai-officer · chief-architect · chief-automation-officer

Data & knowledge (4)

chief-data-officer · chief-digital-officer · chief-knowledge-officer · chief-information-security-officer

Product & design (3)

chief-product-officer · chief-innovation-officer · chief-experience-officer

Revenue & growth (4)

chief-revenue-officer · chief-sales-officer · chief-marketing-officer · chief-growth-officer

Customer & brand (4)

chief-customer-officer · chief-communications-officer · chief-brand-officer · chief-commercial-officer

People (5)

chief-human-resources-officer · chief-people-officer · chief-learning-officer · chief-diversity-officer · chief-happiness-officer

Legal · privacy · trust (4)

chief-legal-officer · chief-privacy-officer · chief-trust-officer · chief-ethics-officer

Risk · security · compliance (3)

chief-risk-officer · chief-security-officer · chief-compliance-officer

Finance ops & investment (3)

chief-accounting-officer · chief-investment-officer · chief-restructuring-officer

Operations & procurement (2)

chief-procurement-officer · chief-administrative-officer

Strategy · transformation · ESG (4)

chief-strategy-officer · chief-transformation-officer · chief-sustainability-officer · chief-esg-officer

Specialist (3)

chief-medical-officer · chief-remote-officer · chief-metaverse-officer (extinct)

Personas live

47 / 48

chief-metaverse-officer EXTINCT

Workflows shipped

221

avg 4.7 per persona

Skills behind workflows

104 pairs

208 author+audit bundles

Contracts

108

per-output template + invariants

Handler subclasses

6

Linear · TimeCritical · PerInstance · MultiOutput · SequentialApproval · PersonaPair

Tier waves

5

Now · Series-A · Scale-up · Enterprise · Niche

### 8.2 — Worked examples · canonical 10 deep-dive

The 10 cards below illustrate the persona-card shape every README under `modules/cuo/<persona-slug>/` renders — the routed scope, an Auto-OK action list (low-stakes, takes the auto-action), a Defers-to-human list (high-stakes, escapes to the operator). The full 47-row catalog is in §8.1 above; the patterns shown here generalise to every persona. Per [`modules/cuo/_template/persona/README.md`](<https://github.com/cyberskill/cyberos/blob/main/modules/cuo/_template/persona/README.md>) the per-persona README always follows the 9-block schema from C-Suite Reference §4.

### 🎯CEO · Vision & Strategy Stephen Cheng (Founder seat)

Strategy memos, OKR cascade reviews, board narrative, weekly state-of-business, runway / fundraising posture. Owner of vision and capital allocation.

Auto OK

draft strategy memosummarise OKR progressgenerate weekly state-of-businessprep board update

Defers to human

send memo to investorsflip Singapore HoldCochange cap-tableterminate executive

### ⚙️COO · Operations shipped · 4 workflows (Series-A wave)

Cycle status digests, blocker triage, cross-team coordination, weekly ops review, vendor performance. Owner of "did we ship".

Auto OK

status digest from PROJflag overdue taskssummarise cycle enddraft 1:1 prep

Defers to human

cancel a projectreassign ownerchange vendoroverride SLA

### 💰CFO · Finance & Runway shipped · 6 workflows (Now + depth)

Cashflow position, AR/AP digests, burn alerts, payroll cycles, VAT/CIT compliance posture. Owner of "do we have runway".

Auto OK

cashflow snapshotAR aging reportdraft invoice (via INV)flag overdue receivable

Defers to human

send invoice to clientexecute wireapprove refundchange banking signatory

### 📣CMO · Marketing & Demand shipped · 4 workflows (Session I)

Campaign briefs, content calendar, channel reports, brand voice consistency. Owner of "do prospects know us".

Auto OK

draft campaign briefsummarise channel performancepropose A/B testenforce brand voice

Defers to human

publish public-facing contentspend marketing budgetmake public statement

### 💻CTO / CIO · Technology & Information Systems co-owned: CEO + CTO seat

Tech-debt triage, security advisories digest, OBS metric review, dependency upgrades, architecture decision records. Owner of "is the platform safe and fast".

Auto OK

propose ADRdraft tech spectest-coverage reportsummarise OBS digestflag CVE

Defers to human

deploy to productionrotate KMS keygrant new capabilitydisable security control

### 👥CHRO · People & Talent shipped · 5 workflows (Series-A + depth)

1:1 prep, performance summaries, onboarding paths, role descriptions, retention signals, growth ladders. Owner of "do we have the right people".

Auto OK

draft 1:1 agendasummarise perf reviewsdraft job descriptiononboarding checklist

Defers to human

make offerterminateadjust comp bandconduct performance conversation

### 🧭CSO · Strategy P3+ emerging

Competitive intel, scenario modelling, M&A scanning, partnership feasibility, strategic option papers. Owner of "what are we doing next".

Auto OK

competitive scanscenario modelpartnership feasibility memooption paper draft

Defers to human

approach partnercommit to strategic shiftexecute M&A LOI

### ⚖️CLO / CCO · Legal & Compliance co-owned: CSO + CLO seat at P2+

Contract redline, NDA triage, GDPR/PDPL audits, DSAR triage, vendor terms review, policy authoring. Owner of "are we compliant".

Auto OK

contract redline drafttriage NDADSAR intakePDPL gap analysispolicy draft

Defers to human

sign contractexecute DPAfile regulator submissionapprove cross-border transfer

### 📊CDO · Data P2+ emerging

Data quality, lineage, residency reviews, schema governance, retention policy, memory integrity oversight. Owner of "is our data trustworthy".

Auto OK

audit memory doctordata quality reportretention policy reviewlineage diagram

Defers to human

purge datachange retention periodapprove cross-border exportdisable encryption

### 🚀CPO · Product P0 co-owned with CEO seat

Product-brief drafts, roadmap analysis, requirements discovery, user-research synthesis, FR catalogue maintenance. Owner of "are we building the right thing". Today's most-exercised persona — the feature-request-author / product-requirements-document-author / requirements-discovery skills all live here.

Auto OK

product-requirements-document-author draftFR catalog refreshrequirements discovery interviewuser-research synthesisroadmap update

Defers to human

ship featuremake customer-facing promisechange pricingcommit to roadmap publicly

### 8.3 — Why one persona is extinct

`chief-metaverse-officer` sits in the catalog as an empty husk — its folder exists but has no shipped workflows and never will. Per [The C-Suite Reference §8 rule 4](<../../../modules/cuo/README.md>), when a C-role concept fails the market test (the metaverse-as-business-platform thesis collapsed 2022-2024), we KEEP its folder as a cautionary tale rather than delete it. New persona proposals MUST reference the extinct list during ADR review and explain why the proposed role isn't another metaverse moment. 

8

## Functional Requirements

The CyberOS FR catalogue is being rebuilt one feature at a time via the open [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) Agent Skill.

Previous FR enumerations were archived 2026-05-14 and are no longer reflected on this page. Specific FRs land here as they are re-authored.

10

## Non-Functional Requirements

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`N(FR pending)`| Phase-1 routing p95| ≤ 5 ms (catalog cached)| fixtures/golden_routing.json benchmark  
`N(FR pending)`| Phase-2 LLM cascade p95| ≤ 800 ms incl. network| AI Gateway latency budget  
`N(FR pending)`| Catalog refresh p95| ≤ 50 ms over 100 skills| catalog.scan benchmark  
`N(FR pending)`| Phase-1 determinism| 100% (same query+catalog → same decision)| 15 routing fixtures, golden tests  
`N(FR pending)`| Escalation rate to LLM| ≤ 10% of queries (after warm-up)| memory audit replay · weekly KPI  
`N(FR pending)`| Defer-to-human rate| ≤ 5% of queries| memory audit replay  
`N(FR pending)`| Test coverage of router.py| ≥ 90% line · 100% branch| coverage.py  
`N(FR pending)`| Availability (in-process)| same as caller| n/a — co-located  
  
11

## Dependencies

CUO is the most-connected module. It consumes memory (records), Skill (invokes), AI Gateway (Phase 2), MCP Gateway (tool surface), AUTH (actor identity). It is consumed by every user-facing module.

graph LR subgraph upstream ["CUO depends on"] AUTH["🔐 AUTH"] AI["⚡ AI Gateway"] MCP["🔌 MCP Gateway"] memory["🧠 memory"] SKILL["🛠 SKILL"] end CUO_M["🎯 CUO"] subgraph downstream ["Used by user-facing modules"] CHAT["💬 CHAT"] EMAIL["✉️ EMAIL"] PROJ["📋 PROJ"] CRM["🤝 CRM"] HR["👥 HR"] KB["📚 KB"] OTHERS["…14 more"] end AUTH --> CUO_M AI --> CUO_M MCP --> CUO_M memory --> CUO_M SKILL --> CUO_M CUO_M --> CHAT CUO_M --> EMAIL CUO_M --> PROJ CUO_M --> CRM CUO_M --> HR CUO_M --> KB CUO_M --> OTHERS classDef shipped fill:#f5ede6,stroke:#45210e classDef planned fill:#fef6e0,stroke:#9c750a class CUO_M,memory,SKILL shipped class AUTH,AI,MCP,CHAT,EMAIL,PROJ,CRM,HR,KB,OTHERS planned 

12

## Compliance scope

Regulation / standard| Article / clause| CUO feature that satisfies it  
---|---|---  
EU AI Act (Reg. 2024/1689)| Art. 12 — Logging| Every decision recorded in memory via memory_bridge · Postgres checkpointer at P2 retains LLM prompts.  
EU AI Act| Art. 13 — Transparency| End-of-response transparency: skill chosen + confidence + alternatives are surfaced to the user.  
EU AI Act| Art. 14 — Human oversight| Below-threshold queries defer to human; CUO never auto-invokes irreversible operations.  
EU AI Act| Art. 26 — Operator obligations| Defer-to-human matrix per persona (auto-OK vs defers) is normative.  
EU AI Act Annex III| § 4 — High-risk classification| CUO does not perform employment / credit / law-enforcement scoring; classification remains limited-risk.  
ISO/IEC 42001 (AIMS)| § 8.4 — AI system operations| Audit-chained decisions provide post-hoc accountability evidence.  
Vietnam PDPL| Art. 14 — Decision transparency| Per-decision rationale is part of the trace row; subject can request via DSAR.  
  
13

## Risk entries

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-CUO-001`| Routing mis-classification (wrong skill picked at high confidence)| Medium| Medium| CPO| 15 golden fixtures · Phase 4 persona pre-classifier · trust-calibration KPI alarmed at p99.  
`R-CUO-002`| Confidence threshold drift (real-world distribution diverges from fixtures)| High| Low| CPO| Weekly KPI review: confidence histogram · escalation rate · defer rate. Threshold tunable per deployment at Phase 2.  
`R-CUO-003`| Persona prompt-injection (skill description tries to expand its own scope)| Medium| High| CSO| Trust model (§7): skill descriptions are UNTRUSTED. Keyword bank + catalog are protocol-defined and version-controlled.  
`R-CUO-004`| LLM non-determinism breaks audit replay (Phase 2+)| High| Medium| CTO| Phase 2 trace rows MUST include full prompt + model + temperature + seed; replay tools accept "best-effort" replay note.  
`R-CUO-005`| Persona switching whiplash (user feels Genie isn't "one" anymore at Phase 4)| Medium| Low| CXO (emerging)| Persona-version stamp on every decision · same conversational style enforced via brand-voice skill.  
`R-CUO-006`| Skill catalog explosion drops routing accuracy| Low (today)| Medium| CPO| Phase 4 persona-router triages first · Phase 2 LLM cascade handles the long tail.  
`R-CUO-007`| Capability bypass (CUO grants tools the skill didn't request)| Low| High| CSO| §6.1–§6.2: CUO MUST respect the skill's `allowed-tools`. Defence in depth: Skill broker enforces independently.  
`R-CUO-008`| **Lumi tenant-id spoofing** — a malicious skill or compromised JWT claims a different tenant_id and writes to the wrong tenant's memory| Low| Critical| CSO| Per AUTH §2.7: `tenant_id` JWT claim is non-removable; Lumi's memory verifies via JWKS reachability before accepting any sync push; cross-tenant write at storage layer is a CI-tested impossibility.  
`R-CUO-009`| Destructive operation auto-invoked despite defer-to-human matrix| Low| Critical| CSO| Defer-to-human matrix (§3.6) is normative + machine-checked; CI test fails if any persona's auto-OK column contains a flag `destructive=true` skill.  
`R-CUO-010`| Catalog drift between route-time and invoke-time creates audit chain that doesn't match disk state| Medium| Medium| CTO| Catalog fingerprint embedded in pre-invocation audit row + version-pinning at invoke; on mismatch broker rejects + emits `catalog_drift` trace.  
`R-CUO-011`| Cross-surface latency budget miss (PROJ inline genie > 800 ms p95 → Members lose flow)| Medium| Medium| CPO| Per-surface latency budgets in §3.7 enforced at OBS; alert page CPO + CTO if p95 exceeds; fall back to "Genie is thinking…" indicator gracefully.  
`R-CUO-012`| Lumi cross-tenant synthesis leaks private memory across tenants| Low| Critical| DPO| Synthesis input strictly filtered by `sync_class ∈ {shareable, team-public, org-only}`; private/personal never enters Lumi; quarterly red-team check on synthesis outputs.  
`R-CUO-013`| Persona prompt drift via Lumi-pushed updates breaks tenant-pinned behaviour| Medium| Medium| CPO| Persona version stamp in audit row; tenants can pin persona versions; Lumi-pushed updates require tenant opt-in at the version-pin level.  
`R-CUO-014`| EU AI Act Art. 12 logging gap (Phase 1 flat-file memory bridge missed some fields needed for audit)| Medium| High| CLO| Phase 2 migration to Postgres checkpointer required by P0 · exit for tenants subject to EU AI Act; per-row schema covers prompt + model + temperature + seed + result hash + alternatives.  
`R-CUO-015`| @lumi rate-limit abuse (one Member spams @lumi → AI Gateway cost overrun)| Medium| Medium| CTO| Per-Member @lumi rate limit (≤ 30/hour default) at AI Gateway; per-tenant monthly @lumi budget surfaced in CUO dashboard.  
`R-CUO-016`| Phase 2 LLM cascade goes down → ambiguous queries lose the cascade fallback| Low| Medium| CTO| Cascade is degradable: when AI Gateway unreachable, CUO falls back to "show top 3 candidates + ask user to pick" (the defer-to-human path); never blocks the request.  
`R-CUO-017`| Genie answers confidently from training-cutoff knowledge on company-specific topics| High| Medium| CPO| RAG over memory + KB enforced before LLM call; system prompt forbids speculation on company-specific facts without source; CDO + CPO maintain "must-cite-source" classifier in CI.  
  
14

## KPIs

KPI| Formula| Source| Target| Current  
---|---|---|---|---  
**Routing confidence distribution**|  histogram of `conf`| memory audit replay| mean ≥ 0.6 · p10 ≥ 0.3| 0.7 / 0.4 (15-fixture eval)  
**Escalation rate**|  queries with 0.10 ≤ conf ≤ 0.50| memory audit replay| ≤ 10%| n/a — Phase 2 pending  
**Defer rate**|  queries with conf < 0.10| memory audit replay| ≤ 5%| 0% (fixtures)  
**Decision latency p95**|  route wall clock| per-request timing| ≤ 5 ms (Phase 1)| ~ 0.4 ms  
**Replay equivalence rate**|  identical decision on second run| fixture re-evaluation| 100% (Phase 1)| 100%  
**Invocation success rate**|  exit_code 0 / total invocations| memory audit replay| ≥ 95%| 100% (15 fixtures)  
**Trust calibration error**|  |confidence − actual_correct_rate|| weekly human review| ≤ 0.10| 0.05 (fixture eval)  
**Per-surface response p95 (PROJ inline)**|  histogram on PROJ inline genie| OBS span on `cuo.invoke{surface=proj.inline}`| ≤ 800 ms| n/a — surface pending  
**Per-surface response p95 (CHAT @lumi)**|  histogram on CHAT @lumi| OBS span on `cuo.invoke{surface=chat.lumi}`| ≤ 4 s incl. LLM| n/a — CHAT pending  
**Destructive-op auto-invoke rate**|  count(destructive ops auto-invoked) / total destructive ops| memory audit replay| = 0 (hard zero)| 0 (fixtures)  
**Lumi sync push success rate**|  successful sync pushes / total attempts| Lumi's memory ingest log| ≥ 0.99 over 28-day window (P3+)| n/a — P3 pending  
**Cross-tenant sync_class violation rate**|  memories with private/personal that reached Lumi / total sync attempts| Lumi pre-write classifier| = 0 (hard zero)| n/a — P3 pending  
**Persona-version stability**|  persona version changes per quarter| memory audit by persona| ≤ 2 per persona per quarter| 0 (P1)  
**@lumi cost per active Member (monthly)**|  AI Gateway cost / DAU| AI Gateway billing| ≤ $5 / DAU / month| n/a — Phase 2 pending  
**Must-cite-source compliance**|  Genie answers that cite a source / answers about company-specific facts| CI classifier on production samples| ≥ 0.95| n/a — Phase 2 pending  
**Dogfooding rate (internal Members)**|  distinct internal Members using Genie ≥ 5x/day| CUO decision events filtered to `tenant_id=org:cyberskill`| 100% of full-time team by P0 · exit| tracking begins P0 · start  
  
15

## RACI matrix

Activity| CEO| CPO| CTO| CXO*| CSO| CLO  
---|---|---|---|---|---|---  
Persona definition (47 C-suite + 1 extinct)| A| R| C| C| I| I  
Keyword bank maintenance| I| R| A| C| I| I  
Phase 2 LLM cascade design| C| C| A/R| I| C| I  
Trust calibration KPI review| I| R| C| A| I| I  
Defer-to-human matrix| I| C| C| I| I| A/R  
EU AI Act Art. 12 compliance| I| C| C| I| C| A/R  
Prompt-injection defence| I| C| R| I| A| C  
  
*CXO seat is emerging at P3+; the CPO carries this work today.

16

## CLI usage — real examples

### 1\. List the skills CUO can route to
    
    
    $ cyberos-cuo catalog --format json | head -30
    {
     "fingerprint": "9c8e2a...4b7d",
     "scanned_at": "2026-05-14T07:30:11Z",
     "skill_count": 208,
     "skills": [
     {"name": "product-requirements-document-author", "version": "0.4.1", "region": null, "keywords": ["prd", "author", "draft prd", "product requirements"]},
     {"name": "vietnam-mst-validate", "version": "0.2.0", "region": "VN", "keywords": ["mst", "tax code", "ma so thue"]},
     {"name": "vietnam-vat-invoice", "version": "0.3.0", "region": "VN", "keywords": ["invoice", "hoa don", "vat", "gtgt"]},...
     ]
    }

### 2\. Route a query without invoking
    
    
    $ cyberos-cuo route "kiểm tra MST 0123456789-001"
    
    decision:
     skill_name: vietnam-mst-validate
     arguments: {mst: "0123456789-001"}
     confidence: 1.0
     rationale: "VN-diacritic query + region=VN bonus + 2 keyword hits + name-substring match"
     routed: true
    
    alternatives:
     - vietnam-tax-filing score=0.3
     - vietnam-vat-invoice score=0.2
    
    (not invoked — pass --invoke to dispatch through the Skill host)

### 3\. Route, invoke, and record in memory
    
    
    $ cyberos-cuo route "validate MST 0123456789" --invoke --record
    
    decision: vietnam-mst-validate (conf=0.7)
    invocation: exit=0 elapsed_ms=24
    stdout: {"ok": true, "format": "10-digit"}
    recorded: memory:/meta/cuo-decisions/1747200611_8c4e.md
     seq=14941 chain=a3c7...2b9f

### 4\. Inspect a past decision (audit replay)
    
    
    $ cyberos view meta/cuo-decisions/1747200611_8c4e.md
    ---
    kind: decisions
    sync_class: private
    classification: internal
    schema: cuo-decision-v1
    ---
    # CUO routing decision
    
    **Query:** validate MST 0123456789
    **Catalog fingerprint:** 9c8e2a4b7d...
    **Decision:** vietnam-mst-validate
    **Confidence:** 0.7
    **Rationale:** 1 name-substring hit + 1 keyword + region tiebreaker
    **Alternatives:** [...]
    **Invoked at:** 2026-05-14T07:30:11Z
    **Result:** exit=0, stdout={"ok": true}

### 5\. Show keyword bank + extractors for a skill
    
    
    $ cyberos-cuo skills vietnam-vat-invoice
    
    skill: vietnam-vat-invoice
    region: VN
    keywords: [invoice, hoa don, vat, gtgt, e-invoice, xuat hoa don]
    extractor: detect-amount
     pattern: r"(\d[\d.,]*)\s*(triệu|trieu|million|k|VND|đồng)"
     ※ structured extraction deferred to Phase 2
    depends_on: [vietnam-mst-validate] ← Phase 3 will walk this
    allowed_tools: [read_file, write_file]

17

## Phase status & code stats

Total LoC (Python)

~1,600

supervisor · handlers · cli · tests/

Test count

49/50 green

1 skip = catalog-completeness invariant

Core modules

8

catalog · validator · router · supervisor · handlers · invoker · memory_bridge · cli

Personas with workflows

47 / 48

chief-metaverse-officer EXTINCT

Workflows live

221

post Tier-C1 depth additions

Routing latency p95

~ 0.4 ms

in-process, catalog cached

Phase / capability| Status  
---|---  
Phase 1 — filesystem catalog scanner · two-stage router · memory dry-run| shipped  
Phase 2 — pluggable Invoker (Mock + Subprocess) · `execute_chain()`| shipped  
Phase 3 — LLMInvoker (mock-llm + Anthropic) · memory audit-chain emission| shipped  
Phase 4 — 6 Handler subclasses · auto-dispatch by `pattern:` frontmatter · 8 new memory audit kinds| shipped  
Postgres checkpointer (EU AI Act Art. 12)| planned · P0 · exit  
Ambient nudge modes (Notify · Question · Review)| planned  
GraphQL subgraph| planned · P0+  
  
18

## References

  * **Bigger picture (§0 above):** 3 strategic roles + cross-module dependency Mermaid + auto-vs-human matrix.
  * **Lumi identity wrapper (§3.5 above):** Lumi vs Genie vs local CUO naming + AUTH JWT shape + cross-tenant synthesis policy.
  * **Skill broker contract (§3.6 above):** 7-step broker sequence + 7-row CUO↔broker contract + 10-row defer-to-human matrix.
  * **Cross-module surfaces (§3.7 above):** 9-row canonical surface table + per-surface latency budgets.
  * **Cross-module page links:** [memory.html](<../memory/index.html>) · [skill.html](<../skill/index.html>) · [auth.html](<../auth/index.html>) · [chat.html](<../chat/index.html>) · [proj.html](<../proj/index.html>) · [ai.html](<../ai/index.html>) · [mcp.html](<../mcp/index.html>) · [obs.html](<../obs/index.html>)
  * **memory auto-sync vision:** [MEMORY_AUTOSYNC_DESIGN.md §5 (capture surfaces)](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) — CUO decisions are one of four canonical capture inputs to the local memory; §6 (Lumi cross-tenant synthesis).
  * **FR authoring discipline:** [modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>) — CUO FRs land via the `feature-request-author` Agent Skill (CUO already-shipped: the first 50 FRs are CUO + memory + Skill).
  * **Phase rollout history:** All four supervisor phases (rule-based routing → pluggable invoker → LLM invoker + memory audit emission → 6-Handler dispatch) shipped 2026-05-18. See `cyberos/CHANGELOG.md` entries tagged `[CUO]` for per-phase details.
  * **AGENTS.md** (normative) — `cyberos/modules/cuo/AGENTS.md`.
  * **SPEC.md** — contract summary — `cyberos/modules/cuo/README.md §10 (Data shapes)`.
  * **ROUTING.md** — keyword-bank rationale + Phase 2 LLM design — `cyberos/modules/cuo/README.md §9 (Routing algorithm)`.
  * **Source:** `cyberos/modules/cuo/cuo/core/`, `cyberos/modules/cuo/tests/`, `cyberos/modules/cuo/tests/fixtures/ (if present)`.
  * **CHANGELOG:** `cyberos/CHANGELOG.md (entries tagged [CUO])` (newest-first).
  * **EU AI Act Art. 12 + 14 + 26:** logging requirements, human oversight, operator obligations — all routed through the audit chain.
  * **Vietnam PDPL (Law 91/2025):** Art. 14 — decision transparency surfaced via DSAR.



[← Previous: memory](<../memory/index.html>) [Next module: SKILL →](<../skill/index.html>)
