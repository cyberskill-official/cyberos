---
title: CUO - Module
source: website/docs/modules/cuo/module.html
migrated: FR-DOCS-002
---

# CyberOS CUO module - canonical persona catalog

Version 2.0.0. Status: normative for the CUO module. Companion files: `README.md` (operational quickstart), `docs/AGENTS.md` (protocol normativity), `docs/SPEC.md` (contract summary), `docs/ROUTING.md` (persona -> workflow -> skill-chain selection), `docs/CHANGELOG.md` (shipping record).

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

This document is the source of truth for **which C-personas exist in this module, how they map to C-Suite Reference §5, when each is needed (stage matrix from §3), and which workflows they orchestrate**. Every persona folder on disk MUST correspond to a row in §4; every row in §4 MUST correspond to a folder on disk OR be marked `planned` in §4.

The CUO (Chief Universal Officer) is the universal persona. All other personas are folders under `cuo/<c-level>/`. Each persona's folder contains a 9-block-schema `README.md` (per C-Suite Reference §4) plus a `workflows/` subfolder. Each workflow is a markdown file that declares a chain of SKILL module skills the persona orchestrates.

## §0 Design rules

§0.1 **Flat persona layout.** Every persona lives at `cuo/<persona-slug>/`. There is no `personas/` subfolder and no functional-group subfolder. Acronym collisions are resolved by suffixing the meaning: `cuo/chief-revenue-officer/` vs `cuo/chief-risk-officer/` vs `cuo/chief-restructuring-officer/`.

§0.2 **Disambiguation is mandatory.** Per C-Suite Reference §2, "every persona spec must include the full disambiguated title plus a one-sentence scope statement to prevent prompt-confusion in AI agents." Folder slugs follow `<acronym>-<meaning>` for the 7 colliding acronyms (CRO, CCO, CDO, CSO, CPO, CAO, CIO, CLO). Unambiguous acronyms use the bare acronym (`cuo/chief-executive-officer/`, `cuo/chief-technology-officer/`, `cuo/chief-financial-officer/`, etc.).

§0.3 **Nine-block schema.** Every persona's `README.md` SHALL render the nine-block template from C-Suite Reference §4: (1) Identity & scope, (2) Information inputs, (3) Stakeholder inputs, (4) Resource inputs, (5) Outputs (strategic / operational / communication / team), (6) Cadence, (7) KPIs, (8) Audit criteria, (9) Tools & stack.

§0.4 **Workflows chain skills.** Each persona's `workflows/<workflow-name>.md` SHALL declare: workflow purpose, inputs, outputs, the chain of SKILL module skills it invokes (in order, with hand-offs), cadence, and audit hooks. Workflows are independently invocable. The CUO supervisor reads a workflow's chain and walks the skills in sequence (or in parallel where dependencies allow).

§0.5 **Stage-aware applicability.** Each persona's README SHALL state the funding-stage prevalence per C-Suite Reference §3 (Seed / Series A / Scale-up / Growth / Enterprise). This signals to client-engagement work which personas are appropriate baselines at the client's stage.

§0.6 **Universal failure-mode reminders.** Each persona's README SHALL include the six universal failure modes from C-Suite Reference §6 as applicable: playbook transplant, activity over outcomes, silo-ing, forecast drift without narrative, hero dependence, AI-washing.

§0.7 **Audit criteria.** Each persona's README SHALL declare quantitative gates (which 3 primary KPIs the persona's outputs must move) and the qualitative 5-dimension rubric (alignment / coherence / customer-grounding / risk-realism / communicability) per §6.

§0.8 **No persona invents new skills inline.** When a workflow needs a skill that doesn't exist in the SKILL module, the workflow file SHALL reference it as `planned:<skill-name>` and the gap SHALL appear in `cuo/docs/NEEDED_SKILLS.md`. New skills are built in the SKILL module (per `skill/_template/HOW_TO_USE.md`), not in the CUO module.

§0.9 **CUO is the universal persona, not a folder.** The CUO sits above all personas as the routing layer. It has no `cuo/cuo/` folder. Its protocol normativity lives in `docs/AGENTS.md`.

## §1 CUO routing model

§1.1 A natural-language request enters the CUO. The CUO:

1. **Identifies the persona.** Which C-role is best suited to drive this work? The router uses keyword matching, the funding-stage matrix (§3), the disambiguation matrix (§2), and the request's domain language.
2. **Selects a workflow.** Within that persona's `workflows/` folder, which workflow's purpose matches the request?
3. **Walks the chain.** The workflow declares a skill chain. The CUO invokes skills in declared order via the SKILL module's CLI (`cyberos-skill run <name>`), passing each step's output as the next step's input.
4. **Records the chain decision** in the memory audit chain per the memory module protocol (AGENTS.md §6, §11).

