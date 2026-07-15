# CyberOS as the company brain: recording, memory, and evaluation

## The goal

Make CyberOS the single place CyberSkill works, so that what people do on it becomes a durable record;
feed that record into the BRAIN (the MEMORY module) where Lumi (GENIE, the CUO) can analyse it; and use
it to evaluate and support people against the three signed documents - the labor contract, the NDA /
non-compete / IP agreement, and the total-rewards and career-path appendix. This note answers one
question: is the current architecture enough, and if not, what is the plan.

## Short answer

The foundations are the right ones and are already live. What is missing is not the hard infrastructure;
it is four specific layers on top: complete capture, an indexing pipeline into the brain, a rubric built
from the three documents, and an evaluation engine with a human in the loop - plus a governance layer that
has to come first for legal and trust reasons. None of that needs a re-architecture. It is additive.

## What already exists (the bones are good)

- A tamper-evident interaction log. The audit chain (`l1_audit_log`) is hash-chained, so a recorded event
  cannot be silently altered later. Auth and chat already emit events into it. That integrity property is
  exactly what you want if records will ever inform an evaluation.
- A brain that can store and recall fast. The MEMORY module is Postgres plus pgvector, which gives
  semantic search over embedded content, alongside normal relational queries and a relational graph for
  relationships. This is the persistent, quick-retrieval store you asked about.
- An analysis engine. Lumi (the CUO / GENIE) already runs on the AI gateway with model routing, spend
  caps, residency, and a guarded dream loop. It is the component that would read the brain and produce
  analysis.
- Identity and isolation. AUTH tells you who did what, per tenant, with row-level security, so records are
  attributable and tenant-scoped.
- Observability. The obs module already collects activity signals (metrics, traces), useful as one input.

## The gaps (what to build)

1. Complete and consistent capture. Today only some events are chained, and in the P0 deploy chat content
   is in the chat database but not mirrored into the brain (we left the audit link off for P0). To
   "record all interactions" you need one event schema and every module emitting to it: chat messages,
   module usage, task and project activity, document and IP activity, sign-ins and presence.
2. An ingestion pipeline into the brain. Raw events need to be embedded into pgvector, summarised into
   rolling summaries so long-term memory stays small and fast, and tiered (hot recent, warm summarised,
   cold archived). This is what turns a log into a brain you can query in milliseconds.
3. A rubric built from the three documents. The contracts define what "good" and "compliant" mean -
   duties and working terms (labor contract), confidentiality, non-compete and IP-assignment obligations
   (the NDA), and the KPIs, rewards, and career milestones (the appendix). These need to be turned into a
   structured, versioned checklist the system can evaluate against. Right now they are prose in three
   files.
4. An evaluation engine with a human in the loop. Lumi maps recorded evidence to rubric items and drafts
   an assessment per person on a cadence; a manager reviews and the employee can respond; and the
   assessment itself is written back to the audit chain. The engine assists a decision; it does not make
   the decision.

## The plan, in phases

Phase 0 - governance first (before capturing more). Write a short, plain monitoring-and-data notice: what
is recorded, why, who can see it, how long it is kept, and the employee's rights. Get the monitoring basis
into the employment documents or an addendum, and have people acknowledge it. This is both a legal
requirement and the thing that makes the team trust the system instead of resenting it. Details in the
governance section below.

Phase 1 - capture. Define one interaction-event schema (who, what, when, where, a content reference, a
type) and have every module emit it. Turn on the chat-to-brain audit link so chat activity chains into
MEMORY. Start with chat and sign-in events, then add task, project, and document events as those modules
come online.

Phase 2 - the brain (persistence and fast retrieval). Build the MEMORY ingestion worker: embed each event
into pgvector with an HNSW index for fast semantic recall, generate rolling per-person and per-channel
summaries so the long-term memory stays compact, and tier storage by age. Keep the hash chain as the
system of record and treat the vector index as a fast lens over it. This is the "persistent memory that
retrieves quickly" you described - a retrieval-augmented brain over your own data.

