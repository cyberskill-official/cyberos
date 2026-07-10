---
title: AI Gateway — Cost-of-everything gate · Provider-agnostic router · Compliance plane · CyberOS
source: website/docs/modules/ai/index.html
migrated: FR-DOCS-002
---

AI Gateway is the **policy-enforcement and routing layer** for every model call inside CyberOS. From the outside it is one gRPC service speaking `chat.complete`, `embed`, `rerank`, and `image.generate`. Inside, a LiteLLM-derived router consults per-tenant policy, applies persona-version system prompts, runs Presidio + custom-VN PII redaction, looks up the prompt-cache, picks a primary provider, retries twice with backoff, fails over to the secondary, accounts for tokens against the tenant's monthly cap, streams the response back over SSE, and emits one `ai.invocation` audit row per call. Zero retention — no provider sees cross-tenant prompts; no cache row crosses a tenant boundary; the only thing that leaves the cluster is the prompt and the response, and even those are PII-scrubbed. 

Strategic role

Cost-of-everything gate

Ships P0 · slice 1 · before AUTH

Status

Planned

P0 · design phase · P0 · slice 1

Build placement

P0 · slice 1 (P0 #1)

Reordered per research review §2.4

Est. LoC

~6,500

Python 3.13 + Rust edge proxy

Providers (P0)

Bedrock · Anthropic · OpenAI

\+ self-hosted BGE

Failover SLA

≤ 30 s

primary down → secondary live

PII redaction recall

≥ 99%

VN + EN test set · CI gate

Cache hit rate

≥ 30% (P0)

≥ 60% (P2+) · tenant-keyed

Cost-cap enforcement

Hard-stop

tenant cannot exceed monthly cap

ZDR (zero retention)

Required

P0 provider contracts

Depends on

memory · OBS

\+ AUTH (P0 · slice 2) · TEN (P2+)

Used by

CUO · Skill · KB · CHAT · …

every LLM-calling module

0

## The bigger picture — three strategic roles

AI Gateway is the most under-rated module in CyberOS. The naive read is "it's a thin proxy in front of LiteLLM." The real read is: **this is the single most leveraged P0 module** , because every other module that touches an LLM (CUO, CHAT, KB, PROJ, every Genie surface) inherits its cost behaviour, compliance posture, and provider resilience. Research review §2.4 explicitly reordered the build sequence to land AI Gateway at P0 · slice 1 — _before AUTH_ — for this reason. 

Role 1 · Cost-of-everything gate

Every LLM call attributed, metered, capped

Token accounting is the difference between a $50/month SaaS and a $50,000/month surprise bill. AI Gateway is the only point where every call is counted, attributed to a tenant + persona + module, capped against a monthly budget, and surfaced as invoice line items. A pre-call check refuses the request if the projected cost exceeds remaining budget; a post-call check reconciles the actual usage to the provider's bill. Tenant cost overrun events = 0 is a hard target, not a soft KPI. 

Role 2 · Provider-agnostic router

Swap providers without touching consumers

LiteLLM-derived router speaks Bedrock + Anthropic + OpenAI + Vertex behind a single gRPC interface. Every consumer (CUO, KB, CHAT) calls `ai.chat.complete` with a model alias; the router resolves the alias to the active provider + falls over to a secondary on 30 s SLA. Vendor lock-in becomes vendor optionality — switching from Anthropic to Vertex is a config change. Geographic residency (SG-1 vs EU-1) is enforced at the router, not by the consumer. 

Role 3 · Compliance plane

Single PII chokepoint + persona stamp + audit row

Presidio + a VN-specific PII recogniser run on every prompt before bytes leave the cluster. Persona-version is stamped on every call so the audit row records which agent identity (cuo-cpo@0.4.1, genie@1.0.2) was talking. EU AI Act Art. 12 logging satisfied via the `ai.invocation` chained memory audit row. ZDR contracts with providers required at P0 — if a provider cannot offer zero retention, AI Gateway refuses to route to them. 

### AI Gateway in the runtime — every LLM call flows through here

flowchart TB subgraph callers["Callers (every LLM-touching module)"] CUO["🎯 CUO router"] KB["📚 KB RAG"] CHAT["💬 CHAT @lumi"] PROJ["📋 PROJ inline genie"] SKILL["🛠 Skill LLM-aided skills"] OBS["👁 OBS auto-triage"] end AIGW["⚡ AI Gateway  
PII scrubber · cost gate · persona stamp · cache · failover"] subgraph providers["Providers"] BEDROCK["AWS Bedrock"] ANTHROPIC["Anthropic API"] OPENAI["OpenAI API"] VERTEX["GCP Vertex (P1+)"] LOCAL["Self-hosted BGE-M3 / rerank"] end subgraph platform["Platform deps"] memory["🧠 memory audit chain"] OBS2["👁 OBS metrics + traces"] AUTH["🔐 AUTH (P0 · slice 2 onward)"] TEN["🏢 TEN tenant config (P2+)"] end CUO --> AIGW KB --> AIGW CHAT --> AIGW PROJ --> AIGW SKILL --> AIGW OBS --> AIGW AIGW -->|"primary"| BEDROCK AIGW -.->|"failover"| ANTHROPIC AIGW -->|"per-tenant cap"| OPENAI AIGW -.->|"P1+"| VERTEX AIGW -->|"embed / rerank"| LOCAL AIGW --> memory AIGW --> OBS2 AIGW -. "JWT verify (P0 · slice 2+)".-> AUTH AIGW -. "tenant policy".-> TEN classDef hub fill:#dbeafe,stroke:#1e40af,stroke-width:3px,color:#1e3a8a classDef caller fill:#e0e7ff,stroke:#3730a3 classDef prov fill:#fef6e0,stroke:#9c750a classDef dep fill:#f5ede6,stroke:#45210e class AIGW hub class CUO,KB,CHAT,PROJ,SKILL,OBS caller class BEDROCK,ANTHROPIC,OPENAI,VERTEX,LOCAL prov class memory,OBS2,AUTH,TEN dep 

AI Gateway is the protocol-level chokepoint. Removing it from the architecture means every consumer reimplements cost accounting, PII scrubbing, and provider failover — and the implementations disagree.

### Auto vs human-in-loop operations matrix

Operation| How it happens| Why this split  
---|---|---  
Provider failover (primary → secondary)| **Auto** within 30 s on consecutive 5xx / timeout| Latency-critical; humans can't react in 30 s. Audit row records the failover event for retro analysis.  
Persona prompt injection| **Auto** based on caller's `agent_persona` JWT claim| Single source of truth in memory; consumers never inject system prompts; eliminates persona drift.  
PII scrubbing| **Auto** on every prompt (Presidio + VN-PII recogniser)| Recall ≥ 99% is the CI gate; nothing leaves the cluster without scrubbing.  
Tenant cost-cap enforcement| **Auto** hard-stop at 100% of monthly cap; warn at 80%| Soft cap = surprise bill. Hard-stop must be configurable per tenant for emergency overrides — gated by CFO approval.  
Cost-cap override (emergency)| **Human approval** — CFO sign-off recorded in memory| Override exists for legit production incidents; defaults to denied; every override is an audit row.  
ZDR-non-compliant provider routing| **Refused** — no auto, no override| A non-ZDR provider is a compliance breach in flight; the only correct behaviour is refuse.  
Cache hit serving| **Auto** when idempotency-key + tenant_id matches| Cache key strictly tenant-scoped; cross-tenant hit is a property-test failure (hard zero).  
Model alias resolution| **Auto** per tenant policy| Tenant pins primary + fallback in their policy; gateway resolves alias at call time; no consumer-side choice.  
Image-generation route (P2+)| **Auto** but always cached + watermarked| Generation cost is real; cache + dedupe by prompt hash; watermark for EU AI Act Art. 50.  
  
1

## Why AI Gateway exists

Letting every module embed its own LLM SDK creates three problems at once. (a) Cost-tracking turns into per-module spreadsheets that never agree. (b) PII redaction becomes a decentralized policy that drifts. (c) Provider failover requires per-module change windows whenever Anthropic / OpenAI degrades. The AI Gateway pattern is the standard answer: pay the cost of one integration once, let every other module call a single typed RPC, and centralise the policy. 

🎯

One door, many providers

LiteLLM router speaks OpenAI / Anthropic / Bedrock / Vertex behind a single API. Switching providers is a config change, not a code change.

🛡

PII never leaves un-scrubbed

Presidio + custom Vietnamese rules redact CCCD, MST, bank accounts, addresses before the request hits a provider — recall ≥ 99% measured.

💸

Cost is a property of the platform

Every call lands one `ai.invocation` row: actor · model · tokens · USD · cache-state. Per-tenant cap, 80% warning, 100% hard stop.

The bet is the same bet AUTH and memory make: pay the cost once at the substrate. Without AI Gateway, each module re-implements PII redaction (and at least one of them gets it wrong), each module embeds its own SDK (and the OpenAI Python client and the Anthropic Python client disagree on streaming chunks), and the regulator's "show me every prompt that touched personal data" question becomes a forensic project. With AI Gateway, that question is a SQL query. 

2

## What it does — 5W1H2C5M

\+ §9.7 + §11.2.1 give the full picture; this table is the working summary.

Axis| Question| Answer  
---|---|---  
**5W · What**|  What is AI Gateway?| A gRPC service that wraps a LiteLLM-derived router. It selects providers, redacts PII, caches deterministic responses, accounts for tokens, and emits audit rows. Single binary today; horizontally scalable behind an L7 load balancer.  
**5W · Who**|  Who calls it?| CUO router (for routing decisions), Skill host (for skill-invoked LLM steps), KB (semantic ingest + retrieval), CHAT (summarisation, smart-reply), Genie (interactive Q&A), Email composer, Project planner. **Owner:** CTO seat.  
**5W · When**|  When does call happen?| Synchronously per user request (chat completion); asynchronously for batch jobs (KB ingest, daily digest). Cache lookup happens first; only cache-miss requests hit a provider.  
**5W · Where**|  Where does it run?| Fargate task in SG-1 (P0); read-only embedder/reranker GPU node (shared with KB). Multi-region active-active at P3+.  
**5W · Why**|  Why a separate layer?| Because per-module SDK adoption creates cost untraceability, PII drift, and failover ratchets. One gateway eliminates all three.  
**1H · How**|  How does it work?| Receive gRPC call → resolve tenant policy from memory (cached) → look up cache → on miss, redact PII → inject persona-version system prompt → call primary provider with 2 retries → on continued failure within 30 s, fail over to secondary → stream response back via SSE → store redacted prompt + response in cache → emit `ai.invocation` audit row → return.  
**2C · Cost**|  Cost?| P0 budget: ≤ $150 / month LLM (N(FR pending)). 50-tenant budget: ≤ $4 / active user / month LLM. Cache hit rate is the dominant lever.  
**2C · Constraints**|  Constraints?| (a) Zero cross-tenant cache. (b) PII recall ≥ 99% measured against a public VN+EN test set. (c) Provider must be on ZDR (Zero-Data-Retention) attested endpoint for sensitive routes. (d) Per-tenant monthly USD cap hard-enforced.  
**5M · Materials**|  Stack?| Python 3.13 · LiteLLM (vendored) · grpc-py · Presidio · regex-based VN PII rules · Redis (cache) · DuckDB (usage roll-up) · OpenTelemetry · self-hosted BGE-M3 embedder + BGE-rerank on a shared L4 GPU.  
**5M · Methods**|  Method choices?| Streaming-first (SSE end-to-end). Circuit breaker per provider × model. Hash-keyed cache (SHA-256 of canonical prompt + model + parameters). Idempotency-Key header for replay safety. Per-route latency budgets (read ≤ 800 ms, write ≤ 2 s, ingest ≤ 5 s).  
**5M · Machines**|  Deployment?| Fargate (CPU). One GPU pod for BGE-M3 + reranker (L4 24GB, shared with KB ingest).  
**5M · Manpower**|  Who maintains?| 0.5 FTE shared CTO + CDO at P0. CDO assumes primary at P1+.  
**5M · Measurement**|  How measured?| N(FR pending) (AI request p95 ≤ 2 s), (FR pending) cache hit rate, (FR pending) PII recall, per-tenant cost dashboard, provider error-rate burndown.  
  
2.5

## Cost-of-everything gate — token economics, budgets, attribution

The naive AI-Gateway design treats cost as a side-effect: count tokens, send a bill. The CyberOS design treats cost as the _primary purpose_ : every call is a fiscal event, attributed to a tenant + persona + module, capped against a budget, and surfaced as an invoice line item the moment it lands. Token accounting is not a reporting feature; it is a runtime gate that refuses a call if it cannot afford to make it. 

### Per-tenant policy (the cost contract)
    
    
    tenant: acme-corp
    ai_policy:
      monthly_cap_usd: 500
      warn_threshold: 0.80         # ping CFO + tenant admin at 80%
      hard_stop: true              # at 100%, refuse new calls
      emergency_override:
        enabled: true
        requires: ["cfo_signoff", "audit_row"]
      primary_provider: bedrock
      fallback_providers: [anthropic, openai]
      require_zdr: true
      residency: sg-1
      per_model_caps:
        bedrock:anthropic.claude-3.5-sonnet:
          max_tokens_per_call: 8000
          daily_call_cap: 5000
        bedrock:anthropic.claude-3-haiku:
          max_tokens_per_call: 4000
          daily_call_cap: 20000
      per_persona_attribution:
        cuo-cpo@0.4.1: { module: cuo, cost_centre: ops }
        cuo-cdo@0.3.0: { module: kb, cost_centre: ops }
        genie-public@1.2.0: { module: portal, cost_centre: cogs }

### Pre + post-call accounting (the 7-step sequence)

sequenceDiagram autonumber participant C as Caller (CUO/KB/CHAT) participant G as AI Gateway participant L as Cost ledger (Postgres) participant P as LLM provider participant B as 🧠 memory audit participant I as 🧾 INV (P2+) C->>G: chat.complete (model, messages, tenant, persona) G->>L: pre-check: estimate cost from token count alt over-budget L-->>G: refuse · 402_PAYMENT_REQUIRED G->>B: ai.invocation_refused (reason=budget_cap) G-->>C: 402 error · suggest model downshift else within budget L-->>G: allow · debit hold G->>P: forward request P-->>G: response · usage{ prompt_tokens, completion_tokens } G->>L: post-check: reconcile hold → actual; release excess G->>B: ai.invocation (chained · persona · cost · cache_state) G->>I: roll up to monthly cost-centre invoice (P2+) G-->>C: response end 

The two checks (pre-estimate + post-reconcile) together prevent both under-budgeting (call goes through, surprise bill) and over-debiting (call refused, hold never released). Holds expire after 60 s if no post-check arrives — defensive.

### Attribution dimensions — who owes for this call?

Dimension| Source| What it answers  
---|---|---  
`tenant_id`| JWT claim (P0 · slice 2+) or X-Tenant header (P0 · slice 1)| Which tenant's budget does this debit?  
`agent_persona`| JWT claim per AUTH §2.7| Which agent identity made this call? (cuo-cpo@0.4.1 vs genie@1.0.2)  
`module`| X-CyberOS-Module header (set by caller)| Which module is the cost centre? (kb / chat / proj / …)  
`cost_centre`| Policy mapping per persona| P&L line: ops · cogs · r&d · g&a · marketing  
`route_class`| chat · embed · rerank · image| Which provider price book applies?  
`cache_state`| Cache lookup result| hit (free) vs miss (paid); aggregate cache savings.  
`failover_path`| primary / fallback / refused| Was the call served by the cheap primary or the secondary at premium?  
  
Every `ai.invocation` audit row carries all seven dimensions. Monthly cost reports slice across any combination — "how much did genie-public cost us in May vs april?", "what's our cache savings rate by tenant?", "which module is closest to its budget cap right now?".

2.6

## Provider abstraction — swap providers without touching consumers

The naive approach to multi-provider is "wrapper SDK per provider." That works until your tenant in EU-1 needs Vertex (residency) and your tenant in SG-1 needs Bedrock (latency). AI Gateway is built on the LiteLLM router because it solves this once: provider auth, retries, streaming semantics, and pricing are model-aliased; consumers never know which provider answered. 

### Model-alias resolution table

Caller-side alias| Resolves to (default)| Fallback| Used by  
---|---|---|---  
`chat.smart`| `bedrock:claude-3.5-sonnet`| `anthropic:claude-sonnet-4.5`| CUO ambiguous tail, KB Q&A, CHAT @lumi  
`chat.fast`| `bedrock:claude-3-haiku`| `anthropic:claude-haiku-4.5`| PROJ inline genie, CRM next-action, OBS triage  
`chat.reason`| `openai:o1-mini`| `anthropic:claude-opus-4.6`| CUO CPO skill (spec auditing), CLO contract review  
`embed.standard`| `self-hosted:bge-m3`| `openai:text-embedding-3-large`| KB ingest, memory semantic search, CHAT context window  
`rerank.standard`| `self-hosted:bge-rerank-v2-m3`| `cohere:rerank-multilingual-v3`| KB Q&A, memory retrieval  
`image.standard`| `openai:gpt-image-1` (P2+)| `local:stable-diffusion-xl`| Marketing skill, design suggestions (P2+)  
  
Per-tenant policy can override any default. Consumers always call by alias; the resolution is opaque.

### Failover semantics — what happens when a provider degrades

Failure mode| Detection window| Action| Audit row emitted  
---|---|---|---  
Single 5xx / timeout| per call| Retry once with backoff (250 ms · 750 ms)| `ai.retry` sub-row  
Consecutive 5xx (≥ 3 in 10 s)| per-tenant per-provider| Mark primary as degraded; route next calls to fallback| `ai.failover_triggered`  
Rate-limit (429)| per provider| Backoff per retry-after header; if fallback available, switch immediately| `ai.rate_limited`  
Provider down > 60 s| circuit breaker open| All routes degrade to fallback; primary probed every 30 s| `ai.circuit_open`  
Provider recovery (3 consecutive 2xx)| circuit half-open → closed| Resume primary routing; emit recovery row| `ai.circuit_closed`  
Both primary + fallback down| multi-failover| Refuse new calls; serve cache only for read routes; alert CTO + page on-call| `ai.degraded_mode`  
Per-tenant SLA breach (failover > 30 s)| tenant-aware| Tenant credit policy applies (P3+); audit row generates SLA credit suggestion| `ai.sla_breach`  
  
### Geographic residency — provider × region matrix

Tenant residency| Allowed primary providers| Allowed fallback| Why  
---|---|---|---  
`sg-1` (Singapore)| Bedrock AP-SE-1 · Anthropic · OpenAI| any of the above| PDPL Art. 38 cross-border requires explicit consent; AP-SE-1 keeps APAC data in-region.  
`eu-1` (Frankfurt)| Bedrock EU-CENTRAL · Vertex EU · Anthropic EU| EU-only providers| GDPR + EU AI Act + EU Data Boundary requires no US transit.  
`us-1`| Bedrock US-EAST · Anthropic · OpenAI · Vertex US| any| Default — no residency pinning required.  
`vn-1` (Vietnam)| Bedrock AP-SE-1 + PDPL DPA · Anthropic with VN DPA| AP-SE-1 only| PDPL Art. 38: cross-border requires bilateral DPA; SG-1 is the closest compliant route.  
  
Residency is pinned in the tenant policy (and in the AUTH JWT at P0 · slice 2+); the router rejects any model alias that resolves to a non-compliant provider for the calling tenant.

2.7

## Compliance plane — PII, persona stamp, ZDR, audit chain

The compliance posture of every AI-using module is determined by what AI Gateway does before bytes leave the cluster. Four protections are stacked in order: **PII redaction → persona-version stamp → ZDR check → audit row emission**. Any link in the chain that fails refuses the call entirely; there is no degraded-compliance mode. 

### The four-link compliance chain

Link| Responsibility| Recall / strictness target| Failure behaviour  
---|---|---|---  
**1\. PII redaction**|  Presidio (EN-base) + custom VN-PII recogniser (CCCD, MST, phone, NĐD, địa chỉ)| Recall ≥ 99% · precision ≥ 95% on CI test set| Below recall threshold → CI gate fails; release blocked. Runtime PII detection failure → call refused with `403_PII_REDACTION_FAILED`.  
**2\. Persona-version stamp**|  System-prompt injection from memory `memories/personas/<version>.md`| 100% of calls carry a persona stamp; no consumer can override| Missing stamp → call refused; the consumer used the LLM SDK directly (forbidden).  
**3\. ZDR check**|  Provider DPA + ZDR contract validation against tenant policy `require_zdr: true`| 100% of calls routed only to ZDR-compliant providers when tenant requires it| Non-ZDR provider attempted → call refused; CTO + DPO alerted; no fallback to non-ZDR.  
**4\. Audit row emission**|  memory `ai.invocation` chained row · 14 fields (see schema below)| 100% emission; chain hash verifiable| memory write fails → call NOT made; better to refuse than make a call we cannot audit.  
  
### The `ai.invocation` audit row schema
    
    
    {
      "seq": 18420,
      "ts_ns": 1763112131_000_000_000,
      "op": "put",
      "path": "meta/ai-invocations/2026-05-15_acme_chat-smart_a3c7.md",
      "extra": {
        "tenant_id": "org:acme-corp",
        "agent_persona": "cuo-cpo@0.4.1",
        "module": "cuo",
        "route_class": "chat",
        "model_alias": "chat.smart",
        "resolved_provider": "bedrock:claude-3.5-sonnet",
        "region": "ap-se-1",
        "failover_path": "primary",
        "cache_state": "miss",
        "prompt_hash": "sha256:a3c7…2b9f",
        "response_hash": "sha256:9f8e…1d2c",
        "usage": { "prompt_tokens": 1240, "completion_tokens": 612, "usd_cost": 0.012 },
        "redaction_applied": true,
        "redaction_count": 3,
        "zdr_confirmed": true,
        "duration_ms": 1843,
        "idempotency_key": "01HZK…"
      },
      "prev_chain": "69e6…488a",
      "chain": "8d2c…5a91"
    }

The row does not contain the prompt or response text — only hashes — to keep memory size bounded. The hashes prove integrity; the actual content is in the (encrypted, short-lived) cache row that the hash addresses.

### Vietnamese PII recogniser — what extends Presidio

VN-PII type| Pattern| Examples (redacted to type)  
---|---|---  
**CCCD** (citizen ID)| 12-digit number, optional formatting| `0790…1234` → `<CCCD>`  
**MST** (tax code)| 10-digit company + optional 3-digit branch| `0123456789-001` → `<MST>`  
**VN phone**|  +84/0 + 9/10 digits; mobile prefixes| `0906878091` → `<VN_PHONE>`  
**NĐD** (legal representative)| Full Vietnamese name + tonal marks heuristic| `Trịnh Thái Anh` → `<VN_PERSON>`  
**VN address**|  quận / phường / đường + number patterns| `207A Nguyễn Văn Thủ, Tân Định, Q1` → `<VN_ADDRESS>`  
**Bank account (VN)**|  VN banks' BIN patterns (Vietcombank/Techcombank/MB/…)| `9704… (Vietcombank)` → `<VN_BANK_ACCT>`  
  
The recogniser ships as a Presidio plugin (Python) with a 200-sample CI test set (50 positive per type, 100 negatives). Recall ≥ 99% gate blocks release; precision ≥ 95% prevents false-positive over-redaction (e.g. don't redact common Vietnamese surnames out of normal prose).

3

## Architecture

Six internal pipelines between the gRPC ingress and the provider egress: tenant policy resolution, persona stamping, PII redaction, cache lookup, router selection, and accounting. The diagram below shows the full request path for a `chat.complete` call. 

graph TB subgraph CLIENTS ["Callers"] CUO["🎯 CUO router"] SKILL["🛠 Skill host"] CHAT["💬 CHAT summarise"] KB["📚 KB ingest"] GENIE["✨ Genie Q&A"] end subgraph AI ["AI Gateway (gRPC + Rust edge proxy)"] ING["edge_proxy.rs  
mTLS + auth"] TPR["tenant_policy.py  
provider-pref · cap · ZDR-attestation"] PER["persona.py  
inject system-prompt by persona_version"] RED["redactor.py  
Presidio + VN rules · ≥ 99% recall"] CACHE["cache.py  
Redis · key=SHA-256(canonical_prompt + model + params + tenant)"] ROUT["router.py (LiteLLM)  
provider-pref · circuit-breaker · 2 retries"] STR["stream.py  
SSE multiplexer"] ACC["accountant.py  
tokens · USD · per-tenant cap"] EMB["embedder  
BGE-M3 (self-hosted)"] RER["reranker  
BGE-rerank-v2-m3 (self-hosted)"] end subgraph PROVIDERS ["External providers (ZDR-attested)"] BED["AWS Bedrock  
Anthropic Sonnet · Haiku"] ANTH["Anthropic API  
Sonnet · Opus"] OAI["OpenAI API  
gpt-4o · o1"] VTX["Vertex AI (P1+)  
Gemini · PaLM"] end subgraph STORES REDIS[("Redis 7  
cache · TTL configurable")] DUCK[("DuckDB  
usage roll-up · hourly")] end subgraph SINKS memory["🧠 memory  
ai.invocation rows"] OBS["👁 OBS  
traces + cost dashboard"] end CUO --> ING SKILL --> ING CHAT --> ING KB --> ING GENIE --> ING ING --> TPR TPR --> PER PER --> RED RED --> CACHE CACHE -->|miss| ROUT CACHE -->|hit| STR ROUT --> BED ROUT --> ANTH ROUT --> OAI ROUT -.P1+.-> VTX ROUT --> STR STR --> ACC ACC --> DUCK CACHE --> REDIS ROUT --> EMB ROUT --> RER ING --> memory ING --> OBS ACC --> memory classDef planned fill:#fef6e0,stroke:#9c750a classDef provider fill:#cba88a,stroke:#4338ca classDef store fill:#f5f3ff,stroke:#7c3aed classDef sink fill:#f5ede6,stroke:#45210e class ING,TPR,PER,RED,CACHE,ROUT,STR,ACC,EMB,RER planned class BED,ANTH,OAI,VTX provider class REDIS,DUCK store class memory,OBS sink 

### Internal components

Component| Path (planned)| Responsibility  
---|---|---  
`edge_proxy.rs`| services/ai-gateway/edge/| Rust mTLS proxy. Verifies caller JWT (AUTH), unwraps tenant_id, forwards to Python core over Unix socket.  
`tenant_policy.py`| services/ai-gateway/core/policy.py| Resolves per-tenant provider preference, cost cap, ZDR-attestation requirement. Caches from memory reads.  
`persona.py`| services/ai-gateway/core/persona.py| Injects persona-version system prompt at the gateway ((FR pending)). Reads `meta/persona///prompt.md` from memory.  
`redactor.py`| services/ai-gateway/core/redactor.py| Presidio + custom VN rule pack. Detects CCCD, MST, bank acct, address, phone, email, name. Replaces with token sentinels; un-redact on response for trusted classes.  
`cache.py`| services/ai-gateway/core/cache.py| Hash-keyed response cache. Key = SHA-256(canonical(prompt) || model || params || tenant_id). NEVER cross-tenant — tenant_id in key is the load-bearing fact.  
`router.py`| services/ai-gateway/core/router.py| LiteLLM-derived. Selects provider by tenant policy + route class. 2 retries with exponential backoff. Circuit-breaker on per-provider error rate. Fails over to secondary within 30 s.  
`stream.py`| services/ai-gateway/core/stream.py| SSE multiplexer. Forwards provider stream chunks as `data: …` events. Backpressure-aware. Handles cancellation mid-stream.  
`accountant.py`| services/ai-gateway/core/accountant.py| Token + USD accounting per tenant. Emits hourly roll-up to DuckDB; per-tenant cap enforcement (80% warning, 100% hard stop). (FR pending).  
`circuit_breaker.py`| services/ai-gateway/core/circuit_breaker.py| Per-provider × per-model breaker. Opens on error rate > 10% / 60 s window; half-opens after 30 s; closes on first success.  
`idempotency.py`| services/ai-gateway/core/idempotency.py| Replay-safe via `Idempotency-Key` header. Same key → same response (returned from short-TTL Redis cache).  
`vn_pii_rules.py`| services/ai-gateway/core/vn_pii_rules.py| Custom VN PII detectors: CCCD (Decree 13 — 12 digits), MST (10/13 digits, validator), VietQR account, Vietnamese full names (rule-based).  
`embedder_client.py`| services/ai-gateway/core/embedder_client.py| Calls the BGE-M3 GPU pod over gRPC. Batches up to 32 requests per call.  
`reranker_client.py`| services/ai-gateway/core/reranker_client.py| Calls the BGE-rerank-v2-m3 GPU pod. Returns ordered score list.  
`audit_bridge.py`| services/ai-gateway/core/audit_bridge.py| Emits one `ai.invocation` row per call: actor · route · model · tokens-in · tokens-out · USD · cache_state · persona_version · redaction_applied. (FR pending).  
`cost_export.py`| services/ai-gateway/tools/cost_export.py| Generates per-tenant monthly invoice line items from DuckDB roll-up.  
  
4

## Data model

AI Gateway is mostly stateless — its source of truth is memory (provider config, tenant policy, persona prompts) and Redis (cache, idempotency keys). DuckDB holds the cost-roll-up for fast dashboard queries. The entities below show the read + write surfaces. 

erDiagram TENANT ||--o{ TENANT_POLICY: "has policy" TENANT ||--o{ AI_INVOCATION: "incurs" TENANT ||--o{ TENANT_BUDGET: "has cap" PROVIDER ||--o{ PROVIDER_MODEL: "offers" PROVIDER_MODEL ||--o{ AI_INVOCATION: "fulfils" PERSONA ||--o{ PERSONA_VERSION: "has versions" PERSONA_VERSION ||--o{ AI_INVOCATION: "stamps" CACHED_RESPONSE ||--o| AI_INVOCATION: "may serve" REDACTION_RULE ||--o{ REDACTION_HIT: "matches" AI_INVOCATION ||--o{ REDACTION_HIT: "produces" TENANT { uuid id PK string slug string country } TENANT_POLICY { uuid tenant_id FK string primary_provider "bedrock or anthropic or openai" string fallback_provider bool require_zdr_attestation obj per_route_overrides "route_class to provider" } TENANT_BUDGET { uuid tenant_id FK int monthly_usd_cents int spent_usd_cents_mtd timestamp window_start bool warning_sent_80pct } PROVIDER { string id PK "bedrock or anthropic or openai or vertex" string region bool zdr_attested string status "active or degraded or disabled" } PROVIDER_MODEL { string id PK "bedrock-anthropic-claude-3-5-sonnet" string provider FK string model_name int context_window decimal price_input_per_1k decimal price_output_per_1k bool streaming_supported } PERSONA { string id PK "cuo or genie or hr-assistant or other" string display_name } PERSONA_VERSION { string persona_id FK string version "v2.3.1" string system_prompt timestamp valid_from timestamp valid_to } CACHED_RESPONSE { string key PK "SHA-256 of prompt model params tenant" bytes response_body int tokens_in int tokens_out timestamp created_at timestamp expires_at string tenant_id "load-bearing - no cross-tenant" } AI_INVOCATION { uuid id PK uuid tenant_id FK string actor "subject_id" string route_class "chat or embed or rerank or image" string model_id FK string persona_version int tokens_in int tokens_out decimal usd_cost int latency_ms string cache_state "miss or hit or bypass" bool redaction_applied string fallover_path "primary or secondary or primary-to-secondary" string memory_chain timestamp ts } REDACTION_RULE { string code PK "vn-cccd or vn-mst or en-email or other" string regex string sentinel "CCCD-sentinel" string locale } REDACTION_HIT { uuid id PK uuid invocation_id FK string rule_code FK int offset int length } 

### Provider + model matrix (P0)

Provider| Model| Route class| ZDR| Pricing /1k tokens  
---|---|---|---|---  
AWS Bedrock| anthropic.claude-3.5-sonnet| chat (default)| ✓| $0.003 in / $0.015 out  
AWS Bedrock| anthropic.claude-3-haiku| chat (cheap)| ✓| $0.00025 in / $0.00125 out  
Anthropic API| claude-sonnet-4.5| chat (high-quality)| ✓ (zero-retention)| $0.003 in / $0.015 out  
OpenAI| gpt-4o| chat (alt)| ✓ (zero-data-retention)| $0.0025 in / $0.01 out  
OpenAI| o1-mini| reasoning (alt)| ✓| $0.003 in / $0.012 out  
Self-hosted| BGE-M3| embed| n/a (in-cluster)| free (amortised GPU)  
Self-hosted| BGE-rerank-v2-m3| rerank| n/a (in-cluster)| free (amortised GPU)  
Vertex AI (P1+)| gemini-2.5-pro| chat (alt)| ✓| $0.00125 in / $0.005 out  
  
5

## API surface

AI Gateway speaks gRPC internally and exposes a thin REST surface for non-internal callers. A federated GraphQL subgraph publishes the read-side (usage, model catalogue). No public MCP tools — the gateway is infrastructure, not directly agent-callable. 

### gRPC API (canonical)
    
    
    syntax = "proto3";
    package cyberos.ai.v1;
    
    service AIGateway {
     / Streaming chat completion (SSE end-to-end).
     rpc ChatComplete(ChatRequest) returns (stream ChatChunk);/ Non-streaming variant for batch jobs.
     rpc ChatCompleteSync(ChatRequest) returns (ChatResponse);/ Embedding (single or batch).
     rpc Embed(EmbedRequest) returns (EmbedResponse);/ Reranking.
     rpc Rerank(RerankRequest) returns (RerankResponse);/ Cost lookup for the calling tenant.
     rpc UsageMTD(TenantRef) returns (UsageReport);/ Model catalogue.
     rpc ListModels(Empty) returns (ModelList);
    }
    
    message ChatRequest {
     repeated Message messages = 1;
     string persona = 2;/ "cuo" | "genie" | "hr-assistant"
     string route_class = 3;/ "chat" | "reasoning"
     string idempotency_key = 4;
     ModelHint hint = 5;/ optional; tenant policy may override
     bool stream = 6;
     map<string, string> metadata = 7;/ free-form, recorded in audit
    }
    
    message Message {
     string role = 1;/ "user" | "assistant" | "system" (system reserved)
     string content = 2;
     repeated Tool tool_calls = 3;
    }
    
    message ChatChunk {
     string content_delta = 1;
     bool done = 2;
     Usage usage = 3;/ emitted on final chunk
    }
    
    message Usage {
     int32 tokens_in = 1;
     int32 tokens_out = 2;
     string model_id = 3;
     string cache_state = 4;
     double usd_cost = 5;
     bool redaction_applied = 6;
     string persona_version = 7;
     string memory_chain = 8;
    }
    
    message EmbedRequest {
     repeated string inputs = 1;
     string model = 2;/ default: "bge-m3"
    }
    
    message EmbedResponse {
     repeated Embedding embeddings = 1;
     Usage usage = 2;
    }
    
    message Embedding { repeated float vector = 1; }

### REST + SSE surface (planned, edge-only)

Method| Path| Purpose  
---|---|---  
POST| `/v1/chat/completions`| OpenAI-compatible chat endpoint (SSE supported with `stream: true`).  
POST| `/v1/embeddings`| OpenAI-compatible embeddings endpoint.  
POST| `/v1/rerank`| Cohere-style rerank endpoint.  
GET| `/v1/models`| List models available to caller's tenant.  
GET| `/v1/usage`| MTD usage report for the caller's tenant.  
GET| `/health`| Liveness + per-provider circuit-breaker state.  
GET| `/metrics`| Prometheus scrape endpoint.  
  
### GraphQL subgraph (read-only)
    
    
    extend schema
     @link(url: "https://specs.apollo.dev/federation/v2.5", import: ["@key", "@requiresScopes"])
    
    type AIInvocation @key(fields: "id") @requiresScopes(scopes: [["ai.usage_read"]]) {
     id: ID!
     tenantId: ID!
     actor: String!
     routeClass: RouteClass!
     modelId: String!
     personaVersion: String!
     tokensIn: Int!
     tokensOut: Int!
     usdCost: Float!
     latencyMs: Int!
     cacheState: CacheState!
     redactionApplied: Boolean!
     failoverPath: String!
     ts: DateTime!
    }
    
    type UsageReport @key(fields: "tenantId month") {
     tenantId: ID!
     month: String! # "2026-05"
     totalCalls: Int!
     totalTokens: Int!
     totalUsdCost: Float!
     capUsdCost: Float!
     percentUsed: Float!
     byModel: [ModelUsage!]!
    }
    
    type ModelUsage {
     modelId: String!
     calls: Int!
     tokensIn: Int!
     tokensOut: Int!
     usdCost: Float!
    }
    
    enum RouteClass { CHAT REASONING EMBED RERANK IMAGE }
    enum CacheState { MISS HIT BYPASS }
    
    type Query {
     aiUsageMTD(tenantId: ID): UsageReport!
     aiInvocations(since: DateTime, limit: Int = 50): [AIInvocation!]!
     @requiresScopes(scopes: [["ai.usage_read"]])
     aiModels: [Model!]!
    }

6

## Key flows

### Flow 1 — Streaming chat completion (cache miss)

sequenceDiagram autonumber participant CUO as CUO router participant AI as AI Gateway participant TP as tenant_policy participant PER as persona injector participant RED as redactor participant CACHE as Redis cache participant ROUT as router participant BED as AWS Bedrock participant STR as SSE stream participant ACC as accountant participant B as 🧠 memory CUO->>AI: ChatComplete(messages, persona="cuo", stream=true) AI->>TP: get_policy(tenant_id) TP-->>AI: {primary:"bedrock", fallback:"anthropic", cap_usd_mtd:150} AI->>PER: inject system-prompt(persona="cuo", version="v2.3.1") PER-->>AI: messages + system AI->>RED: redact(messages) RED-->>AI: messages' + redaction_hits AI->>CACHE: GET sha256(canonical(messages')||model||params||tenant) CACHE-->>AI: miss AI->>ROUT: route(messages', model="claude-3.5-sonnet") ROUT->>BED: invoke streaming completion loop SSE chunks BED-->>ROUT: data: {delta:"..."} ROUT-->>STR: forward STR-->>CUO: data: {delta:"..."} end BED-->>ROUT: done {tokens_in:120, tokens_out:450} ROUT->>ACC: account(tokens, usd=0.0075) ACC->>ACC: check tenant cap (97/150 → OK) AI->>CACHE: SET key TTL=24h AI->>B: ai.invocation row {…} STR-->>CUO: done event with Usage 

Cache-miss latency budget: ≤ 2 s p95 (NFR-AI-001). Provider latency dominates; gateway overhead is < 50 ms typical.

### Flow 2 — Cache hit (deterministic replay)

sequenceDiagram autonumber participant CHAT as CHAT summarise participant AI as AI Gateway participant TP as tenant_policy participant RED as redactor participant CACHE as Redis cache participant STR as SSE stream participant ACC as accountant participant B as 🧠 memory CHAT->>AI: ChatCompleteSync(messages, persona="genie") AI->>TP: policy(tenant) AI->>RED: redact(messages) AI->>CACHE: GET key CACHE-->>AI: HIT {response, tokens, cached_at} AI->>ACC: account(cache=hit, tokens=0) ACC->>ACC: usage incremented; no USD charged AI->>B: ai.invocation row {cache_state:"hit"} AI-->>CHAT: response (≤ 50 ms p95) 

Cache-hit budget: ≤ 50 ms p95. The audit row still records the call — cache hits are tracked separately for invoicing transparency.

### Flow 3 — Provider failover (primary degraded)

sequenceDiagram autonumber participant K as KB ingest participant AI as AI Gateway participant ROUT as router participant CB as circuit_breaker participant BED as AWS Bedrock (primary) participant ANTH as Anthropic API (fallback) participant B as 🧠 memory K->>AI: Embed(inputs, model="bge-m3") Note over AI,BED: bge-m3 is self-hosted; example uses chat AI->>ROUT: chat call, primary=bedrock ROUT->>BED: invoke BED-->>ROUT: 503 Service Unavailable ROUT->>ROUT: retry #1 (backoff 250 ms) ROUT->>BED: invoke BED-->>ROUT: 503 ROUT->>ROUT: retry #2 (backoff 1 s) ROUT->>BED: invoke BED-->>ROUT: 503 ROUT->>CB: record failure → trip breaker for bedrock:claude-3.5 Note over CB: error_rate above 10%/60s → OPEN ROUT->>ANTH: failover invoke ANTH-->>ROUT: response ROUT-->>AI: response with failover_path="primary→secondary" AI->>B: ai.invocation row {failover_path:"primary→secondary"} AI-->>K: response Note over CB: 30 s later HALF_OPEN; first success closes breaker 

FR-AI-008 / FR-AI-009: failover within 30 s of primary failure (NFR-AI-015). The circuit breaker prevents pile-up against a degraded provider.

### Flow 4 — Per-tenant cost cap enforcement

sequenceDiagram autonumber participant U as Module participant AI as AI Gateway participant ACC as accountant participant TPR as tenant_policy participant ALERT as CHAT alert bot participant B as 🧠 memory U->>AI: ChatComplete(…) AI->>ACC: pre-check cap alt spent_mtd < 80% cap ACC-->>AI: allow AI->>AI: …normal flow… else 80% ≤ spent_mtd < 100% ACC-->>AI: allow + warning flag AI->>ALERT: post "tenant X at 84% AI cap" ALERT->>B: budget_warning row AI->>AI: …normal flow… else spent_mtd ≥ 100% ACC-->>AI: hard-stop AI->>B: ai.invocation row {decision:"blocked_cap"} AI-->>U: 429 Quota Exceeded end 

(FR pending): 80% warning, 100% hard stop. Warnings post to the tenant's `#cyberos-alerts` CHAT channel; hard-stop returns 429 with a structured `Retry-After` header pointing at the next billing cycle.

### Flow 5 — PII redaction (Vietnamese CCCD)

sequenceDiagram autonumber participant U as HR module participant AI as AI Gateway participant RED as redactor participant VPI as vn_pii_rules participant ROUT as router participant BED as Bedrock participant B as 🧠 memory U->>AI: ChatComplete("Verify CCCD 037201234567 for Le Van A") AI->>RED: redact(messages) RED->>VPI: scan for VN PII VPI-->>RED: hits [{rule:"vn.cccd", offset:13, len:12}, {rule:"vn.name", offset:30, len:8}] RED->>RED: replace with sentinels - Verify CCCD CCCD_0 for NAME_0 RED-->>AI: redacted messages AI->>ROUT: send to provider (no real CCCD leaves cluster) ROUT->>BED: invoke BED-->>ROUT: response (refers to {{CCCD_0}}, {{NAME_0}}) ROUT-->>AI: response AI->>RED: un-redact in response (caller's tenant scope only) AI->>B: ai.invocation {redaction_applied:true, hits:2} AI-->>U: response with un-redacted CCCD/name 

(FR pending): PII recall ≥ 99% on the VN + EN test set. Sentinels are caller-scoped — the same prompt from a different tenant produces different sentinels, so a cached row from tenant A never round-trips through tenant B.

7

## Request lifecycle

A single AI invocation traverses ten states between caller and audit row. Most of the time is in `Routing` (provider RTT); cache-hit paths skip from `CacheCheck` straight to `Streaming`. 

stateDiagram-v2 [*] --> Received: gRPC ingress Received --> AuthValidated: AUTH verifies JWT, resolves tenant AuthValidated --> PolicyResolved: tenant_policy + budget check PolicyResolved --> Blocked: cap exceeded 100 percent PolicyResolved --> PersonaInjected: system prompt prepended PersonaInjected --> Redacted: PII scrubbed Redacted --> CacheCheck: hash key built CacheCheck --> Streaming: HIT (cached response) CacheCheck --> Routing: MISS Routing --> Streaming: provider produced chunk Routing --> Failover: primary errored twice and 30s elapsed Failover --> Streaming: secondary producing chunks Streaming --> Accounted: tokens + USD totalled Accounted --> Audited: ai.invocation row to memory Audited --> [*] Blocked --> [*]: 429 returned, audit row written 

### Latency budget per route class

Route class| p95 target| p99 target| Source NFR  
---|---|---|---  
**chat (default)**|  ≤ 2 s| ≤ 5 s| N(FR pending)  
chat (cache hit)| ≤ 50 ms| ≤ 200 ms| internal  
chat streaming TTFB| ≤ 500 ms| ≤ 1.2 s| internal · PERF-002  
reasoning (o1, claude-opus)| ≤ 10 s| ≤ 30 s| internal  
embed (BGE-M3)| ≤ 120 ms / item| ≤ 250 ms| internal  
rerank (BGE-rerank)| ≤ 80 ms / pair| ≤ 150 ms| internal  
  
8

## Functional Requirements

The CyberOS FR catalogue is being rebuilt one feature at a time via the open [feature-request-author](<https://github.com/cyberskill/cyberos/tree/main/modules/skill/feature-request-author>) Agent Skill.

Previous FR enumerations were archived 2026-05-14 and are no longer reflected on this page. Specific FRs land here as they are re-authored.

9

## Non-Functional Requirements

Performance NFRs from + reliability from §11.2.2. Cross-referenced at [nfr-catalog.html#ai](<../../reference/nfr-catalog.html#ai>).

NFR ID| Concern| Target| Measurement  
---|---|---|---  
`NFR-AI-001`| AI request p95 latency (chat, miss)| ≤ 2 s| k6 load test against gateway  
`NFR-AI-002`| Streaming TTFB p95| ≤ 500 ms| k6 streaming test (FR-AI-010)  
`NFR-AI-003`| Cache-hit response p95| ≤ 50 ms| internal bench  
`NFR-AI-004`| Embed call p95 (per item)| ≤ 120 ms| BGE-M3 GPU bench (FR-AI-019)  
`NFR-AI-005`| Rerank call p95 (per pair)| ≤ 80 ms| BGE-rerank bench (FR-AI-020)  
`NFR-AI-006`| Cache hit rate at P0| ≥ 30%| weekly roll-up (FR-AI-017)  
`NFR-AI-007`| Cache hit rate at P2+| ≥ 60%| weekly roll-up (FR-AI-017)  
`NFR-AI-008`| Cost ceiling (internal P0)| ≤ $150/month LLM| DuckDB invoice export (FR-AI-007)  
`NFR-AI-009`| Cost ceiling (50-tenant)| ≤ $4/active user/month LLM| per-tenant dashboard  
`NFR-AI-010`| PII redaction recall (VN+EN)| ≥ 99%| VN-PII golden test set (FR-AI-013)  
`NFR-AI-011`| PII redaction precision| ≥ 95%| false-positive rate measured (FR-AI-012)  
`NFR-AI-012`| AI gateway provider failover| continuous on primary outage| chaos test (FR-AI-008/009)  
`NFR-AI-013`| Gateway availability (28-day)| ≥ 99.9%| SLO monitor (FR-OBS-003)  
`NFR-AI-014`| Cross-tenant cache leakage| = 0| property-based test in CI (FR-AI-018)  
`NFR-AI-015`| Failover detection latency| ≤ 30 s| chaos test on circuit breaker (FR-AI-009)  
  
10

## Dependencies

AI Gateway depends on three internal services and four external providers (P0). It is depended on by every CyberOS module that calls an LLM. 

graph LR subgraph upstream ["AI Gateway depends on"] AUTH["🔐 AUTH  
tenant resolution"] memory["🧠 memory  
tenant_policy + persona prompts  
\+ ai.invocation rows"] OBS["👁 OBS  
traces + metrics"] REDIS["⚡ Redis  
cache + idempotency"] BED["AWS Bedrock"] ANTH["Anthropic API"] OAI["OpenAI API"] GPU["BGE-M3 GPU pod"] end AI["🧠 AI Gateway"] subgraph downstream ["Used by"] CUO["🎯 CUO"] SKILL["🛠 Skill host"] CHAT["💬 CHAT"] KB["📚 KB"] GENIE["✨ Genie"] PROJ["📋 PROJ"] EMAIL["✉️ EMAIL"] OTH["…all LLM-using modules"] end AUTH --> AI memory --> AI OBS --> AI REDIS --> AI BED --> AI ANTH --> AI OAI --> AI GPU --> AI AI --> CUO AI --> SKILL AI --> CHAT AI --> KB AI --> GENIE AI --> PROJ AI --> EMAIL AI --> OTH classDef planned fill:#fef6e0,stroke:#9c750a classDef shipped fill:#f5ede6,stroke:#45210e classDef ext fill:#cba88a,stroke:#4338ca class AI,AUTH,OBS,CUO,SKILL,CHAT,KB,GENIE,PROJ,EMAIL,OTH planned class memory,REDIS shipped class BED,ANTH,OAI,GPU ext 

11

## Compliance scope

AI Gateway is the chokepoint for "what did the AI see, and at what cost?" — making it the regulator's first call for AI Act + PDPL questions.

Regulation / standard| Article / clause| AI Gateway feature  
---|---|---  
EU AI Act (Reg. 2024/1689)| Art. 12 — Logging| One `ai.invocation` row per call; full input + redaction + output hash.  
EU AI Act| Art. 13 — Transparency| Per-call model id + persona version surfaced to caller's audit trail.  
EU AI Act| Art. 14 — Human oversight| Destructive tool calls require human-confirm — gateway annotation routed via MCP.  
EU AI Act| Art. 15 — Accuracy, robustness, cybersecurity| Circuit breaker + failover + PII redaction.  
EU AI Act| Art. 26 — Deployer obligations| Persona-version stamping pins the deployed agent version per call.  
Vietnam PDPL (Law 91/2025)| Art. 4 — Lawful processing| PII redaction before extra-tenant transfer; per-tenant data-residency in policy.  
Vietnam Decree 13/2023| Art. 16 — Cross-border transfer| ZDR-attested providers; per-tenant policy can pin EU-only / VN-only.  
GDPR| Art. 25 — Data protection by design| Redaction is on-by-default; bypass requires explicit per-route tenant policy.  
GDPR| Art. 28 — Processor obligations| ZDR contracts on file with Anthropic, OpenAI, AWS Bedrock.  
ISO/IEC 42001 (AIMS)| § 8.3 — AI system lifecycle| Persona-version stamping + provider catalogue + cost tracking close the loop.  
OWASP Gen AI Top-10| LLM01: Prompt injection| System prompt injected at gateway, not in caller-controlled message text.  
OWASP Gen AI Top-10| LLM06: Sensitive info disclosure| PII redaction recall ≥ 99% — measured per release.  
OWASP Gen AI Top-10| LLM10: Model theft| Self-hosted BGE models behind mTLS; no external embedding API used.  
SOC 2 Type II| CC7.2 — Monitoring| Per-tenant cost + latency + cache-hit dashboards.  
  
12

## Risk entries

AI Gateway-specific risks tracked in the [risk register](<../../reference/risk-register.html#ai>).

ID| Risk| Likelihood| Impact| Owner| Mitigation  
---|---|---|---|---|---  
`R-AI-001`| Cross-tenant cache leakage| Low| Catastrophic| CTO| Tenant_id baked into cache key; property-based CI test verifies no cross-tenant hits.  
`R-AI-002`| PII recall regression below 99%| Medium| High| CDO| Test-set CI gate; release blocked if recall < 99%; quarterly red-team adds adversarial samples.  
`R-AI-003`| Tenant cost overrun (cap-bypass bug)| Low| High| CFO| Pre-call check; post-call check; daily cost-reconciliation against provider bill.  
`R-AI-004`| Primary provider extended outage (> 4 h)| Medium| Medium| CTO| 30s failover + per-tenant fallback override; multi-provider posture documented in DR runbook.  
`R-AI-005`| Persona prompt drift between gateway + module| Medium| Medium| CDO| Single source of truth in memory; gateway-only injection ((FR pending)); CI test on each persona-version change.  
`R-AI-006`| Prompt-injection bypasses gateway redaction| Medium| High| CSO| CaMeL-style enforcement; sentinel scheme cannot be guessed by upstream caller; red-team quarterly.  
`R-AI-007`| Provider rate-limit cascade (one tenant starves the rest)| Medium| Medium| CTO| Per-tenant token-bucket on gateway; global circuit-breaker prevents pile-up.  
`R-AI-008`| Cache poisoning via adversarial canonical prompt| Low| High| CSO| Cache key includes tenant_id + idempotency-key; provider response hash compared to in-flight verification on critical routes.  
`R-AI-009`| BGE-M3 GPU pod single point of failure| Medium| Medium| CTO| 2-replica deployment at P1+; CPU fallback (slow) on hot-path embed.  
`R-AI-010`| Vendor SDK CVE blocks release| Medium| Low| CTO| LiteLLM is vendored — patch in-tree; Renovate watches upstream weekly.  
`R-AI-011`| **P0 · slice 1 build sequence slips → AUTH lands first → tenant cost overruns invisible until invoice arrives**|  Medium| Critical| CEO| Hard P0 ship order locked in AUDIT_AND_PLAN_2026_05_14 §3.3; the P0 · slice 1 milestone is the cost-gate go-live, not "AI Gateway feature-complete"; pre-AUTH AI Gateway uses X-Tenant header signed with a static HMAC.  
`R-AI-012`| Persona prompt cache poisoning (Lumi push corrupts a persona used by all tenants)| Low| Critical| CSO| Persona-version pinning per tenant; tenant must opt-in to a new version; Lumi-pushed updates land as a candidate version, not active.  
`R-AI-013`| Provider DPA cancellation (Anthropic / OpenAI changes ZDR terms mid-quarter)| Low| High| CLO| Legal monitor on provider DPAs; tenant policy `require_zdr` rejects non-compliant providers; emergency model-alias re-resolution within 1 hour.  
`R-AI-014`| Cost ledger hold leak (60 s expiry never fires → false debits accumulate)| Medium| Medium| CTO| Hold rows have hard TTL via Postgres scheduled job; nightly reconciliation against provider bill; spot-test alarms on cumulative drift > 5%.  
`R-AI-015`| Streaming SSE buffer leak (long-running stream pins memory)| Medium| Medium| CTO| Per-stream max duration (5 min) + max bytes (16 MB); hard-close on exceed; backpressure surfaced to caller via SSE event.  
`R-AI-016`| Embedding model upgrade (BGE-M3 → next-gen) breaks memory semantic search retrieval| Medium| Medium| CDO| Embedding model version is part of the MEMORY_LINK record; recall regression CI test on every model swap; allow per-tenant pinning.  
`R-AI-017`| Image-generation route at P2+ floods budget (image calls are 100× chat cost)| Medium| High| CFO| Image route default-off; per-tenant explicit opt-in; per-image cap; watermark for EU AI Act Art. 50; cache by prompt hash with extended TTL.  
`R-AI-018`| Geographic residency violation (EU tenant routed via US provider during failover)| Low| Critical| DPO| Residency pinning at router level; failover providers filtered by tenant residency; refuse + alert if no compliant fallback available.  
`R-AI-019`| VN-PII recogniser regression on production (new VN spelling/format misses CCCD)| Medium| High| CDO| 200-sample CI test set; quarterly red-team adds adversarial VN samples; release blocked on recall < 99% for VN-PII subset.  
`R-AI-020`| Self-hosted BGE GPU pod OOM under sustained load → embed becomes the bottleneck| Medium| Medium| CTO| 2-replica L4 deployment at P1+; CPU fallback for hot-path embed (slow but available); load test 5× expected QPS quarterly.  
  
13

## KPIs

9 KPIs covering latency, cost, redaction quality, and reliability.

KPI| Formula| Source| Target  
---|---|---|---  
**Chat p95 latency (miss)**|  histogram| OBS · Prometheus| ≤ 2 s (N(FR pending))  
**Streaming TTFB p95**|  histogram| OBS| ≤ 500 ms  
**Cache hit rate**| `cache_hits / total_calls`| DuckDB roll-up| ≥ 30% (P0)  
**PII redaction recall**| `TP / (TP + FN)` on test set| CI gate| ≥ 99%  
**PII redaction precision**| `TP / (TP + FP)`| CI gate| ≥ 95%  
**Provider failover events**|  count / 28 d| `ai.invocation`| tracked; alert on > 100/day  
**Tenant cost overrun events**|  count / 28 d| accountant| = 0 (hard-stop ensures)  
**Cross-tenant cache leakage**|  property-test count| CI| = 0  
**USD spent vs. budget (MTD)**| `spent / cap` per tenant| dashboard| < 100% (warn at 80%)  
**Per-persona cost share**|  cost grouped by `agent_persona`| memory audit replay| tracked per persona; alert on > 50% concentration to single persona  
**Cache savings rate**| (cache_hits × estimated_cost) / total_billed_cost| cost ledger| ≥ 0.15 (15% savings) by P1 exit  
**Hold-to-actual drift**|  SUM(held - actual) / SUM(actual)| cost ledger nightly reconcile| ≤ 0.05 (≤ 5% drift); alarm on > 0.05  
**Residency-violation refusal rate**|  refusals_due_to_residency / total_attempts| `ai.invocation_refused` audit rows| tracked; spike = misconfigured tenant policy  
**Persona stamp coverage**|  calls_with_stamp / total_calls| audit rows| = 1.0 (hard floor — anything below means a consumer bypassed gateway)  
**ZDR-compliant routing rate**| (calls_to_ZDR_provider for ZDR-required-tenant) / total such calls| audit rows| = 1.0 (hard floor)  
**VN-PII recall (production sample)**|  TP / (TP+FN) on weekly random sample| red-team review| ≥ 0.99  
**Provider-failover MTTR p95**|  histogram on failover events| OBS| ≤ 30 s  
**Dogfooding LLM cost / Member (internal)**|  internal-tenant cost / DAU| cost ledger filtered to `tenant_id=org:cyberskill`| ≤ $10 / DAU / month at P0; ≤ $5 by P1  
  
14

## RACI matrix

Activity| CEO| CTO| CDO| CFO| CSO| DPO  
---|---|---|---|---|---|---  
Service design| A| R| C| I| C| I  
Implementation| I| A| R| I| I| I  
Provider contracts (ZDR, DPA)| C| C| I| A| R| C  
Cost tracking + invoicing| I| C| I| A/R| I| I  
PII rule maintenance (VN+EN)| I| C| A/R| I| C| C  
Persona-prompt curation| A| C| R| I| I| I  
Provider failover drill| I| A/R| C| I| C| I  
Compliance review (AI Act, PDPL)| I| C| C| I| C| A/R  
  
15

## Planned CLI surface

Two CLIs: `cyberos-ai` for operators (tenant policy, cost reports, model catalogue) and the standard OpenAI-compatible curl path for ad-hoc testing.

### 1\. Quick chat call
    
    
    $ curl https://ai.cyberos.com/v1/chat/completions \
     -H "Authorization: Bearer $CYBEROS_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"model":"claude-3.5-sonnet","messages":[{"role":"user","content":"summarise Q1 OKRs"}],"stream":false}'
    
    {
     "id": "ai_01HZJ8…XK",
     "model": "bedrock:anthropic.claude-3.5-sonnet",
     "persona_version": "genie-v1.0.2",
     "choices": [{"message":{"role":"assistant","content":"Q1 OKRs are …"}}],
     "usage": {"prompt_tokens":120,"completion_tokens":450,"usd_cost":0.0075,
     "cache_state":"miss","redaction_applied":false,"failover_path":"primary"}
    }

### 2\. Operator — view MTD usage
    
    
    $ cyberos-ai usage mtd --tenant stephen-personal
    
    Tenant: stephen-personal
    Month: 2026-05
    ─────────────────────────────
    Total calls: 14,823
    Tokens-in: 8.2 M
    Tokens-out: 3.1 M
    USD spent: $97.42 / $150.00 (64.9%)
    
    By model:
     bedrock:claude-3.5-sonnet 11,420 calls $74.20 (76%)
     bedrock:claude-3-haiku 3,210 calls $4.80 (5%)
     bge-m3 (embed) 193 batches free
    
    By cache state:
     hit 4,612 (31.1%) ← above target
     miss 10,211 (68.9%)

### 3\. Operator — update tenant policy
    
    
    $ cyberos-ai policy set --tenant acme-corp \
     --primary bedrock --fallback anthropic \
     --require-zdr true --cap-usd-monthly 500
    
    [policy updated]
     tenant: acme-corp
     primary: bedrock
     fallback: anthropic
     zdr: required
     cap: $500/month
    [audit] memory seq=14841

### 4\. Operator — list models
    
    
    $ cyberos-ai models list
    
    ID ROUTE ZDR PRICE (in/out per 1k)
    bedrock:anthropic.claude-3.5-sonnet chat ✓ $0.003 / $0.015
    bedrock:anthropic.claude-3-haiku chat ✓ $0.00025 / $0.00125
    anthropic:claude-sonnet-4.5 chat ✓ $0.003 / $0.015
    openai:gpt-4o chat ✓ $0.0025 / $0.01
    openai:o1-mini reason ✓ $0.003 / $0.012
    self-hosted:bge-m3 embed — free
    self-hosted:bge-rerank-v2-m3 rerank — free

### 5\. Operator — failover drill
    
    
    $ cyberos-ai chaos failover --provider bedrock --duration 60s
    
    [chaos] injected 100% error rate on bedrock for 60 s
    [detect] primary failure recognised @ +6.2 s
    [failover] secondary (anthropic) active @ +6.4 s
    [recovery] bedrock errors cleared @ +60 s
    [breaker] half-open @ +90 s; closed @ +91 s
    [result] (FR pending) PASSED (failover ≤ 30 s)

### 6\. Operator — export monthly invoice
    
    
    $ cyberos-ai invoice export --tenant acme-corp --month 2026-05 --output invoice.csv
    
    [invoice] tenant=acme-corp month=2026-05 rows=14,823 written invoice.csv (1.2 MB)
    [lines] by_model · by_route · by_persona · by_date

16

## Phase status & estimates

Status

Planned

P0 · design phase · P0 · slice 1

Est. LoC (Python + Rust)

~6,500

Python core + Rust edge proxy

Planned tests

90+

unit + integration + chaos

P0 monthly LLM budget

$150

N(FR pending)

Cache TTL (default)

24 h

per-tenant override

CLI commands

~15 planned

`cyberos-ai`

Capability| Status  
---|---  
LiteLLM-derived router (Bedrock + Anthropic + OpenAI)| planned · P0  
Streaming SSE end-to-end| planned · P0  
PII redaction (Presidio + VN rules)| planned · P0  
Persona-version system-prompt injection| planned · P0  
Response cache (Redis, tenant-keyed)| planned · P0  
Per-tenant cost cap + warning| planned · P0  
Circuit breaker + 30 s failover| planned · P0  
`ai.invocation` audit row per call| planned · P0  
Self-hosted BGE-M3 embedder| planned · P1  
Self-hosted BGE-rerank-v2-m3| planned · P1  
Vertex AI (Gemini) provider| planned · P1+  
Image generation route (DALL-E / Stable Diffusion)| planned · P2+  
Multi-region active-active| planned · P3+  
  
17

## References

  * **Bigger picture (§0 above):** 3 strategic roles + cross-module dependency Mermaid + auto-vs-human matrix.
  * **Cost-of-everything gate (§2.5 above):** per-tenant policy YAML + 7-step pre/post accounting sequence + 7-dimension attribution table.
  * **Provider abstraction (§2.6 above):** 6-row model-alias resolution + 7-row failover semantics + residency × provider matrix.
  * **Compliance plane (§2.7 above):** 4-link chain (PII → persona → ZDR → audit) + `ai.invocation` 14-field schema + VN-PII recogniser table.
  * **Cross-module page links:** [cuo.html](<../cuo/index.html>) · [memory.html](<../memory/index.html>) · [auth.html](<../auth/index.html>) · [kb.html](<../kb/index.html>) · [chat.html](<../chat/index.html>) · [proj.html](<../proj/index.html>) · [obs.html](<../obs/index.html>) · [ten.html](<../ten/index.html>)
  * **Build-readiness audit:** `archive/2026-05-14/AUDIT_AND_PLAN.md` (archived; see `cyberos/CHANGELOG.md`) — AI Gateway placed at P0 · slice 1 (P0 #1), the cost-of-everything gate before AUTH.
  * **Research review:** `archive/2026-05-14/RESEARCH_REVIEW.md` (archived; see `cyberos/CHANGELOG.md`) — explicit "Reorder AI Gateway before AUTH" recommendation; AI Gateway flagged as the highest-leverage cost-control module.
  * **memory auto-sync vision:** [MEMORY_AUTOSYNC_DESIGN.md §7](<../../docs/MEMORY_AUTOSYNC_DESIGN.md>) — every `ai.invocation` audit row is a memory capture event; Lumi cross-tenant synthesis depends on aggregate cost + persona signals at P3+.
  * **FR authoring discipline:** [modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md](<https://github.com/cyberskill/cyberos/blob/main/modules/skill/feature-request-audit/AUTHORING_DISCIPLINE.md>) — AI Gateway FRs land via the `feature-request-author` Agent Skill; "(FR pending)" markers are intentional placeholders.
  * **EU AI Act** (Reg. 2024/1689) — Art. 12 (logging via audit chain), Art. 13 (transparency — persona stamp), Art. 14 (human oversight — CFO cost-override), Art. 15 (accuracy + robustness — failover SLA), Art. 26 (operator obligations), Art. 50 (transparency — watermark for image gen at P2+).
  * **OWASP Gen AI Top-10 (2025):** LLM01 prompt injection · LLM06 sensitive disclosure · LLM08 supply-chain · LLM10 model theft — gateway-level mitigations.
  * **ISO/IEC 42001 (AIMS)** — § 8.3 lifecycle and persona-version stamping.
  * **Vietnam PDPL (Law 91/2025):** Art. 14 DSAR, Art. 20 security obligations, Art. 38 cross-border transfer rules (residency × provider matrix).
  * **LiteLLM upstream** — base router we vendor and extend.
  * **Microsoft Presidio** — PII detection library; VN-PII recogniser ships as a Presidio plugin.
  * **BGE-M3 / BGE-rerank-v2-m3** — self-hosted embedding + rerank models; BAAI release.
  * **Architecture context:** [infrastructure.html#ai](<../../architecture/infrastructure.html#ai>).



[← Previous: AUTH](<../auth/index.html>) [Next module: MCP Gateway →](<../mcp/index.html>)