§1.2 When no persona scores above the confidence threshold, the CUO escalates to the operator with the top-3 candidate personas + their applicability rationales.

§1.3 When a persona is selected but no workflow within it matches, the CUO escalates to the operator with the top-3 candidate workflows.

§1.4 When a workflow references a `planned:<skill-name>` that doesn't exist yet, the CUO emits a `MISSING_SKILL_REQUEST` block and halts the chain at that step.

§1.5 Multi-persona collaboration (Phase 3) is supported via workflow `escalates_to:` and `consults:` declarations - a CTO workflow can escalate a security boundary to `cuo/chief-information-security-officer/` mid-chain.

## §2 Acronym disambiguation (per C-Suite Reference §2)

| Acronym | Possible meanings | Folder slugs |
|---|---|---|
| CRO | Chief Revenue / Risk / Restructuring | `cro-revenue` / `cro-risk` / `cro-restructuring` |
| CCO | Chief Commercial / Compliance / Customer / Communications | `cco-commercial` / `cco-compliance` / `cco-customer` / `cco-communications` |
| CDO | Chief Data / Digital / Diversity | `cdo-data` / `cdo-digital` / `cdo-diversity` |
| CSO | Chief Strategy / Security / Sustainability / Sales | `cso-strategy` / `cso-security` / `cso-sustainability` / `cso-sales` |
| CPO | Chief People / Product / Privacy / Procurement | `cpo-people` / `cpo-product` / `cpo-privacy` / `cpo-procurement` |
| CAO | Chief Administrative / Accounting | `cao-admin` / `cao-accounting` |
| CIO | Chief Information / Investment | `cio-information` / `cio-investment` |
| CLO | Chief Legal / Learning | `clo-legal` / `clo-learning` |

Note: CSO-Security and CISO frequently overlap - CSO-Security is the broader physical+info-sec role; CISO is the info-sec specialist. Both folders exist; the boundary is documented in each persona's §1 Identity-and-scope block.

## §3 Stage applicability (per C-Suite Reference §3)

Each persona's README §1 Identity-and-scope SHALL state the funding stages where the persona is `essential | common | optional | crisis-only | extinct`. The MODULE-level summary lives in this section; the per-persona detail lives in the README.

Stage rows: **Seed** (<10 people), **Series A** (10-50), **Scale-up** (50-200), **Growth** (200-500), **Enterprise** (500+).

For the full role-by-stage matrix, see C-Suite Reference §3 (rendered there as a 22-row x 5-column table). The CUO does NOT re-render that matrix here - single source of truth.

## §4 Persona catalog - status