Phase 3 - the rubric. Parse the three documents into a structured, versioned framework: obligations
(confidentiality, non-compete, IP assignment), working terms, KPIs, and career milestones, each as a
checkable item with its source clause. Store it so an evaluation can cite the exact contract clause it
measured against.

Phase 4 - evaluation. On a cadence, Lumi retrieves the relevant evidence from the brain, maps it to rubric
items, and drafts an evidence-linked assessment. A manager reviews and edits; the employee sees it and can
respond; the final result is audit-chained. Tie it to the appendix so progression and rewards have a
defensible, evidence-based basis rather than a gut feel.

Phase 5 - surfacing. A manager and HR view for assessments, and an employee self-view of their own record
and assessment. Transparency here is a feature, not an afterthought.

## Persistent, fast-retrieval memory design

- System of record: the hash-chained `l1_audit_log` holds every interaction event, append-only and
  tamper-evident.
- Fast lens: pgvector with an HNSW index over embeddings of messages and summaries gives sub-second
  semantic recall ("what did X commit to about project Y", "show IP-related activity last quarter").
- Compaction: rolling summaries per person, per channel, and per time window keep long-term memory small,
  so retrieval stays fast as data grows - you query summaries first, then drill into raw events only when
  needed.
- Tiering: hot (recent raw events, fully indexed), warm (older events behind summaries), cold (archived,
  retrievable on demand). This bounds cost and latency.
- All of this lives in the MEMORY module on Supabase Postgres today; it scales to a dedicated database
  later without changing the model.

## Doing the monitoring responsibly (this protects the company too)

Recording work to manage performance and to enforce signed IP and confidentiality terms is a legitimate,
common, and lawful purpose. The way you do it decides whether it helps or backfires. I am not a lawyer, so
confirm specifics with Vietnamese counsel, but the shape that keeps you safe and trusted:

- Transparency. Tell employees what is recorded and why, in writing. Vietnam's Personal Data Protection
  Decree (13/2023/ND-CP) expects a lawful basis, notice, purpose limitation, and data-subject rights; the
  Labor Code (45/2019/QH14) governs the employment relationship the monitoring sits inside. Covert
  monitoring is the risky path; disclosed monitoring is the defensible one.
- Proportionality and minimization. Record work interactions on the platform - not private life, and not
  keystroke or screen surveillance. Collect what the purpose needs, no more.
- Access control. Only the relevant manager and HR, plus the employee for their own record, can see it.
  Tenant RLS already gives you the spine for this.
- Human in the loop. Lumi drafts and surfaces evidence; a person decides anything that affects pay,
  progression, or employment. Never let the model auto-decide those.
- Retention. Set and enforce limits per data type; do not keep everything forever.

Done this way, the same system that records also builds trust, which raises data quality because people
actually use it - and it gives you a defensible, evidence-based basis for rewards and progression under
the appendix.

## Mapping to what exists, and what is new

- Reuse: AUTH (identity, tenancy), the audit chain (capture and integrity), MEMORY plus pgvector (the
  brain and fast recall), CUO / GENIE (analysis), the AI gateway (models and policy), obs (signals).
- New work, as tasks to add to the catalog: a unified interaction-event schema and emitters
  (capture); the MEMORY ingestion and summarization worker (brain); the rubric model built from the three
  documents; the evaluation engine and its human-review workflow; the governance, consent, and retention
  layer; and the manager and employee views.

## Decisions for you

1. Scope of recording: platform interactions only (recommended), or wider. I recommend platform-only.
2. Transparency model: disclosed to employees (recommended and safer) versus covert (do not).
3. Evaluation autonomy: Lumi assists a human reviewer (recommended) versus auto-scoring. Keep a human in
   the loop for anything consequential.
4. Sequence: I recommend governance (Phase 0) and capture (Phase 1) first, because they are cheap, unblock
   everything, and are the responsible order; the rubric and evaluation engine follow once data is
   flowing.