| Persona slug | Disambiguated title | C-Suite Ref §5 section | Status | Workflows count |
|---|---|---|---|---|
| `_template/persona/` | (canonical scaffold) | meta | shipped | n/a |
| `_template/workflow/` | (canonical scaffold) | meta | shipped | n/a |
| `ceo` | Chief Executive Officer | §5.1 | **shipped (Session D 2026-05-17)** | 5 |
| `coo` | Chief Operating Officer | §5.1 | **shipped (Session F 2026-05-17)** | 4 |
| `cso-strategy` | Chief Strategy Officer | §5.1 | **shipped (Session J 2026-05-18)** | 4 |
| `cgo` | Chief Growth Officer | §5.1 | **shipped (Session L 2026-05-18)** | 4 |
| `cao-admin` | Chief Administrative Officer | §5.1 | **shipped (Session M 2026-05-18)** | 4 |
| `cfo` | Chief Financial Officer | §5.2 | **shipped (Sessions D + G 2026-05-17)** | 6 |
| `clo-legal` | Chief Legal Officer / General Counsel | §5.2 | **shipped (Session E 2026-05-17)** | 5 |
| `cro-revenue` | Chief Revenue Officer | §5.2 | **shipped (Session G 2026-05-17)** | 4 |
| `cao-accounting` | Chief Accounting Officer | §5.2 | **shipped (Session M 2026-05-18)** | 4 |
| `cto` | Chief Technology Officer | §5.3 | **shipped (canonical reference)** | 5 |
| `cio-information` | Chief Information Officer | §5.3 | **shipped (Session N 2026-05-18)** | 4 |
| `ciso` | Chief Information Security Officer | §5.3 | **shipped (Session F 2026-05-17)** | 4 |
| `cdo-data` | Chief Data Officer | §5.3 | **shipped (Session H 2026-05-18)** | 4 |
| `caio` | Chief AI Officer | §5.3 | **shipped (Session H 2026-05-18)** | 4 |
| `cpo-product` | Chief Product Officer | §5.3 | **shipped (Session H 2026-05-18)** | 4 |
| `chief-architect` | Chief Architect / Chief Software Architect | §5.3 | **shipped (Session N 2026-05-18)** | 4 |
| `cmo` | Chief Marketing Officer | §5.4 | **shipped (Session I 2026-05-18)** | 4 |
| `cco-commercial` | Chief Commercial Officer | §5.4 | **shipped (Session L 2026-05-18)** | 4 |
| `cco-customer` | Chief Customer Officer | §5.4 | **shipped (Session H 2026-05-18)** | 4 |
| `cxo` | Chief Experience Officer | §5.4 | **shipped (Session M 2026-05-18)** | 4 |
| `cso-sales` | Chief Sales Officer | §5.4 | **shipped (Session F 2026-05-17)** | 4 |
| `chief-brand-officer` | Chief Brand Officer | §5.4 | **shipped (Session J 2026-05-18)** | 4 |
| `cco-communications` | Chief Communications Officer | §5.4 | **shipped (Session I 2026-05-18)** | 4 |
| `chro` | Chief Human Resources Officer / Chief Human Transformation Officer | §5.5 | **shipped (Sessions F + G 2026-05-17)** | 5 |
| `cpo-people` | Chief People Officer (synonym of CHRO at some firms) | §5.5 | **shipped (Session N 2026-05-18, synonym pointer + 3 variant workflows)** | 4 |
| `cdo-diversity` | Chief Diversity Officer | §5.5 | **shipped (Session N 2026-05-18)** | 4 |
| `clo-learning` | Chief Learning Officer | §5.5 | **shipped (Session N 2026-05-18)** | 4 |
| `chief-happiness-officer` | Chief Happiness Officer | §5.5 | **shipped (Session M 2026-05-18)** | 4 |
| `cco-compliance` | Chief Compliance Officer | §5.6 | **shipped (Session J 2026-05-18)** | 4 |
| `cro-risk` | Chief Risk Officer | §5.6 | **shipped (Session I 2026-05-18)** | 4 |
| `cpo-privacy` | Chief Privacy Officer | §5.6 | **shipped (Session G 2026-05-17)** | 4 |
| `chief-trust-officer` | Chief Trust Officer | §5.6 | **shipped (Session J 2026-05-18)** | 4 |
| `chief-ethics-officer` | Chief Ethics Officer / Chief AI Ethics Officer | §5.6 | **shipped (Session I 2026-05-18)** | 4 |
| `cso-sustainability` | Chief Sustainability Officer | §5.7 | **shipped (Session K 2026-05-18)** | 4 |
| `chief-esg-officer` | Chief ESG Officer | §5.7 | **shipped (Session K 2026-05-18)** | 4 |
| `chief-digital-officer` | Chief Digital Officer | §5.7 | **shipped (Session K 2026-05-18)** | 4 |
| `chief-transformation-officer` | Chief Transformation Officer | §5.7 | **shipped (Session I 2026-05-18)** | 4 |
| `chief-innovation-officer` | Chief Innovation Officer | §5.7 | **shipped (Session J 2026-05-18)** | 4 |
| `chief-knowledge-officer` | Chief Knowledge Officer | §5.7 | **shipped (Session H 2026-05-18)** | 4 |
| `cpo-procurement` | Chief Procurement Officer | §5.7 | **shipped (Session K 2026-05-18)** | 4 |
| `cio-investment` | Chief Investment Officer | §5.7 | **shipped (Session L 2026-05-18)** | 4 |
| `chief-medical-officer` | Chief Medical Officer | §5.7 | **shipped (Session L 2026-05-18)** | 4 |
| `cro-restructuring` | Chief Restructuring Officer | §5.7 | **shipped (Session L 2026-05-18)** | 4 |
| `chief-automation-officer` | Chief Automation Officer | §5.7 | **shipped (Session K 2026-05-18)** | 4 |
| `chief-remote-officer` | Chief Remote Officer (mostly absorbed by CHRO post-2022) | §5.7 | **shipped (Session M 2026-05-18)** | 4 |
| `chief-metaverse-officer` | Chief Metaverse Officer (**EXTINCT** - cautionary-tale entry per §8) | §5.7 | planned | 0 |
| `cso-security` | Chief Security Officer (physical+info super-set of CISO) | §5.7 | **shipped (Session N 2026-05-18)** | 4 |
| `chief-of-staff` | Chief of Staff (operates at C-level despite not strictly being C) | §5.7 | **shipped (Session D 2026-05-17)** | 4 |

**Total: 48 persona folders on disk (47 C-Suite Reference §5 entries + Chief-of-Staff).** 47 personas have shipped first-coverage workflows after Session N (2026-05-18); only `chief-metaverse-officer` remains intentionally EXTINCT per C-Suite Reference §8 rule 4 (cautionary tale). Sessions A-N delivered: CUO rebuild + 21 SDP-original skill pairs + 78 net-new skill pairs across Tiers 1-7 = 104 author+audit pairs / 208 bundles / 108 contracts in the SKILL catalog, plus 194 workflows across 47/48 active personas in the CUO catalog (CTO 5 + CEO 5 + CFO 6 + chief-of-staff 4 + clo-legal 5 + chro 5 + 41 personas with 4 workflows each). Sessions I-N were all "no new skills needed" - 124 niche workflows shipped chaining through the stable 104-pair catalog. **First-coverage phase COMPLETE.** Next phase: depth additions to already-shipped personas OR the CUO v3.0.0 Python supervisor build OR another strategic priority - operator decision.

Per §0.8, when a persona's workflows reference a `planned:<skill-name>`, the skill is enumerated in `cuo/docs/NEEDED_SKILLS.md` and built in the SKILL module before the workflow is callable.

## §5 CyberSkill-specific persona priorities (per C-Suite Reference §7)

CyberSkill is at scale-up stage (~50 people). The personas to build first, in order:

- **Now (<=50 people):** `ceo`, `cto`, `chief-of-staff`, `cfo` (fractional), `clo-legal` (external).
- **Series A / 50-100:** promote `cto` formally; add `coo` (as Head of Delivery); add `cpo-people` / `chro`; add `cso-sales` / `cgo`; add `ciso` (vCISO).
- **100-200 (scale-up):** full `cfo`; elevate `coo`; full `chro`; `cro-revenue`; `cpo-privacy`.
- **200+ (enterprise):** full suite plus `cpo-product`, `cdo-data` / `caio`, `cco-customer`, `chief-knowledge-officer` (consulting-firm-specific high-ROI).

**Consulting-firm-specific (per §7):** `chief-knowledge-officer` (IP/asset codification - the moat for consulting), `chief-of-staff` (operating-system owner), `cro-revenue` with services-revenue fluency.

**Vietnam context:** `cpo-privacy` required for Decree 13/2023; `ciso` increasingly required; titles carry signaling weight with VN clients.

## §6 Commercial baselines (per C-Suite Reference §8)

When advising clients, CyberSkill anchors on these five rules (rendered as a normative reminder in every persona's README §8 Audit criteria block):

1. **Don't add a C-title to solve a process problem.**
2. **Match the title to the funding stage** (use the §3 matrix).
3. **Disambiguate every acronym in writing** (use §2).
4. **Beware hype-cycle titles** (Metaverse, even pure CAIO long-term).
5. **Audit outputs, not titles** (use the §6 framework).

Client-facing engagement work consumes personas from this catalog as commercial baselines. Each persona's README §1 Identity-and-scope MUST disambiguate the title (rule 3); each §8 Audit criteria block MUST cite the universal failure modes (rule 5).

## §7 Workflow design rules

§7.1 Each workflow is a single markdown file at `cuo/<persona-slug>/workflows/<workflow-slug>.md`.

§7.2 Workflow frontmatter SHALL declare:

```yaml
---
workflow_id: <persona-slug>/<workflow-slug>
workflow_version: 1.0.0
purpose: <one-line purpose>
cadence: <daily | weekly | monthly | quarterly | annual | on-demand | per-event>
inputs:
  - { name: <input-name>, source: <where it comes from>, format: <markdown / json / dashboard / verbal-brief> }
outputs:
  - { name: <output-name>, format: <artifact type - e.g. product-requirements-document@1 / statement-of-work@1 / runbook@1>, recipient: <persona or external> }
skill_chain:
  - step: 1
    skill: <skill-name from SKILL module>
    inputs_from: <prior-step output name OR workflow input>
    outputs_to: <next-step input name OR workflow output>
  - step: 2
    skill: <skill-name>
    ...
escalates_to:
  - { persona: cuo/<persona-slug>, when: <condition - e.g. "decision touches security boundary"> }
consults:
  - { persona: cuo/<persona-slug>, when: <condition> }
audit_hooks:
  - <e.g. "each step's output is logged to memory audit chain via memory module">
---
```

§7.3 The workflow body SHALL document: the operator-facing description, when to invoke, how to invoke (CUO routing trigger), expected duration, failure modes per step, and operator-side decisions (where the chain pauses for human input).

§7.4 Workflows that reference `planned:<skill-name>` are valid but non-callable until the skill ships. The `MISSING_SKILL_REQUEST` halt-block makes the dependency explicit.

§7.5 Workflows MAY reference workflows from OTHER personas via `delegates_to:` - e.g. a `ceo` workflow can delegate "draft the GTM plan" to `chief-revenue-officer/draft-gtm-plan.md`.

## §8 Lifecycle

| Phase | Trigger | Output |
|---|---|---|
| Draft | Operator copies `_template/persona/` -> `cuo/<slug>/` | Initial 9-block README |
| Workflow design | Operator copies `_template/workflow/` -> `cuo/<slug>/workflows/<wf>.md` | Workflow files declaring skill chains |
| Skill gap audit | CUO checks every workflow's `skill_chain` against the SKILL module catalog | `cuo/docs/NEEDED_SKILLS.md` updated |
| Ship | All workflows reference shipped skills + audit-loop passes | Persona is callable |
| Refine | Operator usage signals (high HITL rate, low confidence, persona-mismatch escalations) | Workflow revision + bumped `workflow_version` |
| Retire | Persona becomes EXTINCT or absorbed into another persona | Folder moved to `cuo/_retired/` after a 30-day soak |

## §9 Cross-references

- `../../modules/cuo/README.md` (project root) - the source document this catalog implements. §2 acronym matrix, §3 stage matrix, §4 persona template, §5 role profiles, §6 audit framework, §7 CyberSkill-specific, §8 commercial heuristics.
- `skill/` - the source of truth for the skills that workflows chain. `skill/MODULE.md` §3 lists all 46 currently-shipped bundles.
- `memory/` - the memory. Every CUO routing decision + workflow invocation lands in the audit chain per `memory/docs/AGENTS.md` §6.
- `cuo/docs/AGENTS.md` - protocol normativity (replaces the legacy CUO AGENTS.md after the v2.0.0 rebuild).
- `cuo/docs/SPEC.md` - contract summary.
- `cuo/docs/ROUTING.md` - persona -> workflow -> skill-chain selection algorithm.
- `cuo/docs/NEEDED_SKILLS.md` - punch list of skills that workflows reference but the SKILL module doesn't yet ship.

## §10 Migration record (2026-05-17 evening rebuild)

| Action | Reason |
|---|---|
| Wiped legacy `cuo/cuo/` Python implementation (Phase-1 rule-based router, 5 modules + tests + scripts + tools) | The CUO v2.0.0 is a markdown-driven persona/workflow catalog, not a Python runtime. Patterns preserved conceptually in the new SPEC.md (RoutingDecision shape, threshold-based confidence, ARG_EXTRACTORS dispatch, ARG normalization, VN-diacritic-aware scoring). |
| Preserved `cuo/docs/` directory | Lineage. Rewritten in v2.0.0, but the folder retains the historical changelog + legacy AGENTS.md for traceability. |
| Added `cuo/MODULE.md` (this file) | Canonical catalog. |
| Added `cuo/_template/{persona, workflow}/` | Copy-paste scaffolds. |
| Added `cuo/chief-technology-officer/` as the canonical reference persona | CyberSkill IS technical. CTO is the highest-traffic persona for an internal-eng-led consultancy. |
| Marked 46 other personas `planned` | Subsequent build sessions march through them per the FR-authoring loop discipline (one at 10/10 before the next). |

The wipe was destructive. Recovery is via git history. No memory entries were touched.

## §11 Cross-module heartbeat

- `skill/` v2.0.0 (2026-05-17 morning) - 22 author+audit pairs shipped under the flat layout. 17 missing contracts authored in the afternoon rebuild. Rust host wired for the flat layout.
- `cuo/` v2.0.0 (2026-05-17 evening, this rebuild) - persona-folder + workflow-file model. 1 canonical persona shipped; 46 planned.
- `memory/` - protocol unchanged in this rebuild; the CUO continues to honour AGENTS.md §6, §11.
- `skill/public/` - Vietnamese-market skills absorbed from the former `cyberos/public-skills/` per the SKILL `MODULE.md` §8 rewrite (2026-05-17 evening).

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
